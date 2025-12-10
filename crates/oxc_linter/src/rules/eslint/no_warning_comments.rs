use cow_utils::CowUtils;
use lazy_regex::{Regex, regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

fn no_warning_comments_diagnostic(term: &str, comment: &str, span: Span) -> OxcDiagnostic {
    const CHAR_LIMIT: usize = 40;

    let mut comment_to_display = String::new();
    let mut truncated = false;

    for word in comment.split_whitespace() {
        let tmp = if comment_to_display.is_empty() {
            word.to_string()
        } else {
            format!("{comment_to_display} {word}")
        };

        if tmp.len() <= CHAR_LIMIT {
            comment_to_display = tmp;
        } else {
            truncated = true;
            break;
        }
    }

    let display = if truncated { format!("{comment_to_display}...") } else { comment_to_display };

    OxcDiagnostic::warn(format!("Unexpected '{term}' comment: {display}"))
        .with_help("Remove or rephrase this comment")
        .with_label(span)
}

#[derive(Debug, Clone)]
struct NoWarningCommentsConfig {
    terms: Vec<String>,
    patterns: Vec<Regex>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Location {
    Start,
    Anywhere,
}

#[derive(Debug, Clone)]
pub struct NoWarningComments(Box<NoWarningCommentsConfig>);

impl Default for NoWarningComments {
    fn default() -> Self {
        let terms = vec!["todo".to_string(), "fixme".to_string(), "xxx".to_string()];
        let location = Location::Start;
        let decoration = FxHashSet::default();
        Self::new(&terms, &location, &decoration)
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows warning comments such as TODO, FIXME, XXX in code.
    ///
    /// ### Why is this bad?
    ///
    /// Developers often add comments like TODO or FIXME to mark incomplete work or areas
    /// that need attention. While useful during development, these comments can indicate
    /// unfinished code that shouldn't be shipped to production. This rule helps catch
    /// such comments before they make it into production code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // TODO: implement this feature
    /// function doSomething() {}
    ///
    /// // FIXME: this is broken
    /// const x = 1;
    ///
    /// /* XXX: hack */
    /// let y = 2;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // This is a regular comment
    /// function doSomething() {}
    ///
    /// // Note: This explains something
    /// const x = 1;
    /// ```
    ///
    /// ### Options
    ///
    /// This rule has an options object with the following defaults:
    ///
    /// ```json
    /// {
    ///   "terms": ["todo", "fixme", "xxx"],
    ///   "location": "start",
    ///   "decoration": []
    /// }
    /// ```
    ///
    /// #### `terms`
    ///
    /// An array of terms to match. The matching is case-insensitive.
    ///
    /// #### `location`
    ///
    /// Where to check for the terms:
    /// - `"start"` (default): Terms must appear at the start of the comment (after any decoration)
    /// - `"anywhere"`: Terms can appear anywhere in the comment
    ///
    /// #### `decoration`
    ///
    /// An array of characters to ignore at the start of comments when `location` is `"start"`.
    /// Useful for ignoring common comment decorations like `*` in JSDoc-style comments.
    NoWarningComments,
    eslint,
    pedantic
);

impl Rule for NoWarningComments {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let terms = config.and_then(|v| v.get("terms")).and_then(|v| v.as_array()).map_or_else(
            || vec!["todo".to_string(), "fixme".to_string(), "xxx".to_string()],
            |arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.cow_to_lowercase().into_owned())
                    .collect::<Vec<_>>()
            },
        );

        let location = config.and_then(|v| v.get("location")).and_then(|v| v.as_str()).map_or(
            Location::Start,
            |s| {
                if s.eq_ignore_ascii_case("anywhere") {
                    Location::Anywhere
                } else {
                    Location::Start
                }
            },
        );

        let decoration = config
            .and_then(|v| v.get("decoration"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|v| v.as_str()).map(str::to_string).collect::<FxHashSet<_>>()
            })
            .unwrap_or_default();

        Self::new(&terms, &location, &decoration)
    }

    fn run_once(&self, ctx: &LintContext) {
        for comment in ctx.semantic().comments() {
            let comment_text = ctx.source_range(comment.content_span());

            if comment_text.contains("no-warning-comments") {
                continue;
            }

            if let Some(matched_term) = self.matches_warning_term(comment_text) {
                ctx.diagnostic(no_warning_comments_diagnostic(
                    matched_term,
                    comment_text,
                    comment.span,
                ));
            }
        }
    }
}

