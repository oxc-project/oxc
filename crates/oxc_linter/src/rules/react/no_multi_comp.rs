use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, CallExpression, ExportDefaultDeclarationKind, Expression,
        PropertyKey, Statement, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::ContextHost,
    utils::{
        expression_contains_jsx, function_body_contains_jsx, function_contains_jsx,
        is_es5_component, is_es6_component, is_react_component_name,
    },
};

fn no_multi_comp_diagnostic(component_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Declare only one React component per file. Found: {component_name}"
    ))
    .with_help("Move this component to a separate file.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoMultiCompConfig {
    // When `true`, the rule will ignore stateless components and will allow you to have multiple
    // stateless components in the same file. Or one stateful component and one-or-more stateless
    // components in the same file.
    //
    // Stateless basically just means function components, including those created via
    // `memo` and `forwardRef`.
    ignore_stateless: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoMultiComp(NoMultiCompConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents multiple React components from being defined in the same file.
    ///
    /// ### Why is this bad?
    ///
    /// Declaring multiple components in a single file can make it harder to navigate
    /// and maintain the codebase. Each component should ideally be in its own file
    /// for better organization and reusability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Foo({ name }) {
    ///   return <div>Hello {name}</div>;
    /// }
    ///
    /// function Bar({ name }) {
    ///   return <div>Hello again {name}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Foo({ name }) {
    ///   return <div>Hello {name}</div>;
    /// }
    /// ```
    NoMultiComp,
    react,
    restriction,
    none,
    config = NoMultiComp,
);

/// Represents a detected React component
#[derive(Debug, Clone)]
struct DetectedComponent {
    name: String,
    span: Span,
    is_stateless: bool,
}

impl Rule for NoMultiComp {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut components: Vec<DetectedComponent> = Vec::new();

        // Iterate through all AST nodes to find components
        for node in ctx.nodes().iter() {
            if let Some(component) = detect_component(node, ctx) {
                components.push(component);
            }
        }

        // Filter components based on ignoreStateless option
        let relevant_components: Vec<&DetectedComponent> = if self.0.ignore_stateless {
            components.iter().filter(|c| !c.is_stateless).collect()
        } else {
            components.iter().collect()
        };

        // Report all components after the first one
        for component in relevant_components.into_iter().skip(1) {
            ctx.diagnostic(no_multi_comp_diagnostic(&component.name, component.span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Detect if an AST node represents a React component
fn detect_component(node: &AstNode, ctx: &LintContext) -> Option<DetectedComponent> {
    match node.kind() {
        // ES6 class components: class Foo extends React.Component
        AstKind::Class(class) => {
            if is_es6_component(node) {
                let name = class
                    .id
                    .as_ref()
                    .map_or_else(|| "UnnamedComponent".into(), |id| id.name.to_string());
                return Some(DetectedComponent { name, span: class.span, is_stateless: false });
            }
            None
        }

        // ES5 components: createReactClass({...})
        AstKind::CallExpression(call) => {
            if is_es5_component(node) {
                let name =
                    get_component_name_from_parent(node, ctx).unwrap_or("UnnamedComponent".into());
                return Some(DetectedComponent { name, span: call.span, is_stateless: false });
            }

            // Note: We don't detect memo/forwardRef HOC calls here.
            // They are detected via VariableDeclarator to avoid double-counting
            // and to properly get the component name.

            None
        }

        // Function declarations: function Foo() { return <div/> }
        AstKind::Function(func) => {
            let Some(func_id) = &func.id else {
                return None;
            };

            if is_react_component_name(&func_id.name)
                && function_contains_jsx(func)
                && !is_inside_component(node, ctx)
            {
                return Some(DetectedComponent {
                    name: func_id.name.to_string(),
                    span: func.span,
                    is_stateless: true,
                });
            }
            None
        }

        // Variable declarations: const Foo = () => <div/>
        AstKind::VariableDeclarator(decl) => detect_variable_component(decl, node, ctx),

        // Export default HOC: export default React.forwardRef(...)
        AstKind::ExportDefaultDeclaration(export_decl) => {
            // Check if the exported value is a HOC call
            if let ExportDefaultDeclarationKind::CallExpression(call) = &export_decl.declaration
                && is_hoc_component(call, ctx) {
                    return Some(DetectedComponent {
                        name: "UnnamedComponent".into(),
                        span: export_decl.span,
                        is_stateless: true,
                    });
                }
            None
        }

        // Object property methods: { RenderFoo() { return <div/> } }
        AstKind::ObjectProperty(prop) => {
            if let PropertyKey::StaticIdentifier(id) = &prop.key
                && is_react_component_name(&id.name)
                    && let Expression::FunctionExpression(func) = &prop.value
                        && function_contains_jsx(func) && !is_inside_component(node, ctx) {
                            return Some(DetectedComponent {
                                name: id.name.to_string(),
                                span: prop.span,
                                is_stateless: true,
                            });
                        }
            None
        }

        // Assignment expressions: exports.Foo = function() { return <div/> }
        AstKind::AssignmentExpression(assign) => {
            if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
                let prop_name = member.property.name.as_str();
                if is_react_component_name(prop_name) {
                    // Direct function with JSX
                    if expression_contains_jsx(&assign.right) && !is_inside_component(node, ctx) {
                        return Some(DetectedComponent {
                            name: prop_name.to_string(),
                            span: assign.span,
                            is_stateless: true,
                        });
                    }

                    // Function that returns a component (factory)
                    if let Expression::FunctionExpression(func) = &assign.right
                        && returns_component(func) && !is_inside_component(node, ctx) {
                            return Some(DetectedComponent {
                                name: prop_name.to_string(),
                                span: assign.span,
                                is_stateless: true,
                            });
                        }
                }
            }
            None
        }

        _ => None,
    }
}

/// Detect component from variable declarator
fn detect_variable_component(
    decl: &VariableDeclarator,
    node: &AstNode,
    ctx: &LintContext,
) -> Option<DetectedComponent> {
    let name = decl.id.get_identifier_name()?.to_string();

    if !is_react_component_name(&name) {
        return None;
    }

    let init = decl.init.as_ref()?;

    if is_inside_component(node, ctx) {
        return None;
    }

    // Unwrap parenthesized expression if needed
    let init = if let Expression::ParenthesizedExpression(paren) = init {
        &paren.expression
    } else {
        init
    };

    // Arrow function or function expression with JSX
    if expression_contains_jsx(init) {
        return Some(DetectedComponent { name, span: decl.span, is_stateless: true });
    }

    // Sequence expression: const Foo = (0, () => <div/>)
    if let Expression::SequenceExpression(seq) = init
        && let Some(last) = seq.expressions.last()
            && expression_contains_jsx(last) {
                return Some(DetectedComponent { name, span: decl.span, is_stateless: true });
            }

    // HOC: const Foo = memo(() => <div/>)
    if let Expression::CallExpression(call) = init
        && is_hoc_component(call, ctx) {
            return Some(DetectedComponent { name, span: decl.span, is_stateless: true });
        }

    None
}

/// Check if a call expression is a HOC (memo/forwardRef) wrapping a component
fn is_hoc_component(call: &CallExpression, ctx: &LintContext) -> bool {
    let callee_name = get_hoc_callee_name(call, ctx);

    let is_hoc = matches!(
        callee_name.as_deref(),
        Some("memo" | "forwardRef" | "React.memo" | "React.forwardRef")
    );

    if !is_hoc {
        return false;
    }

    // Check if the first argument is a function with JSX
    let Some(first_arg) = call.arguments.first() else {
        return false;
    };

    match first_arg {
        Argument::FunctionExpression(func) => {
            // Skip if it's just a pass-through wrapper (returning a single component)
            if is_passthrough_function(func) {
                return false;
            }
            function_contains_jsx(func)
        }
        Argument::ArrowFunctionExpression(arrow) => {
            // Skip if it's just a pass-through wrapper
            if is_passthrough_arrow(arrow) {
                return false;
            }
            function_body_contains_jsx(&arrow.body)
        }
        _ => false,
    }
}

/// Get the name of a HOC callee, resolving local aliases
fn get_hoc_callee_name(call: &CallExpression, ctx: &LintContext) -> Option<String> {
    // Direct name like React.memo or memo
    if let Some(name) = call.callee_name() {
        return Some(name.to_string());
    }

    // Check for aliased imports: const myMemo = React.memo
    if let Expression::Identifier(ident) = &call.callee {
        // Check if this identifier is bound to a HOC
        let scoping = ctx.scoping();
        if let Some(symbol_id) = scoping.get_binding(scoping.root_scope_id(), &ident.name) {
            let decl_id = ctx.scoping().symbol_declaration(symbol_id);
            let decl_node = ctx.nodes().get_node(decl_id);

            if let AstKind::VariableDeclarator(var_decl) = decl_node.kind()
                && let Some(init) = &var_decl.init {
                    // const forwardRef = React.forwardRef
                    if let Expression::StaticMemberExpression(member) = init
                        && let Expression::Identifier(obj) = &member.object
                            && obj.name == "React"
                                && matches!(member.property.name.as_str(), "memo" | "forwardRef")
                            {
                                return Some(format!("React.{}", member.property.name));
                            }
                }
        }

        // Might be an imported or destructured HOC
        if matches!(ident.name.as_str(), "memo" | "forwardRef") {
            return Some(ident.name.to_string());
        }
    }

    None
}

/// Check if a function just passes through to a single JSX component
fn is_passthrough_function(func: &oxc_ast::ast::Function) -> bool {
    let Some(body) = &func.body else {
        return false;
    };

    if body.statements.len() != 1 {
        return false;
    }

    if let Statement::ReturnStatement(ret) = &body.statements[0]
        && let Some(arg) = &ret.argument {
            return is_simple_jsx_passthrough(arg);
        }

    false
}

/// Check if an arrow function just passes through to a single JSX component
fn is_passthrough_arrow(arrow: &oxc_ast::ast::ArrowFunctionExpression) -> bool {
    // Expression arrow: `() => <Comp {...props} />`
    if let Some(expr) = arrow.get_expression() {
        return is_simple_jsx_passthrough(expr);
    }

    // Block body with single return: `() => { return <Comp {...props} />; }`
    if arrow.body.statements.len() == 1
        && let Statement::ReturnStatement(ret) = &arrow.body.statements[0]
            && let Some(arg) = &ret.argument {
                return is_simple_jsx_passthrough(arg);
            }

    false
}

/// Check if an expression is a simple JSX element that just renders another component
/// This is for cases like: React.forwardRef((props, ref) => <MyComp {...props} ref={ref} />)
fn is_simple_jsx_passthrough(expr: &Expression) -> bool {
    if let Expression::JSXElement(jsx) = expr {
        // Check if it's rendering another component (PascalCase name)
        if let oxc_ast::ast::JSXElementName::IdentifierReference(id) = &jsx.opening_element.name
            && is_react_component_name(&id.name) {
                // Only consider it a passthrough if it spreads props (like {...props})
                // A component that doesn't pass props isn't a simple wrapper
                let has_spread =
                    jsx.opening_element.attributes.iter().any(|attr| {
                        matches!(attr, oxc_ast::ast::JSXAttributeItem::SpreadAttribute(_))
                    });
                return has_spread && jsx.opening_element.attributes.len() <= 2;
            }
    }
    false
}

/// Check if a function returns another function with JSX (component factory)
fn returns_component(func: &oxc_ast::ast::Function) -> bool {
    let Some(body) = &func.body else {
        return false;
    };

    for stmt in &body.statements {
        if let Statement::ReturnStatement(ret) = stmt
            && let Some(arg) = &ret.argument {
                match arg {
                    Expression::FunctionExpression(inner) => {
                        return function_contains_jsx(inner);
                    }
                    Expression::ArrowFunctionExpression(inner) => {
                        return function_body_contains_jsx(&inner.body);
                    }
                    _ => {}
                }
            }
    }

    false
}

/// Get component name from parent node (for anonymous components)
fn get_component_name_from_parent(node: &AstNode, ctx: &LintContext) -> Option<String> {
    for ancestor in ctx.nodes().ancestors(node.id()).skip(1) {
        match ancestor.kind() {
            AstKind::VariableDeclarator(decl) => {
                return decl.id.get_identifier_name().map(|s| s.to_string());
            }
            AstKind::AssignmentExpression(assign) => {
                if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                    return Some(id.name.to_string());
                }
            }
            _ => continue,
        }
    }
    None
}

/// Check if a node is inside another component (nested component)
fn is_inside_component(node: &AstNode, ctx: &LintContext) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()).skip(1) {
        // Inside a class component
        if is_es6_component(ancestor) || is_es5_component(ancestor) {
            return true;
        }

        // Inside a function component
        if let AstKind::Function(func) = ancestor.kind()
            && let Some(id) = &func.id
                && is_react_component_name(&id.name) && function_contains_jsx(func) {
                    return true;
                }

        // Inside an arrow function component
        if let AstKind::ArrowFunctionExpression(arrow) = ancestor.kind()
            && function_body_contains_jsx(&arrow.body) {
                // Check if this arrow is assigned to a component-named variable
                let parent = ctx.nodes().parent_node(ancestor.id());
                if let AstKind::VariableDeclarator(decl) = parent.kind()
                    && let Some(name) = decl.id.get_identifier_name()
                        && is_react_component_name(name.as_str()) {
                            return true;
                        }
            }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"var Hello = require('./components/Hello');
            var HelloJohn = createReactClass({
              render: function() {
                return <Hello name="John" />;
              }
            });"#, None, None),
        ("class Hello extends React.Component {
            render() {
              return <div>Hello {this.props.name}</div>;
            }
          }", None, None),
        ("var Heading = createReactClass({
            render: function() {
              return (
                <div>
                  {this.props.buttons.map(function(button, index) {
                    return <Button {...button} key={index}/>;
                  })}
                </div>
              );
            }
          });", None, None),
        ("function Hello(props) {
            return <div>Hello {props.name}</div>;
          }
          function HelloAgain(props) {
            return <div>Hello again {props.name}</div>;
          }", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
        (r#"function Hello(props) {
              return <div>Hello {props.name}</div>;
            }
            class HelloJohn extends React.Component {
              render() {
                return <Hello name="John" />;
              }
            }"#, Some(serde_json::json!([{ "ignoreStateless": true }])), None),
        (r#"import React, { createElement } from "react"
            const helperFoo = () => {
              return true;
            };
            function helperBar() {
              return false;
            };
            function RealComponent() {
              return createElement("img");
            };"#, None, None),
        (r#"const Hello = React.memo(function(props) {
              return <div>Hello {props.name}</div>;
            });
            class HelloJohn extends React.Component {
              render() {
                return <Hello name="John" />;
              }
            }"#, Some(serde_json::json!([{ "ignoreStateless": true }])), None),
        ("class StoreListItem extends React.PureComponent {
            // A bunch of stuff here
          }
          export default React.forwardRef((props, ref) => <StoreListItem {...props} forwardRef={ref} />);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("class StoreListItem extends React.PureComponent {
            // A bunch of stuff here
          }
          export default React.forwardRef((props, ref) => {
            return <StoreListItem {...props} forwardRef={ref} />
          });", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const HelloComponent = (props) => {
            return <div></div>;
          }
          export default React.forwardRef((props, ref) => <HelloComponent {...props} forwardRef={ref} />);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("class StoreListItem extends React.PureComponent {
            // A bunch of stuff here
          }
          export default React.forwardRef(
            function myFunction(props, ref) {
              return <StoreListItem {...props} forwardedRef={ref} />;
            }
          );", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
        ("const HelloComponent = (props) => {
            return <div></div>;
          }
          class StoreListItem extends React.PureComponent {
            // A bunch of stuff here
          }
          export default React.forwardRef(
            function myFunction(props, ref) {
              return <StoreListItem {...props} forwardedRef={ref} />;
            }
          );", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
        ("const HelloComponent = (props) => {
              return <div></div>;
            }
            export default React.memo((props, ref) => <HelloComponent {...props} />);", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
        (r#"import React from 'react';
            function memo() {
              var outOfScope = "hello"
              return null;
            }
            class ComponentY extends React.Component {
              memoCities = memo((cities) => cities.map((v) => ({ label: v })));
              render() {
                return (
                  <div>
                    <div>Counter</div>
                  </div>
                );
              }
            }"#, None, None),
        (r#"const MenuList = forwardRef(({onClose, ...props}, ref) => {
              const {t} = useTranslation();
              const handleLogout = useLogoutHandler();

              const onLogout = useCallback(() => {
                onClose();
                handleLogout();
              }, [onClose, handleLogout]);

              return (
                <MuiMenuList ref={ref} {...props}>
                  <MuiMenuItem key="logout" onClick={onLogout}>
                    {t('global-logout')}
                  </MuiMenuItem>
                </MuiMenuList>
              );
            });

            MenuList.displayName = 'MenuList';

            MenuList.propTypes = {
              onClose: PropTypes.func,
            };

            MenuList.defaultProps = {
              onClose: () => null,
            };

            export default MenuList;"#, None, None),
        (r#"const MenuList = forwardRef(({ onClose, ...props }, ref) => {
              const onLogout = useCallback(() => {
                onClose()
              }, [onClose])

              return (
                <BlnMenuList ref={ref} {...props}>
                  <BlnMenuItem key="logout" onClick={onLogout}>
                    Logout
                  </BlnMenuItem>
                </BlnMenuList>
              )
            })

            MenuList.displayName = 'MenuList'

            MenuList.propTypes = {
              onClose: PropTypes.func
            }

            MenuList.defaultProps = {
              onClose: () => null
            }

            export default MenuList"#, None, None),
    ];

    let fail = vec![
        ("function Hello(props) {
            return <div>Hello {props.name}</div>;
          }
          function HelloAgain(props) {
            return <div>Hello again {props.name}</div>;
          }", None, None),
        (r#"function Hello(props) {
              return <div>Hello {props.name}</div>;
            }
            class HelloJohn extends React.Component {
              render() {
                return <Hello name="John" />;
              }
            }"#, None, None),
        ("export default {
            RenderHello(props) {
              let {name} = props;
              return <div>{name}</div>;
            },
            RenderHello2(props) {
              let {name} = props;
              return <div>{name}</div>;
            }
          };", None, None),
        ("exports.Foo = function Foo() {
            return <></>
          }

          exports.createSomeComponent = function createSomeComponent(opts) {
            return function Foo() {
              return <>{opts.a}</>
            }
          }", None, None),
        ("class StoreListItem extends React.PureComponent {
            // A bunch of stuff here
          }
            export default React.forwardRef((props, ref) => <div><StoreListItem {...props} forwardRef={ref} /></div>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const HelloComponent = (props) => {
            return <div></div>;
          }
          const HelloComponent2 = React.forwardRef((props, ref) => <div></div>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = React.forwardRef((props, ref) => <><HelloComponent></HelloComponent></>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const forwardRef = React.forwardRef;
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const memo = React.memo;
          const HelloComponent = (props) => {
            return <div></div>;
          };
          const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const {forwardRef} = React;
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const {memo} = React;
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("import React, { memo } from 'react';
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("import {forwardRef} from 'react';
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const { memo } = require('react');
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const {forwardRef} = require('react');
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const forwardRef = require('react').forwardRef;
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        ("const memo = require('react').memo;
          const HelloComponent = (0, (props) => {
            return <div></div>;
          });
          const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
        // We do not support the pragma option.
        // ("import Foo, { memo, forwardRef } from 'foo';
        // const Text = forwardRef(({ text }, ref) => {
        //   return <div ref={ref}>{text}</div>;
        // })
        // const Label = memo(() => <Text />);", None, Some(serde_json::json!({ "settings": { "react": { "pragma": "Foo", }, } })))
    ];

    Tester::new(NoMultiComp::NAME, NoMultiComp::PLUGIN, pass, fail).test_and_snapshot();
}
