use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{AstNode, ast_util::variable_declaration_kind, context::LintContext, rule::Rule};

fn require_post_message_target_origin_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing the `targetOrigin` argument.")
        .with_label(span)
        .with_note("https://developer.mozilla.org/en-US/docs/Web/API/Window/postMessage")
}

#[derive(Debug, Default, Clone)]
pub struct RequirePostMessageTargetOrigin;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using the `targetOrigin` argument with `window.postMessage()`.
    ///
    /// Calls on message ports (such as `event.ports[0]` or `channel.port1`) and on
    /// `BroadcastChannel` or `Worker` instances created in place or assigned to a
    /// `const` (such as `const worker = new Worker(url)`) are ignored, as their
    /// `postMessage()` methods do not take a `targetOrigin` argument.
    ///
    /// Note that this rule may still have false positives, as it is not capable of
    /// detecting all cases correctly without type information. As such, it
    /// may not be a good idea to enable in cases where `postMessage()` may
    /// be called on other `BroadcastChannel` or `Worker` instances, or in
    /// worker/service worker contexts (for example, `WorkerGlobalScope#postMessage`,
    /// where the second argument is a transfer list or options object, not `targetOrigin`).
    ///
    /// ### Why is this bad?
    ///
    /// When calling `window.postMessage()` without the `targetOrigin` argument,
    /// the message cannot be received by any window.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// window.postMessage(message);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// window.postMessage(message, 'https://example.com');
    ///
    /// window.postMessage(message, '*');
    ///
    /// const channel = new BroadcastChannel('example');
    /// channel.postMessage(message);
    /// ```
    RequirePostMessageTargetOrigin,
    unicorn,
    suspicious,
    suggestion,
    version = "0.15.15",
    short_description = "Enforce using the `targetOrigin` argument with `window.postMessage()`.",
);

impl Rule for RequirePostMessageTargetOrigin {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        // ignore "foo.postMessage?.(...message)" and "foo.postMessage(...message)"
        if call_expr.arguments.len() != 1
            || call_expr.optional
            || matches!(&call_expr.arguments[0], Argument::SpreadElement(_))
        {
            return;
        }
        let member_expr = match call_expr.callee.get_member_expr() {
            // ignore "foo[postMessage](message)" and optional members with non-identifier objects
            Some(expr)
                if !expr.is_computed()
                    && (!expr.optional()
                        || matches!(
                            expr.object().without_parentheses(),
                            Expression::Identifier(_)
                        )) =>
            {
                expr
            }
            _ => return,
        };
        if member_expr.static_property_name().is_some_and(|name| name == "postMessage") {
            if is_message_port_expression(member_expr.object())
                || is_non_window_instance(member_expr.object(), ctx)
            {
                return;
            }
            let span = call_expr.arguments[0].span();
            ctx.diagnostic_with_suggestion(
                require_post_message_target_origin_diagnostic(Span::new(span.end, span.end)),
                |fixer| {
                    let text = match member_expr.object() {
                        Expression::Identifier(ident) => {
                            format!(", {}.location.origin", ident.name.as_str())
                        }
                        _ => ", self.location.origin".to_string(),
                    };
                    fixer.insert_text_after_range(span, text)
                },
            );
        }
    }
}

fn is_message_port_expression(expr: &Expression<'_>) -> bool {
    let mut current_expr = expr.without_parentheses();
    loop {
        if let Expression::Identifier(ident) = current_expr
            && matches!(ident.name.as_str(), "port" | "port1" | "port2" | "messagePort")
        {
            return true;
        }

        let Some(member_expr) = current_expr.get_member_expr() else {
            return false;
        };

        if member_expr
            .static_property_name()
            .and_then(JSStr::as_str)
            .is_some_and(|name| matches!(name, "port1" | "port2"))
        {
            return true;
        }

        if member_expr.is_computed()
            && member_expr.object().without_parentheses().get_member_expr().is_some_and(
                |object_member| {
                    object_member.static_property_name().is_some_and(|name| name == "ports")
                },
            )
        {
            return true;
        }

        current_expr = member_expr.object().without_parentheses();
    }
}

