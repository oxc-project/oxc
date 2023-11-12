use oxc_ast::{
    ast::{Argument, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum NoUselessPromiseResolveRejectDiagnostic {
    #[error("eslint-plugin-unicorn(no-useless-promise-resolve-reject): Prefer `{1} value` over `{1} Promise.resolve(value)`.")]
    #[diagnostic(severity(warning), help(""))]
    Resolve(#[label] Span, String),
    #[error("eslint-plugin-unicorn(no-useless-promise-resolve-reject): Prefer `throw error` over `{1} Promise.reject(error)`.")]
    #[diagnostic(severity(warning), help(""))]
    Reject(#[label] Span, String),
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessPromiseResolveReject;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessPromiseResolveReject,
    correctness
);

impl Rule for NoUselessPromiseResolveReject {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        dbg!("1");
        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };
        dbg!("2");

        if !member_expr.object().is_specific_id("Promise") {
            return;
        }
        dbg!("3");

        let MemberExpression::StaticMemberExpression(static_member_expr) = member_expr else {
            return;
        };
        dbg!("4");

        if !matches!(static_member_expr.property.name.as_str(), "resolve" | "reject") {
            return;
        }
        dbg!("5");

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        dbg!("6", parent.kind());
        match parent.kind() {
            AstKind::ExpressionStatement(expr_stmt) => {}
            AstKind::ArrowExpression(arrow_expr) => {}
            AstKind::ReturnStatement(return_stmt) => {}
            AstKind::YieldExpression(yield_expr) => {}
            AstKind::ParenthesizedExpression(paren_expr) => {}
            _ => return,
        }
        dbg!("7");

        let Some((is_async, function_node)) = chchc(node, ctx) else { return };
        dbg!("7", is_async);

        dbg!(function_node.kind());

        if !(is_async || is_promise_callback(function_node, ctx)) {
            return;
        }

        ctx.diagnostic(NoUselessPromiseResolveRejectDiagnostic::Reject(
            node.kind().span(),
            "".to_string(),
        ));
    }
}

fn chchc<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> Option<(bool, &'a AstNode<'b>)> {
    let mut parent = node;

    let fnx = loop {
        if let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) {
            parent = grand_parent;
            if parent.kind().is_function_like() {
                break parent;
            }
        } else {
            return None;
        }
    };

    match fnx.kind() {
        AstKind::ArrowExpression(arrow_expr) => Some((arrow_expr.r#async, parent)),
        AstKind::Function(func) => Some((func.r#async, parent)),
        _ => None,
    }
}

fn is_promise_callback<'a, 'b>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>) -> bool {
    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };

    dbg!(parent.kind());
    let AstKind::Argument(Argument::Expression(expr)) = parent.kind() else { return false };
    dbg!("is_promise_callback 2", expr);
    let Some(parent) = ctx.nodes().parent_node(parent.id()) else {
        return false;
    };
    dbg!("is_promise_callback 2", parent);
    let AstKind::CallExpression(call_expr) = parent.kind() else { return false };

    dbg!("is_promise_callback 3",);
    let Some(member_expr) = call_expr.callee.get_member_expr() else { return false };
    dbg!("is_promise_callback 4",);

    if member_expr.is_computed() {
        return false;
    }
    dbg!("is_promise_callback 5",);
    let Some(static_prop_name) = member_expr.static_property_name() else { return false };
    dbg!("is_promise_callback 6",);

    if call_expr.arguments.len() == 1 && matches!(static_prop_name, "then" | "catch" | "finally") {
        return true;
    }

    dbg!(static_prop_name);

    if call_expr.arguments.len() == 2 && matches!(static_prop_name, "then") {
        if call_expr.arguments[0].is_spread() {
            return false;
        }
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"async () => bar;"#,
        r#"promise.then(() => foo).catch(() => bar).finally(() => baz)"#,
        r#"promise.then(() => foo, () => bar).finally(() => baz)"#,
        r#"promise.then(x, y, () => Promise.resolve(foo))"#,
        r#"promise.catch(x, () => Promise.resolve(foo))"#,
        r#"promise.finally(x, () => Promise.resolve(foo))"#,
        r#"promise[then](() => Promise.resolve(foo))"#,
    ];

    let fail = vec![
        r#"async () => Promise.resolve(bar);"#,
        r#"async () => Promise.reject(bar);"#,
        r#"async () => Promise.resolve();"#,
        r#"async () => Promise.reject();"#,
        r#"async () => Promise.resolve(bar, baz);"#,
        r#"async () => Promise.reject(bar, baz);"#,
        r#"async () => Promise.resolve((bar, baz))"#,
        r#"async () => Promise.resolve({})"#,
        r#"async () => Promise.resolve(...bar);"#,
        r#"async () => Promise.reject(...bar);"#,
        r#"async () => (Promise.resolve(bar));"#,
        r#"promise.then(() => {}, () => Promise.resolve(bar))"#,
        r#"promise.then(() => Promise.resolve(bar), () => Promise.resolve(baz))"#,
        r#"promise.then(() => {}, () => { return Promise.resolve(bar); })"#,
        r#"promise.then(() => {}, async () => Promise.reject(bar))"#,
        r#"promise.then(() => {}, async () => { return Promise.reject(bar); })"#,
    ];

    Tester::new_without_config(NoUselessPromiseResolveReject::NAME, pass, fail).test_and_snapshot();
}
