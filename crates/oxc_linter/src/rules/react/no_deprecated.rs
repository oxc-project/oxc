use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::get_parent_component};

fn no_deprecated_diagnostic(span: Span, deprecated: &str, replacement: &str) -> OxcDiagnostic {
    let message = if replacement.is_empty() {
        format!("`{deprecated}` is deprecated.")
    } else {
        format!("`{deprecated}` is deprecated. Use `{replacement}` instead.")
    };
    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecated;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows usage of deprecated React APIs.
    ///
    /// ### Why is this bad?
    ///
    /// Deprecated APIs may be removed in future React versions, causing your code to break.
    /// It's better to migrate to the recommended alternatives early.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// React.render(<App />, root);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// ReactDOM.render(<App />, root);
    /// ```
    NoDeprecated,
    react,
    correctness
);

/// Deprecated React module methods with their replacements and the version they were deprecated in
/// Format: (method_name, replacement, deprecated_since_major, deprecated_since_minor, deprecated_since_patch)
const DEPRECATED_REACT_METHODS: &[(&str, &str, u32, u32, u32)] = &[
    ("renderComponent", "React.render", 0, 12, 0),
    ("render", "ReactDOM.render", 0, 14, 0),
    ("unmountComponentAtNode", "ReactDOM.unmountComponentAtNode", 0, 14, 0),
    ("findDOMNode", "ReactDOM.findDOMNode", 0, 14, 0),
    ("renderToString", "ReactDOMServer.renderToString", 0, 14, 0),
    ("renderToStaticMarkup", "ReactDOMServer.renderToStaticMarkup", 0, 14, 0),
    ("createClass", "create-react-class package", 15, 5, 0),
];

/// Deprecated ReactDOM methods (React 18)
const DEPRECATED_REACT_DOM_METHODS: &[(&str, &str, u32, u32, u32)] = &[
    ("render", "createRoot", 18, 0, 0),
    ("hydrate", "hydrateRoot", 18, 0, 0),
    ("unmountComponentAtNode", "root.unmount", 18, 0, 0),
];

/// Deprecated ReactDOMServer methods (React 18)
const DEPRECATED_REACT_DOM_SERVER_METHODS: &[(&str, &str, u32, u32, u32)] =
    &[("renderToNodeStream", "renderToPipeableStream", 18, 0, 0)];

/// Deprecated React properties
const DEPRECATED_REACT_PROPERTIES: &[(&str, &str, u32, u32, u32)] = &[
    ("PropTypes", "prop-types package", 15, 5, 0),
    ("DOM", "react-dom-factories package", 15, 6, 0),
];

/// Deprecated lifecycle methods (React 16.9+)
const DEPRECATED_LIFECYCLE_METHODS: &[(&str, &str, u32, u32, u32)] = &[
    ("componentWillMount", "UNSAFE_componentWillMount", 16, 9, 0),
    ("componentWillReceiveProps", "UNSAFE_componentWillReceiveProps", 16, 9, 0),
    ("componentWillUpdate", "UNSAFE_componentWillUpdate", 16, 9, 0),
];

/// Deprecated addons
const DEPRECATED_REACT_ADDONS: &[(&str, &str)] =
    &[("TestUtils", "react-dom/test-utils package"), ("classSet", "classnames package")];

/// Deprecated react-addons-perf exports
const DEPRECATED_PERF_EXPORTS: &[&str] = &["printDOM"];

/// Source type for variable declarator checking
enum VariableDeclaratorSource {
    React,
    ReactAddonsPerf,
    Other,
}

impl Rule for NoDeprecated {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                Self::check_call_expression(call_expr, ctx);
            }
            AstKind::StaticMemberExpression(member_expr) => {
                Self::check_static_member_expression(member_expr, ctx);
            }
            AstKind::MethodDefinition(method_def) => {
                Self::check_method_definition(method_def, node, ctx);
            }
            AstKind::ObjectProperty(obj_prop) => {
                Self::check_object_property(obj_prop, node, ctx);
            }
            AstKind::ImportSpecifier(import_specifier) => {
                Self::check_import_specifier(import_specifier, node, ctx);
            }
            AstKind::VariableDeclarator(var_decl) => {
                Self::check_variable_declarator(var_decl, ctx);
            }
            _ => {}
        }
    }
}

