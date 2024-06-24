use std::fmt::Debug;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_eq_null_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-eq-null): Use '===' to compare with null")
        .with_help("Disallow `null` comparisons without type-checking operators.")
        .with_labels([span0.into()])
}

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
                ctx.diagnostic(no_eq_null_diagnostic(Span::new(
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
