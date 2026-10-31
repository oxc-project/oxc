use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct UnsupportedEslintDirective;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports ESLint and Oxlint configuration comments that Oxlint does not support.
    ///
    /// This rule checks block comments starting with `eslint`, `oxlint`, `eslint-env`,
    /// `oxlint-env`, `global`, `globals`, or `exported`. Rule enable/disable comments
    /// remain supported and are not reported. Line comments are not checked because
    /// ESLint does not recognize these configuration directives in line comments.
    ///
    /// ### Why is this bad?
    ///
    /// Unsupported directives have no effect in Oxlint. Relying on them can produce
    /// unexpected diagnostics when migrating from ESLint. Configure rule severities
    /// and options with `rules`, environments with `env`, and global variables with
    /// `globals` in the Oxlint configuration file. Use ES module exports instead of
    /// `exported` comments.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* eslint no-alert: "warn" */
    /// /* oxlint "no-console": "off" */
    /// /* eslint-env browser */
    /// /* global myGlobal */
    /// /* exported myFunction */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* oxlint-disable no-console */
    /// console.log("This rule is disabled.");
    /// /* oxlint-enable no-console */
    ///
    /// // eslint-disable-next-line no-alert
    /// alert("This line is allowed.");
    ///
    /// export function myFunction() {}
    /// ```
    UnsupportedEslintDirective,
    oxc,
    nursery,
    version = "next",
    short_description = "Reports ESLint and Oxlint configuration comments that Oxlint does not support.",
);

impl Rule for UnsupportedEslintDirective {
    fn run_once(&self, ctx: &LintContext) {
        for comment in ctx.comments() {
            if !comment.is_block() {
                continue;
            }

            let text = ctx.source_range(comment.content_span());
            let is_whitespace = |c| is_white_space(c) || is_line_terminator(c);
            let directive = text
                .trim_start_matches(is_whitespace)
                .split(is_whitespace)
                .next()
                .unwrap_or_default();

            let help = match directive {
                "eslint" | "oxlint" => {
                    "Configure rules in the `rules` field of your Oxlint configuration file."
                }
                "eslint-env" | "oxlint-env" => {
                    "Configure environments in the `env` field of your Oxlint configuration file."
                }
                "global" | "globals" => {
                    "Configure global variables in the `globals` field of your Oxlint configuration file."
                }
                "exported" => "Use ES module exports instead of an `exported` comment.",
                _ => continue,
            };

            ctx.diagnostic(
                OxcDiagnostic::warn(format!(
                    "The `{directive}` directive is not supported by Oxlint."
                ))
                .with_help(help)
                .with_label(comment.span),
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const value = 1;",
        "/**/",
        "/* ordinary comment */",
        "/* Configure eslint in the configuration file. */",
        "/** eslint no-alert: off */",
        "/* eslintConfig no-alert: off */",
        "/* oxlint-config no-alert: off */",
        "/* eslint-environment browser */",
        "/* globalThis.foo */",
        "/* globalsForTests */",
        "/* exportedValue */",
        "/* ESLint no-alert: off */",
        "// eslint no-alert: off",
        "// oxlint no-alert: off",
        "// eslint-env browser",
        "// oxlint-env node",
        "// global foo",
        "// globals foo",
        "// exported foo",
        r#"const text = "/* eslint no-alert: off */";"#,
        "/* eslint-disable */",
        "/* oxlint-disable */",
        "/* eslint-disable no-console */ console.log('hello');",
        "/* oxlint-disable no-console, typescript/no-floating-promises */",
        "/* eslint-disable no-console, @typescript-eslint/no-floating-promises */",
        "/* eslint-enable */",
        "/* oxlint-enable no-console, no-alert */",
        "console.log('hello'); // eslint-disable-line no-console",
        "console.log('hello'); /* oxlint-disable-line no-console */",
        "// oxlint-disable-next-line no-console\nconsole.log('hello');",
        "/* eslint-disable-next-line no-console */\nconsole.log('hello');",
        "/* eslint-disable oxc/unsupported-eslint-directive */\n/* eslint no-alert: off */",
        "// oxlint-disable-next-line oxc/unsupported-eslint-directive\n/* oxlint no-alert: off */",
    ];

    let fail = vec![
        r#"/* oxlint "example/rule1": "error" */"#,
        r#"/* oxlint "no-console":"error" */"#,
        "/* oxlint no-alert: warn */",
        r#"/* oxlint no-alert: "off" */"#,
        "/* oxlint no-alert:off */",
        r#"/* oxlint no-alert:"off" */"#,
        r#"/* oxlint "no-console":0 */"#,
        r#"/* oxlint "no-console":1 */"#,
        r#"/* oxlint "no-console":2 */"#,
        "/* oxlint no-console:0,no-alert:0 */",
        "/* oxlint @typescript-eslint/no-explicit-any:0, no-alert:0 */",
        r#"/* eslint "example/rule1": "error" */"#,
        r#"/* eslint "no-console":"error" */"#,
        "/* eslint no-alert: warn */",
        r#"/* eslint no-alert: "off" */"#,
        "/* eslint no-alert:off */",
        r#"/* eslint no-alert:"off" */"#,
        r#"/* eslint "no-console":0 */"#,
        r#"/* eslint "no-console":1 */"#,
        r#"/* eslint "no-console":2 */"#,
        "/* eslint no-console:0,no-alert:0 */",
        "/* eslint @typescript-eslint/no-explicit-any:0, no-alert:0 */",
        "/*eslint no-alert:off*/",
        "/*\t oxlint\tno-alert: off */",
        "/*\neslint\nno-alert: off\n*/",
        "/*\u{feff}eslint\u{00a0}no-alert: off */",
        r#"/* eslint quotes: ["error", "single"] -- explanation */"#,
        "/* eslint-env browser, node */",
        "/* oxlint-env node */",
        "/* global foo, bar:false */",
        "/* globals foo:writable, bar:readonly */",
        "/* exported foo */ var foo = 1;",
        "/* eslint */",
        "/* oxlint-disable no-console */\n/* eslint no-alert: off */",
    ];

    Tester::new(UnsupportedEslintDirective::NAME, UnsupportedEslintDirective::PLUGIN, pass, fail)
        .test_and_snapshot();
}
