use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_restricted_globals(global_name: &str, suffix: &str, span: Span) -> OxcDiagnostic {
    let warn_text = if suffix.is_empty() {
        format!("Unexpected use of '{global_name}'.")
    } else {
        format!("Unexpected use of '{global_name}'. {suffix}")
    };

    OxcDiagnostic::warn(warn_text).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedGlobals {
    restricted_globals: Box<FxHashMap<String, String>>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule allows you to specify global variable names that you don't want to use in your application.
    ///
    /// ### Why is this bad?
    ///
    /// Disallowing usage of specific global variables can be useful if you want to allow a set of global
    /// variables by enabling an environment, but still want to disallow some of those.
    ///
    /// For instance, early Internet Explorer versions exposed the current DOM event as a global variable
    /// `event`, but using this variable has been considered as a bad practice for a long time. Restricting
    /// this will make sure this variable isn't used in browser code.
    ///
    /// ### Example
    ///
    /// If we have options:
    ///
    /// ```json
    /// "no-restricted-globals": ["error", "event"]
    /// ```
    ///
    /// The following patterns are considered problems:
    ///
    /// ```javascript
    /// function onClick() {
    ///    console.log(event);    // Unexpected global variable 'event'. Use local parameter instead.
    /// }
    /// ```
    NoRestrictedGlobals,
    eslint,
    restriction,
);

impl Rule for NoRestrictedGlobals {
    fn from_configuration(value: serde_json::Value) -> Self {
        let list = match value {
            Value::Array(arr) => arr.iter().fold(FxHashMap::default(), |mut acc, v| match v {
                // "no-restricted-globals": ["error", "event"]
                Value::String(name) => {
                    acc.insert(name.to_string(), String::new());
                    acc
                }
                // "no-restricted-globals": ["error", { "name": "event", "message": "Use local parameter instead." }]
                Value::Object(obj) => {
                    let name = obj.get("name").and_then(Value::as_str).unwrap_or_default();
                    let message = obj.get("message").and_then(Value::as_str).unwrap_or_default();
                    acc.insert(name.to_string(), message.to_string());
                    acc
                }
                _ => acc,
            }),
            _ => FxHashMap::default(),
        };

        Self { restricted_globals: Box::new(list) }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::IdentifierReference(ident) = node.kind() {
            let Some(message) = self.restricted_globals.get(ident.name.as_str()) else {
                return;
            };

            if ctx.semantic().is_reference_to_global_variable(ident) {
                ctx.diagnostic(no_restricted_globals(&ident.name, message, ident.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    const CUSTOM_MESSAGE: &str = "Use bar instead.";

    let pass = vec![
        ("foo", None),
        ("foo", Some(serde_json::json!(["bar"]))),
        ("var foo = 1;", Some(serde_json::json!(["foo"]))),
        ("event", Some(serde_json::json!(["bar"]))),
        ("import foo from 'bar';", Some(serde_json::json!(["foo"]))),
        ("function foo() {}", Some(serde_json::json!(["foo"]))),
        ("function fn() { var foo; }", Some(serde_json::json!(["foo"]))),
        ("foo.bar", Some(serde_json::json!(["bar"]))),
        ("foo", Some(serde_json::json!([{ "name": "bar", "message": "Use baz instead." }]))),
    ];

    let fail = vec![
        ("foo", Some(serde_json::json!(["foo"]))),
        ("function fn() { foo; }", Some(serde_json::json!(["foo"]))),
        ("function fn() { foo; }", Some(serde_json::json!(["foo"]))),
        ("event", Some(serde_json::json!(["foo", "event"]))),
        ("foo", Some(serde_json::json!(["foo"]))),
        ("foo()", Some(serde_json::json!(["foo"]))),
        ("foo.bar()", Some(serde_json::json!(["foo"]))),
        ("foo", Some(serde_json::json!([{ "name": "foo" }]))),
        ("function fn() { foo; }", Some(serde_json::json!([{ "name": "foo" }]))),
        ("function fn() { foo; }", Some(serde_json::json!([{ "name": "foo" }]))),
        ("event", Some(serde_json::json!(["foo", { "name": "event" }]))),
        ("foo", Some(serde_json::json!([{ "name": "foo" }]))),
        ("foo()", Some(serde_json::json!([{ "name": "foo" }]))),
        ("foo.bar()", Some(serde_json::json!([{ "name": "foo" }]))),
        ("foo", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }]))),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
        ),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
        ),
        (
            "event",
            Some(
                serde_json::json!(["foo", { "name": "event", "message": "Use local event parameter." }]),
            ),
        ),
        ("foo", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }]))),
        ("foo()", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }]))),
        ("foo.bar()", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }]))),
        (
            "var foo = obj => hasOwnProperty(obj, 'name');",
            Some(serde_json::json!(["hasOwnProperty"])),
        ),
    ];

    Tester::new(NoRestrictedGlobals::NAME, NoRestrictedGlobals::PLUGIN, pass, fail)
        .test_and_snapshot();
}
