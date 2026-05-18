use fast_glob::glob_match;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrowFunctionExpression, CallExpression, Class, Function, JSXAttributeName,
        JSXExpression, JSXExpressionContainer,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{
        expression_contains_jsx, function_body_contains_jsx, function_contains_jsx,
        is_create_element_call, is_es6_component, is_hoc_call, is_react_component_name,
        is_react_hook,
    },
};

const COMPONENT_AS_PROPS_INFO: &str =
    " If you want to allow component creation in props, set `allowAsProps` option to true.";

fn no_unstable_nested_components_diagnostic(
    span: Span,
    parent_name: Option<&str>,
    is_component_in_prop: bool,
) -> OxcDiagnostic {
    let parent = parent_name.map_or(String::new(), |name| format!(" `{name}`"));
    let mut help = format!(
        "Move this component definition out of the parent component{parent} and pass data as props."
    );
    if is_component_in_prop {
        help.push_str(COMPONENT_AS_PROPS_INFO);
    }

    OxcDiagnostic::warn("Do not define components during render.").with_help(help).with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoUnstableNestedComponentsConfig {
    /// Allow component definitions in props.
    allow_as_props: bool,
    /// Optional custom propTypes validators accepted for eslint-plugin-react compatibility.
    custom_validators: Vec<CompactStr>,
    /// Glob pattern for render-prop names that may receive inline component definitions.
    prop_name_pattern: CompactStr,
}

impl Default for NoUnstableNestedComponentsConfig {
    fn default() -> Self {
        Self {
            allow_as_props: false,
            custom_validators: Vec::new(),
            prop_name_pattern: CompactStr::new("render*"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoUnstableNestedComponents(Box<NoUnstableNestedComponentsConfig>);

impl From<NoUnstableNestedComponentsConfig> for NoUnstableNestedComponents {
    fn from(config: NoUnstableNestedComponentsConfig) -> Self {
        Self(Box::new(config))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows defining React components inside other components.
    ///
    /// ### Why is this bad?
    ///
    /// React compares element types by reference during reconciliation. A component defined during
    /// render gets a new identity on every render, so React remounts the entire nested subtree and
    /// destroys its DOM nodes and state.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component() {
    ///   function UnstableNestedComponent() {
    ///     return <div />;
    ///   }
    ///
    ///   return <UnstableNestedComponent />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function StableComponent() {
    ///   return <div />;
    /// }
    ///
    /// function Component() {
    ///   return <StableComponent />;
    /// }
    /// ```
    NoUnstableNestedComponents,
    react,
    suspicious,
    none,
    config = NoUnstableNestedComponentsConfig,
    version = "next",
);

impl Rule for NoUnstableNestedComponents {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<NoUnstableNestedComponentsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(Self::from)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) => {
                if let Some(candidate) = Self::function_candidate(func, node, ctx) {
                    self.report_candidate(node, ctx, candidate);
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if let Some(candidate) = Self::arrow_candidate(arrow, node, ctx) {
                    self.report_candidate(node, ctx, candidate);
                }
            }
            AstKind::Class(class) => {
                if let Some(candidate) = class_candidate(class, node, ctx) {
                    self.report_candidate(node, ctx, candidate);
                }
            }
            AstKind::CallExpression(call) => {
                if let Some(candidate) = hoc_call_candidate(call, node, ctx) {
                    self.report_candidate(node, ctx, candidate);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[derive(Debug, Clone, Copy)]
struct ComponentCandidate {
    span: Span,
    is_component_in_prop: bool,
}

impl NoUnstableNestedComponents {
    fn report_candidate<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        candidate: ComponentCandidate,
    ) {
        if self.should_ignore_candidate(node, ctx, candidate.is_component_in_prop) {
            return;
        }

        let Some(parent_name) = find_parent_component_name(node, ctx) else {
            return;
        };

        ctx.diagnostic(no_unstable_nested_components_diagnostic(
            candidate.span,
            parent_name.as_deref(),
            candidate.is_component_in_prop && !self.0.allow_as_props,
        ));
    }

    fn function_candidate<'a>(
        func: &Function<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<ComponentCandidate> {
        if is_first_argument_of_hoc_call(node, ctx) {
            return None;
        }

        let is_component_in_prop = is_component_declared_in_prop(node, ctx);
        if is_component_in_prop {
            if function_contains_jsx(func) {
                return Some(ComponentCandidate { span: func.span, is_component_in_prop });
            }
            return None;
        }

        let name = function_name(func, node, ctx)?;
        if is_react_component_name(&name) && function_contains_jsx(func) {
            Some(ComponentCandidate { span: func.span, is_component_in_prop: false })
        } else {
            None
        }
    }

    fn arrow_candidate<'a>(
        arrow: &ArrowFunctionExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<ComponentCandidate> {
        if is_first_argument_of_hoc_call(node, ctx) {
            return None;
        }

        let contains_jsx = function_body_contains_jsx(&arrow.body);
        if !contains_jsx {
            return None;
        }

        let is_component_in_prop = is_component_declared_in_prop(node, ctx);
        if is_component_in_prop {
            return Some(ComponentCandidate { span: arrow.span, is_component_in_prop });
        }

        let name = function_like_name(node, ctx)?;
        if is_react_component_name(&name) {
            Some(ComponentCandidate { span: arrow.span, is_component_in_prop: false })
        } else {
            None
        }
    }

    fn should_ignore_candidate<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        is_component_in_prop: bool,
    ) -> bool {
        is_map_callback(node, ctx)
            || is_return_statement_of_hook(node, ctx)
            || is_allowed_render_prop(node, ctx, &self.0.prop_name_pattern)
            || (is_component_in_prop && self.0.allow_as_props)
    }
}

fn class_candidate<'a>(
    class: &Class<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<ComponentCandidate> {
    if !is_es6_component(node) {
        return None;
    }

    let is_component_in_prop = is_component_declared_in_prop(node, ctx);
    if is_component_in_prop {
        return Some(ComponentCandidate { span: class.span, is_component_in_prop });
    }

    let name = class_name(class, node, ctx)?;
    if is_react_component_name(&name) {
        Some(ComponentCandidate { span: class.span, is_component_in_prop: false })
    } else {
        None
    }
}

fn hoc_call_candidate<'a>(
    call: &CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<ComponentCandidate> {
    if is_first_argument_of_hoc_call(node, ctx) || !is_hoc_component_call(call, ctx) {
        return None;
    }

    Some(ComponentCandidate {
        span: call.span,
        is_component_in_prop: is_component_declared_in_prop(node, ctx),
    })
}

enum ParentComponentName {
    Named(String),
    Anonymous,
}

impl ParentComponentName {
    fn as_deref(&self) -> Option<&str> {
        match self {
            Self::Named(name) => Some(name),
            Self::Anonymous => None,
        }
    }
}

fn find_parent_component_name(
    node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> Option<ParentComponentName> {
    for ancestor_id in ctx.nodes().ancestor_ids(node.id()).filter(|&id| id != node.id()) {
        let ancestor = ctx.nodes().get_node(ancestor_id);
        match ancestor.kind() {
            AstKind::Function(func) => {
                if is_first_argument_of_hoc_call(ancestor, ctx) {
                    continue;
                }
                if !function_contains_jsx(func) {
                    continue;
                }

                if let Some(name) = function_name(func, ancestor, ctx) {
                    if is_react_component_name(&name) {
                        return Some(ParentComponentName::Named(name));
                    }
                    continue;
                }

                if is_anonymous_default_export(ancestor, ctx) {
                    return Some(ParentComponentName::Anonymous);
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if is_first_argument_of_hoc_call(ancestor, ctx)
                    || !function_body_contains_jsx(&arrow.body)
                {
                    continue;
                }

                if let Some(name) = function_like_name(ancestor, ctx) {
                    if is_react_component_name(&name) {
                        return Some(ParentComponentName::Named(name));
                    }
                    continue;
                }

                if is_anonymous_default_export(ancestor, ctx) {
                    return Some(ParentComponentName::Anonymous);
                }
            }
            AstKind::Class(class) => {
                if is_es6_component(ancestor)
                    && let Some(name) = class_name(class, ancestor, ctx)
                    && is_react_component_name(&name)
                {
                    return Some(ParentComponentName::Named(name));
                }
            }
            AstKind::CallExpression(call) => {
                if is_first_argument_of_hoc_call(ancestor, ctx) || !is_hoc_component_call(call, ctx)
                {
                    continue;
                }

                if let Some(name) = function_like_name(ancestor, ctx)
                    && is_react_component_name(&name)
                {
                    return Some(ParentComponentName::Named(name));
                }

                if let Some(name) = hoc_first_argument_name(call)
                    && is_react_component_name(&name)
                {
                    return Some(ParentComponentName::Named(name));
                }

                return Some(ParentComponentName::Anonymous);
            }
            _ => {}
        }
    }
    None
}

fn function_name(func: &Function<'_>, node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<String> {
    func.name().map(|name| name.to_string()).or_else(|| function_like_name(node, ctx))
}

fn function_like_name(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<String> {
    let parent = ctx.nodes().parent_node(node.id());
    match parent.kind() {
        AstKind::VariableDeclarator(decl) => {
            decl.id.get_identifier_name().map(|name| name.to_string())
        }
        AstKind::ObjectProperty(prop) => prop.key.static_name().map(std::borrow::Cow::into_owned),
        AstKind::AssignmentExpression(assign) => {
            assign.left.get_identifier_name().map(ToString::to_string)
        }
        _ => None,
    }
}

fn hoc_first_argument_name(call: &CallExpression<'_>) -> Option<String> {
    let first_arg = call.arguments.first()?;
    match first_arg {
        Argument::FunctionExpression(func) => func.name().map(|name| name.to_string()),
        Argument::CallExpression(call) => hoc_first_argument_name(call),
        _ => None,
    }
}

fn class_name(class: &Class<'_>, node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<String> {
    class.name().map(|name| name.to_string()).or_else(|| function_like_name(node, ctx))
}

fn is_anonymous_default_export(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.nodes().parent_node(node.id()).kind(), AstKind::ExportDefaultDeclaration(_))
}

fn is_component_declared_in_prop(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());

    if matches!(parent.kind(), AstKind::ObjectProperty(_)) {
        return true;
    }

    if is_in_jsx_attribute_expression(node, ctx) {
        return true;
    }

    is_inside_create_element_props_object(node, ctx)
}

fn is_in_jsx_attribute_expression(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let mut previous_id = node.id();
    for ancestor_id in ctx.nodes().ancestor_ids(node.id()).filter(|&id| id != node.id()) {
        let ancestor = ctx.nodes().get_node(ancestor_id);
        match ancestor.kind() {
            AstKind::JSXExpressionContainer(_) => {
                let parent = ctx.nodes().parent_node(ancestor.id());
                return matches!(parent.kind(), AstKind::JSXAttribute(_));
            }
            AstKind::JSXElement(_) | AstKind::JSXFragment(_) => return false,
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) | AstKind::Class(_)
                if ancestor.id() != previous_id =>
            {
                return false;
            }
            _ => {}
        }
        previous_id = ancestor.id();
    }
    false
}

fn is_inside_create_element_props_object(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    for ancestor_id in ctx.nodes().ancestor_ids(node.id()).filter(|&id| id != node.id()) {
        let ancestor = ctx.nodes().get_node(ancestor_id);
        if let AstKind::ObjectExpression(obj) = ancestor.kind() {
            let parent = ctx.nodes().parent_node(ancestor.id());
            let AstKind::CallExpression(call) = parent.kind() else {
                continue;
            };
            if !is_create_element_call(call) {
                continue;
            }
            return call
                .arguments
                .get(1)
                .is_some_and(|arg| matches!(arg, Argument::ObjectExpression(arg_obj) if arg_obj.span == obj.span));
        }
    }
    false
}

fn is_allowed_render_prop(node: &AstNode<'_>, ctx: &LintContext<'_>, pattern: &str) -> bool {
    if is_direct_jsx_child_render_prop(node, ctx) {
        return true;
    }

    if let Some(prop_name) = nearest_jsx_attribute_name(node, ctx) {
        return prop_name == "children" || glob_match(pattern, &prop_name);
    }

    if let Some(prop_name) = direct_object_property_name(node, ctx) {
        return prop_name == "children" || glob_match(pattern, &prop_name);
    }

    false
}

fn nearest_jsx_attribute_name(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<String> {
    for ancestor_id in ctx.nodes().ancestor_ids(node.id()).filter(|&id| id != node.id()) {
        let ancestor = ctx.nodes().get_node(ancestor_id);
        if let AstKind::JSXExpressionContainer(_) = ancestor.kind() {
            let parent = ctx.nodes().parent_node(ancestor.id());
            let AstKind::JSXAttribute(attr) = parent.kind() else {
                return None;
            };
            return match &attr.name {
                JSXAttributeName::Identifier(id) => Some(id.name.to_string()),
                JSXAttributeName::NamespacedName(_) => None,
            };
        }
        if matches!(ancestor.kind(), AstKind::JSXElement(_) | AstKind::JSXFragment(_)) {
            return None;
        }
    }
    None
}

fn direct_object_property_name(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<String> {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::ObjectProperty(prop) = parent.kind() else {
        return None;
    };
    prop.key.static_name().map(std::borrow::Cow::into_owned)
}

fn is_direct_jsx_child_render_prop(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::JSXExpressionContainer(JSXExpressionContainer { expression, .. }) = parent.kind()
    else {
        return false;
    };

    if !jsx_expression_matches_node(expression, node.kind().span()) {
        return false;
    }

    matches!(ctx.nodes().parent_node(parent.id()).kind(), AstKind::JSXElement(_))
}

fn jsx_expression_matches_node(expression: &JSXExpression<'_>, span: Span) -> bool {
    match expression {
        JSXExpression::ArrowFunctionExpression(arrow) => arrow.span == span,
        JSXExpression::FunctionExpression(func) => func.span == span,
        _ => false,
    }
}

fn is_map_callback(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::CallExpression(call) = parent.kind() else {
        return false;
    };

    if call
        .callee
        .as_member_expression()
        .and_then(oxc_ast::ast::MemberExpression::static_property_name)
        != Some("map")
    {
        return false;
    }

    call.arguments.first().is_some_and(|arg| arg.span() == node.kind().span())
}

fn is_return_statement_of_hook(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    if !matches!(parent.kind(), AstKind::ReturnStatement(_)) {
        return false;
    }

    for ancestor_id in ctx.nodes().ancestor_ids(node.id()).filter(|&id| id != node.id()) {
        let ancestor = ctx.nodes().get_node(ancestor_id);
        if let AstKind::CallExpression(call) = ancestor.kind() {
            return is_react_hook(&call.callee);
        }
    }
    false
}

fn is_first_argument_of_hoc_call(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::CallExpression(call) = parent.kind() else {
        return false;
    };
    is_hoc_component_call(call, ctx)
        && call.arguments.first().is_some_and(|arg| arg.span() == node.kind().span())
}

fn is_hoc_component_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    call.callee_name().is_some_and(|name| is_hoc_call(name, ctx))
        && call.arguments.first().is_some_and(|arg| argument_contains_jsx(arg, ctx))
}

fn argument_contains_jsx(arg: &Argument<'_>, ctx: &LintContext<'_>) -> bool {
    match arg {
        Argument::FunctionExpression(func) => function_contains_jsx(func),
        Argument::ArrowFunctionExpression(arrow) => function_body_contains_jsx(&arrow.body),
        Argument::CallExpression(call) => is_hoc_component_call(call, ctx),
        _ => arg.as_expression().is_some_and(expression_contains_jsx),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                    function ParentComponent() {
                      return (
                        <div>
                          <OutsideDefinedFunctionComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(OutsideDefinedFunctionComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <SomeComponent
                          footer={<OutsideDefinedComponent />}
                          header={<div />}
                          />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      return React.createElement(SomeComponent, {
                        footer: React.createElement(OutsideDefinedComponent, null),
                        header: React.createElement("div", null)
                      });
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const MemoizedNestedComponent = React.useCallback(() => <div />, []);
            
                      return (
                        <div>
                          <MemoizedNestedComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      const MemoizedNestedComponent = React.useCallback(
                        () => React.createElement("div", null),
                        []
                      );
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(MemoizedNestedComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const MemoizedNestedFunctionComponent = React.useCallback(
                        function () {
                          return <div />;
                        },
                        []
                      );
            
                      return (
                        <div>
                          <MemoizedNestedFunctionComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      const MemoizedNestedFunctionComponent = React.useCallback(
                        function () {
                          return React.createElement("div", null);
                        },
                        []
                      );
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(MemoizedNestedFunctionComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent(props) {
                      // Should not interfere handler declarations
                      function onClick(event) {
                        props.onClick(event.target.value);
                      }
            
                      const onKeyPress = () => null;
            
                      function getOnHover() {
                        return function onHover(event) {
                          props.onHover(event.target);
                        }
                      }
            
                      return (
                        <div>
                          <button
                            onClick={onClick}
                            onKeyPress={onKeyPress}
                            onHover={getOnHover()}
            
                            // These should not be considered as components
                            maybeComponentOrHandlerNull={() => null}
                            maybeComponentOrHandlerUndefined={() => undefined}
                            maybeComponentOrHandlerBlank={() => ''}
                            maybeComponentOrHandlerString={() => 'hello-world'}
                            maybeComponentOrHandlerNumber={() => 42}
                            maybeComponentOrHandlerArray={() => []}
                            maybeComponentOrHandlerObject={() => {}} />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      function getComponent() {
                        return <div />;
                      }
            
                      return (
                        <div>
                          {getComponent()}
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      function getComponent() {
                        return React.createElement("div", null);
                      }
            
                      return React.createElement("div", null, getComponent());
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                        return (
                          <RenderPropComponent>
                            {() => <div />}
                          </RenderPropComponent>
                        );
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                        return (
                          <RenderPropComponent children={() => <div />} />
                        );
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <ComplexRenderPropComponent
                          listRenderer={data.map((items, index) => (
                            <ul>
                              {items[index].map((item) =>
                                <li>
                                  {item}
                                </li>
                              )}
                            </ul>
                          ))
                          }
                        />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      return React.createElement(
                          RenderPropComponent,
                          null,
                          () => React.createElement("div", null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent(props) {
                      return (
                        <ul>
                          {props.items.map(item => (
                            <li key={item.id}>
                              {item.name}
                            </li>
                          ))}
                        </ul>
                      );
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent(props) {
                      return (
                        <List items={props.items.map(item => {
                          return (
                            <li key={item.id}>
                              {item.name}
                            </li>
                          );
                        })}
                        />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent(props) {
                      return React.createElement(
                        "ul",
                        null,
                        props.items.map(() =>
                          React.createElement(
                            "li",
                            { key: item.id },
                            item.name
                          )
                        )
                      )
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent(props) {
                      return (
                        <ul>
                          {props.items.map(function Item(item) {
                            return (
                              <li key={item.id}>
                                {item.name}
                              </li>
                            );
                          })}
                        </ul>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent(props) {
                      return React.createElement(
                        "ul",
                        null,
                        props.items.map(function Item() {
                          return React.createElement(
                            "li",
                            { key: item.id },
                            item.name
                          );
                        })
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function createTestComponent(props) {
                      return (
                        <div />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function createTestComponent(props) {
                      return React.createElement("div", null);
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <ComponentWithProps footer={() => <div />} />
                      );
                    }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true }])),
        ),
        (
            r#"
                    function ParentComponent() {
                      return React.createElement(ComponentWithProps, {
                        footer: () => React.createElement("div", null)
                      });
                    }
                  "#,
            Some(serde_json::json!([{ "allowAsProps": true }])),
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <SomeComponent item={{ children: () => <div /> }} />
                      )
                    }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true }])),
        ),
        (
            "
                  function ParentComponent() {
                    return (
                      <SomeComponent>
                        {
                          thing.match({
                            renderLoading: () => <div />,
                            renderSuccess: () => <div />,
                            renderFailure: () => <div />,
                          })
                        }
                      </SomeComponent>
                    )
                  }
                  ",
            None,
        ),
        (
            "
                  function ParentComponent() {
                    const thingElement = thing.match({
                      renderLoading: () => <div />,
                      renderSuccess: () => <div />,
                      renderFailure: () => <div />,
                    });
                    return (
                      <SomeComponent>
                        {thingElement}
                      </SomeComponent>
                    )
                  }
                  ",
            None,
        ),
        (
            "
                  function ParentComponent() {
                    return (
                      <SomeComponent>
                        {
                          thing.match({
                            loading: () => <div />,
                            success: () => <div />,
                            failure: () => <div />,
                          })
                        }
                      </SomeComponent>
                    )
                  }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true, }])),
        ),
        (
            "
                  function ParentComponent() {
                    const thingElement = thing.match({
                      loading: () => <div />,
                      success: () => <div />,
                      failure: () => <div />,
                    });
                    return (
                      <SomeComponent>
                        {thingElement}
                      </SomeComponent>
                    )
                  }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true, }])),
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <ComponentForProps renderFooter={() => <div />} />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      return React.createElement(ComponentForProps, {
                        renderFooter: () => React.createElement("div", null)
                      });
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      useEffect(() => {
                        return () => null;
                      });
            
                      return <div />;
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <SomeComponent renderers={{ Header: () => <div /> }} />
                      )
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <SomeComponent renderMenu={() => (
                          <RenderPropComponent>
                            {items.map(item => (
                              <li key={item}>{item}</li>
                            ))}
                          </RenderPropComponent>
                        )} />
                      )
                    }
                  ",
            None,
        ),
        (
            "
                    const ParentComponent = () => (
                      <SomeComponent
                        components={[
                          <ul>
                            {list.map(item => (
                              <li key={item}>{item}</li>
                            ))}
                          </ul>,
                        ]}
                      />
                    );
                 ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const rows = [
                        {
                          name: 'A',
                          render: (props) => <Row {...props} />
                        },
                      ];
            
                      return <Table rows={rows} />;
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return <SomeComponent renderers={{ notComponent: () => null }} />;
                    }
                  ",
            None,
        ),
        (
            r#"
                    const ParentComponent = createReactClass({
                      displayName: "ParentComponent",
                      statics: {
                        getSnapshotBeforeUpdate: function () {
                          return null;
                        },
                      },
                      render() {
                        return <div />;
                      },
                    });
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const rows = [
                        {
                          name: 'A',
                          notPrefixedWithRender: (props) => <Row {...props} />
                        },
                      ];
            
                      return <Table rows={rows} />;
                    }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true, }])),
        ),
        (
            "
                    function ParentComponent() {
                      return <Table
                        rowRenderer={(rowData) => <Row data={data} />}
                      />
                    }
                  ",
            Some(serde_json::json!([{ "propNamePattern": "*Renderer", }])),
        ),
        (
            "
                    function ParentComponent() {
                      return <SomeComponent footer={React.memo(() => <div />)} />;
                    }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true, }])),
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <SomeComponent
                          footer={class Footer extends React.Component {
                            render() {
                              return <div />;
                            }
                          }}
                        />
                      );
                    }
                  ",
            Some(serde_json::json!([{ "allowAsProps": true, }])),
        ),
    ];

    let fail = vec![
        (
            "
                    function ParentComponent() {
                      function UnstableNestedFunctionComponent() {
                        return <div />;
                      }
            
                      return (
                        <div>
                          <UnstableNestedFunctionComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      function UnstableNestedFunctionComponent() {
                        return React.createElement("div", null);
                      }
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedFunctionComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const UnstableNestedVariableComponent = () => {
                        return <div />;
                      }
            
                      return (
                        <div>
                          <UnstableNestedVariableComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      const UnstableNestedVariableComponent = () => {
                        return React.createElement("div", null);
                      }
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedVariableComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    const ParentComponent = () => {
                      function UnstableNestedFunctionComponent() {
                        return <div />;
                      }
            
                      return (
                        <div>
                          <UnstableNestedFunctionComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    const ParentComponent = () => {
                      function UnstableNestedFunctionComponent() {
                        return React.createElement("div", null);
                      }
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedFunctionComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    export default () => {
                      function UnstableNestedFunctionComponent() {
                        return <div />;
                      }
            
                      return (
                        <div>
                          <UnstableNestedFunctionComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    export default () => {
                      function UnstableNestedFunctionComponent() {
                        return React.createElement("div", null);
                      }
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedFunctionComponent, null)
                      );
                    };
                  "#,
            None,
        ),
        (
            "
                    const ParentComponent = () => {
                      const UnstableNestedVariableComponent = () => {
                        return <div />;
                      }
            
                      return (
                        <div>
                          <UnstableNestedVariableComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    const ParentComponent = () => {
                      const UnstableNestedVariableComponent = () => {
                        return React.createElement("div", null);
                      }
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedVariableComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      class UnstableNestedClassComponent extends React.Component {
                        render() {
                          return <div />;
                        }
                      };
            
                      return (
                        <div>
                          <UnstableNestedClassComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      class UnstableNestedClassComponent extends React.Component {
                        render() {
                          return React.createElement("div", null);
                        }
                      }
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedClassComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    class ParentComponent extends React.Component {
                      render() {
                        class UnstableNestedClassComponent extends React.Component {
                          render() {
                            return <div />;
                          }
                        };
            
                        return (
                          <div>
                            <UnstableNestedClassComponent />
                          </div>
                        );
                      }
                    }
                  ",
            None,
        ),
        (
            r#"
                    class ParentComponent extends React.Component {
                      render() {
                        class UnstableNestedClassComponent extends React.Component {
                          render() {
                            return React.createElement("div", null);
                          }
                        }
            
                        return React.createElement(
                          "div",
                          null,
                          React.createElement(UnstableNestedClassComponent, null)
                        );
                      }
                    }
                  "#,
            None,
        ),
        (
            "
                    class ParentComponent extends React.Component {
                      render() {
                        function UnstableNestedFunctionComponent() {
                          return <div />;
                        }
            
                        return (
                          <div>
                            <UnstableNestedFunctionComponent />
                          </div>
                        );
                      }
                    }
                  ",
            None,
        ),
        (
            r#"
                    class ParentComponent extends React.Component {
                      render() {
                        function UnstableNestedClassComponent() {
                          return React.createElement("div", null);
                        }
            
                        return React.createElement(
                          "div",
                          null,
                          React.createElement(UnstableNestedClassComponent, null)
                        );
                      }
                    }
                  "#,
            None,
        ),
        (
            "
                    class ParentComponent extends React.Component {
                      render() {
                        const UnstableNestedVariableComponent = () => {
                          return <div />;
                        }
            
                        return (
                          <div>
                            <UnstableNestedVariableComponent />
                          </div>
                        );
                      }
                    }
                  ",
            None,
        ),
        (
            r#"
                    class ParentComponent extends React.Component {
                      render() {
                        const UnstableNestedClassComponent = () => {
                          return React.createElement("div", null);
                        }
            
                        return React.createElement(
                          "div",
                          null,
                          React.createElement(UnstableNestedClassComponent, null)
                        );
                      }
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      function getComponent() {
                        function NestedUnstableFunctionComponent() {
                          return <div />;
                        };
            
                        return <NestedUnstableFunctionComponent />;
                      }
            
                      return (
                        <div>
                          {getComponent()}
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      function getComponent() {
                        function NestedUnstableFunctionComponent() {
                          return React.createElement("div", null);
                        }
            
                        return React.createElement(NestedUnstableFunctionComponent, null);
                      }
            
                      return React.createElement("div", null, getComponent());
                    }
                  "#,
            None,
        ),
        (
            "
                    function ComponentWithProps(props) {
                      return <div />;
                    }
            
                    function ParentComponent() {
                      return (
                        <ComponentWithProps
                          footer={
                            function SomeFooter() {
                              return <div />;
                            }
                          } />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ComponentWithProps(props) {
                      return React.createElement("div", null);
                    }
            
                    function ParentComponent() {
                      return React.createElement(ComponentWithProps, {
                        footer: function SomeFooter() {
                          return React.createElement("div", null);
                        }
                      });
                    }
                  "#,
            None,
        ),
        (
            "
                    function ComponentWithProps(props) {
                      return <div />;
                    }
            
                    function ParentComponent() {
                        return (
                          <ComponentWithProps footer={() => <div />} />
                        );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ComponentWithProps(props) {
                      return React.createElement("div", null);
                    }
            
                    function ParentComponent() {
                      return React.createElement(ComponentWithProps, {
                        footer: () => React.createElement("div", null)
                      });
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                        return (
                          <RenderPropComponent>
                            {() => {
                              function UnstableNestedComponent() {
                                return <div />;
                              }
            
                              return (
                                <div>
                                  <UnstableNestedComponent />
                                </div>
                              );
                            }}
                          </RenderPropComponent>
                        );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function RenderPropComponent(props) {
                      return props.render({});
                    }
            
                    function ParentComponent() {
                      return React.createElement(
                        RenderPropComponent,
                        null,
                        () => {
                          function UnstableNestedComponent() {
                            return React.createElement("div", null);
                          }
            
                          return React.createElement(
                            "div",
                            null,
                            React.createElement(UnstableNestedComponent, null)
                          );
                        }
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ComponentForProps(props) {
                      return <div />;
                    }
            
                    function ParentComponent() {
                      return (
                        <ComponentForProps notPrefixedWithRender={() => <div />} />
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ComponentForProps(props) {
                      return React.createElement("div", null);
                    }
            
                    function ParentComponent() {
                      return React.createElement(ComponentForProps, {
                        notPrefixedWithRender: () => React.createElement("div", null)
                      });
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <ComponentForProps someMap={{ Header: () => <div /> }} />
                      );
                    }
                  ",
            None,
        ),
        (
            "
                    class ParentComponent extends React.Component {
                      render() {
                        const List = (props) => {
                          const items = props.items
                            .map((item) => (
                              <li key={item.key}>
                                <span>{item.name}</span>
                              </li>
                            ));
            
                          return <ul>{items}</ul>;
                        };
            
                        return <List {...this.props} />;
                      }
                    }
                  ",
            None,
        ),
        (
            "
                  function ParentComponent() {
                    return (
                      <SomeComponent>
                        {
                          thing.match({
                            loading: () => <div />,
                            success: () => <div />,
                            failure: () => <div />,
                          })
                        }
                      </SomeComponent>
                    )
                  }
                  ",
            None,
        ),
        (
            "
                  function ParentComponent() {
                    const thingElement = thing.match({
                      loading: () => <div />,
                      success: () => <div />,
                      failure: () => <div />,
                    });
                    return (
                      <SomeComponent>
                        {thingElement}
                      </SomeComponent>
                    )
                  }
                  ",
            None,
        ),
        (
            "
                  function ParentComponent() {
                    const rows = [
                      {
                        name: 'A',
                        notPrefixedWithRender: (props) => <Row {...props} />
                      },
                    ];
            
                    return <Table rows={rows} />;
                  }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const UnstableNestedComponent = React.memo(() => {
                        return <div />;
                      });
            
                      return (
                        <div>
                          <UnstableNestedComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      const UnstableNestedComponent = React.memo(
                        () => React.createElement("div", null),
                      );
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const UnstableNestedComponent = React.memo(
                        function () {
                          return <div />;
                        }
                      );
            
                      return (
                        <div>
                          <UnstableNestedComponent />
                        </div>
                      );
                    }
                  ",
            None,
        ),
        (
            r#"
                    function ParentComponent() {
                      const UnstableNestedComponent = React.memo(
                        function () {
                          return React.createElement("div", null);
                        }
                      );
            
                      return React.createElement(
                        "div",
                        null,
                        React.createElement(UnstableNestedComponent, null)
                      );
                    }
                  "#,
            None,
        ),
        (
            "
                    function ParentComponent() {
                      NestedComponent = () => <div />;
                      return <NestedComponent />;
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      const UnstableNestedComponent = React.memo(React.forwardRef(() => <div />));
                      return <UnstableNestedComponent />;
                    }
                  ",
            None,
        ),
        (
            "
                    export default React.forwardRef(function ParentComponent() {
                      const UnstableNestedComponent = () => <div />;
                      return <UnstableNestedComponent />;
                    });
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return <SomeComponent footer={React.memo(() => <div />)} />;
                    }
                  ",
            None,
        ),
        (
            "
                    function ParentComponent() {
                      return (
                        <SomeComponent
                          footer={class Footer extends React.Component {
                            render() {
                              return <div />;
                            }
                          }}
                        />
                      );
                    }
                  ",
            None,
        ),
    ];

    Tester::new(NoUnstableNestedComponents::NAME, NoUnstableNestedComponents::PLUGIN, pass, fail)
        .test_and_snapshot();
}
