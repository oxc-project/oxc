use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-is-mounted): Disallow usage of isMounted")]
#[diagnostic(severity(warning), help("Do not use isMounted"))]
struct NoIsMountedDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoIsMounted;

declare_oxc_lint!(
    /// ### What it does
    ///    Disallows the usage of isMounted
    ///
    /// ### Why is this bad?
    ///    isMounted is an anti-pattern, is not available when using ES6 classes, and it is on its way to being officially deprecated.
    ///
    /// ### Example
    /// ```javascript
    ///
    ///     var Hello = createReactClass({
    /// 	    componentDidUpdate: function() {
    ///             if (!this.isMounted()) {
    /// 	            return;
    /// 	        }
    /// 	    }
    ///     });
    /// ```
    NoIsMounted,
    correctness
);

impl Rule for NoIsMounted {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call) = node.kind() {
            if let Expression::MemberExpression(el) = &call.callee {
                if let Expression::ThisExpression(_) = el.object() {
                    if el
                        .static_property_name()
                        .is_some_and(|property_name| property_name == "isMounted")
                    {
                        ctx.diagnostic(NoIsMountedDiagnostic(el.span()));
                    }
                }
            }
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
			        var Hello = function() {
			        };
			      ",
        "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello</div>;
			          }
			        });
			      ",
        "
			        var Hello = createReactClass({
			          componentDidUpdate: function() {
			            someNonMemberFunction(arg);
			            this.someFunc = this.isMounted;
			          },
			          render: function() {
			            return <div>Hello</div>;
			          }
			        });
			      ",
    ];

    let fail = vec![
        "
			        var Hello = createReactClass({
			          componentDidUpdate: function() {
			            if (!this.isMounted()) {
			              return;
			            }
			          },
			          render: function() {
			            return <div>Hello</div>;
			          }
			        });
			      ",
        "
			        var Hello = createReactClass({
			          someMethod: function() {
			            if (!this.isMounted()) {
			              return;
			            }
			          },
			          render: function() {
			            return <div onClick={this.someMethod.bind(this)}>Hello</div>;
			          }
			        });
			      ",
        "
			        class Hello extends React.Component {
			          someMethod() {
			            if (!this.isMounted()) {
			              return;
			            }
			          }
			          render() {
			            return <div onClick={this.someMethod.bind(this)}>Hello</div>;
			          }
			        };
			      ",
    ];

    Tester::new_without_config(NoIsMounted::NAME, pass, fail).test_and_snapshot();
}
