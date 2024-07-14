use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_inside_promise, is_promise_callback},
    AstNode,
};

fn no_callback_in_promise_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-promise(no-callback-in-promise): Avoid calling back inside of a promise",
    )
    .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoCallbackInPromise(Box<NoCallbackInPromiseConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoCallbackInPromiseConfig {
    exceptions: Vec<String>,
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
    /// Disallow calling cb() inside of a then() (use nodeify instead).
    ///
    /// ### Why is this bad?
    ///
    /// As a general rule, callbacks should never be directly invoked inside a
    /// Promise.prototype.then() or Promise.prototype.catch() method. That's because your callback
    /// may be unintentionally be invoked twice. It also can be confusing to mix paradigms.
    ///
    /// ### Example
    /// ```javascript
    /// Promise.resolve()
    /// .then(() => callback(null, 'data'))
    /// .catch((err) => callback(err.message, null))
    /// ```
    NoCallbackInPromise,
    style,
);

pub const CALLBACK_NAMES: phf::Set<&'static str> = phf_set! {
    "done",
    "cb",
    "callback",
    "next",
};

impl Rule for NoCallbackInPromise {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self(Box::new(NoCallbackInPromiseConfig {
            exceptions: obj
                .and_then(|v| v.get("exceptions"))
                .and_then(serde_json::Value::as_array)
                .unwrap_or(&vec![])
                .iter()
                .map(serde_json::Value::as_str)
                .filter(Option::is_some)
                .map(|x| x.unwrap().into())
                .collect::<Vec<String>>(),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.callee_name().is_some_and(|name| {
            CALLBACK_NAMES.contains(name) && !self.exceptions.contains(&name.to_string())
        }) {
            if ctx
                .nodes()
                .ancestors(node.id())
                .any(|node_id| is_inside_callback(ctx.nodes().get_node(node_id), ctx))
            {
                ctx.diagnostic(no_callback_in_promise_diagnostic(call_expr.span));
            }
        } else if is_promise_callback(call_expr) {
            if call_expr.arguments.is_empty() {
                return;
            }

            let Some(Expression::Identifier(first_arg)) = &call_expr.arguments[0].as_expression()
            else {
                return;
            };

            if !self.exceptions.contains(&first_arg.name.to_string())
                && CALLBACK_NAMES.contains(first_arg.name.as_ref())
            {
                ctx.diagnostic(no_callback_in_promise_diagnostic(first_arg.span));
            }
        }
    }
}

fn is_inside_callback<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !matches!(node.kind(), AstKind::ArrowFunctionExpression(_) | AstKind::Function(_)) {
        return false;
    }

    is_inside_promise(node, ctx)
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
