use oxc_semantic::SymbolFlags;

use crate::util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js("export class Foo {};")
        .has_root_symbol("Foo")
        .contains_flags(SymbolFlags::Class | SymbolFlags::Export)
        .has_number_of_references(0)
        .is_exported()
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
fn test_function_expressions() {
    SemanticTester::js("const x = function y() {}")
        .has_some_symbol("y")
        .contains_flags(SymbolFlags::Function)
        .test();
    SemanticTester::js("const x = () => {}")
        .has_some_symbol("x")
        .contains_flags(SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable)
        .test();
}

#[test]
fn test_var_simple() {
    SemanticTester::js("let x; { let y; }")
        .has_some_symbol("x")
        .intersects_flags(SymbolFlags::Variable)
        .contains_flags(SymbolFlags::BlockScopedVariable)
        .test();
}

#[test]
fn test_var_read_write() {
    SemanticTester::js("let x; x += 1")
        .has_root_symbol("x")
        .has_number_of_references(1)
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();

    SemanticTester::js("let a; let b = 1 + (0, ((a)));")
        .has_some_symbol("a")
        .has_number_of_reads(1)
        .has_number_of_writes(0)
        .test();

    SemanticTester::js(
        "
        let x;
        function foo(a) {
            console.log(x(a))
        }",
    )
    .has_some_symbol("x")
    .has_number_of_reads(1)
    .has_number_of_writes(0)
    .test();
}

#[test]
fn test_types_simple() {
    let test = SemanticTester::ts(
        "
    interface A {
      x: number;
      y: string;
    }
    type T = { x: number; y: string; }

    const t: T = { x: 1, y: 'foo' };
    ",
    );
    test.has_root_symbol("A")
        .contains_flags(SymbolFlags::Interface)
        .has_number_of_references(0)
        .test();

    test.has_root_symbol("T")
        .contains_flags(SymbolFlags::TypeAlias)
        .has_number_of_references(1)
        .test();
}

#[test]
fn test_value_used_as_type() {
    // Type annotations (or any type reference) do not resolve to value symbols
    SemanticTester::ts(
        "
    const x = 1;
    function foo(a: x) { }
    ",
    )
    .has_root_symbol("x")
    .intersects_flags(SymbolFlags::Value)
    .has_number_of_references(0)
    .test();

    // T is a value that gets shadowed by a type. When `T` is referenced within
    // a value context, the root `const T` should be the symbol recoreded in the
    // reference.
    let tester = SemanticTester::ts(
        "
const T = 1;
function foo<T extends number>(a: T) {
    return a + T;
}
",
    );

    tester.has_root_symbol("T").has_number_of_reads(1).test();
    // TODO: type annotations not currently recorded as a type/read reference
    // This `T` is the type parameter
    // tester.has_symbol_at_offset(28).has_number_of_reads(1).test();
}

#[test]
fn test_export_flag() {
    let tester = SemanticTester::js(
        "
        const a = 1;
        export { a, b as d };
        class b {}
        export default c;
        function c() {}
    ",
    );

    tester.has_root_symbol("a").contains_flags(SymbolFlags::Export).test();
    tester.has_root_symbol("b").contains_flags(SymbolFlags::Export).test();
    tester.has_root_symbol("c").contains_flags(SymbolFlags::Export).test();
}
