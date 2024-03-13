use oxc_ast::CommentKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::fixer::Fix;
use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "typescript-eslint(prefer-ts-expect-error): Enforce using `@ts-expect-error` over `@ts-ignore`"
)]
#[diagnostic(
    severity(warning),
    help("Use \"@ts-expect-error\" to ensure an error is actually being suppressed.")
)]
struct PreferTsExpectErrorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferTsExpectError;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using @ts-expect-error over @ts-ignore.
    ///
    /// ### Why is this bad?
    /// TypeScript allows you to suppress all errors on a line by placing a comment starting with @ts-ignore or @ts-expect-error immediately before the erroring line.
    /// The two directives work the same, except @ts-expect-error causes a type error if placed before a line that's not erroring in the first place.
    ///
    /// This means it's easy for @ts-ignores to be forgotten about, and remain in code even after the error they were suppressing is fixed.
    /// This is dangerous, as if a new error arises on that line it'll be suppressed by the forgotten about @ts-ignore, and so be missed.
    ///
    /// ### Example
    /// ```javascript
    /// // @ts-ignore
    /// const str: string = 1;
    ///
    /// /**
    /// * Explaining comment
    /// *
    /// * @ts-ignore */
    /// const multiLine: number = 'value';
    /// ```
    PreferTsExpectError,
    pedantic
);

impl Rule for PreferTsExpectError {
    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.semantic().trivias().comments();

        for (kind, span) in comments {
            let raw = span.source_text(ctx.semantic().source_text());

            if !is_valid_ts_ignore_present(kind, raw) {
                continue;
            }

            if kind.is_single_line() {
                let comment_span = Span::new(span.start - 2, span.end);
                ctx.diagnostic_with_fix(PreferTsExpectErrorDiagnostic(comment_span), || {
                    Fix::new(
                        format!("//{}", raw.replace("@ts-ignore", "@ts-expect-error")),
                        comment_span,
                    )
                });
            } else {
                let comment_span = Span::new(span.start - 2, span.end + 2);
                ctx.diagnostic_with_fix(PreferTsExpectErrorDiagnostic(comment_span), || {
                    Fix::new(
                        format!("/*{}*/", raw.replace("@ts-ignore", "@ts-expect-error")),
                        comment_span,
                    )
                });
            }
        }
    }
}

fn get_last_comment_line(comment: CommentKind, raw: &str) -> String {
    if comment.is_single_line() {
        return String::from(raw);
    }

    return String::from(raw.lines().last().unwrap_or(raw));
}

fn is_valid_ts_ignore_present(comment: CommentKind, raw: &str) -> bool {
    let line = get_last_comment_line(comment, raw);

    if comment.is_single_line() {
        test_single_line_comment(&line)
    } else {
        test_multi_line_comment(&line)
    }
}

fn test_single_line_comment(line: &str) -> bool {
    line.trim_start_matches(|c: char| c.is_whitespace() || c == '/').starts_with("@ts-ignore")
}

fn test_multi_line_comment(line: &str) -> bool {
    line.trim_start_matches(|c: char| c.is_whitespace() || c == '/' || c == '*')
        .starts_with("@ts-ignore")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "// @ts-nocheck",
        "// @ts-check",
        "// just a comment containing @ts-ignore somewhere",
        "
{
/*
just a comment containing @ts-ignore somewhere in a block
*/
}
		",
        "// @ts-expect-error",
        "
if (false) {
// @ts-expect-error: Unreachable code error
console.log('hello');
}
		",
        "
/**
* Explaining comment
*
* @ts-expect-error
*
* Not last line
* */
        ",
    ];

    let fail = vec![
        "// @ts-ignore",
        "// @ts-ignore: Suppress next line",
        "///@ts-ignore: Suppress next line",
        "
if (false) {
// @ts-ignore: Unreachable code error
console.log('hello');
}
		",
        "/* @ts-ignore */",
        "
/**
* Explaining comment
*
* @ts-ignore */
		",
        "/* @ts-ignore in a single block */",
        "
/*
// @ts-ignore in a block with single line comments */
        ",
    ];

    let fix = vec![
        ("// @ts-ignore", "// @ts-expect-error", None),
        ("// @ts-ignore: Suppress next line", "// @ts-expect-error: Suppress next line", None),
        ("///@ts-ignore: Suppress next line", "///@ts-expect-error: Suppress next line", None),
        (
            "
if (false) {
// @ts-ignore: Unreachable code error
console.log('hello');
}
            ",
            "
if (false) {
// @ts-expect-error: Unreachable code error
console.log('hello');
}
            ",
            None,
        ),
        ("/* @ts-ignore */", "/* @ts-expect-error */", None),
        (
            "
/**
* Explaining comment
*
* @ts-ignore */
            ",
            "
/**
* Explaining comment
*
* @ts-expect-error */
            ",
            None,
        ),
        ("/* @ts-ignore in a single block */", "/* @ts-expect-error in a single block */", None),
        (
            "
/*
// @ts-ignore in a block with single line comments */
            ",
            "
/*
// @ts-expect-error in a block with single line comments */
            ",
            None,
        ),
    ];

    Tester::new(PreferTsExpectError::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
