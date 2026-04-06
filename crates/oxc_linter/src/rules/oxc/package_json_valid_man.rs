use super::json_utils::{file_start_span, is_json_file};

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

static MANPAGE_REGEX: Lazy<Regex> = lazy_regex!(r"\.[0-9](?:\.gz)?$");

fn invalid_package_json_man_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `man` field in package.json is invalid.")
        .with_help(
            "Use a string or array of strings ending in a manpage section such as `.1` or `.1.gz`.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidMan;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `man` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid manpage metadata can prevent package managers from linking
    /// command documentation correctly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "man": ["./man/demo.md"] }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "man": "./man/demo.1" }
    /// ```
    PackageJsonValidMan,
    oxc,
    correctness
);

impl Rule for PackageJsonValidMan {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };
        let Some(man) = object.get("man") else {
            return;
        };

        if is_valid_man_value(man) {
            return;
        }

        ctx.diagnostic(invalid_package_json_man_diagnostic(file_start_span(source_text)));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn is_valid_man_value(value: &Value) -> bool {
    match value {
        Value::String(value) => is_valid_man_path(value),
        Value::Array(values) => !values.is_empty() && values.iter().all(is_valid_man_entry),
        _ => false,
    }
}

fn is_valid_man_entry(value: &Value) -> bool {
    matches!(value, Value::String(value) if is_valid_man_path(value))
}

fn is_valid_man_path(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && MANPAGE_REGEX.is_match(trimmed)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"man":"./man/demo.1"}"#,
        r#"{"man":"./man/demo.1.gz"}"#,
        r#"{"man":["./man/demo.1","./man/demo.2"]}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"man":""}"#,
        r#"{"man":"./man/demo.md"}"#,
        r#"{"man":[]}"#,
        r#"{"man":["./man/demo.1","./man/demo.md"]}"#,
        r#"{"man":1}"#,
    ];

    Tester::new(PackageJsonValidMan::NAME, PackageJsonValidMan::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
