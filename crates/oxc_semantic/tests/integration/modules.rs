use oxc_semantic::SymbolFlags;

use crate::util::SemanticTester;

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
fn test_export_flag() {
    let code = [
        "export let foo, bar",
        "export const foo = 1, bar = 2",
        "export function foo() { } export function bar() { }",
        "export class foo { } export class bar { }",
        "export const { foo, bar } = o;",
        "export const [ foo, bar ] = array;",
        "var foo, bar; export { foo, bar };",
        "var foo, bar; export { foo as name1, bar as name2 };",
        "var foo, bar; export { foo as default, bar as default2 };",
        "var foo, bar; export default foo; export { bar }",
        "export default function foo() { } export function bar() { }",
        "export default class foo { } export class bar { }",
        "export default function* foo() { } export function* bar() { }",
    ];

    for c in code {
        let test = SemanticTester::js(c);
        test.has_symbol("foo").contains_flags(SymbolFlags::Export).test();
        test.has_symbol("bar").contains_flags(SymbolFlags::Export).test();
    }
}
