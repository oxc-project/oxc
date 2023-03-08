use oxc_ast::{ast::Expression, AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-self-compare): Disallow comparisons where both sides are exactly the same")]
#[diagnostic()]
struct NoSelfCompareDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoSelfCompare {}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow comparisons where both sides are exactly the same
    ///
    /// ### Why is this bad?
    /// Comparing a variable against itself is usually an error, either a typo or refactoring error.
    /// It is confusing to the reader and may potentially introduce a runtime error.
    ///
    /// ### Example
    /// ```javascript
    /// var x = 10;
    /// if (x === x) {
    ///   x = 20;
    /// }
    /// ```
    NoSelfCompare,
    correctness
);

impl Rule for NoSelfCompare {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.get().kind() else {return};
        println!("{:#?}", binary_expr);
        match (&binary_expr.left, &binary_expr.right) {
            (Expression::Identifier(left), Expression::Identifier(right)) => {
                if left.name == right.name {
                    ctx.diagnostic(NoSelfCompareDiagnostic(binary_expr.span));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![("x === x", None)];

    Tester::new(NoSelfCompare::NAME, pass, fail).test_and_snapshot();
}
