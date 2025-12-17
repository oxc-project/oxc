use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::get_outer_member_expression,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_es6_component, is_state_member_expression},
};

fn state_in_constructor_diagnostic(span: Span, is_state_init_constructor: bool) -> OxcDiagnostic {
    let message = if is_state_init_constructor {
        "State initialization should be in a constructor"
    } else {
        "State initialization should be in a class property"
    };
    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateInConstructorConfig {
    /// Enforce state initialization in the constructor.
    #[default]
    Always,
    /// Enforce state initialization with a class property.
    Never,
}

impl StateInConstructorConfig {
    pub const fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }

    pub const fn is_never(&self) -> bool {
        matches!(self, Self::Never)
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct StateInConstructor(Box<StateInConstructorConfig>);

impl std::ops::Deref for StateInConstructor {
    type Target = StateInConstructorConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the state initialization style to be either in a constructor or with a class property.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent state initialization styles can make the codebase harder to maintain and understand.
    /// This rule enforces a consistent pattern across React class components.
    ///
    /// ### Examples
    ///
    /// This rule has two modes: `"always"` and `"never"`.
    ///
    /// #### `"always"` mode
    ///
    /// Will enforce the state initialization style to be in a constructor. This is the default mode.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   state = { bar: 0 }
    ///   render() {
    ///     return <div>Foo</div>
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   constructor(props) {
    ///     super(props)
    ///     this.state = { bar: 0 }
    ///   }
    ///   render() {
    ///     return <div>Foo</div>
    ///   }
    /// }
    /// ```
    ///
    /// #### `"never"` mode
    ///
    /// Will enforce the state initialization style to be with a class property.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   constructor(props) {
    ///     super(props)
    ///     this.state = { bar: 0 }
    ///   }
    ///   render() {
    ///     return <div>Foo</div>
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   state = { bar: 0 }
    ///   render() {
    ///     return <div>Foo</div>
    ///   }
    /// }
    /// ```
    StateInConstructor,
    react,
    style,
    config = StateInConstructorConfig,
);

impl Rule for StateInConstructor {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<StateInConstructor>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::PropertyDefinition(prop_def) => {
                if self.is_always()
                    && !prop_def.r#static
                    && prop_def.key.name().is_some_and(|name| name == "state")
                    && has_parent_es6_component(node, ctx)
                {
                    ctx.diagnostic(state_in_constructor_diagnostic(prop_def.span, true));
                }
            }
            AstKind::AssignmentExpression(assign_expr) => {
                if self.is_never()
                    && let Some(assignment) = assign_expr.left.as_simple_assignment_target()
                    && let Some(outer_member_expression) = get_outer_member_expression(assignment)
                    && is_state_member_expression(outer_member_expression)
                    && is_in_constructor(ctx, node.id())
                    && has_parent_es6_component(node, ctx)
                {
                    ctx.diagnostic(state_in_constructor_diagnostic(assign_expr.span, false));
                }
            }
            _ => (),
        }
    }
}

fn has_parent_es6_component<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|node| is_es6_component(node))
}

/// Checks if a node is inside a constructor method.
pub fn is_in_constructor(ctx: &LintContext, node_id: NodeId) -> bool {
    for ancestor_id in ctx.nodes().ancestor_ids(node_id) {
        if let AstKind::MethodDefinition(method) = ctx.nodes().kind(ancestor_id) {
            return method.kind.is_constructor();
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        class Foo extends React.Component {
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          baz = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.baz = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.baz = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          baz = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          baz = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        const Foo = () => <div>Foo</div>
			      ",
            None,
        ),
        (
            "
			        const Foo = () => <div>Foo</div>
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        function Foo () {
			          return <div>Foo</div>
			        }
			      ",
            None,
        ),
        (
            "
			        function Foo () {
			          return <div>Foo</div>
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          state = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          state = { bar: 0 }
			          baz = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.baz = { bar: 0 }
			          }
			          state = { baz: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            if (foobar) {
			              this.state = { bar: 0 }
			            }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            foobar = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            foobar = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
    ];

    let fail = vec![
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          baz = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          state = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          state = { bar: 0 }
			          baz = { bar: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.baz = { bar: 0 }
			          }
			          state = { baz: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          state = { baz: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            this.state = { bar: 0 }
			          }
			          state = { baz: 0 }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
			        class Foo extends React.Component {
			          constructor(props) {
			            super(props)
			            if (foobar) {
			              this.state = { bar: 0 }
			            }
			          }
			          render() {
			            return <div>Foo</div>
			          }
			        }
			      ",
            Some(serde_json::json!(["never"])),
        ),
        (
            "
              class Foo extends React.Component {
                  constructor(props) {
                      super(props);

                      function helper() {
                          this.state = {bar: 0};
                      }

                      helper();
                  }
              }
			      ",
            Some(serde_json::json!(["never"])),
        ),
    ];

    Tester::new(StateInConstructor::NAME, StateInConstructor::PLUGIN, pass, fail)
        .test_and_snapshot();
}
