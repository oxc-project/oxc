use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn max_lines_diagnostic(count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("File has too many lines ({count})."))
        .with_help(format!("Maximum allowed is {max}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MaxLines(Box<MaxLinesConfig>);

#[derive(Debug, Clone)]
pub struct MaxLinesConfig {
    max: usize,
    skip_blank_lines: bool,
    skip_comments: bool,
}

impl std::ops::Deref for MaxLines {
    type Target = MaxLinesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxLinesConfig {
    fn default() -> Self {
        Self { max: 300, skip_blank_lines: false, skip_comments: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce a maximum number of lines per file.
    ///
    /// ### Why is this bad?
    ///
    /// Some people consider large files a code smell. Large files tend to do a
    /// lot of things and can make it hard following what’s going.  While there
    /// is not an objective maximum number of lines considered acceptable in a
    /// file, most people would agree it should not be in the thousands.
    /// Recommendations usually range from 100 to 500 lines.
    MaxLines,
    pedantic
);

impl Rule for MaxLines {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Self(Box::new(MaxLinesConfig { max, skip_comments: false, skip_blank_lines: false }))
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(300, |v| usize::try_from(v).unwrap_or(300));
            let skip_comments = config
                .and_then(|config| config.get("skipComments"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let skip_blank_lines = config
                .and_then(|config| config.get("skipBlankLines"))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            Self(Box::new(MaxLinesConfig { max, skip_blank_lines, skip_comments }))
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn run_once(&self, ctx: &LintContext) {
        let comment_lines = if self.skip_comments {
            let mut comment_lines: usize = 0;
            for comment in ctx.semantic().comments() {
                if comment.is_line() {
                    let comment_line = ctx.source_text()[..comment.span.start as usize]
                        .lines()
                        .next_back()
                        .unwrap_or("");
                    if line_has_just_comment(comment_line, "//") {
                        comment_lines += 1;
                    }
                } else {
                    let mut start_line =
                        ctx.source_text()[..comment.span.start as usize].lines().count();
                    let comment_start_line = ctx.source_text()[..comment.span.start as usize]
                        .lines()
                        .next_back()
                        .unwrap_or("");
                    if !line_has_just_comment(comment_start_line, "/*") {
                        start_line += 1;
                    }
                    let mut end_line =
                        ctx.source_text()[..=comment.span.end as usize].lines().count();
                    let comment_end_line =
                        ctx.source_text()[comment.span.end as usize..].lines().next().unwrap_or("");
                    if line_has_just_comment(comment_end_line, "*/") {
                        end_line += 1;
                    }
                    comment_lines += end_line - start_line;
                }
            }
            comment_lines
        } else {
            0
        };

        let lines_in_file =
            if ctx.source_text().is_empty() { 1 } else { ctx.source_text().lines().count() };

        let blank_lines = if self.skip_blank_lines {
            ctx.source_text().lines().filter(|&line| line.trim().is_empty()).count()
        } else {
            0
        };

        if lines_in_file.saturating_sub(blank_lines).saturating_sub(comment_lines) > self.max {
            // Point to end of the file for `eslint-disable max-lines` to work.
            let end = ctx.source_text().len().saturating_sub(1) as u32;
            ctx.diagnostic(max_lines_diagnostic(lines_in_file, self.max, Span::new(end, end)));
        }
    }
}

fn line_has_just_comment(line: &str, comment_chars: &str) -> bool {
    if let Some(line) = line.trim().strip_prefix(comment_chars) {
        line.is_empty()
    } else {
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x;", None),
        ("var xy;\nvar xy;", None),
        ("A", Some(serde_json::json!([1]))),
        ("A\n", Some(serde_json::json!([1]))),
        ("A\r", Some(serde_json::json!([1]))),
        ("A\r\n", Some(serde_json::json!([1]))),
        ("var xy;\nvar xy;", Some(serde_json::json!([2]))),
        ("var xy;\nvar xy;\n", Some(serde_json::json!([2]))),
        ("var xy;\nvar xy;", Some(serde_json::json!([{ "max": 2 }]))),
        ("// comment\n", Some(serde_json::json!([{ "max": 0, "skipComments": true }]))),
        ("foo;\n /* comment */\n", Some(serde_json::json!([{ "max": 1, "skipComments": true }]))),
        (
            "//a single line comment
			var xy;
			var xy;
			 /* a multiline
			 really really
			 long comment*/ ",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var x; /* inline comment\nspanning multiple lines */ var z;",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var x; /* inline comment
			 spanning multiple lines */
			var z;",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var x;



			var y;",
            Some(serde_json::json!([{ "max": 2, "skipBlankLines": true }])),
        ),
        (
            "//a single line comment
			var xy;

			var xy;

			 /* a multiline
			 really really
			 long comment*/",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": true }])),
        ),
        (
            "/* eslint-disable max-lines */

            ;
            ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    let fail = vec![
        ("var xyz;\nvar xyz;\nvar xyz;", Some(serde_json::json!([2]))),
        (
            "/* a multiline comment\n that goes to many lines*/\nvar xy;\nvar xy;",
            Some(serde_json::json!([2])),
        ),
        ("//a single line comment\nvar xy;\nvar xy;", Some(serde_json::json!([2]))),
        (
            "var x;



			var y;",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "//a single line comment
			var xy;

			var xy;

			 /* a multiline
			 really really
			 long comment*/",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var x; // inline comment
			var y;
			var z;",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var x; /* inline comment
			 spanning multiple lines */
			var y;
			var z;",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "//a single line comment
			var xy;

			var xy;

			 /* a multiline
			 really really
			 long comment*/",
            Some(serde_json::json!([{ "max": 2, "skipBlankLines": true }])),
        ),
        ("", Some(serde_json::json!([{ "max": 0 }]))),
        (" ", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "
			",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        ("A", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "A
			",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "A
			 ",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "A
			 ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "A

			",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "var a = 'a';
			var x
			var c;
			console.log",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "var a = 'a',
			c,
			x;
",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "var a = 'a',
			c,
			x;
			",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "

			var a = 'a',
			c,
			x;
			",
            Some(serde_json::json!([{ "max": 2, "skipBlankLines": true }])),
        ),
        (
            "var a = 'a';
			var x
			var c;
			console.log
			// some block
			// comments",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var a = 'a';
			var x
			var c;
			console.log
			/* block comments */",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var a = 'a';
			var x
			var c;
			console.log
			/* block comments */
			",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var a = 'a';
			var x
			var c;
			console.log
			/** block

			 comments */",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var a = 'a';


			// comment",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "var a = 'a';
			var x


			var c;
			console.log

			",
            Some(serde_json::json!([{ "max": 2, "skipBlankLines": true }])),
        ),
        (
            "var a = 'a';


			var x
			var c;
			console.log

			",
            Some(serde_json::json!([{ "max": 2, "skipBlankLines": true }])),
        ),
        (
            "var a = 'a';
			//
			var x
			var c;
			console.log
			//",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "// hello world
			/*hello
			 world 2 */
			var a,
			b
			// hh
			c,
			e,
			f;",
            Some(serde_json::json!([{ "max": 2, "skipComments": true }])),
        ),
        (
            "
			var x = '';

			// comment

			var b = '',
			c,
			d,
			e

			// comment",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": true }])),
        ),
    ];

    Tester::new(MaxLines::NAME, pass, fail).test_and_snapshot();
}
