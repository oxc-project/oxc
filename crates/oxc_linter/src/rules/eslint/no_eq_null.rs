use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;
use std::fmt::Debug;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-eq-null): Use '===' to compare with null")]
#[diagnostic(
    severity(warning),
    help("Disallow `null` comparisons without type-checking operators.")
)]
struct NoEqNullDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEqNull;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow null comparisons without type-checking operators.
    ///
    /// ### Why is this bad?
    /// Comparing to null without a type-checking operator (== or !=), can have unintended results as the comparison will evaluate to true when comparing to not just a null, but also an undefined value.
    ///
    /// ### Example
    /// ```javascript
    /// if (foo == null) {
    ///   bar();
    /// }
    /// ```
    NoEqNull,
    restriction
);

impl Rule for NoEqNull {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::BinaryExpression(binary_expression) = node.kind() {
            let bad_operator = matches!(
                binary_expression.operator,
                BinaryOperator::Equality | BinaryOperator::Inequality
            );

            if binary_expression.right.is_literal()
                & binary_expression.right.is_null()
                & bad_operator
                | binary_expression.left.is_literal()
                    & binary_expression.left.is_null()
                    & bad_operator
            {
                ctx.diagnostic(NoEqNullDiagnostic(Span::new(
                    binary_expression.span.start,
                    binary_expression.span.end,
                )));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["if (x === null) { }", "if (null === f()) { }"];

    let fail = vec!["if (x == null) { }", "if (x != null) { }", "do {} while (null == x)"];

    Tester::new(NoEqNull::NAME, pass, fail).test_and_snapshot();
}
