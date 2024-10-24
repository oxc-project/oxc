use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_octal_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Octal literals should not be used.")
        .with_help("Disallow octal literals.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOctal;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow octal literals
    ///
    /// ### Why is this bad?
    /// Because the leading zero which identifies an octal literal has been a source of confusion and error in JavaScript code, ECMAScript 5 deprecates the use of octal numeric literals.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var num = 071;
    /// var result = 5 + 07;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var num  = "071";
    /// ```
    NoOctal,
    correctness,
);

impl Rule for NoOctal {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::NumericLiteral(numeric) = node.kind() {
            let mut chars = numeric.raw.chars();
            match (chars.next(), chars.next()) {
                (Some('0'), Some(v)) => {
                    if !matches!(v, '.' | 'x' | 'X') {
                        ctx.diagnostic(no_octal_diagnostic(numeric.span));
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["var a = 'hello world';", "0x1234", "0X5;", "a = 0;", "0.1", "0.5e1"];

    let fail = vec![
        "var a = 01234;",
        "a = 1 + 01234;",
        "00",
        "08",
        "09.1",
        "09e1",
        "09.1e1",
        "018",
        "019.1",
        "019e1",
        "019.1e1",
    ];

    Tester::new(NoOctal::NAME, pass, fail).test_and_snapshot();
}
