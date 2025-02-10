use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, utils::is_same_expression, AstNode};

fn prefer_logical_operator_over_ternary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using a logical operator over a ternary.")
        .with_help("Switch to \"||\" or \"??\" operator")
        .with_label(span)
}

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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = bar ? bar : baz;
    /// console.log(foo ? foo : bar);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    //  ```javascript
    /// const foo = bar || baz;
    /// console.log(foo ?? bar);
    ///
    /// ```
    PreferLogicalOperatorOverTernary,
    unicorn,
    style,
    pending
);

impl Rule for PreferLogicalOperatorOverTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(conditional_expression) = node.kind() else {
            return;
        };

        if is_same_node(&conditional_expression.test, &conditional_expression.consequent, ctx) {
            ctx.diagnostic(prefer_logical_operator_over_ternary_diagnostic(
                conditional_expression.span,
            ));
        }

        // `!bar ? foo : bar;`
        if let Expression::UnaryExpression(unary_expression) = &conditional_expression.test {
            if unary_expression.operator == UnaryOperator::LogicalNot
                && is_same_node(&unary_expression.argument, &conditional_expression.alternate, ctx)
            {
                ctx.diagnostic(prefer_logical_operator_over_ternary_diagnostic(
                    conditional_expression.span,
                ));
            }
        }
    }
}

fn is_same_node(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    if is_same_expression(left, right, ctx) {
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
                && is_same_node(&left_await_expr.right, &right_await_expr.right, ctx);
        }
        (
            Expression::UnaryExpression(left_await_expr),
            Expression::UnaryExpression(right_await_expr),
        ) => return is_same_node(&left_await_expr.argument, &right_await_expr.argument, ctx),
        (Expression::UpdateExpression(_), Expression::UpdateExpression(_)) => return false,
        (Expression::ParenthesizedExpression(left_paren_expr), _) => {
            return is_same_node(&left_paren_expr.expression, right, ctx);
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

    Tester::new(
        PreferLogicalOperatorOverTernary::NAME,
        PreferLogicalOperatorOverTernary::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
