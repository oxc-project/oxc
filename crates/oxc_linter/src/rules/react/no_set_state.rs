use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::get_parent_component,
    AstNode,
};

fn no_set_state_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use setState").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSetState;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the usage of `this.setState` in React components.
    ///
    /// ### Why is this bad?
    ///
    /// When using an architecture that separates your application state from your UI components
    /// (e.g. Flux), it may be desirable to forbid the use of local component state. This rule is
    /// especially helpful in read-only applications (that don't use forms), since local component
    /// state should rarely be necessary in such cases.
    ///
    /// ### Example
    /// ```jsx
    /// var Hello = createReactClass({
    ///   getInitialState: function() {
    ///     return {
    ///       name: this.props.name
    ///     };
    ///   },
    ///   handleClick: function() {
    ///     this.setState({
    ///       name: this.props.name.toUpperCase()
    ///     });
    ///   },
    ///   render: function() {
    ///     return <div onClick={this.handleClick.bind(this)}>Hello {this.state.name}</div>;
    ///   }
    /// });
    /// ```
    NoSetState,
    react,
    style,
);

impl Rule for NoSetState {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if !matches!(member_expr.object(), Expression::ThisExpression(_))
            || !member_expr.static_property_name().is_some_and(|str| str == "setState")
            || get_parent_component(node, ctx).is_none()
        {
            return;
        }

        ctx.diagnostic(no_set_state_diagnostic(call_expr.callee.span()));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
			        var Hello = function() {
			          this.setState({})
			        };
			      ",
        "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
        "
			        var Hello = createReactClass({
			          componentDidUpdate: function() {
			            someNonMemberFunction(arg);
			            this.someHandler = this.setState;
			          },
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
        "
			        var Hello = function() {
			          this.setState({})
			        };
			        createReactClass({
			          render: function() {
			            let x;
			          }
			        });
			      ",
        "
			        var Hello = function() {
			          this.setState({})
			        };
			        class Other extends React.Component {
			          render() {
			            let x;
			          }
			        };
			      ",
    ];

    let fail = vec![
        "
			        var Hello = createReactClass({
			          componentDidUpdate: function() {
			            this.setState({
			              name: this.props.name.toUpperCase()
			            });
			          },
			          render: function() {
			            return <div>Hello {this.state.name}</div>;
			          }
			        });
			      ",
        "
			        var Hello = createReactClass({
			          someMethod: function() {
			            this.setState({
			              name: this.props.name.toUpperCase()
			            });
			          },
			          render: function() {
			            return <div onClick={this.someMethod.bind(this)}>Hello {this.state.name}</div>;
			          }
			        });
			      ",
        "
			        class Hello extends React.Component {
			          someMethod() {
			            this.setState({
			              name: this.props.name.toUpperCase()
			            });
			          }
			          render() {
			            return <div onClick={this.someMethod.bind(this)}>Hello {this.state.name}</div>;
			          }
			        };
			      ",
        "
			        class Hello extends React.Component {
			          someMethod = () => {
			            this.setState({
			              name: this.props.name.toUpperCase()
			            });
			          }
			          render() {
			            return <div onClick={this.someMethod.bind(this)}>Hello {this.state.name}</div>;
			          }
			        };
			      ",
        "
			        class Hello extends React.Component {
			          render() {
			            return <div onMouseEnter={() => this.setState({dropdownIndex: index})} />;
			          }
			        };
			      ",
    ];

    Tester::new(NoSetState::NAME, NoSetState::PLUGIN, pass, fail).test_and_snapshot();
}
