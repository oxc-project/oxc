use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, rules::oxc::json_utils::is_json_file};

fn json_no_trailing_commas_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Trailing comma in JSON")
        .with_help("Remove the trailing comma. Trailing commas are not allowed in JSON.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsonNoTrailingCommas;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects trailing commas in JSON objects and arrays.
    ///
    /// ### Why is this bad?
    ///
    /// Trailing commas are not allowed in strict JSON per RFC 8259.
    /// Most JSON parsers will reject files with trailing commas.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** JSON:
    /// ```json
    /// { "name": "foo", }
    /// [1, 2, 3,]
    /// ```
    ///
    /// Examples of **correct** JSON:
    /// ```json
    /// { "name": "foo" }
    /// [1, 2, 3]
    /// ```
    JsonNoTrailingCommas,
    oxc,
    correctness,
    none
);

impl Rule for JsonNoTrailingCommas {
    #[expect(clippy::cast_possible_truncation)] // Span uses u32 by design
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source = ctx.full_source_text();
        let bytes = source.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut in_string = false;

        while i < len {
            match bytes[i] {
                b'"' if !in_string => {
                    in_string = true;
                    i += 1;
                }
                b'"' if in_string => {
                    in_string = false;
                    i += 1;
                }
                b'\\' if in_string => {
                    i += 2; // skip escaped char
                }
                b',' if !in_string => {
                    // Check if this comma is followed by a closing bracket/brace
                    let mut j = i + 1;
                    while j < len && matches!(bytes[j], b' ' | b'\t' | b'\n' | b'\r') {
                        j += 1;
                    }
                    if j < len && (bytes[j] == b'}' || bytes[j] == b']') {
                        ctx.diagnostic(json_no_trailing_commas_diagnostic(Span::new(
                            i as u32,
                            (i + 1) as u32,
                        )));
                    }
                    i += 1;
                }
                _ => i += 1,
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        is_json_file(ctx.file_path())
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r#"{"name": "foo"}"#, r#"[1, 2, 3]"#, r#"{"a": [1, 2]}"#, "{}", "[]"];

    let fail = vec![r#"{"name": "foo",}"#, "[1, 2, 3,]", r#"{"a": 1, "b": 2,}"#];

    Tester::new(JsonNoTrailingCommas::NAME, JsonNoTrailingCommas::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
