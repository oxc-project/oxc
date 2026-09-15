use cow_utils::CowUtils;
use lazy_regex::Regex;
use smallvec::SmallVec;
use std::borrow::Cow;

use oxc_allocator::GetAddress;
use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier,
        match_member_expression,
    },
};
use oxc_semantic::{AstNode, ReferenceId, Semantic, SymbolId};
use oxc_str::CompactStr;

use crate::LintContext;
pub use crate::utils::jest::parse_jest_fn::{
    ExpectError, KnownMemberExpressionParentKind, KnownMemberExpressionProperty,
    MemberExpressionElement, ParsedExpectFnCall, ParsedGeneralJestFnCall,
    ParsedJestFnCall as ParsedJestFnCallNew, parse_jest_fn_call,
};
pub use padding_around_block::report_missing_padding_before_jest_block;

mod padding_around_block;
mod parse_jest_fn;

const JEST_METHOD_NAMES: [&str; 19] = [
    "afterAll",
    "afterEach",
    "beforeAll",
    "beforeEach",
    "bench",
    "describe",
    "expect",
    "expectTypeOf",
    "fdescribe",
    "fit",
    "it",
    "jest",
    "pending",
    "suite",
    "test",
    "vi",
    "xdescribe",
    "xit",
    "xtest",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JestFnKind {
    Expect,
    ExpectTypeOf,
    General(JestGeneralFnKind),
    VitestFixture,
    Unknown,
}

impl JestFnKind {
    pub fn from(name: &str) -> Self {
        match name {
            "expect" => Self::Expect,
            "expectTypeOf" => Self::ExpectTypeOf,
            "vi" | "vitest" => Self::General(JestGeneralFnKind::Vitest),
            "bench" => Self::General(JestGeneralFnKind::Bench),
            "jest" => Self::General(JestGeneralFnKind::Jest),
            "describe" | "fdescribe" | "xdescribe" | "suite" => {
                Self::General(JestGeneralFnKind::Describe)
            }
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
    Vitest,
    Bench,
}

pub fn is_type_of_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
    kinds: &[JestFnKind],
) -> bool {
    let jest_fn_call = parse_jest_fn_call(call_expr, possible_jest_node, ctx);
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
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<ParsedGeneralJestFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, possible_jest_node, ctx)?;

    if let ParsedJestFnCallNew::GeneralJest(jest_fn_call) = jest_fn_call {
        return Some(jest_fn_call);
    }
    None
}

pub fn parse_expect_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<ParsedExpectFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, possible_jest_node, ctx)?;

    if let ParsedJestFnCallNew::Expect(jest_fn_call) = jest_fn_call {
        return Some(jest_fn_call);
    }
    None
}

pub struct PossibleJestNode<'a, 'b> {
    pub node: &'b AstNode<'a>,
    pub original: Option<&'a str>, // if this node is imported from 'jest/globals', this field will be Some(original_name), otherwise None
}

/// Collect all possible Jest fn Call Expression,
/// for `expect(1).toBe(1)`, the result will be a collection of node `expect(1)` and node `expect(1).toBe(1)`.
pub fn collect_possible_jest_call_node<'a, 'c>(
    ctx: &'c LintContext<'a>,
) -> Vec<PossibleJestNode<'a, 'c>> {
    iter_possible_jest_call_node(ctx.semantic()).collect()
}

/// Iterate over all possible Jest fn Call Expression,
/// for `expect(1).toBe(1)`, the result will be an iter over node `expect(1)` and node `expect(1).toBe(1)`.
pub fn iter_possible_jest_call_node<'a, 'c>(
    semantic: &'c Semantic<'a>,
) -> impl Iterator<Item = PossibleJestNode<'a, 'c>> + 'c {
    // Some people may write codes like below, we need lookup imported test function and global test function.
    // ```
    // import { jest as Jest } from '@jest/globals';
    // Jest.setTimeout(800);
    // test('test', () => {
    //     expect(1 + 2).toEqual(3);
    // });
    // ```
    let reference_id_with_original_list = collect_ids_referenced_to_import(semantic).chain(
        collect_ids_referenced_to_global(semantic)
            // set the original of global test function to None
            .map(|id| (id, None)),
    );

    // get the longest valid chain of Jest Call Expression
    reference_id_with_original_list.flat_map(move |(reference_id, original)| {
        let mut id = semantic.scoping().get_reference(reference_id).node_id();
        std::iter::from_fn(move || {
            loop {
                let parent = semantic.nodes().parent_node(id);
                let parent_kind = parent.kind();
                if let AstKind::CallExpression(call_expr) = parent_kind
                    && call_expr.callee.address() == semantic.nodes().get_node(id).address()
                {
                    id = parent.id();
                    return Some(PossibleJestNode { node: parent, original });
                } else if matches!(
                    parent_kind,
                    AstKind::StaticMemberExpression(_)
                        | AstKind::TaggedTemplateExpression(_)
                        | AstKind::ComputedMemberExpression(_)
                ) {
                    id = parent.id();
                } else {
                    return None;
                }
            }
        })
    })
}

