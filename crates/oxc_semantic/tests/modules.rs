mod util;

use oxc_semantic::SymbolFlags;
use oxc_syntax::module_record::ExportExportName;
pub use util::SemanticTester;

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

    // FIXME: failing
    // test.has_some_symbol("defaultExport").is_exported().test();
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

    {
        let foo_test = test.has_some_symbol("Foo");
        let (semantic, _) = foo_test.inner();
        let m = semantic.module_record();
        let local_default_entry = m
            .local_export_entries
            .iter()
            .find(|export| matches!(export.export_name, ExportExportName::Default(_)))
            .unwrap();
        assert!(local_default_entry.local_name.name().is_some_and(|name| name == &"Foo"));
        assert!(!m.exported_bindings.contains_key("Foo"));
        assert!(m.export_default.is_some());
        foo_test.contains_flags(SymbolFlags::Export).test();
    }
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
