use oxc_ast::{
    ast::{CallExpression, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_callback_in_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid calling back inside of a promise")
        .with_help("Use `then` and `catch` directly")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCallbackInPromise(Box<NoCallbackInPromiseConfig>);

#[derive(Debug, Clone)]
pub struct NoCallbackInPromiseConfig {
    callbacks: Vec<CompactStr>,
}

impl Default for NoCallbackInPromiseConfig {
    fn default() -> Self {
        Self { callbacks: vec!["callback".into(), "cb".into(), "done".into(), "next".into()] }
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
    correctness,
);

impl Rule for NoCallbackInPromise {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut default_config = NoCallbackInPromiseConfig::default();

        let exceptions: Vec<String> = value
            .get(0)
            .and_then(|v| v.get("exceptions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter().filter_map(serde_json::Value::as_str).map(ToString::to_string).collect()
            })
            .unwrap_or_default();

        default_config.callbacks.retain(|item| !exceptions.contains(&item.to_string()));

        Self(Box::new(default_config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(call_expr) = node.kind().as_call_expression() else {
            return;
        };

        let is_not_callback = call_expr
            .callee
            .get_identifier_reference()
            .map_or(true, |id| self.callbacks.binary_search(&id.name.as_str().into()).is_err());

        if is_not_callback {
            if Self::has_promise_callback(call_expr) {
                let Some(id) = call_expr.arguments.first().and_then(|arg| {
                    arg.as_expression().and_then(Expression::get_identifier_reference)
                }) else {
                    return;
                };

                let name = id.name.as_str();
                if self.callbacks.binary_search(&name.into()).is_ok() {
                    ctx.diagnostic(no_callback_in_promise_diagnostic(id.span));
                }
            }
        } else if ctx
            .nodes()
            .iter_parents(node.id())
            .skip(1)
            .any(|node| Self::is_inside_promise(node, ctx))
        {
            ctx.diagnostic(no_callback_in_promise_diagnostic(node.span()));
        }
    }
}

impl NoCallbackInPromise {
    fn is_inside_promise(node: &AstNode, ctx: &LintContext) -> bool {
        if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
            || !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::Argument(_)))
        {
            return false;
        }

        ctx.nodes().iter_parents(node.id()).nth(2).is_some_and(|node| {
            node.kind().as_call_expression().is_some_and(Self::has_promise_callback)
        })
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
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function thing(cb) { cb() }", None),
        ("doSomething(function(err) { cb(err) })", None),
        ("function thing(callback) { callback() }", None),
        ("doSomething(function(err) { callback(err) })", None),
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
    ];

    Tester::new(NoCallbackInPromise::NAME, pass, fail).test_and_snapshot();
}
