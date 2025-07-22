use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn require_post_message_target_origin_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing the `targetOrigin` argument.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequirePostMessageTargetOrigin;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using the targetOrigin argument with window.postMessage()
    ///
    /// ### Why is this bad?
    ///
    /// When calling window.postMessage() without the targetOrigin argument,
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
    /// ```
    RequirePostMessageTargetOrigin,
    unicorn,
    suspicious,
    suggestion
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
            // ignore "foo[postMessage](message)" and "foo?.postMessage(message)"
            Some(expr) if !(expr.is_computed() || expr.optional()) => expr,
            _ => return,
        };
        if matches!(member_expr.static_property_name(), Some(name) if name == "postMessage") {
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
        "window?.postMessage(message)",
        "window?.[postMessage](message)",
        r"window?.['postMessage'](message)",
        "window.c?.postMessage(message)",
        "window.c.postMessage?.(message)",
        "window.a.b?.postMessage(message)",
        "window?.a?.b?.postMessage(message)",
    ];

    let fail = vec![
        "window.postMessage(message)",
        "self.postMessage(message)",
        "globalThis.postMessage(message)",
        "foo.postMessage(message )",
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
    ];

    let fix = vec![
        (
            "window.postMessage(message)",
            "window.postMessage(message, window.location.origin)",
            None,
        ),
        ("self.postMessage(message)", "self.postMessage(message, self.location.origin)", None),
        (
            "globalThis.postMessage(message)",
            "globalThis.postMessage(message, globalThis.location.origin)",
            None,
        ),
        ("foo.postMessage(message )", "foo.postMessage(message, foo.location.origin )", None),
        (
            "window.postMessage(message,)",
            "window.postMessage(message, window.location.origin,)",
            None,
        ),
        (
            "window.postMessage(message,                 /** comments */  )",
            "window.postMessage(message, window.location.origin,                 /** comments */  )",
            None,
        ),
        (
            "window?.c.postMessage(message)",
            "window?.c.postMessage(message, self.location.origin)",
            None,
        ),
        (
            "window?.a?.b.postMessage(message)",
            "window?.a?.b.postMessage(message, self.location.origin)",
            None,
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
