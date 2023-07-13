use oxc_ast::{
    ast::{BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(eqeqeq): Expected {1} and instead saw {0}")]
#[diagnostic(severity(warning), help("Prefer strict {1} operator"))]
struct EqEqEqDiagnostic(&'static str, &'static str, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct EqEqEq;

declare_oxc_lint!(
    /// ### What it does
    /// Requires the use of the === and !== operators
    ///
    /// ### Why is this bad?
    /// Using non-strict equality operators leads to hard to track bugs due to type coercion.
    ///
    /// ### Example
    /// ```javascript
    /// let a = []
    /// let b = false
    /// a == b
    /// ```
    EqEqEq,
    pedantic
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
        let AstKind::BinaryExpression(binary_expr) = node.kind() else { return };
        if !matches!(binary_expr.operator, BinaryOperator::Equality | BinaryOperator::Inequality) {
            return;
        }

        let is_valid_comparison = match (&binary_expr.left, &binary_expr.right) {
            (Expression::UnaryExpression(unary_expr), _)
            | (_, Expression::UnaryExpression(unary_expr)) => {
                matches!(unary_expr.operator, UnaryOperator::Typeof)
            }
            (lhs, rhs) => {
                lhs.is_null()
                    || rhs.is_null()
                    || (lhs.is_literal_expression() && rhs.is_literal_expression())
            }
        };

        if !is_valid_comparison {
            let operator = binary_expr.operator.as_str();
            let preferred_operator = to_strict_operator(binary_expr.operator).as_str();
            // If the comparison is a `typeof` comparison or both sides are literals with the same type, then it's safe to fix.
            if is_type_of_binary(binary_expr)
                || are_literals_and_same_type(&binary_expr.left, &binary_expr.right)
            {
                ctx.diagnostic_with_fix(
                    EqEqEqDiagnostic(operator, preferred_operator, binary_expr.span),
                    || {
                        let start = binary_expr.left.span().end;
                        let end = binary_expr.right.span().start;
                        Fix::new(preferred_operator, Span { start, end })
                    },
                );
            } else {
                ctx.diagnostic(EqEqEqDiagnostic(operator, preferred_operator, binary_expr.span));
            }
        }
    }
}

/// Checks if either operand of a binary expression is a typeof operation
fn is_type_of_binary(binary_expr: &BinaryExpression) -> bool {
    match (&binary_expr.left, &binary_expr.right) {
        (Expression::UnaryExpression(unary_expr), _)
        | (_, Expression::UnaryExpression(unary_expr)) => {
            matches!(unary_expr.operator, UnaryOperator::Typeof)
        }
        _ => false,
    }
}

/// Checks if operands are literals of the same type
fn are_literals_and_same_type(left: &Expression, right: &Expression) -> bool {
    matches!((left, right), (Expression::BooleanLiteral(_), Expression::BooleanLiteral(_))
        | (Expression::NullLiteral(_), Expression::NullLiteral(_))
        | (Expression::StringLiteral(_), Expression::StringLiteral(_))
        | (Expression::NumberLiteral(_), Expression::NumberLiteral(_))
        | (Expression::BigintLiteral(_), Expression::BigintLiteral(_))
        | (Expression::RegExpLiteral(_), Expression::RegExpLiteral(_))
        | (Expression::TemplateLiteral(_), Expression::TemplateLiteral(_)))
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

    let fix =
        vec![("null==null", "null===null", None), 
        ("'foo'=='foo'", "'123'==='123'", None), 
        ("typeof a == b", "typeof a===b", None),
        ("1000  !=  1000", "1000!==1000", None),
        // The following cases will not be fixed
        ("(1000 + 1)  !=  1000", "(1000 + 1)  !=  1000", None),
        ("a == b", "a == b", None) 
    ];

    let mut tester = Tester::new(EqEqEq::NAME, pass, fail);
    tester.test_and_snapshot();
    tester.test_fix(fix);
}
