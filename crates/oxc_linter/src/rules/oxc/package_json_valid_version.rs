use super::json_utils::is_json_file;

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

static SEMVER_REGEX: Lazy<Regex> = lazy_regex!(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
);

fn invalid_package_json_version_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `version` field in package.json must be a valid semver string.")
        .with_help("Use a version like `1.0.0`, `1.2.3-beta.1`, or `1.0.0+build.5`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidVersion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `version` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid package versions break npm metadata expectations and make
    /// release automation unreliable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "version": "1.0" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "version": "1.0.0" }
    /// ```
    PackageJsonValidVersion,
    oxc,
    correctness
);

impl Rule for PackageJsonValidVersion {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("version") else {
            return;
        };

        match &prop.value {
            JsonValue::String(version, _) if SEMVER_REGEX.is_match(version) => {}
            _ => ctx.diagnostic(invalid_package_json_version_diagnostic(prop.value.span())),
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
        r#"{"version":"1.0.0"}"#,
        r#"{"version":"0.1.0-alpha.1"}"#,
        r#"{"version":"2.3.4+build.5"}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"version":"1.0"}"#,
        r#"{"version":"v1.0.0"}"#,
        r#"{"version":"01.0.0"}"#,
        r#"{"version":1}"#,
    ];

    Tester::new(PackageJsonValidVersion::NAME, PackageJsonValidVersion::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
