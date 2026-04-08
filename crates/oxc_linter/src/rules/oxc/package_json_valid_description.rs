use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn invalid_package_json_description_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `description` field in package.json must be a non-empty string.")
        .with_help("Use a descriptive non-empty package description.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `description` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid descriptions reduce package metadata quality and can break tools
    /// that expect a human-readable string.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "description": "" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "description": "Fast linter for TypeScript" }
    /// ```
    PackageJsonValidDescription,
    oxc,
    correctness
);

impl Rule for PackageJsonValidDescription {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("description") else {
            return;
        };

        let is_valid = match &prop.value {
            JsonValue::String(value, _) => !value.trim().is_empty(),
            _ => false,
        };

        if is_valid {
            return;
        }

        ctx.diagnostic(invalid_package_json_description_diagnostic(prop.value.span()));
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
        r#"{"description":"Fast linter"}"#,
        r#"{"description":"CLI for package publishing"}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![r#"{"description":""}"#, r#"{"description":"   "}"#, r#"{"description":1}"#];

    Tester::new(PackageJsonValidDescription::NAME, PackageJsonValidDescription::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
