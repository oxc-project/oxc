use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        BindingIdentifier, Expression, FunctionType, PrivateFieldExpression, PropertyKey,
        StaticMemberExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_underscore_dangle_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected dangling '_' in '`{name}`'."))
        .with_help(format!("Remove the dangling '_' or add `{name}` to the 'allow' configuration."))
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoUnderscoreDangleConfig {
    /// Whether to allow dangling underscores in members of the `super` object.
    allow_after_super: bool,
    /// Whether to allow dangling underscores in members of the `this.constructor` object.
    allow_after_this_constructor: bool,
    /// An array of variable names that are allowed to have dangling underscores.
    allow: Vec<String>,
    /// Whether to allow dangling underscores in members of the `this` object.
    allow_after_this: bool,
    /// Whether to allow dangling underscores in variable names assigned by array destructuring.
    allow_in_array_destructuring: bool,
    /// Whether to allow dangling underscores in variable names assigned by object destructuring.
    allow_in_object_destructuring: bool,
    /// Whether to allow dangling underscores in function parameter names.
    allow_function_params: bool,
    /// Whether to enforce dangling underscores in class field names.
    enforce_in_class_fields: bool,
    /// Whether to enforce dangling underscores in method names.
    enforce_in_method_names: bool,
}

impl std::ops::Deref for NoUnderscoreDangle {
    type Target = NoUnderscoreDangleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoUnderscoreDangle(Box<NoUnderscoreDangleConfig>);

impl Default for NoUnderscoreDangleConfig {
    fn default() -> Self {
        Self {
            allow_after_super: false,
            allow_after_this_constructor: false,
            allow: Vec::new(),
            allow_after_this: false,
            allow_in_array_destructuring: true,
            allow_in_object_destructuring: true,
            allow_function_params: true,
            enforce_in_class_fields: false,
            enforce_in_method_names: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows dangling underscores in identifiers.
    ///
    /// ### Why is this bad?
    ///
    /// There is a long history of using `_` as a prefix or suffix for private members in JavaScript.
    /// It is however recommended to use the formal private class feature introduced in ES2022.
    /// See <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_elements> for more information.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let foo_;
    /// const __proto__ = {};
    /// foo._bar();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const _ = require('underscore');
    /// const obj = _.contains(items, item);
    /// obj.__proto__ = {};
    /// const file = __filename;
    /// function foo(_bar) {};
    /// const bar = { onClick(_bar) {} };
    /// const baz = (_bar) => {};
    /// ```
    NoUnderscoreDangle,
    eslint,
    suspicious,
    config = NoUnderscoreDangle,
    version = "next",
);

impl Rule for NoUnderscoreDangle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(expr) => self.check_member(ctx, expr),
            AstKind::PrivateFieldExpression(expr) => self.check_private_member(ctx, expr),
            AstKind::BindingIdentifier(ident) => self.check_binding(ctx, node.id(), ident),
            AstKind::MethodDefinition(_) | AstKind::ObjectProperty(_)
                if !self.enforce_in_method_names => {}
            AstKind::PropertyDefinition(_) if !self.enforce_in_class_fields => {}
            AstKind::Function(func) if func.r#type == FunctionType::FunctionExpression => {}
            _ => {
                if let Some((name, span)) = get_identifier(node) {
                    self.report(ctx, span, name);
                }
            }
        }
    }
}

enum BindingContext {
    ArrayDestructure,
    ObjectDestructure,
    FunctionParam,
    Plain,
    NotInteresting,
}

impl NoUnderscoreDangle {
    fn is_allowed(&self, name: &str) -> bool {
        is_always_allowed(name) || self.allow.contains(&name.to_string())
    }

    fn report(&self, ctx: &LintContext, span: Span, name: &str) {
        if self.is_allowed(name) {
            return;
        }
        ctx.diagnostic(no_underscore_dangle_diagnostic(span, name));
    }

