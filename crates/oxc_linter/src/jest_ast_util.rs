use std::borrow::Cow;

use oxc_ast::{
    ast::{CallExpression, Expression, IdentifierReference, MemberExpression},
    AstKind,
};
use oxc_semantic::AstNode;
use oxc_span::{Atom, Span};

use crate::context::LintContext;

pub fn parse_general_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext,
) -> Option<ParsedGeneralJestFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, node, ctx)?;

    if let ParsedJestFnCall::GeneralJestFnCall(jest_fn_call) = jest_fn_call {
        return Some(jest_fn_call);
    }
    None
}

pub fn parse_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext,
) -> Option<ParsedJestFnCall<'a>> {
    let callee = &call_expr.callee;

    // If bailed out, we're not jest function
    let resolved = resolve_to_jest_fn(call_expr, ctx)?;

    // We traverse nodes from `node.callee`, so this `parent_kind` always be CallExpression.
    let chain = get_node_chain(callee, Some(KnownMemberExprPropertyKind::CallExpression));
    let all_member_expr_except_last = chain.iter().rev().skip(1).all(|member| {
        matches!(member.parent_kind, Some(KnownMemberExprPropertyKind::MemberExpression))
    });

    // Check every link in the chain except the last is a member expression
    if !all_member_expr_except_last {
        return None;
    }

    // Ensure that we're at the "top" of the function call chain otherwise when
    // parsing e.g. x().y.z(), we'll incorrectly find & parse "x()" even though
    // the full chain is not a valid jest function call chain
    if ctx.nodes().parent_node(node.id()).is_some_and(|parent_node| {
        matches!(parent_node.kind(), AstKind::CallExpression(_) | AstKind::MemberExpression(_))
    }) {
        return None;
    }

    if let (Some(first), Some(last)) = (chain.first(), chain.last()) {
        // If we're an `each()`, ensure we're the outer CallExpression (i.e `.each()()`)
        if last.name == "each"
            && !matches!(
                callee,
                Expression::CallExpression(_) | Expression::TaggedTemplateExpression(_)
            )
        {
            return None;
        }

        if matches!(callee, Expression::TaggedTemplateExpression(_)) && last.name != "each" {
            return None;
        }

        let kind = JestFnKind::from(&first.name);
        let mut members = Vec::new();
        let mut iter = chain.into_iter();
        let first = iter.next().expect("first ident name");
        let rest = iter;

        // every member node must have a member expression as their parent
        // in order to be part of the call chain we're parsing
        for member in rest {
            members.push(member);
        }

        let name = resolved.local.as_str();
        let is_valid_jest_call = if members.is_empty() {
            VALID_JEST_FN_CALL_CHAINS.iter().any(|chain| chain[0] == name)
        } else if members.len() == 1 {
            VALID_JEST_FN_CALL_CHAINS_2
                .iter()
                .any(|chain| chain[0] == name && chain[1] == members[0].name)
        } else if members.len() == 2 {
            VALID_JEST_FN_CALL_CHAINS_3.iter().any(|chain| {
                chain[0] == name && chain[1] == members[0].name && chain[2] == members[1].name
            })
        } else if members.len() == 3 {
            VALID_JEST_FN_CALL_CHAINS_4.iter().any(|chain| {
                chain[0] == name
                    && chain[1] == members[0].name
                    && chain[2] == members[1].name
                    && chain[3] == members[2].name
            })
        } else {
            false
        };

        if !is_valid_jest_call {
            return None;
        }
        return Some(ParsedJestFnCall::GeneralJestFnCall(ParsedGeneralJestFnCall {
            kind,
            members,
            raw: first.name,
        }));
    }

    None
}

fn resolve_to_jest_fn<'a>(
    call_expr: &'a CallExpression,
    ctx: &'a LintContext,
) -> Option<ResolvedJestFn<'a>> {
    let ident = resolve_first_ident(&call_expr.callee)?;

    if ctx.semantic().is_reference_to_global_variable(ident) {
        return Some(ResolvedJestFn { local: &ident.name });
    }

    None
}

