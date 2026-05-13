use std::time::{SystemTime, UNIX_EPOCH};

use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn expired_todo_diagnostic(date: &str, message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "There is a TODO that is past due date: {date}.{}",
        format_trailing(message)
    ))
    .with_label(span)
}

fn avoid_multiple_dates_diagnostic(dates: &str, message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Avoid using multiple expiration dates in TODO: {dates}.{}",
        format_trailing(message)
    ))
    .with_label(span)
}

fn unexpected_comment_diagnostic(term: &str, comment: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Unexpected '{term}' comment without any conditions: '{}'.",
        comment.trim()
    ))
    .with_label(span)
}

fn format_trailing(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() { String::new() } else { format!(" {trimmed}") }
}

/// User-facing configuration shape — drives the JSON schema and deserialization.
#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ExpiringTodoCommentsConfig {
    /// Terms to match as warning comments (case-insensitive).
    pub terms: Vec<String>,
    /// Regex patterns (matched against the comment line) to ignore.
    pub ignore: Vec<String>,
    /// If `true`, all date expiration checks are skipped.
    pub ignore_dates: bool,
    /// If `true`, date expiration checks are skipped when running on a pull
    /// request (detected via CI env vars).
    pub ignore_dates_on_pull_requests: bool,
    /// If `false`, plain TODO/FIXME/XXX comments without expiration conditions
    /// are reported as well.
    pub allow_warning_comments: bool,
    /// Override the reference date (format: `YYYY-MM-DD`). Useful for tests.
    pub date: Option<String>,
}

impl Default for ExpiringTodoCommentsConfig {
    fn default() -> Self {
        Self {
            terms: vec!["todo".to_string(), "fixme".to_string(), "xxx".to_string()],
            ignore: Vec::new(),
            ignore_dates: false,
            ignore_dates_on_pull_requests: true,
            allow_warning_comments: true,
            date: None,
        }
    }
}

/// Runtime state — the user config with all per-config artifacts precomputed.
/// Built once in `from_configuration`, reused on every file.
#[derive(Debug, Clone)]
struct ExpiringTodoCommentsState {
    /// Lowercased terms (ASCII lowercase). Non-ASCII terms keep their case
    /// since `eq_ignore_ascii_case` only folds ASCII letters.
    terms_lower: Vec<String>,
    ignore_patterns: Vec<Regex>,
    ignore_dates: bool,
    ignore_dates_on_pull_requests: bool,
    allow_warning_comments: bool,
    date: Option<String>,
}

impl Default for ExpiringTodoCommentsState {
    fn default() -> Self {
        // `unwrap` is safe: the default config has no `ignore` patterns to compile.
        Self::from_config(ExpiringTodoCommentsConfig::default()).expect("default config is valid")
    }
}

impl ExpiringTodoCommentsState {
    fn from_config(cfg: ExpiringTodoCommentsConfig) -> Result<Self, lazy_regex::regex::Error> {
        let terms_lower =
            cfg.terms.iter().map(|t| t.cow_to_ascii_lowercase().into_owned()).collect();
        let ignore_patterns =
            cfg.ignore.iter().map(|p| Regex::new(p)).collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            terms_lower,
            ignore_patterns,
            ignore_dates: cfg.ignore_dates,
            ignore_dates_on_pull_requests: cfg.ignore_dates_on_pull_requests,
            allow_warning_comments: cfg.allow_warning_comments,
            date: cfg.date,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExpiringTodoComments(Box<ExpiringTodoCommentsState>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces expiration conditions on TODO/FIXME/XXX comments so they don't
    /// silently rot in the codebase.
    ///
    /// ### Why is this bad?
    ///
    /// TODO comments tend to accumulate over time and are easy to ignore. By
    /// attaching an expiration condition — typically a date — the linter forces
    /// the team to triage the comment once it goes stale instead of letting it
    /// sit there forever.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // TODO [2000-01-01]: too old
    /// // TODO [2200-01-01, 2300-01-01]: multiple expiration dates
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // TODO [2200-12-12]: still in the future
    /// // TODO ISSUE-123: free-form todo (when `allowWarningComments` is true)
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "terms": ["todo", "fixme", "xxx"],
    ///   "ignore": [],
    ///   "ignoreDates": false,
    ///   "ignoreDatesOnPullRequests": true,
    ///   "allowWarningComments": true,
    ///   "date": null
    /// }
    /// ```
    ///
    /// Only date-based expiration conditions (e.g. `[2020-01-01]`) are checked
    /// in this version. Package version, dependency, and engine conditions are
    /// not yet implemented and are treated as free-form text.
    ExpiringTodoComments,
    unicorn,
    restriction,
    config = ExpiringTodoCommentsConfig,
    version = "next",
);

impl Rule for ExpiringTodoComments {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        use serde::de::Error;
        let cfg = serde_json::from_value::<DefaultRuleConfig<ExpiringTodoCommentsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)?;
        let state = ExpiringTodoCommentsState::from_config(cfg)
            .map_err(|e| serde_json::Error::custom(format!("invalid `ignore` regex: {e}")))?;
        Ok(Self(Box::new(state)))
    }

