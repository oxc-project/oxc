// Based on https://github.com/rust-lang/rust-clippy//blob/6246f0446afbe9abff18e8cc1ebaae7505f7cd9e/clippy_lints/src/operators/const_comparisons.rs
use std::cmp::Ordering;

use oxc_ast::{
    ast::{Expression, LogicalExpression, NumericLiteral, UnaryOperator},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, utils::is_same_expression, AstNode};

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

fn constant_comparison_diagnostic(span: Span, evaluates_to: bool, help: String) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("This comparison will always evaluate to {evaluates_to}"))
        .with_help(help)
        .with_label(span)
}

fn identical_expressions_logical_operator(left_span: Span, right_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Both sides of the logical operator are the same")
                    .with_help("This logical expression will always evaluate to the same value as the expression itself.")
                    .with_labels([
                        left_span.label("If this expression evaluates to true"),
                        right_span
                            .label("This expression will always evaluate to true"),
                    ])
}

fn identical_expressions_logical_operator_negated(
    always_truthy: bool,
    left_span: Span,
    right_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected constant comparison")
        .with_help(format!("This logical expression will always evaluate to {always_truthy}"))
        .with_labels([
            left_span.label("If this expression evaluates to true"),
            right_span.label("This expression will never evaluate to true"),
        ])
}

/// <https://rust-lang.github.io/rust-clippy/master/index.html#/impossible>
/// <https://rust-lang.github.io/rust-clippy/master/index.html#/redundant_comparisons>
#[derive(Debug, Default, Clone)]
pub struct ConstComparisons;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for redundant or logically impossible comparisons. This includes:
    /// - Ineffective double comparisons against constants.
    /// - Impossible comparisons involving constants.
    /// - Redundant comparisons where both operands are the same (e.g., a < a).
    ///
    /// ### Why is this bad?
    ///
    /// Such comparisons can lead to confusing or incorrect logic in the program. In many cases:
    /// - Only one of the comparisons has any effect on the result, suggesting that the programmer might have made a mistake, such as flipping one of the comparison operators or using the wrong variable.
    /// - Comparisons like a < a or a >= a are always false or true respectively, making the logic redundant and potentially misleading.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// status_code <= 400 && status_code > 500;
    /// status_code < 200 && status_code <= 299;
    /// status_code > 500 && status_code >= 500;
    /// a < a; // Always false
    /// a >= a; // Always true
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// status_code >= 400 && status_code < 500;
    /// 500 <= status_code && 600 > status_code;
    /// 500 <= status_code && status_code <= 600;
    /// a < b;
    /// a <= b;
    /// ```
    ConstComparisons,
    oxc,
    correctness
);

impl Rule for ConstComparisons {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        Self::check_logical_expression(node, ctx);
        Self::check_binary_expression(node, ctx);
    }
}

impl ConstComparisons {
    fn check_logical_expression<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expr) = node.kind() else {
            return;
        };

        Self::check_logical_expression_const_literal_comparison(logical_expr, ctx);
        Self::check_redundant_logical_expression(logical_expr, ctx);
    }
}
impl ConstComparisons {
    // checks for `x < 42 && x < 42` and `x < 42 && x > 42`
    fn check_logical_expression_const_literal_comparison<'a>(
        logical_expr: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if logical_expr.operator != LogicalOperator::And {
            return;
        }

        let Some((right_cmp_op, right_expr, right_const_expr, _)) =
            comparison_to_const(logical_expr.right.get_inner_expression())
        else {
            return;
        };

        for (left_cmp_op, left_expr, left_const_expr, left_span) in
            all_and_comparison_to_const(logical_expr.left.get_inner_expression())
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

            if !is_same_expression(left_expr, right_expr, ctx) {
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
                    logical_expr.right.get_inner_expression().span(),
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

    /// checks for:
    /// ```ts
    /// a === b && b === a
    /// a === b && a !== b
    /// !a && a
    /// a && !a
    /// ```
    fn check_redundant_logical_expression<'a>(
        logical_expr: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if !matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or) {
            return;
        }

        if is_same_expression(
            logical_expr.left.get_inner_expression(),
            logical_expr.right.get_inner_expression(),
            ctx,
        ) {
            ctx.diagnostic(identical_expressions_logical_operator(
                logical_expr.left.span(),
                logical_expr.right.span(),
            ));
        }

        // if either are `!foo`, check whether it looks like `foo && !foo` or `foo || !foo`
        match (logical_expr.left.get_inner_expression(), logical_expr.right.get_inner_expression())
        {
            (Expression::UnaryExpression(negated_expr), other_expr)
            | (other_expr, Expression::UnaryExpression(negated_expr)) => {
                if negated_expr.operator == UnaryOperator::LogicalNot
                    && is_same_expression(&negated_expr.argument, other_expr, ctx)
                {
                    ctx.diagnostic(identical_expressions_logical_operator_negated(
                        matches!(logical_expr.operator, LogicalOperator::Or),
                        logical_expr.left.span(),
                        logical_expr.right.span(),
                    ));
                }
            }
            _ => {}
        }
    }

    fn check_binary_expression<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        if matches!(
            bin_expr.operator,
            BinaryOperator::LessEqualThan
                | BinaryOperator::GreaterEqualThan
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterThan
        ) && is_same_expression(
            bin_expr.left.get_inner_expression(),
            bin_expr.right.get_inner_expression(),
            ctx,
        ) {
            let is_const_truthy = matches!(
                bin_expr.operator,
                BinaryOperator::LessEqualThan | BinaryOperator::GreaterEqualThan
            );

            ctx.diagnostic(constant_comparison_diagnostic(
                bin_expr.span,
                is_const_truthy,
                format!(
                    "Because `{}` will {} be {} itself",
                    bin_expr.left.span().source_text(ctx.source_text()),
                    if is_const_truthy { "always" } else { "never" },
                    match bin_expr.operator {
                        BinaryOperator::GreaterEqualThan | BinaryOperator::LessEqualThan =>
                            "equal to",
                        BinaryOperator::LessThan => "less then",
                        BinaryOperator::GreaterThan => "greater than",
                        _ => unreachable!(),
                    },
                ),
            ));
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
            match (&bin_expr.left.get_inner_expression(), &bin_expr.right.get_inner_expression()) {
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
            let left_iter = all_and_comparison_to_const(logical_expr.left.get_inner_expression());
            let right_iter = all_and_comparison_to_const(logical_expr.right.get_inner_expression());
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
        // oxc specific
        "a < b",
        "a.b.c < b.b.c",
        "a <= b",
        "a > b",
        "a >= b",
        "class Foo { #a; #b; constructor() { this.#a = 1; }; test() { return this.#a > this.#b } }",
        "!foo && bar",
        "!foo && !bar",
        "foo || bar",
        "!foo || bar",
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
        // Oxc specific
        "a < a",
        "a <= a",
        "a > a",
        "a >= a",
        "a.b.c >= a.b.c",
        "a == b && a == b",
        "a == b || a == b",
        "!foo && !foo",
        "!foo || !foo",
        "class Foo { #a; #b; constructor() { this.#a = 1; }; test() { return this.#a > this.#a } }",
        "!foo && foo",
        "foo && !foo",
        "!foo || foo",
        "foo || !foo",
    ];

    Tester::new(ConstComparisons::NAME, ConstComparisons::PLUGIN, pass, fail).test_and_snapshot();
}
