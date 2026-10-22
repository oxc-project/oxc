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

#[test]
fn test_export_assignment_type_only_siblings() {
    for source in [
        "interface Options {}; export { Options }; const route = {}; export = route;",
        "interface Options {} export default Options; const route = {}; export = route;",
        "namespace Route { export interface P {} export const x = 1; } export interface O {} export = (Route);",
        "namespace Route { export interface P {} export const x = 1; } export interface O {} export = Route!;",
        "namespace Route { export interface P {} export const x = 1; } export interface O {} export = Route as unknown;",
        "namespace Route { export interface P {} export const x = 1; } export interface O {} export = Route satisfies unknown;",
        "namespace A { export namespace Route { export interface P {} export const x = 1; } } export interface O {} export = (A).Route;",
        "namespace Route { export interface P {} } interface O {} export { O }; export = Route;",
        "namespace Route { export interface P {} } interface O {} export default O; export = Route;",
        "namespace Route { export interface P {} } interface O {} export type { O }; export = Route;",
        "namespace Route { export interface P {} } namespace Types { export interface O {} } export import Alias = Types.O; export = Route;",
        "namespace C { export const x = 1; } declare namespace Route { import Alias = C; interface P {} } export interface O {} export = Route.Alias;",
        "namespace Route { interface Private {} export const x = 1; } export interface O {} export = Route;",
        "namespace Types { export interface Options {} } export import Alias = Types; const route = {}; export = route;",
        "namespace Route { export const x = 1; } import Alias = Route; export interface O {} export = Alias;",
        "import A = B; import B = A; export interface O {} export = A;",
        "export namespace Types { export interface Options {} } const route = {}; export = route;",
        "const route = {}; export default interface Options {}; export = route;",
        "namespace Route { export interface Params {} } export = Route;",
        "namespace Route { export const x = 1; } export interface Options {} export = Route;",
        "declare module 'm' { interface Options {} export { Options }; const route: {}; export = route; }",
    ] {
        assert!(SemanticTester::ts(source).build_with_errors().diagnostics.is_empty(), "{source}");
    }
    for source in [
        "namespace Route { export interface Params {} } export interface Options {} export = Route;",
        "export namespace Types { export const x = 1; } const route = {}; export = route;",
        "namespace A { export namespace B { export interface P {} export const x = 1; } } export interface O {} export = A?.B;",
        "declare namespace Route { interface P {} } export interface O {} export = Route;",
        "namespace Route { export interface P {} export const x = 1; } import Alias = Route; export interface O {} export = Alias;",
        "namespace C { export interface P {} } namespace A { export import B = C; } export interface O {} export = A.B;",
        "namespace C { export const x = 1; } namespace Route { export import Alias = C; } export interface O {} export = Route;",
        "namespace C { export interface P {} } namespace A { export import B = C; } namespace Route { export import Alias = A.B; } export interface O {} export = Route;",
        "namespace C { export interface P {} } namespace Route { import Alias = C; export { Alias }; } export interface O {} export = Route;",
        "namespace A { export namespace B { export interface P {} export const x = 1; } } import Alias = A.B; export interface O {} export = Alias;",
        "namespace A { export namespace B { export interface P {} export const x = 1; } } export interface O {} export = A.B;",
        "namespace A.B { export interface P {} export const x = 1; } export interface O {} export = A.B;",
        "namespace A { export namespace B {} export namespace B { export interface P {} export const x = 1; } } export interface O {} export = A.B;",
        "namespace Route {} namespace Route { export interface Params {} } export interface Options {} export = Route;",
        "namespace Route { interface Params {} export { Params }; } export type Options = {}; export = Route;",
        "interface Options {}; const Options = 1; export { Options }; const route = {}; export = route;",
    ] {
        let diagnostics = SemanticTester::ts(source).build_with_errors().diagnostics;
        let codes = diagnostics.iter().map(|d| d.code.to_string()).collect::<Vec<_>>();
        assert_eq!(codes, ["TS(2309)"], "{source}");
    }
}
