use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::ContextHost,
    utils::{AllowedOrDisallowInFunc, function_count_before_lifecycle_component},
};

fn no_did_mount_set_state_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `setState` in `componentDidMount`.")
        .with_help("Updating state after a component mount triggers a second render() call and can lead to property/layout thrashing.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoDidMountSetState(AllowedOrDisallowInFunc);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using `setState` in the `componentDidMount` lifecycle method.
    ///
    /// This rule is not relevant for function components, and so can potentially be
    /// disabled for modern React codebases.
    ///
    /// ### Why is this bad?
    ///
    /// Updating the state after a component mount will trigger a second `render()` call and can lead to property/layout thrashing.
    /// This can cause performance issues and unexpected behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   componentDidMount: function() {
    ///     this.setState({
    ///       name: this.props.name.toUpperCase()
    ///     });
    ///   },
    ///   render: function() {
    ///     return <div>Hello {this.state.name}</div>;
    ///   }
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   componentDidMount: function() {
    ///     this.onMount(function callback(newName) {
    ///       this.setState({
    ///         name: newName
    ///       });
    ///     });
    ///   },
    ///   render: function() {
    ///     return <div>Hello {this.state.name}</div>;
    ///   }
    /// });
    /// ```
    NoDidMountSetState,
    react,
    correctness,
    config = AllowedOrDisallowInFunc,
    version = "1.36.0",
    short_description = "Disallow usage of `setState` in `componentDidMount`.",
);

impl Rule for NoDidMountSetState {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(member_expr) = call_expr.callee.as_member_expression() else { return };

        if !member_expr.object().is_this_expression()
            || member_expr.static_property_name().is_none_or(|name| name != "setState")
        {
            return;
        }

        let Some(function_count_before_component_did_mount) =
            function_count_before_lifecycle_component(node, ctx, "componentDidMount")
        else {
            return;
        };

        let in_nested_function = function_count_before_component_did_mount > 1;

        if in_nested_function && !matches!(self.0, AllowedOrDisallowInFunc::DisallowInFunc) {
            return;
        }

        ctx.diagnostic(no_did_mount_set_state_diagnostic(call_expr.callee.span()));
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
        var Hello = createReactClass({
          render: function() {
            return <div>Hello {this.props.name}</div>;
          }
        });
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {}
        });
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {
            someNonMemberFunction(arg);
            this.someHandler = this.setState;
          }
        });
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {
            someClass.onSomeEvent(function(data) {
              this.setState({
                data: data
              });
            })
          }
        });
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {
            function handleEvent(data) {
              this.setState({
                data: data
              });
            }
            someClass.onSomeEvent(handleEvent)
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidMount() {
            this.handleEvent(() => {
              this.setState({ data: 123 });
            });
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentDidUpdate() {
            this.setState({ data: 123 });
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentWillMount() {
            this.setState({ data: 123 });
          }
        }
        ",
        "
        var Hello = createReactClass({
          componentDidUpdate: function() {
            this.setState({ data: 123 });
          }
        });
        ",
        "
        function Hello() {
          this.setState({ data: 123 });
        }
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {
            setTimeout(() => {
              this.setState({ data: 123 });
            }, 100);
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidMount() {
            Promise.resolve().then(() => {
              this.setState({ data: 123 });
            });
          }
        }
        ",
    ];

    let fail = vec![
        "
        var Hello = createReactClass({
          componentDidMount: function() {
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
          componentDidMount: function componentDidMount() {
            this.setState({
              name: this.props.name.toUpperCase()
            });
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidMount() {
            this.setState({
              name: this.props.name.toUpperCase()
            });
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentDidMount = () => {
            this.setState({
              name: this.props.name.toUpperCase()
            });
          }
        }
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {
            this.setState({ data: 1 });
            someClass.onSomeEvent(function(data) {
              this.setState({ data: 2 });
            })
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidMount() {
            if (true) {
              this.setState({ data: 123 });
            }
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentDidMount() {
            const x = true ? this.setState({ data: 123 }) : null;
          }
        }
        ",
    ];

    Tester::new(NoDidMountSetState::NAME, NoDidMountSetState::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_disallow_in_func() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            var Hello = createReactClass({
              componentDidMount: function() {}
            });
            ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
            var Hello = createReactClass({
              render: function() {
                return <div>Hello {this.props.name}</div>;
              }
            });
            ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
    ];

    let fail = vec![
        (
            "
            var Hello = createReactClass({
              componentDidMount: function() {
                this.setState({
                  name: this.props.name.toUpperCase()
                });
              }
            });
            ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
            var Hello = createReactClass({
              componentDidMount: function() {
                someClass.onSomeEvent(function(data) {
                  this.setState({
                    data: data
                  });
                })
              }
            });
            ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
            var Hello = createReactClass({
              componentDidMount: function() {
                setTimeout(() => {
                  this.setState({ data: 123 });
                }, 100);
              }
            });
            ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
            class Hello extends React.Component {
              componentDidMount() {
                Promise.resolve().then(() => {
                  this.setState({ data: 123 });
                });
              }
            }
            ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
    ];

    Tester::new(NoDidMountSetState::NAME, NoDidMountSetState::PLUGIN, pass, fail)
        .with_snapshot_suffix("disallow_in_func")
        .test_and_snapshot();
}
