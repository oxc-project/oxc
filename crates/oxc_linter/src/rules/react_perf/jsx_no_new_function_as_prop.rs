use oxc_ast::{
    ast::{Expression, JSXAttributeValue, JSXElement, MemberExpression},
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

fn jsx_no_new_function_as_prop_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("JSX attribute values should not contain functions created in the same scope.")
        .with_help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewFunctionAsProp;

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
    JsxNoNewFunctionAsProp,
    perf
);

impl Rule for JsxNoNewFunctionAsProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if node.scope_id() == ctx.scopes().root_scope_id() {
            return;
        }
        if let AstKind::JSXElement(jsx_elem) = node.kind() {
            check_jsx_element(jsx_elem, ctx);
        }
    }

    fn should_run(&self, ctx: &LintContext) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn check_jsx_element<'a>(jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    for item in &jsx_elem.opening_element.attributes {
        match get_prop_value(item) {
            None => return,
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(expr) = container.expression.as_expression() {
                    if let Some(span) = check_expression(expr) {
                        ctx.diagnostic(jsx_no_new_function_as_prop_diagnostic(span));
                    }
                }
            }
            _ => {}
        };
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parenthesized() {
        Expression::ArrowFunctionExpression(expr) => Some(expr.span),
        Expression::FunctionExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Function") {
                return Some(expr.span);
            }

            let member_expr = expr.callee.as_member_expression()?;
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
        r"const Foo = () => <Item callback={this.props.callback} />",
        r"const Foo = () => (<Item promise={new Promise()} />)",
        r"const Foo = () => (<Item onClick={bind(foo)} />)",
        r"const Foo = () => (<Item prop={0} />)",
        r"const Foo = () => { var a; return <Item prop={a} /> }",
        r"const Foo = () => { var a;a = 1; return <Item prop={a} /> }",
        r"const Foo = () => { var a;<Item prop={a} /> }",
        r"function foo ({prop1 = function(){}, prop2}) {
            return <Comp prop={prop2} />
          }",
        r"function foo ({prop1, prop2 = function(){}}) {
            return <Comp prop={prop1} />
          }",
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

    let fail = vec![
        r"const Foo = () => (<Item prop={function(){return true}} />)",
        r"const Foo = () => (<Item prop={() => true} />)",
        r"const Foo = () => (<Item prop={new Function('a', 'alert(a)')}/>)",
        r"const Foo = () => (<Item prop={Function()}/>)",
        r"const Foo = () => (<Item onClick={this.clickHandler.bind(this)} />)",
        r"const Foo = () => (<Item callback={this.props.callback || function() {}} />)",
        r"const Foo = () => (<Item callback={this.props.callback ? this.props.callback : function() {}} />)",
        r"const Foo = () => (<Item prop={this.props.callback || this.props.callback ? this.props.callback : function(){}} />)",
        r"const Foo = () => (<Item prop={this.props.callback || (this.props.cb ? this.props.cb : function(){})} />)",
    ];

    Tester::new(JsxNoNewFunctionAsProp::NAME, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