    fn check_member(&self, ctx: &LintContext, expr: &StaticMemberExpression) {
        let prop = &expr.property;
        if is_prototype_accessor(prop.name.as_str()) {
            return;
        }
        if self.member_object_is_allowed(&expr.object) {
            return;
        }
        self.report(ctx, prop.span, prop.name.as_str());
    }

    fn check_private_member(&self, ctx: &LintContext, expr: &PrivateFieldExpression) {
        if self.member_object_is_allowed(&expr.object) {
            return;
        }
        self.report(ctx, expr.field.span, expr.field.name.as_str());
    }

    fn member_object_is_allowed(&self, object: &Expression) -> bool {
        match object.get_inner_expression() {
            Expression::ThisExpression(_) => self.allow_after_this,
            Expression::Super(_) => self.allow_after_super,
            Expression::StaticMemberExpression(inner) => {
                self.allow_after_this_constructor
                    && inner.property.name == "constructor"
                    && matches!(inner.object, Expression::ThisExpression(_))
            }
            _ => false,
        }
    }

    fn check_binding(&self, ctx: &LintContext, id: NodeId, ident: &BindingIdentifier) {
        let allowed = match binding_context(ctx, id) {
            BindingContext::ArrayDestructure => self.allow_in_array_destructuring,
            BindingContext::ObjectDestructure => self.allow_in_object_destructuring,
            BindingContext::FunctionParam => self.allow_function_params,
            BindingContext::Plain => false,
            BindingContext::NotInteresting => return,
        };
        if allowed {
            return;
        }
        self.report(ctx, ident.span, ident.name.as_str());
    }
}

fn binding_context(ctx: &LintContext, id: NodeId) -> BindingContext {
    let mut destructure_context = None;
    for ancestor in ctx.nodes().ancestors(id) {
        match ancestor.kind() {
            // skip transparent wrappers
            AstKind::AssignmentPattern(_)
            | AstKind::BindingProperty(_)
            | AstKind::BindingRestElement(_)
            | AstKind::FormalParameter(_)
            | AstKind::FormalParameterRest(_) => {}
            AstKind::ArrayPattern(_) => {
                destructure_context.get_or_insert(BindingContext::ArrayDestructure);
            }
            AstKind::ObjectPattern(_) => {
                destructure_context.get_or_insert(BindingContext::ObjectDestructure);
            }
            AstKind::FormalParameters(_) => {
                return if destructure_context.is_some() {
                    BindingContext::NotInteresting
                } else {
                    BindingContext::FunctionParam
                };
            }
            AstKind::VariableDeclarator(_) => {
                return destructure_context.unwrap_or(BindingContext::Plain);
            }
            _ => return BindingContext::NotInteresting,
        }
    }
    BindingContext::NotInteresting
}

fn get_identifier<'a>(node: &AstNode<'a>) -> Option<(&'a str, Span)> {
    match node.kind() {
        AstKind::Function(f) => f.id.as_ref().map(|id| (id.name.as_str(), id.span)),
        AstKind::MethodDefinition(m) => property_key_name_span(&m.key),
        AstKind::PropertyDefinition(p) => property_key_name_span(&p.key),
        AstKind::ObjectProperty(o) if o.method => property_key_name_span(&o.key),
        _ => None,
    }
}

fn has_dangling_underscore(name: &str) -> bool {
    name.starts_with('_') || name.starts_with("#_") || name.ends_with('_')
}

fn is_underscore_only(name: &str) -> bool {
    name == "_"
}

fn is_prototype_accessor(name: &str) -> bool {
    name == "__proto__"
}

fn is_always_allowed(name: &str) -> bool {
    !has_dangling_underscore(name) || is_underscore_only(name)
}

