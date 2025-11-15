use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_callback_in_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid calling back inside of a promise")
        .with_help("Use `then` and `catch` directly")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCallbackInPromise(Box<NoCallbackInPromiseConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoCallbackInPromiseConfig {
    /// List of callback function names to check for within Promise `then` and `catch` methods.
    callbacks: Vec<CompactStr>,
    /// List of callback function names to allow within Promise `then` and `catch` methods.
    exceptions: Vec<CompactStr>,
    /// Boolean as to whether callbacks in timeout functions like `setTimeout` will err.
    timeouts_err: bool,
}

impl Default for NoCallbackInPromiseConfig {
    fn default() -> Self {
        Self {
            callbacks: vec!["callback".into(), "cb".into(), "done".into(), "next".into()],
            exceptions: Vec::new(),
            timeouts_err: false,
        }
    }
}

impl std::ops::Deref for NoCallbackInPromise {
    type Target = NoCallbackInPromiseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows calling a callback function (`cb()`) inside a `Promise.prototype.then()`
    /// or `Promise.prototype.catch()`.
    ///
    /// ### Why is this bad?
    ///
    /// Directly invoking a callback inside a `then()` or `catch()` method can lead to
    /// unexpected behavior, such as the callback being called multiple times. Additionally,
    /// mixing the callback and promise paradigms in this way can make the code confusing
    /// and harder to maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function callback(err, data) {
    ///     console.log('Callback got called with:', err, data)
    ///     throw new Error('My error')
    ///   }
    ///
    /// Promise.resolve()
    ///   .then(() => callback(null, 'data'))
    ///   .catch((err) => callback(err.message, null))
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Promise.resolve()
    ///   .then((data) => { console.log(data) })
    ///   .catch((err) => { console.error(err) })
    /// ```
    NoCallbackInPromise,
    promise,
    correctness,
    config = NoCallbackInPromiseConfig,
);

const TIMEOUT_WHITELIST: [&str; 4] =
    ["setImmediate", "setTimeout", "requestAnimationFrame", "nextTick"];

impl Rule for NoCallbackInPromise {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config: NoCallbackInPromiseConfig = value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        config.callbacks.retain(|item| !config.exceptions.contains(item));
        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(call_expr) = node.kind().as_call_expression() else {
            return;
        };

        let is_not_callback = call_expr
            .callee
            .get_identifier_reference()
            .is_none_or(|id| self.callbacks.binary_search(&id.name.as_str().into()).is_err());

        if is_not_callback {
            let Some(id) = call_expr
                .arguments
                .first()
                .and_then(|arg| arg.as_expression().and_then(Expression::get_identifier_reference))
            else {
                return;
            };
            let name = id.name.as_str();
            if self.callbacks.binary_search(&name.into()).is_err() {
                return;
            }
            if Self::has_promise_callback(call_expr) {
                ctx.diagnostic(no_callback_in_promise_diagnostic(id.span));
                return;
            } else if !self.timeouts_err && Self::is_inside_timeout(node) {
                return;
            }
        }
        let ancestors = ctx.nodes().ancestors(node.id());
        for ancestor in ancestors {
            if !self.timeouts_err && Self::is_inside_timeout(ancestor) {
                break;
            }
            if Self::is_inside_promise(ancestor, ctx) {
                ctx.diagnostic(no_callback_in_promise_diagnostic(node.span()));
                break;
            }
        }
    }
}

impl NoCallbackInPromise {
    fn is_inside_promise(node: &AstNode, ctx: &LintContext) -> bool {
        if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
            return false;
        }

        // Check if the parent is a CallExpression with then/catch
        let parent = ctx.nodes().parent_node(node.id());
        if let Some(call_expr) = parent.kind().as_call_expression() {
            // Check if this function is one of the arguments
            let is_argument = call_expr
                .arguments
                .iter()
                .any(|arg| matches!(arg.as_expression(), Some(expr) if expr.span() == node.span()));

            return is_argument && Self::has_promise_callback(call_expr);
        }

        false
    }

    fn has_promise_callback(call_expr: &CallExpression) -> bool {
        matches!(
            call_expr
                .callee
                .as_member_expression()
                .and_then(MemberExpression::static_property_name),
            Some("then" | "catch")
        )
    }

    fn is_inside_timeout(node: &AstNode) -> bool {
        let Some(call_expr) = node.kind().as_call_expression() else {
            return false;
        };
        match &call_expr.callee {
            Expression::Identifier(ident) => TIMEOUT_WHITELIST.contains(&ident.name.as_str()),
            Expression::StaticMemberExpression(static_member_expr) => {
                TIMEOUT_WHITELIST.contains(&static_member_expr.property.name.as_str())
            }
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function thing(cb) { cb() }", None),
        ("doSomething(function(err) { cb(err) })", None),
        ("function thing(callback) { callback() }", None),
        ("doSomething(function(err) { callback(err) })", None),
        ("a.then(doSomething)", None),
        ("a.then(() => doSomething())", None),
        ("a.then(function(err) { doSomething(err) })", None),
        ("a.then(function(data) { doSomething(data) }, function(err) { doSomething(err) })", None),
        ("a.catch(function(err) { doSomething(err) })", None),
        ("whatever.then((err) => { process.nextTick(() => cb()) })", None),
        ("whatever.then((err) => { setImmediate(() => cb()) })", None),
        ("whatever.then((err) => setImmediate(() => cb()))", None),
        ("whatever.then((err) => process.nextTick(() => cb()))", None),
        ("whatever.then((err) => process.nextTick(cb))", None),
        ("whatever.then((err) => setImmediate(cb))", None),
        ("let thing = (cb) => cb()", None),
        ("doSomething(err => cb(err))", None),
        ("a.then(() => next())", Some(serde_json::json!([{ "exceptions": ["next"] }]))),
        (
            "a.then(() => next()).catch((err) => next(err))",
            Some(serde_json::json!([{ "exceptions": ["next"] }])),
        ),
        ("a.then(next)", Some(serde_json::json!([{ "exceptions": ["next"] }]))),
        ("a.then(next).catch(next)", Some(serde_json::json!([{ "exceptions": ["next"] }]))),
    ];

    let fail = vec![
        ("a.then(cb)", None),
        ("a.then(() => cb())", None),
        ("a.then(function(err) { cb(err) })", None),
        ("a.then(function(data) { cb(data) }, function(err) { cb(err) })", None),
        ("a.catch(function(err) { cb(err) })", None),
        ("a.then(callback)", None),
        ("a.then(() => callback())", None),
        ("a.then(function(err) { callback(err) })", None),
        ("a.then(function(data) { callback(data) }, function(err) { callback(err) })", None),
        ("a.catch(function(err) { callback(err) })", None),
        ("a.then(() => doSomething(cb))", None),
        (
            "function wait (callback) {
    return Promise.resolve()
        .then(() => {
            setTimeout(callback);
        });
}",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "function wait (callback) {
    return Promise.resolve()
        .then(() => {
            setTimeout(() => callback());
        });
}",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "whatever.then((err) => { process.nextTick(() => cb()) })",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "whatever.then((err) => { setImmediate(() => cb()) })",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "whatever.then((err) => setImmediate(() => cb()))",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "whatever.then((err) => process.nextTick(() => cb()))",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "whatever.then((err) => process.nextTick(cb))",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
        (
            "whatever.then((err) => setImmediate(cb))",
            Some(serde_json::json!([{ "timeoutsErr": true }])),
        ),
    ];

    Tester::new(NoCallbackInPromise::NAME, NoCallbackInPromise::PLUGIN, pass, fail)
        .test_and_snapshot();
}
