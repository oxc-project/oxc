use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{context::LintContext, rule::Rule};

fn ban_tslint_comment_diagnostic(tslint_comment: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("tslint comment detected: \"{tslint_comment}\"")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BanTslintComment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows `tslint:<rule-flag>` comments.
    ///
    /// ### Why is this bad?
    ///
    /// Useful when migrating from TSLint to ESLint. Once TSLint has been
    /// removed, this rule helps locate TSLint annotations
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // tslint:disable-next-line
    /// someCode();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// someCode();
    /// ```
    BanTslintComment,
    typescript,
    style,
    fix,
    version = "0.2.9",
    short_description = "This rule disallows `tslint:<rule-flag>` comments.",
);

/// `^\s*tslint:(enable|disable)(?:-(line|next-line))?(:|\s|$)`
fn is_tslint_enable_disable_comment(raw: &str) -> bool {
    let Some(rest) = raw.trim_start().strip_prefix("tslint:") else { return false };
    let Some(rest) = rest.strip_prefix("enable").or_else(|| rest.strip_prefix("disable")) else {
        return false;
    };
    let rest =
        rest.strip_prefix("-line").or_else(|| rest.strip_prefix("-next-line")).unwrap_or(rest);
    rest.is_empty() || rest.starts_with(':') || rest.starts_with(char::is_whitespace)
}

impl Rule for BanTslintComment {
    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.comments();
        for comment in comments {
            let raw = ctx.source_range(comment.content_span());
            if is_tslint_enable_disable_comment(raw) {
                let comment_span = get_full_comment(ctx, comment.span);
                ctx.diagnostic_with_fix(
                    ban_tslint_comment_diagnostic(raw.trim(), comment_span),
                    |fixer| {
                        if comment.is_multiline_block() {
                            fixer.replace(comment_span, "\n")
                        } else {
                            fixer.delete_range(comment_span)
                        }
                    },
                );
            }
        }
    }
}

fn get_full_comment(ctx: &LintContext, span: Span) -> Span {
    let mut span = span;

    let source_size = u32::try_from(ctx.source_text().len()).unwrap();
    let tokens = ctx.source_range(Span::new(span.end, source_size));

    let starts_line = ctx.source_text()[..span.start as usize]
        .chars()
        .rev()
        .take_while(|&c| !is_line_terminator(c))
        .all(char::is_whitespace);
    if starts_line && tokens.starts_with('\n') {
        span.end += 1;
    }

    span
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let a: readonly any[] = [];",
        "let a = new Array();",
        "// some other comment",
        "// TODO: this is a comment that mentions tslint",
        "/* another comment that mentions tslint */",
        "someCode(); // This is a comment that just happens to mention tslint",
        "// tslint:disabled",
        "// TSLint:disable",
    ];

    let fail = vec![
        "/* tslint:disable */",
        "/* tslint:enable */",
        "/* tslint:disable:rule1 rule2 rule3... */",
        "/* tslint:enable:rule1 rule2 rule3... */",
        "// tslint:disable-next-line",
        "someCode(); // tslint:disable-line",
        "// tslint:disable-next-line:rule1 rule2 rule3...",
        "const woah = doSomeStuff();
        // tslint:disable-line
        console.log(woah);
        ",
    ];

    let fix = vec![
        (
            "const woah = doSomeStuff();
        // tslint:disable-line
        console.log(woah);",
            "const woah = doSomeStuff();
                console.log(woah);",
            None,
        ),
        (
            "const woah = doSomeStuff();
        /* tslint:disable-line */
        console.log(woah);",
            "const woah = doSomeStuff();
                console.log(woah);",
            None,
        ),
        ("/* tslint:disable-line */", "", None),
        // Issue: <https://github.com/oxc-project/oxc/issues/8090>
        (r"/*tslint:disable*/É", r"É", None),
        (
            "function f() {\n  return /* tslint:disable */\n  7;\n}",
            "function f() {\n  return \n  7;\n}",
            None,
        ),
        (
            "const a = 1 // tslint:disable-line\nconst b = 2\nconsole.log(a, b)",
            "const a = 1 \nconst b = 2\nconsole.log(a, b)",
            None,
        ),
        (
            "function g() {\n  return /* tslint:disable\n  */ 7\n}",
            "function g() {\n  return \n 7\n}",
            None,
        ),
        (
            "function f() { return /* tslint:disable\r\n*/ 7; }",
            "function f() { return \n 7; }",
            None,
        ),
        ("function f() { return /* tslint:disable\r*/ 7; }", "function f() { return \n 7; }", None),
        (
            "function f() { return /* tslint:disable\u{2028}*/ 7; }",
            "function f() { return \n 7; }",
            None,
        ),
        (
            "function f() { return /* tslint:disable\u{2029}*/ 7; }",
            "function f() { return \n 7; }",
            None,
        ),
        ("const a = 1 // tslint:disable-line\r\nconst b = 2", "const a = 1 \r\nconst b = 2", None),
        ("function f() { return /* tslint:disable */ 7; }", "function f() { return  7; }", None),
        (
            "const a = 1 /* tslint:disable */ /* tslint:enable */\nconst b = 2",
            "const a = 1  \nconst b = 2",
            None,
        ),
        (
            "const a = 1\u{2028}\t// tslint:disable-line\nconst b = 2",
            "const a = 1\u{2028}\tconst b = 2",
            None,
        ),
    ];

    Tester::new(BanTslintComment::NAME, BanTslintComment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
