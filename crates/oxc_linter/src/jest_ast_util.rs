use std::borrow::Cow;

use oxc_ast::{
    ast::{
        CallExpression, Expression, IdentifierName, IdentifierReference,
        ImportDeclarationSpecifier, MemberExpression, ModuleDeclaration,
    },
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{Atom, Span};

use crate::context::LintContext;

pub fn parse_general_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
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
    ctx: &LintContext<'a>,
) -> Option<ParsedJestFnCall<'a>> {
    let callee = &call_expr.callee;

    // If bailed out, we're not jest function
    let resolved = resolve_to_jest_fn(call_expr, ctx)?;

    // only the top level Call expression callee's parent is None, it's not necessary to set it to None, but
    // I didn't know how to pass Expression to it.
    let chain = get_node_chain(callee, None);
    let all_member_expr_except_last = chain
        .iter()
        .rev()
        .skip(1)
        .all(|member| matches!(member.parent, Some(Expression::MemberExpression(_))));

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

    if let Some(last) = chain.last() {
        // If we're an `each()`, ensure we're the outer CallExpression (i.e `.each()()`)
        if last.is_name_equal("each")
            && !matches!(
                callee,
                Expression::CallExpression(_) | Expression::TaggedTemplateExpression(_)
            )
        {
            return None;
        }

        if matches!(callee, Expression::TaggedTemplateExpression(_)) && last.is_name_unequal("each")
        {
            return None;
        }

        let name = resolved.original.unwrap_or(resolved.local).as_str();
        let kind = JestFnKind::from(name);
        let mut members = Vec::new();
        let iter = chain.into_iter().skip(1);
        let rest = iter;

        // every member node must have a member expression as their parent
        // in order to be part of the call chain we're parsing
        for member in rest {
            members.push(member);
        }

        let is_valid_jest_call = if members.is_empty() {
            VALID_JEST_FN_CALL_CHAINS.iter().any(|chain| chain[0] == name)
        } else if members.len() == 1 {
            VALID_JEST_FN_CALL_CHAINS_2
                .iter()
                .any(|chain| chain[0] == name && members[0].is_name_equal(chain[1]))
        } else if members.len() == 2 {
            VALID_JEST_FN_CALL_CHAINS_3.iter().any(|chain| {
                chain[0] == name
                    && members[0].is_name_equal(chain[1])
                    && members[1].is_name_equal(chain[2])
            })
        } else if members.len() == 3 {
            VALID_JEST_FN_CALL_CHAINS_4.iter().any(|chain| {
                chain[0] == name
                    && members[0].is_name_equal(chain[1])
                    && members[1].is_name_equal(chain[2])
                    && members[2].is_name_equal(chain[3])
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
            name: Cow::Borrowed(name),
        }));
    }

    None
}

fn resolve_to_jest_fn<'a>(
    call_expr: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<ResolvedJestFn<'a>> {
    let ident = resolve_first_ident(&call_expr.callee)?;
    if ctx.semantic().is_reference_to_global_variable(ident) {
        return Some(ResolvedJestFn {
            local: &ident.name,
            kind: JestFnFrom::Global,
            original: None,
        });
    }

    let node_id = get_import_decl_node_id(ident, ctx)?;
    let node = ctx.nodes().get_node(node_id);
    let AstKind::ModuleDeclaration(module_decl) = node.kind() else {
        return None;
    };
    let ModuleDeclaration::ImportDeclaration(import_decl) = module_decl else {
        return None;
    };

    if import_decl.source.value == "@jest/globals" {
        let original = import_decl.specifiers.iter().find_map(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                Some(import_specifier.imported.name())
            }
            _ => None,
        });

        return Some(ResolvedJestFn { local: &ident.name, kind: JestFnFrom::Import, original });
    }
    None
}

