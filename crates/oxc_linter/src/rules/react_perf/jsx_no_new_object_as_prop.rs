use oxc_ast::{
    ast::{Expression, JSXAttributeValue, JSXElement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_prop_value, is_constructor_matching_name},
    AstNode,
};

fn jsx_no_new_object_as_prop_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-react-perf(jsx-no-new-object-as-prop): JSX attribute values should not contain objects created in the same scope.")
        .with_help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewObjectAsProp;

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
    JsxNoNewObjectAsProp,
    perf
);

impl Rule for JsxNoNewObjectAsProp {
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
                        ctx.diagnostic(jsx_no_new_object_as_prop_diagnostic(span));
                    }
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

    Tester::new(JsxNoNewObjectAsProp::NAME, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
