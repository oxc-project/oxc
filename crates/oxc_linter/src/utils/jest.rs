use std::borrow::Cow;

use oxc_ast::ast::{CallExpression, Expression, TemplateLiteral};
use oxc_semantic::AstNode;

use crate::LintContext;

mod parse_jest_fn;

use crate::utils::jest::parse_jest_fn::ParsedJestFnCall;
pub use crate::utils::jest::parse_jest_fn::{
    parse_jest_fn_call, ExpectError, KnownMemberExpressionParentKind,
    KnownMemberExpressionProperty, MemberExpressionElement, ParsedExpectFnCall,
    ParsedGeneralJestFnCall,
};

const JEST_METHOD_NAMES: [&str; 14] = [
    "afterAll",
    "afterEach",
    "beforeAll",
    "beforeEach",
    "describe",
    "expect",
    "fdescribe",
    "fit",
    "it",
    "jest",
    "test",
    "xdescribe",
    "xit",
    "xtest",
];

pub const JEST_HOOK_NAMES: [&str; 4] = ["afterAll", "afterEach", "beforeAll", "beforeEach"];

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JestGeneralFnKind {
    Hook,
    Describe,
    Test,
    Jest,
}

pub fn is_jest_file(ctx: &LintContext) -> bool {
    if JEST_METHOD_NAMES
        .iter()
        .any(|name| ctx.scopes().root_unresolved_references().contains_key(*name))
    {
        return true;
    };

    let import_entries = &ctx.semantic().module_record().import_entries;

    return import_entries.iter().any(|import_entry| {
        matches!(import_entry.module_request.name().as_str(), "@jest/globals")
    });
}

pub fn is_type_of_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    kinds: &[JestFnKind],
) -> bool {
    let jest_fn_call = parse_jest_fn_call(call_expr, node, ctx);
    if let Some(jest_fn_call) = jest_fn_call {
        let kind = jest_fn_call.kind();
        if kinds.contains(&kind) {
            return true;
        }
    }

    false
}

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

pub fn parse_expect_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<ParsedExpectFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, node, ctx)?;

    if let ParsedJestFnCall::ExpectFnCall(jest_fn_call) = jest_fn_call {
        return Some(jest_fn_call);
    }
    None
}

/// join name of the expression. e.g.
/// `expect(foo).toBe(bar)`  -> "expect.toBe"
/// `new Foo().bar` -> "Foo.bar"
pub fn get_node_name<'a>(expr: &'a Expression<'a>) -> String {
    let chain = get_node_name_vec(expr);
    chain.join(".")
}

pub fn get_node_name_vec<'a>(expr: &'a Expression<'a>) -> Vec<Cow<'a, str>> {
    let mut chain: Vec<Cow<'a, str>> = Vec::new();

    match expr {
        Expression::Identifier(ident) => chain.push(Cow::Borrowed(ident.name.as_str())),
        Expression::StringLiteral(string_literal) => {
            chain.push(Cow::Borrowed(&string_literal.value));
        }
        Expression::TemplateLiteral(template_literal) if is_pure_string(template_literal) => {
            chain.push(Cow::Borrowed(template_literal.quasi().unwrap()));
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            chain.extend(get_node_name_vec(&tagged_expr.tag));
        }
        Expression::CallExpression(call_expr) => chain.extend(get_node_name_vec(&call_expr.callee)),
        Expression::MemberExpression(member_expr) => {
            chain.extend(get_node_name_vec(member_expr.object()));
            if let Some(name) = member_expr.static_property_name() {
                chain.push(Cow::Borrowed(name));
            }
        }
        Expression::NewExpression(new_expr) => {
            chain.extend(get_node_name_vec(&new_expr.callee));
        }
        _ => {}
    };

    chain
}

fn is_pure_string(template_literal: &TemplateLiteral) -> bool {
    template_literal.expressions.is_empty() && template_literal.quasis.len() == 1
}
