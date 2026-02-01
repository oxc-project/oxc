use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentExpression, AssignmentTarget, CallExpression, Class,
        ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression, Function,
        ObjectProperty, PropertyKey, Statement, VariableDeclarator,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::ContextHost,
    utils::{
        expression_contains_jsx, function_body_contains_jsx, function_contains_jsx, is_hoc_call,
        is_react_component_name,
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
        let mut finder = ComponentFinder::new(ctx);
        finder.visit_program(ctx.nodes().program());

        // Filter components based on ignoreStateless option and report all after the first
        let components = finder
            .components
            .iter()
            .filter(|c| if self.0.ignore_stateless { !c.is_stateless } else { true });

        for component in components.skip(1) {
            ctx.diagnostic(no_multi_comp_diagnostic(&component.name, component.span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Visitor that finds React components while tracking nesting depth.
/// Components found while inside another component are not recorded.
struct ComponentFinder<'a, 'ctx> {
    components: Vec<DetectedComponent>,
    component_depth: usize,
    ctx: &'ctx LintContext<'a>,
    /// Track variable name when visiting VariableDeclarator so we can access it in nested visits
    current_var_name: Option<String>,
}

impl<'a, 'ctx> ComponentFinder<'a, 'ctx> {
    fn new(ctx: &'ctx LintContext<'a>) -> Self {
        Self { components: Vec::new(), component_depth: 0, ctx, current_var_name: None }
    }

    fn record_component(&mut self, name: String, span: Span, is_stateless: bool) {
        if self.component_depth == 0 {
            self.components.push(DetectedComponent { name, span, is_stateless });
        }
    }
}

impl<'a> Visit<'a> for ComponentFinder<'a, '_> {
    fn visit_class(&mut self, class: &Class<'a>) {
        if is_es6_component_class(class) {
            let name = class
                .id
                .as_ref()
                .map_or_else(|| "UnnamedComponent".into(), |id| id.name.to_string());
            self.record_component(name, class.span, false);
            self.component_depth += 1;
            walk::walk_class(self, class);
            self.component_depth -= 1;
        } else {
            walk::walk_class(self, class);
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        // Named function that contains JSX: function Foo() { return <div/> }
        if let Some(func_id) = &func.id
            && is_react_component_name(&func_id.name)
            && function_contains_jsx(func)
        {
            self.record_component(func_id.name.to_string(), func.span, true);
            self.component_depth += 1;
            walk::walk_function(self, func, flags);
            self.component_depth -= 1;
        } else {
            walk::walk_function(self, func, flags);
        }
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let Some(component) = detect_variable_component(decl, self.ctx) {
            self.record_component(component.name, component.span, component.is_stateless);
            // Store var name for potential createReactClass detection in nested call
            self.current_var_name = decl.id.get_identifier_name().map(|s| s.to_string());
            self.component_depth += 1;
            walk::walk_variable_declarator(self, decl);
            self.component_depth -= 1;
            self.current_var_name = None;
        } else {
            // Check if this might contain a createReactClass call
            let old_name = self.current_var_name.take();
            self.current_var_name = decl.id.get_identifier_name().map(|s| s.to_string());
            walk::walk_variable_declarator(self, decl);
            self.current_var_name = old_name;
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // ES5 component: createReactClass({...})
        if is_es5_component_call(call) && self.component_depth == 0 {
            let name = self.current_var_name.clone().unwrap_or_else(|| "UnnamedComponent".into());
            self.record_component(name, call.span, false);
            self.component_depth += 1;
            walk::walk_call_expression(self, call);
            self.component_depth -= 1;
        } else {
            walk::walk_call_expression(self, call);
        }
    }

    fn visit_export_default_declaration(&mut self, export_decl: &ExportDefaultDeclaration<'a>) {
        // export default React.forwardRef(...)
        if let ExportDefaultDeclarationKind::CallExpression(call) = &export_decl.declaration
            && is_hoc_component(call, self.ctx)
        {
            self.record_component("UnnamedComponent".into(), export_decl.span, true);
            self.component_depth += 1;
            walk::walk_export_default_declaration(self, export_decl);
            self.component_depth -= 1;
        } else {
            walk::walk_export_default_declaration(self, export_decl);
        }
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        // { RenderFoo() { return <div/> } }
        // Note: Similar to assignment expressions, we don't increment depth here
        // because the inner function will also be detected separately.
        if let PropertyKey::StaticIdentifier(id) = &prop.key
            && is_react_component_name(&id.name)
            && let Expression::FunctionExpression(func) = &prop.value
            && function_contains_jsx(func)
        {
            self.record_component(id.name.to_string(), prop.span, true);
        }
        walk::walk_object_property(self, prop);
    }

    fn visit_assignment_expression(&mut self, assign: &AssignmentExpression<'a>) {
        // exports.Foo = function() { return <div/> }
        // Note: We don't increment depth here because assignment expressions are not
        // considered "containers" that would prevent inner functions from being detected.
        // Both `exports.Foo` and `function Foo()` inside it should be detected separately.
        if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
            let prop_name = member.property.name.as_str();
            if is_react_component_name(prop_name) {
                let is_component = expression_contains_jsx(&assign.right)
                    || matches!(&assign.right, Expression::FunctionExpression(func) if returns_component(func));

                if is_component {
                    self.record_component(prop_name.to_string(), assign.span, true);
                }
            }
        }
        walk::walk_assignment_expression(self, assign);
    }
}

/// Detect component from variable declarator (without ancestor check - handled by visitor depth)
fn detect_variable_component(
    decl: &VariableDeclarator,
    ctx: &LintContext,
) -> Option<DetectedComponent> {
    let name = decl.id.get_identifier_name()?.to_string();

    if !is_react_component_name(&name) {
        return None;
    }

    let init = decl.init.as_ref()?;

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
        && expression_contains_jsx(last)
    {
        return Some(DetectedComponent { name, span: decl.span, is_stateless: true });
    }

    // HOC: const Foo = memo(() => <div/>)
    if let Expression::CallExpression(call) = init
        && is_hoc_component(call, ctx)
    {
        return Some(DetectedComponent { name, span: decl.span, is_stateless: true });
    }

    None
}

/// Check if a call expression is a HOC (memo/forwardRef) wrapping a component
fn is_hoc_component(call: &CallExpression, ctx: &LintContext) -> bool {
    let Some(callee_name) = get_hoc_callee_name(call, ctx) else {
        return false;
    };

    if !is_hoc_call(&callee_name, ctx) {
        return false;
    }

    let Some(first_arg) = call.arguments.first() else {
        return false;
    };

    match first_arg {
        Argument::FunctionExpression(func) => {
            !is_passthrough_function(func) && function_contains_jsx(func)
        }
        Argument::ArrowFunctionExpression(arrow) => {
            !is_passthrough_arrow(arrow) && function_body_contains_jsx(&arrow.body)
        }
        _ => false,
    }
}

/// Get the name of a HOC callee, resolving local aliases
fn get_hoc_callee_name(call: &CallExpression, ctx: &LintContext) -> Option<String> {
    // Direct name like React.memo, memo, or forwardRef
    if let Some(name) = call.callee_name() {
        return Some(name.to_string());
    }

    // Check for aliased imports: const myMemo = React.memo
    let Expression::Identifier(ident) = &call.callee else {
        return None;
    };

    let scoping = ctx.scoping();
    let symbol_id = scoping.get_binding(scoping.root_scope_id(), &ident.name)?;
    let decl_id = scoping.symbol_declaration(symbol_id);
    let decl_node = ctx.nodes().get_node(decl_id);

    let AstKind::VariableDeclarator(var_decl) = decl_node.kind() else {
        return None;
    };

    let Expression::StaticMemberExpression(member) = var_decl.init.as_ref()? else {
        return None;
    };

    let Expression::Identifier(obj) = &member.object else {
        return None;
    };

    if obj.name == "React" && matches!(member.property.name.as_str(), "memo" | "forwardRef") {
        return Some(format!("React.{}", member.property.name));
    }

    None
}

/// Check if a function just passes through to a single JSX component
fn is_passthrough_function(func: &oxc_ast::ast::Function) -> bool {
    let Some(body) = &func.body else {
        return false;
    };

    let [Statement::ReturnStatement(ret)] = body.statements.as_slice() else {
        return false;
    };

    ret.argument.as_ref().is_some_and(is_simple_jsx_passthrough)
}

/// Check if an arrow function just passes through to a single JSX component
fn is_passthrough_arrow(arrow: &oxc_ast::ast::ArrowFunctionExpression) -> bool {
    // Expression arrow: `() => <Comp {...props} />`
    if let Some(expr) = arrow.get_expression() {
        return is_simple_jsx_passthrough(expr);
    }

    // Block body with single return: `() => { return <Comp {...props} />; }`
    let [Statement::ReturnStatement(ret)] = arrow.body.statements.as_slice() else {
        return false;
    };

    ret.argument.as_ref().is_some_and(is_simple_jsx_passthrough)
}

/// Check if an expression is a simple JSX element that just renders another component
/// This is for cases like: React.forwardRef((props, ref) => <MyComp {...props} ref={ref} />)
fn is_simple_jsx_passthrough(expr: &Expression) -> bool {
    let Expression::JSXElement(jsx) = expr else {
        return false;
    };

    // Check if it's rendering another component (PascalCase name)
    let oxc_ast::ast::JSXElementName::IdentifierReference(id) = &jsx.opening_element.name else {
        return false;
    };

    if !is_react_component_name(&id.name) {
        return false;
    }

    // Only consider it a passthrough if it spreads props (like {...props})
    // A component that doesn't pass props isn't a simple wrapper
    let has_spread = jsx
        .opening_element
        .attributes
        .iter()
        .any(|attr| matches!(attr, oxc_ast::ast::JSXAttributeItem::SpreadAttribute(_)));

    has_spread && jsx.opening_element.attributes.len() <= 2
}

/// Check if a function returns another function with JSX (component factory)
fn returns_component(func: &Function) -> bool {
    let Some(body) = &func.body else {
        return false;
    };

    body.statements
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::ReturnStatement(ret) => ret.argument.as_ref(),
            _ => None,
        })
        .any(expression_contains_jsx)
}

/// Check if a class is an ES6 React component (extends React.Component or React.PureComponent)
fn is_es6_component_class(class: &Class) -> bool {
    let Some(super_class) = &class.super_class else {
        return false;
    };

    if let Some(member_expr) = super_class.as_member_expression()
        && let Expression::Identifier(ident) = member_expr.object()
    {
        return ident.name == "React"
            && member_expr
                .static_property_name()
                .is_some_and(|name| name == "Component" || name == "PureComponent");
    }

    if let Some(ident_reference) = super_class.get_identifier_reference() {
        return ident_reference.name == "Component" || ident_reference.name == "PureComponent";
    }

    false
}

/// Check if a call expression is createReactClass
fn is_es5_component_call(call: &CallExpression) -> bool {
    if let Some(member_expr) = call.callee.as_member_expression()
        && let Expression::Identifier(ident) = member_expr.object()
    {
        return ident.name == "React"
            && member_expr.static_property_name() == Some("createReactClass");
    }

    if let Some(ident_reference) = call.callee.get_identifier_reference() {
        return ident_reference.name == "createReactClass";
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
