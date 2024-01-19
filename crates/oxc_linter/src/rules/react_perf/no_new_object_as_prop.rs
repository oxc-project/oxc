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
#[error("eslint-plugin-react-perf(no-new-object-as-prop):")]
#[diagnostic(severity(warning), help(""))]
struct NoNewObjectAsPropDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNewObjectAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Objects that are local to the current method from being used as values of JSX props
    ///
    /// ```javascript
    /// // Bad
    /// <Item config={{}} />
    /// <Item config={new Object()} />
    /// <Item config={Object()} />
    /// <Item config={this.props.config || {}} />
    /// <Item config={this.props.config ? this.props.config : {}} />
    // <div style={{display: 'none'}} />
    ///
    /// // Good
    /// <Item config={staticConfig} />
    /// ```
    NoNewObjectAsProp,
    correctness
);

impl Rule for NoNewObjectAsProp {
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
        ctx.diagnostic(NoNewObjectAsPropDiagnostic(jsx_elem.opening_element.span));
    }
}

fn check_expression(expr: &Expression) -> bool {
    match expr {
        Expression::ObjectExpression(_) => true,
        Expression::CallExpression(expr) => check_constructor(&expr.callee, "Object"),
        Expression::NewExpression(expr) => check_constructor(&expr.callee, "Object"),
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

    let pass = vec![r"<Item config={staticConfig} />"];

    let fail = vec![
        r"<Item config={{}} />",
        r"<Item config={new Object()} />",
        r"<Item config={Object()} />",
        r"<div style={{display: 'none'}} />",
        r"<Item config={this.props.config || {}} />",
        r"<Item config={this.props.config ? this.props.config : {}} />",
    ];

    Tester::new(NoNewObjectAsProp::NAME, pass, fail).test_and_snapshot();
}
