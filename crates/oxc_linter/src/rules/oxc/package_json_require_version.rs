use super::json_utils::{file_start_span, is_json_file};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

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
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };

        if object.contains_key("version") {
            return;
        }

        ctx.diagnostic(missing_package_json_version_diagnostic(file_start_span(source_text)));
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
