use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_same_expression};

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = bar ? bar : baz;
    /// console.log(foo ? foo : bar);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = bar || baz;
    /// console.log(foo ?? bar);
    /// ```
    PreferLogicalOperatorOverTernary,
    unicorn,
    style,
    suggestion,
);

impl Rule for PreferLogicalOperatorOverTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(conditional_expression) = node.kind() else {
            return;
        };

        // `foo ? foo : bar` -> `foo || bar`
        if is_same_node(&conditional_expression.test, &conditional_expression.consequent, ctx) {
            let test_expr = preferred_test_expr(
                &conditional_expression.test,
                &conditional_expression.consequent,
            );
            let test_text =
                wrap_nullish_coalesce_operand(test_expr, ctx.source_range(test_expr.span()));
            let alternate_text = wrap_nullish_coalesce_operand(
                &conditional_expression.alternate,
                ctx.source_range(conditional_expression.alternate.span()),
            );
            let span = conditional_expression.span;

            ctx.diagnostic_with_suggestion(
                prefer_logical_operator_over_ternary_diagnostic(span),
                |fixer| fixer.replace(span, format!("{test_text} || {alternate_text}")),
            );
            return;
        }

        // `!bar ? foo : bar` -> `bar || foo`
        if let Expression::UnaryExpression(unary_expression) = &conditional_expression.test
            && unary_expression.operator == UnaryOperator::LogicalNot
            && is_same_node(&unary_expression.argument, &conditional_expression.alternate, ctx)
        {
            let alternate_expr = preferred_alternate_expr(
                &conditional_expression.test,
                &conditional_expression.alternate,
                ctx,
            );
            let alternate_text = wrap_nullish_coalesce_operand(
                alternate_expr,
                ctx.source_range(alternate_expr.span()),
            );
            let consequent_text = wrap_nullish_coalesce_operand(
                &conditional_expression.consequent,
                ctx.source_range(conditional_expression.consequent.span()),
            );
            let span = conditional_expression.span;

            ctx.diagnostic_with_suggestion(
                prefer_logical_operator_over_ternary_diagnostic(span),
                |fixer| fixer.replace(span, format!("{alternate_text} || {consequent_text}")),
            );
        }
    }
}

fn preferred_test_expr<'a>(
    test: &'a Expression<'a>,
    consequent: &'a Expression<'a>,
) -> &'a Expression<'a> {
    if matches!(test, Expression::ParenthesizedExpression(_)) {
        return consequent.get_inner_expression();
    }

    test.get_inner_expression()
}

fn preferred_alternate_expr<'a>(
    test: &'a Expression<'a>,
    alternate: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> &'a Expression<'a> {
    let Expression::UnaryExpression(outer_unary) = test else {
        return alternate;
    };
    if outer_unary.operator != UnaryOperator::LogicalNot {
        return alternate;
    }
    if !is_same_node(&outer_unary.argument, alternate, ctx) {
        return alternate;
    }
    let Expression::UnaryExpression(inner_unary) = &outer_unary.argument else {
        return alternate;
    };
    if inner_unary.operator != UnaryOperator::LogicalNot {
        return alternate;
    }

    &inner_unary.argument
}

fn wrap_nullish_coalesce_operand(expr: &Expression, text: &str) -> String {
    if matches!(expr, Expression::ParenthesizedExpression(_)) {
        return text.to_string();
    }

    match expr.without_parentheses() {
        Expression::LogicalExpression(logical_expr) if logical_expr.operator.is_coalesce() => {
            format!("({text})")
        }
        _ => text.to_string(),
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
        "const foo = [];
         !+a ? b : +a",
        "const foo = [];
         a && b ? a && b : 1",
    ];

    let fix = vec![
        ("foo ? foo : bar;", "foo || bar;"),
        ("foo.bar ? foo.bar : foo.baz", "foo.bar || foo.baz"),
        ("foo?.bar ? foo.bar : baz", "foo?.bar || baz"),
        ("foo?.bar ? foo?.bar : baz", "foo?.bar || baz"),
        ("!bar ? foo : bar;", "bar || foo;"),
        ("!!bar ? foo : !bar;", "bar || foo;"),
        ("foo() ? foo() : bar", "foo() || bar"),
        ("foo ? foo : a && b", "foo || a && b"),
        ("foo ? foo : a || b", "foo || a || b"),
        ("foo ? foo : a ?? b", "foo || (a ?? b)"),
        ("a && b ? a && b : bar", "a && b || bar"),
        ("a || b ? a || b : bar", "a || b || bar"),
        ("a ?? b ? a ?? b : bar", "(a ?? b) || bar"),
        ("foo ? foo : await a", "foo || await a"),
        ("await a ? await a : foo", "await a || foo"),
        ("await a ? (await (a)) : (foo)", "await a || (foo)"),
        ("(await a) ? await (a) : (foo)", "await (a) || (foo)"),
        ("(await a) ? (await (a)) : (foo)", "await (a) || (foo)"),
    ];

    Tester::new(
        PreferLogicalOperatorOverTernary::NAME,
        PreferLogicalOperatorOverTernary::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
