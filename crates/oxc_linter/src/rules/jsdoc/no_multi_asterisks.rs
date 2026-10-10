use std::ops::Range;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_multi_asterisks_diagnostic(span: Span, location: Location) -> OxcDiagnostic {
    let message = match location {
        Location::MiddleLine => "Should be no multiple asterisks on middle lines.",
        Location::EndLine => "Should be no multiple asterisks on end lines.",
    };
    OxcDiagnostic::warn(message).with_help("Remove the extra asterisks.").with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoMultiAsterisks {
    /// Whether to allow asterisks after a space, as with Markdown emphasis such as `*bold*`
    /// at the start of a line's text.
    ///
    /// Defaults to `false`.
    allow_whitespace: bool,
    /// Whether to report extra asterisks at the end of a comment, for example when it is closed
    /// with two asterisks and a slash.
    ///
    /// Defaults to `true`.
    prevent_at_end: bool,
    /// Whether to report extra asterisks at the start of the lines in the middle of a comment,
    /// for example a line that starts with `**`.
    ///
    /// Defaults to `true`.
    prevent_at_middle_lines: bool,
}

impl Default for NoMultiAsterisks {
    fn default() -> Self {
        Self { allow_whitespace: false, prevent_at_end: true, prevent_at_middle_lines: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents use of multiple asterisks at the beginning of lines in JSDoc comments,
    /// and at the end of the comment before the closing `*/`.
    ///
    /// ### Why is this bad?
    ///
    /// A line that starts with more than one asterisk, such as `**`, or a comment closed
    /// with `**/`, is usually a typo. The extra asterisks add noise and can be mistaken for
    /// Markdown emphasis.
    ///
    /// Only the first offending line of each comment is reported, so fixing one line can
    /// reveal the next.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /**
    ///  * Desc.
    ///  **
    ///  */
    ///
    /// /**
    ///  * Desc.
    ///  **/
    ///
    /// /** Desc. **/
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /**
    ///  * Desc.
    ///  *
    ///  */
    ///
    /// /** Desc. */
    ///
    /// /**
    ///  * **Bold**
    ///  */
    /// ```
    NoMultiAsterisks,
    jsdoc,
    style,
    fix,
    config = NoMultiAsterisks,
    version = "next",
    short_description = "Prevents use of multiple asterisks at the beginning of lines.",
);

impl Rule for NoMultiAsterisks {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let source_text = ctx.source_text();

        for jsdoc in ctx.jsdoc().iter_all() {
            // `jsdoc.span` is the content between `/**` and `*/`.
            let (Some(start), end) = (jsdoc.span.start.checked_sub(3), jsdoc.span.end + 2) else {
                continue;
            };
            let comment_start = start as usize;
            let Some(comment) = source_text.get(comment_start..end as usize) else {
                continue;
            };

            let Some(problem) = self.find_problem(comment) else {
                continue;
            };

            // Offsets are relative to the start of the comment.
            #[expect(clippy::cast_possible_truncation)]
            let to_span = |range: &Range<usize>| {
                Span::new((comment_start + range.start) as u32, (comment_start + range.end) as u32)
            };
            let span = to_span(&problem.remove);
            ctx.diagnostic_with_fix(
                no_multi_asterisks_diagnostic(span, problem.location),
                |fixer| fixer.delete_range(span),
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Location {
    MiddleLine,
    EndLine,
}

/// An offending line. All offsets are relative to the start of the comment.
#[derive(Debug, PartialEq, Eq)]
struct Problem {
    location: Location,
    /// The text to remove. Reported and fixed.
    remove: Range<usize>,
}

/// One line of a JSDoc comment, split the way `comment-parser` splits it:
///
/// ```text
/// <leading whitespace><delimiter><post-delimiter><description><end>
///        ·····         `/**` or `*`   whitespace                `*/`
/// ```
struct Line {
    /// `/**` on the first line, `*` on lines that start with an asterisk, otherwise empty.
    delimiter: Range<usize>,
    post_delimiter: Range<usize>,
    /// Everything after the post-delimiter, up to the closing `*/` if the line has one.
    description: Range<usize>,
    /// Whether the line ends with `*/`.
    has_end: bool,
}

impl Line {
    fn new(line: &str, line_start: usize, is_first: bool) -> Self {
        let mut pos = line.len() - line.trim_start().len();
        let mut delimiter = pos..pos;
        let mut post_delimiter = pos..pos;

        let rest = &line[pos..];
        if is_first && rest.starts_with("/**") && !rest.starts_with("/***") {
            delimiter = pos..pos + 3;
            pos += 3;
            let spaces = line[pos..].len() - line[pos..].trim_start().len();
            post_delimiter = pos..pos + spaces;
            pos += spaces;
        }

        let rest = &line[pos..];
        let has_end = rest.trim_end().ends_with("*/");

        if delimiter.is_empty() && rest.starts_with('*') && !rest.starts_with("*/") {
            delimiter = pos..pos + 1;
            pos += 1;
            let spaces = line[pos..].len() - line[pos..].trim_start().len();
            post_delimiter = pos..pos + spaces;
            pos += spaces;
        }

        let description_end = if has_end { line.trim_end().len() - 2 } else { line.len() };
        let description = pos..description_end.max(pos);

        let shift = |range: Range<usize>| range.start + line_start..range.end + line_start;
        Self {
            delimiter: shift(delimiter),
            post_delimiter: shift(post_delimiter),
            description: shift(description),
            has_end,
        }
    }
}

impl NoMultiAsterisks {
    /// Finds the first offending line in a comment, including its `/**` and `*/`.
    fn find_problem(&self, comment: &str) -> Option<Problem> {
        if !comment.starts_with("/**")
            || comment.starts_with("/***")
            || !comment.ends_with("*/")
            || comment.len() < 5
        {
            return None;
        }

        let mut line_start = 0;

        for (index, raw_line) in comment.split('\n').enumerate() {
            let start = line_start;
            line_start += raw_line.len() + 1;
            let text = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            let line = Line::new(text, start, index == 0);
            let description = &comment[line.description.clone()];

            if self.prevent_at_middle_lines && !line.has_end {
                let len = if self.allow_whitespace {
                    if line.post_delimiter.is_empty() { leading_asterisks(description) } else { 0 }
                } else {
                    leading_whitespace_and_single_asterisks(description)
                };
                if len > 0 {
                    return Some(Problem {
                        location: Location::MiddleLine,
                        remove: line.description.start..line.description.start + len,
                    });
                }
            }

            if self.prevent_at_end && line.has_end {
                let is_single_line = comment[line.delimiter.clone()] == *"/**";
                if let Some(remove) = self.end_asterisks(comment, &line, is_single_line) {
                    return Some(Problem { location: Location::EndLine, remove });
                }
            }
        }

        None
    }

    /// The extra asterisks before the closing `*/` of `line`, if any.
    ///
    /// The `*` of `*/` is the last asterisk of the run. Any asterisk before it, apart from the
    /// line's own leading delimiter, is an extra one.
    fn end_asterisks(
        &self,
        comment: &str,
        line: &Line,
        is_single_line: bool,
    ) -> Option<Range<usize>> {
        let description = &line.description;
        let text = &comment[description.clone()];

        // The text before the slash of `*/`, i.e. the description and the `*` of `*/`.
        let mut run = String::with_capacity(text.len() + 1);
        run.push_str(text);
        run.push('*');

        let is_allowed = |c: char| {
            if self.allow_whitespace { c == '*' } else { matches!(c, '*' | ' ' | '\t') }
        };
        let run_start = run
            .char_indices()
            .rev()
            .find(|&(_, c)| !is_allowed(c))
            .map_or(0, |(index, c)| index + c.len_utf8());
        let asterisks = run[run_start..].matches('*').count();

        if asterisks >= 2 {
            // Single-line comments keep the whitespace in front of the asterisks.
            let first = if is_single_line {
                run_start + run[run_start..].find('*').unwrap_or_default()
            } else {
                run_start
            };
            return Some(description.start + first..description.end);
        }

        // `**/` and `* */`: the line's own delimiter is the extra asterisk.
        let only_whitespace_before_end = text.trim().is_empty();
        if !is_single_line && !line.delimiter.is_empty() && only_whitespace_before_end {
            return Some(line.delimiter.clone());
        }

        None
    }
}

/// Length of the match of `^([\t ]|\*(?!\*))+`.
fn leading_whitespace_and_single_asterisks(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut len = 0;
    while let Some(&byte) = bytes.get(len) {
        match byte {
            b' ' | b'\t' => len += 1,
            b'*' if bytes.get(len + 1) != Some(&b'*') => len += 1,
            _ => break,
        }
    }
    len
}

/// Length of the match of `^\*+`.
fn leading_asterisks(text: &str) -> usize {
    text.bytes().take_while(|&byte| byte == b'*').count()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                  /**
                   *
                   * Desc. ***
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Desc. ***
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   * sth */
                  ",
            None,
        ),
        (
            "
                  /**
                   **
                   *
                   */
                  ",
            Some(serde_json::json!([ { "preventAtMiddleLines": false, }, ])),
        ),
        (
            "
                  /**
                   *
                   *
                   **/
                  ",
            Some(serde_json::json!([ { "preventAtEnd": false, }, ])),
        ),
        (
            "
                  /**
                   * With a desc.
                   * and ***
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * and ***
                   * With a desc.
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * With a desc.
                   * With a desc.
                   * Desc. */
                  ",
            None,
        ),
        (
            "
                  /**
                   * With a description
                   * @tag {SomeType} and a tag with details
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /** abc */
                  function foo() {
                      //
                  }
                  ",
            None,
        ),
        (
            "
                  /** foo */
                  function foo(): void {
                      //
                  }
                  ",
            None,
        ), // { "parser": typescriptEslintParser, },
        (
            "
                  /** @aTag abc */
                  function foo() {
                      //
                  }
                  ",
            None,
        ),
        (
            "
                  /**
                   * **Bold**
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Preserve user's bold statement when fixing.
                   *
                   * **Bold example:** Hi there.
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * The method does 2 things:
                   * * Thing 1
                   * * Thing 2
                   */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
        (
            "
                  /**
                   * This multi-line comment contains some
                   * *non-standard bold* syntax
                   */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
        (
            "
                  /** abc */
                  function foo() {
                      //
                  }
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
        // One asterisk before the closing `*/` is fine, even without a space.
        (
            "
                  /**
                   * Desc.*/
                  ",
            None,
        ),
        // The line's own leading asterisk is not an extra asterisk.
        (
            "
                  /**
                   * Desc. */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
    ];

    let fail = vec![
        (
            "
                  /**
                   *
                   **
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   *
                   **
                   */
                  ",
            Some(serde_json::json!([ { "preventAtMiddleLines": true, }, ])),
        ),
        (
            "
                  /**
                   *
                   **
                   */
                  ",
            Some(serde_json::json!([ { "preventAtEnd": false, }, ])),
        ),
        (
            "
                  /**
                   * With a description
                   * @tag {SomeType} and a tag with details
                   **
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   **
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   **/
                  ",
            None,
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   **/
                  ",
            Some(serde_json::json!([ { "preventAtEnd": true, }, ])),
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   abc * **/
                  ",
            Some(serde_json::json!([ { "preventAtEnd": true, }, ])),
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   **/
                  ",
            Some(serde_json::json!([ { "preventAtMiddleLines": false, }, ])),
        ),
        (
            "
                  /** Desc. **/
                  ",
            None,
        ),
        (
            "
                  /** @someTag name desc. **/
                  ",
            None,
        ),
        (
            "
                  /** abc * */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Preserve user's whitespace when fixing (though one may also
                   *   use an align rule)
                   *
                   * */
                  ",
            Some(serde_json::json!([ { "preventAtEnd": true, }, ])),
        ),
        (
            "
                  /**
                   * The method does 2 things:
                   * * Thing 1
                   * * Thing 2
                   */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": false, }, ])),
        ),
        (
            "
                  /**
                   * This multi-line comment contains some
                   * *non-standard bold* syntax
                   */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": false, }, ])),
        ),
        (
            "
                  /** Desc. **/
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
        (
            "
                  /**
                   * foo **/
                  ",
            None,
        ),
        (
            "
                  /** @tag **/
                  ",
            None,
        ),
        (
            "
                  /**
                   * @tag **/
                  ",
            None,
        ),
        (
            "
                  /**
                   * é **/
                  ",
            None,
        ),
        (
            "
                  /**\r\n * a\r\n **\r\n */
                  ",
            None,
        ),
        (
            "
                  /**
                   * */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
    ];

    let fix = vec![
        (
            "
                  /**
                   *
                   **
                   */
                  ",
            "
                  /**
                   *
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   *
                   **
                   */
                  ",
            "
                  /**
                   *
                   *
                   */
                  ",
            Some(serde_json::json!([ { "preventAtMiddleLines": true, }, ])),
        ),
        (
            "
                  /**
                   *
                   **
                   */
                  ",
            "
                  /**
                   *
                   *
                   */
                  ",
            Some(serde_json::json!([ { "preventAtEnd": false, }, ])),
        ),
        (
            "
                  /**
                   * With a description
                   * @tag {SomeType} and a tag with details
                   **
                   */
                  ",
            "
                  /**
                   * With a description
                   * @tag {SomeType} and a tag with details
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   **
                   *
                   */
                  ",
            "
                  /**
                   *
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   **/
                  ",
            "
                  /**
                   * Desc.
                   *
                   */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   **/
                  ",
            "
                  /**
                   * Desc.
                   *
                   */
                  ",
            Some(serde_json::json!([ { "preventAtEnd": true, }, ])),
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   abc * **/
                  ",
            "
                  /**
                   * Desc.
                   *
                   abc*/
                  ",
            Some(serde_json::json!([ { "preventAtEnd": true, }, ])),
        ),
        (
            "
                  /**
                   * Desc.
                   *
                   **/
                  ",
            "
                  /**
                   * Desc.
                   *
                   */
                  ",
            Some(serde_json::json!([ { "preventAtMiddleLines": false, }, ])),
        ),
        (
            "
                  /** Desc. **/
                  ",
            "
                  /** Desc. */
                  ",
            None,
        ),
        (
            "
                  /** @someTag name desc. **/
                  ",
            "
                  /** @someTag name desc. */
                  ",
            None,
        ),
        (
            "
                  /** abc * */
                  ",
            "
                  /** abc */
                  ",
            None,
        ),
        (
            "
                  /**
                   * Preserve user's whitespace when fixing (though one may also
                   *   use an align rule)
                   *
                   * */
                  ",
            "
                  /**
                   * Preserve user's whitespace when fixing (though one may also
                   *   use an align rule)
                   *
                    */
                  ",
            Some(serde_json::json!([ { "preventAtEnd": true, }, ])),
        ),
        (
            "
                  /**
                   * The method does 2 things:
                   * * Thing 1
                   * * Thing 2
                   */
                  ",
            "
                  /**
                   * The method does 2 things:
                   * Thing 1
                   * * Thing 2
                   */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": false, }, ])),
        ),
        (
            "
                  /**
                   * This multi-line comment contains some
                   * *non-standard bold* syntax
                   */
                  ",
            "
                  /**
                   * This multi-line comment contains some
                   * non-standard bold* syntax
                   */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": false, }, ])),
        ),
        (
            "
                  /** Desc. **/
                  ",
            "
                  /** Desc. */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
        // Only the extra asterisks are removed, the line's own `*` stays.
        (
            "
                  /**
                   * foo **/
                  ",
            "
                  /**
                   * foo*/
                  ",
            None,
        ),
        (
            "
                  /** @tag **/
                  ",
            "
                  /** @tag */
                  ",
            None,
        ),
        (
            "
                  /**
                   * @tag **/
                  ",
            "
                  /**
                   * @tag*/
                  ",
            None,
        ),
        (
            "
                  /**
                   * é **/
                  ",
            "
                  /**
                   * é*/
                  ",
            None,
        ),
        (
            "
                  /**\r\n * a\r\n **\r\n */
                  ",
            "
                  /**\r\n * a\r\n *\r\n */
                  ",
            None,
        ),
        (
            "
                  /**
                   * */
                  ",
            "
                  /**
                    */
                  ",
            Some(serde_json::json!([ { "allowWhitespace": true, }, ])),
        ),
    ];

    Tester::new(NoMultiAsterisks::NAME, NoMultiAsterisks::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
