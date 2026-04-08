use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn invalid_package_json_bin_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `bin` field in package.json is invalid.")
        .with_help("Use a non-empty string or an object whose values are non-empty strings.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidBin;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `bin` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid `bin` metadata can break CLI installation and command
    /// resolution for published packages.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "bin": { "demo": 123 } }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "bin": "./bin/demo.js" }
    /// ```
    PackageJsonValidBin,
    oxc,
    correctness
);

impl Rule for PackageJsonValidBin {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("bin") else {
            return;
        };

        if is_valid_bin_value(&prop.value) {
            return;
        }

        ctx.diagnostic(invalid_package_json_bin_diagnostic(prop.value.span()));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn is_valid_bin_value(value: &JsonValue<'_>) -> bool {
    match value {
        JsonValue::String(value, _) => !value.trim().is_empty(),
        JsonValue::Object(object) => {
            !object.properties.is_empty()
                && object
                    .properties
                    .iter()
                    .all(|p| matches!(&p.value, JsonValue::String(v, _) if !v.trim().is_empty()))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"bin":"./bin/demo.js"}"#,
        r#"{"bin":{"demo":"./bin/demo.js"}}"#,
        r#"{"bin":{"demo":"./bin/demo.js","demo-dev":"./bin/dev.js"}}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"bin":""}"#,
        r#"{"bin":{}}"#,
        r#"{"bin":{"demo":""}}"#,
        r#"{"bin":{"demo":1}}"#,
        r#"{"bin":1}"#,
    ];

    Tester::new(PackageJsonValidBin::NAME, PackageJsonValidBin::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