    fn run_once(&self, ctx: &LintContext) {
        let state = &*self.0;

        let today_owned;
        let today: &str = if let Some(d) = state.date.as_deref() {
            d
        } else {
            today_owned = today_string();
            &today_owned
        };

        let in_pr = is_running_in_pull_request();
        let skip_dates = state.ignore_dates || (state.ignore_dates_on_pull_requests && in_pr);

        for comment in ctx.semantic().comments() {
            let content_span = comment.content_span();
            let content = ctx.source_range(content_span);

            let mut line_offset: u32 = 0;
            for (idx, raw_line) in content.split('\n').enumerate() {
                #[expect(clippy::cast_possible_truncation)]
                let raw_line_len = raw_line.len() as u32;
                // Consume this line and its newline regardless of whether we
                // emit a diagnostic.
                let line_offset_for_this_line = line_offset;
                line_offset += raw_line_len + 1;

                // For continuation lines of a block comment, strip leading
                // whitespace + `*` decoration so we can match TODO terms that
                // appear at the start of the visible content.
                let is_block_continuation = comment.is_block() && idx > 0;
                let (line, line_offset_within_raw) =
                    strip_comment_decoration(raw_line, is_block_continuation);
                if line.is_empty() {
                    continue;
                }

                if state.ignore_patterns.iter().any(|r| r.is_match(line)) {
                    continue;
                }

                let Some(matched_term) = find_term_in_line(line, &state.terms_lower) else {
                    continue;
                };

                let report_span = if comment.is_block() {
                    let line_start =
                        content_span.start + line_offset_for_this_line + line_offset_within_raw;
                    #[expect(clippy::cast_possible_truncation)]
                    let line_end = line_start + line.len() as u32;
                    Span::new(line_start, line_end)
                } else {
                    comment.span
                };

                process_line(
                    line,
                    matched_term,
                    report_span,
                    today,
                    skip_dates,
                    state.allow_warning_comments,
                    ctx,
                );
            }
        }
    }
}

fn process_line(
    line: &str,
    matched_term: &str,
    span: Span,
    today: &str,
    skip_dates: bool,
    allow_warning: bool,
    ctx: &LintContext,
) {
    let Some(bracket) = extract_bracket_arguments(line) else {
        if !allow_warning {
            ctx.diagnostic(unexpected_comment_diagnostic(matched_term, line, span));
        }
        return;
    };

    let dates = classify_arguments(bracket);

    if dates.len() > 1 {
        let joined = dates.join(", ");
        let msg = todo_message_after_brackets(line);
        ctx.diagnostic(avoid_multiple_dates_diagnostic(&joined, msg, span));
        return;
    }

    if let Some(date) = dates.first() {
        // Strict `<` to match upstream `reachedDate(past, now) = past < now`:
        // a TODO dated *today* is still valid until the day rolls over.
        if !skip_dates && *date < today {
            let msg = todo_message_after_brackets(line);
            ctx.diagnostic(expired_todo_diagnostic(date, msg, span));
        }
        // A date condition was present: do not fall through to bare-warning.
        return;
    }

    // No date conditions found. Treat the same as a bare TODO: if the user
    // opted out of allowing warning comments, report it. Non-date arguments
    // (e.g. `[invalid]`) are not yet recognized as conditions in this
    // increment.
    if !allow_warning {
        ctx.diagnostic(unexpected_comment_diagnostic(matched_term, line, span));
    }
}

