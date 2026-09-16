use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::Comment;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn expected_block_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a block comment instead of consecutive line comments.")
        .with_help("Merge these line comments into a single block comment.")
        .with_label(span)
}

fn expected_bare_block_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a block comment without padding stars.")
        .with_help("Remove the `*` at the start of each line of this block comment.")
        .with_label(span)
}

fn start_newline_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a linebreak after '/*'.")
        .with_help("Move the comment text onto its own line.")
        .with_label(span)
}

fn end_newline_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a linebreak before '*/'.")
        .with_help("Move the closing delimiter onto its own line.")
        .with_label(span)
}

fn missing_star_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a '*' at the start of this line.")
        .with_help("Add a `*` aligned with the `*` of the opening delimiter.")
        .with_label(span)
}

fn alignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected this line to be aligned with the start of the comment.")
        .with_help("Align the `*` of this line with the `*` of the opening delimiter.")
        .with_label(span)
}

fn expected_lines_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected multiple line comments instead of a block comment.")
        .with_help("Split this block comment into consecutive line comments.")
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CommentStyle {
    /// Block comments with an aligned `*` at the start of every line.
    #[default]
    StarredBlock,
    /// Block comments without padding stars.
    BareBlock,
    /// Consecutive line comments.
    SeparateLines,
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct SeparateLinesOptionsJson {
    /// Whether to also apply `"separate-lines"` to JSDoc comments (`/** ... */`).
    #[serde(rename = "checkJSDoc")]
    check_jsdoc: bool,
    /// Whether to also apply `"separate-lines"` to exclamation comments (`/*! ... */`).
    check_exclamation: bool,
}

/// Configuration for the `multiline-comment-style` rule.
///
/// The first element selects the comment style. When it is `"separate-lines"`, a
/// second element may supply additional options.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[expect(dead_code)]
struct MultilineCommentStyleOptions(CommentStyle, Option<SeparateLinesOptionsJson>);

#[derive(Debug, Clone, Default)]
pub struct MultilineCommentStyle {
    style: CommentStyle,
    check_jsdoc: bool,
    check_exclamation: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a particular style for multiline comments: either a block comment
    /// with aligned stars, a bare block comment, or consecutive line comments.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing block comments and runs of line comments in the same codebase makes
    /// multiline prose harder to scan and harder to edit, because every comment has
    /// to be re-indented or re-prefixed by hand in a different way.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"starred-block"` option:
    /// ```js
    /// // this line
    /// // calls foo()
    /// foo();
    ///
    /// /* this line
    ///    calls foo() */
    /// foo();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"starred-block"` option:
    /// ```js
    /// /*
    ///  * this line
    ///  * calls foo()
    ///  */
    /// foo();
    ///
    /// // single-line comment
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"bare-block"` option:
    /// ```js
    /// /* this line
    ///    calls foo() */
    /// foo();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"separate-lines"` option:
    /// ```js
    /// // this line
    /// // calls foo()
    /// foo();
    /// ```
    ///
    /// ### Options
    ///
    /// The `"separate-lines"` option accepts an object with these properties:
    ///
    /// #### checkJSDoc
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Whether JSDoc comments should be reported too.
    ///
    /// #### checkExclamation
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Whether exclamation comments (`/*! ... */`) should be reported too.
    MultilineCommentStyle,
    eslint,
    style,
    fix_conditional,
    config = MultilineCommentStyleOptions,
    version = "next",
    short_description = "Enforce a particular style for multiline comments.",
);

impl Rule for MultilineCommentStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let Some(arr) = value.as_array() else {
            return Ok(Self::default());
        };

        let style = arr
            .first()
            .and_then(|v| serde_json::from_value::<CommentStyle>(v.clone()).ok())
            .unwrap_or_default();

        let options = arr.get(1).map_or_else(
            || Ok(SeparateLinesOptionsJson::default()),
            |v| serde_json::from_value::<SeparateLinesOptionsJson>(v.clone()),
        )?;

        Ok(Self {
            style,
            check_jsdoc: options.check_jsdoc,
            check_exclamation: options.check_exclamation,
        })
    }

    fn run_once(&self, ctx: &LintContext) {
        let source_text = ctx.source_text();
        let comments = ctx.semantic().comments();
        if comments.is_empty() {
            return;
        }

        let lines = SourceLines::new(source_text);

        // Keep only the comments that start their own line. A comment preceded by a
        // token on the same line is left alone, exactly like ESLint's
        // `tokenBefore.loc.end.line < comment.loc.start.line` check.
        let eligible = comments
            .iter()
            .filter(|comment| {
                let is_hashbang = comment.span.start == 0 && source_text.starts_with("#!");
                !is_hashbang
                    && !is_directive_comment(ctx.source_range(comment.content_span()))
                    && is_whitespaces(lines.text_before_on_line(comment.span.start))
            })
            .collect::<Vec<_>>();

        let groups = group_comments(&eligible, &lines, source_text);

        for group in &groups {
            // A lone comment that fits on one line is never a multiline comment.
            if group.len() == 1 && lines.is_single_line(group[0].span) {
                continue;
            }

            match self.style {
                CommentStyle::StarredBlock => check_starred_block(group, ctx, &lines),
                CommentStyle::BareBlock => check_bare_block(group, ctx, &lines),
                CommentStyle::SeparateLines => self.check_separate_lines(group, ctx, &lines),
            }
        }
    }
}

impl MultilineCommentStyle {
    fn check_separate_lines(&self, group: &[&Comment], ctx: &LintContext, lines: &SourceLines) {
        let first = group[0];
        let is_jsdoc = is_jsdoc_comment(group, ctx);
        let is_exclamation = is_exclamation_comment(group, ctx);

        if first.is_line()
            || (!self.check_jsdoc && is_jsdoc)
            || (!self.check_exclamation && is_exclamation)
        {
            return;
        }

        let mut comment_lines = comment_lines(group, ctx, lines);

        if is_jsdoc || is_exclamation {
            if comment_lines.len() < 2 {
                comment_lines.clear();
            } else {
                comment_lines = comment_lines[1..comment_lines.len() - 1].to_vec();
            }
        }

        // A block comment followed by code on the same line cannot become line comments.
        if !is_whitespaces(lines.text_after_on_line(first.span.end)) {
            return;
        }

        let initial_offset = lines.initial_offset(first.span.start).to_string();
        ctx.diagnostic_with_fix(
            expected_lines_diagnostic(Span::sized(first.span.start, 2)),
            |fixer| {
                fixer
                    .replace(first.span, convert_to_separate_lines(&comment_lines, &initial_offset))
            },
        );
    }
}

