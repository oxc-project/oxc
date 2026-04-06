use super::json_utils::{file_start_span, is_json_file};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn invalid_package_json_private_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `private` field in package.json must be a boolean.")
        .with_help("Use `true` or `false` for the package privacy flag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidPrivate;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `private` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid `private` values make package publish intent ambiguous and break
    /// npm metadata expectations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "private": "true" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "private": true }
    /// ```
    PackageJsonValidPrivate,
    oxc,
    correctness
);

impl Rule for PackageJsonValidPrivate {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };
        let Some(private) = object.get("private") else {
            return;
        };

        if matches!(private, Value::Bool(_)) {
            return;
        }

        ctx.diagnostic(invalid_package_json_private_diagnostic(file_start_span(source_text)));
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

    let pass = vec![r#"{"private":true}"#, r#"{"private":false}"#, r#"{"name":"demo"}"#];

    let fail = vec![r#"{"private":"true"}"#, r#"{"private":1}"#, r#"{"private":{}}"#];

    Tester::new(PackageJsonValidPrivate::NAME, PackageJsonValidPrivate::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
