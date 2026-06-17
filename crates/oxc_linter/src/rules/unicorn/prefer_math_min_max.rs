use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, BinaryOperator, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_same_expression};

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
    /// Prefers use of `Math.min()` and `Math.max()` instead of ternary
    /// expressions when performing simple comparisons.
    ///
    /// ### Why is this bad?
    ///
    /// Using `Math.min()` and `Math.max()` for simple comparisons is more
    /// concise, easier to understand, and less prone to errors than ternary
    /// expressions. They clearly express the intent to find the minimum or
    /// maximum value.
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
    unicorn,
    pedantic,
    fix,
    version = "0.10.1",
    short_description = "Prefers use of `Math.min()` and `Math.max()` instead of ternary expressions when performing simple comparisons.",
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
            // Emit the operands' original source text rather than reconstructing them, so word
            // operators keep their spacing (`typeof x`, not `typeofx`) and member operands aren't
            // dropped (`delete obj.p`, not `delete`). Matches eslint-plugin-unicorn, which builds
            // the fix from `sourceCode.getText`.
            let consequent = ctx.source_range(conditional_expr.consequent.span());
            let alternate = ctx.source_range(conditional_expr.alternate.span());
            let method = match condition_type {
                TypeOptions::Max => "max",
                TypeOptions::Min => "min",
                TypeOptions::Unknown => unreachable!(),
            };
            fixer
                .replace(conditional_expr.span, format!("Math.{method}({consequent}, {alternate})"))
        });
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
        (r"const foo = height < 100 ? height : 100;", r"const foo = Math.min(height, 100);"),
        (r"const foo = height <= -100 ? height : -100;", r"const foo = Math.min(height, -100);"),
        (
            r"const foo = 150.34 < height ? 150.34 : height;",
            r"const foo = Math.min(150.34, height);",
        ),
        (r"const foo = -0.34 <= height ? -0.34 : height;", r"const foo = Math.min(-0.34, height);"),
        (r"const foo = height > 10e3 ? 10e3 : height;", r"const foo = Math.min(10e3, height);"),
        (r"const foo = height >= -10e3 ? -10e3 : height;", r"const foo = Math.min(-10e3, height);"),
        (r"const foo = 10e3 > height ? height : 10e3;", r"const foo = Math.min(height, 10e3);"),
        (r"const foo = 10e3 >= height ? height : 10e3;", r"const foo = Math.min(height, 10e3);"),
        ("return height > 100 ? height : 100;", "return Math.max(height, 100);"),
        ("return height >= 50 ? height : 50;", "return Math.max(height, 50);"),
        (
            "return (10e3 > height ? 10e3 : height) || 200;",
            "return (Math.max(10e3, height)) || 200;",
        ),
        (
            "return (-10e3 >= height ? -10e3 : height) || 200;",
            "return (Math.max(-10e3, height)) || 200;",
        ),
        (
            "return (height < 2.99 ? 2.99 : height) || 0.99;",
            "return (Math.max(2.99, height)) || 0.99;",
        ),
        (
            "return (height <= -0.99 ? -0.99 : height) || -3.99;",
            "return (Math.max(-0.99, height)) || -3.99;",
        ),
        ("return 10e6 < height ? height : 10e6;", "return Math.max(height, 10e6);"),
        ("return -10e4 <= height ? height : -10e4;", "return Math.max(height, -10e4);"),
        ("const foo = typeof x < bar ? typeof x : bar;", "const foo = Math.min(typeof x, bar);"),
        ("const foo = void x < bar ? void x : bar;", "const foo = Math.min(void x, bar);"),
        (
            "const foo = delete obj.p < bar ? delete obj.p : bar;",
            "const foo = Math.min(delete obj.p, bar);",
        ),
    ];

    Tester::new(PreferMathMinMax::NAME, PreferMathMinMax::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
