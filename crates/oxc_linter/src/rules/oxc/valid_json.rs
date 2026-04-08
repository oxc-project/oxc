use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{context::LintContext, json_parser::parse_json, rule::Rule};

fn valid_json_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid JSON: {message}"))
        .with_help("Fix the JSON syntax error.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ValidJsonConfig {
    allow_comments: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ValidJson(Box<ValidJsonConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports invalid JSON documents.
    ///
    /// ### Why is this bad?
    ///
    /// JSON-backed rule families such as locale catalogs cannot be linted
    /// reliably if the source file is not valid JSON.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "message": }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "message": "hello" }
    /// ```
    ValidJson,
    oxc,
    correctness,
    config = ValidJsonConfig
);

impl Rule for ValidJson {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let mut config = ValidJsonConfig::default();

        match value {
            serde_json::Value::Null => {}
            serde_json::Value::Array(values) => {
                for value in values {
                    match value {
                        serde_json::Value::String(option) if option == "allowComments" => {
                            config.allow_comments = true;
                        }
                        other => {
                            config = serde_json::from_value(other)?;
                        }
                    }
                }
            }
            serde_json::Value::String(option) if option == "allowComments" => {
                config.allow_comments = true;
            }
            other => {
                config = serde_json::from_value(other)?;
            }
        }

        Ok(Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let owned_stripped;
        let parse_source = if self.0.allow_comments {
            let mut stripped = source_text.to_string();
            if json_strip_comments::strip(&mut stripped).is_ok() {
                owned_stripped = stripped;
                owned_stripped.as_str()
            } else {
                source_text
            }
        } else {
            source_text
        };

        let result = parse_json(parse_source);
        if result.errors.is_empty() && result.root.is_some() {
            return;
        }

        for error in &result.errors {
            ctx.diagnostic(valid_json_diagnostic(&error.message, error.span));
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && is_json_file(ctx.file_path())
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"{"message":"hello"}"#, None),
        (r#"{"nested":{"message":"hello"}}"#, None),
        ("{\n  // comment\n  \"message\": \"hello\"\n}", Some(json!(["allowComments"]))),
        ("{\n  // comment\n  \"message\": \"hello\"\n}", Some(json!([{ "allowComments": true }]))),
    ];

    let fail = vec![
        (r#"{"message":}"#, None),
        (r#"{"message":"hello",}"#, None),
        ("{\n  // comment\n  \"message\": \"hello\"\n}", None),
    ];

    Tester::new(ValidJson::NAME, ValidJson::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
