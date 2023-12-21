mod util;

use oxc_semantic::SymbolFlags;
use util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js("export class Foo {}")
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

#[ignore = "type aliases currently aren't in the symbol table"]
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
fn test_property_definition() {
    let tester = SemanticTester::js(
        "
        class Foo {
            #baz = 42;
            test() {
                this.#baz = 44;
                this.#baz += 1;
                console.log(this.#baz + 1);
                console.log(this.#baz++);
                [this.#baz] = [1];
                [...this.#baz] = [1]
                [this.#baz = 1] = 1
            }
        }
        ",
    );

    tester
        .has_some_symbol("baz")
        .contains_flags(SymbolFlags::PropertyDefinition)
        .has_number_of_references(7)
        .has_number_of_reads(3)
        .has_number_of_writes(6)
        .test();
}

#[test]
fn test_method_definition() {
    let tester = SemanticTester::js(
        "
        class Foo {
            set #bar(a) {}
            get #bar() {}
            get #bae() {}
            #baz() {}
            test() {
                this.#bar = 43;
                console.log(this.#bar)

                console.log(this.#bae)

                let a = this.#baz();
            }
        }
        ",
    );

    tester
        .has_some_symbol("baz")
        .contains_flags(SymbolFlags::ClassMethod)
        .has_number_of_references(1)
        .has_number_of_reads(1)
        .test();

    tester
        .has_some_symbol("bar")
        .contains_flags(SymbolFlags::ClassGetAccessor | SymbolFlags::ClassSetAccessor)
        .has_number_of_references(2)
        .has_number_of_writes(1)
        .has_number_of_reads(1)
        .test();

    tester
        .has_some_symbol("bae")
        .contains_flags(SymbolFlags::ClassGetAccessor)
        .has_number_of_references(1)
        .has_number_of_reads(1)
        .test();
}
