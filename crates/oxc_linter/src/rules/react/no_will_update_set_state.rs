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
    utils::{is_es5_component, is_es6_component, supports_unsafe_lifecycle_prefix},
};

fn no_will_update_set_state_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `setState` in `componentWillUpdate`.")
        .with_help("Updating state during the update step can lead to indeterminate component state and is not allowed.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NoWillUpdateSetStateConfig {
    #[default]
    #[serde(skip)]
    Allowed,
    DisallowInFunc,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoWillUpdateSetState(NoWillUpdateSetStateConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using `setState` in the `componentWillUpdate` lifecycle method.
    ///
    /// ### Why is this bad?
    ///
    /// Updating the state during the component update step can lead to indeterminate component state and is not allowed.
    /// This can cause unexpected behavior and bugs in your React application.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   componentWillUpdate: function() {
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
    ///   componentWillUpdate: function() {
    ///     this.props.prepareHandler();
    ///   },
    ///   render: function() {
    ///     return <div>Hello {this.state.name}</div>;
    ///   }
    /// });
    /// ```
    NoWillUpdateSetState,
    react,
    correctness,
    config = NoWillUpdateSetStateConfig,
);

impl Rule for NoWillUpdateSetState {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
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

        let react_version = ctx.settings().react.version.as_ref();
        let check_unsafe_prefix = supports_unsafe_lifecycle_prefix(react_version);

        let component_will_update_index =
            ancestors.iter().position(|ancestor| match ancestor.kind() {
                AstKind::ObjectProperty(prop) => prop.key.static_name().is_some_and(|key| {
                    if key == "UNSAFE_componentWillUpdate" && !check_unsafe_prefix {
                        return false;
                    }
                    key == "componentWillUpdate" || key == "UNSAFE_componentWillUpdate"
                }),
                AstKind::MethodDefinition(method) => method.key.static_name().is_some_and(|name| {
                    if name == "UNSAFE_componentWillUpdate" && !check_unsafe_prefix {
                        return false;
                    }
                    name == "componentWillUpdate" || name == "UNSAFE_componentWillUpdate"
                }),
                AstKind::PropertyDefinition(prop) => prop.key.static_name().is_some_and(|name| {
                    if name == "UNSAFE_componentWillUpdate" && !check_unsafe_prefix {
                        return false;
                    }
                    name == "componentWillUpdate" || name == "UNSAFE_componentWillUpdate"
                }),
                _ => false,
            });

        let Some(component_will_update_idx) = component_will_update_index else {
            return;
        };

        let in_component_will_update = ancestors[component_will_update_idx..]
            .iter()
            .any(|ancestor| is_es5_component(ancestor) || is_es6_component(ancestor));

        if !in_component_will_update {
            return;
        }

        let function_count_before_component_will_update = ancestors[..component_will_update_idx]
            .iter()
            .filter(|ancestor| {
                matches!(
                    ancestor.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            })
            .count();

        let in_nested_function = function_count_before_component_will_update > 1;

        if in_nested_function && !matches!(self.0, NoWillUpdateSetStateConfig::DisallowInFunc) {
            return;
        }

        ctx.diagnostic(no_will_update_set_state_diagnostic(call_expr.callee.span()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {}
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            someNonMemberFunction(arg);
			            this.someHandler = this.setState;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            someClass.onSomeEvent(function(data) {
			              this.setState({
			                data: data
			              });
			            })
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            function handleEvent(data) {
			              this.setState({
			                data: data
			              });
			            }
			            someClass.onSomeEvent(handleEvent)
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          UNSAFE_componentWillUpdate() {
			            this.setState({
			              data: data
			            });
			          }
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.2.0" } } })),
        ),
    ];

    let fail = vec![
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            this.setState({
			              data: data
			            });
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          componentWillUpdate() {
			            this.setState({
			              data: data
			            });
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            this.setState({
			              data: data
			            });
			          }
			        });
			      ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          componentWillUpdate() {
			            this.setState({
			              data: data
			            });
			          }
			        }
			      ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
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
			        class Hello extends React.Component {
			          componentWillUpdate() {
			            someClass.onSomeEvent(function(data) {
			              this.setState({
			                data: data
			              });
			            })
			          }
			        }
			      ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            if (true) {
			              this.setState({
			                data: data
			              });
			            }
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          componentWillUpdate() {
			            if (true) {
			              this.setState({
			                data: data
			              });
			            }
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentWillUpdate: function() {
			            someClass.onSomeEvent((data) => this.setState({data: data}));
			          }
			        });
			      ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          componentWillUpdate() {
			            someClass.onSomeEvent((data) => this.setState({data: data}));
			          }
			        }
			      ",
            Some(serde_json::json!(["disallow-in-func"])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          UNSAFE_componentWillUpdate() {
			            this.setState({
			              data: data
			            });
			          }
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.3.0" } } })),
        ),
        (
            "
			        var Hello = createReactClass({
			          UNSAFE_componentWillUpdate: function() {
			            this.setState({
			              data: data
			            });
			          }
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.3.0" } } })),
        ),
    ];

    Tester::new(NoWillUpdateSetState::NAME, NoWillUpdateSetState::PLUGIN, pass, fail)
        .test_and_snapshot();
}
