use std::fmt::Write;

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, Expression, ImportDeclarationSpecifier, PropertyKey, StaticMemberExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    config::ReactVersion,
    context::LintContext,
    rule::Rule,
    utils::{get_parent_component, is_es5_component},
};

#[derive(Debug, Clone, Copy)]
struct DeprecatedApi {
    name: &'static str,
    since: (u32, u32, u32),
    replacement: Option<&'static str>,
    reference: Option<&'static str>,
}

const DEPRECATED_APIS: &[DeprecatedApi] = &[
    DeprecatedApi {
        name: "React.renderComponent",
        since: (0, 12, 0),
        replacement: Some("React.render"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.renderComponentToString",
        since: (0, 12, 0),
        replacement: Some("React.renderToString"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.renderComponentToStaticMarkup",
        since: (0, 12, 0),
        replacement: Some("React.renderToStaticMarkup"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.isValidComponent",
        since: (0, 12, 0),
        replacement: Some("React.isValidElement"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.PropTypes.component",
        since: (0, 12, 0),
        replacement: Some("React.PropTypes.element"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.PropTypes.renderable",
        since: (0, 12, 0),
        replacement: Some("React.PropTypes.node"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.isValidClass",
        since: (0, 12, 0),
        replacement: None,
        reference: None,
    },
    DeprecatedApi {
        name: "this.transferPropsTo",
        since: (0, 12, 0),
        replacement: Some("spread operator ({...})"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.addons.classSet",
        since: (0, 13, 0),
        replacement: Some("the npm module classnames"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.addons.cloneWithProps",
        since: (0, 13, 0),
        replacement: Some("React.cloneElement"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.render",
        since: (0, 14, 0),
        replacement: Some("ReactDOM.render"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.unmountComponentAtNode",
        since: (0, 14, 0),
        replacement: Some("ReactDOM.unmountComponentAtNode"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.findDOMNode",
        since: (0, 14, 0),
        replacement: Some("ReactDOM.findDOMNode"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.renderToString",
        since: (0, 14, 0),
        replacement: Some("ReactDOMServer.renderToString"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.renderToStaticMarkup",
        since: (0, 14, 0),
        replacement: Some("ReactDOMServer.renderToStaticMarkup"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.addons.LinkedStateMixin",
        since: (15, 0, 0),
        replacement: None,
        reference: None,
    },
    DeprecatedApi {
        name: "ReactPerf.printDOM",
        since: (15, 0, 0),
        replacement: Some("ReactPerf.printOperations"),
        reference: None,
    },
    DeprecatedApi {
        name: "Perf.printDOM",
        since: (15, 0, 0),
        replacement: Some("Perf.printOperations"),
        reference: None,
    },
    DeprecatedApi {
        name: "ReactPerf.getMeasurementsSummaryMap",
        since: (15, 0, 0),
        replacement: Some("ReactPerf.getWasted"),
        reference: None,
    },
    DeprecatedApi {
        name: "Perf.getMeasurementsSummaryMap",
        since: (15, 0, 0),
        replacement: Some("Perf.getWasted"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.createClass",
        since: (15, 5, 0),
        replacement: Some("the npm module create-react-class"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.addons.TestUtils",
        since: (15, 5, 0),
        replacement: Some("ReactDOM.TestUtils"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.PropTypes",
        since: (15, 5, 0),
        replacement: Some("the npm module prop-types"),
        reference: None,
    },
    DeprecatedApi {
        name: "React.DOM",
        since: (15, 6, 0),
        replacement: Some("the npm module react-dom-factories"),
        reference: None,
    },
    DeprecatedApi {
        name: "componentWillMount",
        since: (16, 9, 0),
        replacement: Some("UNSAFE_componentWillMount"),
        reference: Some(
            "https://reactjs.org/docs/react-component.html#unsafe_componentwillmount. Use https://github.com/reactjs/react-codemod#rename-unsafe-lifecycles to automatically update your components.",
        ),
    },
    DeprecatedApi {
        name: "componentWillReceiveProps",
        since: (16, 9, 0),
        replacement: Some("UNSAFE_componentWillReceiveProps"),
        reference: Some(
            "https://reactjs.org/docs/react-component.html#unsafe_componentwillreceiveprops. Use https://github.com/reactjs/react-codemod#rename-unsafe-lifecycles to automatically update your components.",
        ),
    },
    DeprecatedApi {
        name: "componentWillUpdate",
        since: (16, 9, 0),
        replacement: Some("UNSAFE_componentWillUpdate"),
        reference: Some(
            "https://reactjs.org/docs/react-component.html#unsafe_componentwillupdate. Use https://github.com/reactjs/react-codemod#rename-unsafe-lifecycles to automatically update your components.",
        ),
    },
    DeprecatedApi {
        name: "ReactDOM.render",
        since: (18, 0, 0),
        replacement: Some("createRoot"),
        reference: Some("https://reactjs.org/link/switch-to-createroot"),
    },
    DeprecatedApi {
        name: "ReactDOM.hydrate",
        since: (18, 0, 0),
        replacement: Some("hydrateRoot"),
        reference: Some("https://reactjs.org/link/switch-to-createroot"),
    },
    DeprecatedApi {
        name: "ReactDOM.unmountComponentAtNode",
        since: (18, 0, 0),
        replacement: Some("root.unmount"),
        reference: Some("https://reactjs.org/link/switch-to-createroot"),
    },
    DeprecatedApi {
        name: "ReactDOMServer.renderToNodeStream",
        since: (18, 0, 0),
        replacement: Some("renderToPipeableStream"),
        reference: Some("https://reactjs.org/docs/react-dom-server.html#rendertonodestream"),
    },
];

fn no_deprecated_diagnostic(api: &DeprecatedApi, span: Span) -> OxcDiagnostic {
    let mut message = format!(
        "{} is deprecated since React {}.{}.{}",
        api.name, api.since.0, api.since.1, api.since.2
    );
    if let Some(replacement) = api.replacement {
        write!(message, ", use {replacement} instead").unwrap();
    }

    let diagnostic = OxcDiagnostic::warn(message).with_label(span);
    if let Some(reference) = api.reference {
        diagnostic.with_help(format!("See {reference}"))
    } else {
        diagnostic
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecated;

// <https://github.com/jsx-eslint/eslint-plugin-react/blob/master/docs/rules/no-deprecated.md>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows usage of APIs deprecated by React.
    ///
    /// ### Why is this bad?
    ///
    /// Deprecated React APIs may be removed in future React releases and often have
    /// safer or better-supported replacements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// React.render(<MyComponent />, root);
    /// React.createClass({});
    /// ReactDOM.render(<div />, container);
    /// componentWillMount() {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { createRoot } from 'react-dom/client';
    /// const root = createRoot(container);
    /// UNSAFE_componentWillMount() {}
    /// ```
    NoDeprecated,
    react,
    correctness,
    version = "1.63.0",
);

impl Rule for NoDeprecated {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(member_expr) => {
                if let Some(name) = static_member_path(member_expr) {
                    self.check_deprecation(&name, member_expr.span, ctx);
                }
            }
            AstKind::ImportDeclaration(import_decl) => {
                let Some(module_name) = canonical_module_name(import_decl.source.value.as_str())
                else {
                    return;
                };
                let Some(specifiers) = &import_decl.specifiers else {
                    return;
                };

                for specifier in specifiers {
                    let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) = specifier
                    else {
                        continue;
                    };
                    let name = format!("{module_name}.{}", import_specifier.imported.name());
                    self.check_deprecation(&name, import_specifier.span, ctx);
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                let BindingPattern::ObjectPattern(pattern) = &declarator.id else {
                    return;
                };
                let Some(init) = &declarator.init else {
                    return;
                };
                let Some(module_name) = initializer_module_name(init) else {
                    return;
                };

                for property in &pattern.properties {
                    let Some((property_name, span)) = property_key_name_and_span(&property.key)
                    else {
                        continue;
                    };
                    let name = format!("{module_name}.{property_name}");
                    self.check_deprecation(&name, span, ctx);
                }
            }
            AstKind::MethodDefinition(method_def) => {
                let Some(name) = method_def.key.static_name() else {
                    return;
                };
                if get_parent_component(node, ctx).is_some() {
                    self.check_deprecation(name.as_ref(), method_def.key.span(), ctx);
                }
            }
            AstKind::ObjectProperty(obj_prop) => {
                let Some(name) = obj_prop.key.static_name() else {
                    return;
                };
                if ctx.nodes().ancestors(node.id()).any(is_es5_component) {
                    self.check_deprecation(name.as_ref(), obj_prop.key.span(), ctx);
                }
            }
            _ => {}
        }
    }
}

impl NoDeprecated {
    fn check_deprecation(&self, name: &str, span: Span, ctx: &LintContext<'_>) {
        let Some(api) = DEPRECATED_APIS.iter().find(|api| api.name == name) else {
            return;
        };
        if is_react_version_at_least(ctx.settings().react.version.as_ref(), api.since) {
            ctx.diagnostic(no_deprecated_diagnostic(api, span));
        }
    }
}

fn is_react_version_at_least(version: Option<&ReactVersion>, since: (u32, u32, u32)) -> bool {
    version.is_none_or(|version| version.is_at_least(since.0, since.1, since.2))
}

fn canonical_module_name(module_name: &str) -> Option<&'static str> {
    match module_name {
        "react" => Some("React"),
        "react-addons-perf" => Some("ReactPerf"),
        "react-dom" => Some("ReactDOM"),
        "react-dom/server" => Some("ReactDOMServer"),
        _ => None,
    }
}

fn canonical_object_name(name: &str) -> Option<&'static str> {
    match name {
        "React" => Some("React"),
        "ReactPerf" => Some("ReactPerf"),
        "Perf" => Some("Perf"),
        "ReactDOM" => Some("ReactDOM"),
        "ReactDOMServer" => Some("ReactDOMServer"),
        _ => None,
    }
}

fn initializer_module_name(init: &Expression<'_>) -> Option<&'static str> {
    match init.without_parentheses() {
        Expression::Identifier(ident) => canonical_object_name(ident.name.as_str()),
        Expression::CallExpression(call) => {
            canonical_module_name(call.common_js_require()?.value.as_str())
        }
        _ => None,
    }
}

fn static_member_path(member_expr: &StaticMemberExpression<'_>) -> Option<String> {
    let object = expression_path(member_expr.object.without_parentheses())?;
    Some(format!("{object}.{}", member_expr.property.name))
}

fn expression_path(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::Identifier(ident) => Some(ident.name.to_string()),
        Expression::ThisExpression(_) => Some("this".to_string()),
        Expression::StaticMemberExpression(member_expr) => static_member_path(member_expr),
        Expression::ParenthesizedExpression(paren) => expression_path(&paren.expression),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member_expr) => {
                static_member_path(member_expr)
            }
            _ => None,
        },
        _ => None,
    }
}

fn property_key_name_and_span<'a>(key: &'a PropertyKey<'a>) -> Option<(&'a str, Span)> {
    match key {
        PropertyKey::Identifier(ident) => Some((ident.name.as_str(), ident.span)),
        PropertyKey::StaticIdentifier(ident) => Some((ident.name.as_str(), ident.span)),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var element = React.createElement('p', {}, null);", None, None),
        ("var clone = React.cloneElement(element);", None, None),
        ("ReactDOM.cloneElement(child, container);", None, None),
        (
            "ReactDOM.findDOMNode(instance);",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "17.0.0" } } })),
        ),
        ("ReactDOM.createPortal(child, container);", None, None),
        ("ReactDOMServer.renderToString(element);", None, None),
        ("ReactDOMServer.renderToStaticMarkup(element);", None, None),
        ("var Foo = createReactClass({ render: function() {} })", None, None),
        (
            "var Foo = createReactClassNonReact({ componentWillMount: function() {}, componentWillReceiveProps: function() {}, componentWillUpdate: function() {} });",
            None,
            None,
        ),
        (
            "var Foo = { componentWillMount: function() {}, componentWillReceiveProps: function() {}, componentWillUpdate: function() {} };",
            None,
            None,
        ),
        (
            "class Foo { constructor() {} componentWillMount() {} componentWillReceiveProps() {} componentWillUpdate() {} }",
            None,
            None,
        ),
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
            "React.PropTypes",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "15.4.0" } } })),
        ),
        (
            "class Foo extends React.Component { componentWillMount() {} componentWillReceiveProps() {} componentWillUpdate() {} }",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.8.0" } } })),
        ),
        (
            "import React from 'react'; let { default: defaultReactExport, ...allReactExports } = React;",
            None,
            None,
        ),
        (
            "import { render, hydrate } from 'react-dom'; import { renderToNodeStream } from 'react-dom/server'; ReactDOM.render(element, container); ReactDOM.unmountComponentAtNode(container); ReactDOMServer.renderToNodeStream(element);",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "17.999.999" } } })),
        ),
        (
            "import ReactDOM, { createRoot } from 'react-dom/client'; ReactDOM.createRoot(container); const root = createRoot(container); root.unmount();",
            None,
            None,
        ),
        (
            "import ReactDOM, { hydrateRoot } from 'react-dom/client'; ReactDOM.hydrateRoot(container, <App/>); hydrateRoot(container, <App/>);",
            None,
            None,
        ),
        (
            "import ReactDOMServer, { renderToPipeableStream } from 'react-dom/server'; ReactDOMServer.renderToPipeableStream(<App />, {}); renderToPipeableStream(<App />, {});",
            None,
            None,
        ),
        ("import { renderToString } from 'react-dom/server';", None, None),
        ("const { renderToString } = require('react-dom/server');", None, None),
    ];

    let fail = vec![
        ("React.renderComponent()", None, None),
        ("this.transferPropsTo()", None, None),
        ("React.addons.TestUtils", None, None),
        ("React.addons.classSet()", None, None),
        ("React.render(element, container);", None, None),
        ("React.unmountComponentAtNode(container);", None, None),
        ("React.findDOMNode(instance);", None, None),
        ("React.renderToString(element);", None, None),
        ("React.renderToStaticMarkup(element);", None, None),
        ("React.createClass({});", None, None),
        ("React.PropTypes", None, None),
        ("var {createClass} = require('react');", None, None),
        ("var {createClass, PropTypes} = require('react');", None, None),
        ("import {createClass} from 'react';", None, None),
        ("import {createClass, PropTypes} from 'react';", None, None),
        ("import React from 'react'; const {createClass, PropTypes} = React;", None, None),
        ("import {printDOM} from 'react-addons-perf';", None, None),
        ("import ReactPerf from 'react-addons-perf'; const {printDOM} = ReactPerf;", None, None),
        ("const {printDOM} = Perf;", None, None),
        ("React.DOM.div", None, None),
        (
            "class Bar extends React.PureComponent { componentWillMount() {} componentWillReceiveProps() {} componentWillUpdate() {} };",
            None,
            None,
        ),
        (
            "function Foo() { return class Bar extends React.PureComponent { componentWillMount() {} componentWillReceiveProps() {} componentWillUpdate() {} }; }",
            None,
            None,
        ),
        (
            "class Bar extends PureComponent { componentWillMount() {} componentWillReceiveProps() {} componentWillUpdate() {} };",
            None,
            None,
        ),
        (
            "class Foo extends React.Component { constructor() {} componentWillMount() {} componentWillReceiveProps() {} componentWillUpdate() {} }",
            None,
            None,
        ),
        (
            "var Foo = createReactClass({ componentWillMount: function() {}, componentWillReceiveProps: function() {}, componentWillUpdate: function() {} })",
            None,
            None,
        ),
        (
            "import { render } from 'react-dom'; ReactDOM.render(<div></div>, container);",
            None,
            None,
        ),
        (
            "import { hydrate } from 'react-dom'; ReactDOM.hydrate(<div></div>, container);",
            None,
            None,
        ),
        (
            "import { unmountComponentAtNode } from 'react-dom'; ReactDOM.unmountComponentAtNode(container);",
            None,
            None,
        ),
        (
            "import { renderToNodeStream } from 'react-dom/server'; ReactDOMServer.renderToNodeStream(element);",
            None,
            None,
        ),
    ];

    Tester::new(NoDeprecated::NAME, NoDeprecated::PLUGIN, pass, fail).test_and_snapshot();
}
