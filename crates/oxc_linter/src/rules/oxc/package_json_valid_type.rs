use super::json_utils::{file_start_span, is_json_file};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn invalid_package_json_type_diagnostic(actual: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `type` field in package.json must be `module` or `commonjs`, not `{actual}`."
    ))
    .with_help("Use a valid Node package type or remove the field.")
    .with_label(span)
}

fn non_string_package_json_type_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `type` field in package.json must be a string.")
        .with_help("Use `module` or `commonjs` for the package type.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `type` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid package types break Node module resolution expectations and make
    /// package behavior harder to reason about.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "type": "esm" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "type": "module" }
    /// ```
    PackageJsonValidType,
    oxc,
    correctness
);

impl Rule for PackageJsonValidType {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(package_type) = value.get("type") else {
            return;
        };

        let span = file_start_span(source_text);
        match package_type {
            Value::String(kind) if kind == "module" || kind == "commonjs" => {}
            Value::String(kind) => ctx.diagnostic(invalid_package_json_type_diagnostic(kind, span)),
            _ => ctx.diagnostic(non_string_package_json_type_diagnostic(span)),
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"name":"demo"}"#,
        r#"{"name":"demo","type":"module"}"#,
        r#"{"name":"demo","type":"commonjs"}"#,
    ];

    let fail = vec![
        r#"{"name":"demo","type":"esm"}"#,
        r#"{"name":"demo","type":true}"#,
        r#"{"name":"demo","type":1}"#,
    ];

    Tester::new(PackageJsonValidType::NAME, PackageJsonValidType::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
