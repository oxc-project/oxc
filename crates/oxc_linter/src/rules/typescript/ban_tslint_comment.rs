use lazy_static::lazy_static;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, fixer::Fix, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(ban-tslint-comment): tslint comment detected: \"{0}\"")]
#[diagnostic(severity(warning))]
struct BanTslintCommentDiagnostic(String, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct BanTslintComment;

declare_oxc_lint!(
    /// ### What it does
    /// This rule disallows `tslint:<rule-flag>` comments
    ///
    /// ### Why is this bad?
    /// Useful when migrating from TSLint to ESLint. Once TSLint has been
    /// removed, this rule helps locate TSLint annotations
    ///
    /// ### Example
    /// ```javascript
    /// // tslint:disable-next-line
    /// someCode();
    /// ```
    BanTslintComment,
    style
);

impl Rule for BanTslintComment {
    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.semantic().trivias().comments();
        let source_text_len = ctx.semantic().source_text().len();

        for (start, comment) in comments {
            let raw = &ctx.semantic().source_text()[*start as usize..comment.end() as usize];

            if is_tslint_comment_directive(raw) {
                let comment_span = get_full_comment(
                    source_text_len,
                    *start,
                    comment.end(),
                    comment.is_multi_line(),
                );

                ctx.diagnostic_with_fix(
                    BanTslintCommentDiagnostic(raw.trim().to_string(), comment_span),
                    || Fix::delete(comment_span),
                );
            }
        }
    }
}

fn is_tslint_comment_directive(raw: &str) -> bool {
    lazy_static! {
        static ref ENABLE_DISABLE_REGEX: Regex =
            Regex::new(r"^\s*tslint:(enable|disable)(?:-(line|next-line))?(:|\s|$)").unwrap();
    }

    ENABLE_DISABLE_REGEX.is_match(raw)
}

fn get_full_comment(source_text_len: usize, start: u32, end: u32, is_multi_line: bool) -> Span {
    let comment_start = start - 2;
    let mut comment_end = if is_multi_line { end + 2 } else { end };

    // Take into account new line at the end of the comment
    if source_text_len > comment_end as usize {
        comment_end += 1;
    }

    Span::new(comment_start, comment_end)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"let a: readonly any[] = [];",
        r"let a = new Array();",
        r"// some other comment",
        r"// TODO: this is a comment that mentions tslint",
        r"/* another comment that mentions tslint */",
        r"someCode(); // This is a comment that just happens to mention tslint",
    ];

    let fail = vec![
        r"/* tslint:disable */",
        r"/* tslint:enable */",
        r"/* tslint:disable:rule1 rule2 rule3... */",
        r"/* tslint:enable:rule1 rule2 rule3... */",
        r"// tslint:disable-next-line",
        r"someCode(); // tslint:disable-line",
        r"// tslint:disable-next-line:rule1 rule2 rule3...",
        r"const woah = doSomeStuff();
        // tslint:disable-line
        console.log(woah);
        ",
    ];

    let fix = vec![
        (
            r"const woah = doSomeStuff();
        // tslint:disable-line
        console.log(woah);",
            r"const woah = doSomeStuff();
                console.log(woah);",
            None,
        ),
        (
            r"const woah = doSomeStuff();
        /* tslint:disable-line */
        console.log(woah);",
            r"const woah = doSomeStuff();
                console.log(woah);",
            None,
        ),
        (r"/* tslint:disable-line */", r"", None),
    ];

    Tester::new(BanTslintComment::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
