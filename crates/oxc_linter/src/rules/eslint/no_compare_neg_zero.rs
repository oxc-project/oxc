use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_compare_neg_zero_diagnostic(operator: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use the {operator} operator to compare against -0."))
        .with_help("Use Object.is(x, -0) to test equality with -0 and use 0 for other cases")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCompareNegZero;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow comparing against -0
    ///
    /// ### Why is this bad?
    /// The rule should warn against code that tries to compare against -0,
    /// since that will not work as intended. That is, code like x === -0 will
    /// pass for both +0 and -0. The author probably intended Object.is(x, -0).
    ///
    /// ### Example
    /// ```javascript
    /// if (x === -0) {}
    /// ```
    NoCompareNegZero,
    correctness,
    conditional_fix_suggestion
);

impl Rule for NoCompareNegZero {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };
        if Self::should_check(expr.operator) {
            let op = expr.operator.as_str();
            let is_left_neg_zero = is_neg_zero(&expr.left);
            let is_right_neg_zero = is_neg_zero(&expr.right);
            if is_left_neg_zero || is_right_neg_zero {
                if expr.operator == BinaryOperator::StrictEquality {
                    ctx.diagnostic_with_suggestion(
                        no_compare_neg_zero_diagnostic(op, expr.span),
                        |fixer| {
                            // replace `x === -0` with `Object.is(x, -0)`
                            let value = if is_left_neg_zero {
                                ctx.source_range(expr.right.span())
                            } else {
                                ctx.source_range(expr.left.span())
                            };
                            fixer.replace(expr.span, format!("Object.is({value}, -0)"))
                        },
                    );
                } else {
                    // <https://tc39.es/ecma262/#%E2%84%9D>
                    // <https://tc39.es/ecma262/#sec-numeric-types-number-lessThan>
                    // The mathematical value of +0𝔽 and -0𝔽 is the mathematical value 0.
                    // It's safe to replace -0 with 0
                    ctx.diagnostic_with_fix(
                        no_compare_neg_zero_diagnostic(op, expr.span),
                        |fixer| {
                            let start = if is_left_neg_zero {
                                expr.left.span().start
                            } else {
                                expr.right.span().start
                            };
                            let end = start + 1;
                            let span = Span::new(start, end);
                            fixer.delete(&span)
                        },
                    );
                }
            }
        }
    }
}

impl NoCompareNegZero {
    fn should_check(operator: BinaryOperator) -> bool {
        operator.is_compare() || operator.is_equality()
    }
}

fn is_neg_zero(expr: &Expression) -> bool {
    let Expression::UnaryExpression(unary) = expr.get_inner_expression() else {
        return false;
    };
    if unary.operator != UnaryOperator::UnaryNegation {
        return false;
    }
    match &unary.argument {
        Expression::NumericLiteral(number) => number.value == 0.0,
        Expression::BigIntLiteral(bigint) => bigint.is_zero(),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("x === 0", None),
        ("0 === x", None),
        ("x == 0", None),
        ("0 == x", None),
        ("x === '0'", None),
        ("'0' === x", None),
        ("x == '0'", None),
        ("'0' == x", None),
        ("x === '-0'", None),
        ("'-0' === x", None),
        ("x == '-0'", None),
        ("'-0' == x", None),
        ("x === -1", None),
        ("-1 === x", None),
        ("x < 0", None),
        ("0 < x", None),
        ("x <= 0", None),
        ("0 <= x", None),
        ("x > 0", None),
        ("0 > x", None),
        ("x >= 0", None),
        ("0 >= x", None),
        ("x != 0", None),
        ("0 != x", None),
        ("x !== 0", None),
        ("0 !== x", None),
        ("Object.is(x, -0)", None),
    ];

    let fail = vec![
        ("x === -0", None),
        ("-0 === x", None),
        ("x == -0", None),
        ("-0 == x", None),
        ("x > -0", None),
        ("-0 > x", None),
        ("x >= -0", None),
        ("-0 >= x", None),
        ("x < -0", None),
        ("-0 < x", None),
        ("x <= -0", None),
        ("-0 <= x", None),
        // BigInt Literal
        ("-0n <= x", None),
    ];

    let fix = vec![
        ("x === -0", "Object.is(x, -0)", None),
        ("-0 === x", "Object.is(x, -0)", None),
        ("x == -0", "x == 0", None),
        ("-0 == x", "0 == x", None),
        ("x > -0", "x > 0", None),
        ("-0 > x", "0 > x", None),
        ("x >= -0", "x >= 0", None),
        ("-0 >= x", "0 >= x", None),
        ("x < -0", "x < 0", None),
        ("-0 < x", "0 < x", None),
        ("x <= -0", "x <= 0", None),
        ("-0 <= x", "0 <= x", None),
        ("-0n <= x", "0n <= x", None),
    ];

    Tester::new(NoCompareNegZero::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
