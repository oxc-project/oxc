use super::json_utils::{is_json_file, property_deletion_span};

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

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
    correctness,
    fix
);

impl Rule for PackageJsonValidType {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some((index, prop)) =
            object.properties.iter().enumerate().find(|(_, p)| p.key == "type")
        else {
            return;
        };

        match &prop.value {
            JsonValue::String(kind, _) if *kind == "module" || *kind == "commonjs" => {}
            JsonValue::String(kind, _) => {
                ctx.diagnostic(invalid_package_json_type_diagnostic(kind, prop.value.span()));
            }
            _ => {
                let delete_span = property_deletion_span(source_text, object, prop, index);
                ctx.diagnostic_with_fix(
                    non_string_package_json_type_diagnostic(prop.value.span()),
                    |fixer| fixer.delete_range(delete_span),
                );
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
        r#"{"name":"demo","type":"module"}"#,
        r#"{"name":"demo","type":"commonjs"}"#,
    ];

    let fail = vec![
        r#"{"name":"demo","type":"esm"}"#,
        r#"{"name":"demo","type":true}"#,
        r#"{"name":"demo","type":1}"#,
    ];

    // Non-string types get auto-deleted (restoring Node's default behavior)
    let fix = vec![
        (r#"{"name":"demo","type":true}"#, r#"{"name":"demo"}"#, None),
        (r#"{"name":"demo","type":1}"#, r#"{"name":"demo"}"#, None),
    ];

    Tester::new(PackageJsonValidType::NAME, PackageJsonValidType::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
