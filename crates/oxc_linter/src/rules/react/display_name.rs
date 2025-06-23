use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, Class, ClassElement, Expression, JSXChild, StaticMemberExpression,
        VariableDeclaration,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::PropName;
use oxc_span::GetSpan;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};
use oxc_macros::declare_oxc_lint;

fn display_name_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("React component is missing a displayName property")
        .with_help("Add a displayName property to the component")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DisplayNameConfig {
    ignore_transpiler_name: bool,
    check_context_objects: bool,
}

#[derive(Debug, Default, Clone)]
pub struct DisplayName(Box<DisplayNameConfig>);

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule allows you to name your component. This name is used by React in debugging messages.
    ///
    /// ### Why is this bad?
    ///
    /// When debugging React components, there will be missing identifiers for the components that lack a displayName property.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   render: function() {
    ///     return <div>Hello {this.props.name}</div>;
    ///   }
    /// });
    ///
    /// const Hello = React.memo(({ a }) => {
    ///   return <>{a}</>
    /// })
    ///
    /// export default ({ a }) => {
    ///   return <>{a}</>
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   displayName: 'Hello',
    ///   render: function() {
    ///     return <div>Hello {this.props.name}</div>;
    ///   }
    /// });
    ///
    /// const Hello = React.memo(function Hello({ a }) {
    ///   return <>{a}</>
    /// })
    /// ```
    ///
    DisplayName,
    react,
    style,
);

