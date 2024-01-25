use oxc_ast::{
    ast::{
        Expression, JSXAttributeValue, JSXElement, JSXExpression, JSXExpressionContainer,
        MemberExpression,
    },
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
#[error("eslint-plugin-react-perf(jsx-no-new-function-as-props): JSX attribute values should not contain functions created in the same scope.")]
#[diagnostic(severity(warning), help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array)."))]
struct JsxNoNewFunctionAsPropsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewFunctionAsProps;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Functions that are local to the current method from being used as values of JSX props
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <Item callback={new Function(...)} />
    /// <Item callback={this.props.callback || function() {}} />
    ///
    /// // Good
    /// <Item callback={this.props.callback} />
    /// ```
    JsxNoNewFunctionAsProps,
    correctness
);

impl Rule for JsxNoNewFunctionAsProps {
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
                    ctx.diagnostic(JsxNoNewFunctionAsPropsDiagnostic(span));
                }
            }
            _ => {}
        };
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parenthesized() {
        Expression::ArrowExpression(expr) => Some(expr.span),
        Expression::FunctionExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Function") {
                return Some(expr.span);
            }

            let Expression::MemberExpression(member_expr) = &expr.callee else {
                return None;
            };

            let property_name = MemberExpression::static_property_name(member_expr);

            if property_name == Some("bind") {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::NewExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Function") {
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

    let pass = vec![
        r"<Item callback={this.props.callback} />",
        r"<Item promise={new Promise()} />",
        r"<Item onClick={bind(foo)} />",
        r"<Item prop={0} />",
        r"var a;<Item prop={a} />",
        r"var a;a = 1;<Item prop={a} />",
        r"var a;<Item prop={a} />",
        r"function foo ({prop1 = function(){}, prop2}) {
            return <Comp prop={prop2} />
          }",
        r"function foo ({prop1, prop2 = function(){}}) {
            return <Comp prop={prop1} />
          }",
    ];

    let fail = vec![
        r"<Item prop={function(){return true}} />",
        r"<Item prop={() => true} />",
        r"<Item prop={new Function('a', 'alert(a)')}/>",
        r"<Item prop={Function()}/>",
        r"<Item onClick={this.clickHandler.bind(this)} />",
        r"<Item callback={this.props.callback || function() {}} />",
        r"<Item callback={this.props.callback ? this.props.callback : function() {}} />",
        r"<Item prop={this.props.callback || this.props.callback ? this.props.callback : function(){}} />",
        r"<Item prop={this.props.callback || (this.props.cb ? this.props.cb : function(){})} />",
    ];

    Tester::new(JsxNoNewFunctionAsProps::NAME, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
