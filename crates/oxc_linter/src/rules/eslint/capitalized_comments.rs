use lazy_regex::{Regex, RegexBuilder};
use oxc_ast::Comment;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule};

/// Common directive prefixes that should be ignored (module-level to avoid items-after-statements)
const DIRECTIVES: &[&str] = &[
    "eslint-disable",
    "eslint-enable",
    "eslint-env",
    "eslint ",
    "jshint",
    "jscs",
    "istanbul",
    "global ",
    "globals ",
    "exported",
];

fn capitalized_comments_diagnostic(
    span: Span,
    wrong_case: &str,
    correct_case: &str,
) -> OxcDiagnostic {
    let article = if wrong_case == "uppercase" { "an" } else { "a" };
    OxcDiagnostic::warn(format!("Comments should not begin with {article} {wrong_case} letter"))
        .with_help(format!("Change the first letter of the comment to {correct_case}"))
        .with_label(span)
}

/// Configuration for either line or block comments
#[derive(Debug, Clone, Default)]
#[expect(clippy::struct_field_names)]
struct CommentConfig {
    ignore_pattern: Option<Regex>,
    ignore_inline_comments: bool,
    ignore_consecutive_comments: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum CapitalizeOption {
    #[default]
    Always,
    Never,
}

/// Internal configuration structure (boxed for size optimization)
#[derive(Debug, Clone, Default)]
pub struct CapitalizedCommentsConfig {
    capitalize: CapitalizeOption,
    line_config: CommentConfig,
    block_config: CommentConfig,
}

/// Rule struct wraps config in Box to keep RuleEnum small (16 bytes)
#[derive(Debug, Clone, Default)]
pub struct CapitalizedComments(Box<CapitalizedCommentsConfig>);

impl std::ops::Deref for CapitalizedComments {
    type Target = CapitalizedCommentsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[expect(clippy::struct_field_names)]
struct CommentConfigJson {
    ignore_pattern: Option<String>,
    ignore_inline_comments: Option<bool>,
    ignore_consecutive_comments: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct OptionsJson {
    ignore_pattern: Option<String>,
    ignore_inline_comments: Option<bool>,
    ignore_consecutive_comments: Option<bool>,
    line: Option<CommentConfigJson>,
    block: Option<CommentConfigJson>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces or disallows capitalization of the first letter of a comment.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent capitalization of comments can make code harder to read.
    /// This rule helps enforce a consistent style across the codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"always"` option:
    /// ```js
    /// // lowercase comment
    /// /* lowercase block comment */
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"always"` option:
    /// ```js
    /// // Capitalized comment
    /// /* Capitalized block comment */
    /// // 123 - comments starting with non-letters are ignored
    /// ```
    CapitalizedComments,
    eslint,
    style,
    fix,
    config = OptionsJson
);

impl Rule for CapitalizedComments {
    fn from_configuration(value: serde_json::Value) -> Self {
        let arr = value.as_array();

        // Parse capitalize option (first element)
        let capitalize = arr
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "never" => CapitalizeOption::Never,
                _ => CapitalizeOption::Always,
            })
            .unwrap_or_default();

        // Parse options object (second element)
        let options: Option<OptionsJson> =
            arr.and_then(|arr| arr.get(1)).and_then(|v| serde_json::from_value(v.clone()).ok());

        let (line_config, block_config) = if let Some(opts) = options {
            // Build base config from top-level options
            // ESLint uses ^\s*(?:pattern) for prefix matching on normalized text
            let base_ignore_pattern = opts
                .ignore_pattern
                .as_ref()
                .and_then(|p| RegexBuilder::new(&format!(r"^\s*(?:{p})")).build().ok());
            let base_ignore_inline = opts.ignore_inline_comments.unwrap_or(false);
            let base_ignore_consecutive = opts.ignore_consecutive_comments.unwrap_or(false);

            // Build line config (merge with line-specific options)
            let line_config = CommentConfig {
                ignore_pattern: opts
                    .line
                    .as_ref()
                    .and_then(|l| l.ignore_pattern.as_ref())
                    .and_then(|p| RegexBuilder::new(&format!(r"^\s*(?:{p})")).build().ok())
                    .or_else(|| base_ignore_pattern.clone()),
                ignore_inline_comments: opts
                    .line
                    .as_ref()
                    .and_then(|l| l.ignore_inline_comments)
                    .unwrap_or(base_ignore_inline),
                ignore_consecutive_comments: opts
                    .line
                    .as_ref()
                    .and_then(|l| l.ignore_consecutive_comments)
                    .unwrap_or(base_ignore_consecutive),
            };

            // Build block config (merge with block-specific options)
            let block_config = CommentConfig {
                ignore_pattern: opts
                    .block
                    .as_ref()
                    .and_then(|b| b.ignore_pattern.as_ref())
                    .and_then(|p| RegexBuilder::new(&format!(r"^\s*(?:{p})")).build().ok())
                    .or(base_ignore_pattern),
                ignore_inline_comments: opts
                    .block
                    .as_ref()
                    .and_then(|b| b.ignore_inline_comments)
                    .unwrap_or(base_ignore_inline),
                ignore_consecutive_comments: opts
                    .block
                    .as_ref()
                    .and_then(|b| b.ignore_consecutive_comments)
                    .unwrap_or(base_ignore_consecutive),
            };

            (line_config, block_config)
        } else {
            (CommentConfig::default(), CommentConfig::default())
        };

