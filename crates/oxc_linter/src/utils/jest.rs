use std::borrow::Cow;

use oxc_ast::{
    ast::{CallExpression, Expression, ModuleDeclaration, TemplateLiteral},
    AstKind,
};
use oxc_semantic::{AstNode, ReferenceId};
use phf::phf_set;

use crate::LintContext;

mod parse_jest_fn;

use crate::utils::jest::parse_jest_fn::ParsedJestFnCall;
pub use crate::utils::jest::parse_jest_fn::{
    parse_jest_fn_call, ExpectError, KnownMemberExpressionParentKind,
    KnownMemberExpressionProperty, MemberExpressionElement, ParsedExpectFnCall,
    ParsedGeneralJestFnCall,
};

const JEST_METHOD_NAMES: phf::Set<&'static str> = phf_set![
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
    "pending"
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

/// Collect all possible Jest fn Call Expression,
/// for `expect(1).toBe(1)`, the result will be a collection of node `expect(1)` and node `expect(1).toBe(1)`.
pub fn collect_possible_jest_call_node<'a, 'b>(ctx: &'b LintContext<'a>) -> Vec<&'b AstNode<'a>> {
    let import_entries = &ctx.semantic().module_record().import_entries;

    // Whether test functions are imported from 'jest/globals'.
    // Not support mix global Jest functions with import Jest functions
    let is_import_mode = import_entries
        .iter()
        .any(|import_entry| matches!(import_entry.module_request.name().as_str(), "@jest/globals"));

    let reference_ids = if is_import_mode {
        collect_ids_referenced_to_import(ctx)
    } else if JEST_METHOD_NAMES
        .iter()
        .any(|name| ctx.scopes().root_unresolved_references().contains_key(*name))
    {
        collect_ids_referenced_to_global(ctx)
    } else {
        // we are not test file, just return empty vec.
        vec![]
    };

    // The longest length of Jest chains is 4, e.g.`expect(1).not.resolved.toBe()`.
    // We take 4 ancestors of node and collect all Call Expression.
    // The invalid Jest Call Expression will be bypassed in `parse_jest_fn_call`
    reference_ids.iter().fold(vec![], |mut acc, id| {
        let mut id = ctx.symbols().get_reference(*id).node_id();
        for _ in 0..4 {
            let parent = ctx.nodes().parent_node(id);
            if let Some(parent) = parent {
                let parent_kind = parent.kind();
                if matches!(parent_kind, AstKind::CallExpression(_)) {
                    acc.push(parent);
                    id = parent.id();
                } else if matches!(
                    parent_kind,
                    AstKind::MemberExpression(_) | AstKind::TaggedTemplateExpression(_)
                ) {
                    id = parent.id();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        acc
    })
}

fn collect_ids_referenced_to_import(ctx: &LintContext) -> Vec<ReferenceId> {
    ctx.symbols()
        .resolved_references
        .iter_enumerated()
        .filter(|(symbol_id, _)| {
            if ctx.symbols().get_flag(*symbol_id).is_import_binding() {
                let id = ctx.symbols().get_declaration(*symbol_id);
                let node = ctx.nodes().get_node(id);
                let AstKind::ModuleDeclaration(module_decl) = node.kind() else {
                    return false;
                };
                let ModuleDeclaration::ImportDeclaration(import_decl) = module_decl else {
                    return false;
                };

                return import_decl.source.value == "@jest/globals";
            }

            false
        })
        .flat_map(|(_, reference_ids)| reference_ids.clone())
        .collect::<Vec<ReferenceId>>()
}

fn collect_ids_referenced_to_global(ctx: &LintContext) -> Vec<ReferenceId> {
    ctx.scopes()
        .root_unresolved_references()
        .iter()
        .filter(|(name, _)| JEST_METHOD_NAMES.contains(name.as_str()))
        .flat_map(|(_, reference_ids)| reference_ids.clone())
        .collect::<Vec<ReferenceId>>()
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
