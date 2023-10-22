use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Do not use findDOMNode. It doesn’t work with function components and is deprecated in StrictMode. See https://reactjs.org/docs/react-dom.html#finddomnode")]
#[diagnostic(severity(warning))]
struct NoFindDomNodeDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoFindDomNode;

declare_oxc_lint!(
    /// ### What it does
    /// This rule disallows the use of `findDOMNode`.
    ///
    /// ### Why is this bad?
    /// Facebook will eventually deprecate `findDOMNode` as it blocks certain improvements in React in the future.
    ///
    /// ### Example
    /// ```javascript
    /// class MyComponent extends Component {
    ///   componentDidMount() {
    ///     findDOMNode(this).scrollIntoView();
    ///   }
    ///   render() {
    ///     return <div />;
    ///   }
    ///  }
    /// ```
    NoFindDomNode,
    correctness
);

impl Rule for NoFindDomNode {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        if call_expr.callee.get_identifier_reference().is_some_and(|x| x.name == "findDOMNode") {
            ctx.diagnostic(NoFindDomNodeDiagnostic(call_expr.span));
            return;
        }

        let Some(member) = call_expr.callee.get_member_expr() else { return };
        let Some(prop_name) = member.static_property_name() else { return };
        if prop_name == "findDOMNode" {
            ctx.diagnostic(NoFindDomNodeDiagnostic(call_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var Hello = function() {};", None),
        (
            r#"
            var Hello = createReactClass({
              render: function() {
                return <div>Hello</div>;
              }
            });              
            "#,
            None,
        ),
        (
            r#"
            var Hello = createReactClass({
              componentDidMount: function() {
                someNonMemberFunction(arg);
                this.someFunc = React.findDOMNode;
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            "#,
            None,
        ),
        (
            r#"
            var Hello = createReactClass({
              componentDidMount: function() {
                React.someFunc(this);
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            "#,
            None,
        ),
    ];

    let fail = vec![
        (
            r#"
            var Hello = createReactClass({
              componentDidMount: function() {
                React.findDOMNode(this).scrollIntoView();
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            "#,
            None,
        ),
        (
            r#"
            var Hello = createReactClass({
              componentDidMount: function() {
                ReactDOM.findDOMNode(this).scrollIntoView();
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            "#,
            None,
        ),
        (
            r#"
            class Hello extends Component {
              componentDidMount() {
                findDOMNode(this).scrollIntoView();
              }
              render() {
                return <div>Hello</div>;
              }
            }
            "#,
            None,
        ),
        (
            r#"
            class Hello extends Component {
              componentDidMount() {
                this.node = findDOMNode(this);
              }
              render() {
                return <div>Hello</div>;
              }
            }            
            "#,
            None,
        ),
    ];

    Tester::new(NoFindDomNode::NAME, pass, fail).test_and_snapshot();
}