fn collect_ids_referenced_to_import<'a, 'c>(
    semantic: &'c Semantic<'a>,
) -> impl Iterator<Item = (ReferenceId, Option<&'a str>)> + 'c {
    semantic
        .scoping()
        .resolved_references()
        .enumerate()
        .filter_map(|(symbol_id, reference_ids)| {
            let symbol_id = SymbolId::from_usize(symbol_id);
            if semantic.scoping().symbol_flags(symbol_id).is_import() {
                let id = semantic.scoping().symbol_declaration(symbol_id);
                let AstKind::ImportDeclaration(import_decl) = semantic.nodes().parent_kind(id)
                else {
                    return None;
                };
                let name = semantic.scoping().symbol_name(symbol_id);

                if matches!(
                    import_decl.source.value.as_str(),
                    "@jest/globals" | "vitest" | "vite-plus/test" | "@effect/vitest"
                ) {
                    let original = find_original_name(import_decl, name);
                    return Some(
                        reference_ids.iter().map(move |&reference_id| (reference_id, original)),
                    );
                }
            }

            None
        })
        .flatten()
}

/// Find name in the Import Declaration, not use name because of lifetime not long enough.
fn find_original_name<'a>(import_decl: &'a ImportDeclaration<'a>, name: &str) -> Option<&'a str> {
    import_decl.specifiers.iter().flatten().find_map(|specifier| match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
            if import_specifier.local.name.as_str() == name {
                return Some(import_specifier.imported.name().as_str());
            }
            None
        }
        _ => None,
    })
}

fn collect_ids_referenced_to_global<'c>(
    semantic: &'c Semantic,
) -> impl Iterator<Item = ReferenceId> + 'c + use<'c> {
    semantic
        .scoping()
        .root_unresolved_references()
        .iter()
        .filter(|(name, _)| JEST_METHOD_NAMES.contains(&name.as_str()))
        .flat_map(|(_, reference_ids)| reference_ids.iter().copied())
}

/// join name of the expression. e.g.
/// `expect(foo).toBe(bar)`  -> "expect.toBe"
/// `new Foo().bar` -> "Foo.bar"
pub fn get_node_name<'a>(expr: &'a Expression<'a>) -> CompactStr {
    let chain = get_node_name_vec(expr);
    chain.join(".").into()
}

pub fn get_node_name_vec<'a>(expr: &'a Expression<'a>) -> SmallVec<[Cow<'a, str>; 4]> {
    let mut chain: SmallVec<[Cow<'a, str>; 4]> = SmallVec::new();

    match expr {
        Expression::Identifier(ident) => chain.push(Cow::Borrowed(ident.name.as_str())),
        Expression::StringLiteral(string_literal) => {
            chain.push(Cow::Borrowed(&string_literal.value));
        }
        Expression::TemplateLiteral(template_literal) => {
            if let Some(quasi) = template_literal.single_quasi() {
                chain.push(Cow::Borrowed(quasi.as_str()));
            }
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            chain.extend(get_node_name_vec(&tagged_expr.tag));
        }
        Expression::CallExpression(call_expr) => chain.extend(get_node_name_vec(&call_expr.callee)),
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            chain.extend(get_node_name_vec(member_expr.object()));
            if let Some(name) = member_expr.static_property_name() {
                chain.push(Cow::Borrowed(name));
            }
        }
        Expression::NewExpression(new_expr) => {
            chain.extend(get_node_name_vec(&new_expr.callee));
        }
        _ => {}
    }

    chain
}

pub fn is_equality_matcher(matcher: &KnownMemberExpressionProperty) -> bool {
    matcher.is_name_equal("toBe")
        || matcher.is_name_equal("toEqual")
        || matcher.is_name_equal("toStrictEqual")
}

/// Checks if node names returned by getNodeName matches any of the given star patterns
pub fn matches_assert_function_name(name: &str, patterns: &[Regex]) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(name))
}

pub fn convert_pattern(pattern: &str) -> CompactStr {
    // Pre-process pattern, e.g.
    // request.*.expect -> request.[a-z\\d]*.expect
    // request.**.expect -> request.[a-z\\d\\.]*.expect
    // request.**.expect* -> request.[a-z\\d\\.]*.expect[a-z\\d]*
    let pattern = pattern
        .split('.')
        .map(|p| {
            if p == "**" {
                CompactStr::from("[a-z\\d\\.]*")
            } else {
                p.cow_replace('*', "[a-z\\d]*").into()
            }
        })
        .collect::<Vec<_>>()
        .join("\\.");

    // 'a.b.c' -> /^a\.b\.c(\.|$)/iu
    format!("(?ui)^{pattern}(\\.|$)").into()
}
