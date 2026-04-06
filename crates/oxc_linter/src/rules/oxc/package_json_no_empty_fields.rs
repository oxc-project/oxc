use super::json_utils::{file_start_span, is_json_file};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn empty_package_json_field_diagnostic(
    field_name: &str,
    field_kind: &str,
    span: oxc_span::Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `{field_name}` field in package.json should not be an empty {field_kind}."
    ))
    .with_help("Remove empty package.json fields that do not carry any information.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonNoEmptyFields;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports unnecessary empty top-level arrays and objects in package.json.
    ///
    /// ### Why is this bad?
    ///
    /// Empty package fields add noise without changing package behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "keywords": [] }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "keywords": ["oxlint"] }
    /// ```
    PackageJsonNoEmptyFields,
    oxc,
    correctness
);

impl Rule for PackageJsonNoEmptyFields {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };

        let span = file_start_span(source_text);
        for (field_name, field_value) in object {
            match field_value {
                Value::Array(items) if items.is_empty() => {
                    ctx.diagnostic(empty_package_json_field_diagnostic(field_name, "array", span));
                }
                Value::Object(entries) if entries.is_empty() => {
                    ctx.diagnostic(empty_package_json_field_diagnostic(field_name, "object", span));
                }
                _ => {}
            }
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
        r#"{"keywords":["oxlint"]}"#,
        r#"{"publishConfig":{"access":"public"}}"#,
        r#"{"exports":{"." :"./dist/index.js"}}"#,
    ];

    let fail =
        vec![r#"{"keywords":[]}"#, r#"{"publishConfig":{}}"#, r#"{"files":[],"scripts":{}}"#];

    Tester::new(PackageJsonNoEmptyFields::NAME, PackageJsonNoEmptyFields::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