fn check_starred_block(group: &[&Comment], ctx: &LintContext, lines: &SourceLines) {
    let first = group[0];
    let comment_lines = comment_lines(group, ctx, lines);

    if comment_lines.iter().any(|line| line.contains("*/")) {
        return;
    }

    if group.len() > 1 {
        let last = group[group.len() - 1];
        let initial_offset = lines.initial_offset(first.span.start).to_string();
        ctx.diagnostic_with_fix(
            expected_block_diagnostic(Span::new(first.span.start, last.span.end)),
            |fixer| {
                if comment_lines.iter().any(|line| line.starts_with('/')) {
                    fixer.noop()
                } else {
                    fixer.replace(
                        Span::new(first.span.start, last.span.end),
                        convert_to_starred_block(&comment_lines, &initial_offset),
                    )
                }
            },
        );
        return;
    }

    let value = ctx.source_range(first.content_span());
    let value_lines = split_lines(value);
    let initial_offset = lines.initial_offset(first.span.start);
    let expected_line_prefix = format!("{initial_offset} *");

    // `/*` must be followed by a linebreak.
    let first_value_line = value_lines[0];
    let first_line_rest = first_value_line.strip_prefix(['*', '!']).unwrap_or(first_value_line);
    if !is_whitespaces(first_line_rest) {
        let insert_at =
            if value.starts_with(['*', '!']) { first.span.start + 1 } else { first.span.start };
        let offset_len = leading_whitespace(first_line_rest).len();
        let text = format!("\n{expected_line_prefix}{}", if offset_len == 0 { " " } else { "" });
        ctx.diagnostic_with_fix(
            start_newline_diagnostic(Span::sized(first.span.start, 2)),
            |fixer| fixer.insert_text_after_range(Span::sized(insert_at, 2), text),
        );
    }

    // `*/` must be preceded by a linebreak.
    if !is_whitespaces(value_lines[value_lines.len() - 1]) {
        let text = format!("\n{expected_line_prefix}/");
        ctx.diagnostic_with_fix(
            end_newline_diagnostic(Span::new(first.span.end - 2, first.span.end)),
            |fixer| fixer.replace(Span::new(first.span.end - 2, first.span.end), text),
        );
    }

    // Every line but the first must start with the aligned `*`.
    let first_line_index = lines.line_index(first.span.start);
    let last_line_index = lines.line_index(first.span.end);
    for line_index in (first_line_index + 1)..=last_line_index {
        let line_text = lines.line_text(line_index);
        if line_text.starts_with(&expected_line_prefix) {
            continue;
        }

        let line_start = lines.line_start(line_index);
        let line_span = Span::sized(line_start, u32_len(line_text));
        let is_alignment = is_starred_comment_line(line_text);

        let prefix = leading_whitespace(line_text);
        let (diagnostic, replacement_span, replacement) = if is_alignment {
            (
                alignment_diagnostic(line_span),
                Span::sized(line_start, u32_len(prefix) + 1),
                expected_line_prefix.clone(),
            )
        } else {
            (
                missing_star_diagnostic(line_span),
                Span::sized(line_start, u32_len(prefix)),
                format!(
                    "{expected_line_prefix}{}",
                    missing_star_offset(&value_lines, line_text, prefix, first_line_index, lines)
                ),
            )
        };

        ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(replacement_span, replacement));
    }
}

/// Computes the indentation that a line missing its `*` should keep, by aligning it
/// with the first non-blank line of the comment.
fn missing_star_offset(
    value_lines: &[&str],
    line_text: &str,
    line_prefix: &str,
    first_line_index: usize,
    lines: &SourceLines,
) -> String {
    for (idx, value_line) in value_lines.iter().enumerate() {
        if is_whitespaces(value_line) {
            continue;
        }

        let align_with = lines.line_text(first_line_index + idx);
        // `/^(\s*(?:\/?\*)?(\s*))/`
        let leading = leading_whitespace(align_with);
        let after_leading = &align_with[leading.len()..];
        let star = if after_leading.starts_with("/*") {
            "/*"
        } else if after_leading.starts_with('*') {
            "*"
        } else {
            ""
        };
        let inner_offset = leading_whitespace(&after_leading[star.len()..]);
        let prefix_len = leading.len() + star.len() + inner_offset.len();

        let mut offset =
            format!("{}{inner_offset}", &line_prefix[line_prefix.len().min(prefix_len)..]);

        // `/^\s*\/?\*\s/`
        let has_space_after_star = !star.is_empty() && !inner_offset.is_empty();
        if !is_whitespaces(line_text) && !has_space_after_star {
            offset.insert(0, ' ');
        }

        return offset;
    }

    // Every line of the comment is blank. ESLint leaves its `offset` variable
    // undefined here and interpolates it, writing the literal text `undefined` into
    // the source; an empty offset is the same fix without the corruption.
    String::new()
}

fn check_bare_block(group: &[&Comment], ctx: &LintContext, lines: &SourceLines) {
    if is_jsdoc_comment(group, ctx) || is_exclamation_comment(group, ctx) {
        return;
    }

    let first = group[0];
    let comment_lines = comment_lines(group, ctx, lines);
    let initial_offset = lines.initial_offset(first.span.start).to_string();

    // Consecutive line comments become a single block comment.
    if first.is_line()
        && comment_lines.len() > 1
        && !comment_lines.iter().any(|line| line.contains("*/"))
    {
        let last = group[group.len() - 1];
        let span = Span::new(first.span.start, last.span.end);
        let replacement = convert_to_block(&comment_lines, &initial_offset);
        ctx.diagnostic_with_fix(expected_block_diagnostic(span), |fixer| {
            fixer.replace(span, replacement)
        });
    }

    // Block comments must not have a `*` at the start of each line.
    if is_starred_block_comment(group, ctx) {
        let replacement = convert_to_block(&comment_lines, &initial_offset);
        ctx.diagnostic_with_fix(
            expected_bare_block_diagnostic(Span::sized(first.span.start, 2)),
            |fixer| fixer.replace(first.span, replacement),
        );
    }
}

// ----------------------------------------------------------------------------
// Comment grouping
// ----------------------------------------------------------------------------