        Self(Box::new(CapitalizedCommentsConfig { capitalize, line_config, block_config }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let source_text = ctx.source_text();
        let comments: Vec<&Comment> = ctx.semantic().comments().iter().collect();

        for (i, comment) in comments.iter().enumerate() {
            // Skip shebang comments
            if comment.span.start == 0 && source_text.starts_with("#!") {
                continue;
            }

            let config = if comment.is_line() { &self.line_config } else { &self.block_config };

            // Get the comment content (without the // or /* */)
            let content = ctx.source_range(comment.content_span());

            // Normalize the content: remove leading whitespace and JSDoc-style * markers
            // This is what ESLint calls "getTextWithoutAsterisks"
            let normalized = normalize_comment_text(content);

            // Find the first letter position in original content
            let Some((first_letter_offset, first_letter)) = find_first_letter(content) else {
                continue; // No letter found, skip
            };

            // Check if this is a directive comment (using normalized text)
            if is_directive_comment(&normalized) {
                continue;
            }

            // Check ignorePattern (using normalized text, like ESLint)
            if let Some(ref pattern) = config.ignore_pattern
                && pattern.is_match(&normalized)
            {
                continue;
            }

            // Check ignoreInlineComments
            if config.ignore_inline_comments && is_inline_comment(source_text, comment.span) {
                continue;
            }

            // Check ignoreConsecutiveComments
            if config.ignore_consecutive_comments
                && i > 0
                && is_consecutive_comment(source_text, comments[i - 1], comment)
            {
                continue;
            }

            // Check capitalization using ESLint's approach:
            // - A letter is "uppercase" if it differs from its lowercase form
            // - A letter is "lowercase" if it differs from its uppercase form
            // - Letters with no case distinction (CJK, Hebrew, Arabic, etc.) are skipped
            let letter_str = first_letter.to_string();
            let lower = first_letter.to_lowercase().to_string();
            let upper = first_letter.to_uppercase().to_string();

            // Skip letters with no case distinction (e.g., CJK characters)
            if lower == upper {
                continue;
            }

            let is_uppercase = letter_str != lower;
            let needs_fix = match self.capitalize {
                CapitalizeOption::Always => !is_uppercase,
                CapitalizeOption::Never => is_uppercase,
            };

            if needs_fix {
                let (wrong_case, correct_case) = if self.capitalize == CapitalizeOption::Always {
                    ("lowercase", "uppercase")
                } else {
                    ("uppercase", "lowercase")
                };

                // Calculate the span for the first letter
                #[expect(clippy::cast_possible_truncation)]
                let letter_start = comment.content_span().start + first_letter_offset as u32;
                #[expect(clippy::cast_possible_truncation)]
                let letter_end = letter_start + first_letter.len_utf8() as u32;
                let letter_span = Span::new(letter_start, letter_end);

                ctx.diagnostic_with_fix(
                    capitalized_comments_diagnostic(comment.span, wrong_case, correct_case),
                    |fixer| {
                        let fixed_letter = if self.capitalize == CapitalizeOption::Always {
                            first_letter.to_uppercase().to_string()
                        } else {
                            first_letter.to_lowercase().to_string()
                        };
                        fixer.replace(letter_span, fixed_letter)
                    },
                );
            }
        }
    }
}

/// Normalize comment text by removing JSDoc-style asterisk markers.
/// This matches ESLint's "getTextWithoutAsterisks" behavior.
///
/// For example:
/// - "* description" -> "description"
/// - "\n * line1\n * line2" -> "line1\n line2"
fn normalize_comment_text(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            // Remove leading * and following whitespace
            if let Some(rest) = trimmed.strip_prefix('*') { rest.trim_start() } else { trimmed }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the first Unicode letter in the content and return its byte offset and the char
fn find_first_letter(content: &str) -> Option<(usize, char)> {
    let trimmed = content.trim_start();
    let whitespace_len = content.len() - trimmed.len();

    // Skip markdown-style markers like * at the start of JSDoc comments
    let trimmed = trimmed.trim_start_matches(|c: char| c == '*' || c.is_whitespace());
    let extra_offset = content.len() - trimmed.len() - whitespace_len;

    for (i, c) in trimmed.char_indices() {
        if c.is_alphabetic() {
            return Some((whitespace_len + extra_offset + i, c));
        }
        // Non-letter, non-whitespace character - check if it's something that should stop us
        if !c.is_whitespace() && !c.is_alphabetic() {
            // If it's a digit or special char at the start, the whole comment is "non-cased"
            return None;
        }
    }
    None
}

/// Check if the comment content is a directive comment that should be ignored
fn is_directive_comment(normalized: &str) -> bool {
    let trimmed = normalized.trim_start();

    // Check known directive prefixes
    for directive in DIRECTIVES {
        if trimmed.starts_with(directive) {
            return true;
        }
    }

    // Check if it looks like a URL (any scheme://)
    // This matches ESLint's behavior which uses a general pattern
    if let Some(colon_pos) = trimmed.find(':') {
        // Check if there's "://" after the colon and the scheme is alphanumeric
        if trimmed.get(colon_pos..colon_pos + 3) == Some("://") {
            let scheme = &trimmed[..colon_pos];
            // Scheme must be non-empty and contain only alphanumeric, +, -, .
            if !scheme.is_empty()
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
            {
                return true;
            }
        }
    }

    false
}

/// Check if a comment is inline (has code on the same line before AND after it)
fn is_inline_comment(source_text: &str, span: Span) -> bool {
    let start = span.start as usize;
    let end = span.end as usize;

    // Find the start of the line
    let line_start = source_text[..start].rfind('\n').map_or(0, |i| i + 1);
    // Find the end of the line
    let line_end = source_text[end..].find('\n').map_or(source_text.len(), |i| end + i);

    // Check if there's non-whitespace content before the comment on the same line
    let before = source_text[line_start..start].trim();
    let after = source_text[end..line_end].trim();

    // Must have content BOTH before AND after to be inline
    !before.is_empty() && !after.is_empty()
}

/// Check if two comments are consecutive (no code between them)
fn is_consecutive_comment(source_text: &str, prev: &Comment, curr: &Comment) -> bool {
    let between = &source_text[prev.span.end as usize..curr.span.start as usize];
    // Check if there's only whitespace between the comments
    between.chars().all(char::is_whitespace)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Basic uppercase (default "always")
        ("// Uppercase", None),
        ("/* Uppercase */", None),
        ("/** Uppercase */", None),
        // Numbers and non-letters at start
        ("// 123", None),
        ("/* 123 */", None),
        // Empty comments
        ("// ", None),
        ("/* */", None),
        // Directive comments
        ("// eslint-disable-line", None),
        ("/* eslint-disable */", None),
        ("/* global foo */", None),
        ("/* globals foo */", None),
        ("/* exported foo */", None),
        ("// istanbul ignore next", None),
        ("// jshint asi:true", None),
        ("// jscs: enable", None),
        // URLs (various schemes)
        ("// https://example.com", None),
        ("// http://example.com", None),
        ("// ftp://example.com", None),
        ("// file://path/to/file", None),
        ("// custom-scheme://resource", None),
        // JSDoc with URL on second line
        ("/**\n * https://example.com\n */", None),
        // Shebang
        ("#!/usr/bin/env node", None),
        // With "always" option
        ("// Uppercase", Some(serde_json::json!(["always"]))),
        // With "never" option
        ("// lowercase", Some(serde_json::json!(["never"]))),
        // ignorePattern (prefix match on normalized text)
        ("// pragma: no cover", Some(serde_json::json!(["always", { "ignorePattern": "pragma" }]))),
        // ignorePattern in JSDoc
        (
            "/**\n * pragma: something\n */",
            Some(serde_json::json!(["always", { "ignorePattern": "pragma" }])),
        ),
        // ignoreInlineComments
        (
            "foo(/* ignored */ a);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        // ignoreConsecutiveComments
        (
            "// Valid comment\n// following comment",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        // Non-cased letters (CJK, Hebrew, Arabic) - should be ignored
        ("// 你好世界", None),              // Chinese
        ("// こんにちは", None),            // Japanese Hiragana
        ("// مرحبا", None),                 // Arabic
        ("// שלום", None),                  // Hebrew
        ("/* 中文注释 */", None),           // Chinese in block comment
        // Non-cased with "never" option - should also pass
        ("// 你好", Some(serde_json::json!(["never"]))),
    ];

    let fail = vec![
        // Basic lowercase (default "always")
        ("// lowercase", None),
        ("/* lowercase */", None),
        ("/** lowercase */", None),
        // With "always" option
        ("// lowercase", Some(serde_json::json!(["always"]))),
        // With "never" option
        ("// Uppercase", Some(serde_json::json!(["never"]))),
        ("/* Uppercase */", Some(serde_json::json!(["never"]))),
        // Non-matching pattern
        ("// not matching", Some(serde_json::json!(["always", { "ignorePattern": "something" }]))),
    ];

    let fix = vec![
        // Lowercase to uppercase
        ("// lowercase", "// Lowercase", None),
        ("/* lowercase */", "/* Lowercase */", None),
        // Uppercase to lowercase with "never"
        ("// Uppercase", "// uppercase", Some(serde_json::json!(["never"]))),
        ("/* Uppercase */", "/* uppercase */", Some(serde_json::json!(["never"]))),
    ];

    Tester::new(CapitalizedComments::NAME, CapitalizedComments::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
