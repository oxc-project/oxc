use oxc_ast::{
    ast::{Expression, JSXAttributeValue, JSXElement, JSXExpression, JSXExpressionContainer},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{check_constructor, get_prop_value},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react-perf(no-new-array-as-prop): JSX attribute values should not contain Arrays created in the same scope.")]
#[diagnostic(severity(warning))]
struct NoNewArrayAsPropDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNewArrayAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Arrays that are local to the current method from being used as values of JSX props
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <Item list={[]} />

    /// <Item list={new Array()} />
    /// <Item list={Array()} />
    /// <Item list={this.props.list || []} />
    /// <Item list={this.props.list ? this.props.list : []} />
    ///
    /// // Good
    /// <Item list={this.props.list} />
    /// ```
    NoNewArrayAsProp,
    correctness
);

impl Rule for NoNewArrayAsProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                check_jsx_element(jsx_elem, ctx);
            }
            _ => {}
        };
    }
}

fn check_jsx_element<'a>(jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    if jsx_elem.opening_element.attributes.iter().any(|item| match get_prop_value(item) {
        None => false,
        Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
            expression: JSXExpression::Expression(expr),
            ..
        })) => check_expression(expr),
        _ => false,
    }) {
        ctx.diagnostic(NoNewArrayAsPropDiagnostic(jsx_elem.opening_element.span));
    }
}

fn check_expression(expr: &Expression) -> bool {
    match expr {
        Expression::ArrayExpression(_) => true,
        Expression::CallExpression(expr) => check_constructor(&expr.callee, "Array"),
        Expression::NewExpression(expr) => check_constructor(&expr.callee, "Array"),
        Expression::LogicalExpression(expr) => {
            check_expression(&expr.left) || check_expression(&expr.right)
        }
        Expression::ConditionalExpression(expr) => {
            check_expression(&expr.consequent) || check_expression(&expr.alternate)
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r"<Item list={this.props.list} />"];

    let fail = vec![
        r"<Item list={[]} />",
        r"<Item list={new Array()} />",
        r"<Item list={Array()} />",
        r"<Item list={this.props.list || []} />",
        r"<Item list={this.props.list ? this.props.list : []} />",
    ];

    Tester::new(NoNewArrayAsProp::NAME, pass, fail).test_and_snapshot();
}
