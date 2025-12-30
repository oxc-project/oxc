use lazy_regex::{Regex, RegexBuilder};
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::Comment;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

/// Common directive prefixes that should be ignored (module-level to avoid items-after-statements)
const DIRECTIVES: &[&str] = &[
    "eslint-disable",
    "eslint-enable",
    "eslint-env",
    "eslint ",
    "oxlint-disable",
    "oxlint-enable",
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
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

impl CommentConfigJson {
    fn into_comment_config(self, base: &CommentConfigJson) -> CommentConfig {
        let pattern = self.ignore_pattern.as_ref().or(base.ignore_pattern.as_ref());
        CommentConfig {
            ignore_pattern: pattern
                .and_then(|p| RegexBuilder::new(&format!(r"^\s*(?:{p})")).build().ok()),
            ignore_inline_comments: self
                .ignore_inline_comments
                .or(base.ignore_inline_comments)
                .unwrap_or(false),
            ignore_consecutive_comments: self
                .ignore_consecutive_comments
                .or(base.ignore_consecutive_comments)
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct OptionsJson {
    #[serde(flatten)]
    base: CommentConfigJson,
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
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let Some(arr) = value.as_array() else {
            return Ok(Self::default());
        };

        let capitalize =
            arr.first().and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();

        let options: OptionsJson =
            arr.get(1).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();

        let line_config = options.line.unwrap_or_default().into_comment_config(&options.base);
        let block_config = options.block.unwrap_or_default().into_comment_config(&options.base);

        Ok(Self(Box::new(CapitalizedCommentsConfig { capitalize, line_config, block_config })))
    }

    fn run_once(&self, ctx: &LintContext) {
        let source_text = ctx.source_text();
        let comments = ctx.semantic().comments();

        // Skip shebang comment if present
        let comments_iter =
            comments.iter().skip_while(|c| c.span.start == 0 && source_text.starts_with("#!"));

        // Iterate with (prev, current) pairs to check consecutive comments
        let iter = std::iter::once(None).chain(comments_iter.clone().map(Some)).zip(comments_iter);

        for (prev_comment, comment) in iter {
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

            // Check if this is a directive comment (using original content, not normalized)
            // ESLint only ignores directives when they start at the beginning (after whitespace)
            // e.g., "// eslint-disable" is ignored, but "//* eslint-disable" is not
            if is_directive_comment(content) {
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
                && prev_comment
                    .is_some_and(|prev| is_consecutive_comment(source_text, prev, comment))
            {
                continue;
            }

            // Check capitalization using ESLint's approach:
            // - A letter is "uppercase" if it differs from its lowercase form
            // - A letter is "lowercase" if it differs from its uppercase form
            // - Letters with no case distinction (CJK, Hebrew, Arabic, etc.) are skipped
            let lower = first_letter.to_lowercase().collect::<String>();
            let upper = first_letter.to_uppercase().collect::<String>();

            // Skip letters with no case distinction (e.g., CJK characters)
            if lower == upper {
                continue;
            }

            let is_uppercase = first_letter.is_uppercase();
            let (wrong_case, correct_case, fixed_letter) = match self.capitalize {
                CapitalizeOption::Always if !is_uppercase => ("lowercase", "uppercase", upper),
                CapitalizeOption::Never if is_uppercase => ("uppercase", "lowercase", lower),
                _ => continue,
            };

            // Calculate the span for the first letter
            #[expect(clippy::cast_possible_truncation)]
            let letter_start = comment.content_span().start + first_letter_offset as u32;
            #[expect(clippy::cast_possible_truncation)]
            let letter_end = letter_start + first_letter.len_utf8() as u32;
            let letter_span = Span::new(letter_start, letter_end);

            ctx.diagnostic_with_fix(
                capitalized_comments_diagnostic(comment.span, wrong_case, correct_case),
                |fixer| fixer.replace(letter_span, fixed_letter.clone()),
            );
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
    use itertools::Itertools;
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            // Remove leading * and following whitespace
            trimmed.strip_prefix('*').map_or(trimmed, str::trim_start)
        })
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
        // Non-letter, non-whitespace character (digit or special char) means comment is "non-cased"
        if !c.is_whitespace() {
            return None;
        }
    }
    None
}

/// Check if the comment content is a directive comment that should be ignored
fn is_directive_comment(content: &str) -> bool {
    let trimmed = content.trim_start();

    // Oxlint/ESLint disable/enable directives.
    // e.g. `eslint-disable-next-line`, `oxlint-disable-line`, etc.
    if trimmed.starts_with("eslint-") || trimmed.starts_with("oxlint-") {
        return true;
    }

    // Check known directive prefixes
    if DIRECTIVES.iter().any(|d| trimmed.starts_with(d)) {
        return true;
    }

    // Check if it looks like a URL (any scheme://)
    // This matches ESLint's behavior which uses a general pattern
    trimmed.find(':').is_some_and(|colon_pos| {
        trimmed.get(colon_pos..colon_pos + 3) == Some("://") && {
            let scheme = &trimmed[..colon_pos];
            !scheme.is_empty()
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        }
    })
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
        ("//Uppercase", None),
        ("// Uppercase", None),
        ("/*Uppercase */", None),
        ("/* Uppercase */", None),
        (
            "/*
			Uppercase */",
            None,
        ),
        ("/** Uppercase */", None),
        (
            "/**
			Uppercase */",
            None,
        ),
        ("//Über", None),
        ("//Π", None),
        (
            "/* Uppercase
			second line need not be uppercase */",
            None,
        ),
        ("// ", None),
        ("//	", None),
        ("/* */", None),
        ("/*	*/", None),
        (
            "/*
			*/", None,
        ),
        (
            "/*
*/", None,
        ),
        (
            "/*
			*/", None,
        ),
        ("/* */", None),
        ("/* */", None),
        ("//123", None),
        ("// 123", None),
        ("/*123*/", None),
        ("/* 123 */", None),
        ("/**123 */", None),
        ("/** 123 */", None),
        (
            "/**
			123 */",
            None,
        ),
        (
            "/*
			123 */",
            None,
        ),
        (
            "/*123
			second line need not be uppercase */",
            None,
        ),
        (
            "/**
			 * @fileoverview This is a file */",
            None,
        ),
        ("// jscs: enable", None),
        ("// jscs:disable", None),
        ("// eslint-disable-line", None),
        ("// eslint-disable-next-line", None),
        ("// oxlint-disable-line", None),
        ("// oxlint-disable-next-line", None),
        ("/* eslint semi:off */", None),
        ("/* eslint-enable */", None),
        ("/* oxlint-disable */", None),
        ("/* oxlint-enable */", None),
        ("/* istanbul ignore next */", None),
        ("/* jshint asi:true */", None),
        ("/* jscs: enable */", None),
        ("/* global var1, var2 */", None),
        ("/* global var1:true, var2 */", None),
        ("/* globals var1, var2 */", None),
        ("/* globals var1:true, var2 */", None),
        ("/* exported myVar */", None),
        ("#!foo", None),
        ("#!foo", Some(serde_json::json!(["always"]))),
        ("#!Foo", Some(serde_json::json!(["never"]))),
        ("#!/usr/bin/env node", None),
        ("#!/usr/bin/env node", Some(serde_json::json!(["always"]))),
        ("#!/usr/bin/env node", Some(serde_json::json!(["never"]))),
        ("//Uppercase", Some(serde_json::json!(["always"]))),
        ("// Uppercase", Some(serde_json::json!(["always"]))),
        ("/*Uppercase */", Some(serde_json::json!(["always"]))),
        ("/* Uppercase */", Some(serde_json::json!(["always"]))),
        (
            "/*
			Uppercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("/** Uppercase */", Some(serde_json::json!(["always"]))),
        (
            "/**
			Uppercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("//Über", Some(serde_json::json!(["always"]))),
        ("//Π", Some(serde_json::json!(["always"]))),
        (
            "/* Uppercase
			second line need not be uppercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("//123", Some(serde_json::json!(["always"]))),
        ("// 123", Some(serde_json::json!(["always"]))),
        ("/*123*/", Some(serde_json::json!(["always"]))),
        ("/**123*/", Some(serde_json::json!(["always"]))),
        ("/* 123 */", Some(serde_json::json!(["always"]))),
        ("/** 123*/", Some(serde_json::json!(["always"]))),
        (
            "/**
			123*/",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			123 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*123
			second line need not be uppercase */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/**
			 @todo: foobar
			 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/**
			 * @fileoverview This is a file */",
            Some(serde_json::json!(["always"])),
        ),
        ("// jscs: enable", Some(serde_json::json!(["always"]))),
        ("// jscs:disable", Some(serde_json::json!(["always"]))),
        ("// eslint-disable-line", Some(serde_json::json!(["always"]))),
        ("// eslint-disable-next-line", Some(serde_json::json!(["always"]))),
        ("// oxlint-disable-line", Some(serde_json::json!(["always"]))),
        ("// oxlint-disable-next-line", Some(serde_json::json!(["always"]))),
        ("/* eslint semi:off */", Some(serde_json::json!(["always"]))),
        ("/* eslint-enable */", Some(serde_json::json!(["always"]))),
        ("/* oxlint-disable */", Some(serde_json::json!(["always"]))),
        ("/* oxlint-enable */", Some(serde_json::json!(["always"]))),
        ("/* istanbul ignore next */", Some(serde_json::json!(["always"]))),
        ("/* jshint asi:true */", Some(serde_json::json!(["always"]))),
        ("/* jscs: enable */", Some(serde_json::json!(["always"]))),
        ("/* global var1, var2 */", Some(serde_json::json!(["always"]))),
        ("/* global var1:true, var2 */", Some(serde_json::json!(["always"]))),
        ("/* globals var1, var2 */", Some(serde_json::json!(["always"]))),
        ("/* globals var1:true, var2 */", Some(serde_json::json!(["always"]))),
        ("/* exported myVar */", Some(serde_json::json!(["always"]))),
        ("//lowercase", Some(serde_json::json!(["never"]))),
        ("// lowercase", Some(serde_json::json!(["never"]))),
        ("/*lowercase */", Some(serde_json::json!(["never"]))),
        ("/* lowercase */", Some(serde_json::json!(["never"]))),
        (
            "/*
			lowercase */",
            Some(serde_json::json!(["never"])),
        ),
        ("//über", Some(serde_json::json!(["never"]))),
        ("//π", Some(serde_json::json!(["never"]))),
        (
            "/* lowercase
			Second line need not be lowercase */",
            Some(serde_json::json!(["never"])),
        ),
        ("//123", Some(serde_json::json!(["never"]))),
        ("// 123", Some(serde_json::json!(["never"]))),
        ("/*123*/", Some(serde_json::json!(["never"]))),
        ("/* 123 */", Some(serde_json::json!(["never"]))),
        (
            "/*
			123 */",
            Some(serde_json::json!(["never"])),
        ),
        (
            "/*123
			second line need not be uppercase */",
            Some(serde_json::json!(["never"])),
        ),
        (
            "/**
			 @TODO: foobar
			 */",
            Some(serde_json::json!(["never"])),
        ),
        (
            "/**
			 * @Fileoverview This is a file */",
            Some(serde_json::json!(["never"])),
        ),
        ("// matching", Some(serde_json::json!(["always", { "ignorePattern": "match" }]))),
        ("// Matching", Some(serde_json::json!(["never", { "ignorePattern": "Match" }]))),
        ("// bar", Some(serde_json::json!(["always", { "ignorePattern": "foo|bar" }]))),
        ("// Bar", Some(serde_json::json!(["never", { "ignorePattern": "Foo|Bar" }]))),
        (
            "foo(/* ignored */ a);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(/* Ignored */ a);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(/*
			ignored */ a);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(/*
			Ignored */ a);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "// This comment is valid since it is capitalized,
			// and this one is valid since it follows a valid one,
			// and same with this one.",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        (
            "/* This comment is valid since it is capitalized, */
			/* and this one is valid since it follows a valid one, */
			/* and same with this one. */",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        (
            "/*
			 * This comment is valid since it is capitalized,
			 */
			/* and this one is valid since it follows a valid one, */
			/*
			 * and same with this one.
			 */",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        (
            "// This comment is valid since it is capitalized,
			// and this one is valid since it follows a valid one,
			foo();
			// This comment now has to be capitalized.",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        ("// https://github.com", Some(serde_json::json!(["always"]))),
        ("// HTTPS://GITHUB.COM", Some(serde_json::json!(["never"]))),
        (
            "// Valid capitalized line comment
			/* Valid capitalized block comment */
			// lineCommentIgnorePattern
			/* blockCommentIgnorePattern */",
            Some(
                serde_json::json!([				"always",				{					"line": {						"ignorePattern": "lineCommentIgnorePattern",					},					"block": {						"ignorePattern": "blockCommentIgnorePattern",					},				},			]),
            ),
        ),
    ];

    let fail = vec![
        ("//lowercase", None),
        ("// lowercase", None),
        ("/*lowercase */", None),
        ("/* lowercase */", None),
        ("/** lowercase */", None),
        (
            "/*
			lowercase */",
            None,
        ),
        (
            "/**
			lowercase */",
            None,
        ),
        ("//über", None),
        ("//π", None),
        (
            "/* lowercase
			Second line need not be lowercase */",
            None,
        ),
        ("// ꮳꮃꭹ", None),
        ("/* 𐳡𐳡𐳡 */", None),
        ("//lowercase", Some(serde_json::json!(["always"]))),
        ("// lowercase", Some(serde_json::json!(["always"]))),
        ("/*lowercase */", Some(serde_json::json!(["always"]))),
        ("/* lowercase */", Some(serde_json::json!(["always"]))),
        ("/** lowercase */", Some(serde_json::json!(["always"]))),
        (
            "/**
			lowercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("//über", Some(serde_json::json!(["always"]))),
        ("//π", Some(serde_json::json!(["always"]))),
        (
            "/* lowercase
			Second line need not be lowercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("//Uppercase", Some(serde_json::json!(["never"]))),
        ("// Uppercase", Some(serde_json::json!(["never"]))),
        ("/*Uppercase */", Some(serde_json::json!(["never"]))),
        ("/* Uppercase */", Some(serde_json::json!(["never"]))),
        (
            "/*
			Uppercase */",
            Some(serde_json::json!(["never"])),
        ),
        ("//Über", Some(serde_json::json!(["never"]))),
        ("//Π", Some(serde_json::json!(["never"]))),
        (
            "/* Uppercase
			second line need not be uppercase */",
            Some(serde_json::json!(["never"])),
        ),
        ("// Გ", Some(serde_json::json!(["never"]))),
        ("// 𑢢", Some(serde_json::json!(["never"]))),
        ("//* jscs: enable", Some(serde_json::json!(["always"]))),
        ("//* jscs:disable", Some(serde_json::json!(["always"]))),
        ("//* eslint-disable-line", Some(serde_json::json!(["always"]))),
        ("//* eslint-disable-next-line", Some(serde_json::json!(["always"]))),
        (
            "/*
			 * eslint semi:off */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 * eslint-env node */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  istanbul ignore next */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  jshint asi:true */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  jscs: enable */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  global var1, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  global var1:true, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  globals var1, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  globals var1:true, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  exported myVar */",
            Some(serde_json::json!(["always"])),
        ),
        ("foo(/* invalid */a);", Some(serde_json::json!(["always"]))),
        (
            "foo(/* invalid */a);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": false }])),
        ),
        (
            "foo(a, // not an inline comment
			b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a, /* not an inline comment */
			b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* not an inline comment */b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* not an inline comment */
			b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a, // Not an inline comment
			b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a, /* Not an inline comment */
			b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* Not an inline comment */b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* Not an inline comment */
			b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        ("// not matching", Some(serde_json::json!(["always", { "ignorePattern": "ignored?" }]))),
        ("// Not matching", Some(serde_json::json!(["never", { "ignorePattern": "ignored?" }]))),
        (
            "// This comment is valid since it is capitalized,
			// and this one is valid since it follows a valid one,
			foo();
			// this comment is now invalid.",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        (
            "// this comment is invalid since it is not capitalized,
			// but this one is ignored since it is consecutive.",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": true }])),
        ),
        (
            "// This comment is invalid since it is not capitalized,
			// But this one is ignored since it is consecutive.",
            Some(serde_json::json!(["never", { "ignoreConsecutiveComments": true }])),
        ),
        (
            "// This comment is valid since it is capitalized,
			// but this one is invalid even if it follows a valid one.",
            Some(serde_json::json!(["always", { "ignoreConsecutiveComments": false }])),
        ),
        ("// should fail. https://github.com", Some(serde_json::json!(["always"]))),
        ("// Should fail. https://github.com", Some(serde_json::json!(["never"]))),
    ];

    let fix = vec![
        ("//lowercase", "//Lowercase", None),
        ("// lowercase", "// Lowercase", None),
        ("/*lowercase */", "/*Lowercase */", None),
        ("/* lowercase */", "/* Lowercase */", None),
        ("/** lowercase */", "/** Lowercase */", None),
        (
            "/*
			lowercase */",
            "/*
			Lowercase */",
            None,
        ),
        (
            "/**
			lowercase */",
            "/**
			Lowercase */",
            None,
        ),
        ("//über", "//Über", None),
        ("//π", "//Π", None),
        (
            "/* lowercase
			Second line need not be lowercase */",
            "/* Lowercase
			Second line need not be lowercase */",
            None,
        ),
        ("// ꮳꮃꭹ", "// Ꮳꮃꭹ", None),
        ("/* 𐳡𐳡𐳡 */", "/* 𐲡𐳡𐳡 */", None),
        ("//lowercase", "//Lowercase", Some(serde_json::json!(["always"]))),
        ("// lowercase", "// Lowercase", Some(serde_json::json!(["always"]))),
        ("/*lowercase */", "/*Lowercase */", Some(serde_json::json!(["always"]))),
        ("/* lowercase */", "/* Lowercase */", Some(serde_json::json!(["always"]))),
        ("/** lowercase */", "/** Lowercase */", Some(serde_json::json!(["always"]))),
        (
            "/**
			lowercase */",
            "/**
			Lowercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("//über", "//Über", Some(serde_json::json!(["always"]))),
        ("//π", "//Π", Some(serde_json::json!(["always"]))),
        (
            "/* lowercase
			Second line need not be lowercase */",
            "/* Lowercase
			Second line need not be lowercase */",
            Some(serde_json::json!(["always"])),
        ),
        ("//Uppercase", "//uppercase", Some(serde_json::json!(["never"]))),
        ("// Uppercase", "// uppercase", Some(serde_json::json!(["never"]))),
        ("/*Uppercase */", "/*uppercase */", Some(serde_json::json!(["never"]))),
        ("/* Uppercase */", "/* uppercase */", Some(serde_json::json!(["never"]))),
        (
            "/*
			Uppercase */",
            "/*
			uppercase */",
            Some(serde_json::json!(["never"])),
        ),
        ("//Über", "//über", Some(serde_json::json!(["never"]))),
        ("//Π", "//π", Some(serde_json::json!(["never"]))),
        (
            "/* Uppercase
			second line need not be uppercase */",
            "/* uppercase
			second line need not be uppercase */",
            Some(serde_json::json!(["never"])),
        ),
        ("// Გ", "// გ", Some(serde_json::json!(["never"]))),
        ("// 𑢢", "// 𑣂", Some(serde_json::json!(["never"]))),
        ("//* jscs: enable", "//* Jscs: enable", Some(serde_json::json!(["always"]))),
        ("//* jscs:disable", "//* Jscs:disable", Some(serde_json::json!(["always"]))),
        ("//* eslint-disable-line", "//* Eslint-disable-line", Some(serde_json::json!(["always"]))),
        (
            "//* eslint-disable-next-line",
            "//* Eslint-disable-next-line",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 * eslint semi:off */",
            "/*
			 * Eslint semi:off */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 * eslint-env node */",
            "/*
			 * Eslint-env node */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  istanbul ignore next */",
            "/*
			 *  Istanbul ignore next */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  jshint asi:true */",
            "/*
			 *  Jshint asi:true */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  jscs: enable */",
            "/*
			 *  Jscs: enable */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  global var1, var2 */",
            "/*
			 *  Global var1, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  global var1:true, var2 */",
            "/*
			 *  Global var1:true, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  globals var1, var2 */",
            "/*
			 *  Globals var1, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  globals var1:true, var2 */",
            "/*
			 *  Globals var1:true, var2 */",
            Some(serde_json::json!(["always"])),
        ),
        (
            "/*
			 *  exported myVar */",
            "/*
			 *  Exported myVar */",
            Some(serde_json::json!(["always"])),
        ),
        ("foo(/* invalid */a);", "foo(/* Invalid */a);", Some(serde_json::json!(["always"]))),
        (
            "foo(/* invalid */a);",
            "foo(/* Invalid */a);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": false }])),
        ),
        (
            "foo(a, // not an inline comment
			b);",
            "foo(a, // Not an inline comment
			b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a, /* not an inline comment */
			b);",
            "foo(a, /* Not an inline comment */
			b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* not an inline comment */b);",
            "foo(a,
			/* Not an inline comment */b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* not an inline comment */
			b);",
            "foo(a,
			/* Not an inline comment */
			b);",
            Some(serde_json::json!(["always", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a, // Not an inline comment
			b);",
            "foo(a, // not an inline comment
			b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a, /* Not an inline comment */
			b);",
            "foo(a, /* not an inline comment */
			b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* Not an inline comment */b);",
            "foo(a,
			/* not an inline comment */b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "foo(a,
			/* Not an inline comment */
			b);",
            "foo(a,
			/* not an inline comment */
			b);",
            Some(serde_json::json!(["never", { "ignoreInlineComments": true }])),
        ),
        (
            "// not matching",
            "// Not matching",
            Some(serde_json::json!(["always", { "ignorePattern": "ignored?" }])),
        ),
        (
            "// Not matching",
            "// not matching",
            Some(serde_json::json!(["never", { "ignorePattern": "ignored?" }])),
        ),
        (
            "// should fail. https://github.com",
            "// Should fail. https://github.com",
            Some(serde_json::json!(["always"])),
        ),
        (
            "// Should fail. https://github.com",
            "// should fail. https://github.com",
            Some(serde_json::json!(["never"])),
        ),
    ];
    Tester::new(CapitalizedComments::NAME, CapitalizedComments::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
