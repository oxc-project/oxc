use oxc_ast::{
    ast::{BindingPatternKind, FormalParameter},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_promise, context::LintContext, rule::Rule, AstNode};

fn no_promise_in_callback_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-promise(no-promise-in-callback): Avoid using promises inside of callbacks",
    )
    .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoPromiseInCallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using promises inside of callbacks.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// a(function(err) { doThing().then(a) })
    /// function x(err) { Promise.all() }
    /// ```
    NoPromiseInCallback,
    style,
);

fn is_inside_callback<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !matches!(node.kind(), AstKind::ArrowFunctionExpression(_) | AstKind::Function(_)) {
        return false;
    }

    // is callback with "err" or "error" as first parameter
    if !is_callback(node) {
        return false;
    }

    // Don't warn about valid chained promises
    if is_inside_promise(node, ctx) {
        return false;
    }

    true
}

impl Rule for NoPromiseInCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_promise(call_expr) {
            return;
        }

        // Don't warn about returning promises, it's likely not really a callback function.
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        if matches!(parent.kind(), AstKind::ReturnStatement(_)) {
            return;
        }

        if ctx
            .nodes()
            .ancestors(node.id())
            .any(|node_id| is_inside_callback(ctx.nodes().get_node(node_id), ctx))
        {
            ctx.diagnostic(no_promise_in_callback_diagnostic(call_expr.span));
        }
    }
}

fn is_inside_promise<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // kind: Argument(FunctionExpression)
    let Some(func_argument_expr) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };

    let Some(call_expr_node) = ctx.nodes().parent_node(func_argument_expr.id()) else {
        return false;
    };

    let AstKind::CallExpression(call_expr) = call_expr_node.kind() else {
        return false;
    };

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    let Some(prop_name) = member_expr.static_property_name() else {
        return false;
    };

    if !matches!(prop_name, "catch" | "then") {
        return false;
    }

    true
}

fn get_first_parameter<'a>(node: &AstNode<'a>) -> Option<&'a FormalParameter<'a>> {
    return match node.kind() {
        AstKind::Function(func) => func.params.items.first(),
        AstKind::ArrowFunctionExpression(arrow) => arrow.params.items.first(),
        _ => None,
    };
}

fn is_callback(node: &AstNode) -> bool {
    let Some(first_param) = get_first_parameter(node) else { return false };

    let BindingPatternKind::BindingIdentifier(first_param_ident) = &first_param.pattern.kind else {
        return false;
    };

    if !matches!(first_param_ident.name.as_str(), "error" | "err") {
        return false;
    }

    true
}

#[test]
fn test() {
    use crate::tester::Tester;

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

    Tester::new(NoPromiseInCallback::NAME, pass, fail).test_and_snapshot();
}
