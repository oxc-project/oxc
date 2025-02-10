use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn unambiguous_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This module could be mistakenly parsed as script instead of module")
        .with_help("Add at least one import or export statement to unambiguously mark this file as a module")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct Unambiguous;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Warn if a `module` could be mistakenly parsed as a `script` and not pure ESM module
    ///
    /// ### Why is this bad?
    ///
    /// For ESM-only environments helps to determine files that not pure ESM modules
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function x() {}
    ///
    /// (function x() { return 42 })()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import 'foo'
    /// function x() { return 42 }
    ///
    /// export function x() { return 42 }
    ///
    /// (function x() { return 42 })()
    /// export {} // simple way to mark side-effects-only file as 'module' without any imports/exports
    /// ```
    Unambiguous,
    import,
    restriction
);

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/unambiguous.md>
impl Rule for Unambiguous {
    fn run_once(&self, ctx: &LintContext<'_>) {
        if !ctx.module_record().has_module_syntax {
            ctx.diagnostic(unambiguous_diagnostic(Span::default()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // looks like in original rule that should pass for ecmaVersion before 2015
        // r"function x() {}",
        // r#""use strict"; function y() {}"#,
        r#"import y from "z"; function x() {}"#,
        r#"import * as y from "z"; function x() {}"#,
        r#"import { y } from "z"; function x() {}"#,
        r#"import z, { y } from "z"; function x() {}"#,
        "function x() {}; export {}",
        "function x() {}; export { x }",
        r#"function x() {}; export { y } from "z""#,
        r#"function x() {}; export * as y from "z""#,
        "export function x() {}",
    ];

    let fail = vec![r"function x() {}", r"(function x() { return 42 })()"];

    Tester::new(Unambiguous::NAME, Unambiguous::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
