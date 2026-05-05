use oxc_ast::{
    AstKind,
    ast::{Expression, StaticMemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::expand_span_to_statement_boundaries,
    rule::{Rule, TupleRuleConfig},
    rules::ContextHost,
    utils::{FunctionLike, get_parent_component, get_parent_stateless_component},
};

//   noDestructPropsInSFCArg: 'Must never use destructuring props assignment in SFC argument',
// noDestructContextInSFCArg: 'Must never use destructuring context assignment in SFC argument',
// noDestructAssignment: 'Must never use destructuring {{type}} assignment',
// useDestructAssignment: 'Must use destructuring {{type}} assignment',
// destructureInSignature: 'Must destructure props in the function signature.',

fn no_destruct_props_in_sfc_arg_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Must never use destructuring props assignment in SFC argument.")
        .with_label(span)
}

fn no_destruct_context_in_sfc_arg_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Must never use destructuring context assignment in SFC argument.")
        .with_label(span)
}

fn no_destruct_assignment_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Must never use destructuring {name} assignment.")).with_label(span)
}

fn use_destruct_assignment_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Must use destructuring {name} assignment.")).with_label(span)
}

fn destructure_in_signature_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Must destructure props in the function signature.").with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    #[default]
    Always,
    Never,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DestructureInSignature {
    Always,
    #[default]
    Ignore,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct DestructuringAssignmentOptions {
    // Whether to ignore class fields when destructuring.
    ignore_class_fields: bool,
    // Whether to ignore destructuring in function signature.
    destructure_in_signature: DestructureInSignature,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(default)]
pub struct DestructuringAssignmentTupleConfig(Mode, DestructuringAssignmentOptions);

#[derive(Debug, Clone)]
pub struct DestructuringAssignmentConfig {
    apply_never: bool,
    apply_to_class_fields: bool,
    apply_to_signature: bool,
}

impl From<DestructuringAssignmentTupleConfig> for DestructuringAssignmentConfig {
    fn from(value: DestructuringAssignmentTupleConfig) -> Self {
        let DestructuringAssignmentTupleConfig(mode, options) = value;

        DestructuringAssignmentConfig {
            apply_never: matches!(mode, Mode::Never),
            apply_to_class_fields: !options.ignore_class_fields,
            apply_to_signature: matches!(
                options.destructure_in_signature,
                DestructureInSignature::Always
            ),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DestructuringAssignment(Box<DestructuringAssignmentConfig>);

impl std::ops::Deref for DestructuringAssignment {
    type Target = DestructuringAssignmentConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DestructuringAssignment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let config = DestructuringAssignmentTupleConfig::deserialize(deserializer)?;
        Ok(Self(Box::new(config.into())))
    }
}

impl Default for DestructuringAssignmentConfig {
    fn default() -> Self {
        Self { apply_never: false, apply_to_class_fields: true, apply_to_signature: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent usage of destructuring assignment of props, state, and context.
    ///
    /// ### Why is this bad?
    ///
    /// Destructuring can make it easier to read and understand what properties are being used in a component.
    ///
    /// ### Options
    ///
    /// This rule takes one string and one optional object as arguments:
    ///
    /// ```jsonc
    /// {
    ///   "rules": {
    ///     "react/destructuring-assignment": [
    ///       "error",
    ///       "always", // or "never"
    ///       {
    ///         "ignoreClassFields": false,
    ///         "destructureInSignature": "ignore" // or "always"
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// - `"always"` (default): enforce destructuring of `props`, `state`, and `context`.
    /// - `"never"`: forbid destructuring of `props`, `state`, and `context`.
    /// - `ignoreClassFields` (default `false`): when `true`, ignore class field
    ///   declarations such as `bar = this.props.bar`.
    /// - `destructureInSignature` (default `"ignore"`): when set to `"always"`,
    ///   require props destructuring to happen in the function signature.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule, when configured with `"always"` (the default):
    ///
    /// ```jsx
    /// const MyComponent = (props) => {
    ///   return (<div id={props.id} />)
    /// };
    /// ```
    ///
    /// ```jsx
    /// const Foo = class extends React.PureComponent {
    ///   render() {
    ///     return <div>{this.context.foo}</div>;
    ///   }
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule, when configured with `"always"` (the default):
    ///
    /// ```jsx
    /// const MyComponent = ({id}) => {
    ///   return (<div id={id} />)
    /// };
    /// ```
    ///
    /// ```jsx
    /// const MyComponent = (props, context) => {
    ///   const { id } = props;
    ///   return (<div id={id} />)
    /// };
    /// ```
    ///
    /// ```jsx
    /// const Foo = class extends React.PureComponent {
    ///   render() {
    ///     const { title } = this.context;
    ///     return <div>{title}</div>;
    ///   }
    /// };
    /// ```
    ///
    /// Examples of **incorrect** code for this rule, when configured with `"never"`:
    ///
    /// ```jsx
    /// const MyComponent = ({id}) => {
    ///   return (<div id={id} />)
    /// };
    /// ```
    ///
    /// ```jsx
    /// const MyComponent = (props) => {
    ///   const { id } = props;
    ///   return (<div id={id} />)
    /// };
    /// ```
    ///
    /// ```jsx
    /// const Foo = class extends React.PureComponent {
    ///   render() {
    ///     const { title } = this.state;
    ///     return <div>{title}</div>;
    ///   }
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule, when configured with `"never"`:
    ///
    /// ```jsx
    /// const MyComponent = (props) => {
    ///   return (<div id={props.id} />)
    /// };
    /// ```
    ///
    /// ```jsx
    /// const Foo = class extends React.PureComponent {
    ///   render() {
    ///     return <div>{this.state.title}</div>;
    ///   }
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule, when configured with `["always", { "ignoreClassFields": true }]`:
    ///
    /// ```jsx
    /// class Foo extends React.PureComponent {
    ///   bar = this.props.bar
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule, when configured with `["always", { "destructureInSignature": "always" }]`:
    ///
    /// ```jsx
    /// function Foo(props) {
    ///   const {a} = props;
    ///   return <>{a}</>
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule, when configured with `["always", { "destructureInSignature": "always" }]`:
    ///
    /// ```jsx
    /// function Foo({a}) {
    ///   return <>{a}</>
    /// }
    /// ```
    ///
    /// ```jsx
    /// // Ignores when props is used elsewhere
    /// function Foo(props) {
    ///   const {a} = props;
    ///   useProps(props); // NOTE: it is a bad practice to pass the props object anywhere else!
    ///   return <Goo a={a}/>
    /// }
    /// ```
    DestructuringAssignment,
    react,
    style,
    fix,
    config = DestructuringAssignmentTupleConfig,
    version = "next",
);

impl DestructuringAssignment {
    fn handle_object_pattern(
        &self,
        node_id: NodeId,
        ctx: &LintContext,
    ) -> Option<(Span, Span, Span)> {
        for ancestor in ctx.nodes().ancestors(node_id) {
            let node = ctx.nodes().get_node(node_id);
            match ancestor.kind() {
                AstKind::VariableDeclarator(decl) => {
                    let Some(init) = &decl.init else {
                        break;
                    };

                    if let Some(prop_name) = get_this_member_name(init) {
                        if get_parent_component(node, ctx).is_some() {
                            ctx.diagnostic(no_destruct_assignment_diagnostic(decl.span, prop_name));
                        }
                        break;
                    }

                    let Some(id_ref) = init.get_identifier_reference() else {
                        break;
                    };
                    let Some(parent) = get_parent_stateless_component(node, ctx) else {
                        break;
                    };
                    let obj_name = id_ref.name.as_str();
                    let params = parent.params();
                    let Some(param) = params.items.iter().find(|p| {
                        p.pattern
                            .get_binding_identifier()
                            .is_some_and(|id| id.name.as_str() == obj_name)
                    }) else {
                        break;
                    };

                    if self.apply_never {
                        ctx.diagnostic(no_destruct_assignment_diagnostic(decl.span, obj_name));
                    } else if self.apply_to_signature {
                        let binding = param.pattern.get_binding_identifier().unwrap();
                        let used_more_than_once = ctx
                            .scoping()
                            .get_resolved_references(binding.symbol_id())
                            .filter(|reference| !reference.is_type())
                            .count()
                            > 1;
                        if !used_more_than_once {
                            let object_pattern_span = node.span();
                            let declaration_span = ctx.nodes().parent_node(decl.node_id()).span();
                            let param_span = param.pattern.span();
                            return Some((object_pattern_span, declaration_span, param_span));
                        }
                    }
                    break;
                }
                AstKind::FormalParameter(_) | AstKind::FormalParameters(_) => break,
                _ => {}
            }
        }
        None
    }

    fn should_skip_member(&self, node: &AstNode, ctx: &LintContext) -> bool {
        match ctx.nodes().parent_kind(node.id()) {
            AstKind::AssignmentExpression(_) => true,
            AstKind::PropertyDefinition(_) => !self.apply_to_class_fields,
            AstKind::TemplateLiteral(_) => {
                !self.apply_to_class_fields
                    && matches!(
                        ctx.nodes().parent_kind(ctx.nodes().parent_id(node.id())),
                        AstKind::PropertyDefinition(_)
                    )
            }
            _ => false,
        }
    }
}

impl Rule for DestructuringAssignment {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(member) => {
                if self.should_skip_member(node, ctx) || self.apply_never {
                    return;
                }
                if let Some(parent) = get_parent_stateless_component(node, ctx) {
                    handle_function_component_usage(member, ctx, &parent);
                } else if get_parent_component(node, ctx).is_some() {
                    handle_class_component_usage(member, ctx);
                }
            }
            AstKind::FormalParameter(param)
                if param.pattern.is_object_pattern() && self.apply_never =>
            {
                if let Some(parent) = get_parent_stateless_component(node, ctx) {
                    let params = parent.params();
                    if params.items[0].span == param.span {
                        ctx.diagnostic(no_destruct_props_in_sfc_arg_diagnostic(param.span));
                    } else if params.items[1].span == param.span {
                        ctx.diagnostic(no_destruct_context_in_sfc_arg_diagnostic(param.span));
                    }
                }
            }
            AstKind::ObjectPattern(_) if self.apply_never || self.apply_to_signature => {
                let Some((object_pattern_span, decl_span, param_span)) =
                    self.handle_object_pattern(node.id(), ctx)
                else {
                    return;
                };
                ctx.diagnostic_with_fix(destructure_in_signature_diagnostic(decl_span), |fixer| {
                    let mut fix = fixer.new_fix_with_capacity(2);
                    fix.push(
                        fixer.replace(
                            param_span,
                            fixer.source_range(object_pattern_span).to_string(),
                        ),
                    );
                    let expanded_decl_span =
                        expand_span_to_statement_boundaries(fixer.source_text(), decl_span);
                    fix.push(fixer.delete_range(expanded_decl_span));
                    fix.with_message("Replace object pattern with destructuring in signature")
                });
            }
            _ => {}
        }
    }
}

fn handle_class_component_usage<'a>(node: &StaticMemberExpression<'a>, ctx: &LintContext<'a>) {
    let Some(prop_name) = get_this_member_name(&node.object) else {
        return;
    };
    ctx.diagnostic(use_destruct_assignment_diagnostic(node.span, prop_name));
}

fn handle_function_component_usage<'a>(
    node: &StaticMemberExpression<'a>,
    ctx: &LintContext<'a>,
    parent: &FunctionLike<'a>,
) {
    let params = parent.params();
    let Some(id_ref) = node.object.get_identifier_reference() else {
        return;
    };
    let obj_name = id_ref.name.as_str();
    let matched_name = params.items.iter().find_map(|param| {
        let param_id = param.pattern.get_binding_identifier()?;
        let param_name = param_id.name.as_str();
        (obj_name == param_name).then_some(param_name)
    });
    if let Some(name) = matched_name {
        ctx.diagnostic(use_destruct_assignment_diagnostic(node.span, name));
    }
}

fn get_this_member_name<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    let Expression::StaticMemberExpression(member) = expr else { return None };
    let Expression::ThisExpression(_) = &member.object else { return None };
    let prop_name = member.property.name.as_str(); // "props", "context", or "state"
    matches!(prop_name, "props" | "context" | "state").then_some(prop_name)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                    export const revisionStates2 = {
                        [A.b]: props => {
                          return props.editor !== null
                            ? 'xyz'
                            : 'abc'
                        },
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    export function hof(namespace) {
                      const initialState = {
                        bounds: null,
                        search: false,
                      };
                      return (props) => {
                        const {x, y} = props
                        if (y) {
                          return <span>{y}</span>;
                        }
                        return <span>{x}</span>
                      };
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    export function hof(namespace) {
                      const initialState = {
                        bounds: null,
                        search: false,
                      };

                      return (state = initialState, action) => {
                        if (action.type === 'ABC') {
                          return {...state, bounds: stuff ? action.x : null};
                        }

                        if (action.namespace !== namespace) {
                          return state;
                        }

                        return null
                      };
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    const MyComponent = ({ id, className }) => (
                      <div id={id} className={className} />
                    );
                  ",
            None,
            None,
        ),
        (
            "
                    const MyComponent = ({ id, className }) => (
                      <div id={id} className={className} />
                    );
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const MyComponent = (props) => {
                      const { id, className } = props;
                      return <div id={id} className={className} />
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    const MyComponent = (props) => {
                      const { id, className } = props;
                      return <div id={id} className={className} />
                    };
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const MyComponent = (props) => (
                      <div id={id} props={props} />
                    );
                  ",
            None,
            None,
        ),
        (
            "
                    const MyComponent = (props) => (
                      <div id={id} props={props} />
                    );
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const MyComponent = (props, { color }) => (
                      <div id={id} props={props} color={color} />
                    );
                  ",
            None,
            None,
        ),
        (
            "
                    const MyComponent = (props, { color }) => (
                      <div id={id} props={props} color={color} />
                    );
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        return <div>{this.props.foo}</div>;
                      }
                    };
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    class Foo extends React.Component {
                      doStuff() {}
                      render() {
                        return <div>{this.props.foo}</div>;
                      }
                    }
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        const { foo } = this.props;
                        return <div>{foo}</div>;
                      }
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        const { foo } = this.props;
                        return <div>{foo}</div>;
                      }
                    };
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const MyComponent = (props) => {
                      const { h, i } = hi;
                      return <div id={props.id} className={props.className} />
                    };
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      constructor() {
                        this.state = {};
                        this.state.foo = 'bar';
                      }
                    };
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const div = styled.div`
                      & .button {
                        border-radius: ${props => props.borderRadius}px;
                      }
                    `
                  ",
            None,
            None,
        ),
        (
            "
                    export default (context: $Context) => ({
                      foo: context.bar
                    });
                  ",
            None,
            None,
        ),
        (
            "
                    class Foo {
                      bar(context) {
                        return context.baz;
                      }
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    class Foo {
                      bar(props) {
                        return props.baz;
                      }
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    class Foo extends React.Component {
                      bar = this.props.bar
                    }
                  ",
            Some(serde_json::json!(["always", { "ignoreClassFields": true }])),
            None,
        ),
        (
            "
                    class Input extends React.Component {
                      id = `${this.props.name}`;
                      render() {
                        return <div />;
                      }
                    }
                  ",
            Some(serde_json::json!(["always", { "ignoreClassFields": true }])),
            None,
        ),
        (
            "
                    function Foo({ context }) {
                      const d = context.describe();
                      return <div>{d}</div>;
                    }
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    const obj = {
                      foo(arg) {
                        const a = arg.func();
                        return null;
                      },
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    const columns = [
                      {
                        render: (val) => {
                          if (val.url) {
                            return (
                              <a href={val.url}>
                                {val.test}
                              </a>
                            );
                          }
                          return null;
                        },
                      },
                    ];
                  ",
            None,
            None,
        ),
        (
            "
                    const columns = [
                      {
                        render: val => <span>{val}</span>,
                      },
                      {
                        someRenderFunc: function(val) {
                          if (val.url) {
                            return (
                              <a href={val.url}>
                                {val.test}
                              </a>
                            );
                          }
                          return null;
                        },
                      },
                    ];
                  ",
            None,
            None,
        ),
        (
            "
                    export default (fileName) => {
                      const match = fileName.match(/some expression/);
                      if (match) {
                        return fn;
                      }
                      return null;
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    class C extends React.Component {
                      componentDidMount() {
                        const { forwardRef } = this.props;

                        this.ref.current.focus();

                        if (typeof forwardRef === 'function') {
                          forwardRef(this.ref);
                        }
                      }
                      render() {
                        return <div />;
                      }
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    function Foo(props) {
                      const {a} = props;
                      return <Goo {...props}>{a}</Goo>;
                    }
                  ",
            Some(serde_json::json!(["always", { "destructureInSignature": "always" }])),
            None,
        ),
        (
            "
                    function Foo(props) {
                      const {a} = props;
                      return <Goo f={() => props}>{a}</Goo>;
                    }
                  ",
            Some(serde_json::json!(["always", { "destructureInSignature": "always" }])),
            None,
        ),
        (
            "
                    import { useContext } from 'react';

                    const MyComponent = (props) => {
                      const {foo} = useContext(aContext);
                      return <div>{foo}</div>
                    };
                  ",
            Some(serde_json::json!(["always"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.9.0" } } })),
        ),
        (
            "
                    import { useContext } from 'react';

                    const MyComponent = (props) => {
                      const foo = useContext(aContext);
                      return <div>{foo.test}</div>
                    };
                  ",
            Some(serde_json::json!(["never"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.9.0" } } })),
        ),
        (
            "
                    import { useContext } from 'react';

                    const MyComponent = (props) => {
                      const foo = useContext(aContext);
                      return <div>{foo.test}</div>
                    };
                  ",
            Some(serde_json::json!(["always"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.9.0" } } })),
        ),
        (
            "
                    const MyComponent = (props) => {
                      const foo = useContext(aContext);
                      return <div>{foo.test}</div>
                    };
                  ",
            Some(serde_json::json!(["always"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.8.999" } } })),
        ),
        (
            "
                    const MyComponent = (props) => {
                      const {foo} = useContext(aContext);
                      return <div>{foo}</div>
                    };
                  ",
            Some(serde_json::json!(["never"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.8.999" } } })),
        ),
        (
            "
                    const MyComponent = (props) => {
                      const {foo} = useContext(aContext);
                      return <div>{foo}</div>
                    };
                  ",
            Some(serde_json::json!(["always"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.8.999" } } })),
        ),
        (
            "
                    const MyComponent = (props) => {
                      const foo = useContext(aContext);
                      return <div>{foo.test}</div>
                    };
                  ",
            Some(serde_json::json!(["never"])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.8.999" } } })),
        ),
        (
            "
                    import { useContext } from 'react';

                    const MyComponent = (props) => {
                      const foo = useContext(aContext);
                      return <div>{foo?.test}</div>
                    };
                  ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
                    const MyComponent = (props) => {
                      return (<div id={props.id} />)
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    const MyComponent = ({ id, className }) => (
                      <div id={id} className={className} />
                    );
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const MyComponent = (props, { color }) => (
                      <div id={props.id} className={props.className} />
                    );
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        return <div>{this.props.foo}</div>;
                      }
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        return <div>{this.state.foo}</div>;
                      }
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        return <div>{this.context.foo}</div>;
                      }
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    class Foo extends React.Component {
                      render() { return this.foo(); }
                      foo() {
                        return this.props.children;
                      }
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    var Hello = createReactClass({
                      render: function() {
                        return <Text>{this.props.foo}</Text>;
                      }
                    });
                  ",
            None,
            None,
        ),
        (
            "
                    module.exports = {
                      Foo(props) {
                        return <p>{props.a}</p>;
                      }
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    export default function Foo(props) {
                      return <p>{props.a}</p>;
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    function hof() {
                      return (props) => <p>{props.a}</p>;
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        const foo = this.props.foo;
                        return <div>{foo}</div>;
                      }
                    };
                    ",
            None,
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        const { foo } = this.props;
                        return <div>{foo}</div>;
                      }
                    };
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const MyComponent = (props) => {
                      const { id, className } = props;
                      return <div id={id} className={className} />
                    };
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const Foo = class extends React.PureComponent {
                      render() {
                        const { foo } = this.state;
                        return <div>{foo}</div>;
                      }
                    };
                  ",
            Some(serde_json::json!(["never"])),
            None,
        ),
        (
            "
                    const columns = [
                      {
                        CustomComponentName: function(props) {
                          if (props.url) {
                            return (
                              <a href={props.url}>
                                {props.test}
                              </a>
                            );
                          }
                          return null;
                        },
                      },
                    ];
                  ",
            None,
            None,
        ),
        (
            "
                    function Foo(props, context) {
                      const d = context.describe();
                      return <div>{d}</div>;
                    }
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    export default (props) => {
                      const match = props.str.match(/some expression/);
                      if (match) {
                        return <span>jsx</span>;
                      }
                      return null;
                    };
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
        (
            "
                    import React from 'react';

                    const TestComp = (props) => {
                      props.onClick3102();

                      return (
                        <div
                          onClick={(evt) => {
                            if (props.onClick3102) {
                              props.onClick3102(evt);
                            }
                          }}
                        >
                          <div />
                        </div>
                      );
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    export const revisionStates2 = {
                        [A.b]: props => {
                          return props.editor !== null
                            ? <span>{props.editor}</span>
                            : null
                        },
                    };
                  ",
            None,
            None,
        ),
        (
            "
                    export function hof(namespace) {
                      const initialState = {
                        bounds: null,
                        search: false,
                      };
                      return (props) => {
                        if (props.y) {
                          return <span>{props.y}</span>;
                        }
                        return <span>{props.x}</span>
                      };
                    }
                  ",
            None,
            None,
        ),
        (
            "
                    type Props = { text: string };
                    export const MyComponent: React.FC<Props> = (props) => {
                      type MyType = typeof props.text;
                      return <div>{props.text as MyType}</div>;
                    };
                  ",
            Some(serde_json::json!(["always", { "destructureInSignature": "always" }])),
            None,
        ),
        (
            "
                    type Props = { text: string };
                    export const MyOtherComponent: React.FC<Props> = (props) => {
                      const { text } = props;
                      type MyType = typeof props.text;
                      return <div>{text as MyType}</div>;
                    };
                  ",
            Some(serde_json::json!(["always", { "destructureInSignature": "always" }])),
            None,
        ),
        (
            "
                    function C(props: Props) {
                      void props.a
                      typeof props.b
                      return <div />
                    }
                  ",
            Some(serde_json::json!(["always"])),
            None,
        ),
    ];

    let fix = vec![
        (
            "function Foo(props) { const {a} = props; return <>{a}</> }",
            "function Foo({a}) { return <>{a}</> }",
            Some(serde_json::json!(["always", { "destructureInSignature": "always" }])),
        ),
        (
            "function Foo(props: FooProps) { const {a} = props; return <>{a}</> }",
            "function Foo({a}: FooProps) { return <>{a}</> }",
            Some(serde_json::json!(["always", { "destructureInSignature": "always" }])),
        ),
    ];

    Tester::new(DestructuringAssignment::NAME, DestructuringAssignment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