impl NoWarningComments {
    fn new(terms: &[String], location: &Location, decoration: &FxHashSet<String>) -> Self {
        let patterns = Self::build_patterns(terms, location, decoration);
        Self(Box::new(NoWarningCommentsConfig { terms: terms.to_vec(), patterns }))
    }

    fn build_patterns(
        terms: &[String],
        location: &Location,
        decoration: &FxHashSet<String>,
    ) -> Vec<Regex> {
        let decoration_chars: String = decoration.iter().map(|s| regex::escape(s)).collect();

        terms
            .iter()
            .filter_map(|term| {
                let ends_with_word =
                    term.chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_');
                let suffix = if ends_with_word { r"\b" } else { "" };
                let escaped_term = regex::escape(term);

                let pattern = match location {
                    Location::Start => {
                        format!(r"(?i)^[\s{decoration_chars}]*{escaped_term}{suffix}")
                    }
                    Location::Anywhere => {
                        let starts_with_word =
                            term.chars().next().is_some_and(|c| c.is_alphanumeric() || c == '_');
                        let prefix = if starts_with_word { r"\b" } else { "" };

                        format!(r"(?i){prefix}{escaped_term}{suffix}")
                    }
                };

                Regex::new(&pattern).ok()
            })
            .collect()
    }