/// Strip leading whitespace and `*` decoration from a comment line, returning
/// the trimmed slice and the byte offset of its start within `raw_line`. Also
/// strips trailing whitespace.
///
/// `is_block_continuation` controls whether `*` characters are treated as
/// decoration — only continuation lines of a block comment have a leading
/// `* ` decoration; the first line of a block comment does not.
fn strip_comment_decoration(raw_line: &str, is_block_continuation: bool) -> (&str, u32) {
    let bytes = raw_line.as_bytes();
    let mut start = 0;
    if is_block_continuation {
        while start < bytes.len() && (bytes[start] == b' ' || bytes[start] == b'\t') {
            start += 1;
        }
        while start < bytes.len() && bytes[start] == b'*' {
            start += 1;
        }
    }
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    let mut end = bytes.len();
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    #[expect(clippy::cast_possible_truncation)]
    let start_u32 = start as u32;
    (&raw_line[start..end], start_u32)
}

/// Case-insensitive (ASCII) search for any of the lowercase `terms` in
/// `line`. Returns the matched substring borrowed from `line`, preserving
/// the original casing for the diagnostic.
fn find_term_in_line<'a>(line: &'a str, terms_lower: &[String]) -> Option<&'a str> {
    let line_bytes = line.as_bytes();
    for term in terms_lower {
        let term_bytes = term.as_bytes();
        if term_bytes.is_empty() || term_bytes.len() > line_bytes.len() {
            continue;
        }
        for (i, window) in line_bytes.windows(term_bytes.len()).enumerate() {
            if window.eq_ignore_ascii_case(term_bytes) {
                // Safe: window indices align with UTF-8 boundaries because
                // `term` is ASCII (we lowercased with `to_ascii_lowercase`).
                return Some(&line[i..i + term_bytes.len()]);
            }
        }
    }
    None
}

