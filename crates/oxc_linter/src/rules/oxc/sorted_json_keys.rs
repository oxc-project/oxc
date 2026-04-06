use std::{cmp::Ordering, ffi::OsStr};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn sorted_json_keys_diagnostic() -> OxcDiagnostic {
    OxcDiagnostic::warn("JSON object keys should be sorted.")
        .with_help("Sort object keys consistently to keep locale files easy to diff and review.")
        .with_label(Span::default())
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum JsonSortOrder {
    Desc,
    #[default]
    Asc,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SortedJsonKeysConfig {
    order: JsonSortOrder,
    indent_spaces: usize,
}

impl Default for SortedJsonKeysConfig {
    fn default() -> Self {
        Self { order: JsonSortOrder::Asc, indent_spaces: 2 }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortedJsonKeys(Box<SortedJsonKeysConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces recursively sorted keys in JSON files.
    ///
    /// ### Why is this bad?
    ///
    /// Locale JSON files are easier to compare, review, and synchronize when
    /// object keys stay in a deterministic order.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "b": "two", "a": "one" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "a": "one", "b": "two" }
    /// ```
    SortedJsonKeys,
    oxc,
    style,
    fix,
    config = SortedJsonKeysConfig
);

impl Rule for SortedJsonKeys {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortedJsonKeysConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };

        if is_json_value_sorted(&value, self.0.order) {
            return;
        }

        let sorted = sort_json_value(&value, self.0.order);
        let Ok(mut expected) = serialize_json(&sorted, self.0.indent_spaces) else {
            return;
        };

        if source_text.ends_with('\n') {
            expected.push('\n');
        }

        let file_span = Span::new(0, source_text.len() as u32);
        ctx.diagnostic_with_fix(sorted_json_keys_diagnostic(), |fixer| {
            fixer.replace_full_source_range(file_span, expected)
        });
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && ctx.file_extension().is_some_and(|ext| ext == OsStr::new("json"))
    }
}

fn sort_json_value(value: &Value, order: JsonSortOrder) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|(left_key, _), (right_key, _)| {
                let ordering = left_key.cmp(right_key);
                match order {
                    JsonSortOrder::Asc => ordering,
                    JsonSortOrder::Desc => ordering.reverse(),
                }
            });

            let mut sorted = Map::with_capacity(object.len());
            for (key, value) in entries {
                sorted.insert(key.clone(), sort_json_value(value, order));
            }
            Value::Object(sorted)
        }
        Value::Array(array) => {
            Value::Array(array.iter().map(|item| sort_json_value(item, order)).collect())
        }
        _ => value.clone(),
    }
}

fn is_json_value_sorted(value: &Value, order: JsonSortOrder) -> bool {
    match value {
        Value::Object(object) => {
            let mut previous_key: Option<&str> = None;
            for (key, value) in object {
                if let Some(previous_key) = previous_key {
                    let ordering = previous_key.cmp(key.as_str());
                    let in_order = match order {
                        JsonSortOrder::Asc => ordering != Ordering::Greater,
                        JsonSortOrder::Desc => ordering != Ordering::Less,
                    };
                    if !in_order {
                        return false;
                    }
                }

                if !is_json_value_sorted(value, order) {
                    return false;
                }

                previous_key = Some(key.as_str());
            }

            true
        }
        Value::Array(array) => array.iter().all(|item| is_json_value_sorted(item, order)),
        _ => true,
    }
}

fn serialize_json(value: &Value, indent_spaces: usize) -> Result<String, serde_json::Error> {
    let indent = vec![b' '; indent_spaces.max(1)];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut output = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut output, formatter);
    value.serialize(&mut serializer)?;
    Ok(String::from_utf8(output).expect("serde_json emitted invalid UTF-8"))
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"{"a":"one","b":"two"}"#, None),
        (r#"{"a":{"b":"two","c":"three"},"z":"last"}"#, None),
        (r#"{"b":"two","a":"one"}"#, Some(json!([{ "order": "desc", "indentSpaces": 2 }]))),
    ];

    let fail = vec![
        (r#"{"b":"two","a":"one"}"#, None),
        (r#"{"z":"last","a":{"c":"three","b":"two"}}"#, None),
        (r#"{"a":"one","b":"two"}"#, Some(json!([{ "order": "desc", "indentSpaces": 2 }]))),
    ];

    let fix = vec![
        (r#"{"b":"two","a":"one"}"#, "{\n  \"a\": \"one\",\n  \"b\": \"two\"\n}", None),
        (
            r#"{"z":"last","a":{"c":"three","b":"two"}}"#,
            "{\n  \"a\": {\n    \"b\": \"two\",\n    \"c\": \"three\"\n  },\n  \"z\": \"last\"\n}",
            None,
        ),
        (
            r#"{"a":"one","b":"two"}"#,
            "{\n  \"b\": \"two\",\n  \"a\": \"one\"\n}",
            Some(json!([{ "order": "desc", "indentSpaces": 2 }])),
        ),
    ];

    Tester::new(SortedJsonKeys::NAME, SortedJsonKeys::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