fn get_import_decl_node_id(ident: &IdentifierReference, ctx: &LintContext) -> Option<AstNodeId> {
    let symbol_table = ctx.semantic().symbols();
    let reference_id = ident.reference_id.get()?;
    let reference = symbol_table.get_reference(reference_id);
    // import binding is not a write reference
    if reference.is_write() {
        return None;
    }
    let symbol_id = reference.symbol_id()?;
    if symbol_table.get_flag(symbol_id).is_import_binding() {
        return Some(symbol_table.get_declaration(symbol_id));
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
    pub name: Cow<'a, str>,
}

pub struct ParsedExpectFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub raw: Cow<'a, str>,
    pub name: Cow<'a, str>,
    // pub args: Vec<&'a Expression<'a>>
    // TODO: add `modifiers`, `matcher` for this struct.
}

struct ResolvedJestFn<'a> {
    pub local: &'a Atom,
    pub original: Option<&'a Atom>,
    #[allow(unused)]
    kind: JestFnFrom,
}

pub enum JestFnFrom {
    Global,
    Import,
}

pub struct KnownMemberExpressionProperty<'a> {
    pub element: MemberExpressionElement<'a>,
    pub parent: Option<&'a Expression<'a>>,
    pub span: Span,
}

impl<'a> KnownMemberExpressionProperty<'a> {
    pub fn name(&self) -> Option<Cow<'a, str>> {
        match &self.element {
            MemberExpressionElement::Expression(expr) => match expr {
                Expression::Identifier(ident) => Some(Cow::Borrowed(ident.name.as_str())),
                Expression::StringLiteral(string_literal) => {
                    Some(Cow::Borrowed(string_literal.value.as_str()))
                }
                Expression::TemplateLiteral(template_literal) => Some(Cow::Borrowed(
                    template_literal.quasi().expect("get string content").as_str(),
                )),
                _ => None,
            },
            MemberExpressionElement::IdentName(ident_name) => {
                Some(Cow::Borrowed(ident_name.name.as_str()))
            }
        }
    }
    pub fn is_name_equal(&self, name: &str) -> bool {
        self.name().map_or(false, |n| n == name)
    }
    pub fn is_name_unequal(&self, name: &str) -> bool {
        !self.is_name_equal(name)
    }
}

pub enum MemberExpressionElement<'a> {
    Expression(&'a Expression<'a>),
    IdentName(&'a IdentifierName),
}

impl<'a> MemberExpressionElement<'a> {
    pub fn from_member_expr(
        member_expr: &'a MemberExpression<'a>,
    ) -> Option<(Span, MemberExpressionElement<'a>)> {
        let Some((span, _)) = member_expr.static_property_info() else { return None };
        match member_expr {
            MemberExpression::ComputedMemberExpression(expr) => {
                Some((span, Self::Expression(&expr.expression)))
            }
            MemberExpression::StaticMemberExpression(expr) => {
                Some((span, Self::IdentName(&expr.property)))
            }
            // Jest fn chains don't have private fields, just ignore it.
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }
}

/// Port from [eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest/blob/a058f22f94774eeea7980ea2d1f24c6808bf3e2c/src/rules/utils/parseJestFnCall.ts#L36-L51)
fn get_node_chain<'a>(
    expr: &'a Expression<'a>,
    parent: Option<&'a Expression<'a>>,
) -> Vec<KnownMemberExpressionProperty<'a>> {
    let mut chain = Vec::new();

    match expr {
        Expression::MemberExpression(member_expr) => {
            chain.extend(get_node_chain(member_expr.object(), Some(expr)));
            if let Some((span, element)) = MemberExpressionElement::from_member_expr(member_expr) {
                chain.push(KnownMemberExpressionProperty { element, parent: Some(expr), span });
            }
        }
        Expression::Identifier(ident) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent,
                span: ident.span,
            });
        }
        Expression::CallExpression(call_expr) => {
            let sub_chain = get_node_chain(&call_expr.callee, Some(expr));
            chain.extend(sub_chain);
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            let sub_chain = get_node_chain(&tagged_expr.tag, Some(expr));
            chain.extend(sub_chain);
        }
        Expression::StringLiteral(string_literal) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent,
                span: string_literal.span,
            });
        }
        Expression::TemplateLiteral(template_literal) => {
            if template_literal.expressions.is_empty() && template_literal.quasis.len() == 1 {
                chain.push(KnownMemberExpressionProperty {
                    element: MemberExpressionElement::Expression(expr),
                    parent,
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
