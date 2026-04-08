use super::json_utils::is_json_file;

use lazy_regex::regex_is_match;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use spdx::Expression;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn invalid_package_json_license_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `license` field in package.json must be a valid SPDX expression, `UNLICENSED`, or `SEE LICENSE IN <file>`.")
        .with_help("Use a valid SPDX identifier or expression such as `MIT` or `(MIT OR Apache-2.0)`, `UNLICENSED`, or `SEE LICENSE IN LICENSE.md`.")
        .with_label(span)
}

fn non_string_package_json_license_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `license` field in package.json must be a string.")
        .with_help("Use a string value such as `MIT`, `(MIT OR Apache-2.0)`, or `UNLICENSED`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidLicense;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `license` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid license metadata breaks npm package metadata expectations and
    /// makes license auditing less reliable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "license": "GPL3" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "license": "GPL-3.0-only" }
    /// ```
    PackageJsonValidLicense,
    oxc,
    correctness
);

impl Rule for PackageJsonValidLicense {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("license") else {
            return;
        };

        let JsonValue::String(license, _) = &prop.value else {
            ctx.diagnostic(non_string_package_json_license_diagnostic(prop.value.span()));
            return;
        };

        if is_valid_license_value(license) {
            return;
        }

        ctx.diagnostic(invalid_package_json_license_diagnostic(prop.value.span()));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn is_valid_license_value(license: &str) -> bool {
    license == "UNLICENSED"
        || is_see_license_reference(license)
        || is_valid_spdx_license_expression(license)
}

fn is_see_license_reference(license: &str) -> bool {
    regex_is_match!(r"^SEE LICENSE IN \S(?:.*\S)?$", license)
}

fn is_valid_spdx_license_expression(license: &str) -> bool {
    !matches!(license, "NONE" | "NOASSERTION")
        && !license.contains("LicenseRef-")
        && !license.contains("DocumentRef-")
        && Expression::parse(license).is_ok()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"license":"MIT"}"#,
        r#"{"license":"(MIT OR Apache-2.0)"}"#,
        r#"{"license":"GPL-2.0-only WITH Classpath-exception-2.0"}"#,
        r#"{"license":"UNLICENSED"}"#,
        r#"{"license":"SEE LICENSE IN LICENSE.md"}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"license":1}"#,
        r#"{"license":"GPL3"}"#,
        r#"{"license":"MIT OR NOPE"}"#,
        r#"{"license":"NOASSERTION"}"#,
        r#"{"license":"LicenseRef-Custom"}"#,
        r#"{"license":"SEE LICENSE IN "}"#,
        r#"{"license":{"type":"MIT"}}"#,
    ];

    Tester::new(PackageJsonValidLicense::NAME, PackageJsonValidLicense::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
