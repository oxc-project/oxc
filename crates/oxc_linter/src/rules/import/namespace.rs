use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactString, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(namespace): ")]
#[diagnostic(severity(warning), help(""))]
struct NamespaceDiagnostic(CompactString, #[label] pub Span);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/namespace.md>
#[derive(Debug, Default, Clone)]
pub struct Namespace;

declare_oxc_lint!(
    /// ### What it does
    /// TODO
    Namespace,
    nursery
);

impl Rule for Namespace {
    fn run_once(&self, _ctx: &LintContext<'_>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import "./malformed.js""#,
        // r#"import * as foo from './empty-folder';"#,
        // r#"import * as names from "./named-exports"; console.log((names.b).c);"#,
        // r#"import * as names from "./named-exports"; console.log(names.a);"#,
        // r#"import * as names from "./re-export-names"; console.log(names.foo);"#,
        // r#"import * as elements from './jsx';"#,
        // r#"import * as foo from "./jsx/re-export.js";
        // console.log(foo.jsxFoo);"#,
        // r#"import * as foo from "./jsx/bar/index.js";
        // console.log(foo.Baz1);
        // console.log(foo.Baz2);
        // console.log(foo.Qux1);
        // console.log(foo.Qux2);"#,
        // r#"import * as foo from './common';"#,
        // r#"import * as names from "./named-exports"; const { a } = names"#,
        // r#"import * as names from "./named-exports"; const { d: c } = names"#,
        // r#"import * as names from "./named-exports";
        // const { c } = foo,
        // { length } = "names",
        // alt = names;"#,
        // r#"import * as names from "./named-exports"; const { ExportedClass: { length } } = names"#,
        // r#"import * as names from "./named-exports"; function b(names) { const { c } = names }"#,
        // r#"import * as names from "./named-exports"; function b() { let names = null; const { c } = names }"#,
        // r#"import * as names from "./named-exports"; const x = function names() { const { c } = names }"#,
        // r#"export * as names from "./named-exports""#,
        // r#"export defport, * as names from "./named-exports""#,
        // r#"export * as names from "./does-not-exist""#,
        // r#"import * as Endpoints from "./issue-195/Endpoints"; console.log(Endpoints.Users)"#,
        // r#"function x() { console.log((names.b).c); } import * as names from "./named-exports";"#,
        // r#"import * as names from './default-export';"#,
        // r#"import * as names from './default-export'; console.log(names.default)"#,
        // r#"export * as names from "./default-export""#,
        // r#"export defport, * as names from "./default-export""#,
        // r#"import * as names from './named-exports'; console.log(names['a']);"#,
        // r#"import * as names from './named-exports'; const {a, b, ...rest} = names;"#,
        // r#"import * as names from './named-exports'; const {a, b, ...rest} = names;"#,
        // r#"import * as ns from './re-export-common'; const {foo} = ns;"#,
        // r#"import * as Names from "./named-exports"; const Foo = <Names.a/>"#,
        // r#"import * as foo from "./typescript-declare-nested"
        // foo.bar.MyFunction()"#,
        // r#"import { foobar } from "./typescript-declare-interface""#,
        // r#"export * from "typescript/lib/typescript.d""#,
        // r#"export = function name() {}"#,
        // r#"for (let { foo, bar } of baz) {}"#,
        // r#"for (let [ foo, bar ] of baz) {}"#,
        // r#"const { x, y } = bar"#,
        // r#"const { x, y, ...z } = bar"#,
        // r#"let x; export { x }"#,
        // r#"let x; export { x as y }"#,
        // r#"export const x = null"#,
        // r#"export var x = null"#,
        // r#"export let x = null"#,
        // r#"export default x"#,
        // r#"export default class x {}"#,
        // r#"import json from "./data.json""#,
        // r#"import foo from "./foobar.json";"#,
        // r#"import foo from "./foobar";"#,
        // r#"import { foo } from "./issue-370-commonjs-namespace/bar""#,
        // r#"export * from "./issue-370-commonjs-namespace/bar""#,
        // r#"import * as a from "./commonjs-namespace/a"; a.b"#,
        // r#"import { foo } from "./ignore.invalid.extension""#,
        // r#"import * as color from './color';
        // export const getBackgroundFromColor = (color) => color.bg;
        // export const getExampleColor = () => color.example"#,
        // r#"import * as middle from './middle';

        // console.log(middle.myName);"#,
        // r#"import * as names from './default-export-string';"#,
        // r#"import * as names from './default-export-string'; console.log(names.default)"#,
        // r#"import * as names from './default-export-namespace-string';"#,
        // r#"import * as names from './default-export-namespace-string'; console.log(names.default)"#,
        // r#"import { "b" as b } from "./deep/a"; console.log(b.c.d.e)"#,
        // r#"import { "b" as b } from "./deep/a"; var {c:{d:{e}}} = b"#,
        // r#"import * as a from "./deep/a"; console.log(a.b.c.d.e)"#,
        // r#"import { b } from "./deep/a"; console.log(b.c.d.e)"#,
        // r#"import * as a from "./deep/a"; console.log(a.b.c.d.e.f)"#,
        // r#"import * as a from "./deep/a"; var {b:{c:{d:{e}}}} = a"#,
        // r#"import { b } from "./deep/a"; var {c:{d:{e}}} = b"#,
        // r#"import * as a from "./deep-es7/a"; console.log(a.b.c.d.e)"#,
        // r#"import { b } from "./deep-es7/a"; console.log(b.c.d.e)"#,
        // r#"import * as a from "./deep-es7/a"; console.log(a.b.c.d.e.f)"#,
        // r#"import * as a from "./deep-es7/a"; var {b:{c:{d:{e}}}} = a"#,
        // r#"import { b } from "./deep-es7/a"; var {c:{d:{e}}} = b"#,
    ];

    let fail = vec![
        // r#"import * as names from './named-exports'; console.log(names.c)"#,
        // r#"import * as names from './named-exports'; console.log(names['a']);"#,
        // r#"import * as foo from './bar'; foo.foo = 'y';"#,
        // r#"import * as foo from './bar'; foo.x = 'y';"#,
        // r#"import * as names from "./named-exports"; const { c } = names"#,
        // r#"import * as names from "./named-exports"; function b() { const { c } = names }"#,
        // r#"import * as names from "./named-exports"; const { c: d } = names"#,
        // r#"import * as names from "./named-exports"; const { c: { d } } = names"#,
        // r#"import * as Endpoints from "./issue-195/Endpoints"; console.log(Endpoints.Foo)"#,
        // r#"import * as namespace from './malformed.js';"#,
        // r#"import b from './deep/default'; console.log(b.e)"#,
        // r#"console.log(names.c); import * as names from './named-exports';"#,
        // r#"function x() { console.log(names.c) } import * as names from './named-exports';"#,
        // r#"import * as ree from "./re-export"; console.log(ree.default)"#,
        // r#"import * as Names from "./named-exports"; const Foo = <Names.e/>"#,
        // r#"import { "b" as b } from "./deep/a"; console.log(b.e)"#,
        // r#"import { "b" as b } from "./deep/a"; console.log(b.c.e)"#,
        // r#"import * as a from "./deep/a"; console.log(a.b.e)"#,
        // r#"import { b } from "./deep/a"; console.log(b.e)"#,
        // r#"import * as a from "./deep/a"; console.log(a.b.c.e)"#,
        // r#"import { b } from "./deep/a"; console.log(b.c.e)"#,
        // r#"import * as a from "./deep/a"; var {b:{ e }} = a"#,
        // r#"import * as a from "./deep/a"; var {b:{c:{ e }}} = a"#,
        // r#"import * as a from "./deep-es7/a"; console.log(a.b.e)"#,
        // r#"import { b } from "./deep-es7/a"; console.log(b.e)"#,
        // r#"import * as a from "./deep-es7/a"; console.log(a.b.c.e)"#,
        // r#"import { b } from "./deep-es7/a"; console.log(b.c.e)"#,
        // r#"import * as a from "./deep-es7/a"; var {b:{ e }} = a"#,
        // r#"import * as a from "./deep-es7/a"; var {b:{c:{ e }}} = a"#,
    ];

    Tester::new(Namespace::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