    fn matches_warning_term(&self, comment_text: &str) -> Option<&str> {
        self.0
            .terms
            .iter()
            .zip(&self.0.patterns)
            .find_map(|(term, pattern)| pattern.is_match(comment_text).then_some(term.as_str()))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("// any comment", Some(serde_json::json!([{ "terms": ["fixme"] }]))),
        ("// any comment", Some(serde_json::json!([{ "terms": ["fixme", "todo"] }]))),
        ("// any comment", None),
        ("// any comment", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "// any comment with TODO, FIXME or XXX",
            Some(serde_json::json!([{ "location": "start" }])),
        ),
        ("// any comment with TODO, FIXME or XXX", None),
        ("/* any block comment */", Some(serde_json::json!([{ "terms": ["fixme"] }]))),
        ("/* any block comment */", Some(serde_json::json!([{ "terms": ["fixme", "todo"] }]))),
        ("/* any block comment */", None),
        ("/* any block comment */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "/* any block comment with TODO, FIXME or XXX */",
            Some(serde_json::json!([{ "location": "start" }])),
        ),
        ("/* any block comment with TODO, FIXME or XXX */", None),
        ("/* any block comment with (TODO, FIXME's or XXX!) */", None),
        (
            "// comments containing terms as substrings like TodoMVC",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "anywhere" }])),
        ),
        (
            "// special regex characters don't cause a problem",
            Some(serde_json::json!([{ "terms": ["[aeiou]"], "location": "anywhere" }])),
        ),
        (
            r#"/*eslint no-warning-comments: [2, { "terms": ["todo", "fixme", "any other term"], "location": "anywhere" }]*/

			var x = 10;
			"#,
            None,
        ),
        (
            r#"/*eslint no-warning-comments: [2, { "terms": ["todo", "fixme", "any other term"], "location": "anywhere" }]*/

			var x = 10;
			"#,
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        ("// foo", Some(serde_json::json!([{ "terms": ["foo-bar"] }]))),
        (
            "/** multi-line block comment with lines starting with
			TODO
			FIXME or
			XXX
			*/",
            None,
        ),
        ("//!TODO ", Some(serde_json::json!([{ "decoration": ["*"] }]))),
        (
            "// not a todo! here",
            Some(serde_json::json!([{ "terms": ["todo!"], "location": "start" }])), // additional test not in ESLint
        ),
    ];

    let fail = vec![
        ("// fixme", None),
        ("// any fixme", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("// any fixme", Some(serde_json::json!([{ "terms": ["fixme"], "location": "anywhere" }]))),
        ("// any FIXME", Some(serde_json::json!([{ "terms": ["fixme"], "location": "anywhere" }]))),
        ("// any fIxMe", Some(serde_json::json!([{ "terms": ["fixme"], "location": "anywhere" }]))),
        (
            "/* any fixme */",
            Some(serde_json::json!([{ "terms": ["FIXME"], "location": "anywhere" }])),
        ),
        (
            "/* any FIXME */",
            Some(serde_json::json!([{ "terms": ["FIXME"], "location": "anywhere" }])),
        ),
        (
            "/* any fIxMe */",
            Some(serde_json::json!([{ "terms": ["FIXME"], "location": "anywhere" }])),
        ),
        (
            "// any fixme or todo",
            Some(serde_json::json!([{ "terms": ["fixme", "todo"], "location": "anywhere" }])),
        ),
        (
            "/* any fixme or todo */",
            Some(serde_json::json!([{ "terms": ["fixme", "todo"], "location": "anywhere" }])),
        ),
        ("/* any fixme or todo */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("/* fixme and todo */", None),
        ("/* fixme and todo */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("/* any fixme */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("/* fixme! */", Some(serde_json::json!([{ "terms": ["fixme"] }]))),
        (
            "// regex [litera|$]",
            Some(serde_json::json!([{ "terms": ["[litera|$]"], "location": "anywhere" }])),
        ),
        ("/* eslint one-var: 2 */", Some(serde_json::json!([{ "terms": ["eslint"] }]))),
        (
            "/* eslint one-var: 2 */",
            Some(serde_json::json!([{ "terms": ["one"], "location": "anywhere" }])),
        ),
        (
            "/* any block comment with TODO, FIXME or XXX */",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "/* any block comment with (TODO, FIXME's or XXX!) */",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "/**
			 *any block comment
			*with (TODO, FIXME's or XXX!) **/",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "// any comment with TODO, FIXME or XXX",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        ("// TODO: something small", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "// TODO: something really longer than 40 characters",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "/* TODO: something
			 really longer than 40 characters
			 and also a new line */",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        ("// TODO: small", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "// https://github.com/eslint/eslint/pull/13522#discussion_r470293411 TODO",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "// Comment ending with term followed by punctuation TODO!",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "anywhere" }])),
        ),
        (
            "// Comment ending with term including punctuation TODO!",
            Some(serde_json::json!([{ "terms": ["todo!"], "location": "anywhere" }])),
        ),
        (
            "// Comment ending with term including punctuation followed by more TODO!!!",
            Some(serde_json::json!([{ "terms": ["todo!"], "location": "anywhere" }])),
        ),
        (
            "// !TODO comment starting with term preceded by punctuation",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "anywhere" }])),
        ),
        (
            "// !TODO comment starting with term including punctuation",
            Some(serde_json::json!([{ "terms": ["!todo"], "location": "anywhere" }])),
        ),
        (
            "// !!!TODO comment starting with term including punctuation preceded by more",
            Some(serde_json::json!([{ "terms": ["!todo"], "location": "anywhere" }])),
        ),
        (
            "// FIX!term ending with punctuation followed word character",
            Some(serde_json::json!([{ "terms": ["FIX!"], "location": "anywhere" }])),
        ),
        (
            "// Term starting with punctuation preceded word character!FIX",
            Some(serde_json::json!([{ "terms": ["!FIX"], "location": "anywhere" }])),
        ),
        (
            "//!XXX comment starting with no spaces (anywhere)",
            Some(serde_json::json!([{ "terms": ["!xxx"], "location": "anywhere" }])),
        ),
        (
            "//!XXX comment starting with no spaces (start)",
            Some(serde_json::json!([{ "terms": ["!xxx"], "location": "start" }])),
        ),
        (
            "/*
			TODO undecorated multi-line block comment (start)
			*/",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "start" }])),
        ),
        (
            "///// TODO decorated single-line comment with decoration array
			 /////",
            Some(
                serde_json::json!([				{ "terms": ["todo"], "location": "start", "decoration": ["*", "/"] },			]),
            ),
        ),
        (
            "///*/*/ TODO decorated single-line comment with multiple decoration characters (start)
			 /////",
            Some(
                serde_json::json!([				{ "terms": ["todo"], "location": "start", "decoration": ["*", "/"] },			]),
            ),
        ),
        (
            "//**TODO term starts with a decoration character",
            Some(
                serde_json::json!([				{ "terms": ["*todo"], "location": "start", "decoration": ["*"] },			]),
            ),
        ),
        (
            "// todo! with punctuation at start",
            Some(serde_json::json!([{ "terms": ["todo!"], "location": "start" }])),
        ), // additional test not in ESLint
    ];

    Tester::new(NoWarningComments::NAME, NoWarningComments::PLUGIN, pass, fail).test_and_snapshot();
}
