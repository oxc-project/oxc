use crate::utils::get_prop_value;
use oxc_ast::{
    ast::{
        Expression, JSXAttributeValue, JSXElement, JSXExpression, JSXExpressionContainer,
        MemberExpression,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react-perf(no-new-function-as-props):")]
#[diagnostic(severity(warning), help(""))]
struct NoNewFunctionAsPropsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNewFunctionAsProps;

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
    NoNewFunctionAsProps,
    correctness
);

// TODO native allow list params?
impl Rule for NoNewFunctionAsProps {
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
        })) => match expr {
            Expression::ArrowExpression(_) => true,
            Expression::FunctionExpression(_) => true,
            Expression::CallExpression(expr) => {
                if check_constructor(&expr.callee) {
                    return true;
                }

                let Expression::MemberExpression(member_expr) = &expr.callee else {
                    return false;
                };

                let property_name = MemberExpression::static_property_name(&member_expr);

                property_name == Some("bind")
            }
            Expression::NewExpression(expr) => check_constructor(&expr.callee),
            _ => false,
        },
        _ => false,
    }) {
        ctx.diagnostic(NoNewFunctionAsPropsDiagnostic(jsx_elem.opening_element.span));
    }
}

fn check_constructor<'a>(callee: &Expression<'a>) -> bool {
    let Expression::Identifier(ident) = callee else {
        return false;
    };
    ident.name == "Function"
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
        // LogicalExpression
        // r"<Item callback={this.props.callback || function() {}} />",//err
        // r"<Item callback={this.props.callback ? this.props.callback : function() {}} />",//err
    ];

    Tester::new(NoNewFunctionAsProps::NAME, pass, fail).test_and_snapshot();
}
