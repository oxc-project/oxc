use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

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
                let (comment_span, replacement) = comment_replacement(ctx, comment.span);
                ctx.diagnostic_with_fix(
                    ban_tslint_comment_diagnostic(raw.trim(), comment_span),
                    |fixer| {
                        if replacement.is_empty() {
                            fixer.delete_range(comment_span)
                        } else {
                            fixer.replace(comment_span, replacement)
                        }
                    },
                );
            }
        }
    }
}

fn comment_replacement(ctx: &LintContext, span: Span) -> (Span, String) {
    let mut span = span;
    let source = ctx.source_text();
    let before = &source[..span.start as usize];
    let after = &source[span.end as usize..];
    let is_line_terminator = |c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}');
    let starts_line = before.rsplit(is_line_terminator).next().unwrap().trim().is_empty();

    // Only consume the following newline when the comment is alone on its line.
    if starts_line {
        let newline_len = if after.starts_with("\r\n") {
            2
        } else {
            after.chars().next().filter(|c| is_line_terminator(*c)).map_or(0, char::len_utf8)
        };
        if newline_len > 0 {
            span.end += u32::try_from(newline_len).unwrap();
            return (span, String::new());
        }
    }

    // Line terminators inside block comments participate in ASI too.
    let mut replacement: String =
        ctx.source_range(span).chars().filter(|c| is_line_terminator(*c)).collect();
    if replacement.is_empty()
        && !before.is_empty()
        && !after.is_empty()
        && !before.ends_with(char::is_whitespace)
        && !after.starts_with(char::is_whitespace)
    {
        replacement.push(' ');
    }
    (span, replacement)
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
        ("function f() { return /* tslint:disable */\n7; }", "function f() { return \n7; }", None),
        ("const a = 1 // tslint:disable-line\nconst b = 2", "const a = 1 \nconst b = 2", None),
        ("function f() { return /* tslint:disable\n*/ 7; }", "function f() { return \n 7; }", None),
        ("const a = 1 // tslint:disable-line\r\nconst b = 2", "const a = 1 \r\nconst b = 2", None),
        ("// tslint:disable\r\nconst a = 1", "const a = 1", None),
        (
            "function f() { return /* tslint:disable\u{2028}*/ 7; }",
            "function f() { return \u{2028} 7; }",
            None,
        ),
        ("let/* tslint:disable */value = 1", "let value = 1", None),
    ];

    Tester::new(BanTslintComment::NAME, BanTslintComment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
