use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_es5_component, is_es6_component},
};

fn no_did_update_set_state_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `setState` in `componentDidUpdate`.")
        .with_help("Updating state after a component update triggers a second render() call and can lead to property/layout thrashing.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NoDidUpdateSetStateConfig {
    #[default]
    #[serde(skip)]
    Allowed,
    /// Also disallow `setState` calls in nested functions within `componentDidUpdate`.
    DisallowInFunc,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoDidUpdateSetState(NoDidUpdateSetStateConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using `setState` in the `componentDidUpdate` lifecycle method.
    ///
    /// ### Why is this bad?
    ///
    /// Updating the state after a component update will trigger a second `render()` call and can lead to property/layout thrashing.
    /// This can cause performance issues and unexpected behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   componentDidUpdate: function() {
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
    ///   componentDidUpdate: function() {
    ///     this.onUpdate(function callback(newName) {
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
    NoDidUpdateSetState,
    react,
    correctness,
    config = NoDidUpdateSetStateConfig,
);

impl Rule for NoDidUpdateSetState {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value).unwrap_or_default().into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(member_expr) = call_expr.callee.as_member_expression() else { return };

        if !matches!(member_expr.object(), Expression::ThisExpression(_))
            || member_expr.static_property_name().is_none_or(|name| name != "setState")
        {
            return;
        }

        let ancestors: Vec<_> = ctx.nodes().ancestors(node.id()).skip(1).collect();

        let component_did_update_index =
            ancestors.iter().position(|ancestor| match ancestor.kind() {
                AstKind::ObjectProperty(prop)
                    if prop.key.static_name().is_some_and(|key| key == "componentDidUpdate") =>
                {
                    true
                }
                AstKind::MethodDefinition(method)
                    if method
                        .key
                        .static_name()
                        .is_some_and(|name| name == "componentDidUpdate") =>
                {
                    true
                }
                AstKind::PropertyDefinition(prop)
                    if prop.key.static_name().is_some_and(|name| name == "componentDidUpdate") =>
                {
                    true
                }
                _ => false,
            });

        let Some(component_did_update_idx) = component_did_update_index else {
            return;
        };

        let in_component_did_update = ancestors[component_did_update_idx..]
            .iter()
            .any(|ancestor| is_es5_component(ancestor) || is_es6_component(ancestor));

        if !in_component_did_update {
            return;
        }

        let function_count_before_component_did_update = ancestors[..component_did_update_idx]
            .iter()
            .filter(|ancestor| {
                matches!(
                    ancestor.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            })
            .count();

        let in_nested_function = function_count_before_component_did_update > 1;

        if in_nested_function && !matches!(self.0, NoDidUpdateSetStateConfig::DisallowInFunc) {
            return;
        }

        ctx.diagnostic(no_did_update_set_state_diagnostic(call_expr.callee.span()));
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
          componentDidUpdate: function() {}
        });
        ",
        "
        var Hello = createReactClass({
          componentDidUpdate: function() {
            someNonMemberFunction(arg);
            this.someHandler = this.setState;
          }
        });
        ",
        "
        var Hello = createReactClass({
          componentDidUpdate: function() {
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
          componentDidUpdate: function() {
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
          componentDidUpdate() {
            this.handleEvent(() => {
              this.setState({ data: 123 });
            });
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentDidMount() {
            this.setState({ data: 123 });
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentWillUpdate() {
            this.setState({ data: 123 });
          }
        }
        ",
        "
        var Hello = createReactClass({
          componentDidMount: function() {
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
          componentDidUpdate: function() {
            setTimeout(() => {
              this.setState({ data: 123 });
            }, 100);
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidUpdate() {
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
          componentDidUpdate: function componentDidUpdate() {
            this.setState({
              name: this.props.name.toUpperCase()
            });
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidUpdate() {
            this.setState({
              name: this.props.name.toUpperCase()
            });
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentDidUpdate = () => {
            this.setState({
              name: this.props.name.toUpperCase()
            });
          }
        }
        ",
        "
        var Hello = createReactClass({
          componentDidUpdate: function() {
            this.setState({ data: 1 });
            someClass.onSomeEvent(function(data) {
              this.setState({ data: 2 });
            })
          }
        });
        ",
        "
        class Hello extends React.Component {
          componentDidUpdate() {
            if (true) {
              this.setState({ data: 123 });
            }
          }
        }
        ",
        "
        class Hello extends React.Component {
          componentDidUpdate() {
            const x = true ? this.setState({ data: 123 }) : null;
          }
        }
        ",
    ];

    Tester::new(NoDidUpdateSetState::NAME, NoDidUpdateSetState::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_disallow_in_func() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            var Hello = createReactClass({
              componentDidUpdate: function() {}
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
              componentDidUpdate: function() {
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
              componentDidUpdate: function() {
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
              componentDidUpdate: function() {
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
              componentDidUpdate() {
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

    Tester::new(NoDidUpdateSetState::NAME, NoDidUpdateSetState::PLUGIN, pass, fail)
        .with_snapshot_suffix("disallow_in_func")
        .test_and_snapshot();
}