fn resolve_first_ident<'a>(expr: &'a Expression) -> Option<&'a IdentifierReference> {
    match expr {
        Expression::Identifier(ident) => Some(ident),
        Expression::MemberExpression(member_expr) => resolve_first_ident(member_expr.object()),
        Expression::CallExpression(call_expr) => resolve_first_ident(&call_expr.callee),
        Expression::TaggedTemplateExpression(tagged_expr) => resolve_first_ident(&tagged_expr.tag),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub enum JestFnKind {
    Expect,
    General(JestGeneralFnKind),
    Unknown,
}

impl JestFnKind {
    pub fn from(name: &str) -> Self {
        match name {
            "expect" => Self::Expect,
            "jest" => Self::General(JestGeneralFnKind::Jest),
            "describe" | "fdescribe" | "xdescribe" => Self::General(JestGeneralFnKind::Describe),
            "fit" | "it" | "test" | "xit" | "xtest" => Self::General(JestGeneralFnKind::Test),
            "beforeAll" | "beforeEach" | "afterAll" | "afterEach" => {
                Self::General(JestGeneralFnKind::Hook)
            }
            _ => Self::Unknown,
        }
    }

    pub fn to_general(self) -> Option<JestGeneralFnKind> {
        match self {
            Self::General(kind) => Some(kind),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum JestGeneralFnKind {
    Hook,
    Describe,
    Test,
    Jest,
}

pub enum ParsedJestFnCall<'a> {
    GeneralJestFnCall(ParsedGeneralJestFnCall<'a>),
    #[allow(unused)]
    ExpectFnCall(ParsedExpectFnCall<'a>),
}

pub struct ParsedGeneralJestFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub raw: Cow<'a, str>,
}

pub struct ParsedExpectFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub raw: Cow<'a, str>,
    // pub args: Vec<&'a Expression<'a>>
    // TODO: add `modifiers`, `matcher` for this struct.
}

struct ResolvedJestFn<'a> {
    pub local: &'a Atom,
}

pub struct KnownMemberExpressionProperty<'a> {
    pub name: Cow<'a, str>,
    pub kind: KnownMemberExprPropertyKind,
    pub parent_kind: Option<KnownMemberExprPropertyKind>,
    pub span: Span,
}

pub enum KnownMemberExprPropertyKind {
    CallExpression,
    Identifier,
    MemberExpression,
    StringLiteral,
    TaggedTemplateExpression,
}

impl KnownMemberExprPropertyKind {
    pub fn from(expr: &Expression) -> Option<Self> {
        match expr {
            Expression::MemberExpression(_) => Some(Self::MemberExpression),
            Expression::Identifier(_) => Some(Self::Identifier),
            // We make sure TemplateLiteral are static when call it, so we can treat it as StringLiteral.
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                Some(Self::StringLiteral)
            }
            Expression::CallExpression(_) => Some(Self::CallExpression),
            Expression::TaggedTemplateExpression(_) => Some(Self::TaggedTemplateExpression),
            // The result of `get_node_chain` only contains nodes above, mark others as None.
            _ => None,
        }
    }

    pub fn from_member_expr(member_expr: &MemberExpression) -> Option<Self> {
        match member_expr {
            MemberExpression::StaticMemberExpression(_) => Some(Self::Identifier),
            MemberExpression::ComputedMemberExpression(expr) => Self::from(&expr.expression),
            // Jest test cases will never be there, just return None
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }
}

/// Port from [eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest/blob/a058f22f94774eeea7980ea2d1f24c6808bf3e2c/src/rules/utils/parseJestFnCall.ts#L36-L51)
fn get_node_chain<'a>(
    expr: &'a Expression<'a>,
    parent_kind: Option<KnownMemberExprPropertyKind>,
) -> Vec<KnownMemberExpressionProperty<'a>> {
    let mut chain = Vec::new();

    match expr {
        Expression::MemberExpression(member_expr) => {
            chain.extend(get_node_chain(
                member_expr.object(),
                KnownMemberExprPropertyKind::from(expr),
            ));

            if let Some((span, name)) = member_expr.static_property_info() {
                if let Some(kind) = KnownMemberExprPropertyKind::from_member_expr(member_expr) {
                    let parent_kind = KnownMemberExprPropertyKind::from(expr);
                    chain.push(KnownMemberExpressionProperty {
                        name: Cow::Borrowed(name),
                        kind,
                        parent_kind,
                        span,
                    });
                }
            }
        }
        Expression::Identifier(ident) => {
            chain.push(KnownMemberExpressionProperty {
                name: Cow::Borrowed(ident.name.as_str()),
                kind: KnownMemberExprPropertyKind::Identifier,
                parent_kind,
                span: ident.span,
            });
        }
        Expression::CallExpression(call_expr) => {
            let sub_chain = get_node_chain(
                &call_expr.callee,
                Some(KnownMemberExprPropertyKind::CallExpression),
            );
            chain.extend(sub_chain);
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            let sub_chain = get_node_chain(
                &tagged_expr.tag,
                Some(KnownMemberExprPropertyKind::TaggedTemplateExpression),
            );
            chain.extend(sub_chain);
        }
        Expression::StringLiteral(string_literal) => {
            chain.push(KnownMemberExpressionProperty {
                name: Cow::Borrowed(string_literal.value.as_str()),
                kind: KnownMemberExprPropertyKind::StringLiteral,
                parent_kind,
                span: string_literal.span,
            });
        }
        Expression::TemplateLiteral(template_literal) => {
            if template_literal.expressions.is_empty() && template_literal.quasis.len() == 1 {
                chain.push(KnownMemberExpressionProperty {
                    name: Cow::Borrowed(
                        template_literal.quasi().expect("get string content").as_str(),
                    ),
                    kind: KnownMemberExprPropertyKind::StringLiteral,
                    parent_kind,
                    span: template_literal.span,
                });
            }
        }
        _ => {}
    };

    chain
}

