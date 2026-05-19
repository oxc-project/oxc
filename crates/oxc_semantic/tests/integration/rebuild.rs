use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn parse_and_build_scoping<'a>(allocator: &'a Allocator, source: &'a str) -> oxc_semantic::Scoping {
    let parser_ret = Parser::new(allocator, source, SourceType::default()).parse();
    assert!(parser_ret.errors.is_empty(), "parser errors: {:?}", parser_ret.errors);
    let program = allocator.alloc(parser_ret.program);
    SemanticBuilder::new().build(program).semantic.into_scoping()
}

#[test]
fn reset_preserves_allocator_capacity_and_clears_contents() {
    let allocator = Allocator::default();
    let source = "let a = 1; let b = 2; function f(x) { return x + a; }";
    let mut scoping = parse_and_build_scoping(&allocator, source);

    assert!(scoping.symbols_len() > 0);
    assert!(scoping.scopes_len() > 0);
    assert!(scoping.references_len() > 0);

    scoping.reset();

    assert_eq!(scoping.symbols_len(), 0, "symbols should be cleared");
    assert_eq!(scoping.scopes_len(), 0, "scopes should be cleared");
    assert_eq!(scoping.references_len(), 0, "references should be cleared");
    assert!(scoping.no_side_effects().is_empty());
    assert!(scoping.root_unresolved_references().is_empty());
}

#[test]
fn with_scoping_produces_same_data_as_fresh_build() {
    let allocator = Allocator::default();
    let source = "
        let a = 1;
        function f(x) { return x + a; }
        class C { m() { return new C(); } }
        const result = f(a) + new C().m();
    ";
    let parser_ret = Parser::new(&allocator, source, SourceType::default()).parse();
    assert!(parser_ret.errors.is_empty());
    let program = allocator.alloc(parser_ret.program);

    // Fresh build for the expected baseline.
    let fresh = SemanticBuilder::new().build(program).semantic.into_scoping();

    // Reset-and-rebuild path.
    let mut scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
    scoping.reset();
    let rebuilt =
        SemanticBuilder::new().with_scoping(scoping).build(program).semantic.into_scoping();

    assert_eq!(fresh.symbols_len(), rebuilt.symbols_len());
    assert_eq!(fresh.scopes_len(), rebuilt.scopes_len());
    assert_eq!(fresh.references_len(), rebuilt.references_len());

    let fresh_names: Vec<&str> = fresh.symbol_names().collect();
    let rebuilt_names: Vec<&str> = rebuilt.symbol_names().collect();
    assert_eq!(fresh_names, rebuilt_names);
}
