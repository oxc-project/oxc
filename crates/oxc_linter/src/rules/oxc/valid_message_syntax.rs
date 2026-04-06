use super::json_utils::{
    display_path, file_start_span, is_json_file, join_array_path, join_object_path,
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn valid_message_syntax_diagnostic(path: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Translation message at `{}` must be a non-empty string.",
        display_path(path)
    ))
    .with_help("Use a non-empty string for locale message leaves.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MessageSyntaxKind {
    #[default]
    NonEmptyString,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ValidMessageSyntaxConfig {
    syntax: MessageSyntaxKind,
}

#[derive(Debug, Default, Clone)]
pub struct ValidMessageSyntax(Box<ValidMessageSyntaxConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates locale message leaves inside JSON catalogs.
    ///
    /// ### Why is this bad?
    ///
    /// Empty or non-string message values usually indicate broken translations
    /// or malformed catalog data.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "title": "" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "title": "Dashboard" }
    /// ```
    ValidMessageSyntax,
    oxc,
    correctness,
    config = ValidMessageSyntaxConfig
);

impl Rule for ValidMessageSyntax {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<ValidMessageSyntaxConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };

        let span = file_start_span(source_text);
        let mut invalid_paths = Vec::new();
        collect_invalid_message_paths(&value, "", self.0.syntax, &mut invalid_paths);

        for path in invalid_paths {
            ctx.diagnostic(valid_message_syntax_diagnostic(&path, span));
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && is_json_file(ctx.file_path())
    }
}

fn collect_invalid_message_paths(
    value: &Value,
    path: &str,
    syntax: MessageSyntaxKind,
    invalid_paths: &mut Vec<String>,
) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                collect_invalid_message_paths(
                    value,
                    &join_object_path(path, key),
                    syntax,
                    invalid_paths,
                );
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                collect_invalid_message_paths(
                    value,
                    &join_array_path(path, index),
                    syntax,
                    invalid_paths,
                );
            }
        }
        Value::String(message) => {
            if matches!(syntax, MessageSyntaxKind::NonEmptyString) && message.trim().is_empty() {
                invalid_paths.push(display_path(path).to_string());
            }
        }
        _ => {
            if matches!(syntax, MessageSyntaxKind::NonEmptyString) {
                invalid_paths.push(display_path(path).to_string());
            }
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"{"title":"Dashboard","cta":"Continue"}"#, None),
        (r#"{"items":["One","Two"],"nested":{"subtitle":"Ready"}}"#, None),
        (r#"{"title":"Dashboard"}"#, Some(json!([{ "syntax": "non-empty-string" }]))),
    ];

    let fail = vec![
        (r#"{"title":""}"#, None),
        (r#"{"title":"   "}"#, None),
        (r#"{"title":1}"#, None),
        (r#"{"items":["One",""]}"#, None),
    ];

    Tester::new(ValidMessageSyntax::NAME, ValidMessageSyntax::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
