/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
use crate::react_compiler_diagnostics::{
    CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, CompilerSuggestion,
    CompilerSuggestionOperation, ErrorCategory, Position, SourceLocation,
};

/// A comment's text and byte range, plus the byte-offset loc that surfaces as the
/// labeled span on a suppression diagnostic. The former Babel front-end carried
/// this on `CommentData`; the oxc front-end builds it directly from oxc comments.
#[derive(Debug, Clone)]
pub struct CommentData {
    pub value: String,
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub loc: Option<CommentLoc>,
}

#[derive(Debug, Clone)]
pub struct CommentLoc {
    pub start_index: Option<u32>,
    pub end_index: Option<u32>,
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct SuppressionRange {
    pub disable_comment: CommentData,
    pub enable_comment: Option<CommentData>,
    pub source: SuppressionSource,
}

/// Build a Babel-shaped [`CommentData`] from an oxc comment, stripping the `//`
/// or `/* */` delimiters from the value exactly as the former Babel bridge did.
fn comment_data(comment: &oxc_ast::ast::Comment, source_text: &str) -> CommentData {
    let raw = &source_text[comment.span.start as usize..comment.span.end as usize];
    let value = if matches!(comment.kind, oxc_ast::ast::CommentKind::Line) {
        raw.strip_prefix("//").unwrap_or(raw).trim().to_string()
    } else {
        raw.strip_prefix("/*").unwrap_or(raw).strip_suffix("*/").unwrap_or(raw).trim().to_string()
    };
    CommentData {
        value,
        start: Some(comment.span.start),
        end: Some(comment.span.end),
        // Only the byte `index` is load-bearing here: it surfaces as the labeled
        // span offset/length on the suppression diagnostic. Line/column are unused
        // by downstream consumers of this loc.
        loc: Some(CommentLoc {
            start_index: Some(comment.span.start),
            end_index: Some(comment.span.end),
        }),
    }
}

/// Check if a comment value matches `eslint-disable-next-line <rule>` for any rule in `rule_names`.
fn matches_eslint_disable_next_line(value: &str, rule_names: &[String]) -> bool {
    if let Some(rest) = value.strip_prefix("eslint-disable-next-line ") {
        return rule_names.iter().any(|name| rest.starts_with(name.as_str()));
    }
    // Also check with leading space (comment values often have leading whitespace)
    let trimmed = value.trim_start();
    if let Some(rest) = trimmed.strip_prefix("eslint-disable-next-line ") {
        return rule_names.iter().any(|name| rest.starts_with(name.as_str()));
    }
    false
}

/// Check if a comment value matches `eslint-disable <rule>` for any rule in `rule_names`.
fn matches_eslint_disable(value: &str, rule_names: &[String]) -> bool {
    if let Some(rest) = value.strip_prefix("eslint-disable ") {
        return rule_names.iter().any(|name| rest.starts_with(name.as_str()));
    }
    let trimmed = value.trim_start();
    if let Some(rest) = trimmed.strip_prefix("eslint-disable ") {
        return rule_names.iter().any(|name| rest.starts_with(name.as_str()));
    }
    false
}

/// Check if a comment value matches `eslint-enable <rule>` for any rule in `rule_names`.
fn matches_eslint_enable(value: &str, rule_names: &[String]) -> bool {
    if let Some(rest) = value.strip_prefix("eslint-enable ") {
        return rule_names.iter().any(|name| rest.starts_with(name.as_str()));
    }
    let trimmed = value.trim_start();
    if let Some(rest) = trimmed.strip_prefix("eslint-enable ") {
        return rule_names.iter().any(|name| rest.starts_with(name.as_str()));
    }
    false
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
    let after_kind = if after_dollar_flow.starts_with("FixMe") {
        // Skip "FixMe" + any word characters
        let rest = &after_dollar_flow["FixMe".len()..];
        let word_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(rest.len());
        &rest[word_end..]
    } else if after_dollar_flow.starts_with("ExpectedError") {
        &after_dollar_flow["ExpectedError".len()..]
    } else if after_dollar_flow.starts_with("Issue") {
        &after_dollar_flow["Issue".len()..]
    } else {
        return false;
    };

    // Must be followed by "[react-rule"
    after_kind.starts_with("[react-rule")
}

/// Parse eslint-disable/enable and Flow suppression comments from program comments.
/// Equivalent to findProgramSuppressions in Suppression.ts
pub fn find_program_suppressions(
    comments: &[oxc_ast::ast::Comment],
    source_text: &str,
    rule_names: Option<&[String]>,
    flow_suppressions: bool,
) -> Vec<SuppressionRange> {
    let mut suppression_ranges: Vec<SuppressionRange> = Vec::new();
    let mut disable_comment: Option<CommentData> = None;
    let mut enable_comment: Option<CommentData> = None;
    let mut source: Option<SuppressionSource> = None;

    let has_rules = matches!(rule_names, Some(names) if !names.is_empty());

    for comment in comments {
        let data = comment_data(comment, source_text);

        if data.start.is_none() || data.end.is_none() {
            continue;
        }

        // Check for eslint-disable-next-line (only if not already within a block)
        if disable_comment.is_none() && has_rules {
            if let Some(names) = rule_names {
                if matches_eslint_disable_next_line(&data.value, names) {
                    disable_comment = Some(data.clone());
                    enable_comment = Some(data.clone());
                    source = Some(SuppressionSource::Eslint);
                }
            }
        }

        // Check for Flow suppression (only if not already within a block)
        if flow_suppressions && disable_comment.is_none() && matches_flow_suppression(&data.value) {
            disable_comment = Some(data.clone());
            enable_comment = Some(data.clone());
            source = Some(SuppressionSource::Flow);
        }

        // Check for eslint-disable (block start)
        if has_rules {
            if let Some(names) = rule_names {
                if matches_eslint_disable(&data.value, names) {
                    disable_comment = Some(data.clone());
                    source = Some(SuppressionSource::Eslint);
                }
            }
        }

        // Check for eslint-enable (block end)
        if has_rules {
            if let Some(names) = rule_names {
                if matches_eslint_enable(&data.value, names) {
                    if matches!(source, Some(SuppressionSource::Eslint)) {
                        enable_comment = Some(data.clone());
                    }
                }
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
) -> Vec<&SuppressionRange> {
    let mut suppressions_in_scope: Vec<&SuppressionRange> = Vec::new();

    for suppression in suppressions {
        let disable_start = match suppression.disable_comment.start {
            Some(s) => s,
            None => continue,
        };

        // The suppression is within the function
        if disable_start > fn_start
            && (suppression.enable_comment.is_none()
                || suppression
                    .enable_comment
                    .as_ref()
                    .and_then(|c| c.end)
                    .map_or(false, |end| end < fn_end))
        {
            suppressions_in_scope.push(suppression);
        }

        // The suppression wraps the function
        if disable_start < fn_start
            && (suppression.enable_comment.is_none()
                || suppression
                    .enable_comment
                    .as_ref()
                    .and_then(|c| c.end)
                    .map_or(false, |end| end > fn_end))
        {
            suppressions_in_scope.push(suppression);
        }
    }

    suppressions_in_scope
}

/// Convert suppression ranges to a CompilerError.
pub fn suppressions_to_compiler_error(suppressions: &[SuppressionRange]) -> CompilerError {
    assert!(!suppressions.is_empty(), "Expected at least one suppression comment source range");

    let mut error = CompilerError::new();

    for suppression in suppressions {
        let (disable_start, disable_end) =
            match (suppression.disable_comment.start, suppression.disable_comment.end) {
                (Some(s), Some(e)) => (s, e),
                _ => continue,
            };

        let (reason, suggestion) = match suppression.source {
            SuppressionSource::Eslint => (
                "React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled",
                "Remove the ESLint suppression and address the React error",
            ),
            SuppressionSource::Flow => (
                "React Compiler has skipped optimizing this component because one or more React rule violations were reported by Flow",
                "Remove the Flow suppression and address the React error",
            ),
        };

        let description = format!(
            "React Compiler only works when your components follow all the rules of React, disabling them may result in unexpected or incorrect behavior. Found suppression `{}`",
            suppression.disable_comment.value.trim()
        );

        let mut diagnostic =
            CompilerDiagnostic::new(ErrorCategory::Suppression, reason, Some(description));

        diagnostic.suggestions = Some(vec![CompilerSuggestion {
            description: suggestion.to_string(),
            range: (disable_start as usize, disable_end as usize),
            op: CompilerSuggestionOperation::Remove,
            text: None,
        }]);

        // Add error detail with location info
        let loc = suppression.disable_comment.loc.as_ref().map(|l| SourceLocation {
            start: Position { line: 0, column: 0, index: l.start_index },
            end: Position { line: 0, column: 0, index: l.end_index },
        });

        diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Error {
            loc,
            message: Some("Found React rule suppression".to_string()),
            identifier_name: None,
        });

        error.push_diagnostic(diagnostic);
    }

    error
}