/// Collects runs of line comments that sit on consecutive lines with nothing else
/// between them. Every other comment forms a group of its own.
fn group_comments<'c>(
    eligible: &[&'c Comment],
    lines: &SourceLines,
    source_text: &str,
) -> Vec<Vec<&'c Comment>> {
    let mut groups: Vec<Vec<&Comment>> = Vec::with_capacity(eligible.len());

    for (index, comment) in eligible.iter().enumerate() {
        let is_continuation = index > 0 && comment.is_line() && {
            let prev = eligible[index - 1];
            prev.is_line()
                && is_whitespaces(&source_text[prev.span.end as usize..comment.span.start as usize])
                && lines.line_index(comment.span.start) == lines.line_index(prev.span.end) + 1
        };

        if is_continuation && let Some(last) = groups.last_mut() {
            last.push(comment);
            continue;
        }

        groups.push(vec![comment]);
    }

    groups
}

// ----------------------------------------------------------------------------
// Comment shape predicates
// ----------------------------------------------------------------------------

fn is_starred_comment_line(line: &str) -> bool {
    line[leading_whitespace(line).len()..].starts_with('*')
}

fn is_starred_block_comment(group: &[&Comment], ctx: &LintContext) -> bool {
    let first = group[0];
    if first.is_line() {
        return false;
    }

    let lines = split_lines(ctx.source_range(first.content_span()));
    let last = lines.len() - 1;
    lines.iter().enumerate().all(|(i, line)| {
        if i == 0 || i == last { is_whitespaces(line) } else { is_starred_comment_line(line) }
    })
}

fn is_jsdoc_comment(group: &[&Comment], ctx: &LintContext) -> bool {
    is_decorated_block_comment(group, ctx, '*')
}

fn is_exclamation_comment(group: &[&Comment], ctx: &LintContext) -> bool {
    is_decorated_block_comment(group, ctx, '!')
}

/// Shared shape of JSDoc (`/** ... */`) and exclamation (`/*! ... */`) comments:
/// a marker alone on the first line, indented content, and a blank closing line.
fn is_decorated_block_comment(group: &[&Comment], ctx: &LintContext, marker: char) -> bool {
    let first = group[0];
    if first.is_line() {
        return false;
    }

    let lines = split_lines(ctx.source_range(first.content_span()));
    let Some(rest) = lines[0].strip_prefix(marker) else {
        return false;
    };

    is_whitespaces(rest)
        && lines.len() > 1
        && lines[1..lines.len() - 1].iter().all(|line| has_space_in_leading_whitespace(line))
        && is_whitespaces(lines[lines.len() - 1])
}

/// Comments whose position and shape carry meaning to another tool, which this rule
/// must never rewrite.
fn is_directive_comment(value: &str) -> bool {
    // `/^\/\s*<(?:reference|amd-)/` — triple-slash references are matched unindented.
    if let Some(rest) = value.strip_prefix('/') {
        let rest = &rest[leading_whitespace(rest).len()..];
        if let Some(rest) = rest.strip_prefix('<')
            && (rest.starts_with("reference") || rest.starts_with("amd-"))
        {
            return true;
        }
    }

    let value = &value[leading_whitespace(value).len()..];

    if value.starts_with("eslint") || value.starts_with("jscs") {
        return true;
    }

    if let Some(rest) = value.strip_prefix("@ts-")
        && ["expect-error", "ignore", "nocheck", "check"]
            .iter()
            .any(|directive| is_bare_directive(rest, directive))
    {
        return true;
    }

    if is_bare_directive(value, "prettier-ignore") {
        return true;
    }

    // `/^\s*[vc]8\s+ignore(?![\w-])/`
    if let Some(rest) = value.strip_prefix(['v', 'c']).and_then(|rest| rest.strip_prefix('8'))
        && let Some(rest) = skip_required_whitespace(rest)
        && is_bare_directive(rest, "ignore")
    {
        return true;
    }

    // `/^\s*node:coverage\s+(?:disable|enable|ignore\s+next)(?![\w-])/`
    if let Some(rest) = value.strip_prefix("node:coverage")
        && let Some(rest) = skip_required_whitespace(rest)
    {
        if is_bare_directive(rest, "disable") || is_bare_directive(rest, "enable") {
            return true;
        }
        if let Some(rest) = rest.strip_prefix("ignore")
            && let Some(rest) = skip_required_whitespace(rest)
            && is_bare_directive(rest, "next")
        {
            return true;
        }
    }

    // `/^\s*webpack(?:ChunkName|...)\s*:/`
    if let Some(rest) = value.strip_prefix("webpack")
        && let Some(rest) =
            WEBPACK_DIRECTIVES.iter().find_map(|directive| rest.strip_prefix(*directive))
        && rest[leading_whitespace(rest).len()..].starts_with(':')
    {
        return true;
    }

    ["jshint", "jslint", "istanbul", "globals", "global", "exported"].iter().any(|directive| {
        value.strip_prefix(*directive).and_then(skip_required_whitespace).is_some()
    })
}

const WEBPACK_DIRECTIVES: [&str; 9] = [
    "ChunkName",
    "FetchPriority",
    "Mode",
    "Exports",
    "Include",
    "Exclude",
    "Prefetch",
    "Preload",
    "Ignore",
];

/// Matches `directive` followed by the `(?![\w-])` boundary the directive patterns use.
fn is_bare_directive(value: &str, directive: &str) -> bool {
    value.strip_prefix(directive).is_some_and(|rest| {
        !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '-')
    })
}

/// Consumes `\s+`, returning `None` when there is no whitespace to consume.
fn skip_required_whitespace(value: &str) -> Option<&str> {
    let whitespace = leading_whitespace(value);
    if whitespace.is_empty() { None } else { Some(&value[whitespace.len()..]) }
}

// ----------------------------------------------------------------------------
// Comment content normalization
// ----------------------------------------------------------------------------

/// The lines of a comment group with their leading decoration removed, ready to be
/// re-emitted in any of the three styles.
fn comment_lines(group: &[&Comment], ctx: &LintContext, lines: &SourceLines) -> Vec<String> {
    let first = group[0];

    if first.is_line() {
        let values = group
            .iter()
            .map(|comment| ctx.source_range(comment.content_span()))
            .collect::<Vec<_>>();
        return process_separate_line_comments(&values);
    }

    let value = ctx.source_range(first.content_span());

    if is_starred_block_comment(group, ctx) {
        return process_starred_block_comment(value);
    }

    process_bare_block_comment(value, lines.initial_offset(first.span.start))
}

