use std::borrow::Cow;

use oxc_ast::{
    ast::{
        match_member_expression, CallExpression, Expression, ImportDeclaration,
        ImportDeclarationSpecifier, TemplateLiteral,
    },
    AstKind,
};
use oxc_semantic::{AstNode, ReferenceId, Semantic};
use oxc_span::CompactStr;
use phf::phf_set;

use crate::LintContext;

mod parse_jest_fn;
pub use crate::utils::jest::parse_jest_fn::{
    parse_jest_fn_call, ExpectError, KnownMemberExpressionParentKind,
    KnownMemberExpressionProperty, MemberExpressionElement, ParsedExpectFnCall,
    ParsedGeneralJestFnCall, ParsedJestFnCall as ParsedJestFnCallNew,
};

pub const JEST_METHOD_NAMES: phf::Set<&'static str> = phf_set![
    "afterAll",
    "afterEach",
    "beforeAll",
    "beforeEach",
    "describe",
    "expect",
    "expectTypeOf",
    "fdescribe",
    "fit",
    "it",
    "jest",
    "vi",
    "test",
    "xdescribe",
    "xit",
    "xtest",
    "pending"
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JestFnKind {
    Expect,
    ExpectTypeOf,
    General(JestGeneralFnKind),
    Unknown,
}

impl JestFnKind {
    pub fn from(name: &str) -> Self {
        match name {
            "expect" => Self::Expect,
            "expectTypeOf" => Self::ExpectTypeOf,
            "vi" => Self::General(JestGeneralFnKind::Vitest),
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
    Vitest,
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
    ["spec.js", "spec.jsx", "spec.ts", "spec.tsx", "test.js", "test.jsx", "test.ts", "test.tsx"]
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
        let mut id = semantic.symbols().get_reference(reference_id).node_id();
        std::iter::from_fn(move || loop {
            let parent = semantic.nodes().parent_node(id);
            if let Some(parent) = parent {
                let parent_kind = parent.kind();
                if matches!(parent_kind, AstKind::CallExpression(_)) {
                    id = parent.id();
                    return Some(PossibleJestNode { node: parent, original });
                } else if matches!(
                    parent_kind,
                    AstKind::MemberExpression(_) | AstKind::TaggedTemplateExpression(_)
                ) {
                    id = parent.id();
                } else {
                    return None;
                }
            } else {
                return None;
            }
        })
    })
}

fn collect_ids_referenced_to_import<'a, 'c>(
    semantic: &'c Semantic<'a>,
) -> impl Iterator<Item = (ReferenceId, Option<&'a str>)> + 'c {
    semantic
        .symbols()
        .resolved_references
        .iter_enumerated()
        .filter_map(|(symbol_id, reference_ids)| {
            if semantic.symbols().get_flags(symbol_id).is_import() {
                let id = semantic.symbols().get_declaration(symbol_id);
                let Some(AstKind::ImportDeclaration(import_decl)) =
                    semantic.nodes().parent_kind(id)
                else {
                    return None;
                };
                let name = semantic.symbols().get_name(symbol_id);

                if matches!(import_decl.source.value.as_str(), "@jest/globals" | "vitest") {
                    let original = find_original_name(import_decl, name);
                    let mut ret = vec![];
                    for reference_id in reference_ids {
                        ret.push((*reference_id, original));
                    }

                    return Some(ret);
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
) -> impl Iterator<Item = ReferenceId> + 'c {
    semantic
        .scopes()
        .root_unresolved_references()
        .iter()
        .filter(|(name, _)| JEST_METHOD_NAMES.contains(name.as_str()))
        .flat_map(|(_, reference_ids)| reference_ids.iter().copied())
}

/// join name of the expression. e.g.
/// `expect(foo).toBe(bar)`  -> "expect.toBe"
/// `new Foo().bar` -> "Foo.bar"
pub fn get_node_name<'a>(expr: &'a Expression<'a>) -> CompactStr {
    let chain = get_node_name_vec(expr);
    chain.join(".").into()
}

pub fn get_node_name_vec<'a>(expr: &'a Expression<'a>) -> Vec<Cow<'a, str>> {
    let mut chain: Vec<Cow<'a, str>> = Vec::new();

    match expr {
        Expression::Identifier(ident) => chain.push(Cow::Borrowed(ident.name.as_str())),
        Expression::StringLiteral(string_literal) => {
            chain.push(Cow::Borrowed(&string_literal.value));
        }
        Expression::TemplateLiteral(template_literal) if is_pure_string(template_literal) => {
            chain.push(Cow::Borrowed(template_literal.quasi().unwrap().as_str()));
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
    };

    chain
}

fn is_pure_string(template_literal: &TemplateLiteral) -> bool {
    template_literal.expressions.is_empty() && template_literal.quasis.len() == 1
}

pub fn is_equality_matcher(matcher: &KnownMemberExpressionProperty) -> bool {
    matcher.is_name_equal("toBe")
        || matcher.is_name_equal("toEqual")
        || matcher.is_name_equal("toStrictEqual")
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use crate::{options::LintOptions, ContextHost};

    #[test]
    fn test_is_jest_file() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let parser_ret = Parser::new(&allocator, "", source_type).parse();
        let semantic_ret =
            SemanticBuilder::new().with_cfg(true).build(&parser_ret.program).semantic;
        let semantic_ret = Rc::new(semantic_ret);

        let build_ctx = |path: &'static str| {
            Rc::new(ContextHost::new(path, Rc::clone(&semantic_ret), LintOptions::default()))
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
