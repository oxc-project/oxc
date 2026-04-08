use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn invalid_package_json_keywords_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The `keywords` field in package.json must be an array of non-empty strings.",
    )
    .with_help("Use an array of keyword strings for package discoverability.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidKeywords;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `keywords` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid keywords break npm search metadata and reduce package
    /// discoverability. The field must be an array of non-empty strings.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "keywords": "lint" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "keywords": ["lint", "typescript"] }
    /// ```
    PackageJsonValidKeywords,
    oxc,
    correctness
);

impl Rule for PackageJsonValidKeywords {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("keywords") else {
            return;
        };

        match &prop.value {
            JsonValue::Array(arr) => {
                for element in &arr.elements {
                    match element {
                        JsonValue::String(s, _) if !s.trim().is_empty() => {}
                        _ => {
                            ctx.diagnostic(invalid_package_json_keywords_diagnostic(
                                element.span(),
                            ));
                        }
                    }
                }
            }
            _ => ctx.diagnostic(invalid_package_json_keywords_diagnostic(prop.value.span())),
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
        r#"{"keywords":["lint","typescript"]}"#,
        r#"{"keywords":["cli"]}"#,
        r#"{"keywords":[]}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"keywords":"lint"}"#,
        r#"{"keywords":1}"#,
        r#"{"keywords":["lint",""]}"#,
        r#"{"keywords":["lint",1]}"#,
    ];

    Tester::new(PackageJsonValidKeywords::NAME, PackageJsonValidKeywords::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
