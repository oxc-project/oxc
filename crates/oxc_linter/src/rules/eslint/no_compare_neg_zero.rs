use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_compare_neg_zero_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "eslint(no-compare-neg-zero): Do not use the {x0} operator to compare against -0."
    ))
    .with_help("Use Object.is(x, -0) to test equality with -0 and use 0 for other cases")
    .with_labels([span1.into()])
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
    correctness
);

impl Rule for NoCompareNegZero {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };
        if Self::should_check(expr.operator) {
            let op = expr.operator.as_str();
            if is_neg_zero(&expr.left) || is_neg_zero(&expr.right) {
                ctx.diagnostic(no_compare_neg_zero_diagnostic(op, expr.span));
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

    Tester::new(NoCompareNegZero::NAME, pass, fail).test_and_snapshot();
}
