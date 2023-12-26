// Based on https://github.com/rust-lang/rust-clippy//blob/00e9372987755dece96561ef2eef0785c8742e55/clippy_lints/src/operators/erasing_op.rs
use oxc_ast::{
    ast::{BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "oxc(erasing-op): Unexpected erasing operation. This expression will always evaluate to zero."
)]
#[diagnostic(severity(warning), help("This is most likely not the intended outcome. Consider removing the operation, or directly assigning zero to the variable"))]
struct ErasingOpDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ErasingOp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for erasing operations, e.g., `x * 0``.
    ///
    /// Based on https://rust-lang.github.io/rust-clippy/master/#/erasing_op
    ///
    /// ### Why is this bad?
    ///
    /// The whole expression can be replaced by zero. This is most likely not the intended outcome and should probably be corrected.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// let x = 1;
    /// let y = x * 0;
    ///
    /// // Good
    /// let x = 1;
    /// let y = 0;
    /// ```
    ErasingOp,
    correctness
);

impl Rule for ErasingOp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expression) = node.kind() else { return };

        match binary_expression.operator {
            BinaryOperator::Multiplication | BinaryOperator::BitwiseAnd => {
                check_op(binary_expression, &binary_expression.left, ctx);
                check_op(binary_expression, &binary_expression.right, ctx);
            }
            BinaryOperator::Division => {
                check_op(binary_expression, &binary_expression.left, ctx);
            }
            _ => (),
        }
    }
}

fn is_number_value(expr: &Expression, value: f64) -> bool {
    if let Expression::NumberLiteral(number_literal) = expr {
        (number_literal.value - value).abs() < f64::EPSILON
    } else {
        false
    }
}

fn check_op<'a, 'b>(
    binary_expression: &'b BinaryExpression<'a>,
    op: &'b Expression<'a>,
    ctx: &LintContext<'a>,
) {
    if is_number_value(op, 0.0) {
        ctx.diagnostic(ErasingOpDiagnostic(binary_expression.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["x * 1;", "1 * x;", "5 & x;", "x / 1;", "1 / x;"];

    let fail = vec!["x * 0;", "0 * x;", "0 & x;", "0 / x;"];

    Tester::new_without_config(ErasingOp::NAME, pass, fail).test_and_snapshot();
}
