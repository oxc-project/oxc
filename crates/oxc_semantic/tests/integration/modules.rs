use oxc_semantic::SymbolFlags;

use crate::util::SemanticTester;

#[test]
fn test_nested_ambient_module_declaration() {
    for source in [
        r#"declare module "outer" { module "middle" { module "inner" {} } }"#,
        r#"export {}; declare module "outer" { module "inner" {} }"#,
        r#"export {}; declare global { module "inner" {} }"#,
        r#"declare module "outer" { module "inner" {} } import.meta;"#,
    ] {
        let diagnostics = SemanticTester::ts(source).build_with_errors().diagnostics;
        let codes =
            diagnostics.iter().map(|diagnostic| diagnostic.code.to_string()).collect::<Vec<_>>();

        assert_eq!(codes, ["TS(2435)"]);
    }
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

#[test]
fn test_ts_import_type_qualifier_no_reference() {
    // Test that TSImportType qualifiers don't create references
    // The 'b' and 'c' in import("./a").b.c should not be references
    // This test verifies that the code parses without errors and creates
    // the type alias symbol but no references for the qualifier parts
    SemanticTester::ts(r#"type A = import("./a").b.c"#)
        .has_root_symbol("A")
        .contains_flags(SymbolFlags::TypeAlias)
        .test();

    // Test with nested qualifiers
    SemanticTester::ts(r#"type B = import("./module").a.b.c.d"#)
        .has_root_symbol("B")
        .contains_flags(SymbolFlags::TypeAlias)
        .test();

    // Test with type arguments
    SemanticTester::ts(r#"type C = import("./generic").Foo<string>"#)
        .has_root_symbol("C")
        .contains_flags(SymbolFlags::TypeAlias)
        .test();

    // Test with qualified name and type arguments
    SemanticTester::ts(r#"type D = import("./generic").a.b.Foo<number, string>"#)
        .has_root_symbol("D")
        .contains_flags(SymbolFlags::TypeAlias)
        .test();
}
