use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, utils::is_same_reference, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-logical-operator-over-ternary): Prefer using a logical operator over a ternary.")]
#[diagnostic(severity(warning), help("Switch to \"||\" or \"??\" operator"))]
struct PreferLogicalOperatorOverTernaryDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferLogicalOperatorOverTernary;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule finds ternary expressions that can be simplified to a logical operator.
    ///
    /// ### Why is this bad?
    ///
    /// Using a logical operator is shorter and simpler than a ternary expression.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// // Bad
    /// const foo = bar ? bar : baz;
    /// console.log(foo ? foo : bar);
    ///
    /// // Good
    /// const foo = bar || baz;
    /// console.log(foo ?? bar);
    ///
    /// ```
    PreferLogicalOperatorOverTernary,
    style
);

impl Rule for PreferLogicalOperatorOverTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(conditional_expression) = node.kind() else { return };

        if is_same_node(&conditional_expression.test, &conditional_expression.consequent, ctx) {
            ctx.diagnostic(PreferLogicalOperatorOverTernaryDiagnostic(conditional_expression.span));
        }

        // `!bar ? foo : bar;`
        if let Expression::UnaryExpression(unary_expression) = &conditional_expression.test {
            if unary_expression.operator == UnaryOperator::LogicalNot
                && is_same_node(&unary_expression.argument, &conditional_expression.alternate, ctx)
            {
                ctx.diagnostic(PreferLogicalOperatorOverTernaryDiagnostic(
                    conditional_expression.span,
                ));
            }
        }
    }
}

fn is_same_node(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    if is_same_reference(left, right, ctx) {
        return true;
    }

    match (left, right) {
        (
            Expression::AwaitExpression(left_await_expr),
            Expression::AwaitExpression(right_await_expr),
        ) => return is_same_node(&left_await_expr.argument, &right_await_expr.argument, ctx),
        (
            Expression::LogicalExpression(left_await_expr),
            Expression::LogicalExpression(right_await_expr),
        ) => {
            return is_same_node(&left_await_expr.left, &right_await_expr.left, ctx)
                && is_same_node(&left_await_expr.right, &right_await_expr.right, ctx)
        }
        (
            Expression::UnaryExpression(left_await_expr),
            Expression::UnaryExpression(right_await_expr),
        ) => return is_same_node(&left_await_expr.argument, &right_await_expr.argument, ctx),
        (Expression::UpdateExpression(_), Expression::UpdateExpression(_)) => return false,
        (Expression::ParenthesizedExpression(left_paren_expr), _) => {
            return is_same_node(&left_paren_expr.expression, right, ctx)
        }
        (_, Expression::ParenthesizedExpression(right_paren_expr)) => {
            return is_same_node(left, &right_paren_expr.expression, ctx);
        }
        _ => {}
    }

    left.span().source_text(ctx.source_text()) == right.span().source_text(ctx.source_text())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo ? foo1 : bar;",
        "foo.bar ? foo.bar1 : foo.baz",
        "foo.bar ? foo1.bar : foo.baz",
        "++foo ? ++foo : bar;",
        "!!bar ? foo : bar;",
    ];

    let fail = vec![
        "foo ? foo : bar;",
        "foo.bar ? foo.bar : foo.baz",
        "foo?.bar ? foo.bar : baz",
        "foo?.bar ? foo?.bar : baz",
        "!bar ? foo : bar;",
        "!!bar ? foo : !bar;",
        "foo() ? foo() : bar",
        "foo ? foo : a && b",
        "foo ? foo : a || b",
        "foo ? foo : a ?? b",
        "a && b ? a && b : bar",
        "a || b ? a || b : bar",
        "a ?? b ? a ?? b : bar",
        "foo ? foo : await a",
        "await a ? await a : foo",
        "await a ? (await (a)) : (foo)",
        "(await a) ? await (a) : (foo)",
        "(await a) ? (await (a)) : (foo)",
    ];

    Tester::new_without_config(PreferLogicalOperatorOverTernary::NAME, pass, fail)
        .test_and_snapshot();
}
