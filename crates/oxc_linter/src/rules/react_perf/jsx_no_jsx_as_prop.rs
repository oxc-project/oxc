use oxc_ast::{
    ast::{Expression, JSXAttributeValue, JSXElement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::get_prop_value, AstNode};

fn jsx_no_jsx_as_prop_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-react-perf(jsx-no-jsx-as-prop): JSX attribute values should not contain other JSX.")
        .with_help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoJsxAsProp;

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
    JsxNoJsxAsProp,
    perf
);

impl Rule for JsxNoJsxAsProp {
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
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(expr) = container.expression.as_expression() {
                    if let Some(span) = check_expression(expr) {
                        ctx.diagnostic(jsx_no_jsx_as_prop_diagnostic(span));
                    }
                }
            }
            _ => {}
        };
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parenthesized() {
        Expression::JSXElement(expr) => Some(expr.span),
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

    let pass = vec![r"<Item callback={this.props.jsx} />"];

    let fail = vec![
        r"<Item jsx={<SubItem />} />",
        r"<Item jsx={this.props.jsx || <SubItem />} />",
        r"<Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />",
        r"<Item jsx={this.props.jsx || (this.props.component ? this.props.component : <SubItem />)} />",
    ];

    Tester::new(JsxNoJsxAsProp::NAME, pass, fail).with_react_perf_plugin(true).test_and_snapshot();
}
