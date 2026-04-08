use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

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
    correctness,
    fix
);

impl Rule for PackageJsonValidPrivate {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("private") else {
            return;
        };

        if matches!(&prop.value, JsonValue::Boolean(_, _)) {
            return;
        }

        // Auto-fix string "true"/"false" to boolean true/false
        if let JsonValue::String(value, _) = &prop.value
            && (*value == "true" || *value == "false")
        {
            let replacement = if *value == "true" { "true" } else { "false" };
            ctx.diagnostic_with_fix(
                invalid_package_json_private_diagnostic(prop.value.span()),
                |fixer| fixer.replace_full_source_range(prop.value.span(), replacement),
            );
            return;
        }

        ctx.diagnostic(invalid_package_json_private_diagnostic(prop.value.span()));
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

    let fail = vec![
        r#"{"private":"true"}"#,
        r#"{"private":"false"}"#,
        r#"{"private":1}"#,
        r#"{"private":{}}"#,
    ];

    let fix = vec![
        (r#"{"private":"true"}"#, r#"{"private":true}"#, None),
        (r#"{"private":"false"}"#, r#"{"private":false}"#, None),
    ];

    Tester::new(PackageJsonValidPrivate::NAME, PackageJsonValidPrivate::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