fn process_separate_line_comments(values: &[&str]) -> Vec<String> {
    let all_lines_have_leading_space =
        values.iter().all(|value| is_whitespaces(value) || value.starts_with(' '));

    values
        .iter()
        .map(|value| {
            if all_lines_have_leading_space {
                value.strip_prefix(' ').unwrap_or(value)
            } else {
                value
            }
            .to_string()
        })
        .collect()
}

fn process_starred_block_comment(value: &str) -> Vec<String> {
    let all = split_lines(value);
    if all.len() < 2 {
        return Vec::new();
    }

    let lines = all[1..all.len() - 1]
        .iter()
        .map(|line| if is_whitespaces(line) { "" } else { *line })
        .collect::<Vec<_>>();

    let all_lines_have_leading_space = lines.iter().all(|line| {
        let without_star = strip_first_star(line, false);
        is_whitespaces(&without_star) || without_star.starts_with(' ')
    });

    lines.iter().map(|line| strip_first_star(line, all_lines_have_leading_space)).collect()
}

fn process_bare_block_comment(value: &str, initial_offset: &str) -> Vec<String> {
    let leading_whitespace_len = initial_offset.chars().count() + 3;
    let lines = split_lines(value);

    let lines_info = lines
        .iter()
        .map(|line| {
            let line = if is_whitespaces(line) { "" } else { *line };
            // `/^(\s*\*?\s*)(.*)/`
            let leading = leading_whitespace(line);
            let after_leading = &line[leading.len()..];
            let star_len = usize::from(after_leading.starts_with('*'));
            let inner = leading_whitespace(&after_leading[star_len..]);
            let offset_len = leading.chars().count() + star_len + inner.chars().count();
            let offset = &line[..leading.len() + star_len + inner.len()];
            (offset, offset_len, &line[leading.len() + star_len + inner.len()..])
        })
        .collect::<Vec<_>>();

    // Line up every line against the least indented one; the first line is skipped
    // because it shares its line with the opening delimiter.
    let offset_len = lines_info
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, (offset, _, contents))| !(offset.is_empty() && contents.is_empty()))
        .filter_map(|(_, (_, offset_len, _))| leading_whitespace_len.checked_sub(*offset_len))
        .max()
        .unwrap_or(0);

    lines_info
        .iter()
        .map(|(offset, line_offset_len, contents)| {
            if *line_offset_len > leading_whitespace_len {
                let skip = leading_whitespace_len.saturating_sub(offset_len);
                let kept = offset.chars().skip(skip).collect::<String>();
                format!("{kept}{contents}")
            } else {
                (*contents).to_string()
            }
        })
        .collect()
}

/// Removes the first `/\s*\*/` match from `line`, optionally taking one space with it.
///
/// The regex is unanchored, so the match is the first `*` on the line together with
/// the run of whitespace immediately preceding it.
fn strip_first_star(line: &str, strip_trailing_space: bool) -> String {
    let Some(star) = line.find('*') else {
        return line.to_string();
    };

    let start = line[..star]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_js_whitespace(*c))
        .last()
        .map_or(star, |(i, _)| i);

    let mut end = star + 1;
    if strip_trailing_space && line[end..].starts_with(' ') {
        end += 1;
    }

    format!("{}{}", &line[..start], &line[end..])
}

// ----------------------------------------------------------------------------
// Comment construction
// ----------------------------------------------------------------------------

fn convert_to_starred_block(comment_lines: &[String], initial_offset: &str) -> String {
    let body = comment_lines
        .iter()
        .map(|line| format!("{initial_offset} * {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!("/*\n{body}\n{initial_offset} */")
}

fn convert_to_separate_lines(comment_lines: &[String], initial_offset: &str) -> String {
    comment_lines
        .iter()
        .map(|line| format!("// {line}"))
        .collect::<Vec<_>>()
        .join(&format!("\n{initial_offset}"))
}

fn convert_to_block(comment_lines: &[String], initial_offset: &str) -> String {
    format!("/* {} */", comment_lines.join(&format!("\n{initial_offset}   ")))
}

// ----------------------------------------------------------------------------
// Source text helpers
// ----------------------------------------------------------------------------

/// Byte offsets of every line start, so comments can be related to the raw lines
/// they were written on.
struct SourceLines<'a> {
    text: &'a str,
    line_starts: Vec<u32>,
}

impl<'a> SourceLines<'a> {
    fn new(text: &'a str) -> Self {
        let bytes = text.as_bytes();
        let mut line_starts = vec![0];
        let mut i = 0;

        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    i += 1;
                    line_starts.push(u32_from(i));
                }
                b'\r' => {
                    i += 1;
                    if bytes.get(i) == Some(&b'\n') {
                        i += 1;
                    }
                    line_starts.push(u32_from(i));
                }
                // U+2028 and U+2029 are line terminators too.
                0xE2 if bytes.get(i + 1) == Some(&0x80)
                    && matches!(bytes.get(i + 2), Some(0xA8 | 0xA9)) =>
                {
                    i += 3;
                    line_starts.push(u32_from(i));
                }
                _ => i += 1,
            }
        }

        Self { text, line_starts }
    }

    fn line_index(&self, offset: u32) -> usize {
        self.line_starts.partition_point(|&start| start <= offset) - 1
    }

    fn line_start(&self, index: usize) -> u32 {
        self.line_starts[index]
    }

    /// The line's text without its terminator.
    fn line_text(&self, index: usize) -> &'a str {
        let start = self.line_starts[index] as usize;
        let end = self.line_starts.get(index + 1).map_or(self.text.len(), |&end| end as usize);
        let line = &self.text[start..end];

        line.strip_suffix("\r\n")
            .or_else(|| line.strip_suffix('\n'))
            .or_else(|| line.strip_suffix('\r'))
            .or_else(|| line.strip_suffix('\u{2028}'))
            .or_else(|| line.strip_suffix('\u{2029}'))
            .unwrap_or(line)
    }

    fn is_single_line(&self, span: Span) -> bool {
        self.line_index(span.start) == self.line_index(span.end)
    }

    /// The text between the start of the line and `offset`.
    fn text_before_on_line(&self, offset: u32) -> &'a str {
        &self.text[self.line_start(self.line_index(offset)) as usize..offset as usize]
    }

    /// The text between `offset` and the end of the line.
    fn text_after_on_line(&self, offset: u32) -> &'a str {
        let line_index = self.line_index(offset);
        let line_end = self.line_start(line_index) as usize + self.line_text(line_index).len();
        &self.text[offset as usize..line_end.max(offset as usize)]
    }

    /// The indentation a comment starting at `offset` is written at.
    fn initial_offset(&self, offset: u32) -> &'a str {
        self.text_before_on_line(offset)
    }
}

