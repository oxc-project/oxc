mod util;
use oxc_semantic::SymbolFlags;
use util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js("export class Foo {}")
        .has_root_symbol("Foo")
        .contains_flags(SymbolFlags::Class | SymbolFlags::Export)
        .has_number_of_references(0)
        .test();

    SemanticTester::js("class Foo {}; let f = new Foo()")
        .has_root_symbol("Foo")
        .has_number_of_reads(1)
        .test();
}

#[test]
fn test_function_simple() {
    SemanticTester::js("function foo() { return }")
        .has_root_symbol("foo")
        .contains_flags(SymbolFlags::Function)
        .test();
}

#[test]
fn test_var_simple() {
    SemanticTester::js("let x; { let y; }")
    .has_some_symbol("x")
    .intersects_flags(SymbolFlags::Variable)
    .test();
}
