use oxc_ast::Span;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Bad comparison sequence")]
#[diagnostic(
    severity(warning),
    help("Comparison result should not be used directly as an operand of another comparison.")
)]
struct BadComparisonSequenceDiagnostic(Span);

/// https://deepscan.io/docs/rules/bad-comparison-sequence
#[derive(Debug, Default, Clone)]
pub struct BadComparisonSequence;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when the comparison operator is applied two or more times in a row.
    ///
    /// ### Why is this bad?
    /// Because comparison operator is a binary operator, it is impossible to compare three or more operands at once.
    /// If comparison operator is used to compare three or more operands, only the first two operands are compared and the rest is compared with its result of boolean type.
    ///
    /// ### Example
    /// ```javascript
    /// if (a == b == c) {
    ///  console.log("a, b, and c are the same");
    /// }
    /// ```
    BadComparisonSequence,
    correctness
);

impl Rule for BadComparisonSequence {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![];

    Tester::new(BadComparisonSequence::NAME, pass, fail).test_and_snapshot();
}
