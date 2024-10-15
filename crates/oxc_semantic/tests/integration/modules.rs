use oxc_semantic::{SemanticBuilderReturn, SymbolFlags};

use crate::util::SemanticTester;

#[test]
fn test_exports() {
    let test = SemanticTester::js(
        "
        function foo(a, b) {
            let c = a + b;
            return c / 2
        }

        export class ExportModifier {
            constructor(x) {
                this.x = x;
            }
        }

        const defaultExport = 1;

        export { foo };
        export default defaultExport;
        ",
    );

    test.has_some_symbol("foo").is_exported().test();
    test.has_some_symbol("defaultExport").is_exported().test();
}

#[test]
fn test_exported_named_function() {
    let test = SemanticTester::js(
        "
    export function foo(a) {
        let x = 1;
    }
    ",
    );
    test.has_some_symbol("foo").is_exported().test();
    for name in &["a", "x"] {
        test.has_some_symbol(name).is_not_exported().test();
    }

    SemanticTester::ts("export function foo<T>(a: T) { a.length }")
        .has_some_symbol("T")
        .is_not_exported()
        .test();

    SemanticTester::tsx(
        "
    import React from 'react';
    export const Counter: React.FC<{ count: number }> = ({ count }) => (
        <div>{count}</div>
    )
    ",
    )
    .has_some_symbol("Counter")
    .is_exported()
    .contains_flags(
        SymbolFlags::ConstVariable
            .union(SymbolFlags::BlockScopedVariable)
            .union(SymbolFlags::Export),
    )
    .test();
}

#[test]
fn test_exported_default_function() {
    let test = SemanticTester::js(
        "
    export default function foo(a) {
        let x = 1;
    }
    ",
    );
    for name in &["a", "x"] {
        test.has_some_symbol(name).is_not_exported().test();
    }

    let test = SemanticTester::ts("export default function <T extends string>(a: T) { a.length }");
    test.has_some_symbol("a").is_not_exported().test();
    test.has_some_symbol("T").is_not_exported().test();
}

#[test]
fn test_exported_named_class() {
    let test = SemanticTester::ts(
        "
    export class Foo<T> {
        constructor(a) {
            this.a = a;
        }

        bar() {
            return this.a;
        }
    }
    ",
    );

    test.has_class("Foo");
    test.has_some_symbol("Foo").is_exported().test();
    // NOTE: bar() is not a symbol. Should it be?
    for name in &["a", "T"] {
        test.has_some_symbol(name).is_not_exported().test();
    }

    SemanticTester::ts(
        "
    class Foo {};
    export { Foo }
    ",
    )
    .has_some_symbol("Foo")
    .is_exported()
    .test();
}

#[test]
fn test_exported_default_class() {
    let test = SemanticTester::ts(
        "
    export default class Foo<T> {
        constructor(a) {
            this.a = a;
        }
    }
    ",
    );

    test.has_class("Foo");
    test.has_some_symbol("a").is_not_exported().test();
    test.has_some_symbol("T").is_not_exported().test();
}

// FIXME
#[test]
#[ignore]
fn test_exported_enum() {
    let test = SemanticTester::ts(
        "
        export enum Foo {
            A = 1,
            B,
        }
    ",
    );
    test.has_some_symbol("Foo").is_exported().contains_flags(SymbolFlags::RegularEnum).test();
    test.has_some_symbol("A").is_not_exported().contains_flags(SymbolFlags::EnumMember).test();
    test.has_some_symbol("B").is_not_exported().contains_flags(SymbolFlags::EnumMember).test();
}

// FIXME
#[test]
#[ignore]
fn test_exported_interface() {
    let test = SemanticTester::ts(
        "
        export interface Foo<T> {
            a: T;
        }
        ",
    );
    test.has_root_symbol("Foo").is_exported().contains_flags(SymbolFlags::Interface).test();
    test.has_some_symbol("a").is_not_exported().test();
    test.has_some_symbol("T").is_not_exported().test();
}

#[test]
fn test_exports_in_namespace() {
    let test = SemanticTester::ts(
        "
    export const x = 1;
    namespace N {
        function foo() {
            return 1
        }
        export function bar() {
            return foo();
        }
        export const x = 2
    }
    ",
    );
    test.has_some_symbol("bar").is_exported().test();
    let semantic = test.build();
    assert!(!semantic.module_record().exported_bindings.contains_key("bar"));

    // namespace exported, member is not
    let sources =
        ["export namespace N { function foo() {} } ", "export namespace N { const foo = 1 } "];
    for src in sources {
        let test = SemanticTester::ts(src);
        test.has_some_symbol("N").contains_flags(SymbolFlags::NameSpaceModule).is_exported().test();
        test.has_some_symbol("foo").is_not_exported().test();
    }

    // namespace and member are both exported
    let sources = [
        "export namespace N { export function foo() {} } ",
        "export namespace N { export const foo = 1 } ",
    ];
    for src in sources {
        let test = SemanticTester::ts(src);
        test.has_some_symbol("N").contains_flags(SymbolFlags::NameSpaceModule).is_exported().test();
        test.has_some_symbol("foo").is_exported().test();
    }

    // namespace is not exported, but member is
    let sources =
        ["namespace N { export function foo() {} } ", "namespace N { export const foo = 1 } "];
    for src in sources {
        let test = SemanticTester::ts(src);
        test.has_some_symbol("N")
            .contains_flags(SymbolFlags::NameSpaceModule)
            .is_not_exported()
            .test();
        test.has_some_symbol("foo").is_exported().test();
    }
}

#[test]
fn test_export_in_invalid_scope() {
    let test = SemanticTester::js(
        "
    function foo() {
        export const x = 1;
    }",
    )
    .expect_errors(true);
    test.has_some_symbol("x").contains_flags(SymbolFlags::Export).test();
    let SemanticBuilderReturn { semantic, errors } = test.build_with_errors();
    assert!(
        !errors.is_empty(),
        "expected an export within a function to produce a check error, but no errors were produced"
    );
    assert!(semantic.module_record().exported_bindings.is_empty());
}

#[test]
fn test_import_assignment() {
    SemanticTester::ts("import Foo = require('./foo')")
        .has_root_symbol("Foo")
        .contains_flags(SymbolFlags::Import)
        .test();

    SemanticTester::ts("import { Foo } from './foo'; import Baz = Foo.Bar.Baz")
        .has_root_symbol("Baz")
        .contains_flags(SymbolFlags::Import)
        .test();
}

#[test]
fn test_import_type() {
    SemanticTester::ts(r#"import { type "<A>" as someA } from './a'; "#)
        .has_root_symbol("someA")
        .contains_flags(SymbolFlags::TypeImport)
        .test();
}
