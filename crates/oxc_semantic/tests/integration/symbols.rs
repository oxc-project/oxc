use oxc_semantic::{Reference, SymbolFlags};

use crate::util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js("export class Foo {};")
        .has_root_symbol("Foo")
        .contains_flags(SymbolFlags::Class)
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

    SemanticTester::js("let x = 0; let foo = (0, x++)")
        .has_some_symbol("x")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();

    SemanticTester::js("let x = 0; x++")
        .has_some_symbol("x")
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

    SemanticTester::ts("function foo<T>(): T { }")
        .has_some_symbol("T")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .has_number_of_references_where(1, Reference::is_type)
        .test();

    SemanticTester::ts("function foo<T>(a: T): void {}")
        .has_some_symbol("T")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .has_number_of_references_where(1, Reference::is_type)
        .test();
}

#[test]
fn test_multiple_ts_type_alias_declaration() {
    let tester = SemanticTester::ts(
        "
        type A<AB> = AB
        type B<AB> = AB
    ",
    );

    tester
        .has_symbol("AB")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .test();
}

#[test]
fn test_function_with_type_parameter() {
    let tester = SemanticTester::ts(
        "
        function mdl<M>() {
            function mdl2() {
                function mdl3() {
                    return '' as M;
                }
            }
        }
        function kfc<K>() {
            function kfc2<K>() {
                function kfc3<K>() {
                    return '' as K;
                }
            }
        }
        ",
    );

    tester.has_symbol("M").has_number_of_references(1).test();
    tester.has_symbol("K").has_number_of_references(0).test();
}

#[test]
fn test_class_with_type_parameter() {
    let tester = SemanticTester::ts(
        "
        class Cls<T> {
            a: T;
            method<K>(b: T | K) {}
            func() {
                type D = T;
                return null as D
            }
        }
        type B = any;
        class ClassB<B> {
            b: B;
        }
        ",
    );

    tester.has_symbol("T").has_number_of_references(3).test();
    tester.has_symbol("K").has_number_of_references(1).test();
    tester.has_symbol("D").has_number_of_references(1).test();

    // type B is not referenced
    tester.has_symbol("B").has_number_of_references(0).test();
}

#[test]
fn test_class_with_accessor() {
    SemanticTester::ts(
        "
    type T = 1;

    abstract class Foo {
        accessor prop: T;
    }
    ",
    )
    .has_some_symbol("T")
    .has_number_of_references(1)
    .has_number_of_references_where(1, Reference::is_type)
    .test();
}

#[test]
fn test_ts_mapped_type() {
    let tester = SemanticTester::ts(
        "
        type M<T> = { [K in keyof T]: T[K] };
        type Y = any;
        type X<T> = { [Y in keyof T]: T[Y] };
        ",
    );

    tester.has_symbol("T").has_number_of_references(2).test();
    tester.has_symbol("K").has_number_of_references(1).test();

    // type Y is not referenced
    tester.has_symbol("Y").has_number_of_references(0).test();
}

#[test]
fn test_ts_interface_declaration_with_type_parameter() {
    let tester = SemanticTester::ts(
        "
        type A = any;
        interface ITA<A> {
            a: A;
        }
        interface ITB<B> {
            b: B;
        }
        ",
    );

    tester.has_symbol("B").has_number_of_references(1).test();

    // type A is not referenced
    tester.has_symbol("A").has_number_of_references(0).test();
}

#[test]
fn test_ts_infer_type() {
    let tester = SemanticTester::ts(
        "
        type T = T extends infer U ? U : never;

        type C = any;
        type K = K extends infer C ? K : never;
        ",
    );

    tester.has_symbol("T").has_number_of_references(1).test();
    tester.has_symbol("U").has_number_of_references(1).test();

    // type C is not referenced
    tester.has_symbol("C").has_number_of_references(0).test();
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
    // a value context, the root `const T` should be the symbol recorded in the
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
}

#[test]
fn test_type_used_as_value() {
    SemanticTester::ts(
        "
    type T = number;
    let x = T;
    ",
    )
    .has_some_symbol("T")
    .has_number_of_reads(0)
    .test();
}

#[test]
fn test_type_query() {
    SemanticTester::ts(
        "
    type T = number;
    let x: typeof T;
    ",
    )
    .has_some_symbol("T")
    .has_number_of_reads(0)
    .test();
}

#[test]
fn test_ts_interface_heritage() {
    // NOTE: interface heritage clauses can only be identifiers or qualified
    // names, but we handle references on invalid heritage clauses anyways.
    SemanticTester::ts(
        "
        type Heritage = { x: number; y: string; };
        interface A extends (Heritage.x) {}
    ",
    )
    .expect_errors(true)
    .has_some_symbol("Heritage")
    .has_number_of_references(1)
    .test();
}

#[test]
fn test_arrow_implicit_return() {
    SemanticTester::js("let i = 0; const x = () => i")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(0)
        .test();

    SemanticTester::js("let i = 0; const x = () => ++i")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();

    SemanticTester::js("let i = 0; const x = () => { ++i }")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();

    SemanticTester::js("let i = 0; const x = () => (0, ++i)")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();

    SemanticTester::js("let i = 0; const x = () => (++i, 0)")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();

    SemanticTester::js("let i = 1; const foo = () => () => { i++ }")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();
}

#[test]
fn test_arrow_explicit_return() {
    SemanticTester::js("let i = 0; const x = () => { return i }")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(0)
        .test();

    SemanticTester::js("let i = 0; const x = () => { return ++i }")
        .has_root_symbol("i")
        .has_number_of_reads(1)
        .has_number_of_writes(1)
        .test();
}

#[test]
fn test_tagged_templates() {
    // https://github.com/oxc-project/oxc/issues/5391
    SemanticTester::tsx(
        "
        import styled from 'styled-components';

        import { Prose, ProseProps } from './prose';

        interface Props extends ProseProps {
          density?: number;
        }
        export const HandMarkedPaperBallotProse = styled(Prose)<Props>`
          line-height: ${({ density }) => (density !== 0 ? '1.1' : '1.3')};
        `;
    ",
    )
    .has_some_symbol("density")
    .has_number_of_reads(1)
    .has_number_of_writes(0)
    .test();
}

#[test]
fn test_module_like_declarations() {
    SemanticTester::ts("namespace A { export const x = 1; }")
        .has_root_symbol("A")
        .contains_flags(SymbolFlags::NameSpaceModule)
        .test();

    SemanticTester::ts("module A { export const x = 1; }")
        .has_root_symbol("A")
        .contains_flags(SymbolFlags::NameSpaceModule)
        .test();

    SemanticTester::ts(r#"module "A" { export const x = 1; }"#)
        .has_root_symbol("A")
        .contains_flags(SymbolFlags::NameSpaceModule)
        .test();

    let test = SemanticTester::ts("declare global { interface Window { x: number; } }");
    let semantic = test.build();
    let global = semantic.symbols().names().find(|name| *name == "global");
    assert!(
        global.is_none(),
        "A symbol should not be created for global augmentation declarations."
    );
}

#[test]
fn test_class_merging() {
    // classes can be merged with interfaces, resulting in a single symbol
    SemanticTester::ts(
        "
        class Foo {}
        interface Foo {}
    ",
    )
    .has_root_symbol("Foo")
    .contains_flags(SymbolFlags::Class)
    .contains_flags(SymbolFlags::Interface)
    .test();
}
