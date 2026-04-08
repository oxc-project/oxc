use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn invalid_package_json_engines_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `engines` field in package.json is invalid.")
        .with_help("Use an object with runtime names as keys and semver ranges as values, e.g. `{ \"node\": \">=18\" }`.")
        .with_label(span)
}

fn invalid_engine_value_diagnostic(key: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `engines.{key}` value in package.json must be a non-empty string."
    ))
    .with_help("Use a semver range string like `>=18`, `^20.0.0`, or `*`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidEngines;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `engines` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid `engines` metadata breaks runtime compatibility checks and
    /// can confuse package managers that enforce engine constraints.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "engines": "node >= 18" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "engines": { "node": ">=18" } }
    /// ```
    PackageJsonValidEngines,
    oxc,
    correctness
);

impl Rule for PackageJsonValidEngines {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("engines") else {
            return;
        };

        let JsonValue::Object(engines) = &prop.value else {
            ctx.diagnostic(invalid_package_json_engines_diagnostic(prop.value.span()));
            return;
        };

        for entry in &engines.properties {
            match &entry.value {
                JsonValue::String(value, _) if !value.trim().is_empty() => {}
                _ => {
                    ctx.diagnostic(invalid_engine_value_diagnostic(entry.key, entry.value.span()));
                }
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
        r#"{"engines":{"node":">=18"}}"#,
        r#"{"engines":{"node":">=18","npm":">=9"}}"#,
        r#"{"engines":{"node":"*"}}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"engines":"node >= 18"}"#,
        r#"{"engines":1}"#,
        r#"{"engines":{"node":""}}"#,
        r#"{"engines":{"node":1}}"#,
        r#"{"engines":{"node":">=18","npm":""}}"#,
    ];

    Tester::new(PackageJsonValidEngines::NAME, PackageJsonValidEngines::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
