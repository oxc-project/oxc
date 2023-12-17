use std::cmp::Ordering;

use oxc_ast::{
    ast::{Expression, NumberLiteral},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, utils::is_same_reference, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum ConstComparisonsDiagnostic {
    #[error("oxc(const-comparisons): Unexpected redundant comparison. Left-hand side of `&&` operator has no effect")]
    #[diagnostic(severity(warning), help("{1}"))]
    RedundantLeftHandSide(#[label] Span, String),
    #[error("oxc(const-comparisons): Unexpected redundant comparison. Right-hand side of `&&` operator has no effect")]
    #[diagnostic(severity(warning), help("{1}"))]
    RedundantRightHandSide(#[label] Span, String),
    #[error("oxc(const-comparisons): Unexpected constant comparison")]
    #[diagnostic(severity(warning), help("{1}"))]
    Impossible(#[label] Span, String),
}

#[derive(Debug, Default, Clone)]
pub struct ConstComparisons;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for redundant comparisons between constants:
    ///  - Checks for ineffective double comparisons against constants.
    ///  - Checks for impossible comparisons against constants.
    ///
    /// ### Why is this bad?
    ///
    /// Only one of the comparisons has any effect on the result, the programmer probably intended to flip one of the comparison operators, or compare a different value entirely.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// status_code <= 400 && status_code > 500;
    /// status_code < 200 && status_code <= 299;
    /// status_code > 500 && status_code >= 500;
    ///
    /// // Good
    /// status_code >= 400 && status_code < 500;
    /// 500 <= status_code && 600 > status_code;
    /// 500 <= status_code && status_code <= 600;
    /// ```
    ConstComparisons,
    correctness
);

impl Rule for ConstComparisons {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expr) = node.kind() else {
            return;
        };

        if logical_expr.operator != LogicalOperator::And {
            return;
        }

        let Some((left_cmp_op, left_expr, left_const_expr)) =
            comparison_to_const(logical_expr.left.without_parenthesized())
        else {
            return;
        };
        let Some((right_cmp_op, right_expr, right_const_expr)) =
            comparison_to_const(logical_expr.right.without_parenthesized())
        else {
            return;
        };

        let Some(ordering) = left_const_expr.value.partial_cmp(&right_const_expr.value) else {
            return;
        };

        // Rule out the `x >= 42 && x <= 42` corner case immediately
        // Mostly to simplify the implementation, but it is also covered by `clippy::double_comparisons`
        if matches!(
            (&left_cmp_op, &right_cmp_op, ordering),
            (CmpOp::Le | CmpOp::Ge, CmpOp::Le | CmpOp::Ge, Ordering::Equal)
        ) {
            return;
        }

        if !is_same_reference(left_expr, right_expr, ctx) {
            return;
        }

        if left_cmp_op.direction() == right_cmp_op.direction() {
            let lhs_str = left_const_expr.value.to_string();
            let rhs_str = right_const_expr.value.to_string();
            // We already know that either side of `&&` has no effect,
            // but emit a different error message depending on which side it is
            if left_side_is_useless(left_cmp_op, ordering) {
                ctx.diagnostic(ConstComparisonsDiagnostic::RedundantLeftHandSide(
                    node.kind().span(),
                    format!("if `{rhs_str}` evaluates to true, {lhs_str}` will always evaluate to true as well")
                ));
            } else {
                ctx.diagnostic(ConstComparisonsDiagnostic::RedundantRightHandSide(
                    node.kind().span(),
                    format!("if `{lhs_str}` evaluates to true, {rhs_str}` will always evaluate to true as well")
                ));
            }
        } else if !comparison_is_possible(left_cmp_op.direction(), ordering) {
            let lhs_str = left_const_expr.value.to_string();
            let rhs_str = right_const_expr.value.to_string();
            let expr_str = left_expr.span().source_text(ctx.source_text());
            let diagnostic_note = match ordering {
                Ordering::Less => format!(
                    "since `{lhs_str}` < `{rhs_str}`, the expression evaluates to false for any value of `{expr_str}`"
                ),
                Ordering::Equal => {
                    format!("`{expr_str}` cannot simultaneously be greater than and less than `{lhs_str}`")
                },
                Ordering::Greater => format!(
                    "since `{lhs_str}` > `{rhs_str}`, the expression evaluates to false for any value of `{expr_str}`"
                ),
            };

            ctx.diagnostic(ConstComparisonsDiagnostic::Impossible(
                node.kind().span(),
                diagnostic_note,
            ));
        }
    }
}

// Extract a comparison between a const and non-const
// Flip yoda conditionals, turnings expressions like `42 < x` into `x > 42`
fn comparison_to_const<'a, 'b>(
    expr: &'b Expression<'a>,
) -> Option<(CmpOp, &'b Expression<'a>, &'b NumberLiteral<'a>)> {
    if let Expression::BinaryExpression(bin_expr) = expr {
        if let Ok(cmp_op) = CmpOp::try_from(bin_expr.operator) {
            match (&bin_expr.left.without_parenthesized(), &bin_expr.right.without_parenthesized())
            {
                (Expression::NumberLiteral(lit), _) => {
                    return Some((cmp_op.reverse(), &bin_expr.right, lit));
                }
                (_, Expression::NumberLiteral(lit)) => {
                    return Some((cmp_op, &bin_expr.left, lit));
                }
                _ => {}
            }
        }
    }

    None
}

fn left_side_is_useless(left_cmp_op: CmpOp, ordering: Ordering) -> bool {
    // Special-case for equal constants with an inclusive comparison
    if ordering == Ordering::Equal {
        match left_cmp_op {
            CmpOp::Lt | CmpOp::Gt => false,
            CmpOp::Le | CmpOp::Ge => true,
        }
    } else {
        #[allow(clippy::match_same_arms)]
        match (left_cmp_op.direction(), ordering) {
            (CmpOpDirection::Lesser, Ordering::Less) => false,
            (CmpOpDirection::Lesser, Ordering::Equal) => false,
            (CmpOpDirection::Lesser, Ordering::Greater) => true,
            (CmpOpDirection::Greater, Ordering::Less) => true,
            (CmpOpDirection::Greater, Ordering::Equal) => false,
            (CmpOpDirection::Greater, Ordering::Greater) => false,
        }
    }
}

fn comparison_is_possible(left_cmp_direction: CmpOpDirection, ordering: Ordering) -> bool {
    #[allow(clippy::match_same_arms)]
    match (left_cmp_direction, ordering) {
        (CmpOpDirection::Lesser, Ordering::Less | Ordering::Equal) => false,
        (CmpOpDirection::Lesser, Ordering::Greater) => true,
        (CmpOpDirection::Greater, Ordering::Greater | Ordering::Equal) => false,
        (CmpOpDirection::Greater, Ordering::Less) => true,
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CmpOpDirection {
    Lesser,
    Greater,
}

#[derive(Clone, Copy)]
enum CmpOp {
    Lt,
    Le,
    Ge,
    Gt,
}

impl CmpOp {
    fn reverse(self) -> Self {
        match self {
            Self::Lt => Self::Gt,
            Self::Le => Self::Ge,
            Self::Ge => Self::Le,
            Self::Gt => Self::Lt,
        }
    }

    fn direction(self) -> CmpOpDirection {
        match self {
            Self::Lt | Self::Le => CmpOpDirection::Lesser,
            Self::Ge | Self::Gt => CmpOpDirection::Greater,
        }
    }
}

impl TryFrom<BinaryOperator> for CmpOp {
    type Error = ();

    fn try_from(bin_op: BinaryOperator) -> Result<Self, Self::Error> {
        match bin_op {
            BinaryOperator::LessThan => Ok(Self::Lt),
            BinaryOperator::LessEqualThan => Ok(Self::Le),
            BinaryOperator::GreaterThan => Ok(Self::Gt),
            BinaryOperator::GreaterEqualThan => Ok(Self::Ge),
            _ => Err(()),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Cases adapted from https://github.com/rust-lang/rust-clippy//blob/cc61aeea54138d3d037452754456539cf4ae6192/tests/ui/const_comparisons.rs#L39
    let pass = vec![
        "status_code >= 400 && status_code < 500;",
        // Yoda conditions
        // Correct
        "500 <= status_code && 600 > status_code;",
        // Correct
        "500 <= status_code && status_code <= 600;",
        // Yoda conditions, comparing two different types
        // Correct
        "500 <= status && 600 > status;",
        // Correct
        "500 <= status && status <= 600;",
        // incorrect, but allowed since the expressions are different
        "500 <= foo && 600 > bar;",
        "500 <= foo.baz.que && bar.aaa.fff <= 600;",
        "500 <= foo[0].que && foo[1].que <= 600",
        "styleCodes.length >= 5 && styleCodes[2] >= 0",
    ];

    let fail = vec![
        "status_code <= 400 && status_code > 500;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: since `400` < `500`, the expression evaluates to false for any value of `st
        "status_code > 500 && status_code < 400;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: since `500` > `400`, the expression evaluates to false for any value of `st
        "status_code < 500 && status_code > 500;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: `status_code` cannot simultaneously be greater than and less than `500`

        // Yoda conditions
        // Incorrect
        "500 >= status_code && 600 < status_code;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: since `500` < `600`, the expression evaluates to false for any value of `st
        // Incorrect
        "500 >= status_code && status_code > 600;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: since `500` < `600`, the expression evaluates to false for any value of `st

        // Yoda conditions, comparing two different types
        // Incorrect
        "500 >= status && 600 < status;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: since `500` < `600`, the expression evaluates to false for any value of `st
        // Incorrect
        "500 >= status && status > 600;",
        //~^ ERROR: boolean expression will never evaluate to 'true'
        //~| NOTE: since `500` < `600`, the expression evaluates to false for any value of `st

        // Expressions where one of the sides has no effect
        "status_code < 200 && status_code <= 299;",
        //~^ ERROR: right-hand side of `&&` operator has no effect
        "status_code > 200 && status_code >= 299;",
        //~^ ERROR: left-hand side of `&&` operator has no effect

        // Useless left
        "status_code >= 500 && status_code > 500;",
        //~^ ERROR: left-hand side of `&&` operator has no effect
        // Useless right
        "status_code > 500 && status_code >= 500;",
        //~^ ERROR: right-hand side of `&&` operator has no effect
        // Useless left
        "status_code <= 500 && status_code < 500;",
        //~^ ERROR: left-hand side of `&&` operator has no effect
        // Useless right
        "status_code < 500 && status_code <= 500;",
        //~^ ERROR: right-hand side of `&&` operator has no effect
    ];

    Tester::new_without_config(ConstComparisons::NAME, pass, fail).test_and_snapshot();
}
