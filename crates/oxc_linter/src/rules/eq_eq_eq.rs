use oxc_ast::{
    ast::{BinaryOperator, Expression},
    AstKind, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{autofix::Fix, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(eqeqeq): Require the use of === and !==")]
#[diagnostic(severity(warning), help("Prefer strict (in)equality operator"))]
struct EqEqEqDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct EqEqEq;

declare_oxc_lint!(
    /// ### What it does
    /// Requires the use of the === and !== operators
    ///
    /// ### Why is this bad?
    /// Using non-strict equality operators leads tricky bugs due to type coercion.
    ///
    /// ### Example
    /// ```javascript
    /// let a = []
    /// let b = false
    /// a == b
    /// ```
    EqEqEq
);

fn to_strict_operator(operator: BinaryOperator) -> BinaryOperator {
    match operator {
        BinaryOperator::Equality => BinaryOperator::StrictEquality,
        BinaryOperator::Inequality => BinaryOperator::StrictInequality,
        _ => unreachable!(),
    }
}

impl Rule for EqEqEq {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.get().kind() else { return };
        if !matches!(binary_expr.operator, BinaryOperator::Equality | BinaryOperator::Inequality) {
            return;
        }

        let is_valid_comparison = match (&binary_expr.left, &binary_expr.right) {
            (Expression::UnaryExpression(unary_expr), _)
            | (_, Expression::UnaryExpression(unary_expr)) => {
                unary_expr.operator.is_keyword() && unary_expr.operator.as_str() == "typeof"
            }
            (lhs, rhs) => {
                (lhs.is_null() || rhs.is_null())
                    || lhs.is_literal_expression() && rhs.is_literal_expression()
            }
        };

        if !is_valid_comparison {
            ctx.diagnostic(EqEqEqDiagnostic(binary_expr.span));
            ctx.fix(Fix::new(to_strict_operator(binary_expr.operator).as_str(), binary_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("typeof foo == 'undefined'", None),
        ("'hello' != 'world'", None),
        ("0 == 0", None),
        ("true == true", None),
        ("foo == null", None),
    ];

    let fail = vec![
        ("a == b", None),
        ("foo == true", None),
        ("bananas != 1", None),
        ("value == undefined", None),
    ];

    Tester::new(EqEqEq::NAME, pass, fail).test_and_snapshot();
}
