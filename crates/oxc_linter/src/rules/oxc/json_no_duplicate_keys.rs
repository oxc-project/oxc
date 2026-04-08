use rustc_hash::FxHashMap;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
    rules::oxc::json_utils::is_json_file,
};

fn json_no_duplicate_keys_diagnostic(span: Span, key: &str, first_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate key \"{key}\" in JSON object"))
        .with_help("Remove one of the duplicate keys. The last value will be used by JSON parsers.")
        .with_labels([span.label("duplicate key here"), first_span.label("first occurrence")])
}

#[derive(Debug, Default, Clone)]
pub struct JsonNoDuplicateKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects duplicate keys in JSON objects.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate keys in JSON objects are technically allowed by the spec but
    /// the behavior is undefined — most parsers silently use the last value.
    /// This can hide bugs and cause confusion.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** JSON:
    /// ```json
    /// { "name": "foo", "name": "bar" }
    /// ```
    ///
    /// Examples of **correct** JSON:
    /// ```json
    /// { "name": "foo", "alias": "bar" }
    /// ```
    JsonNoDuplicateKeys,
    oxc,
    correctness,
    none
);

impl Rule for JsonNoDuplicateKeys {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);

        if let Some(root) = &result.root {
            check_value(root, ctx);
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        is_json_file(ctx.file_path())
    }
}

fn check_value(value: &JsonValue<'_>, ctx: &LintContext<'_>) {
    match value {
        JsonValue::Object(obj) => {
            let mut seen: FxHashMap<&str, Span> = FxHashMap::default();
            for prop in &obj.properties {
                if let Some(&first_span) = seen.get(prop.key) {
                    ctx.diagnostic(json_no_duplicate_keys_diagnostic(
                        prop.key_span,
                        prop.key,
                        first_span,
                    ));
                } else {
                    seen.insert(prop.key, prop.key_span);
                }
                check_value(&prop.value, ctx);
            }
        }
        JsonValue::Array(arr) => {
            for elem in &arr.elements {
                check_value(elem, ctx);
            }
        }
        _ => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"name": "foo", "version": "1.0"}"#,
        r#"{"a": 1, "b": 2, "c": 3}"#,
        r#"[1, 2, 3]"#,
        r#"{}"#,
    ];

    let fail = vec![
        r#"{"name": "foo", "name": "bar"}"#,
        r#"{"a": 1, "b": 2, "a": 3}"#,
        r#"{"outer": {"inner": 1, "inner": 2}}"#,
    ];

    Tester::new(JsonNoDuplicateKeys::NAME, JsonNoDuplicateKeys::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