impl NoDeprecated {
    fn check_call_expression(call_expr: &oxc_ast::ast::CallExpression, ctx: &LintContext) {
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        let Some((prop_span, prop_name)) = member_expr.static_property_info() else {
            return;
        };

        let react_settings = &ctx.settings().react;
        let pragma = react_settings.get_pragma();

        // Check for this.transferPropsTo()
        if prop_name == "transferPropsTo"
            && matches!(member_expr.object(), Expression::ThisExpression(_))
        {
            ctx.diagnostic(no_deprecated_diagnostic(
                call_expr.callee.span(),
                "this.transferPropsTo",
                "spread operator ({...})",
            ));
            return;
        }

        let object = member_expr.object();

        // Handle React.addons.* calls (using pragma)
        if let Some(inner_member) = object.get_member_expr()
            && let Some((_, inner_prop)) = inner_member.static_property_info()
            && inner_prop == "addons"
            && inner_member.object().is_specific_id(pragma)
        {
            if let Some((_, replacement)) =
                DEPRECATED_REACT_ADDONS.iter().find(|(name, _)| *name == prop_name)
            {
                ctx.diagnostic(no_deprecated_diagnostic(
                    call_expr.callee.span(),
                    &format!("{pragma}.addons.{prop_name}"),
                    replacement,
                ));
            }
            return;
        }

        // Check React.* (pragma.*) deprecated methods
        if object.is_specific_id(pragma) {
            if let Some((_, replacement, major, minor, patch)) =
                DEPRECATED_REACT_METHODS.iter().find(|(name, _, _, _, _)| *name == prop_name)
                && react_settings.version_at_least(*major, *minor, *patch)
            {
                ctx.diagnostic(no_deprecated_diagnostic(
                    prop_span,
                    &format!("{pragma}.{prop_name}"),
                    replacement,
                ));
            }
            return;
        }

        // Check ReactDOM.* deprecated methods
        if object.is_specific_id("ReactDOM") {
            if let Some((_, replacement, major, minor, patch)) =
                DEPRECATED_REACT_DOM_METHODS.iter().find(|(name, _, _, _, _)| *name == prop_name)
                && react_settings.version_at_least(*major, *minor, *patch)
            {
                ctx.diagnostic(no_deprecated_diagnostic(
                    prop_span,
                    &format!("ReactDOM.{prop_name}"),
                    replacement,
                ));
            }
            return;
        }

        // Check ReactDOMServer.* deprecated methods
        if object.is_specific_id("ReactDOMServer")
            && let Some((_, replacement, major, minor, patch)) = DEPRECATED_REACT_DOM_SERVER_METHODS
                .iter()
                .find(|(name, _, _, _, _)| *name == prop_name)
            && react_settings.version_at_least(*major, *minor, *patch)
        {
            ctx.diagnostic(no_deprecated_diagnostic(
                prop_span,
                &format!("ReactDOMServer.{prop_name}"),
                replacement,
            ));
        }
    }

    fn check_static_member_expression(
        member_expr: &oxc_ast::ast::StaticMemberExpression,
        ctx: &LintContext,
    ) {
        let prop_name = member_expr.property.name.as_str();
        let prop_span = member_expr.property.span;
        let object = &member_expr.object;

        let react_settings = &ctx.settings().react;
        let pragma = react_settings.get_pragma();

        // Handle React.addons.* access (not as call) - using pragma
        if let Some(inner_member) = object.get_member_expr()
            && let Some((_, inner_prop)) = inner_member.static_property_info()
            && inner_prop == "addons"
            && inner_member.object().is_specific_id(pragma)
        {
            if let Some((_, replacement)) =
                DEPRECATED_REACT_ADDONS.iter().find(|(name, _)| *name == prop_name)
            {
                ctx.diagnostic(no_deprecated_diagnostic(
                    member_expr.span,
                    &format!("{pragma}.addons.{prop_name}"),
                    replacement,
                ));
            }
            return;
        }

        // Check React.PropTypes, React.DOM access - using pragma
        if object.is_specific_id(pragma)
            && let Some((_, replacement, major, minor, patch)) =
                DEPRECATED_REACT_PROPERTIES.iter().find(|(name, _, _, _, _)| *name == prop_name)
            && react_settings.version_at_least(*major, *minor, *patch)
        {
            ctx.diagnostic(no_deprecated_diagnostic(
                prop_span,
                &format!("{pragma}.{prop_name}"),
                replacement,
            ));
        }
    }

    fn check_method_definition<'a>(
        method_def: &oxc_ast::ast::MethodDefinition,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(prop_name) = method_def.key.static_name() else {
            return;
        };

        let react_settings = &ctx.settings().react;

