use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_ast::{AstKind, AstType, ast::Program};
use oxc_ast_visit::Visit;
use oxc_parser::{
    ParseOptions, Source,
    config::{NoTokensParserConfig, ParserConfig, RuntimeParserConfig, TokensParserConfig},
};
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::node::NodeId;

#[derive(Default)]
struct NodeIds {
    seen: FxHashSet<NodeId>,
    nodes: Vec<(AstType, Span, NodeId)>,
}

impl<'a> Visit<'a> for NodeIds {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let id = kind.node_id();
        if matches!(kind, AstKind::Program(_)) {
            assert_eq!(id, NodeId::ROOT);
        } else {
            assert_ne!(id, NodeId::DUMMY, "Unassigned ID for {kind:?}");
        }
        assert!(self.seen.insert(id), "Duplicate ID for {kind:?}");
        self.nodes.push((kind.ty(), kind.span(), id));
    }
}

fn collect(program: &Program<'_>) -> Vec<(AstType, Span, NodeId)> {
    let mut ids = NodeIds::default();
    ids.visit_program(program);
    ids.nodes
}

fn check<C: ParserConfig>(
    source_text: &str,
    source_type: SourceType,
    config: C,
    preserve_parens: bool,
) {
    let allocator = Allocator::default();
    let source = Source::from_str(source_text, source_type, &allocator);
    let options = ParseOptions { preserve_parens, ..ParseOptions::default() };
    let parsed = source.parse_with_config(options, config);
    assert!(parsed.diagnostics.is_empty(), "{source_text}: {:?}", parsed.diagnostics);
    assert!(!parsed.fatal_error, "{source_text}");
    let nodes = collect(&parsed.program);
    assert_eq!(nodes[0].0, AstType::Program);
}

#[test]
fn assigns_ids_in_all_parser_configs() {
    let cases = [
        ("", SourceType::mjs()),
        ("/* comments only */", SourceType::mjs()),
        ("#!/usr/bin/env node\n'use strict';\nvalue;", SourceType::mjs()),
        ("function f([first, ...rest]) { return [first, , `${rest.length}`]; }", SourceType::mjs()),
        (
            "type Pair<T> = [left: T, right: T]; const pair: Pair<number> = [1, 2];",
            SourceType::ts(),
        ),
        (
            "const element = <Component prop={value}><>{/* comment */}{value}</></Component>;",
            SourceType::jsx(),
        ),
        ("const element: JSX.Element = <Component<string> prop={value} />;", SourceType::tsx()),
    ];
    for (source_text, source_type) in cases {
        for preserve_parens in [false, true] {
            check(source_text, source_type, NoTokensParserConfig, preserve_parens);
            check(source_text, source_type, TokensParserConfig, preserve_parens);
            for tokens in [false, true] {
                check(source_text, source_type, RuntimeParserConfig::new(tokens), preserve_parens);
            }
        }
    }
}

#[test]
fn ids_survive_speculation_and_cover_grammar() {
    for source_text in [
        "const f = (a = call()) => a; const value = (a + b);",
        "const f = async (a = call()) => a; const value = async(a + b);",
        "const f = <T>(a: T): T => a; const value = (a as number);",
        "({ a: [b = c, ...rest], d = e } = value); [first, , ...rest] = value;",
        "const value = ((a ? (b): string => b : c));",
    ] {
        for preserve_parens in [false, true] {
            check(source_text, SourceType::ts(), NoTokensParserConfig, preserve_parens);
        }
    }
}

#[test]
fn ids_survive_unambiguous_await_reparsing() {
    for source_text in [
        "before(); await /x/u; between(); await /y/g; export {}; tail();",
        "before(); await /a/ /b/g; between(); await /x/u; export {}; tail();",
        "await\nawait /x/u; export {};",
        "const x = await /x/u; between(); const y = await /y/g; export {};",
        "#!/usr/bin/env node\n'use strict';\nawait /* regexp */ /x/u; export {};",
    ] {
        check(source_text, SourceType::unambiguous(), NoTokensParserConfig, true);
        check(source_text, SourceType::unambiguous(), TokensParserConfig, true);
    }
}

#[test]
fn assigns_ids_to_recovered_nodes() {
    for (source_text, source_type) in [
        ("interface I { [key: boolean]: string; }", SourceType::ts()),
        ("interface I extends (value) {}", SourceType::ts()),
        ("const value = <><div /></div>;", SourceType::jsx()),
    ] {
        let allocator = Allocator::default();
        let parsed = Source::from_str(source_text, source_type, &allocator).parse();
        assert!(!parsed.fatal_error, "{source_text}: {:?}", parsed.diagnostics);
        assert!(!parsed.diagnostics.is_empty(), "{source_text}");
        assert!(collect(&parsed.program).len() > 1);
    }
}

#[test]
fn fatal_errors_return_only_the_root_node() {
    let allocator = Allocator::default();
    let source = Source::from_str("before(); const value = ;", SourceType::mjs(), &allocator);
    let parsed = source.parse();
    assert!(parsed.fatal_error);
    assert_eq!(collect(&parsed.program), [(AstType::Program, Span::default(), NodeId::ROOT)]);
}

#[test]
fn expression_parser_assigns_nonzero_ids() {
    for source_text in ["left + right", "(a + b)", "({ a: [b = c, ...rest] } = value)"] {
        let allocator = Allocator::default();
        let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
        let expression = source.parse_expression().unwrap();
        let mut ids = NodeIds::default();
        ids.visit_expression(&expression);
        assert_ne!(ids.nodes, []);
    }
}

#[test]
fn each_parse_starts_a_new_id_sequence() {
    let allocator = Allocator::default();
    let source_text = "'use strict'; function f(a) { return a + 1; }";
    let parse = || Source::from_str(source_text, SourceType::mjs(), &allocator).parse().program;
    let first = parse();
    Source::from_str("unrelated();", SourceType::mjs(), &allocator).parse();
    let second = parse();
    assert_eq!(collect(&first), collect(&second));
}
