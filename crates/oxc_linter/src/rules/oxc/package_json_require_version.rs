use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn missing_package_json_version_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("package.json should declare a `version` field.")
        .with_help("Add a package version such as `\"version\": \"1.0.0\"`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonRequireVersion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires the `version` field to be present in package.json.
    ///
    /// ### Why is this bad?
    ///
    /// An explicit version keeps package metadata complete and matches npm
    /// package manifest expectations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "name": "demo" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "name": "demo", "version": "1.0.0" }
    /// ```
    PackageJsonRequireVersion,
    oxc,
    correctness
);

impl Rule for PackageJsonRequireVersion {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };

        if object.get("version").is_some() {
            return;
        }

        ctx.diagnostic(missing_package_json_version_diagnostic(Span::new(0, 1)));
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

    let pass =
        vec![r#"{"name":"demo","version":"1.0.0"}"#, r#"{"version":"0.1.0","type":"module"}"#];

    let fail = vec![r#"{"name":"demo"}"#, r#"{"type":"module"}"#];

    Tester::new(PackageJsonRequireVersion::NAME, PackageJsonRequireVersion::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