        if let Some((deprecated, replacement, major, minor, patch)) =
            DEPRECATED_LIFECYCLE_METHODS.iter().find(|(name, _, _, _, _)| *name == prop_name)
            && react_settings.version_at_least(*major, *minor, *patch)
            && get_parent_component(node, ctx).is_some()
        {
            ctx.diagnostic(no_deprecated_diagnostic(
                method_def.key.span(),
                deprecated,
                replacement,
            ));
        }
    }

    fn check_object_property<'a>(
        obj_prop: &oxc_ast::ast::ObjectProperty,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(prop_name) = obj_prop.key.static_name() else {
            return;
        };

        let react_settings = &ctx.settings().react;

        if let Some((deprecated, replacement, major, minor, patch)) =
            DEPRECATED_LIFECYCLE_METHODS.iter().find(|(name, _, _, _, _)| *name == prop_name)
            && react_settings.version_at_least(*major, *minor, *patch)
            && get_parent_component(node, ctx).is_some()
        {
            ctx.diagnostic(no_deprecated_diagnostic(obj_prop.key.span(), deprecated, replacement));
        }
    }

    fn check_import_specifier(
        import_specifier: &oxc_ast::ast::ImportSpecifier,
        node: &AstNode,
        ctx: &LintContext,
    ) {
        let imported_name = import_specifier.imported.name();
        let react_settings = &ctx.settings().react;

        // Check for deprecated imports from 'react'
        if Self::is_import_from_react(node, ctx) {
            if imported_name == "createClass" && react_settings.version_at_least(15, 5, 0) {
                ctx.diagnostic(no_deprecated_diagnostic(
                    import_specifier.span,
                    &imported_name,
                    "create-react-class package",
                ));
            } else if imported_name == "PropTypes" && react_settings.version_at_least(15, 5, 0) {
                ctx.diagnostic(no_deprecated_diagnostic(
                    import_specifier.span,
                    &imported_name,
                    "prop-types package",
                ));
            }
        }

        // Check for deprecated imports from react-addons-perf
        if DEPRECATED_PERF_EXPORTS.contains(&imported_name.as_str())
            && Self::is_import_from_react_addons_perf(node, ctx)
        {
            ctx.diagnostic(no_deprecated_diagnostic(import_specifier.span, &imported_name, ""));
        }
    }

    fn check_variable_declarator(var_decl: &oxc_ast::ast::VariableDeclarator, ctx: &LintContext) {
        let Some(init) = &var_decl.init else {
            return;
        };

        let oxc_ast::ast::BindingPattern::ObjectPattern(pattern) = &var_decl.id else {
            return;
        };

        let react_settings = &ctx.settings().react;
        let pragma = react_settings.get_pragma();

        // Determine the source type based on the initializer
        let source_type = match init {
            Expression::CallExpression(call_expr) => {
                if Self::is_require_react(call_expr) {
                    VariableDeclaratorSource::React
                } else {
                    VariableDeclaratorSource::Other
                }
            }
            Expression::Identifier(ident) => {
                if ident.name == pragma {
                    VariableDeclaratorSource::React
                } else if Self::is_react_addons_perf_binding(&ident.name, ctx) {
                    VariableDeclaratorSource::ReactAddonsPerf
                } else {
                    VariableDeclaratorSource::Other
                }
            }
            _ => VariableDeclaratorSource::Other,
        };

        match source_type {
            VariableDeclaratorSource::React => {
                for prop in &pattern.properties {
                    let Some(key) = prop.key.static_name() else {
                        continue;
                    };
                    if key == "createClass" && react_settings.version_at_least(15, 5, 0) {
                        ctx.diagnostic(no_deprecated_diagnostic(
                            prop.span,
                            &key,
                            "create-react-class package",
                        ));
                    } else if key == "PropTypes" && react_settings.version_at_least(15, 5, 0) {
                        ctx.diagnostic(no_deprecated_diagnostic(
                            prop.span,
                            &key,
                            "prop-types package",
                        ));
                    }
                }
            }
            VariableDeclaratorSource::ReactAddonsPerf => {
                for prop in &pattern.properties {
                    let Some(key) = prop.key.static_name() else {
                        continue;
                    };
                    if DEPRECATED_PERF_EXPORTS.contains(&key.as_ref()) {
                        ctx.diagnostic(no_deprecated_diagnostic(prop.span, &key, ""));
                    }
                }
            }
            VariableDeclaratorSource::Other => {}
        }
    }

    /// Check if an identifier was imported from 'react-addons-perf' or required from it
    fn is_react_addons_perf_binding(name: &str, ctx: &LintContext) -> bool {
        // Check import entries
        if ctx.module_record().import_entries.iter().any(|entry| {
            entry.module_request.name() == "react-addons-perf" && entry.local_name.name() == name
        }) {
            return true;
        }

        // Check for require('react-addons-perf') pattern by looking at semantic bindings
        for symbol_id in ctx.scoping().symbol_ids() {
            if ctx.scoping().symbol_name(symbol_id) != name {
                continue;
            }

            // Get the declaration node for this symbol
            let decl_id = ctx.scoping().symbol_declaration(symbol_id);
            let decl_node = ctx.nodes().get_node(decl_id);

            // Check if this is a variable declarator with require('react-addons-perf')
            if let AstKind::VariableDeclarator(var_decl) = decl_node.kind()
                && let Some(Expression::CallExpression(call_expr)) = &var_decl.init
                && Self::is_require_react_addons_perf(call_expr)
            {
                return true;
            }
        }

        false
    }

    /// Check if import is from 'react'
    fn is_import_from_react(node: &AstNode, ctx: &LintContext) -> bool {
        for ancestor_id in ctx.nodes().ancestor_ids(node.id()) {
            let ancestor = ctx.nodes().get_node(ancestor_id);
            if let AstKind::ImportDeclaration(import_decl) = ancestor.kind() {
                return import_decl.source.value == "react";
            }
        }
        false
    }

    /// Check if import is from 'react-addons-perf'
    fn is_import_from_react_addons_perf(node: &AstNode, ctx: &LintContext) -> bool {
        for ancestor_id in ctx.nodes().ancestor_ids(node.id()) {
            let ancestor = ctx.nodes().get_node(ancestor_id);
            if let AstKind::ImportDeclaration(import_decl) = ancestor.kind() {
                return import_decl.source.value == "react-addons-perf";
            }
        }
        false
    }

    /// Check if this is require('react')
    fn is_require_react(call_expr: &oxc_ast::ast::CallExpression) -> bool {
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return false;
        };
        if ident.name != "require" {
            return false;
        }
        let Some(arg) = call_expr.arguments.first() else {
            return false;
        };
        let Some(Expression::StringLiteral(lit)) = arg.as_expression() else {
            return false;
        };
        lit.value == "react"
    }

    /// Check if this is require('react-addons-perf')
    fn is_require_react_addons_perf(call_expr: &oxc_ast::ast::CallExpression) -> bool {
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return false;
        };
        if ident.name != "require" {
            return false;
        }
        let Some(arg) = call_expr.arguments.first() else {
            return false;
        };
        let Some(Expression::StringLiteral(lit)) = arg.as_expression() else {
            return false;
        };
        lit.value == "react-addons-perf"
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var element = React.createElement('p', {}, null);", None, None),
        ("var clone = React.cloneElement(element);", None, None),
        ("ReactDOM.cloneElement(child, container);", None, None),
        ("ReactDOM.findDOMNode(instance);", None, None),
        ("ReactDOM.createPortal(child, container);", None, None),
        ("ReactDOMServer.renderToString(element);", None, None),
        ("ReactDOMServer.renderToStaticMarkup(element);", None, None),
        (
            "
            var Foo = createReactClass({
              render: function() {}
            })
            ",
            None,
            None,
        ),
        (
            "
            var Foo = createReactClassNonReact({
              componentWillMount: function() {},
              componentWillReceiveProps: function() {},
              componentWillUpdate: function() {}
            });
            ",
            None,
            None,
        ),
        (
            "
            var Foo = {
              componentWillMount: function() {},
              componentWillReceiveProps: function() {},
              componentWillUpdate: function() {}
            };
            ",
            None,
            None,
        ),
        (
            "
            class Foo {
              constructor() {}
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            }
            ",
            None,
            None,
        ),
        // Version-based tests: these should pass because the version is before deprecation
        (
            "React.renderComponent()",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "0.11.0" } } })),
        ),
        (
            "React.createClass()",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "15.4.0" } } })),
        ),
        (
            "PropTypes",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "15.4.0" } } })),
        ),
        (
            "
            class Foo extends React.Component {
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            }
            ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.8.0" } } })),
        ),
        (
            r#"
            import React from "react";
            let { default: defaultReactExport, ...allReactExports } = React;
            "#,
            None,
            None,
        ),
        (
            "
            import { render, hydrate } from 'react-dom';
            import { renderToNodeStream } from 'react-dom/server';
            ReactDOM.render(element, container);
            ReactDOM.unmountComponentAtNode(container);
            ReactDOMServer.renderToNodeStream(element);
            ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "17.999.999" } } })),
        ),
        (
            "
            import ReactDOM, { createRoot } from 'react-dom/client';
            ReactDOM.createRoot(container);
            const root = createRoot(container);
            root.unmount();
            ",
            None,
            None,
        ),
        (
            "
            import ReactDOM, { hydrateRoot } from 'react-dom/client';
            ReactDOM.hydrateRoot(container, <App/>);
            hydrateRoot(container, <App/>);
            ",
            None,
            None,
        ),
        (
            "
            import ReactDOMServer, { renderToPipeableStream } from 'react-dom/server';
            ReactDOMServer.renderToPipeableStream(<App />, {});
            renderToPipeableStream(<App />, {});
            ",
            None,
            None,
        ),
        ("import { renderToString } from 'react-dom/server';", None, None),
        ("const { renderToString } = require('react-dom/server');", None, None),
    ];

    let fail = vec![
        ("React.renderComponent()", None, None),
        // Pragma test: Foo is used as React pragma
        (
            "Foo.renderComponent()",
            None,
            Some(serde_json::json!({ "settings": { "react": { "pragma": "Foo" } } })),
        ),
        // Skip: JSX comment pragma not supported
        // ("/** @jsx Foo */ Foo.renderComponent()", None, None),
        ("this.transferPropsTo()", None, None),
        ("React.addons.TestUtils", None, None),
        ("React.addons.classSet()", None, None),
        ("React.render(element, container);", None, None),
        ("React.unmountComponentAtNode(container);", None, None),
        ("React.findDOMNode(instance);", None, None),
        ("React.renderToString(element);", None, None),
        ("React.renderToStaticMarkup(element);", None, None),
        ("React.createClass({});", None, None),
        // Pragma test: Foo is used as React pragma
        (
            "Foo.createClass({});",
            None,
            Some(serde_json::json!({ "settings": { "react": { "pragma": "Foo" } } })),
        ),
        ("React.PropTypes", None, None),
        ("var {createClass} = require('react');", None, None),
        ("var {createClass, PropTypes} = require('react');", None, None),
        ("import {createClass} from 'react';", None, None),
        ("import {createClass, PropTypes} from 'react';", None, None),
        (
            "
            import React from 'react';
            const {createClass, PropTypes} = React;
            ",
            None,
            None,
        ),
        ("import {printDOM} from 'react-addons-perf';", None, None),
        (
            "
            const ReactPerf = require('react-addons-perf');
            const {printDOM} = ReactPerf;
            ",
            None,
            None,
        ),
        (
            "
            import ReactPerf from 'react-addons-perf';
            const {printDOM} = ReactPerf;
            ",
            None,
            None,
        ),
        ("React.DOM.div", None, None),
        (
            "
            class Bar extends React.PureComponent {
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            };
            ",
            None,
            None,
        ),
        (
            "
            function Foo() {
              return class Bar extends React.PureComponent {
                componentWillMount() {}
                componentWillReceiveProps() {}
                componentWillUpdate() {}
              };
            }
            ",
            None,
            None,
        ),
        (
            "
            class Bar extends PureComponent {
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            };
            ",
            None,
            None,
        ),
        (
            "
            class Foo extends React.Component {
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            }
            ",
            None,
            None,
        ),
        (
            "
            class Foo extends Component {
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            }
            ",
            None,
            None,
        ),
        (
            "
            var Foo = createReactClass({
              componentWillMount: function() {},
              componentWillReceiveProps: function() {},
              componentWillUpdate: function() {}
            })
            ",
            None,
            None,
        ),
        (
            "
            class Foo extends React.Component {
              constructor() {}
              componentWillMount() {}
              componentWillReceiveProps() {}
              componentWillUpdate() {}
            }
            ",
            None,
            None,
        ),
        (
            "
            import { render } from 'react-dom';
            ReactDOM.render(<div></div>, container);
            ",
            None,
            None,
        ),
        (
            "
            import { hydrate } from 'react-dom';
            ReactDOM.hydrate(<div></div>, container);
            ",
            None,
            None,
        ),
        (
            "
            import { unmountComponentAtNode } from 'react-dom';
            ReactDOM.unmountComponentAtNode(container);
            ",
            None,
            None,
        ),
        (
            "
            import { renderToNodeStream } from 'react-dom/server';
            ReactDOMServer.renderToNodeStream(element);
            ",
            None,
            None,
        ),
    ];

    Tester::new(NoDeprecated::NAME, NoDeprecated::PLUGIN, pass, fail).test_and_snapshot();
}
