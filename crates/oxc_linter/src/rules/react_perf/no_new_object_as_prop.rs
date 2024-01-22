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
    utils::{get_prop_value, is_constructor_matching_name},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react-perf(no-new-object-as-prop): JSX attribute values should not contain objects created in the same scope.")]
#[diagnostic(severity(warning), help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array)."))]
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
    restriction
);

impl Rule for NoNewObjectAsProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_elem) = node.kind() {
            check_jsx_element(jsx_elem, ctx);
        }
    }
}

fn check_jsx_element<'a>(jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    for item in &jsx_elem.opening_element.attributes {
        match get_prop_value(item) {
            None => return,
            Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                expression: JSXExpression::Expression(expr),
                ..
            })) => {
                if let Some(span) = check_expression(expr) {
                    ctx.diagnostic(NoNewObjectAsPropDiagnostic(span));
                }
            }
            _ => {}
        };
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parenthesized() {
        Expression::ObjectExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Object") {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::NewExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Object") {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::LogicalExpression(expr) => {
            check_expression(&expr.left).or_else(|| check_expression(&expr.right))
        }
        Expression::ConditionalExpression(expr) => {
            check_expression(&expr.consequent).or_else(|| check_expression(&expr.alternate))
        }
        _ => None,
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
        r"<Item config={this.props.config || (this.props.default ? this.props.default : {})} />",
    ];

    Tester::new(NoNewObjectAsProp::NAME, pass, fail).test_and_snapshot();
}
