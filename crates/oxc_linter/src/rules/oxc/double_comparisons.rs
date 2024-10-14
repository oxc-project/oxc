use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, utils::is_same_reference, AstNode};

fn double_comparisons_diagnostic(span: Span, operator: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected double comparisons.")
        .with_help(format!(
            "This logical expression can be simplified. Try using the `{operator}` operator instead."
        ))
        .with_label(span)
}

/// <https://rust-lang.github.io/rust-clippy/master/index.html#/double_comparisons>
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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// x === y || x < y;
    /// x < y || x === y;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// x <= y;
    /// x >= y;
    /// ```
    DoubleComparisons,
    correctness,
    fix
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

        ctx.diagnostic_with_fix(
            double_comparisons_diagnostic(logical_expr.span, new_op),
            |fixer| {
                let modified_code = {
                    let mut codegen = fixer.codegen();
                    codegen.print_expression(llhs);
                    codegen.print_ascii_byte(b' ');
                    codegen.print_str(new_op);
                    codegen.print_ascii_byte(b' ');
                    codegen.print_expression(lrhs);
                    codegen.into_source_text()
                };

                fixer.replace(logical_expr.span, modified_code)
            },
        );
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

    let fix = vec![
        ("x == y || x < y", "x <= y"),
        ("x < y || x == y", "x <= y"),
        ("x == y || x > y", "x >= y"),
        ("x > y || x == y", "x >= y"),
        ("x < y || x > y", "x != y"),
        ("x > y || x < y", "x != y"),
        ("x <= y && x >= y", "x == y"),
        ("x >= y && x <= y", "x == y"),
        ("x === y || x < y", "x <= y"),
        ("x < y || x === y", "x <= y"),
        ("x === y || x > y", "x >= y"),
        ("x > y || x === y", "x >= y"),
    ];

    Tester::new(DoubleComparisons::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