/// Extract the contents of the first balanced `[...]` group in `line`, or
/// `None` if no opening bracket exists.
fn extract_bracket_arguments(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let start = bytes.iter().position(|&b| b == b'[')?;
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&line[start + 1..i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract ISO 8601 date arguments from the comma-separated bracket contents.
/// Non-date arguments are ignored (this increment only supports date
/// conditions). Returns slices borrowed from `bracket`.
fn classify_arguments(bracket: &str) -> Vec<&str> {
    bracket.split(',').map(str::trim).filter(|s| is_iso_date(s)).collect()
}

/// Returns true if `s` is exactly `YYYY-MM-DD` with ASCII digits.
fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[0..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit)
}

/// Return the trailing message after the bracket group, stripped of a
/// leading `:` and surrounding whitespace.
fn todo_message_after_brackets(line: &str) -> &str {
    let Some(end) = line.find(']') else {
        return line.trim();
    };
    let tail = line[end + 1..].trim_start();
    tail.strip_prefix(':').unwrap_or(tail).trim()
}

/// Best-effort detection that the lint run is happening for a pull-request.
/// The upstream rule uses the `ci-info` package which covers ~30 CI providers;
/// we only handle the ones most likely to surface this rule. Expanding this
/// list is welcome.
fn is_running_in_pull_request() -> bool {
    if std::env::var("GITHUB_EVENT_NAME").as_deref() == Ok("pull_request") {
        return true;
    }
    std::env::var("CI_PULL_REQUEST").is_ok()
        || std::env::var("CIRCLE_PULL_REQUEST").is_ok()
        || std::env::var("BITRISE_PULL_REQUEST").is_ok()
        || std::env::var("BUILDKITE_PULL_REQUEST").as_deref().is_ok_and(|v| v != "false")
}

/// Convert the current system time to a UTC `YYYY-MM-DD` string without
/// pulling in a date/time crate.
fn today_string() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    #[expect(clippy::cast_possible_wrap)]
    let days = (secs / 86_400) as i64;
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Convert days since 1970-01-01 (UTC) to a (year, month, day) triple using
/// Howard Hinnant's algorithm: <http://howardhinnant.github.io/date_algorithms.html>.
///
/// Casts here are all safe within the documented `i64` input range used by
/// `today_string` — the intermediate values stay within their bounded ranges
/// (shown in inline comments).
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
fn days_to_ymd(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}

#[cfg(test)]
mod date_tests {
    use super::days_to_ymd;

    #[test]
    fn epoch() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn known_dates() {
        assert_eq!(days_to_ymd(31), (1970, 2, 1));
        assert_eq!(days_to_ymd(365), (1971, 1, 1));
        // 2000 was a leap year (divisible by 400).
        assert_eq!(days_to_ymd(10_957), (2000, 1, 1));
        assert_eq!(days_to_ymd(10_957 + 59), (2000, 2, 29));
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    // Use a fixed reference date so tests are deterministic across runs.
    let default_cfg = json!([{ "date": "2020-01-01" }]);
    let warn_bare_cfg = json!([{ "date": "2020-01-01", "allowWarningComments": false }]);
    let ignore_dates_cfg = json!([{ "date": "2020-01-01", "ignoreDates": true }]);

    let pass = vec![
        ("// TODO [2200-12-12]: future date", Some(default_cfg.clone())),
        ("// FIXME [2200-12-12]: future date", Some(default_cfg.clone())),
        ("// XXX [2200-12-12]: future date", Some(default_cfg.clone())),
        ("// TODO (lubien) [2200-12-12]: future date with author", Some(default_cfg.clone())),
        ("// TODO [invalid]", Some(default_cfg.clone())),
        ("// TODO ISSUE-123 fix later", Some(default_cfg.clone())),
        ("// TODO", Some(default_cfg.clone())),
        ("// TODO []", Some(default_cfg.clone())),
        ("// TODO [no meaning at all]", Some(default_cfg.clone())),
        ("// TODO [2001-01-01]: ignored because ignoreDates", Some(ignore_dates_cfg.clone())),
        ("// not a fixme or todo", Some(default_cfg.clone())),
        // Today is the reference date — upstream `reachedDate` is strict `<`,
        // so a TODO dated today is still valid until tomorrow.
        ("// TODO [2020-01-01]: today is not yet expired", Some(default_cfg.clone())),
        (
            "// TODO [2200-12-12, 2200-12-13]: ignored multiple dates via ignore",
            Some(json!([{
                "date": "2020-01-01",
                "ignore": ["multiple dates via ignore"],
            }])),
        ),
        // A future date counts as a valid expiration condition even when
        // `allowWarningComments` is false: the comment is not "bare".
        ("// TODO [2999-12-01]: Y3K bug", Some(warn_bare_cfg.clone())),
        // A comment without any TODO term is never flagged.
        ("const x = 1; // just a comment", None),
    ];

    let fail = vec![
        ("// TODO [2000-01-01]: too old", Some(default_cfg.clone())),
        ("// fixme [2000-01-01]: too old", Some(default_cfg.clone())),
        ("// xxx [2000-01-01]: too old", Some(default_cfg.clone())),
        ("// ToDo [2000-01-01]: too old", Some(default_cfg.clone())),
        ("// fIxME [2000-01-01]: too old", Some(default_cfg.clone())),
        // The day before the reference date is expired.
        ("// TODO [2019-12-31]: yesterday is expired", Some(default_cfg.clone())),
        ("// TODO [2200-12-12, 2200-12-12]: multiple dates", Some(default_cfg.clone())),
        ("// TODO [2200-12-12, 2300-01-01]: multiple dates", Some(default_cfg.clone())),
        ("// TODO", Some(warn_bare_cfg.clone())),
        // Non-date arguments don't count as conditions, so this is treated as
        // a bare warning comment.
        ("// TODO [no meaning at all]", Some(warn_bare_cfg)),
        (
            "/* TODO [2000-01-01]: line one\n * TODO [2000-01-01]: line two\n * TODO [2000-01-01] line three */",
            Some(default_cfg.clone()),
        ),
    ];

    Tester::new(ExpiringTodoComments::NAME, ExpiringTodoComments::PLUGIN, pass, fail)
        .test_and_snapshot();
}
