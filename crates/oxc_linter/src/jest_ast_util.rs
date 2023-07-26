use std::borrow::Cow;

use oxc_ast::ast::{CallExpression, Expression, IdentifierReference};
use oxc_span::Atom;

use crate::context::LintContext;

pub enum JestFnKind {
    Hook,
    Describe,
    Test,
    Expect,
    Jest,
    Unknown,
}

impl JestFnKind {
    pub fn from(name: &str) -> Self {
        match name {
            "expect" => Self::Expect,
            "jest" => Self::Jest,
            "describe" | "fdescribe" | "xdescribe" => Self::Describe,
            "fit" | "it" | "test" | "xit" | "xtest" => Self::Test,
            "beforeAll" | "beforeEach" | "afterAll" | "afterEach" => Self::Hook,
            _ => Self::Unknown,
        }
    }
}

pub struct ParsedJestFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<Cow<'a, str>>,
    pub raw: Cow<'a, str>,
}

pub fn parse_jest_fn_call<'a>(
    call_expr: &'a CallExpression,
    ctx: &'a LintContext,
) -> Option<ParsedJestFnCall<'a>> {
    let callee = &call_expr.callee;

    // if bailed out, we're not a jest function
    let resolved = resolve_to_jest_fn(call_expr, ctx)?;

    let chain = get_node_chain(callee);
    if let (Some(first), Some(last)) = (chain.first(), chain.last()) {
        // if we're an `each()`, ensure we're the outer CallExpression (i.e `.each()()`)
        if last == "each"
            && !matches!(
                callee,
                Expression::CallExpression(_) | Expression::TaggedTemplateExpression(_)
            )
        {
            return None;
        }

        if matches!(callee, Expression::TaggedTemplateExpression(_)) && last != "each" {
            return None;
        }

        let kind = JestFnKind::from(first);
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
            VALID_JEST_FN_CALL_CHAINS_2.iter().any(|chain| chain[0] == name && chain[1] == members[0])
        } else if members.len() == 2 {
            VALID_JEST_FN_CALL_CHAINS_3.iter().any(|chain| chain[0] == name && chain[1] == members[0] && chain[2] == members[1])
        } else if members.len() == 3 {
            VALID_JEST_FN_CALL_CHAINS_4.iter().any(|chain| chain[0] == name && chain[1] == members[0] && chain[2] == members[1] && chain[3] == members[2])
        } else { false };

        if !is_valid_jest_call {
            return None;
        }
        return Some(ParsedJestFnCall { kind, members, raw: first });
    }

    None
}

struct ResolvedJestFn<'a> {
    pub local: &'a Atom,
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

/// a.b.c -> ["a", "b"]
/// a[`b`] - > ["a", "b"]
/// a["b"] - > ["a", "b"]
/// a[b] - > ["a", "b"]
fn get_node_chain<'a>(expr: &'a Expression) -> Vec<Cow<'a, str>> {
    let mut chain = Vec::new();
    match expr {
        Expression::MemberExpression(member_expr) => {
            chain.extend(get_node_chain(member_expr.object()));
            if let Some(name) = member_expr.static_property_name() {
                chain.push(Cow::Borrowed(name));
            }
        }
        Expression::Identifier(ident) => {
            chain.push(Cow::Borrowed(ident.name.as_str()));
        }
        Expression::CallExpression(call_expr) => {
            let sub_chain = get_node_chain(&call_expr.callee);
            chain.extend(sub_chain);
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            let sub_chain = get_node_chain(&tagged_expr.tag);
            chain.extend(sub_chain);
        }
        Expression::StringLiteral(string_literal) => {
            chain.push(Cow::Borrowed(string_literal.value.as_str()));
        }
        Expression::TemplateLiteral(template_literal) => {
            if template_literal.expressions.is_empty() && template_literal.quasis.len() == 1 {
                chain.push(Cow::Borrowed(
                    template_literal.quasi().expect("get string content").as_str(),
                ));
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
