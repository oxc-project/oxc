use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-is-mounted): Do not use isMounted.")]
#[diagnostic(severity(warning), help("isMounted is an anti-pattern, is not available when using ES6 classes, and it is on its way to being officially deprecated."))]
struct NoIsMountedDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoIsMounted;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent usage of isMounted.
    ///
    /// ### Why is this bad?
    /// isMounted is an anti-pattern, is not available when using ES6 classes, and it is on its way to being officially deprecated.
    ///
    /// ### Example
    /// ```javascript
    /// var Hello = createReactClass({
    ///  handleClick: function() {
    ///    setTimeout(function() {
    ///      if (this.isMounted()) {
    ///        return;
    ///      }
    ///    });
    ///  },
    ///  render: function() {
    ///    return <div onClick={this.handleClick.bind(this)}>Hello</div>;
    ///  }
    ///});
    /// ```
    NoIsMounted,
    correctness
);

impl Rule for NoIsMounted {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(v) = node.kind() else { return };
        let callee = &v.callee.without_parenthesized();

        if let Expression::MemberExpression(v) = callee {
            let Expression::ThisExpression(_) = &v.object() else { return };

            let Some((property_span, static_property_name)) = v.static_property_info() else {
                return;
            };

            if static_property_name != "isMounted" {
                return;
            };

            if is_in_arr_or_iter(node, ctx) {
                ctx.diagnostic(NoIsMountedDiagnostic(property_span));
            }
        }
    }
}

fn is_in_arr_or_iter<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let mut node = node;

    loop {
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return false;
        };

        match parent.kind() {
            AstKind::MethodDefinition(_) | AstKind::PropertyDefinition(_) => {
                return true;
            }
            _ => {}
        }
        node = parent;
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        var Hello = function() {
			        };
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello</div>;
			          }
			        });
			      ",
            None,
        ),
        (
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
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentDidUpdate: function() {
			            if (isMounted()) {
						  return;
						}
			          },
			          render: function() {
			            return <div>Hello</div>;
			          }
			        });
			      ",
            None,
        ),
    ];

    let fail = vec![
        (
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
            None,
        ),
        (
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
            None,
        ),
        (
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
            None,
        ),
        (
            "
					var Hello = createReactClass({
						handleClick: function() {
							setTimeout(function() {
								if (this.isMounted()) {
									return;
								}
							});
						},
						render: function() {
							return <div onClick={this.handleClick.bind(this)}>Hello</div>;
						}
					});
			      ",
            None,
        ),
    ];

    Tester::new(NoIsMounted::NAME, pass, fail).test_and_snapshot();
}
