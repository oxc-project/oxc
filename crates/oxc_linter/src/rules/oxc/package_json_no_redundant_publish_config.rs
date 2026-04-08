use super::json_utils::{file_start_span, is_json_file};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use serde::Serialize;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn redundant_publish_config_access_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "`publishConfig.access` is redundant for unscoped packages because they are always public.",
    )
    .with_help("Remove the redundant `publishConfig.access` field.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonNoRedundantPublishConfig;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Warns when `publishConfig.access` is used in unscoped packages.
    ///
    /// ### Why is this bad?
    ///
    /// Unscoped npm packages are always public, so `publishConfig.access`
    /// has no effect there.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "name": "demo", "publishConfig": { "access": "public" } }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "name": "@scope/demo", "publishConfig": { "access": "public" } }
    /// ```
    PackageJsonNoRedundantPublishConfig,
    oxc,
    correctness,
    fix
);

impl Rule for PackageJsonNoRedundantPublishConfig {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };

        let Some(Value::String(package_name)) = object.get("name") else {
            return;
        };
        if package_name.starts_with('@') {
            return;
        }

        let Some(Value::Object(publish_config)) = object.get("publishConfig") else {
            return;
        };
        if !publish_config.contains_key("access") {
            return;
        }

        let Some(updated) = remove_publish_config_access(&value) else {
            return;
        };
        let Some(expected) = serialize_updated_json(&updated, source_text) else {
            return;
        };

        #[expect(clippy::cast_possible_truncation)]
        let file_span = oxc_span::Span::new(0, source_text.len() as u32);
        ctx.diagnostic_with_fix(
            redundant_publish_config_access_diagnostic(file_start_span(source_text)),
            |fixer| fixer.replace_full_source_range(file_span, expected),
        );
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn remove_publish_config_access(value: &Value) -> Option<Value> {
    let mut object = value.as_object()?.clone();
    let Value::Object(publish_config) = object.get("publishConfig")?.clone() else {
        return None;
    };

    let mut publish_config = publish_config;
    publish_config.remove("access");
    object.insert("publishConfig".to_string(), Value::Object(publish_config));
    Some(Value::Object(object))
}

fn serialize_updated_json(value: &Value, source_text: &str) -> Option<String> {
    let indent = vec![b' '; 2];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut output = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut output, formatter);
    value.serialize(&mut serializer).ok()?;
    let mut formatted = String::from_utf8(output).ok()?;
    if source_text.ends_with('\n') {
        formatted.push('\n');
    }
    Some(formatted)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"name":"@scope/demo","publishConfig":{"access":"public"}}"#,
        r#"{"name":"demo","publishConfig":{"registry":"https://example.com"}}"#,
        r#"{"name":"demo"}"#,
        r#"{"publishConfig":{"access":"public"}}"#,
    ];

    let fail = vec![
        r#"{"name":"demo","publishConfig":{"access":"public"}}"#,
        r#"{"name":"demo","publishConfig":{"access":"restricted"}}"#,
        r#"{"name":"demo","publishConfig":{"access":"public","registry":"https://example.com"}}"#,
    ];

    let fix = vec![
        (
            r#"{"name":"demo","publishConfig":{"access":"public"}}"#,
            "{\n  \"name\": \"demo\",\n  \"publishConfig\": {}\n}",
        ),
        (
            r#"{"name":"demo","publishConfig":{"access":"restricted"}}"#,
            "{\n  \"name\": \"demo\",\n  \"publishConfig\": {}\n}",
        ),
        (
            r#"{"name":"demo","publishConfig":{"access":"public","registry":"https://example.com"}}"#,
            "{\n  \"name\": \"demo\",\n  \"publishConfig\": {\n    \"registry\": \"https://example.com\"\n  }\n}",
        ),
    ];

    Tester::new(
        PackageJsonNoRedundantPublishConfig::NAME,
        PackageJsonNoRedundantPublishConfig::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .change_rule_path("package.json")
    .test_and_snapshot();
}