const VALID_JEST_FN_CALL_CHAINS: [[&str; 1]; 12] = [
    ["afterAll"],
    ["afterEach"],
    ["beforeAll"],
    ["beforeEach"],
    ["describe"],
    ["fdescribe"],
    ["xdescribe"],
    ["it"],
    ["fit"],
    ["xit"],
    ["test"],
    ["xtest"],
];

const VALID_JEST_FN_CALL_CHAINS_2: [[&str; 2]; 23] = [
    ["describe", "each"],
    ["describe", "only"],
    ["describe", "skip"],
    ["fdescribe", "each"],
    ["xdescribe", "each"],
    ["it", "concurrent"],
    ["it", "each"],
    ["it", "failing"],
    ["it", "only"],
    ["it", "skip"],
    ["it", "todo"],
    ["fit", "each"],
    ["fit", "failing"],
    ["xit", "each"],
    ["xit", "failing"],
    ["test", "concurrent"],
    ["test", "each"],
    ["test", "failing"],
    ["test", "only"],
    ["test", "skip"],
    ["test", "todo"],
    ["xtest", "each"],
    ["xtest", "failing"],
];

const VALID_JEST_FN_CALL_CHAINS_3: [[&str; 3]; 12] = [
    ["describe", "only", "each"],
    ["describe", "skip", "each"],
    ["it", "concurrent", "each"],
    ["it", "only", "each"],
    ["it", "only", "failing"],
    ["it", "skip", "each"],
    ["it", "skip", "failing"],
    ["test", "concurrent", "each"],
    ["test", "only", "each"],
    ["test", "only", "failing"],
    ["test", "skip", "each"],
    ["test", "skip", "failing"],
];

const VALID_JEST_FN_CALL_CHAINS_4: [[&str; 4]; 4] = [
    ["it", "concurrent", "only", "each"],
    ["it", "concurrent", "skip", "each"],
    ["test", "concurrent", "only", "each"],
    ["test", "concurrent", "skip", "each"],
];