impl Rule for DisplayName {
    fn from_configuration(value: serde_json::Value) -> Self {
        if value.is_array() {
            let value = value.get(0);

            if let Some(value) = value {
                let ignore_transpiler_name =
                    value.get("ignoreTranspilerName").and_then(Value::as_bool).unwrap_or_default();
                let check_context_objects =
                    value.get("checkContextObjects").and_then(Value::as_bool).unwrap_or_default();

                return Self(Box::new(DisplayNameConfig {
                    ignore_transpiler_name,
                    check_context_objects,
                }));
            }
        }

        let ignore_transpiler_name =
            value.get("ignoreTranspilerName").and_then(Value::as_bool).unwrap_or_default();
        let check_context_objects =
            value.get("checkContextObjects").and_then(Value::as_bool).unwrap_or_default();

        Self(Box::new(DisplayNameConfig { ignore_transpiler_name, check_context_objects }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let config = &self.0;

        println!("\nsource_code: {:?} \n", ctx.source_text());

        let ignore_transpiler_name = config.ignore_transpiler_name;
        let _check_context_objects = config.check_context_objects;

        let mut class_names_with_display_names_modified: Vec<Span> = vec![];
        let mut class_names_initialized_with_no_display_name_property: Vec<Span> = vec![];

        let mut class_ids: Vec<(String, Span)> = vec![];

        for node in ctx.nodes() {
            match node.kind() {
                AstKind::VariableDeclaration(decl) => {
                    let result = process_variable_declaration_node(decl, ignore_transpiler_name);

                    let (
                        class_names_initialized_with_no_display_name_property_result,
                        class_names_with_display_names_modified_result,
                    ) = result;

                    class_names_initialized_with_no_display_name_property
                        .extend(class_names_initialized_with_no_display_name_property_result);
                    class_names_with_display_names_modified
                        .extend(class_names_with_display_names_modified_result);
                }
                AstKind::Class(class) => {
                    let has_display_name = class.body.body.iter().any(|element| {
                        if let ClassElement::PropertyDefinition(prop_def) = element {
                            prop_def.key.static_name()
                                == Some(std::borrow::Cow::Borrowed("displayName"))
                        } else {
                            false
                        }
                    });

                    if !has_display_name {
                        class_names_initialized_with_no_display_name_property.push(class.span);
                        // } else {
                        //     class_names_with_display_names_modified.push(class.span);
                    }

                    for element in &class.body.body {
                        //     println!("element: {element:?}");

                        match element {
                            ClassElement::PropertyDefinition(prop_def) => {
                                println!("prop_def: {prop_def:?}");
                                if prop_def.key.static_name()
                                    == Some(std::borrow::Cow::Borrowed("displayName"))
                                {
                                    class_names_with_display_names_modified.push(class.span);
                                }
                            }
                            ClassElement::MethodDefinition(method_def) => {
                                println!("method_def: {method_def:?}");
                                if method_def.key.static_name()
                                    == Some(std::borrow::Cow::Borrowed("displayName"))
                                {
                                    class_names_with_display_names_modified.push(class.span);
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Some(id) = &class.id {
                        class_ids.push((id.name.to_string(), class.span));
                    }

                    let result = process_class_node(class, ignore_transpiler_name);

                    let (
                        class_names_initialized_with_no_display_name_property_result,
                        class_names_with_display_names_modified_result,
                    ) = result;

                    class_names_initialized_with_no_display_name_property
                        .extend(class_names_initialized_with_no_display_name_property_result);
                    class_names_with_display_names_modified
                        .extend(class_names_with_display_names_modified_result);
                }
                AstKind::ExpressionStatement(expr_stmt) => {
                    if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                        if let AssignmentTarget::StaticMemberExpression(id) = &assign.left {
                            if let Expression::Identifier(identifier) = &id.object {
                                let result = class_ids
                                    .iter()
                                    .filter(|(name, _)| name == &identifier.name.to_string())
                                    .map(|(_, span)| span)
                                    .collect::<Vec<_>>();

                                println!("result: {result:?}");
                                println!("identifier: {identifier:?}");
                                println!("class_ids: {class_ids:?}");

                                if !result.is_empty() {
                                    class_names_initialized_with_no_display_name_property =
                                        class_names_initialized_with_no_display_name_property
                                            .iter()
                                            .filter_map(|span| {
                                                if result.iter().any(|r| {
                                                    is_span_equal_or_inside_other_span(r, span)
                                                        || is_span_equal_or_inside_other_span(
                                                            span, r,
                                                        )
                                                }) {
                                                    None
                                                } else {
                                                    Some(*span)
                                                }
                                            })
                                            .collect();
                                }
                            }

                            if &id.property.name == "displayName" {
                                class_names_with_display_names_modified.push(expr_stmt.span);
                            }
                        }

                        if let AssignmentTarget::AssignmentTargetIdentifier(identifier) =
                            &assign.left
                        {
                            println!("identifier: {identifier:?}");
                            println!("class_ids: {class_ids:?}");
                            println!("ignore_transpiler_name: {ignore_transpiler_name}");

                            if !ignore_transpiler_name
                                && class_ids
                                    .iter()
                                    .any(|(name, _)| name == &identifier.name.to_string())
                            {
                                class_names_with_display_names_modified.push(expr_stmt.span);
                            }
                        }

                        if let Expression::CallExpression(call) = &assign.right {
                            if let Some(name) = call.callee_name() {
                                if (name == "createClass" || name == "createReactClass")
                                    && !ignore_transpiler_name
                                {
                                    class_names_with_display_names_modified.push(expr_stmt.span);
                                }
                            }
                        }
                    }
                }
                AstKind::Function(func_decl) => {
                    println!("func_decl: {func_decl:?}");

                    if let Some(_id) = &func_decl.id {
                        if !ignore_transpiler_name {
                            class_names_with_display_names_modified.push(func_decl.span);
                        } else {
                            class_names_initialized_with_no_display_name_property
                                .push(func_decl.span);
                        }
                    } else {
                        class_names_initialized_with_no_display_name_property.push(func_decl.span);
                    }
                }
                _ => {}
            }
        }

        // Check for name prop usage in render methods
        let name_prop_usage = detect_name_prop_in_render(ctx, ignore_transpiler_name);

        println!("\nname_prop_usage: {name_prop_usage:?}\n");
        class_names_with_display_names_modified.extend(name_prop_usage);

        let result = get_result(
            &class_names_initialized_with_no_display_name_property,
            &class_names_with_display_names_modified,
        );

        if let Some(class) = result.first() {
            ctx.diagnostic(display_name_diagnostic(*class));
        }
    }
}

/// Detects if a render method contains `this.props.name` in JSX expressions
fn detect_name_prop_in_render(ctx: &LintContext, ignore_transpiler_name: bool) -> Vec<Span> {
    if ignore_transpiler_name {
        return vec![];
    }

    let mut name_prop_usage: Vec<Span> = vec![];

    for node in ctx.nodes() {
        if let AstKind::Class(class) = node.kind() {
            for element in &class.body.body {
                if let oxc_ast::ast::ClassElement::MethodDefinition(method_def) = element {
                    if method_def.key.static_name() == Some(std::borrow::Cow::Borrowed("render")) {
                        if let Some(body) = &method_def.value.body {
                            for stmt in &body.statements {
                                if let oxc_ast::ast::Statement::ReturnStatement(ret_stmt) = stmt {
                                    if let Some(expr) = &ret_stmt.argument {
                                        if check_expression_for_name_prop(expr) {
                                            name_prop_usage.push(expr.span());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    name_prop_usage
}

/// Recursively checks if an expression contains `this.props.name`
fn check_expression_for_name_prop(expr: &Expression) -> bool {
    match expr {
        Expression::JSXElement(jsx_elem) => {
            // Check JSX children for name prop usage
            for child in &jsx_elem.children {
                if check_jsx_child_for_name_prop(child) {
                    return true;
                }
            }
            false
        }
        Expression::JSXFragment(jsx_frag) => {
            // Check JSX fragment children for name prop usage
            for child in &jsx_frag.children {
                if check_jsx_child_for_name_prop(child) {
                    return true;
                }
            }
            false
        }
        Expression::StaticMemberExpression(member_expr) => {
            // Check if this is `this.props.name`
            check_static_member_for_name_prop(member_expr)
        }
        _ => false,
    }
}

/// Checks if a JSX child contains `this.props.name`
fn check_jsx_child_for_name_prop(child: &JSXChild) -> bool {
    match child {
        JSXChild::ExpressionContainer(container) => {
            match &container.expression {
                jsx_expr => {
                    // Use as_expression() to convert JSXExpression to Expression
                    if let Some(expr) = jsx_expr.as_expression() {
                        check_expression_for_name_prop(expr)
                    } else {
                        false
                    }
                }
            }
        }
        JSXChild::Element(jsx_elem) => {
            // Recursively check JSX element children
            for child in &jsx_elem.children {
                if check_jsx_child_for_name_prop(child) {
                    return true;
                }
            }
            false
        }
        JSXChild::Fragment(jsx_frag) => {
            // Recursively check JSX fragment children
            for child in &jsx_frag.children {
                if check_jsx_child_for_name_prop(child) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Checks if a static member expression is `this.props.name`
fn check_static_member_for_name_prop(member_expr: &StaticMemberExpression) -> bool {
    // Check if the property is "name"
    if member_expr.property.name != "name" {
        return false;
    }

    // Check if the object is `this.props`
    match &member_expr.object {
        Expression::StaticMemberExpression(props_member) => {
            // Check if this is `this.props`
            if props_member.property.name == "props" {
                match &props_member.object {
                    Expression::ThisExpression(_) => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn process_variable_declaration_node(
    decl: &VariableDeclaration,
    ignore_transpiler_name: bool,
) -> (Vec<Span>, Vec<Span>) {
    let mut class_names_initialized_with_no_display_name_property: Vec<Span> = vec![];
    let mut class_names_with_display_names_modified: Vec<Span> = vec![];

    for decl in &decl.declarations {
        if let Some(init) = &decl.init {
            // Check for createReactClass
            if let Expression::CallExpression(call) = init {
                if let Some(name) = call.callee_name() {
                    if name == "createClass" || name == "createReactClass" {
                        let contains_display_name = call.arguments.iter().any(|arg| {
                            if let Some(Expression::ObjectExpression(obj_expr)) =
                                arg.as_expression()
                            {
                                for prop in &obj_expr.properties {
                                    if let Some((name, _)) = prop.prop_name() {
                                        return name == "displayName";
                                    }
                                }
                            }
                            false
                        });

                        if contains_display_name || !ignore_transpiler_name {
                            class_names_with_display_names_modified.push(decl.span());
                        } else {
                            class_names_initialized_with_no_display_name_property.push(decl.span());
                        }
                    }
                } else if ignore_transpiler_name {
                    class_names_initialized_with_no_display_name_property.push(decl.span());
                }
            }

            if let Expression::FunctionExpression(func_expr) = init {
                println!("func_expr: {func_expr:?}");
                // if let Some(id) = &func_expr.id {
                if !ignore_transpiler_name {
                    class_names_with_display_names_modified.push(decl.span);
                    // } else {
                    //     class_names_initialized_with_no_display_name_property.push(id.span);
                }
                // }
            }
        }
    }

    (class_names_initialized_with_no_display_name_property, class_names_with_display_names_modified)
}

fn process_class_node(class: &Class, ignore_transpiler_name: bool) -> (Vec<Span>, Vec<Span>) {
    let mut class_names_with_display_names_modified: Vec<Span> = vec![];
    let mut class_names_initialized_with_no_display_name_property: Vec<Span> = vec![];

    println!("class: {class:?}");

    if let Some(name) = &class.name() {
        if !name.is_empty() && !ignore_transpiler_name {
            class_names_with_display_names_modified.push(class.span);
            // If the class name has a valid identifier, that is considered a displayName
        } else if ignore_transpiler_name {
            class_names_initialized_with_no_display_name_property.push(class.span);
        }
    } else {
        class_names_initialized_with_no_display_name_property.push(class.span);
    }

    (class_names_initialized_with_no_display_name_property, class_names_with_display_names_modified)
}

fn is_span_equal_or_inside_other_span(span: &Span, other_span: &Span) -> bool {
    let are_spans_equal = span.start == other_span.start && span.end == other_span.end;

    let is_modified_span_inside_original_class_span =
        span.start >= other_span.start && span.end <= other_span.end;

    are_spans_equal || is_modified_span_inside_original_class_span
}

fn get_result(
    class_names_initialized_with_no_display_name_property: &Vec<Span>,
    class_names_with_display_names_modified: &Vec<Span>,
) -> Vec<Span> {
    let result: Vec<Span> = class_names_initialized_with_no_display_name_property
        .iter()
        .filter_map(|outer_span| {
            // only consider the class spans that have not had their display names explicitly set later.
            if class_names_with_display_names_modified.is_empty()
                || !class_names_with_display_names_modified.iter().any(|same_or_inner_span| {
                    is_span_equal_or_inside_other_span(same_or_inner_span, outer_span)
                        || is_span_equal_or_inside_other_span(outer_span, same_or_inner_span)
                })
            {
                Some(*outer_span)
            } else {
                None
            }
        })
        .collect();

    println!(
        "class_names_initialized_with_no_display_name_property: {class_names_initialized_with_no_display_name_property:?}"
    );
    println!(
        "class_names_with_display_names_modified: {class_names_with_display_names_modified:?}"
    );
    println!("result: {result:?}");

    result
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = React.createClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
        	        class Hello extends React.Component {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	        Hello.displayName = 'Hello'
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        class Hello {
        	          render() {
        	            return 'Hello World';
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello extends Greetings {
        	          static text = 'Hello World';
        	          render() {
        	            return Hello.text;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello {
        	          method;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello extends React.Component {
        	          static get displayName() {
        	            return 'Hello';
        	          }
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        class Hello extends React.Component {
        	          static displayName = 'Widget';
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
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
            None,
            None,
        ),
        (
            "
        	        class Hello extends React.Component {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        export default class Hello {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello;
        	        Hello = createReactClass({
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        module.exports = createReactClass({
        	          "displayName": "Hello",
        	          "render": function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      "#,
            None,
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            let { a, ...b } = obj;
        	            let c = { ...d };
        	            return <div />;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        export default class {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        export const Hello = React.memo(function Hello() {
        	          return <p />;
        	        })
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello = function() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        function Hello() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello = () => {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        module.exports = function Hello() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
			        function Hello() {
			          return <div>Hello {this.props.name}</div>;
			        }
			        Hello.displayName = 'Hello';
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = () => {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	        Hello.displayName = 'Hello';
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = function() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	        Hello.displayName = 'Hello';
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Mixins = {
        	          Greetings: {
        	            Hello: function() {
        	              return <div>Hello {this.props.name}</div>;
        	            }
        	          }
        	        }
        	        Mixins.Greetings.Hello.displayName = 'Hello';
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          render: function() {
        	            return <div>{this._renderHello()}</div>;
        	          },
        	          _renderHello: function() {
        	            return <span>Hello {this.props.name}</span>;
        	          }
        	        });
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            return <div>{this._renderHello()}</div>;
        	          },
        	          _renderHello: function() {
        	            return <span>Hello {this.props.name}</span>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        const Mixin = {
        	          Button() {
        	            return (
        	              <button />
        	            );
        	          }
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        var obj = {
        	          pouf: function() {
        	            return any
        	          }
        	        };
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var obj = {
        	          pouf: function() {
        	            return any
        	          }
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        export default {
        	          renderHello() {
        	            let {name} = this.props;
        	            return <div>{name}</div>;
        	          }
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React, { createClass } from 'react';
        	        export default createClass({
        	          displayName: 'Foo',
        	          render() {
        	            return <h1>foo</h1>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            r#"
        	        import React, {Component} from "react";
        	        function someDecorator(ComposedComponent) {
        	          return class MyDecorator extends Component {
        	            render() {return <ComposedComponent {...this.props} />;}
        	          };
        	        }
        	        module.exports = someDecorator;
        	      "#,
            None,
            None,
        ),
        (
            r#"
        	        import React, {createElement} from "react";
        	        const SomeComponent = (props) => {
        	          const {foo, bar} = props;
        	          return someComponentFactory({
        	            onClick: () => foo(bar("x"))
        	          });
        	        };
        	      "#,
            None,
            None,
        ),
        (
            "
        	        const element = (
        	          <Media query={query} render={() => {
        	            renderWasCalled = true
        	            return <div/>
        	          }}/>
        	        )
        	      ",
            None,
            None,
        ),
        (
            "
        	        const element = (
        	          <Media query={query} render={function() {
        	            renderWasCalled = true
        	            return <div/>
        	          }}/>
        	        )
        	      ",
            None,
            None,
        ),
        (
            "
        	        module.exports = {
        	          createElement: tagName => document.createElement(tagName)
        	        };
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        const { createElement } = document;
        	        createElement("a");
        	      "#,
            None,
            None,
        ),
        (
            "
        	        import React from 'react'
        	        import { string } from 'prop-types'

        	        function Component({ world }) {
        	          return <div>Hello {world}</div>
        	        }

        	        Component.propTypes = {
        	          world: string,
        	        }

        	        export default React.memo(Component)
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const ComponentWithMemo = React.memo(function Component({ world }) {
        	          return <div>Hello {world}</div>
        	        })
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react';

        	        const Hello = React.memo(function Hello() {
        	          return;
        	        });
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const ForwardRefComponentLike = React.forwardRef(function ComponentLike({ world }, ref) {
        	          return <div ref={ref}>Hello {world}</div>
        	        })
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        function F() {
        	          let items = [];
        	          let testData = [
        	            {a: "test1", displayName: "test2"}, {a: "test1", displayName: "test2"}];
        	          for (let item of testData) {
        	              items.push({a: item.a, b: item.displayName});
        	          }
        	          return <div>{items}</div>;
        	        }
        	      "#,
            None,
            None,
        ),
        // NOTE: this test throws an unexpected token error.
        // (
        //     r#"
        // 	        import {Component} from "react";
        // 	        type LinkProps = {...{}};
        // 	        class Link extends Component<LinkProps> {}
        // 	      "#,
        //     None,
        //     None,
        // ),
        (
            r#"
        	        const x = {
        	          title: "URL",
        	          dataIndex: "url",
        	          key: "url",
        	          render: url => (
        	            <a href={url} target="_blank" rel="noopener noreferrer">
        	              <p>lol</p>
        	            </a>
        	          )
        	        }
        	      "#,
            None,
            None,
        ),
        (
            "
        	        const renderer = a => function Component(listItem) {
        	          return <div>{a} {listItem}</div>;
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        const Comp = React.forwardRef((props, ref) => <main />);
        	        Comp.displayName = 'MyCompName';
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        const Comp = React.forwardRef((props, ref) => <main data-as="yes" />) as SomeComponent;
        	        Comp.displayName = 'MyCompNameAs';
        	      "#,
            None,
            None,
        ),
        (
            "
        	        function Test() {
        	          const data = [
        	            {
        	              name: 'Bob',
        	            },
        	          ];

        	          const columns = [
        	            {
        	              Header: 'Name',
        	              accessor: 'name',
        	              Cell: ({ value }) => <div>{value}</div>,
        	            },
        	          ];

        	          return <ReactTable columns={columns} data={data} />;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        const f = (a) => () => {
        	          if (a) {
        	            return null;
        	          }
        	          return 1;
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Test {
        	          render() {
        	            const data = [
        	              {
        	                name: 'Bob',
        	              },
        	            ];

        	            const columns = [
        	              {
        	                Header: 'Name',
        	                accessor: 'name',
        	                Cell: ({ value }) => <div>{value}</div>,
        	              },
        	            ];

        	            return <ReactTable columns={columns} data={data} />;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        export const demo = (a) => (b) => {
        	          if (a == null) return null;
        	          return b;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        let demo = null;
        	        demo = (a) => {
        	          if (a == null) return null;
        	          return f(a);
        	        };",
            None,
            None,
        ),
        (
            "
        	        obj._property = (a) => {
        	          if (a == null) return null;
        	          return f(a);
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        _variable = (a) => {
        	          if (a == null) return null;
        	          return f(a);
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = () => () => null;
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = {
        	          property: () => () => null
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = function() {return function() {return null;};};
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = {
        	          property: function() {return function() {return null;};}
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        function MyComponent(props) {
        	          return <b>{props.name}</b>;
        	        }

        	        const MemoizedMyComponent = React.memo(
        	          MyComponent,
        	          (prevProps, nextProps) => prevProps.name === nextProps.name
        	        )
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(function({ world }, ref) {
        	            return <div ref={ref}>Hello {world}</div>
        	        })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.14.0",        },      } }),
            ),
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(({ world }, ref) => {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.7.0",        },      } }),
            ),
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(function ComponentLike({ world }, ref) {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.12.1",        },      } }),
            ),
        ),
        (
            "
        	        export const ComponentWithForwardRef = React.memo(
        	          React.forwardRef(function Component({ world }) {
        	            return <div>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "0.14.11",        },      } }),
            ),
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(function({ world }, ref) {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.7.1",        },      } }),
            ),
        ),
        (
            r#"
        	        import React from 'react';

        	        const Hello = React.createContext();
        	        Hello.displayName = "HelloContext"
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	        Hello.displayName = "HelloContext"
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        const Hello = createContext();

        	        const obj = {};
        	        obj.displayName = "False positive";

        	        Hello.displayName = "HelloContext"
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import * as React from 'react';

        	        const Hello = React.createContext();

        	        const obj = {};
        	        obj.displayName = "False positive";

        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        const obj = {};
        	        obj.displayName = "False positive";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.2.0",        },      } }),
            ),
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": ">16.3.0",        },      } }),
            ),
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        let Hello;
        	        Hello = createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	      ",
            Some(serde_json::json!([{ "checkContextObjects": false }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": ">16.3.0",        },      } }),
            ),
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        var Hello;
        	        Hello = createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        var Hello;
        	        Hello = React.createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
    ];

    // let pass: Vec<(&'static str, Option<Value>, Option<Value>)> = vec![];

    let fail: Vec<_> = vec![
        (
            r#"
        	        var Hello = createReactClass({
        	          render: function() {
        	            return React.createElement("div", {}, "text content");
        	          }
        	        });
        	      "#,
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            r#"
        	        var Hello = React.createClass({
        	          render: function() {
        	            return React.createElement("div", {}, "text content");
        	          }
        	        });
        	      "#,
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        function HelloComponent() {
			          return createReactClass({
			            render: function() {
			              return <div>Hello {this.props.name}</div>;
			            }
			          });
			        }
			        module.exports = HelloComponent();
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        module.exports = () => {
			          return <div>Hello {props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = function() {
			          return <div>Hello {props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = createReactClass({
			          render() {
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
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = Foo.createClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "pragma": "Foo",          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        /** @jsx Foo */
			        var Hello = Foo.createClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        const Mixin = {
			          Button() {
			            return (
			              <button />
			            );
			          }
			        };
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        function Hof() {
			          return function () {
			            return <div />
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            r#"
			        import React, { createElement } from "react";
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(({ world }) => {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(function() {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(({ world }, ref) => {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(function({ world }, ref) {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(({ world }, ref) => {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.6.0",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "0.14.2",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function ComponentLike({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.0.1",        },      } }),
            ),
        ),
        (
            r#"
			        import React from "react";
			        const { createElement } = React;
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            r#"
			        import React from "react";
			        const createElement = React.createElement;
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            r#"
			        module.exports = function () {
			          function a () {}
			          const b = function b () {}
			          const c = function () {}
			          const d = () => {}
			          const obj = {
			            a: function a () {},
			            b: function b () {},
			            c () {},
			            d: () => {},
			          }
			          return React.createElement("div", {}, "text content");
			        }
			      "#,
            None,
            None,
        ),
        (
            r#"
			        module.exports = () => {
			          function a () {}
			          const b = function b () {}
			          const c = function () {}
			          const d = () => {}
			          const obj = {
			            a: function a () {},
			            b: function b () {},
			            c () {},
			            d: () => {},
			          }

			          return React.createElement("div", {}, "text content");
			        }
			      "#,
            None,
            None,
        ),
        (
            "
			        export default class extends React.Component {
			          render() {
			            function a () {}
			            const b = function b () {}
			            const c = function () {}
			            const d = () => {}
			            const obj = {
			              a: function a () {},
			              b: function b () {},
			              c () {},
			              d: () => {},
			            }
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export default class extends React.PureComponent {
			          render() {
			            return <Card />;
			          }
			        }

			        const Card = (() => {
			          return React.memo(({ }) => (
			            <div />
			          ));
			        })();
			      ",
            None,
            None,
        ),
        (
            "
			        const renderer = a => listItem => (
			          <div>{a} {listItem}</div>
			        );
			      ",
            None,
            None,
        ),
        (
            "
			        const processData = (options?: { value: string }) => options?.value || 'no data';

			        export const Component = observer(() => {
			          const data = processData({ value: 'data' });
			          return <div>{data}</div>;
			        });

			        export const Component2 = observer(() => {
			          const data = processData();
			          return <div>{data}</div>;
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "componentWrapperFunctions": ["observer"] } })),
        ),
        (
            "
			        import React from 'react';

			        const Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import * as React from 'react';

			        const Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        const Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        var Hello;
			        Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        var Hello;
			        Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
    ];

    // let fail: Vec<(&'static str, Option<Value>, Option<Value>)> = vec![        (
    //     r#"
    // 	        var Hello = createReactClass({
    // 	          render: function() {
    // 	            return React.createElement("div", {}, "text content");
    // 	          }
    // 	        });
    // 	      "#,
    //     Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
    //     None,
    // ),];

    Tester::new(DisplayName::NAME, DisplayName::PLUGIN, pass, fail).test_and_snapshot();
}
