mod util;

use oxc_semantic::SymbolFlags;
pub use util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js("export class Foo {};")
        .has_root_symbol("Foo")
        .contains_flags(SymbolFlags::Class | SymbolFlags::Export)
        .has_number_of_references(0)
        .is_exported()
        .test();

    SemanticTester::js("export default class Foo {};")
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
fn test_type_parameter() {
    let tester = SemanticTester::ts(
        "
        type A<AB> = AB
        type B<AB> = AB
        interface C<CC> {
            C: CC;
        }
        class D<DD> {
            a: DD;
            method(b: DD) {
                
            }
        }
        function F<FF>() {
            return '' as FF
        }
    ",
    );

    tester
        .has_symbol("AB")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .test();

    tester
        .has_symbol("CC")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .test();

    tester
        .has_symbol("DD")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(2)
        .test();

    tester
        .has_symbol("FF")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .test();
}

#[test]
fn test_type_parameter_name_same_with_other_symbols() {
    SemanticTester::ts(
        "
        interface CC {
        }
        class A<CC> {
          a: CC;
        }
    ",
    )
    .has_symbol("CC")
    .test();
}
