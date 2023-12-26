use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, utils::is_same_reference, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("oxc(double-comparisons): Unexpected double comparisons.")]
#[diagnostic(
    severity(warning),
    help("This logical expression can be simplified. Try using the `{1}` operator instead.")
)]
struct DoubleComparisonsDiagnostic(#[label] pub Span, &'static str);

/// https://rust-lang.github.io/rust-clippy/master/index.html#/double_comparisons
#[derive(Debug, Default, Clone)]
pub struct DoubleComparisons;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks for double comparisons in logical expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Redundant comparisons can be confusing and make code harder to understand.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// x === y || x < y;
    /// x < y || x === y;
    ///
    /// // Good
    /// x <= y;
    /// x >= y;
    /// ```
    DoubleComparisons,
    correctness,
);

#[allow(clippy::similar_names)]
impl Rule for DoubleComparisons {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expr) = node.kind() else {
            return;
        };

        let (lkind, llhs, lrhs, rkind, rlhs, rrhs) = match (&logical_expr.left, &logical_expr.right)
        {
            (
                Expression::BinaryExpression(left_bin_expr),
                Expression::BinaryExpression(right_bin_expr),
            ) => (
                left_bin_expr.operator,
                &left_bin_expr.left,
                &left_bin_expr.right,
                right_bin_expr.operator,
                &right_bin_expr.left,
                &right_bin_expr.right,
            ),
            _ => return,
        };

        // check that (LLHS === RLHS && LRHS === RRHS) || (LLHS === RRHS && LRHS === RLHS)
        if !((is_same_reference(llhs, rlhs, ctx) && is_same_reference(lrhs, rrhs, ctx))
            || (is_same_reference(llhs, rrhs, ctx) && is_same_reference(lrhs, rlhs, ctx)))
        {
            return;
        }

        #[rustfmt::skip]
        let new_op = match (logical_expr.operator, lkind, rkind) {
            (LogicalOperator::Or, BinaryOperator::Equality | BinaryOperator::StrictEquality, BinaryOperator::LessThan)
            | (LogicalOperator::Or, BinaryOperator::LessThan, BinaryOperator::Equality | BinaryOperator::StrictEquality) => "<=",
            (LogicalOperator::Or, BinaryOperator::Equality | BinaryOperator::StrictEquality, BinaryOperator::GreaterThan)
            | (LogicalOperator::Or, BinaryOperator::GreaterThan, BinaryOperator::Equality | BinaryOperator::StrictEquality) => ">=",
            (LogicalOperator::Or, BinaryOperator::LessThan, BinaryOperator::GreaterThan)
            | (LogicalOperator::Or, BinaryOperator::GreaterThan, BinaryOperator::LessThan) => "!=",
            (LogicalOperator::And, BinaryOperator::LessEqualThan, BinaryOperator::GreaterEqualThan)
            | (LogicalOperator::And, BinaryOperator::GreaterEqualThan,BinaryOperator::LessEqualThan) => "==",
            _ => return,
        };

        ctx.diagnostic(DoubleComparisonsDiagnostic(logical_expr.span, new_op));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "x == y && x == y",
        "x == y && x != y",
        "x != y && x == y",
        "x != y && x != y",
        "x < y && x < y",
        "x < y && x <= y",
        "x <= y && x < y",
        "x <= y && x <= y",
        "x > y && x > y",
        "x > y && x >= y",
        "x >= y && x > y",
        "x >= y && x >= y",
        "x >= y && x >= y",
        "x == y || fs < y",
        "x < y || ab == y",
        "x == y || qr > y",
        "(first.range[0] <= second.range[0] && first.range[1] >= second.range[0])",
    ];

    let fail = vec![
        "x == y || x < y",
        "x < y || x == y",
        "x == y || x > y",
        "x > y || x == y",
        "x < y || x > y",
        "x > y || x < y",
        "x <= y && x >= y",
        "x >= y && x <= y",
        "x === y || x < y",
        "x < y || x === y",
        "x === y || x > y",
        "x > y || x === y",
    ];

    Tester::new_without_config(DoubleComparisons::NAME, pass, fail).test_and_snapshot();
}
