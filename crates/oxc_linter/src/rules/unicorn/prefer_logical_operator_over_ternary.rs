use oxc_ast::{
    ast::{ChainElement, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

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
    correctness
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
        _ => {}
    }

    left.span().source_text(ctx.source_text()) == right.span().source_text(ctx.source_text())
}

fn is_same_reference(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    match (left, right) {
        (
            Expression::ChainExpression(left_chain_expr),
            Expression::MemberExpression(right_member_expr),
        ) => {
            if let ChainElement::MemberExpression(v) = &left_chain_expr.expression {
                return is_same_member_expression(v, right_member_expr, ctx);
            }
        }
        (
            Expression::MemberExpression(left_chain_expr),
            Expression::ChainExpression(right_member_expr),
        ) => {
            if let ChainElement::MemberExpression(v) = &right_member_expr.expression {
                return is_same_member_expression(left_chain_expr, v, ctx);
            }
        }

        // super // this
        (Expression::Super(_), Expression::Super(_))
        | (Expression::ThisExpression(_), Expression::ThisExpression(_))
        | (Expression::NullLiteral(_), Expression::NullLiteral(_)) => return true,

        (Expression::Identifier(left_ident), Expression::Identifier(right_ident)) => {
            return left_ident.name == right_ident.name
        }

        (Expression::StringLiteral(left_str), Expression::StringLiteral(right_str)) => {
            return left_str.value == right_str.value
        }
        (Expression::NumberLiteral(left_num), Expression::NumberLiteral(right_num)) => {
            return left_num.raw == right_num.raw
        }
        (Expression::RegExpLiteral(left_regexp), Expression::RegExpLiteral(right_regexp)) => {
            return left_regexp.regex.pattern == right_regexp.regex.pattern
                && left_regexp.regex.flags == right_regexp.regex.flags
        }
        (Expression::BooleanLiteral(left_bool), Expression::BooleanLiteral(right_bool)) => {
            return left_bool.value == right_bool.value
        }

        (
            Expression::ChainExpression(left_chain_expr),
            Expression::ChainExpression(right_chain_expr),
        ) => {
            if let ChainElement::MemberExpression(left_member_expr) = &left_chain_expr.expression {
                if let ChainElement::MemberExpression(right_member_expr) =
                    &right_chain_expr.expression
                {
                    return is_same_member_expression(left_member_expr, right_member_expr, ctx);
                }
            }
        }
        (
            Expression::MemberExpression(left_member_expr),
            Expression::MemberExpression(right_member_expr),
        ) => return is_same_member_expression(left_member_expr, right_member_expr, ctx),
        _ => {}
    }

    false
}

fn is_same_member_expression(
    left: &MemberExpression,
    right: &MemberExpression,
    ctx: &LintContext,
) -> bool {
    let left_static_property_name = left.static_property_name();
    let right_static_property_name = right.static_property_name();

    if left_static_property_name != right_static_property_name {
        return false;
    };

    return is_same_reference(left.object(), right.object(), ctx);
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
    ];

    Tester::new_without_config(PreferLogicalOperatorOverTernary::NAME, pass, fail)
        .test_and_snapshot();
}
