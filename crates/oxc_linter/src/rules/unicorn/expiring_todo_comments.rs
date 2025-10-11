use cow_utils::CowUtils;
use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use time::{Date, OffsetDateTime, PrimitiveDateTime, format_description::well_known::Iso8601};

use crate::{context::LintContext, rule::Rule};

fn missing_expiring_todo_comment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "TODO-style comments must include an expiration date or tracking reference.",
    )
    .with_help("Add an ISO-8601 date in the future, an issue identifier, or a URL.")
    .with_label(span)
}

fn expired_expiring_todo_comment_diagnostic(span: Span, raw_date: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("This tracked comment expired on {raw_date}."))
        .with_help("Update the expiration date or remove the comment.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ExpiringTodoComments(Box<ExpiringTodoCommentsConfig>);

#[derive(Debug, Clone)]
pub struct ExpiringTodoCommentsConfig {
    terms: Vec<String>,
    ignore: Vec<String>,
    allow_issue_reference: bool,
    allow_ticket_reference: bool,
    allow_url_reference: bool,
    current_date: Option<Date>,
}

impl Default for ExpiringTodoCommentsConfig {
    fn default() -> Self {
        Self {
            terms: vec!["todo".to_string(), "fixme".to_string()],
            ignore: Vec::new(),
            allow_issue_reference: true,
            allow_ticket_reference: true,
            allow_url_reference: true,
            current_date: None,
        }
    }
}

impl std::ops::Deref for ExpiringTodoComments {
    type Target = ExpiringTodoCommentsConfig;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires TODO-style comments to include an expiration date or a tracking reference
    /// (such as an issue ID or URL) so they can be revisited in a timely manner.
    ///
    /// ### Why is this bad?
    ///
    /// Leaving TODO or FIXME comments without a tracking reference makes it hard to
    /// ensure they are addressed before becoming stale. Storing a due date or ticket
    /// number keeps the work discoverable and encourages timely cleanup.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // TODO: remove this temporary workaround
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // TODO [2030-01-01]: remove this temporary workaround
    /// // TODO: remove once #1234 is resolved
    /// ```
    ExpiringTodoComments,
    unicorn,
    style,
);

impl Rule for ExpiringTodoComments {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = ExpiringTodoCommentsConfig::default();
        if let Some(options) = value.get(0) {
            if let Some(terms) = options.get("terms").and_then(serde_json::Value::as_array) {
                let collected = terms
                    .iter()
                    .filter_map(|term| term.as_str())
                    .map(|term| term.cow_to_ascii_lowercase().into_owned())
                    .collect::<Vec<_>>();
                if !collected.is_empty() {
                    config.terms = collected;
                }
            }

            if let Some(ignore) = options.get("ignore").and_then(serde_json::Value::as_array) {
                config.ignore = ignore
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(|value| value.cow_to_ascii_lowercase().into_owned())
                    .collect();
            }

            if let Some(value) =
                options.get("allowIssueReference").and_then(serde_json::Value::as_bool)
            {
                config.allow_issue_reference = value;
            }

            if let Some(value) =
                options.get("allowTicketReference").and_then(serde_json::Value::as_bool)
            {
                config.allow_ticket_reference = value;
            }

            if let Some(value) = options.get("allowUrl").and_then(serde_json::Value::as_bool) {
                config.allow_url_reference = value;
            }

            if let Some(current_date) =
                options.get("currentDate").and_then(serde_json::Value::as_str)
            {
                config.current_date = parse_iso_date(current_date);
            }
        }

        Self(Box::new(config))
    }

    fn run_once(&self, ctx: &LintContext) {
        let today = self.current_date.unwrap_or_else(current_utc_date);

        for comment in ctx.comments() {
            let raw = ctx.source_range(comment.content_span());
            let normalized = normalize_comment_text(raw);
            if normalized.is_empty() {
                continue;
            }

            let lowercase = normalized.as_str().cow_to_ascii_lowercase();
            if self.ignore.iter().any(|pattern| lowercase.contains(pattern.as_str())) {
                continue;
            }

            if !self.contains_tracked_term(lowercase.as_ref()) {
                continue;
            }

            match self.assess_comment(&normalized, today) {
                CommentStatus::Valid => {}
                CommentStatus::Missing => {
                    ctx.diagnostic(missing_expiring_todo_comment_diagnostic(comment.span));
                }
                CommentStatus::Expired { raw_date } => {
                    ctx.diagnostic(expired_expiring_todo_comment_diagnostic(
                        comment.span,
                        &raw_date,
                    ));
                }
            }
        }
    }
}

impl ExpiringTodoCommentsConfig {
    fn contains_tracked_term(&self, comment: &str) -> bool {
        self.terms.iter().any(|term| contains_term(comment, term.as_str()))
    }

