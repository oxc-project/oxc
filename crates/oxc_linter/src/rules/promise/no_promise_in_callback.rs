use oxc_ast::{ast::FormalParameters, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, utils::is_promise, AstNode};

fn no_promise_in_callback_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using promises inside of callbacks.")
        .with_help("Use either promises or callbacks exclusively for handling asynchronous code.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoPromiseInCallback;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows the use of Promises within error-first callback functions.
    ///
    /// ### Why is this bad?
    /// Mixing Promises and callbacks can lead to unclear and inconsistent code.
    /// Promises and callbacks are different patterns for handling asynchronous code.
    /// Mixing them makes the logic flow harder to follow and complicates error handling,
    /// as callbacks rely on an error-first pattern, while Promises use `catch`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// doSomething((err, val) => {
    ///   if (err) console.error(err)
    ///   else doSomethingElse(val).then(console.log)
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// promisify(doSomething)()
    ///   .then(doSomethingElse)
    ///   .then(console.log)
    ///   .catch(console.error)
    /// ```
    NoPromiseInCallback,
    promise,
    suspicious,
);

impl Rule for NoPromiseInCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if is_promise(call_expr).is_none() {
            return;
        }

        // When a Promise is returned in a ReturnStatement, the function is most likely
        // being used as part of a Promise chain rather than as a callback function.
        // To avoid false positives, this case is intentionally excluded from the scope of this rule.
        if let Some(AstKind::ReturnStatement(_)) = ctx.nodes().parent_kind(node.id()) {
            return;
        };

        let mut ancestors = ctx.nodes().ancestors(node.id());
        if ancestors.any(|node| is_callback_function(node, ctx)) {
            ctx.diagnostic(no_promise_in_callback_diagnostic(call_expr.callee.span()));
        }
    }
}

fn is_callback_function<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !node.kind().is_function_like() {
        return false;
    }

    if is_within_promise_handler(node, ctx) {
        return false;
    }

    match node.kind() {
        AstKind::Function(function) => is_error_first_callback(&function.params),
        AstKind::ArrowFunctionExpression(arrow_function) => {
            is_error_first_callback(&arrow_function.params)
        }
        _ => false,
    }
}

fn is_error_first_callback(params: &FormalParameters) -> bool {
    let Some(first_parameter) = params.items.first() else {
        return false;
    };

    let Some(ident) = first_parameter.pattern.get_binding_identifier() else {
        return false;
    };

    matches!(ident.name.as_str(), "err" | "error")
}

fn is_within_promise_handler<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
        return false;
    }

    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };
    if !matches!(ctx.nodes().kind(parent.id()), AstKind::Argument(_)) {
        return false;
    };

    let Some(AstKind::CallExpression(call_expr)) = ctx.nodes().parent_kind(parent.id()) else {
        return false;
    };

    matches!(call_expr.callee_name(), Some("then" | "catch"))
}

#[test]
fn test() {
    use crate::tester::Tester;

    // The following test cases are based on the original
    // implementation from eslint-plugin-promise and are licensed under the ISC License.
    //
    // Copyright (c) 2020, Jamund Ferguson
    // https://github.com/eslint-community/eslint-plugin-promise/blob/266ddbb03076c05c362a6daecb9382b80cdd7108/__tests__/no-promise-in-callback.js
    let pass = vec![
        "go(function() { return Promise.resolve(4) })",
        "go(function() { return a.then(b) })",
        "go(function() { b.catch(c) })",
        "go(function() { b.then(c, d) })",
        "go(() => Promise.resolve(4))",
        "go((errrr) => a.then(b))",
        "go((helpers) => { b.catch(c) })",
        "go((e) => { b.then(c, d) })",
        "a.catch((err) => { b.then(c, d) })",
        "var x = function() { return Promise.resolve(4) }",
        "function y() { return Promise.resolve(4) }",
        "function then() { return Promise.reject() }",
        "doThing(function(x) { return Promise.reject(x) })",
        "doThing().then(function() { return Promise.all([a,b,c]) })",
        "doThing().then(function() { return Promise.resolve(4) })",
        "doThing().then(() => Promise.resolve(4))",
        "doThing().then(() => Promise.all([a]))",
        "a(function(err) { return doThing().then(a) })",
    ];

    let fail = vec![
        "a(function(err) { doThing().then(a) })",
        "a(function(error, zup, supa) { doThing().then(a) })",
        "a(function(error) { doThing().then(a) })",
        "a((error) => { doThing().then(a) })",
        "a((error) => doThing().then(a))",
        "a((err, data) => { doThing().then(a) })",
        "a((err, data) => doThing().then(a))",
        "function x(err) { Promise.all() }",
        "function x(err) { Promise.allSettled() }",
        "function x(err) { Promise.any() }",
        "let x = (err) => doThingWith(err).then(a)",
    ];

    Tester::new(NoPromiseInCallback::NAME, NoPromiseInCallback::PLUGIN, pass, fail)
        .test_and_snapshot();
}
