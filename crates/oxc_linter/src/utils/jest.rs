use cow_utils::CowUtils;
use lazy_regex::Regex;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::borrow::Cow;

use oxc_allocator::GetAddress;
use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier, Statement,
        match_member_expression,
    },
};
use oxc_semantic::{AstNode, ReferenceId, Semantic, SymbolId};
use oxc_str::CompactStr;

pub use crate::utils::jest::parse_jest_fn::{
    ExpectError, KnownMemberExpressionParentKind, KnownMemberExpressionProperty,
    MemberExpressionElement, ParsedExpectFnCall, ParsedGeneralJestFnCall,
    ParsedJestFnCall as ParsedJestFnCallNew, parse_jest_fn_call,
};
use crate::{LintContext, utils::is_vitest_import_source};
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

/// <https://jestjs.io/docs/configuration#testmatch-arraystring>
pub fn is_jest_file(ctx: &LintContext) -> bool {
    if ctx.file_path().components().any(|c| match c {
        std::path::Component::Normal(p) => p == std::ffi::OsStr::new("__tests__"),
        _ => false,
    }) {
        return true;
    }

    let file_path = ctx.file_path().to_string_lossy();
    [
        "spec.js", "spec.jsx", "spec.ts", "spec.tsx", "spec.mjs", "spec.cjs", "spec.mts",
        "spec.cts", "test.js", "test.jsx", "test.ts", "test.tsx", "test.mjs", "test.cjs",
        "test.mts", "test.cts",
    ]
    .iter()
    .any(|ext| file_path.ends_with(ext))
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
    // Imported name, including Vitest tests derived with `.extend()`. None for globals.
    pub original: Option<&'a str>,
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
                    AstKind::StaticMemberExpression(_) | AstKind::TaggedTemplateExpression(_)
                ) || matches!(
                    parent_kind,
                    AstKind::ComputedMemberExpression(member)
                        if member.object.address() == semantic.nodes().get_node(id).address()
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
    let has_vitest_import = semantic.nodes().program().body.iter().any(|statement| {
        matches!(statement, Statement::ImportDeclaration(import_decl)
            if is_vitest_import_source(import_decl.source.value.as_str()))
    });
    let mut fixture_bindings = FxHashMap::default();
    semantic
        .scoping()
        .resolved_references()
        .enumerate()
        .filter_map(move |(symbol_id, reference_ids)| {
            let symbol_id = SymbolId::from_usize(symbol_id);
            let original = if semantic.scoping().symbol_flags(symbol_id).is_import() {
                let id = semantic.scoping().symbol_declaration(symbol_id);
                let AstKind::ImportDeclaration(import_decl) = semantic.nodes().parent_kind(id)
                else {
                    return None;
                };
                let name = semantic.scoping().symbol_name(symbol_id);

                if !matches!(
                    import_decl.source.value.as_str(),
                    "@jest/globals" | "vitest" | "vite-plus/test" | "@effect/vitest"
                ) {
                    return None;
                }
                find_original_name(import_decl, name)
            } else if has_vitest_import {
                Some(find_vitest_fixture_name(symbol_id, semantic, &mut fixture_bindings)?)
            } else {
                return None;
            };
            Some(reference_ids.iter().map(move |&reference_id| (reference_id, original)))
        })
        .flatten()
}

#[derive(Clone, Copy)]
struct VitestTestBinding<'a> {
    name: &'a str,
    is_extended: bool,
}

/// Follow stable aliases and `.extend()` calls back to a named Vitest test import.
fn find_vitest_fixture_name<'a>(
    mut symbol_id: SymbolId,
    semantic: &Semantic<'a>,
    bindings: &mut FxHashMap<SymbolId, Option<VitestTestBinding<'a>>>,
) -> Option<&'a str> {
    let scoping = semantic.scoping();
    let mut pending = SmallVec::<[(SymbolId, bool); 4]>::new();

    let mut binding = loop {
        if let Some(binding) = bindings.get(&symbol_id) {
            break *binding;
        }
        let flags = scoping.symbol_flags(symbol_id);
        let declaration_id = scoping.symbol_declaration(symbol_id);
        if flags.is_import() {
            let AstKind::ImportDeclaration(import_decl) =
                semantic.nodes().parent_kind(declaration_id)
            else {
                return None;
            };
            let original = find_original_name(import_decl, scoping.symbol_name(symbol_id))?;
            break (is_vitest_import_source(import_decl.source.value.as_str())
                && matches!(original, "test" | "it"))
            .then_some(VitestTestBinding { name: original, is_extended: false });
        }

        if !flags.is_variable() || scoping.symbol_is_mutated(symbol_id) {
            return None;
        }

        let AstKind::VariableDeclarator(declaration) = semantic.nodes().kind(declaration_id) else {
            return None;
        };
        if !declaration.id.is_binding_identifier() {
            return None;
        }
        let mut expression = declaration.init.as_ref()?.get_inner_expression();
        let mut is_extended = false;
        while let Expression::CallExpression(call) = expression {
            let member = call.callee.get_inner_expression().as_member_expression()?;
            if member.static_property_name() != Some("extend") {
                return None;
            }
            is_extended = true;
            expression = member.object().get_inner_expression();
        }
        let Expression::Identifier(ident) = expression else { return None };
        // Mark in-progress bindings as unresolved to break cycles. Cache completed
        // chains as well, so long chains of aliases are only traversed once.
        bindings.insert(symbol_id, None);
        pending.push((symbol_id, is_extended));
        symbol_id = scoping.get_reference(ident.reference_id()).symbol_id()?;
    };
    bindings.insert(symbol_id, binding);
    while let Some((symbol_id, is_extended)) = pending.pop() {
        if let Some(binding) = &mut binding {
            binding.is_extended |= is_extended;
        }
        bindings.insert(symbol_id, binding);
    }
    binding.filter(|binding| binding.is_extended).map(|binding| binding.name)
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

#[cfg(test)]
mod test {
    use std::{rc::Rc, sync::Arc};

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use crate::{
        ContextHost, ModuleRecord,
        context::{ContextSubHost, ContextSubHostOptions},
        options::LintOptions,
    };

    #[test]
    fn test_is_jest_file() {
        let allocator = Allocator::default();

        let build_ctx = |path: &'static str| {
            let source_type = SourceType::default();
            let parser_ret = Parser::new(&allocator, "", source_type).parse();
            let program = allocator.alloc(parser_ret.program);
            let semantic = SemanticBuilder::new_linter().build(program).semantic;
            Rc::new(ContextHost::new(
                path,
                vec![ContextSubHost::new(
                    semantic,
                    Arc::new(ModuleRecord::default()),
                    0,
                    ContextSubHostOptions::default(),
                )],
                &allocator,
                LintOptions::default(),
                Arc::default(),
            ))
            .spawn_for_test()
        };

        let ctx = build_ctx("foo.js");
        assert!(!super::is_jest_file(&ctx));

        let ctx = build_ctx("foo.test.js");
        assert!(super::is_jest_file(&ctx));

        let ctx = build_ctx("__tests__/foo/test.spec.js");
        assert!(super::is_jest_file(&ctx));
    }
}