fn property_key_name_span<'a>(key: &PropertyKey<'a>) -> Option<(&'a str, Span)> {
    match key {
        PropertyKey::StaticIdentifier(i) => Some((i.name.as_str(), i.span)),
        PropertyKey::PrivateIdentifier(i) => Some((i.name.as_str(), i.span)),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo_bar = 1;", None),
        ("function foo_bar() {}", None),
        ("foo.bar.__proto__;", None),
        ("console.log(__filename); console.log(__dirname);", None),
        ("var _ = require('underscore');", None),
        ("var a = b._;", None),
        ("function foo(_bar) {}", None),
        ("function foo(bar_) {}", None),
        ("(function _foo() {})", None),
        ("function foo(_bar) {}", Some(serde_json::json!([{}]))),
        ("function foo( _bar = 0) {}", None), // { "ecmaVersion": 6 },
        ("const foo = { onClick(_bar) { } }", None), // { "ecmaVersion": 6 },
        ("const foo = { onClick(_bar = 0) { } }", None), // { "ecmaVersion": 6 },
        ("const foo = (_bar) => {}", None),   // { "ecmaVersion": 6 },
        ("const foo = (_bar = 0) => {}", None), // { "ecmaVersion": 6 },
        ("function foo( ..._bar) {}", None),  // { "ecmaVersion": 6 },
        ("const foo = (..._bar) => {}", None), // { "ecmaVersion": 6 },
        ("const foo = { onClick(..._bar) { } }", None), // { "ecmaVersion": 6 },
        ("export default function() {}", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var _foo = 1", Some(serde_json::json!([{ "allow": ["_foo"] }]))),
        ("var __proto__ = 1;", Some(serde_json::json!([{ "allow": ["__proto__"] }]))),
        ("foo._bar;", Some(serde_json::json!([{ "allow": ["_bar"] }]))),
        ("function _foo() {}", Some(serde_json::json!([{ "allow": ["_foo"] }]))),
        ("this._bar;", Some(serde_json::json!([{ "allowAfterThis": true }]))),
        (
            "class foo { constructor() { super._bar; } }",
            Some(serde_json::json!([{ "allowAfterSuper": true }])),
        ), // { "ecmaVersion": 6 },
        ("class foo { _onClick() { } }", None), // { "ecmaVersion": 6 },
        ("class foo { onClick_() { } }", None), // { "ecmaVersion": 6 },
        ("const o = { _onClick() { } }", None), // { "ecmaVersion": 6 },
        ("const o = { onClick_() { } }", None), // { "ecmaVersion": 6 },
        (
            "const o = { _onClick() { } }",
            Some(serde_json::json!([{ "allow": ["_onClick"], "enforceInMethodNames": true }])),
        ), // { "ecmaVersion": 6 },
        ("const o = { _foo: 'bar' }", None),    // { "ecmaVersion": 6 },
        ("const o = { foo_: 'bar' }", None),    // { "ecmaVersion": 6 },
        ("this.constructor._bar", Some(serde_json::json!([{ "allowAfterThisConstructor": true }]))),
        ("const foo = { onClick(bar) { } }", None), // { "ecmaVersion": 6 },
        ("const foo = (bar) => {}", None),          // { "ecmaVersion": 6 },
        ("function foo(_bar) {}", Some(serde_json::json!([{ "allowFunctionParams": true }]))),
        ("function foo( _bar = 0) {}", Some(serde_json::json!([{ "allowFunctionParams": true }]))), // { "ecmaVersion": 6 },
        (
            "const foo = { onClick(_bar) { } }",
            Some(serde_json::json!([{ "allowFunctionParams": true }])),
        ), // { "ecmaVersion": 6 },
        ("const foo = (_bar) => {}", Some(serde_json::json!([{ "allowFunctionParams": true }]))), // { "ecmaVersion": 6 },
        ("function foo(bar) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        (
            "const foo = { onClick(bar) { } }",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        ("const foo = (bar) => {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        (
            "function foo(_bar) {}",
            Some(serde_json::json!([{ "allowFunctionParams": false, "allow": ["_bar"] }])),
        ),
        (
            "const foo = { onClick(_bar) { } }",
            Some(serde_json::json!([{ "allowFunctionParams": false, "allow": ["_bar"] }])),
        ), // { "ecmaVersion": 6 },
        (
            "const foo = (_bar) => {}",
            Some(serde_json::json!([{ "allowFunctionParams": false, "allow": ["_bar"] }])),
        ), // { "ecmaVersion": 6 },
        ("function foo([_bar]) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        (
            "function foo([_bar] = []) {}",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        ("function foo( { _bar }) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        (
            "function foo( { _bar = 0 } = {}) {}",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        ("function foo(...[_bar]) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 2016 },
        ("const [_foo] = arr", None), // { "ecmaVersion": 6 },
        ("const [_foo] = arr", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("const [_foo] = arr", Some(serde_json::json!([{ "allowInArrayDestructuring": true }]))), // { "ecmaVersion": 6 },
        (
            "const [foo, ...rest] = [1, 2, 3]",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const [foo, _bar] = [1, 2, 3]",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false, "allow": ["_bar"] }])),
        ), // { "ecmaVersion": 2022 },
        ("const { _foo } = obj", None), // { "ecmaVersion": 6 },
        ("const { _foo } = obj", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("const { _foo } = obj", Some(serde_json::json!([{ "allowInObjectDestructuring": true }]))), // { "ecmaVersion": 6 },
        (
            "const { foo, bar: _bar } = { foo: 1, bar: 2 }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false, "allow": ["_bar"] }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo, _bar } = { foo: 1, _bar: 2 }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false, "allow": ["_bar"] }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo, _bar: bar } = { foo: 1, _bar: 2 }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class foo { _field; }", None), // { "ecmaVersion": 2022 },
        ("class foo { _field; }", Some(serde_json::json!([{ "enforceInClassFields": false }]))), // { "ecmaVersion": 2022 },
        ("class foo { #_field; }", None), // { "ecmaVersion": 2022 },
        ("class foo { #_field; }", Some(serde_json::json!([{ "enforceInClassFields": false }]))), // { "ecmaVersion": 2022 },
        ("class foo { _field; }", Some(serde_json::json!([{}]))), // { "ecmaVersion": 2022 },
        ("import foo from 'foo.json' with { _type: 'json' }", None), // { "ecmaVersion": 2025 },
        ("export * from 'foo.json' with { _type: 'json' }", None), // { "ecmaVersion": 2025 },
        ("export { default } from 'foo.json' with { _type: 'json' }", None), // { "ecmaVersion": 2025 },
        ("import('foo.json', { _with: { _type: 'json' } })", None), // { "ecmaVersion": 2025 },
        ("import('foo.json', { 'with': { _type: 'json' } })", None), // { "ecmaVersion": 2025 },
        ("import('foo.json', { _with: { _type } })", None),         // { "ecmaVersion": 2025 }
        ("const o = { _foo: 'bar' }", Some(serde_json::json!([{ "enforceInMethodNames": true }]))), // { "ecmaVersion": 6 },
        (
            "function foo([_bar]) {}",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false }])),
        ), // { "ecmaVersion": 6 },
    ];

    let fail = vec![
        ("var _foo = 1", None),
        ("var foo_ = 1", None),
        ("function _foo() {}", None),
        ("function foo_() {}", None),
        ("var __proto__ = 1;", None),
        ("foo._bar;", None),
        ("this._prop;", None),
        ("class foo { constructor() { super._prop; } }", None), // { "ecmaVersion": 6 },
        (
            "class foo { constructor() { this._prop; } }",
            Some(serde_json::json!([{ "allowAfterSuper": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "class foo { _onClick() { } }",
            Some(serde_json::json!([{ "enforceInMethodNames": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "class foo { onClick_() { } }",
            Some(serde_json::json!([{ "enforceInMethodNames": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "const o = { _onClick() { } }",
            Some(serde_json::json!([{ "enforceInMethodNames": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "const o = { onClick_() { } }",
            Some(serde_json::json!([{ "enforceInMethodNames": true }])),
        ), // { "ecmaVersion": 6 },
        ("this.constructor._bar", None),
        ("function foo(_bar) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))),
        ("(function foo(_bar) {})", Some(serde_json::json!([{ "allowFunctionParams": false }]))),
        ("function foo(bar, _foo) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))),
        (
            "const foo = { onClick(_bar) { } }",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        ("const foo = (_bar) => {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        ("function foo(_bar = 0) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        (
            "const foo = { onClick(_bar = 0) { } }",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "const foo = (_bar = 0) => {}",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        ("function foo(..._bar) {}", Some(serde_json::json!([{ "allowFunctionParams": false }]))), // { "ecmaVersion": 6 },
        (
            "const foo = { onClick(..._bar) { } }",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "const foo = (..._bar) => {}",
            Some(serde_json::json!([{ "allowFunctionParams": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "const [foo, _bar] = [1, 2]",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const [_foo = 1] = arr",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const [foo, ..._rest] = [1, 2, 3]",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const [foo, [bar_, baz]] = [1, [2, 3]]",
            Some(serde_json::json!([{ "allowInArrayDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { _foo, bar } = { _foo: 1, bar: 2 }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { _foo = 1 } = obj",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { bar: _foo = 1 } = obj",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo: _foo, bar } = { foo: 1, bar: 2 }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo, ..._rest} = { foo: 1, bar: 2, baz: 3 }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo: [_bar, { a: _a, b } ] } = { foo: [1, { a: 'a', b: 'b' }] }",
            Some(
                serde_json::json!([ { "allowInArrayDestructuring": false, "allowInObjectDestructuring": false, }, ]),
            ),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo: [_bar, { a: _a, b } ] } = { foo: [1, { a: 'a', b: 'b' }] }",
            Some(
                serde_json::json!([ { "allowInArrayDestructuring": true, "allowInObjectDestructuring": false, }, ]),
            ),
        ), // { "ecmaVersion": 2022 },
        (
            "const [{ foo: [_bar, _, { bar: _baz }] }] = [{ foo: [1, 2, { bar: 'a' }] }]",
            Some(
                serde_json::json!([ { "allowInArrayDestructuring": false, "allowInObjectDestructuring": false, }, ]),
            ),
        ), // { "ecmaVersion": 2022 },
        (
            "const { foo, bar: { baz, _qux } } = { foo: 1, bar: { baz: 3, _qux: 4 } }",
            Some(serde_json::json!([{ "allowInObjectDestructuring": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class foo { #_bar() {} }", Some(serde_json::json!([{ "enforceInMethodNames": true }]))), // { "ecmaVersion": 2022 },
        ("class foo { #bar_() {} }", Some(serde_json::json!([{ "enforceInMethodNames": true }]))), // { "ecmaVersion": 2022 },
        ("class foo { _field; }", Some(serde_json::json!([{ "enforceInClassFields": true }]))), // { "ecmaVersion": 2022 },
        ("class foo { #_field; }", Some(serde_json::json!([{ "enforceInClassFields": true }]))), // { "ecmaVersion": 2022 },
        ("class foo { field_; }", Some(serde_json::json!([{ "enforceInClassFields": true }]))), // { "ecmaVersion": 2022 },
        ("class foo { #field_; }", Some(serde_json::json!([{ "enforceInClassFields": true }]))), // { "ecmaVersion": 2022 },
        ("var __filename = 1;", None),
        ("class Foo { #_x; foo() { this.#_x; } }", None),
    ];

    Tester::new(NoUnderscoreDangle::NAME, NoUnderscoreDangle::PLUGIN, pass, fail)
        .test_and_snapshot();
}
