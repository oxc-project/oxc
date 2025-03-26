use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

const CHAR_LIMIT: usize = 40;

// This regex is used to detect if the comment itself is a configuration comment
// for the no-warning-comments rule. In such a case we do not warn.
static SELF_CONFIG_REGEX: Lazy<Regex> = lazy_regex!(r"(?i)\bno-warning-comments\b");

/// Determines where in a comment a warning term should be matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WarningLocation {
    Start,
    Anywhere,
}

impl Default for WarningLocation {
    fn default() -> Self {
        WarningLocation::Start
    }
}

/// The no-warning-comments rule checks all comments in a source file and reports a diagnostic
/// when a comment contains one of the disallowed warning terms.
#[derive(Debug, Default, Clone)]
pub struct NoWarningComments {
    warning_regexes: Vec<Regex>,
    terms: Vec<String>
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows specified warning terms in comments.
    ///
    /// ### Why is this bad?
    ///
    /// Warning comments like "TODO", "FIXME", or "XXX" can indicate unfinished code or technical debt.
    ///
    /// ### Examples
    ///
    /// With the default configuration:
    /// ```js
    /// // TODO: refactor this code
    /// // fixme: handle edge cases
    /// // xxx: temporary workaround
    /// ```
    ///
    /// These comments would be reported.
    ///
    /// ### Options
    ///
    /// An object with the following optional properties:
    /// - `terms`: an array of warning terms (default: `["todo", "fixme", "xxx"]`)
    /// - `location`: either `"start"` (the term must appear at the start of a comment, ignoring whitespace and decoration) or `"anywhere"` (default is `"start"`)
    /// - `decoration`: an array of single-character strings to allow as decoration before the term
    NoWarningComments,
    eslint,
    pedantic
);

impl Rule for NoWarningComments {
    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.semantic().comments();
        for comment in comments {
            // Obtain the raw comment content (i.e. the inner text, excluding the comment markers)
            let raw = ctx.source_range(comment.content_span());
            // If the comment is a self-configuration comment for this rule, skip it.
            if SELF_CONFIG_REGEX.is_match(raw) {
                continue;
            }
            // For each configured warning regex, check if it matches the comment.
            for (i, regex) in self.warning_regexes.iter().enumerate() {
                if regex.is_match(raw) {
                    let matched_term = &self.terms[i];
                    let truncated = truncate_comment(raw);
                    let comment_span = get_full_comment(ctx, comment.span);
                    ctx.diagnostic(no_warning_comments_diagnostic(matched_term, &truncated, comment_span));
                }
            }
        }
    }

    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        // Parse "location" option (default is "start")
        let location_str = config
            .and_then(|cfg| cfg.get("location"))
            .and_then(Value::as_str)
            .unwrap_or("start");
        let location = match location_str {
            "anywhere" => WarningLocation::Anywhere,
            _ => WarningLocation::Start,
        };

        // Parse "terms" option; if not provided, use defaults.
        let terms: Vec<String> = if let Some(cfg) = config {
            if let Some(arr) = cfg.get("terms").and_then(Value::as_array) {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                vec!["todo".to_string(), "fixme".to_string(), "xxx".to_string()]
            }
        } else {
            vec!["todo".to_string(), "fixme".to_string(), "xxx".to_string()]
        };

        // Parse "decoration" option; join the array into a single string.
        let decoration = if let Some(cfg) = config {
            if let Some(arr) = cfg.get("decoration").and_then(Value::as_array) {
                let mut dec_chars = String::new();
                for v in arr {
                    if let Some(s) = v.as_str() {
                        dec_chars.push_str(s);
                    }
                }
                dec_chars
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Build a regex for each warning term.
        let warning_regexes = terms
            .iter()
            .map(|term| convert_to_regex(term, location, &decoration))
            .collect();

        Self {
            warning_regexes,
            terms
        }
    }
}

/// Constructs a diagnostic for a warning comment.
fn no_warning_comments_diagnostic(matched_term: &str, comment: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unresolved '{}' comment.", matched_term)).with_label(span)
}

/// Converts a warning term into a Regex that will match the term in a comment.
///
/// If `location` is `Start`, the regex will match the term at the beginning of the comment,
/// allowing for leading whitespace and any characters specified by `decoration`.
/// Otherwise, if `location` is `Anywhere`, word boundaries are added if the term starts or ends with a word character.
fn convert_to_regex(term: &str, location: WarningLocation, decoration: &str) -> Regex {
    let escaped_term = term;
    let escaped_decoration = decoration;
    let prefix = match location {
        WarningLocation::Start => format!("^[\\s{}]*", escaped_decoration),
        WarningLocation::Anywhere => {
            if term.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
                String::from("\\b")
            } else {
                String::new()
            }
        }
    };
    let suffix = if term
        .chars()
        .last()
        .map(|c| c.is_alphanumeric() || c == '_')
        .unwrap_or(false)
    {
        "\\b"
    } else {
        ""
    };
    let pattern = format!("(?i){}{}{}", prefix, escaped_term, suffix);
    Regex::new(&pattern).unwrap()
}

/// Truncates the comment text to a maximum length of CHAR_LIMIT characters,
/// preserving whole words. If truncation occurs, "..." is appended.
fn truncate_comment(comment: &str) -> String {
    let mut comment_to_display = String::new();
    let mut truncated = false;
    for word in comment.split_whitespace() {
        if !comment_to_display.is_empty() {
            let candidate = format!("{} {}", comment_to_display, word);
            if candidate.len() <= CHAR_LIMIT {
                comment_to_display = candidate;
            } else {
                truncated = true;
                break;
            }
        } else {
            if word.len() <= CHAR_LIMIT {
                comment_to_display.push_str(word);
            } else {
                comment_to_display.push_str(&word[..CHAR_LIMIT]);
                truncated = true;
                break;
            }
        }
    }
    if truncated {
        comment_to_display.push_str("...");
    }
    comment_to_display
}

/// Returns the full span of a comment, extending past the end if a newline is found.
/// This mimics the behavior in ban_tslint_comment.rs.
fn get_full_comment(ctx: &LintContext, span: Span) -> Span {
    let mut span = span;
    let source_text = ctx.source_text();
    let source_size = u32::try_from(source_text.len()).unwrap();
    let following_text = ctx.source_range(Span::new(span.end, source_size));
    if following_text.chars().next() == Some('\n') {
        span.end += 1;
    }
    span
}