/// `\s` as JavaScript defines it, which is not quite Rust's `char::is_whitespace`.
const fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{9}'
            | '\u{a}'
            | '\u{b}'
            | '\u{c}'
            | '\u{d}'
            | '\u{20}'
            | '\u{a0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

fn is_whitespaces(value: &str) -> bool {
    value.chars().all(is_js_whitespace)
}

fn leading_whitespace(value: &str) -> &str {
    let end = value.find(|c: char| !is_js_whitespace(c)).unwrap_or(value.len());
    &value[..end]
}

/// `/^\s* /` — the leading whitespace run must contain a literal space.
fn has_space_in_leading_whitespace(line: &str) -> bool {
    leading_whitespace(line).contains(' ')
}

/// Splits on `/\r\n|[\r\n  ]/`, which is not the same set of breaks as
/// [`str::lines`].
fn split_lines(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(&value[start..i]);
                i += 1;
                start = i;
            }
            b'\r' => {
                lines.push(&value[start..i]);
                i += 1;
                if bytes.get(i) == Some(&b'\n') {
                    i += 1;
                }
                start = i;
            }
            0xE2 if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(0xA8 | 0xA9)) =>
            {
                lines.push(&value[start..i]);
                i += 3;
                start = i;
            }
            _ => i += 1,
        }
    }

    lines.push(&value[start..]);
    lines
}

#[expect(clippy::cast_possible_truncation)]
fn u32_from(value: usize) -> u32 {
    value as u32
}

