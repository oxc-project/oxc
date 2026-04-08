use super::json_utils::{is_json_file, property_deletion_span};

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

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
    correctness,
    fix
);

impl Rule for PackageJsonNoEmptyFields {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };

        for (index, prop) in object.properties.iter().enumerate() {
            let kind = match &prop.value {
                JsonValue::Array(arr) if arr.elements.is_empty() => "array",
                JsonValue::Object(obj) if obj.properties.is_empty() => "object",
                _ => continue,
            };

            let delete_span = property_deletion_span(source_text, object, prop, index);
            ctx.diagnostic_with_fix(
                empty_package_json_field_diagnostic(prop.key, kind, prop.value.span()),
                |fixer| fixer.delete_range(delete_span),
            );
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

    let fail = vec![
        r#"{"keywords":[]}"#,
        r#"{"publishConfig":{}}"#,
        r#"{"files":[],"scripts":{}}"#,
        r#"{"name":"demo","keywords":[]}"#,
        r#"{"keywords":[],"name":"demo"}"#,
    ];

    let fix = vec![
        // Only property
        (r#"{"keywords":[]}"#, r#"{}"#, None),
        (r#"{"publishConfig":{}}"#, r#"{}"#, None),
        // First property with trailing comma — remove property + comma
        (r#"{"keywords":[],"name":"demo"}"#, r#"{"name":"demo"}"#, None),
        // Last property with leading comma — remove comma + property
        (r#"{"name":"demo","keywords":[]}"#, r#"{"name":"demo"}"#, None),
    ];

    Tester::new(PackageJsonNoEmptyFields::NAME, PackageJsonNoEmptyFields::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
