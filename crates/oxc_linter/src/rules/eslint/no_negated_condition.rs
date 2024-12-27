use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::ast::{ConditionalExpression, Expression, IfStatement, Statement};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

fn no_negated_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated condition.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNegatedCondition;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows the use of negated conditions in `if` statements to improve readability.
    ///
    /// ### Why is this bad?
    ///
    /// Negated conditions can make code harder to read and understand, especially in complex logic.
    /// It is often clearer to use positive conditions or to refactor the code structure.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (!isReady) {
    ///   doSomething();
    /// } else {
    ///   doSomethingElse();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (isReady) {
    ///   doSomethingElse();
    /// } else {
    ///   doSomething();
    /// }
    /// ```
    NoNegatedCondition,
    style,
    pending,
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                if !has_else_without_condition(if_stmt) {
                    return;
                }

                if is_negated_if(if_stmt) {
                    ctx.diagnostic(no_negated_condition_diagnostic(if_stmt.span()));
                }
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                if is_negated_if_conditional(conditional_expr) {
                    ctx.diagnostic(no_negated_condition_diagnostic(conditional_expr.span()));
                }
            }
            _ => {}
        }
    }
}

fn has_else_without_condition(node: &IfStatement) -> bool {
    matches!(node.alternate, Some(Statement::BlockStatement(_)))
}

fn is_negated_unary_expression(test: &Expression) -> bool {
    matches!(test, Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot)
}

fn is_negated_binary_expression(test: &Expression) -> bool {
    matches!(
        test,
        Expression::BinaryExpression(binary)
            if binary.operator == BinaryOperator::Inequality
                || binary.operator == BinaryOperator::StrictInequality
    )
}

fn is_negated_if(node: &IfStatement) -> bool {
    is_negated_unary_expression(&node.test) || is_negated_binary_expression(&node.test)
}

fn is_negated_if_conditional(node: &ConditionalExpression) -> bool {
    is_negated_unary_expression(&node.test) || is_negated_binary_expression(&node.test)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (a) {}",
        "if (a) {} else {}",
        "if (!a) {}",
        "if (!a) {} else if (b) {}",
        "if (!a) {} else if (b) {} else {}",
        "if (a == b) {}",
        "if (a == b) {} else {}",
        "if (a != b) {}",
        "if (a != b) {} else if (b) {}",
        "if (a != b) {} else if (b) {} else {}",
        "if (a !== b) {}",
        "if (a === b) {} else {}",
        "a ? b : c",
    ];

    let fail = vec![
        "if (!a) {;} else {;}",
        "if (a != b) {;} else {;}",
        "if (a !== b) {;} else {;}",
        "!a ? b : c",
        "a != b ? c : d",
        "a !== b ? c : d",
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::CATEGORY, pass, fail)
        .test_and_snapshot();
}
