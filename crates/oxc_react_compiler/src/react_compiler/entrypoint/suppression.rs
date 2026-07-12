/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
use oxc_ast::ast::Comment;
use oxc_diagnostics::Diagnostics;

use crate::diagnostics::ErrorCategory;

#[derive(Debug, Clone, Copy)]
pub enum SuppressionSource {
    Eslint,
    Flow,
}

/// Captures the start and end range of a pair of eslint-disable ... eslint-enable comments.
/// In the case of a CommentLine or a relevant Flow suppression, both the disable and enable
/// point to the same comment.
///
/// The enable comment can be missing in the case where only a disable block is present,
/// ie the rest of the file has potential React violations.
#[derive(Debug, Clone, Copy)]
pub struct SuppressionRange {
    pub disable_comment: Comment,
    pub enable_comment: Option<Comment>,
    pub source: SuppressionSource,
}

/// The comment's text without `//` or `/* */` delimiters, trimmed. Matches how
/// the former Babel front-end populated comment values.
fn comment_value(comment: Comment, source_text: &str) -> &str {
    comment.content_span().source_text(source_text).trim()
}

/// Check if a comment value matches `eslint-disable-next-line <rule>` for any rule in `rule_names`.
fn matches_eslint_disable_next_line(value: &str, rule_names: &[String]) -> bool {
    value
        .strip_prefix("eslint-disable-next-line ")
        .is_some_and(|rest| rule_names.iter().any(|name| rest.starts_with(name.as_str())))
}

/// Check if a comment value matches `eslint-disable <rule>` for any rule in `rule_names`.
fn matches_eslint_disable(value: &str, rule_names: &[String]) -> bool {
    value
        .strip_prefix("eslint-disable ")
        .is_some_and(|rest| rule_names.iter().any(|name| rest.starts_with(name.as_str())))
}

/// Check if a comment value matches `eslint-enable <rule>` for any rule in `rule_names`.
fn matches_eslint_enable(value: &str, rule_names: &[String]) -> bool {
    value
        .strip_prefix("eslint-enable ")
        .is_some_and(|rest| rule_names.iter().any(|name| rest.starts_with(name.as_str())))
}

/// Check if a comment value matches a Flow suppression pattern.
/// Matches: $FlowFixMe[react-rule, $FlowFixMe_xxx[react-rule,
///          $FlowExpectedError[react-rule, $FlowIssue[react-rule
fn matches_flow_suppression(value: &str) -> bool {
    // Find "$Flow" anywhere in the value
    let Some(idx) = value.find("$Flow") else {
        return false;
    };
    let after_dollar_flow = &value[idx + "$Flow".len()..];

    // Match FlowFixMe (with optional word chars), FlowExpectedError, or FlowIssue
    let after_kind = if let Some(rest) = after_dollar_flow.strip_prefix("FixMe") {
        // Skip "FixMe" + any word characters
        let word_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(rest.len());
        &rest[word_end..]
    } else if let Some(rest) = after_dollar_flow.strip_prefix("ExpectedError") {
        rest
    } else if let Some(rest) = after_dollar_flow.strip_prefix("Issue") {
        rest
    } else {
        return false;
    };

    // Must be followed by "[react-rule"
    after_kind.starts_with("[react-rule")
}

/// Parse eslint-disable/enable and Flow suppression comments from program comments.
/// Equivalent to findProgramSuppressions in Suppression.ts
pub fn find_program_suppressions(
    comments: &[Comment],
    source_text: &str,
    rule_names: Option<&[String]>,
    flow_suppressions: bool,
) -> Vec<SuppressionRange> {
    let mut suppression_ranges: Vec<SuppressionRange> = Vec::new();
    let mut disable_comment: Option<Comment> = None;
    let mut enable_comment: Option<Comment> = None;
    let mut source: Option<SuppressionSource> = None;

    let rule_names = rule_names.filter(|names| !names.is_empty());

    for &comment in comments {
        let value = comment_value(comment, source_text);

        // Check for eslint-disable-next-line (only if not already within a block)
        if disable_comment.is_none() {
            if let Some(names) = rule_names {
                if matches_eslint_disable_next_line(value, names) {
                    disable_comment = Some(comment);
                    enable_comment = Some(comment);
                    source = Some(SuppressionSource::Eslint);
                }
            }
        }

        // Check for Flow suppression (only if not already within a block)
        if flow_suppressions && disable_comment.is_none() && matches_flow_suppression(value) {
            disable_comment = Some(comment);
            enable_comment = Some(comment);
            source = Some(SuppressionSource::Flow);
        }

        if let Some(names) = rule_names {
            // Check for eslint-disable (block start)
            if matches_eslint_disable(value, names) {
                disable_comment = Some(comment);
                source = Some(SuppressionSource::Eslint);
            }

            // Check for eslint-enable (block end)
            if matches_eslint_enable(value, names)
                && matches!(source, Some(SuppressionSource::Eslint))
            {
                enable_comment = Some(comment);
            }
        }

        // If we have a complete suppression, push it
        if disable_comment.is_some() && source.is_some() {
            suppression_ranges.push(SuppressionRange {
                disable_comment: disable_comment.take().unwrap(),
                enable_comment: enable_comment.take(),
                source: source.take().unwrap(),
            });
        }
    }

    suppression_ranges
}

/// Check if suppression ranges overlap with a function's source range.
/// A suppression affects a function if:
/// 1. The suppression is within the function's body
/// 2. The suppression wraps the function
pub fn filter_suppressions_that_affect_function(
    suppressions: &[SuppressionRange],
    fn_start: u32,
    fn_end: u32,
) -> Vec<SuppressionRange> {
    let mut suppressions_in_scope: Vec<SuppressionRange> = Vec::new();

    for suppression in suppressions {
        let disable_start = suppression.disable_comment.span.start;
        let enable_end = suppression.enable_comment.map(|c| c.span.end);

        // The suppression is within the function
        if disable_start > fn_start && enable_end.is_none_or(|end| end < fn_end) {
            suppressions_in_scope.push(*suppression);
        }

        // The suppression wraps the function
        if disable_start < fn_start && enable_end.is_none_or(|end| end > fn_end) {
            suppressions_in_scope.push(*suppression);
        }
    }

    suppressions_in_scope
}

/// Convert suppression ranges to diagnostics.
pub fn suppressions_to_diagnostics(
    suppressions: &[SuppressionRange],
    source_text: &str,
) -> Diagnostics {
    assert!(!suppressions.is_empty(), "Expected at least one suppression comment source range");

    let mut error = Diagnostics::new();

    for suppression in suppressions {
        let reason = match suppression.source {
            SuppressionSource::Eslint => {
                "React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled"
            }
            SuppressionSource::Flow => {
                "React Compiler has skipped optimizing this component because one or more React rule violations were reported by Flow"
            }
        };

        let description = format!(
            "React Compiler only works when your components follow all the rules of React, disabling them may result in unexpected or incorrect behavior. Found suppression `{}`",
            comment_value(suppression.disable_comment, source_text)
        );

        error.push(
            ErrorCategory::Suppression
                .diagnostic(reason)
                .with_help(description)
                .with_label(suppression.disable_comment.span.label("Found React rule suppression")),
        );
    }

    error
}
