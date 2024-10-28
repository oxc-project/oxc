use std::borrow::Cow;

use oxc_ast::{
    ast::{BinaryExpression, BinaryOperator, Expression, UnaryOperator},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_same_reference, AstNode};

fn prefer_math_min_max_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `Math.min()` and `Math.max()` over ternaries for simple comparisons.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferMathMinMax;

#[derive(Debug)]
enum TypeOptions {
    Min,
    Max,
    Unknown,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers use of [`Math.min()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/min)
    /// and [`Math.max()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/max) instead of
    /// ternary expressions when performing simple comparisons for more concise, easier to understand, and less prone to errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// height > 50 ? 50 : height;
    /// height > 50 ? height : 50;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// Math.min(height, 50);
    /// Math.max(height, 50);
    /// ```
    PreferMathMinMax,
    pedantic,
    fix
);

impl Rule for PreferMathMinMax {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(conditional_expr) = node.kind() else {
            return;
        };

        let Expression::BinaryExpression(test_expr) = &conditional_expr.test else {
            return;
        };

        let condition_type =
            is_min_max(test_expr, &conditional_expr.consequent, &conditional_expr.alternate, ctx);

        if matches!(condition_type, TypeOptions::Unknown) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_math_min_max_diagnostic(conditional_expr.span), |fixer| {
            let Some(consequent) = get_expr_value(&conditional_expr.consequent) else {
                return fixer.noop();
            };
            let Some(alternate) = get_expr_value(&conditional_expr.alternate) else {
                return fixer.noop();
            };

            match condition_type {
                TypeOptions::Max => fixer.replace(
                    conditional_expr.span,
                    Cow::Owned(format!("Math.max({consequent}, {alternate})")),
                ),
                TypeOptions::Min => fixer.replace(
                    conditional_expr.span,
                    Cow::Owned(format!("Math.min({consequent}, {alternate})")),
                ),
                TypeOptions::Unknown => unreachable!(),
            }
        });
    }
}

fn get_expr_value(expr: &Expression) -> Option<String> {
    match expr {
        Expression::NumericLiteral(lit) => Some(lit.to_string()),
        Expression::UnaryExpression(lit) => {
            let mut unary_str: String = String::from(lit.operator.as_str());

            let Some(unary_lit) = get_expr_value(&lit.argument) else {
                return Some(unary_str.to_string());
            };

            unary_str.push_str(unary_lit.as_str());

            Some(unary_str.to_string())
        }
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}

fn is_same_expression(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    if is_same_reference(left, right, ctx) {
        return true;
    }

    match (left, right) {
        (Expression::UnaryExpression(left_expr), Expression::UnaryExpression(right_expr)) => {
            left_expr.operator == UnaryOperator::UnaryNegation
                && right_expr.operator == UnaryOperator::UnaryNegation
                && is_same_reference(&left_expr.argument, &right_expr.argument, ctx)
        }
        _ => false,
    }
}

fn is_min_max(
    condition: &BinaryExpression,
    consequent: &Expression,
    alternate: &Expression,
    ctx: &LintContext,
) -> TypeOptions {
    let is_matched = matches!(
        (condition.left.get_inner_expression(), condition.right.get_inner_expression()),
        (Expression::NumericLiteral(_) | Expression::UnaryExpression(_), Expression::Identifier(_))
            | (
                Expression::Identifier(_),
                Expression::NumericLiteral(_) | Expression::UnaryExpression(_)
            )
    );

    if !condition.operator.is_compare() || !is_matched {
        return TypeOptions::Unknown;
    }

    if is_same_expression(&condition.left, consequent, ctx)
        && is_same_expression(&condition.right, alternate, ctx)
    {
        if matches!(condition.operator, BinaryOperator::LessThan | BinaryOperator::LessEqualThan) {
            return TypeOptions::Min;
        }
        return TypeOptions::Max;
    } else if is_same_expression(&condition.left, alternate, ctx)
        && is_same_expression(&condition.right, consequent, ctx)
    {
        if matches!(condition.operator, BinaryOperator::LessThan | BinaryOperator::LessEqualThan) {
            return TypeOptions::Max;
        }
        return TypeOptions::Min;
    }

    TypeOptions::Unknown
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = height ? height : 50;",
        r"const foo = height == 'success' ? height : 'failed';",
        r"Math.min(height, 50);",
        r"Math.max(height, 50);",
        r"Math.min(50, height);",
        r"Math.max(-50, height);",
        r"const foo = height < 50n ? height : 50n;",
    ];

    let fail: Vec<&str> = vec![
        r"const foo = height < 50 ? height : 50;",
        r"const foo = height <= -50 ? height : -50;",
        r"const foo = 150.34 < height ? 150.34 : height;",
        r"const foo = -2.34 <= height ? -2.34 : height;",
        r"const foo = height > 10e3 ? 10e3 : height;",
        r"const foo = height >= -10e3 ? -10e3 : height;",
        r"const foo = 50 > height ? height : 50;",
        r"const foo = 50 >= height ? height : 50;",
        r"return height > 100 ? height : 100;",
        r"return height >= 50 ? height : 50;",
        r"const foo = (50 > height ? 50 : height) || 0;",
        r"const foo = (50 >= height ? 50 : height) || 0;",
        r"return (height < 50 ? 50 : height) || 100;",
        r"return (height <= 50 ? 50 : height) || 200;",
        r"const foo = 50 < height ? height: 50;",
        r"const foo = 50 <= height ? height: 50;",
    ];

    let fix = vec![
        (r"const foo = height < 100 ? height : 100;", r"const foo = Math.min(height, 100);", None),
        (
            r"const foo = height <= -100 ? height : -100;",
            r"const foo = Math.min(height, -100);",
            None,
        ),
        (
            r"const foo = 150.34 < height ? 150.34 : height;",
            r"const foo = Math.min(150.34, height);",
            None,
        ),
        (
            r"const foo = -0.34 <= height ? -0.34 : height;",
            r"const foo = Math.min(-0.34, height);",
            None,
        ),
        (
            r"const foo = height > 10e3 ? 10e3 : height;",
            r"const foo = Math.min(10e3, height);",
            None,
        ),
        (
            r"const foo = height >= -10e3 ? -10e3 : height;",
            r"const foo = Math.min(-10e3, height);",
            None,
        ),
        (
            r"const foo = 10e3 > height ? height : 10e3;",
            r"const foo = Math.min(height, 10e3);",
            None,
        ),
        (
            r"const foo = 10e3 >= height ? height : 10e3;",
            r"const foo = Math.min(height, 10e3);",
            None,
        ),
        ("return height > 100 ? height : 100;", "return Math.max(height, 100);", None),
        ("return height >= 50 ? height : 50;", "return Math.max(height, 50);", None),
        (
            "return (10e3 > height ? 10e3 : height) || 200;",
            "return (Math.max(10e3, height)) || 200;",
            None,
        ),
        (
            "return (-10e3 >= height ? -10e3 : height) || 200;",
            "return (Math.max(-10e3, height)) || 200;",
            None,
        ),
        (
            "return (height < 2.99 ? 2.99 : height) || 0.99;",
            "return (Math.max(2.99, height)) || 0.99;",
            None,
        ),
        (
            "return (height <= -0.99 ? -0.99 : height) || -3.99;",
            "return (Math.max(-0.99, height)) || -3.99;",
            None,
        ),
        ("return 10e6 < height ? height : 10e6;", "return Math.max(height, 10e6);", None),
        ("return -10e4 <= height ? height : -10e4;", "return Math.max(height, -10e4);", None),
    ];

    Tester::new(PreferMathMinMax::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
