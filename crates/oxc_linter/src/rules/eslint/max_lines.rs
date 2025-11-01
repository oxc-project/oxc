use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, utils::count_comment_lines};

fn max_lines_diagnostic(count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("File has too many lines ({count})."))
        .with_help(format!("Maximum allowed is {max}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MaxLines(Box<MaxLinesConfig>);

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxLinesConfig {
    /// Maximum number of lines allowed per file.
    max: usize,
    /// Whether to ignore blank lines when counting.
    skip_blank_lines: bool,
    /// Whether to ignore comments when counting.
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
    ///
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
    eslint,
    pedantic,
    config = MaxLinesConfig,
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

    #[expect(clippy::cast_possible_truncation)]
    fn run_once(&self, ctx: &LintContext) {
        let comment_lines = if self.skip_comments {
            ctx.semantic()
                .comments()
                .iter()
                .map(|comment| count_comment_lines(comment, ctx.source_text()))
                .sum()
        } else {
            0
        };

        let lines_in_file = if self.skip_blank_lines {
            ctx.source_text().lines().filter(|&line| !line.trim().is_empty()).count()
        } else {
            // Intentionally counting newline bytes instead of using .lines() for performance (see PR 11242)
            let newlines = ctx.source_text().bytes().filter(|ch| *ch == b'\n').count();
            if ctx.source_text().ends_with('\n') { newlines } else { newlines + 1 }
        };

        let final_lines = lines_in_file.max(1).saturating_sub(comment_lines);
        if final_lines > self.max {
            // Point to end of the file for `eslint-disable max-lines` to work.
            let end = ctx.source_text().len().saturating_sub(1) as u32;
            ctx.diagnostic(max_lines_diagnostic(final_lines, self.max, Span::empty(end)));
        }
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

    Tester::new(MaxLines::NAME, MaxLines::PLUGIN, pass, fail).test_and_snapshot();
}
