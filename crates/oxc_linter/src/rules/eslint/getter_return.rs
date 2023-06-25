use oxc_ast::{
    ast::{
        Argument, CallExpression, ChainElement, Expression, FunctionBody, MemberExpression,
        MethodDefinitionKind, ObjectExpression, ObjectPropertyKind, PropertyKind,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use super::array_callback_return::return_checker;
use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(getter-return): Expected to always return a value in getter.")]
#[diagnostic(severity(warning), help("Return a value from all code paths in getter."))]
struct GetterReturnDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct GetterReturn {
    pub allow_implicit: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Requires all getters to have a return statement
    ///
    /// ### Why is this bad?
    /// Getters should always return a value. If they don't, it's probably a mistake.
    ///
    /// ### Example
    /// ```javascript
    /// class Person{
    ///     get name(){
    ///         // no return
    ///     }
    /// }
    /// ```
    GetterReturn,
    correctness
);

impl GetterReturn {
    fn is_correct_getter(&self, function_body: &FunctionBody) -> bool {
        // Filter on target methods on Arrays
        let return_status = return_checker::check_function_body(function_body);

        if self.allow_implicit {
            return_status.must_return()
        } else {
            return_status == return_checker::StatementReturnStatus::AlwaysExplicit
        }
    }

    fn check_object_descriptor(&self, object: &ObjectExpression) -> Option<Span> {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else { continue };
            if !property.key.static_name().is_some_and(|name| name == "get") {
                continue;
            }

            match &property.value {
                Expression::FunctionExpression(function) => {
                    let Some(body) = &function.body else { continue };
                    if !self.is_correct_getter(body) {
                        let span = Span::new(property.key.span().start, function.params.span.start);
                        return Some(span);
                    }
                }
                Expression::ArrowExpression(arrow) if !arrow.expression => {
                    if !self.is_correct_getter(&arrow.body) {
                        let span = Span::new(property.key.span().start, arrow.params.span.start);
                        return Some(span);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn check_property(&self, call: &CallExpression) -> Option<Span> {
        let Some(Argument::Expression(Expression::ObjectExpression(object))) =
            call.arguments.get(2) else { return None };

        self.check_object_descriptor(object)
    }

    fn check_properties(&self, call: &CallExpression) -> Option<Vec<Span>> {
        let Some(Argument::Expression(Expression::ObjectExpression(object))) =
            call.arguments.get(1) else { return None };

        let error_spans = object
            .properties
            .iter()
            .filter_map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else { return None };
                let Expression::ObjectExpression(object) = &property.value else { return None };
                self.check_object_descriptor(object)
            })
            .collect();

        Some(error_spans)
    }
}

impl Rule for GetterReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::MethodDefinition(method) if method.kind == MethodDefinitionKind::Get => {
                let Some(body) = &method.value.body else { return };

                if self.is_correct_getter(body) {
                    return;
                }

                let span = Span::new(method.span.start, method.key.span().end);
                ctx.diagnostic(GetterReturnDiagnostic(span));
            }
            AstKind::ObjectProperty(property) if property.kind == PropertyKind::Get => {
                let Expression::FunctionExpression(function) = &property.value else { return };
                let Some(body) = &function.body else { return };

                if self.is_correct_getter(body) {
                    return;
                }

                let span = Span::new(property.span.start, property.key.span().end);
                ctx.diagnostic(GetterReturnDiagnostic(span));
            }
            AstKind::CallExpression(call) => {
                let member = match call.callee.get_inner_expression() {
                    Expression::ChainExpression(chain) => {
                        let ChainElement::MemberExpression(member) = &chain.expression else { return };
                        member
                    }
                    Expression::MemberExpression(member) => member,
                    _ => return,
                };

                let MemberExpression::StaticMemberExpression(static_member) = &member.0 else { return };

                let Expression::Identifier(object_ident) = &static_member.object.get_inner_expression() else { return };

                match (object_ident.name.as_str(), static_member.property.name.as_str()) {
                    ("Object" | "Reflect", "defineProperty") => {
                        if let Some(span) = self.check_property(call) {
                            ctx.diagnostic(GetterReturnDiagnostic(span));
                        }
                    }
                    ("Object", "create" | "defineProperties") => {
                        let Some(spans) = self.check_properties(call) else { return };
                        for span in spans {
                            ctx.diagnostic(GetterReturnDiagnostic(span));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_implicit = value
            .get(0)
            .and_then(|config| config.get("allowImplicit"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { allow_implicit }
    }
}

#[allow(clippy::too_many_lines)]
#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        ("var foo = { get bar(){return true;} };", None),
        (
            "var foo = { get bar() {return;} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "var foo = { get bar(){return true;} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "var foo = { get bar(){if(bar) {return;} return true;} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){return true;} }", None),
        ("class foo { get bar(){if(baz){return true;} else {return false;} } }", None),
        ("class foo { get(){return true;} }", None),
        (
            "class foo { get bar(){return true;} }",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){return;} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});",
            None,
        ),
        ("Object.defineProperties(foo, { bar: { get: function () {return true;}} });", None),
        (
            "Object.defineProperties(foo, { bar: { get: function () { ~function (){ return true; }(); return true;}} });",
            None,
        ),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});",
            None,
        ),
        ("Object.create(foo, { bar: { get() {return true;} } });", None),
        ("Object.create(foo, { bar: { get: function () {return true;} } });", None),
        ("Object.create(foo, { bar: { get: () => {return true;} } });", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function () {return true;}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){return;}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {return true;}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {return;}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("var get = function(){};", None),
        ("var get = function(){ return true; };", None),
        ("var foo = { bar(){} };", None),
        ("var foo = { bar(){ return true; } };", None),
        ("var foo = { bar: function(){} };", None),
        ("var foo = { bar: function(){return;} };", None),
        ("var foo = { bar: function(){return true;} };", None),
        ("var foo = { get: function () {} }", None),
        ("var foo = { get: () => {}};", None),
        ("class C { get; foo() {} }", None),
        ("foo.defineProperty(null, { get() {} });", None),
        ("foo.defineProperties(null, { bar: { get() {} } });", None),
        ("foo.create(null, { bar: { get() {} } });", None),
    ];

    let fail = vec![
        ("var foo = { get bar() {} };", None),
        ("var foo = { get\n bar () {} };", None),
        ("var foo = { get bar(){if(baz) {return true;}} };", None),
        ("var foo = { get bar() { ~function () {return true;}} };", None),
        ("var foo = { get bar() { return; } };", None),
        ("var foo = { get bar() {} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        (
            "var foo = { get bar() {if (baz) {return;}} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){} }", None),
        ("var foo = class {\n  static get\nbar(){} }", None),
        ("class foo { get bar(){ if (baz) { return true; }}}", None),
        ("class foo { get bar(){ ~function () { return true; }()}}", None),
        ("class foo { get bar(){} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        (
            "class foo { get bar(){if (baz) {return true;} } }",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("Object.defineProperty(foo, 'bar', { get: function (){}});", None),
        ("Object.defineProperty(foo, 'bar', { get: function getfoo (){}});", None),
        ("Object.defineProperty(foo, 'bar', { get(){} });", None),
        ("Object.defineProperty(foo, 'bar', { get: () => {}});", None),
        ("Object.defineProperty(foo, \"bar\", { get: function (){if(bar) {return true;}}});", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){ ~function () { return true; }()}});",
            None,
        ),
        ("Reflect.defineProperty(foo, 'bar', { get: function (){}});", None),
        ("Object.create(foo, { bar: { get: function() {} } })", None),
        ("Object.create(foo, { bar: { get() {} } })", None),
        ("Object.create(foo, { bar: { get: () => {} } })", None),
        (
            "Object.defineProperties(foo, { bar: { get: function () {}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function (){if(bar) {return true;}}}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {~function () { return true; }()}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.create(foo, { bar: { get: function (){} } });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function (){}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("Object?.defineProperty(foo, 'bar', { get: function (){} });", None),
        ("(Object?.defineProperty)(foo, 'bar', { get: function (){} });", None),
        (
            "Object?.defineProperty(foo, 'bar', { get: function (){} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "(Object?.defineProperty)(foo, 'bar', { get: function (){} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "(Object?.create)(foo, { bar: { get: function (){} } });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
    ];

    Tester::new(GetterReturn::NAME, pass, fail).test_and_snapshot();
}
