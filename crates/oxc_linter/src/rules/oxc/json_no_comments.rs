use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, rules::oxc::json_utils::is_json_file};

fn json_no_comments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Comments are not allowed in JSON")
        .with_help("Remove the comment. Standard JSON (RFC 8259) does not support comments. Use JSONC if you need comments.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsonNoComments;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects JavaScript-style comments (`//` and `/* */`) in JSON files.
    ///
    /// ### Why is this bad?
    ///
    /// Standard JSON (RFC 8259) does not allow comments. Most strict JSON
    /// parsers will reject files containing comments.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** JSON:
    /// ```json
    /// {
    ///     // This is a comment
    ///     "name": "foo"
    /// }
    /// ```
    ///
    /// Examples of **correct** JSON:
    /// ```json
    /// { "name": "foo" }
    /// ```
    JsonNoComments,
    oxc,
    correctness,
    none
);

impl Rule for JsonNoComments {
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
                    i += 2;
                }
                b'/' if !in_string && i + 1 < len => {
                    match bytes[i + 1] {
                        b'/' => {
                            // Single-line comment
                            let start = i;
                            i += 2;
                            while i < len && bytes[i] != b'\n' {
                                i += 1;
                            }
                            ctx.diagnostic(json_no_comments_diagnostic(Span::new(
                                start as u32,
                                i as u32,
                            )));
                        }
                        b'*' => {
                            // Block comment
                            let start = i;
                            i += 2;
                            while i + 1 < len {
                                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                                    i += 2;
                                    break;
                                }
                                i += 1;
                            }
                            ctx.diagnostic(json_no_comments_diagnostic(Span::new(
                                start as u32,
                                i as u32,
                            )));
                        }
                        _ => i += 1,
                    }
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

    let pass = vec![r#"{"name": "foo"}"#, r#"{"url": "https://example.com"}"#, "{}"];

    let fail = vec!["{\n// comment\n\"a\": 1\n}", "{\n/* block */\n\"a\": 1\n}"];

    Tester::new(JsonNoComments::NAME, JsonNoComments::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
