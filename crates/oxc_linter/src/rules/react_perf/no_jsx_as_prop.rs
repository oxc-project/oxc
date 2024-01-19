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

use crate::{context::LintContext, rule::Rule, utils::get_prop_value, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react-perf(no-jsx-as-prop): JSX attribute values should not contain other JSX.")]
#[diagnostic(severity(warning))]
struct NoJsxAsPropDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoJsxAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent JSX that are local to the current method from being used as values of JSX props
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <Item jsx={<SubItem />} />
    /// <Item jsx={this.props.jsx || <SubItem />} />
    /// <Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />
    ///
    /// // Good
    /// <Item callback={this.props.jsx} />
    /// ```
    NoJsxAsProp,
    correctness
);

impl Rule for NoJsxAsProp {
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
        ctx.diagnostic(NoJsxAsPropDiagnostic(jsx_elem.opening_element.span));
    }
}

fn check_expression(expr: &Expression) -> bool {
    match expr {
        Expression::JSXElement(_) => true,
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

    let pass = vec![r"<Item callback={this.props.jsx} />"];

    let fail = vec![
        r"<Item jsx={<SubItem />} />",
        r"<Item jsx={this.props.jsx || <SubItem />} />",
        r"<Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />",
    ];

    Tester::new(NoJsxAsProp::NAME, pass, fail).test_and_snapshot();
}