fn u32_len(value: &str) -> u32 {
    u32_from(value.len())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "\n            /*\n             * this is\n             * a comment\n             */\n        ",
            None,
        ),
        (
            "\n            /**\n             * this is\n             * a JSDoc comment\n             */\n        ",
            None,
        ),
        (
            "\n            /*!\n             * this is\n             * an exclamation comment\n             */\n        ",
            None,
        ),
        ("\n            /*! this is a single line exclamation comment */\n        ", None),
        (
            "\n            /* eslint semi: [\n              \"error\"\n            ] */\n        ",
            None,
        ),
        ("\n            // this is a single-line comment\n        ", None),
        ("\n            /* foo */\n        ", None),
        (
            "\n            // this is a comment\n            foo();\n            // this is another comment\n        ",
            None,
        ),
        (
            "\n            /*\n             * Function overview\n             * ...\n             */\n\n            // Step 1: Do the first thing\n            foo();\n        ",
            None,
        ),
        (
            "\n            /*\n             * Function overview\n             * ...\n             */\n\n            /*\n             * Step 1: Do the first thing.\n             * The first thing is foo().\n             */\n            foo();\n        ",
            None,
        ),
        ("\t\t/**\n\t\t * this comment\n\t\t * is tab-aligned\n\t\t */", None),
        ("/**\r\n * this comment\r\n * uses windows linebreaks\r\n */", None),
        ("/**\u{2029} * this comment\u{2029} * uses paragraph separators\u{2029} */", None),
        ("\n            foo(/* this is an\n                inline comment */);\n        ", None),
        (
            "\n            // The following line comment\n            // contains '*/'.\n        ",
            None,
        ),
        (
            "\n                // The following line comment\n                // contains '*/'.\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 * this is\n                 * a comment\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /**\n                 * this is\n                 * a JSDoc comment\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /* eslint semi: [\n                  \"error\"\n                ] */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                // this is a single-line comment\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        ("\n                /* foo */\n            ", Some(serde_json::json!(["starred-block"]))),
        (
            "\n                /*\n                 * foo\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        ("\n                /* foo */\n            ", Some(serde_json::json!(["bare-block"]))),
        (
            "\n                /*\n                   foo */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                   foo\n                */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n              foo */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n            foo\n        */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // this is\n                // a comment\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /* this is\n                   a comment */ foo;\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                // a comment\n\n                // another comment\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                // a comment\n\n                // another comment\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // a comment\n\n                // another comment\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /* eslint semi: \"error\" */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /**\n                 * This is\n                 * a JSDoc comment\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /**\n                 * This is\n                 * a JSDoc comment\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /**\n                 * This is\n                 * a JSDoc comment\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*!\n                 * This is\n                 * an exclamation comment\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*!\n                 * This is\n                 * an exclamation comment\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /*!\n                 * This is\n                 * an exclamation comment\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /* This is\n                   a comment */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /* This is\n                         a comment */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /* eslint semi: [\n                    \"error\"\n                ] */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /* The value of 5\n                 + 4 is 9, and the value of 5\n                 * 4 is 20. */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *    foo\n                 *  bar\n                 *   baz\n                 * qux\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /*    foo\n                 *  bar\n                 *   baz\n                 * qux\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /**\n                 *    JSDoc blocks\n                 *  are\n                 *   ignored\n                 * !\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /**\n                 *    JSDoc blocks\n                 *  are\n                 *   ignored\n                 * !\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /**\n                 *    JSDoc blocks\n                 *  are\n                 *   ignored\n                 * !\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*!\n                 *    Exclamation blocks\n                 *  are\n                 *   ignored\n                 * !\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*!\n                 *    Exclamation blocks\n                 *  are\n                 *   ignored\n                 * !\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /*!\n                 *    Exclamation blocks\n                 *  are\n                 *   ignored\n                 * !\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * // a line comment\n                 *some.code();\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// djb2 algorithm\nlet hash = 5381;\nfor (let i = 0; i < view.length; i++) {\n\n  // eslint-disable\n  // hash * 33 + current byte -> truncate\n  hash = (((hash << 5) + hash) + view[i]) | 0;\n}",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// eslint-disable\n// @ts-nocheck\n\nimport type { ESLint } from 'eslint';",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// @ts-expect-error TS(2322) some message\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// @ts-ignore\n// @ts-expect-error\n// @ts-nocheck\n// @ts-check\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// keep this matrix laid out by hand\n// prettier-ignore\nconst m = [\n  1,0,0,\n  0,1,0,\n  0,0,1,\n];",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// v8 ignore next\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// v8 ignore if -- @preserve\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// v8 ignore start -- @preserve\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// v8 ignore stop -- @preserve\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// v8 ignore file -- @preserve\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// c8 ignore next\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// c8 ignore start\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// c8 ignore stop\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// node:coverage ignore next\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// node:coverage disable\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// node:coverage enable\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackChunkName: \"chunk\"\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackFetchPriority: \"high\"\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackMode: \"lazy\"\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackExports: [\"default\"]\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackInclude: /\\.json$/\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackExclude: /\\.noimport\\.json$/\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackPrefetch: true\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackPreload: true\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// why this directive is needed\n// webpackIgnore: true\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "let x = 5; // first number\n// second number\nlet y = 10;\n\nconsole.log(x + y);",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "/// <reference types=\"vite/client\" />\n/// <reference path=\"./types.d.ts\" />\n/// <reference lib=\"es2020\" />",
            Some(serde_json::json!(["starred-block"])),
        ),
    ];

    let fail = vec![
        ("\n                // these are\n                // line comments\n            ", None),
        ("\n                //foo\n                ///bar\n            ", None),
        (
            "\n                // foo\n                // bar\n\n                // baz\n                // qux\n            ",
            None,
        ),
        (
            "\n                //  foo\n                // bar\n                //    baz\n                // qux\n            ",
            None,
        ),
        (
            "\n                //  foo\n                //\n                //    baz\n                // qux\n            ",
            None,
        ),
        (
            "\n                //    foo\n                     // bar\n           //  baz\n                // qux\n            ",
            None,
        ),
        (
            "\n                /* this block\n                 * is missing a newline at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /** this JSDoc comment\n                 * is missing a newline at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*! this Exclamation comment\n                 * is missing a newline\n                 * at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*! this Exclamation comment is missing a newline at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * this block\n                 * is missing a newline at the end*/\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                   is missing a '*' at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                 is missing a '*' at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                is missing a '*' at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                      * has a '*' with the wrong offset at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                  /*\n                   * the following line\n                 * has a '*' with the wrong offset at the start\n                   */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the last line of this comment\n                 * is misaligned\n                   */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                *\n                 * is blank\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                  *\n                 * is blank\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the last line of this comment\n                 * is misaligned\n                   */ foo\n            ",
            None,
        ),
        (
            "\n                /*\n                 * foo\n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /**\n                 * JSDoc\n                 * Comment\n                 */\n            ",
            Some(serde_json::json!(["separate-lines",{"checkJSDoc":true}])),
        ),
        (
            "\n                /*!\n                 * Exclamation\n                 * Comment\n                 */\n            ",
            Some(serde_json::json!(["separate-lines",{"checkExclamation":true}])),
        ),
        (
            "\n                /* foo\n                 *bar\n                 baz\n                 qux*/\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                // foo\n                // bar\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // foo\n                //\n                // bar\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                //foo\n                //bar\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                //   foo\n                //   bar\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // foo\n              // bar\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                //    foo\n                     // bar\n           //  baz\n                // qux\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                * foo\n                * bar\n                */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                *foo\n                *bar\n                */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                *   foo\n                *   bar\n                */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                * foo\n             * bar\n                */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *    foo\n                 *  bar\n                 *   baz\n                 * qux\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        ("/*\n{\n    \"foo\": 1,\n    \"bar\": 2\n}\n*/", None),
        (
            "\n                /*\n                {\n                \t\"foo\": 1,\n                \t\"bar\": 2\n                }\n                */\n            ",
            None,
        ),
        ("/*\n{\n\t  \"foo\": 1,\n\t  \"bar\": 2\n}\n*/", None),
        (
            "\n                /*\n                {\n               \t\"foo\": 1,\n               \t\"bar\": 2\n                }\n                */\n            ",
            None,
        ),
        (
            "\n                \t /*\n                      \t    {\n                  \t    \"foo\": 1,\n                \t   \"bar\": 2\n                }\n                */\n            ",
            None,
        ),
        (
            "\n                //{\n                //    \"foo\": 1,\n                //    \"bar\": 2\n                //}\n            ",
            None,
        ),
        (
            "\n                /*\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 *\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 *\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *\n                 *{\n                 *    \"foo\": 1,\n                 *    \"bar\": 2\n                 *}\n                 *\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *{\n                 *    \"foo\": 1,\n                 *    \"bar\": 2\n                 *}\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *   {\n                 *       \"foo\": 1,\n                 *       \"bar\": 2\n                 *   }\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n            *{\n                 *    \"foo\": 1,\n                    *    \"bar\": 2\n                 *}\n                  */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *   {\n                 *       \"foo\": 1,\n                 *       \"bar\": 2\n           *}\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                {\n                    \"foo\": 1,\n                    \"bar\": 2\n                }\n                */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /* {\n                       \"foo\": 1,\n                       \"bar\": 2\n                   } */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * foo\n                 *\n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * foo\n                 * \n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * foo\n                 *\n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 * foo\n                 * \n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // foo\n                //\n                // bar\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                // foo\n                // \n                // bar\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                // foo\n                // \n                // bar\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /* foo\n\n                   bar */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /* foo\n                   \n                   bar */\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        ("/* foo\n\n   bar */", Some(serde_json::json!(["starred-block"]))),
        (
            "\n                /* foo\n                   \n                   bar */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        ("/*foo\n\n  bar */", Some(serde_json::json!(["starred-block"]))),
        ("/*foo\n   \n  bar */", Some(serde_json::json!(["starred-block"]))),
        ("/*\n // a line comment\n some.code();\n */", Some(serde_json::json!(["starred-block"]))),
        (
            "\n                /*\n                 // a line comment\n                 * some.code();\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                ////This comment is in\n                //`separate-lines` format.\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                // // This comment is in\n                // `separate-lines` format.\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        ("/*\n{\n\t\"foo\": 1,\n\t//\"bar\": 2\n}\n*/", Some(serde_json::json!(["starred-block"]))),
        (
            "// This is\n// a multiline comment\n// @ts-expect-error TS(2322) some message\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// This is\n// a multiline comment\n// @ts-expect-error: TS(2322) some message\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// Top A\n// Top B\n// @ts-ignore some message\n// Bottom A\n// Bottom B\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// prettier-ignorement\n// v8 ignored next\n// node:coverage ignored next\n// webpackChunkNameExtra: \"chunk\"",
            Some(serde_json::json!(["starred-block"])),
        ),
        // A blank-bodied block comment. ESLint writes the literal text `undefined`
        // here; oxlint leaves the line empty instead.
        ("/*\n\n*/", Some(serde_json::json!(["starred-block"]))),
    ];

    let fix = vec![
        (
            "\n                // these are\n                // line comments\n            ",
            "\n                /*\n                 * these are\n                 * line comments\n                 */\n            ",
            None,
        ),
        (
            "\n                // foo\n                // bar\n\n                // baz\n                // qux\n            ",
            "\n                /*\n                 * foo\n                 * bar\n                 */\n\n                /*\n                 * baz\n                 * qux\n                 */\n            ",
            None,
        ),
        (
            "\n                //  foo\n                // bar\n                //    baz\n                // qux\n            ",
            "\n                /*\n                 *  foo\n                 * bar\n                 *    baz\n                 * qux\n                 */\n            ",
            None,
        ),
        (
            "\n                //  foo\n                //\n                //    baz\n                // qux\n            ",
            "\n                /*\n                 *  foo\n                 * \n                 *    baz\n                 * qux\n                 */\n            ",
            None,
        ),
        (
            "\n                //    foo\n                     // bar\n           //  baz\n                // qux\n            ",
            "\n                /*\n                 *    foo\n                 * bar\n                 *  baz\n                 * qux\n                 */\n            ",
            None,
        ),
        (
            "\n                /* this block\n                 * is missing a newline at the start\n                 */\n            ",
            "\n                /*\n                 * this block\n                 * is missing a newline at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /** this JSDoc comment\n                 * is missing a newline at the start\n                 */\n            ",
            "\n                /**\n                 * this JSDoc comment\n                 * is missing a newline at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*! this Exclamation comment\n                 * is missing a newline\n                 * at the start\n                 */\n            ",
            "\n                /*!\n                 * this Exclamation comment\n                 * is missing a newline\n                 * at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*! this Exclamation comment is missing a newline at the start\n                 */\n            ",
            "\n                /*!\n                 * this Exclamation comment is missing a newline at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * this block\n                 * is missing a newline at the end*/\n            ",
            "\n                /*\n                 * this block\n                 * is missing a newline at the end\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                   is missing a '*' at the start\n                 */\n            ",
            "\n                /*\n                 * the following line\n                 * is missing a '*' at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                 is missing a '*' at the start\n                 */\n            ",
            "\n                /*\n                 * the following line\n                 * is missing a '*' at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                is missing a '*' at the start\n                 */\n            ",
            "\n                /*\n                 * the following line\n                 * is missing a '*' at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                      * has a '*' with the wrong offset at the start\n                 */\n            ",
            "\n                /*\n                 * the following line\n                 * has a '*' with the wrong offset at the start\n                 */\n            ",
            None,
        ),
        (
            "\n                  /*\n                   * the following line\n                 * has a '*' with the wrong offset at the start\n                   */\n            ",
            "\n                  /*\n                   * the following line\n                   * has a '*' with the wrong offset at the start\n                   */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the last line of this comment\n                 * is misaligned\n                   */\n            ",
            "\n                /*\n                 * the last line of this comment\n                 * is misaligned\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                *\n                 * is blank\n                 */\n            ",
            "\n                /*\n                 * the following line\n                 *\n                 * is blank\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the following line\n                  *\n                 * is blank\n                 */\n            ",
            "\n                /*\n                 * the following line\n                 *\n                 * is blank\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * the last line of this comment\n                 * is misaligned\n                   */ foo\n            ",
            "\n                /*\n                 * the last line of this comment\n                 * is misaligned\n                 */ foo\n            ",
            None,
        ),
        (
            "\n                /*\n                 * foo\n                 * bar\n                 */\n            ",
            "\n                // foo\n                // bar\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /**\n                 * JSDoc\n                 * Comment\n                 */\n            ",
            "\n                // JSDoc\n                // Comment\n            ",
            Some(serde_json::json!(["separate-lines",{"checkJSDoc":true}])),
        ),
        (
            "\n                /*!\n                 * Exclamation\n                 * Comment\n                 */\n            ",
            "\n                // Exclamation\n                // Comment\n            ",
            Some(serde_json::json!(["separate-lines",{"checkExclamation":true}])),
        ),
        (
            "\n                /* foo\n                 *bar\n                 baz\n                 qux*/\n            ",
            "\n                // foo\n                // bar\n                // baz\n                // qux\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                // foo\n                // bar\n            ",
            "\n                /* foo\n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // foo\n                //\n                // bar\n            ",
            "\n                /* foo\n                   \n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                //foo\n                //bar\n            ",
            "\n                /* foo\n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                //   foo\n                //   bar\n            ",
            "\n                /*   foo\n                     bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // foo\n              // bar\n            ",
            "\n                /* foo\n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                //    foo\n                     // bar\n           //  baz\n                // qux\n            ",
            "\n                /*    foo\n                   bar\n                    baz\n                   qux */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                * foo\n                * bar\n                */\n            ",
            "\n                /* foo\n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                *foo\n                *bar\n                */\n            ",
            "\n                /* foo\n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                *   foo\n                *   bar\n                */\n            ",
            "\n                /*   foo\n                     bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                * foo\n             * bar\n                */\n            ",
            "\n                /* foo\n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *    foo\n                 *  bar\n                 *   baz\n                 * qux\n                 */\n            ",
            "\n                /*    foo\n                    bar\n                     baz\n                   qux */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "/*\n{\n    \"foo\": 1,\n    \"bar\": 2\n}\n*/",
            "/*\n * {\n *     \"foo\": 1,\n *     \"bar\": 2\n * }\n */",
            None,
        ),
        (
            "\n                /*\n                {\n                \t\"foo\": 1,\n                \t\"bar\": 2\n                }\n                */\n            ",
            "\n                /*\n                 * {\n                 * \t\"foo\": 1,\n                 * \t\"bar\": 2\n                 * }\n                 */\n            ",
            None,
        ),
        (
            "/*\n{\n\t  \"foo\": 1,\n\t  \"bar\": 2\n}\n*/",
            "/*\n * {\n * \t  \"foo\": 1,\n * \t  \"bar\": 2\n * }\n */",
            None,
        ),
        (
            "\n                /*\n                {\n               \t\"foo\": 1,\n               \t\"bar\": 2\n                }\n                */\n            ",
            "\n                /*\n                 * {\n                 * \"foo\": 1,\n                 * \"bar\": 2\n                 * }\n                 */\n            ",
            None,
        ),
        (
            "\n                \t /*\n                      \t    {\n                  \t    \"foo\": 1,\n                \t   \"bar\": 2\n                }\n                */\n            ",
            "\n                \t /*\n                \t  * {\n                \t  * \"foo\": 1,\n                \t  * \"bar\": 2\n                \t  * }\n                \t  */\n            ",
            None,
        ),
        (
            "\n                //{\n                //    \"foo\": 1,\n                //    \"bar\": 2\n                //}\n            ",
            "\n                /*\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 */\n            ",
            None,
        ),
        (
            "\n                /*\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 */\n            ",
            "\n                // {\n                //     \"foo\": 1,\n                //     \"bar\": 2\n                // }\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 *\n                 */\n            ",
            "\n                // \n                // {\n                //     \"foo\": 1,\n                //     \"bar\": 2\n                // }\n                // \n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *\n                 * {\n                 *     \"foo\": 1,\n                 *     \"bar\": 2\n                 * }\n                 *\n                 */\n            ",
            "\n                /* \n                   {\n                       \"foo\": 1,\n                       \"bar\": 2\n                   }\n                    */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *\n                 *{\n                 *    \"foo\": 1,\n                 *    \"bar\": 2\n                 *}\n                 *\n                 */\n            ",
            "\n                /* \n                   {\n                       \"foo\": 1,\n                       \"bar\": 2\n                   }\n                    */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 *{\n                 *    \"foo\": 1,\n                 *    \"bar\": 2\n                 *}\n                 */\n            ",
            "\n                // {\n                //     \"foo\": 1,\n                //     \"bar\": 2\n                // }\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *   {\n                 *       \"foo\": 1,\n                 *       \"bar\": 2\n                 *   }\n                 */\n            ",
            "\n                //   {\n                //       \"foo\": 1,\n                //       \"bar\": 2\n                //   }\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n            *{\n                 *    \"foo\": 1,\n                    *    \"bar\": 2\n                 *}\n                  */\n            ",
            "\n                // {\n                //     \"foo\": 1,\n                //     \"bar\": 2\n                // }\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 *   {\n                 *       \"foo\": 1,\n                 *       \"bar\": 2\n           *}\n                 */\n            ",
            "\n                //    {\n                //        \"foo\": 1,\n                //        \"bar\": 2\n                // }\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                {\n                    \"foo\": 1,\n                    \"bar\": 2\n                }\n                */\n            ",
            "\n                // \n                // {\n                //     \"foo\": 1,\n                //     \"bar\": 2\n                // }\n                // \n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /* {\n                       \"foo\": 1,\n                       \"bar\": 2\n                   } */\n            ",
            "\n                // {\n                //     \"foo\": 1,\n                //     \"bar\": 2\n                // } \n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * foo\n                 *\n                 * bar\n                 */\n            ",
            "\n                // foo\n                // \n                // bar\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * foo\n                 * \n                 * bar\n                 */\n            ",
            "\n                // foo\n                // \n                // bar\n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /*\n                 * foo\n                 *\n                 * bar\n                 */\n            ",
            "\n                /* foo\n                   \n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /*\n                 * foo\n                 * \n                 * bar\n                 */\n            ",
            "\n                /* foo\n                   \n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                // foo\n                //\n                // bar\n            ",
            "\n                /*\n                 * foo\n                 * \n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                // foo\n                // \n                // bar\n            ",
            "\n                /*\n                 * foo\n                 * \n                 * bar\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                // foo\n                // \n                // bar\n            ",
            "\n                /* foo\n                   \n                   bar */\n            ",
            Some(serde_json::json!(["bare-block"])),
        ),
        (
            "\n                /* foo\n\n                   bar */\n            ",
            "\n                // foo\n                // \n                // bar \n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "\n                /* foo\n                   \n                   bar */\n            ",
            "\n                // foo\n                // \n                // bar \n            ",
            Some(serde_json::json!(["separate-lines"])),
        ),
        (
            "/* foo\n\n   bar */",
            "/*\n * foo\n * \n * bar \n */",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /* foo\n                   \n                   bar */\n            ",
            "\n                /*\n                 * foo\n                 * \n                 * bar \n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "/*foo\n\n  bar */",
            "/*\n * foo\n *\n * bar \n */",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "/*foo\n   \n  bar */",
            "/*\n * foo\n * \n * bar \n */",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "/*\n // a line comment\n some.code();\n */",
            "/*\n * // a line comment\n * some.code();\n */",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "\n                /*\n                 // a line comment\n                 * some.code();\n                 */\n            ",
            "\n                /*\n                 * // a line comment\n                 * some.code();\n                 */\n            ",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "/*\n{\n\t\"foo\": 1,\n\t//\"bar\": 2\n}\n*/",
            "/*\n * {\n * \t\"foo\": 1,\n * \t//\"bar\": 2\n * }\n */",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// This is\n// a multiline comment\n// @ts-expect-error TS(2322) some message\nfoo();",
            "/*\n * This is\n * a multiline comment\n */\n// @ts-expect-error TS(2322) some message\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// This is\n// a multiline comment\n// @ts-expect-error: TS(2322) some message\nfoo();",
            "/*\n * This is\n * a multiline comment\n */\n// @ts-expect-error: TS(2322) some message\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// Top A\n// Top B\n// @ts-ignore some message\n// Bottom A\n// Bottom B\nfoo();",
            "/*\n * Top A\n * Top B\n */\n// @ts-ignore some message\n/*\n * Bottom A\n * Bottom B\n */\nfoo();",
            Some(serde_json::json!(["starred-block"])),
        ),
        (
            "// prettier-ignorement\n// v8 ignored next\n// node:coverage ignored next\n// webpackChunkNameExtra: \"chunk\"",
            "/*\n * prettier-ignorement\n * v8 ignored next\n * node:coverage ignored next\n * webpackChunkNameExtra: \"chunk\"\n */",
            Some(serde_json::json!(["starred-block"])),
        ),
        ("/*\n\n*/", "/*\n *\n */", Some(serde_json::json!(["starred-block"]))),
    ];

    Tester::new(MultilineCommentStyle::NAME, MultilineCommentStyle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
