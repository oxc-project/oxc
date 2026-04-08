use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::{DefaultRuleConfig, Rule},
};

fn valid_message_syntax_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Translation message must be a non-empty string.")
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
        let result = parse_json(source_text);
        let Some(root) = &result.root else {
            return;
        };

        let mut invalid_spans = Vec::new();
        collect_invalid_message_spans(root, self.0.syntax, &mut invalid_spans);

        for span in invalid_spans {
            ctx.diagnostic(valid_message_syntax_diagnostic(span));
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && is_json_file(ctx.file_path())
    }
}

fn collect_invalid_message_spans(
    value: &JsonValue<'_>,
    syntax: MessageSyntaxKind,
    invalid_spans: &mut Vec<Span>,
) {
    match value {
        JsonValue::Object(object) => {
            for prop in &object.properties {
                collect_invalid_message_spans(&prop.value, syntax, invalid_spans);
            }
        }
        JsonValue::Array(array) => {
            for element in &array.elements {
                collect_invalid_message_spans(element, syntax, invalid_spans);
            }
        }
        JsonValue::String(message, span) => {
            if matches!(syntax, MessageSyntaxKind::NonEmptyString) && message.trim().is_empty() {
                invalid_spans.push(*span);
            }
        }
        _ => {
            if matches!(syntax, MessageSyntaxKind::NonEmptyString) {
                invalid_spans.push(value.span());
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
