// Based on https://github.com/rust-lang/rust-clippy//blob/6246f0446afbe9abff18e8cc1ebaae7505f7cd9e/clippy_lints/src/operators/const_comparisons.rs
use std::cmp::Ordering;

use oxc_ast::{
    ast::{Expression, NumericLiteral},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, utils::is_same_reference, AstNode};

fn redundant_left_hand_side(span: Span, span1: Span, help: String) -> OxcDiagnostic {
    OxcDiagnostic::warn("Left-hand side of `&&` operator has no effect.")
        .with_help(help)
        .with_labels([
            span.label("If this evaluates to `true`"),
            span1.label("This will always evaluate to true."),
        ])
}

fn redundant_right_hand_side(span: Span, span1: Span, help: String) -> OxcDiagnostic {
    OxcDiagnostic::warn("Right-hand side of `&&` operator has no effect.")
        .with_help(help)
        .with_labels([
            span.label("If this evaluates to `true`"),
            span1.label("This will always evaluate to true."),
        ])
}

fn impossible(span: Span, span1: Span, x2: &str, x3: &str, x4: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected constant comparison").with_help(x4.to_string()).with_labels([
        span.label(format!("Requires that {x2}")),
        span1.label(format!("Requires that {x3}")),
    ])
}

/// <https://rust-lang.github.io/rust-clippy/master/index.html#/impossible>
/// <https://rust-lang.github.io/rust-clippy/master/index.html#/redundant_comparisons>
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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// status_code <= 400 && status_code > 500;
    /// status_code < 200 && status_code <= 299;
    /// status_code > 500 && status_code >= 500;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
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

        let Some((right_cmp_op, right_expr, right_const_expr, _)) =
            comparison_to_const(logical_expr.right.without_parentheses())
        else {
            return;
        };

        for (left_cmp_op, left_expr, left_const_expr, left_span) in
            all_and_comparison_to_const(logical_expr.left.without_parentheses())
        {
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
                let lhs_str = left_span.source_text(ctx.source_text());
                let rhs_str = logical_expr.right.span().source_text(ctx.source_text());
                // We already know that either side of `&&` has no effect,
                // but emit a different error message depending on which side it is
                if left_side_is_useless(left_cmp_op, ordering) {
                    ctx.diagnostic(redundant_left_hand_side(left_span, logical_expr.right.span(), format!("if `{rhs_str}` evaluates to true, `{lhs_str}` will always evaluate to true as well")));
                } else {
                    ctx.diagnostic(redundant_right_hand_side(logical_expr.right.span(), left_span, format!("if `{lhs_str}` evaluates to true, `{rhs_str}` will always evaluate to true as well")));
                }
            } else if !comparison_is_possible(left_cmp_op.direction(), ordering) {
                let lhs_str = left_const_expr.span.source_text(ctx.source_text());
                let rhs_str = right_const_expr.span.source_text(ctx.source_text());
                let expr_str = left_expr.span().source_text(ctx.source_text());
                let diagnostic_note = match ordering {
                    Ordering::Less => format!(
                        "since `{lhs_str}` < `{rhs_str}`, the expression evaluates to false for any value of `{expr_str}`"
                    ),
                    Ordering::Equal => {
                        format!(
                            "`{expr_str}` cannot simultaneously be greater than and less than `{lhs_str}`"
                        )
                    }
                    Ordering::Greater => format!(
                        "since `{lhs_str}` > `{rhs_str}`, the expression evaluates to false for any value of `{expr_str}`"
                    ),
                };

                ctx.diagnostic(impossible(
                    left_span,
                    logical_expr.right.without_parentheses().span(),
                    &format!(
                        "`{} {} {}` ",
                        expr_str,
                        left_cmp_op,
                        left_const_expr.span.source_text(ctx.source_text())
                    ),
                    &format!(
                        "`{} {} {}` ",
                        expr_str,
                        right_cmp_op,
                        right_const_expr.span.source_text(ctx.source_text())
                    ),
                    &diagnostic_note,
                ));
            }
        }
    }
}

// Extract a comparison between a const and non-const
// Flip yoda conditionals, turnings expressions like `42 < x` into `x > 42`
fn comparison_to_const<'a, 'b>(
    expr: &'b Expression<'a>,
) -> Option<(CmpOp, &'b Expression<'a>, &'b NumericLiteral<'a>, Span)> {
    if let Expression::BinaryExpression(bin_expr) = expr {
        if let Ok(cmp_op) = CmpOp::try_from(bin_expr.operator) {
            match (&bin_expr.left.without_parentheses(), &bin_expr.right.without_parentheses()) {
                (Expression::NumericLiteral(lit), _) => {
                    return Some((cmp_op.reverse(), &bin_expr.right, lit, bin_expr.span));
                }
                (_, Expression::NumericLiteral(lit)) => {
                    return Some((cmp_op, &bin_expr.left, lit, bin_expr.span));
                }
                _ => {}
            }
        }
    }

    None
}

fn all_and_comparison_to_const<'a, 'b>(
    expr: &'b Expression<'a>,
) -> Box<dyn Iterator<Item = (CmpOp, &'b Expression<'a>, &'b NumericLiteral<'a>, Span)> + 'b> {
    match expr {
        Expression::LogicalExpression(logical_expr)
            if logical_expr.operator == LogicalOperator::And =>
        {
            let left_iter = all_and_comparison_to_const(logical_expr.left.without_parentheses());
            let right_iter = all_and_comparison_to_const(logical_expr.right.without_parentheses());
            Box::new(left_iter.chain(right_iter))
        }
        _ => {
            if let Some((cmp_op, expr, lit, span)) = comparison_to_const(expr) {
                Box::new(std::iter::once((cmp_op, expr, lit, span)))
            } else {
                Box::new(std::iter::empty())
            }
        }
    }
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

impl std::fmt::Display for CmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Ge => write!(f, ">="),
            Self::Gt => write!(f, ">"),
        }
    }
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
        "status_code <= 400 || foo() && status_code > 500;",
        "status_code > 500 && foo() && bar || status_code < 400;",
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

        // Multiple comparisons

        // always evaluates to false
        "status_code <= 400 && foo() && status_code > 500;",
        // always evaluates to false
        "status_code > 500 && foo() && bar && status_code < 400;",
        // always evaluates to false
        "foo() && bar && status_code < 500 && status_code > 500;",
        // Yoda
        "500 >= status_code && baz && 600 < status_code;",
        "que && 500 >= status_code && baz && status_code > 600;",
        // Yoda - two different types
        "baz && 500 >= status && 600 < status;",
        "500 >= status && baz && que() && status > 600;",
        // Multiple comparisons - useless left
        "foo() && status_code >= 500 && status_code > 500;",
        "status_code <= 500 && foo() && status_code < 500;",
        // Useless left
        "status_code >= 500 && response && status_code > 500;",
        // Useless right
        "response && status_code > 500 && status_code >= 500;",
        // Useless left
        "status_code <= 500 && response && status_code < 500;",
        // Useless right
        "status_code < 500 && response && status_code <= 500;",
    ];

    Tester::new(ConstComparisons::NAME, pass, fail).test_and_snapshot();
}
