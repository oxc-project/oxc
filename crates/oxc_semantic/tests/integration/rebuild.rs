use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn parse_and_build_scoping<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> oxc_semantic::Scoping {
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
