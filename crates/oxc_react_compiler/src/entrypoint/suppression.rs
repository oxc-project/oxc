/// ESLint suppression handling.
///
/// Port of `Entrypoint/Suppression.ts` from the React Compiler.
///
/// Detects ESLint disable comments that suppress React-related rules,
/// which indicates the code may be breaking React rules and should be
/// skipped during compilation.
use oxc_ast::ast::Comment;
use oxc_span::Span;

use crate::compiler_error::{
    CompilerError, CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory, SourceLocation,
};

/// A suppression range from an eslint-disable comment pair.
#[derive(Debug, Clone)]
pub struct SuppressionRange {
    pub start: u32,
    pub end: Option<u32>,
    pub source: SuppressionSource,
}

/// Source of a suppression (ESLint or Flow).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionSource {
    Eslint,
    Flow,
}

/// Default ESLint rules that indicate React rule suppressions.
pub const DEFAULT_ESLINT_SUPPRESSION_RULES: &[&str] =
    &["react-hooks/rules-of-hooks", "react-hooks/exhaustive-deps"];

/// Check if comment text matches `eslint-disable-next-line <rule>` for any of the given rules.
fn matches_disable_next_line(text: &str, rule_names: &[String]) -> bool {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("eslint-disable-next-line") {
        let rest = rest.trim();
        if rest.is_empty() {
            // Blanket disable-next-line — matches
            return true;
        }
        return rule_names.iter().any(|rule| rest.contains(rule.as_str()));
    }
    false
}

/// Check if comment text matches `eslint-disable <rule>` for any of the given rules.
fn matches_disable(text: &str, rule_names: &[String]) -> bool {
    let trimmed = text.trim();
    // Must match "eslint-disable " but NOT "eslint-disable-next-line"
    if let Some(rest) = trimmed.strip_prefix("eslint-disable") {
        // Make sure it's not "eslint-disable-next-line"
        if rest.starts_with('-') {
            return false;
        }
        let rest = rest.trim();
        if rest.is_empty() {
            // Blanket disable
            return true;
        }
        return rule_names.iter().any(|rule| rest.contains(rule.as_str()));
    }
    false
}

/// Check if comment text matches `eslint-enable <rule>` for any of the given rules.
fn matches_enable(text: &str, rule_names: &[String]) -> bool {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("eslint-enable") {
        let rest = rest.trim();
        if rest.is_empty() {
            return true;
        }
        return rule_names.iter().any(|rule| rest.contains(rule.as_str()));
    }
    false
}

/// Check if comment text matches a Flow suppression pattern.
///
/// Matches patterns like `$FlowFixMe[react-rule`, `$FlowExpectedError[react-rule`,
/// `$FlowIssue[react-rule`.
fn matches_flow_suppression(text: &str) -> bool {
    let trimmed = text.trim();
    // Look for $FlowFixMe*, $FlowExpectedError, $FlowIssue followed by [react-rule
    for prefix in &["$FlowFixMe", "$FlowExpectedError", "$FlowIssue"] {
        if let Some(pos) = trimmed.find(prefix) {
            let after = &trimmed[pos + prefix.len()..];
            // For $FlowFixMe, skip any word chars (e.g. $FlowFixMeSomething)
            let after = if *prefix == "$FlowFixMe" {
                let skip = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').count();
                &after[skip..]
            } else {
                after
            };
            if after.starts_with("[react-rule") {
                return true;
            }
        }
    }
    false
}

/// Find all program-level suppression ranges from comments.
///
/// Port of `findProgramSuppressions` from `Suppression.ts`.
///
/// Scans all comments in the program for eslint-disable/enable directives
/// and Flow suppression patterns, returning suppression ranges.
pub fn find_program_suppressions(
    comments: &[Comment],
    source_text: &str,
    rule_names: Option<&[String]>,
    flow_suppressions: bool,
) -> Vec<SuppressionRange> {
    let mut suppression_ranges = Vec::new();
    let mut disable_start: Option<u32> = None;
    let mut enable_end: Option<u32> = None;
    let mut source: Option<SuppressionSource> = None;

    // Convert rule_names to a usable form
    let rules: Vec<String> = match rule_names {
        Some(rules) if !rules.is_empty() => rules.to_vec(),
        _ => Vec::new(),
    };
    let has_rules = !rules.is_empty();

    for comment in comments {
        let comment_start = comment.span.start;
        let comment_end = comment.span.end;

        // Get the comment text content (without the comment markers)
        let content_span = comment.content_span();
        let comment_text = &source_text[content_span.start as usize..content_span.end as usize];

        // Check for eslint-disable-next-line (only if not already in a block)
        if disable_start.is_none() && has_rules && matches_disable_next_line(comment_text, &rules) {
            disable_start = Some(comment_start);
            enable_end = Some(comment_end);
            source = Some(SuppressionSource::Eslint);
        }

        // Check for Flow suppression (only if not already in a block)
        if flow_suppressions && disable_start.is_none() && matches_flow_suppression(comment_text) {
            disable_start = Some(comment_start);
            enable_end = Some(comment_end);
            source = Some(SuppressionSource::Flow);
        }

        // Check for eslint-disable (block start)
        if has_rules && matches_disable(comment_text, &rules) {
            disable_start = Some(comment_start);
            source = Some(SuppressionSource::Eslint);
        }

        // Check for eslint-enable (block end)
        if has_rules
            && matches_enable(comment_text, &rules)
            && source == Some(SuppressionSource::Eslint)
        {
            enable_end = Some(comment_end);
        }

        // If we have a complete suppression, push it
        if let (Some(start), Some(src)) = (disable_start, source) {
            suppression_ranges.push(SuppressionRange { start, end: enable_end, source: src });
            disable_start = None;
            enable_end = None;
            source = None;
        }
    }

    suppression_ranges
}

/// Filter suppression ranges to those that affect a given function span.
///
/// Port of `filterSuppressionsThatAffectFunction` from Suppression.ts.
///
/// A suppression affects a function if:
/// 1. The suppression is within the function's body; or
/// 2. The suppression wraps the function
pub fn filter_suppressions_that_affect_function(
    suppressions: &[SuppressionRange],
    fn_span: Span,
) -> Vec<&SuppressionRange> {
    let fn_start = fn_span.start;
    let fn_end = fn_span.end;

    suppressions
        .iter()
        .filter(|s| {
            let disable_start = s.start;

            // The suppression is within the function
            let within = disable_start > fn_start
                && match s.end {
                    None => true,
                    Some(enable_end) => enable_end < fn_end,
                };

            // The suppression wraps the function
            let wraps = disable_start < fn_start
                && match s.end {
                    None => true,
                    Some(enable_end) => enable_end > fn_end,
                };

            within || wraps
        })
        .collect()
}

/// Convert suppression ranges that affect a function into a CompilerError.
///
/// Port of `suppressionsToCompilerError` from Suppression.ts.
pub fn suppressions_to_compiler_error(suppressions: &[&SuppressionRange]) -> CompilerError {
    let mut error = CompilerError::new();
    for suppression in suppressions {
        let reason = match suppression.source {
            SuppressionSource::Eslint => {
                "React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled. React Compiler only works when it can safely apply React rules of hooks and other React rules."
            }
            SuppressionSource::Flow => {
                "React Compiler has skipped optimizing this component because a Flow suppression was found."
            }
        };
        error.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Suppression,
            reason: reason.to_string(),
            description: None,
            loc: Some(SourceLocation::Source(Span::new(
                suppression.start,
                suppression.end.unwrap_or(suppression.start),
            ))),
            suggestions: None,
        }));
    }
    error
}
