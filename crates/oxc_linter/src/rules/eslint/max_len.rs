use oxc_ast::{ast::JSXExpression, AstKind, CommentKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

struct IgnoreOptions {
    pattern_re: Option<Regex>,
    urls: bool,
    strings: bool,
    template_literals: bool,
    reg_exp_literals: bool,
}

#[derive(Debug, Error, Diagnostic)]
#[error("This line has a length of {current_length:?}. Maximum allowed is {max:?}.")]
#[diagnostic(
    severity(warning),
    help("Consider breaking this line into multiple lines or shortening comments/codes where applicable")
)]
struct MaxLenDiagnostic {
    current_length: usize,
    max: usize,
    #[label]
    span: Span,
}

impl MaxLenDiagnostic {
    fn new(current_length: usize, max: usize, span: Span) -> Self {
        Self { current_length, max, span: Span::new(span.start, span.end) }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error(
    "This line has a comment length of {current_length:?}. Maximum allowed is {max_comment:?}."
)]
#[diagnostic(
    severity(warning),
    help("Consider breaking this line into multiple lines or shortening comments/codes where applicable")
)]
struct MaxCommentLenDiagnostic {
    current_length: usize,
    max_comment: usize,
    #[label]
    span: Span,
}

impl MaxCommentLenDiagnostic {
    fn new(current_length: usize, max_comment: usize, span: Span) -> Self {
        Self { current_length, max_comment, span: Span::new(span.start, span.end) }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MaxLen(Box<MaxLenConfig>);

#[derive(Debug, Clone)]
pub struct MaxLenConfig {
    code: usize,
    tab_width: usize,
    comments: usize,
    ignore_pattern: String,
    ignore_comments: bool,
    ignore_trailing_comments: bool,
    ignore_urls: bool,
    ignore_strings: bool,
    ignore_template_literals: bool,
    ignore_reg_exp_literals: bool,
}

impl std::ops::Deref for MaxLen {
    type Target = MaxLenConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxLenConfig {
    fn default() -> Self {
        Self {
            code: 80,     // the default max length
            tab_width: 4, // the default tab width
            comments: 0,
            ignore_pattern: String::new(),
            ignore_comments: false,
            ignore_trailing_comments: false,
            ignore_urls: false,
            ignore_strings: false,
            ignore_template_literals: false,
            ignore_reg_exp_literals: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce a maximum length of characters per line
    ///
    /// ### Why is this bad?
    /// Lines that are too long can be difficult to read, understand, and maintain.
    /// Excessively long lines can also result in horizontal scrolling which hinders readability.
    /// While there is no objective maximum length considered acceptable for a line,
    /// a commonly used standard is 80 characters per line.
    ///
    /// ### Example
    /// ```javascript
    /// // This line is fine
    /// const example = "This is a concise example.";
    ///
    /// // This line might be considered too long and difficult to read without wrapping or horizontal scrolling
    /// const tooLongExample = "This line is an example of a line that stretches far beyond the conventional length and could be hard to read.";
    /// ```
    MaxLen,
    style
);

// Define a structure to store the spans.
struct LiteralSpans {
    strings: Vec<Span>,
    template_literals: Vec<Span>,
    reg_exp_literals: Vec<Span>,
    jsx_empty_spans: Vec<Span>,
}

impl LiteralSpans {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            template_literals: Vec::new(),
            reg_exp_literals: Vec::new(),
            jsx_empty_spans: Vec::new(),
        }
    }

    fn collect_from_ctx(&mut self, ctx: &LintContext) {
        for node in ctx.semantic().nodes().iter() {
            match node.kind() {
                AstKind::StringLiteral(node) => self.strings.push(node.span),
                AstKind::TemplateLiteral(node) => self.template_literals.push(node.span),
                AstKind::RegExpLiteral(node) => self.reg_exp_literals.push(node.span),
                AstKind::JSXExpressionContainer(node) => {
                    if let JSXExpression::EmptyExpression(_) = node.expression {
                        self.jsx_empty_spans.push(node.span);
                    }
                }
                _ => {}
            }
        }
    }
}

// Find the line index of a given index
fn find_line_index(index: usize, line_starts: &[usize]) -> usize {
    line_starts.binary_search(&index).unwrap_or_else(|x| x - 1)
}

fn is_trailing_comment(
    line_index: usize,
    line_span: Span,
    comment: &(CommentKind, Span),
    line_starts: &[usize],
) -> bool {
    let (_, span) = comment;
    let comment_start_line = find_line_index(span.start as usize, line_starts);
    let comment_end_line = find_line_index(span.end as usize, line_starts);
    let mut is_trailing = false;

    if (comment_start_line == line_index && line_index <= comment_end_line)
        && (line_span.end == span.end || comment_end_line > line_index)
    {
        is_trailing = true;
    }

    is_trailing
}

fn is_full_line_comment(
    line_index: usize,
    line_span: Span,
    line_text: &str,
    comment: &(CommentKind, Span),
    line_starts: &[usize],
) -> bool {
    let (_, span) = comment;
    let comment_start_line = find_line_index(span.start as usize, line_starts);
    let comment_end_line = find_line_index(span.end as usize, line_starts);
    let mut is_full_comment = false;
    let mut is_first_token_on_first_line = false;

    if span.start >= line_span.start && comment_start_line == line_index {
        let diff = (span.start - line_span.start) as usize;
        if line_text.len() >= diff {
            is_first_token_on_first_line = line_text[..diff].trim().is_empty();
        }
    }

    if (is_first_token_on_first_line || comment_start_line < line_index)
        && (comment_end_line > line_index
            || (line_span.end == span.end && comment_end_line == line_index))
    {
        is_full_comment = true;
    }

    is_full_comment
}

fn generate_updated_comments(
    comments: &[(CommentKind, Span)],
    jsx_empty_spans: &[Span],
    line_starts: &[usize],
) -> Vec<(CommentKind, Span)> {
    let mut updated_comments: Vec<(CommentKind, Span)> = Vec::new();
    let mut jsx_index = 0; // the point of jsx_empty_spans
    let mut old_comments_index = 0; // the point of comments
    let comment_length: u32 = 2; // the len of "//" "/*" "/*"

    // Use two pointers to traverse comments and JSX empty nodes to generate new comment nodes.
    while old_comments_index < comments.len() && jsx_index < jsx_empty_spans.len() {
        let (kind, span) = &comments[old_comments_index];

        let jsx_node = jsx_empty_spans[jsx_index];
        let jsx_node_first_line = find_line_index(jsx_node.start as usize, line_starts);
        let jsx_node_last_line = find_line_index(jsx_node.end as usize, line_starts);

        // If the current comment is included by the jsx_empty_span.
        if span.start >= jsx_node.start
            && span.end <= jsx_node.end
            && jsx_node_first_line == jsx_node_last_line
        {
            // If this is the first comment contained by the current jsx_empty_span, then add the jsx_empty_span to updated_comments.
            if updated_comments.last().map_or(true, |last| last.1 != jsx_node) {
                updated_comments.push((*kind, jsx_node));
            }

            old_comments_index += 1; // move to next comment
        } else if span.start > jsx_node.end {
            // If the start position of the current comment is after the current jsx_empty_span, move to the next jsx_empty_span.
            jsx_index += 1;
        } else {
            let new_span = match kind {
                CommentKind::SingleLine => {
                    // If the current comment is not in any jsx_empty_span, add it directly to the result.
                    // add the length of the comment to the start of the comment //
                    Span::new(span.start - comment_length, span.end)
                }
                CommentKind::MultiLine => {
                    // If the current comment is not in any jsx_empty_span, add it directly to the result.
                    // add the length of the comment to the start of the comment /* and */
                    Span::new(span.start - comment_length, span.end + comment_length)
                }
            };

            updated_comments.push((*kind, new_span));
            old_comments_index += 1; // move to next comment
        }
    }

    // After the traversal is complete, if there are remaining comments and they are not in any jsx_empty_span
    // they also need to be added to the result.
    for comment in comments.iter().skip(old_comments_index) {
        let (kind, span) = &comment;

        let new_span = match kind {
            CommentKind::SingleLine => {
                // If the current comment is not in any jsx_empty_span, add it directly to the result.
                // add the length of the comment to the start of the comment //
                Span::new(span.start - comment_length, span.end)
            }
            CommentKind::MultiLine => {
                // If the current comment is not in any jsx_empty_span, add it directly to the result.
                // add the length of the comment to the start of the comment /* and */
                Span::new(span.start - comment_length, span.end + comment_length)
            }
        };

        updated_comments.push((*kind, new_span));
    }

    updated_comments
}

impl MaxLen {
    fn compute_line_length(line: &str, tab_width: usize) -> usize {
        let mut extra_character_count: usize = 0;
        let mut last_index: usize = 0; // Record the position after the previous segment

        for segment in line.split('\t').enumerate() {
            let (segment_index, segment) = segment;
            let segment_length = segment.chars().count();

            // Only add to last_index if not on the first segment (since split will not find a tab at the start of the string)
            if segment_index > 0 {
                let total_offset = last_index + extra_character_count;
                let previous_tab_stop_offset =
                    if tab_width != 0 { total_offset % tab_width } else { 0 };

                let space_count = tab_width - previous_tab_stop_offset;
                extra_character_count += space_count; // Add the additional space count due to a tab
            }

            // Update last_index to the current position after adding the segment length
            last_index += segment_length;
        }

        // Calculate the full length of the line including extra spaces added due to tabs
        last_index + extra_character_count
    }

    fn check_is_in_line(strings_or_literals: &[Span], line_span: Span) -> bool {
        strings_or_literals.iter().any(|span| {
            (span.start >= line_span.start && line_span.end >= span.start) // in start
                || (span.end >= line_span.start && line_span.end >= span.end) // in end
                || (span.start <= line_span.start && line_span.end <= span.end) // in middle or single line
        })
    }

    fn should_ignore_line(
        text_to_measure: &str,
        literal_spans: &LiteralSpans,
        line_span: Span,
        ignore_options: &IgnoreOptions,
    ) -> bool {
        if let Some(pattern_re) = ignore_options.pattern_re.as_ref() {
            if pattern_re.is_match(text_to_measure) {
                return true;
            }
        }

        if ignore_options.urls
            && (text_to_measure.contains("http://") || text_to_measure.contains("https://"))
        {
            return true;
        }

        if ignore_options.strings {
            return Self::check_is_in_line(&literal_spans.strings, line_span);
        }

        if ignore_options.template_literals {
            return Self::check_is_in_line(&literal_spans.template_literals, line_span);
        }

        if ignore_options.reg_exp_literals {
            return Self::check_is_in_line(&literal_spans.reg_exp_literals, line_span);
        }

        false
    }
}

impl Rule for MaxLen {
    fn from_configuration(value: serde_json::Value) -> Self {
        // support [{ "code": 80, "tabWidth": 4 }] == [80, 4]
        // support [{ "code": 80, "tabWidth": 4, "ignoreComments": true }] == [80, 4, { "ignoreComments": true }]
        let param1 = value.get(0);
        let param2 = value.get(1);
        let param3 = value.get(2);
        let mut config = value.get(1);

        config = match param3 {
            Some(Value::Object(_)) => param3,
            _ => config,
        };

        let default_value = match param1 {
            Some(Value::Number(n)) if n.is_u64() => {
                usize::try_from(n.as_u64().unwrap_or_default()).unwrap_or(80)
            }
            _ => 80,
        };

        let code = config
            .and_then(|config| config.get("code"))
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(default_value, |v| usize::try_from(v).unwrap_or(default_value));

        let default_value = match param2 {
            Some(Value::Number(n)) if n.is_u64() => {
                usize::try_from(n.as_u64().unwrap_or_default()).unwrap_or(4)
            }
            _ => 4,
        };

        let tab_width = config
            .and_then(|config| config.get("tabWidth"))
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(default_value, |v| usize::try_from(v).unwrap_or(default_value));

        let ignore_comments = config
            .and_then(|config| config.get("ignoreComments"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let ignore_strings = config
            .and_then(|config| config.get("ignoreStrings"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let ignore_template_literals = config
            .and_then(|config| config.get("ignoreTemplateLiterals"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let ignore_reg_exp_literals = config
            .and_then(|config| config.get("ignoreRegExpLiterals"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let ignore_trailing_comments = config
            .and_then(|config| config.get("ignoreTrailingComments"))
            .and_then(Value::as_bool)
            .unwrap_or(ignore_comments);

        let ignore_urls = config
            .and_then(|config| config.get("ignoreUrls"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let ignore_pattern = config
            .and_then(|config| config.get("ignorePattern"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let comments = config
            .and_then(|config| config.get("comments"))
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(0, |v| usize::try_from(v).unwrap_or(0));

        Self(Box::new(MaxLenConfig {
            code,
            tab_width,
            comments,
            ignore_pattern,
            ignore_comments,
            ignore_trailing_comments,
            ignore_urls,
            ignore_strings,
            ignore_template_literals,
            ignore_reg_exp_literals,
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let full_text = ctx.source_text();

        // per line start index, used to find the line number of a given index, used in fn is_comment()
        let mut line_starts: Vec<usize> = vec![0];
        for (i, c) in full_text.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        let mut literal_spans = LiteralSpans::new();
        literal_spans.collect_from_ctx(ctx);

        let updated_comments = generate_updated_comments(
            &ctx.semantic().trivias().comments().collect::<Vec<_>>(),
            &literal_spans.jsx_empty_spans,
            &line_starts,
        );

        let pattern_re = if self.ignore_pattern.is_empty() {
            None
        } else {
            Regex::new(&self.ignore_pattern).ok()
        };

        let mut line_start_index: u32 = 0;
        let max_comment_length = self.comments;
        let mut cur_comment_index = 0;
        let comments_len = updated_comments.len();
        for (line_index, line_text) in full_text.lines().enumerate() {
            let mut text_to_measure = line_text;
            // Convert the length of line_text to u32 safely, handling any error.
            let line_end_index = if let Ok(length) = u32::try_from(line_text.len()) {
                line_start_index.saturating_add(length)
            } else {
                eprintln!("Length of line text is too large for u32");
                break;
            };

            let line_span = Span::new(line_start_index, line_end_index);
            let mut line_is_comment = false;
            if cur_comment_index < comments_len {
                let mut comment;
                while cur_comment_index < comments_len {
                    comment = &updated_comments[cur_comment_index];
                    if find_line_index(comment.1.start as usize, &line_starts) <= line_index {
                        cur_comment_index += 1;
                    } else {
                        break;
                    }
                }

                cur_comment_index = cur_comment_index.saturating_sub(1);

                comment = &updated_comments[cur_comment_index];

                line_is_comment =
                    is_full_line_comment(line_index, line_span, line_text, comment, &line_starts);

                if line_is_comment && self.ignore_comments {
                    line_start_index = line_end_index + 1;
                    continue;
                }

                let mut last_index = cur_comment_index;
                let mut is_trailing = !line_is_comment
                    && is_trailing_comment(line_index, line_span, comment, &line_starts);

                while (self.ignore_comments || self.ignore_trailing_comments) && is_trailing {
                    let (_, span) = comment;
                    text_to_measure =
                        text_to_measure[..(span.start - line_span.start) as usize].trim_end();

                    // last_index - 1 can't be less than 0
                    if last_index == 0 {
                        break;
                    }

                    last_index -= 1;
                    comment = &updated_comments[last_index];

                    let text_len = u32::try_from(text_to_measure.len()).unwrap_or(0);
                    let new_span = Span::new(line_span.start, line_span.start + text_len);

                    is_trailing = is_trailing_comment(line_index, new_span, comment, &line_starts);
                }
            }

            // ignore strings and literals if set true
            if Self::should_ignore_line(
                text_to_measure,
                &literal_spans,
                line_span,
                &IgnoreOptions {
                    pattern_re: pattern_re.clone(),
                    urls: self.ignore_urls,
                    strings: self.ignore_strings,
                    template_literals: self.ignore_template_literals,
                    reg_exp_literals: self.ignore_reg_exp_literals,
                },
            ) {
                line_start_index = line_end_index + 1;
                continue;
            }

            let actual_code_length = if self.tab_width == 0 {
                text_to_measure.len()
            } else {
                Self::compute_line_length(text_to_measure, self.tab_width)
            };

            let comment_length_applies = line_is_comment && max_comment_length > 0;
            if comment_length_applies {
                if actual_code_length > max_comment_length {
                    let error = MaxCommentLenDiagnostic::new(
                        actual_code_length,
                        max_comment_length,
                        line_span,
                    );

                    ctx.diagnostic(error);
                }
            } else if actual_code_length > self.code {
                let error = MaxLenDiagnostic::new(actual_code_length, self.code, line_span);
                ctx.diagnostic(error);
            }

            line_start_index = line_end_index.saturating_add(1); // move to the start of next line
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("var x = 5;\nvar x = 2;", None),
        ("var x = 5;\nvar x = 2;", Some(json!([{ "code": 80, "tabWidth": 4 }]))),
        (
            "\t\t\tvar i = 1;\n\t\t\tvar j = 1;",
            Some(json!([{ "code": 15, "tabWidth": 1 }])),
        ),
        ("var one\t\t= 1;\nvar three\t= 3;", Some(json!([{ "code": 16, "tabWidth": 4 }]))),
        (
            "\tvar one\t\t= 1;\n\tvar three\t= 3;",
            Some(json!([{ "code": 20, "tabWidth": 4 }])),
        ),
        ("var i = 1;\r\nvar i = 1;\n", Some(json!([{ "code": 10, "tabWidth": 4 }]))),
        (
            "\n// Blank line on top\nvar foo = module.exports = {};\n",
            Some(json!([{ "code": 80, "tabWidth": 4 }])),
        ),
        ("\n// Blank line on top\nvar foo = module.exports = {};\n", None),
        (
            "var foo = module.exports = {}; // really long trailing comment",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreComments": true }])),
        ),
        (
            "foo(); \t// strips entire comment *and* trailing whitespace",
            Some(json!([{ "code": 6, "tabWidth": 4, "ignoreComments": true }])),
        ),
        (
            "// really long comment on its own line sitting here",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreComments": true }])),
        ),
        (
            "var foo = module.exports = {}; /* inline some other comments */ //more",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreComments": true }])),
        ),
        ("var /*inline-comment*/ i = 1;", None),
        (
            "var /*inline-comment*/ i = 1; // with really long trailing comment",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreComments": true }])),
        ),
        (
            "foo('http://example.com/this/is/?a=longish&url=in#here');",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreUrls": true }])),
        ),
        (
            "foo(bar(bazz('this is a long'), 'line of'), 'stuff');",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignorePattern": "foo.+bazz\\(" }])),
        ),
        (
            "/* hey there! this is a multiline\n   comment with longish lines in various places\n   but\n   with a short line-length */",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreComments": true }])),
        ),
        (
            "// I like short comments\nfunction butLongSourceLines() { weird(eh()) }",
            Some(json!([{ "code": 80, "tabWidth": 4, "comments": 30 }])),
        ),
        (
            "// I like longer comments and shorter code\nfunction see() { odd(eh()) }",
            Some(json!([{ "code": 30, "tabWidth": 4, "comments": 80 }])),
        ),
        (
            "// Full line comment\nsomeCode(); // With a long trailing comment.",
            Some(
                json!([{ "code": 30, "tabWidth": 4, "comments": 20, "ignoreTrailingComments": true }]),
            ),
        ),
        (
            "var foo = module.exports = {}; // really long trailing comment",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreTrailingComments": true }])),
        ),
        (
            "var foo = module.exports = {}; /* inline some other comments */ //more",
            Some(json!([{ "code": 40, "tabWidth": 4, "ignoreTrailingComments": true }])),
        ),
        (
            "var foo = module.exports = {}; // really long trailing comment",
            Some(
                json!([{ "code": 40, "tabWidth": 4, "ignoreComments": true, "ignoreTrailingComments": false }]),
            ),
        ),

        // ignoreStrings, ignoreTemplateLiterals and ignoreRegExpLiterals options
        (
            "var foo = veryLongIdentifier;\nvar bar = 'this is a very long string';",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreStrings": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = \"this is a very long string\";",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreStrings": true }])),
        ),
        (
            "var str = \"this is a very long string\\\nwith continuation\";",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreStrings": true }])),
        ),
        (
            "var str = \"this is a very long string\\\nwith continuation\\\nand with another very very long continuation\\\nand ending\";",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreStrings": true }])),
        ),
        (
            "var foo = <div className=\"this is a very long string\"></div>;",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreStrings": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = `this is a very long string`;",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreTemplateLiterals": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = `this is a very long string\nand this is another line that is very long`;",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreTemplateLiterals": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = `this is a very long string\nand this is another line that is very long\nand here is another\n and another!`;",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreTemplateLiterals": true }])),
        ),
        (
            "var foo = /this is a very long pattern/;",
            Some(json!([{ "code": 29, "tabWidth": 4, "ignoreRegExpLiterals": true }])),
        ),

        // check indented comment lines
        (
            "function foo() {\n//this line has 29 characters\n}",
            Some(json!([{ "code": 40, "tabWidth": 4, "comments": 29 }])),
        ),
        (
            "function foo() {\n    //this line has 33 characters\n}",
            Some(json!([{ "code": 40, "tabWidth": 4, "comments": 33 }])),
        ),
        (
            "function foo() {\n/*this line has 29 characters\nand this one has 21*/\n}",
            Some(json!([{ "code": 40, "tabWidth": 4, "comments": 29 }])),
        ),
        (
            "function foo() {\n    /*this line has 33 characters\n   and this one has 25*/\n}",
            Some(json!([{ "code": 40, "tabWidth": 4, "comments": 33 }])),
        ),
        (
            "function foo() {\n    var a; /*this line has 40 characters\n    and this one has 36 characters*/\n}",
            Some(json!([{ "code": 40, "tabWidth": 4, "comments": 36 }])),
        ),
        (
            "function foo() {\n    /*this line has 33 characters\n    and this one has 43 characters*/ var a;\n}",
            Some(json!([{ "code": 43, "tabWidth": 4, "comments": 33 }])),
        ),

        // blank line
        ("", None),

        // Multi-code-point unicode glyphs
        ("'🙂😀😆😎😊😜😉👍'", Some(json!([10]))),

        // Astral symbols in pattern (only matched by unicode regexes)
        (
            "var longNameLongName = '𝌆𝌆'",
            Some(json!([{ "code": 5, "ignorePattern": "𝌆{2}" }])),
        ),
        ("\tfoo", Some(json!([4, 0]))),

        // jsx
        (
            "var jsx = (<>\n  { /* this line has 38 characters */}\n</>)",
            Some(json!([15, { "comments": 38 }])),
        ),
        (
            "var jsx = (<>\n\t\t{ /* this line has 40 characters */}\n</>)",
            Some(json!([15, 4, { "comments": 44 }])),
        ),
        (
            "var jsx = (<>\n  <> text </>{ /* this line has 49 characters */}\n</>)",
            Some(json!([13, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line has 44 characters */}\n</>)",
            Some(json!([44, { "comments": 37 }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line has 44 characters */}\n</>)",
            Some(json!([37, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = <Foo\n         attr = {a && b/* this line has 57 characters */}\n></Foo>;",
            Some(json!([57])),
        ),
        (
            "var jsx = <Foo\n         attr = {/* this line has 57 characters */a && b}\n></Foo>;",
            Some(json!([57])),
        ),
        (
            "var jsx = <Foo\n         attr = \n          {a & b/* this line has 50 characters */}\n></Foo>;",
            Some(json!([50])),
        ),
        (
            "var jsx = (<>\n  <> </> {/* this line with two separate comments */} {/* have 80 characters */}\n</>)",
            Some(json!([80])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have 80 characters */}\n</>)",
            Some(json!([37, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have 80 characters */}\n</>)",
            Some(json!([37, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have > 80 characters */ /* another comment in same braces */}\n</>)",
            Some(json!([37, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have > 80 characters */ /* another comment in same braces */}\n</>)",
            Some(json!([37, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/*\n       this line has 34 characters\n   */}\n</>)",
            Some(json!([33, { "comments": 34 }])),
        ),
        (
            "var jsx = (<>\n  {/*\n       this line has 34 characters\n   */}\n</>)",
            Some(json!([33, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n  {a & b /* this line has 34 characters\n   */}\n</>)",
            Some(json!([33, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n  {a & b /* this line has 34 characters\n   */}\n</>)",
            Some(json!([33, { "ignoreComments": true }])),
        ),
    ];

    let fail = vec![
        ("\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\tvar i = 1;", Some(json!([80]))),
        ("var x = 5, y = 2, z = 5;", Some(json!([10, 4]))),
        ("\t\t\tvar i = 1;", Some(json!([15, 4]))),
        ("\t\t\tvar i = 1;\n\t\t\tvar j = 1;", Some(json!([15, 4]))),
        (
            "var /*this is a long non-removed inline comment*/ i = 1;",
            Some(json!([20, 4, { "ignoreComments": true }])),
        ),
        ("var foobar = 'this line isn\\'t matched by the regexp';\nvar fizzbuzz = 'but this one is matched by the regexp';\n", Some(json!([20, 4, { "ignorePattern": "fizzbuzz" }]))),
        (
            "var longLine = 'will trigger'; // even with a comment",
            Some(json!([10, 4, { "ignoreComments": true }])),
        ),
        (
            "var foo = module.exports = {}; // really long trailing comment",
            Some(json!([40, 4])),
        ),
        (
            "foo('http://example.com/this/is/?a=longish&url=in#here');",
            Some(json!([40, 4])),
        ),
        (
            "foo(bar(bazz('this is a long'), 'line of'), 'stuff');",
            Some(json!([40, 4])),
        ),
        (
            "// A comment that exceeds the max comment length.",
            Some(json!([80, 4, { "comments": 20 }])),
        ),
        (
            "// A comment that exceeds the max comment length and the max code length, but will fail for being too long of a comment",
            Some(json!([40, 4, { "comments": 80 }])),
        ),
        ("// A comment that exceeds the max comment length.", Some(json!([20]))),
        (
            "//This is very long comment with more than 40 characters which is invalid",
            Some(json!([40, 4, { "ignoreTrailingComments": true }])),
        ),

        // check indented comment lines
        (
            "function foo() {\n//this line has 29 characters\n}",
            Some(json!([40, 4, { "comments": 28 }])),
        ),
        (
            "function foo() {\n    //this line has 33 characters\n}",
            Some(json!([40, 4, { "comments": 32 }])),
        ),
        (
            "function foo() {\n/*this line has 29 characters\nand this one has 32 characters*/\n}",
            Some(json!([40, 4, { "comments": 28 }])),
        ),
        (
            "function foo() {\n    /*this line has 33 characters\n    and this one has 36 characters*/\n}",
            Some(json!([40, 4, { "comments": 32 }])),
        ),
        (
            "function foo() {\n    var a; /*this line has 40 characters\n    and this one has 36 characters*/\n}",
            Some(json!([39, 4, { "comments": 35 }])),
        ),
        (
            "function foo() {\n    /*this line has 33 characters\n    and this one has 43 characters*/ var a;\n}",
            Some(json!([42, 4, { "comments": 32 }])),
        ),

        // check comments with the same length as non-comments
        (
            "// This commented line has precisely 51 characters.\nvar x = 'This line also has exactly 51 characters';",
            Some(json!([20, { "ignoreComments": true }])),
        ),

        // ignoreStrings and ignoreTemplateLiterals options
        (
            "var foo = veryLongIdentifier;\nvar bar = 'this is a very long string';",
            Some(json!([29, { "ignoreStrings": false, "ignoreTemplateLiterals": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = /this is a very very long pattern/;",
            Some(json!([29, { "ignoreStrings": false, "ignoreRegExpLiterals": false }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = new RegExp('this is a very very long pattern');",
            Some(json!([29, { "ignoreStrings": false, "ignoreTemplateLiterals": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = \"this is a very long string\";",
            Some(json!([29, { "ignoreStrings": false, "ignoreTemplateLiterals": true }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = `this is a very long string`;",
            Some(json!([29, { "ignoreStrings": false, "ignoreTemplateLiterals": false }])),
        ),
        (
            "var foo = veryLongIdentifier;\nvar bar = `this is a very long string\nand this is another line that is very long`;",
            Some(json!([29, { "ignoreStrings": false, "ignoreTemplateLiterals": false }])),
        ),
        (
            "var foo = <div>this is a very very very long string</div>;",
            Some(json!([29, 4, { "ignoreStrings": true }])),
        ),

        // Multi-code-point unicode glyphs
        (
            "'🙁😁😟☹️😣😖😩😱👎'",
            Some(json!([10])),
        ),
        (
            "a",
            Some(json!([0])),
        ),

        // jsx
        (
            "var jsx = (<>\n  { /* this line has 38 characters */}\n</>)",
            Some(json!([15, { "comments": 37 }])),
        ),
        (
            "var jsx = (<>\n\t\t{ /* this line has 40 characters */}\n</>)",
            Some(json!([15, 4, { "comments": 40 }])),
        ),
        (
            "var jsx = (<>\n{ 38/* this line has 38 characters */}\n</>)",
            Some(json!([15, { "comments": 38 }])),
        ),
        (
            "var jsx = (<>\n{ 38/* this line has 38 characters */}\n</>)",
            Some(json!([37, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n{ 38/* this line has 38 characters */}\n</>)",
            Some(json!([37, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n   <> 50 </>{ 50/* this line has 50 characters */}\n</>)",
            Some(json!([49, { "comments": 100 }])),
        ),
        (
            "var jsx = (<>\n         {/* this line has 44 characters */}\n  <> </> {/* this line has 44 characters */}\n</>)",
            Some(json!([37, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = <Foo\n         attr = {a && b/* this line has 57 characters */}\n></Foo>;",
            Some(json!([56])),
        ),
        (
            "var jsx = <Foo\n         attr = {/* this line has 57 characters */a && b}\n></Foo>;",
            Some(json!([56])),
        ),
        (
            "var jsx = <Foo\n         attr = {a & b/* this line has 56 characters */}\n></Foo>;",
            Some(json!([55, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = <Foo\n         attr = \n          {a & b /* this line has 51 characters */}\n></Foo>;",
            Some(json!([30, { "comments": 44 }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have 80 characters */}\n</>)",
            Some(json!([79])),
        ),
        (
            "var jsx = (<>\n  <> </> {/* this line with two separate comments */} {/* have 87 characters */} <> </>\n</>)",
            Some(json!([85, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have 87 characters */} <> </>\n</>)",
            Some(json!([37, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this line with two separate comments */} {/* have > 80 characters */ /* another comment in same braces */}\n</>)",
            Some(json!([37])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this is not treated as a comment */ a & b} {/* trailing */ /* comments */}\n</>)",
            Some(json!([37, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n  {/* this line has 37 characters */}\n  <> </> {/* this is not treated as a comment */ a & b} {/* trailing */ /* comments */}\n</>)",
            Some(json!([37, { "ignoreComments": true }])),
        ),
        (
            "var jsx = (<>\n12345678901234{/*\n*/}\n</>)",
            Some(json!([14, { "ignoreTrailingComments": true }])),
        ),
        (
            "var jsx = (<>\n{/*\nthis line has 31 characters */}\n</>)",
            Some(json!([30, { "comments": 100 }])),
        ),
    ];

    Tester::new(MaxLen::NAME, pass, fail).test_and_snapshot();
}