/// Returns `true` if `expr` is a `BroadcastChannel` or `Worker` instance created in place, or an
/// identifier bound to a `const` initialized with one. Their `postMessage()` methods take no
/// `targetOrigin` argument.
fn is_non_window_instance(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    let expr = expr.get_inner_expression();
    if is_non_window_constructor_call(expr) {
        return true;
    }

    let Expression::Identifier(ident) = expr else {
        return false;
    };
    let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    let AstKind::VariableDeclarator(declarator) = ctx.symbol_declaration(symbol_id).kind() else {
        return false;
    };
    variable_declaration_kind(declarator, ctx.nodes())
        .is_some_and(VariableDeclarationKind::is_const)
        && declarator
            .init
            .as_ref()
            .is_some_and(|init| is_non_window_constructor_call(init.get_inner_expression()))
}

fn is_non_window_constructor_call(expr: &Expression<'_>) -> bool {
    let Expression::NewExpression(new_expr) = expr else {
        return false;
    };
    let Expression::Identifier(callee) = &new_expr.callee else {
        return false;
    };
    matches!(callee.name.as_str(), "BroadcastChannel" | "Worker")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "window.postMessage(message, targetOrigin)",
        "postMessage(message)",
        "window.postMessage",
        "window.postMessage()",
        "window.postMessage(message, targetOrigin, transfer)",
        "window.postMessage(...message)",
        "window[postMessage](message)",
        r#"window["postMessage"](message)"#,
        "window.notPostMessage(message)",
        "window.postMessage?.(message)",
        "window?.[postMessage](message)",
        r"window?.['postMessage'](message)",
        "window.c?.postMessage(message)",
        "window.c.postMessage?.(message)",
        "window.a.b?.postMessage(message)",
        "window?.a?.b?.postMessage(message)",
        "event.ports[0].postMessage(message)",
        "channel.port1.postMessage(message)",
        "channel['port2'].postMessage(message)",
        "event?.ports[0].postMessage(message)",
        // https://github.com/oxc-project/oxc/issues/28596
        "const channel = new BroadcastChannel('example'); channel.postMessage({ type: 'ping' });",
        "const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' }); worker.postMessage({ type: 'ping' });",
        "new BroadcastChannel('example').postMessage(message)",
        "(new Worker(url)).postMessage(message)",
        "const worker = new Worker(url); worker?.postMessage(message)",
        "const worker = new Worker(url) as Worker; (worker as Worker).postMessage(message)",
        "import { Worker } from 'node:worker_threads'; const worker = new Worker(file); worker.postMessage(data);",
        "const channel = new BroadcastChannel('example'); function send() { channel.postMessage(message); }",
    ];

    let fail = vec![
        "window.postMessage(message)",
        "self.postMessage(message)",
        "globalThis.postMessage(message)",
        "foo.postMessage(message )",
        "foo?.postMessage(message )",
        "foo.postMessage( ((message)) )",
        "foo.postMessage(message,)",
        "foo.postMessage(message , )",
        "foo.window.postMessage(message)",
        "document.defaultView.postMessage(message)",
        "getWindow().postMessage(message)",
        "window.postMessage(message,                 /** comments */  )",
        "window.c.postMessage(message)",
        "window?.c.postMessage(message)",
        "window?.a.b.postMessage(message)",
        "window.a?.b.postMessage(message)",
        "window?.a?.b.postMessage(message)",
        "const popup = window.open(url); popup.postMessage(message)",
        "let target = new Worker(url); target = window.parent; target.postMessage(message)",
        "const worker = new Worker(url); function send(worker) { worker.postMessage(message); }",
        "new Foo().postMessage(message)",
    ];

    let fix = vec![
        ("window.postMessage(message)", "window.postMessage(message, window.location.origin)"),
        ("self.postMessage(message)", "self.postMessage(message, self.location.origin)"),
        (
            "globalThis.postMessage(message)",
            "globalThis.postMessage(message, globalThis.location.origin)",
        ),
        ("foo.postMessage(message )", "foo.postMessage(message, foo.location.origin )"),
        ("foo?.postMessage(message )", "foo?.postMessage(message, foo.location.origin )"),
        ("window.postMessage(message,)", "window.postMessage(message, window.location.origin,)"),
        (
            "window.postMessage(message,                 /** comments */  )",
            "window.postMessage(message, window.location.origin,                 /** comments */  )",
        ),
        ("window?.c.postMessage(message)", "window?.c.postMessage(message, self.location.origin)"),
        (
            "window?.a?.b.postMessage(message)",
            "window?.a?.b.postMessage(message, self.location.origin)",
        ),
    ];

    Tester::new(
        RequirePostMessageTargetOrigin::NAME,
        RequirePostMessageTargetOrigin::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
