use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
// use oxc_span::{CompactStr, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
};

// #[derive(Debug, Error, Diagnostic)]
// #[error("")]
// #[diagnostic(severity(warning), help(""))]
// struct NoDeprecatedDiagnostic(CompactStr, #[label] pub Span);

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-deprecated.md>
#[derive(Debug, Default, Clone)]
pub struct NoDeprecated;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports use of a deprecated name, as indicated by a JSDoc block with
    /// a @deprecated tag or TomDoc Deprecated: comment.
    ///
    /// ### Why is this bad?
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// ```
    NoDeprecated,
    import,
    nursery
);

impl Rule for NoDeprecated {
    fn run_once(&self, _ctx: &LintContext<'_>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"import { x } from './fake'",
        r"import bar from './bar'",
        r"import { fine } from './deprecated'",
        r"import { _undocumented } from './deprecated'",
        r"import { fn } from './deprecated'",
        r"import { fine } from './tomdoc-deprecated'",
        r"import { _undocumented } from './tomdoc-deprecated'",
        r"import * as depd from './deprecated'",
        r"import * as depd from './deprecated'; console.log(depd.fine())",
        r"import { deepDep } from './deep-deprecated'",
        r"import { deepDep } from './deep-deprecated'; console.log(deepDep.fine())",
        r"import { deepDep } from './deep-deprecated'; function x(deepDep) { console.log(deepDep.MY_TERRIBLE_ACTION) }",
        r"for (let { foo, bar } of baz) {}",
        r"for (let [ foo, bar ] of baz) {}",
        r"const { x, y } = bar",
        r"const { x, y, ...z } = bar",
        r"let x; export { x }",
        r"let x; export { x as y }",
        r"export const x = null",
        r"export var x = null",
        r"export let x = null",
        r"export default x",
        r"export default class x {}",
        r#"import json from "./data.json""#,
        r#"import foo from "./foobar.json";"#,
        r#"import foo from "./foobar";"#,
        r#"import { foo } from "./issue-370-commonjs-namespace/bar""#,
        r#"export * from "./issue-370-commonjs-namespace/bar""#,
        r#"import * as a from "./commonjs-namespace/a"; a.b"#,
        r#"import { foo } from "./ignore.invalid.extension""#,
        // hoisting
        r#"function x(deepDep) { console.log(deepDep.MY_TERRIBLE_ACTION) } import { deepDep } from "./deep-deprecated""#,
        // TypeScript
        r#"import * as hasDeprecated from "./ts-deprecated.ts""#,
    ];

    let fail = vec![
        // r#"import './malformed.js'"#,
        // r#"import { fn } from './deprecated'"#,
        // r#"import TerribleClass from './deprecated'"#,
        // r#"import { MY_TERRIBLE_ACTION } from './deprecated'"#,
        // r#"import { fn } from './deprecated'"#,
        // r#"import { fn } from './tomdoc-deprecated'"#,
        // r#"import TerribleClass from './tomdoc-deprecated'"#,
        // r#"import { MY_TERRIBLE_ACTION } from './tomdoc-deprecated'"#,
        // r#"import { MY_TERRIBLE_ACTION } from './deprecated'; function shadow(MY_TERRIBLE_ACTION) { console.log(MY_TERRIBLE_ACTION); }"#,
        // r#"import { MY_TERRIBLE_ACTION, fine } from './deprecated'; console.log(fine)"#,
        // r#"import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(MY_TERRIBLE_ACTION)"#,
        // r#"import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(someOther.MY_TERRIBLE_ACTION)"#,
        // r#"import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(MY_TERRIBLE_ACTION.whatever())"#,
        // r#"import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(MY_TERRIBLE_ACTION(this, is, the, worst))"#,
        // r#"import Thing from './deprecated-file'"#,
        // r#"import Thing from './deprecated-file'; console.log(other.Thing)"#,
        // r#"import * as depd from './deprecated'; console.log(depd.MY_TERRIBLE_ACTION)"#,
        // r#"import * as deep from './deep-deprecated'; console.log(deep.deepDep.MY_TERRIBLE_ACTION)"#,
        // r#"import { deepDep } from './deep-deprecated'; console.log(deepDep.MY_TERRIBLE_ACTION)"#,
        // r#"import { deepDep } from './deep-deprecated'; function x(deepNDep) { console.log(deepDep.MY_TERRIBLE_ACTION) }"#,
        // // hoisting
        // r#"console.log(MY_TERRIBLE_ACTION); import { MY_TERRIBLE_ACTION } from "./deprecated""#,
        // // TypeScript
        // r#"import { foo } from "./ts-deprecated.ts"; console.log(foo())"#,
    ];

    Tester::new(NoDeprecated::NAME, NoDeprecated::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