    fn assess_comment(&self, comment: &str, today: Date) -> CommentStatus {
        if let Some(date_match) = find_date(comment) {
            if date_match.date < today {
                return CommentStatus::Expired { raw_date: date_match.raw };
            }
            return CommentStatus::Valid;
        }

        if self.allow_issue_reference && has_issue_reference(comment) {
            return CommentStatus::Valid;
        }

        if self.allow_ticket_reference && has_ticket_reference(comment) {
            return CommentStatus::Valid;
        }

        if self.allow_url_reference && has_url(comment) {
            return CommentStatus::Valid;
        }

        CommentStatus::Missing
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommentStatus {
    Valid,
    Missing,
    Expired { raw_date: String },
}

#[derive(Debug, Clone)]
struct DateMatch {
    raw: String,
    date: Date,
}

fn contains_term(comment: &str, term: &str) -> bool {
    let comment_bytes = comment.as_bytes();
    let term_bytes = term.as_bytes();

    if term_bytes.is_empty() || term_bytes.len() > comment_bytes.len() {
        return false;
    }

    let mut index = 0usize;
    let term_len = term_bytes.len();
    let comment_len = comment_bytes.len();

    while index + term_len <= comment_len {
        if &comment_bytes[index..index + term_len] == term_bytes {
            let before_ok = index == 0 || !is_identifier_byte(comment_bytes[index - 1]);
            let after_ok = index + term_len == comment_len
                || !is_identifier_byte(comment_bytes[index + term_len]);
            if before_ok && after_ok {
                return true;
            }
            index += term_len;
        } else {
            index += 1;
        }
    }

    false
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn find_date(comment: &str) -> Option<DateMatch> {
    let bytes = comment.as_bytes();
    for captures in ISO_DATE_PATTERN.captures_iter(comment) {
        let m = captures.get(1)?;
        let start = m.start();
        let end = m.end();
        let before_ok = start == 0 || !is_identifier_byte(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_identifier_byte(bytes[end]);
        if !before_ok || !after_ok {
            continue;
        }

        let raw = m.as_str();
        if let Some(date) = parse_iso_date(raw) {
            return Some(DateMatch { raw: raw.to_string(), date });
        }
    }
    None
}

fn parse_iso_date(raw: &str) -> Option<Date> {
    if let Ok(date) = Date::parse(raw, &Iso8601::DEFAULT) {
        return Some(date);
    }
    if let Ok(datetime) = OffsetDateTime::parse(raw, &Iso8601::DEFAULT) {
        return Some(datetime.date());
    }
    if let Ok(datetime) = PrimitiveDateTime::parse(raw, &Iso8601::DEFAULT) {
        return Some(datetime.date());
    }
    None
}

fn has_issue_reference(comment: &str) -> bool {
    if comment.as_bytes().windows(2).any(|window| window[0] == b'#' && window[1].is_ascii_digit()) {
        return true;
    }

    let lowercase = comment.cow_to_ascii_lowercase();
    let keywords = ["issue", "issues", "pr", "gh", "pull request", "pull-request"];
    keywords.iter().any(|keyword| has_keyword_number(lowercase.as_ref(), keyword))
}

fn has_ticket_reference(comment: &str) -> bool {
    let bytes = comment.as_bytes();
    let len = bytes.len();
    let mut index = 0usize;

    while index < len {
        if bytes[index].is_ascii_uppercase() {
            let start = index;
            index += 1;
            while index < len
                && (bytes[index].is_ascii_uppercase() || bytes[index].is_ascii_digit())
            {
                index += 1;
            }

            let key_len = index - start;
            if key_len >= 2 && index < len && bytes[index] == b'-' {
                let mut digits = index + 1;
                while digits < len && bytes[digits].is_ascii_digit() {
                    digits += 1;
                }
                if digits > index + 1 {
                    let before_ok = start == 0 || !is_identifier_byte(bytes[start - 1]);
                    let after_ok = digits == len || !is_identifier_byte(bytes[digits]);
                    if before_ok && after_ok {
                        return true;
                    }
                }
                index = digits;
                continue;
            }
        }
        index += 1;
    }

    false
}

fn has_url(comment: &str) -> bool {
    let bytes = comment.as_bytes();
    let len = bytes.len();
    let mut index = 0usize;

    while index < len {
        if bytes[index].is_ascii_alphabetic() {
            let mut scheme_end = index + 1;
            while scheme_end < len
                && (bytes[scheme_end].is_ascii_alphanumeric()
                    || matches!(bytes[scheme_end], b'+' | b'.' | b'-'))
            {
                scheme_end += 1;
            }

            if scheme_end + 2 < len
                && bytes[scheme_end] == b':'
                && bytes[scheme_end + 1] == b'/'
                && bytes[scheme_end + 2] == b'/'
            {
                return true;
            }

            index = scheme_end;
        } else {
            index += 1;
        }
    }

    false
}

fn current_utc_date() -> Date {
    OffsetDateTime::now_utc().date()
}

fn normalize_comment_text(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(stripped) = trimmed.strip_prefix("//") {
        return stripped.trim_start().to_string();
    }

    if let Some(stripped) = trimmed.strip_prefix("/*") {
        let inner = stripped.strip_suffix("*/").unwrap_or(stripped);
        let mut normalized = String::new();
        for line in inner.lines() {
            let line = line.trim_start();
            let line = line.strip_prefix('*').map_or(line, |value| value.trim_start());
            if !line.is_empty() {
                if !normalized.is_empty() {
                    normalized.push(' ');
                }
                normalized.push_str(line);
            }
        }
        let result = normalized.trim().to_string();
        if result.is_empty() {
            return String::new();
        }
        return result;
    }

    trimmed.to_string()
}

static ISO_DATE_PATTERN: Lazy<Regex> = lazy_regex!(
    r"(?xi)
    (
        \d{4}
        -\d{2}
        -\d{2}
        (?:[T\s]
            \d{2}:\d{2}
            (?::\d{2}(?:\.\d{1,9})?)?
            (?:Z|[+-]\d{2}:?\d{2})?
        )?
    )
    "
);

fn has_keyword_number(comment: &str, keyword: &str) -> bool {
    let comment_bytes = comment.as_bytes();
    let keyword_bytes = keyword.as_bytes();
    if keyword_bytes.is_empty() {
        return false;
    }

    let mut offset = 0usize;
    while let Some(position) = comment[offset..].find(keyword) {
        let start = offset + position;
        let end = start + keyword_bytes.len();

        let before_ok = start == 0 || !is_identifier_byte(comment_bytes[start - 1]);
        let after_ok = end == comment_bytes.len() || !is_identifier_byte(comment_bytes[end]);
        if before_ok && after_ok {
            let mut index = end;
            while index < comment_bytes.len() && comment_bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            if index < comment_bytes.len() && matches!(comment_bytes[index], b':' | b'-') {
                index += 1;
            }
            while index < comment_bytes.len() && comment_bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            if index < comment_bytes.len() && comment_bytes[index] == b'#' {
                index += 1;
            }

            let mut has_digit = false;
            while index < comment_bytes.len() && comment_bytes[index].is_ascii_digit() {
                has_digit = true;
                index += 1;
            }

            if has_digit {
                return true;
            }
        }

        offset = end;
    }

    false
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::tester::Tester;

    #[test]
    fn defaults() {
        let config = json!([{ "currentDate": "2024-05-01" }]);

        let pass = vec![
            ("// TODO [2024-12-01]: clean up", Some(config.clone())),
            ("// FIXME: remove after #123", Some(config.clone())),
            ("// TODO rewrite after https://example.com/ticket", Some(config.clone())),
            ("/*\n * TODO [2024-11-11]: migrate\n */", Some(config.clone())),
            ("// TODO: tracked by ABC-42", Some(config.clone())),
            ("// TODO issue 2042", Some(config.clone())),
            ("// TODO: see GH-777", Some(config.clone())),
            ("// TODO follow ftp://example.com/info", Some(config.clone())),
        ];

        let fail = vec![
            ("// TODO: missing tracking", Some(config.clone())),
            ("// FIXME [2024-04-01]: expired", Some(config.clone())),
            ("// TODO (#abc) not a valid ticket", Some(config.clone())),
            ("// TODO: issue without digits", Some(config.clone())),
            ("// TODO tracked by ab-42", Some(config)),
        ];

        Tester::new(ExpiringTodoComments::NAME, ExpiringTodoComments::PLUGIN, pass, fail).test();
    }

    #[test]
    fn configuration() {
        let pass = vec![
            (
                "// note: tracked later",
                Some(
                    json!([{ "terms": ["note"], "ignore": ["tracked"], "currentDate": "2024-05-01" }]),
                ),
            ),
            (
                "// note handled by XY-99",
                Some(json!([{ "terms": ["note"], "currentDate": "2024-05-01" }])),
            ),
        ];

        let fail = vec![
            (
                "// NOTE still outstanding",
                Some(json!([{ "terms": ["note"], "currentDate": "2024-05-01" }])),
            ),
            (
                "// note tracked by #42",
                Some(
                    json!([{ "terms": ["note"], "allowIssueReference": false, "currentDate": "2024-05-01" }]),
                ),
            ),
        ];

        Tester::new(ExpiringTodoComments::NAME, ExpiringTodoComments::PLUGIN, pass, fail).test();
    }

    #[test]
    fn helper_detection() {
        assert!(has_issue_reference("TODO issue 99"));
        assert!(has_issue_reference("TODO: tracked by GH-123"));
        assert!(has_issue_reference("TODO pull request #42"));
        assert!(!has_issue_reference("TODO talking about tissue"));

        assert!(has_ticket_reference("TODO ABCD-999"));
        assert!(has_ticket_reference("TODO XY2-15 should pass"));
        assert!(!has_ticket_reference("TODO ab-42"));
        assert!(!has_ticket_reference("TODO A-1"));

        assert!(has_url("TODO https://example.com"));
        assert!(has_url("TODO ftp://example.com"));
        assert!(!has_url("TODO example.com"));

        assert_eq!(normalize_comment_text("//  TODO spaced"), "TODO spaced");
        assert_eq!(
            normalize_comment_text("/*\n * TODO tighten\n * tracking\n */"),
            "TODO tighten tracking"
        );
    }
}
