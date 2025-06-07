use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

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
    #[expect(clippy::cast_possible_truncation)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let arguments = &call_expr.arguments;
        if arguments.len() != 1 || call_expr.optional {
            return;
        }
        let arg = &call_expr.arguments[0];
        // ignore "foo.postMessage(...message)"
        if matches!(arg, Argument::SpreadElement(_)) {
            return;
        }
        if !is_method_call(call_expr, None, Some(&["postMessage"]), Some(1), Some(1)) {
            return;
        }
        // ignore "foo['postMessage'](message)"
        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        // ignore "foo[postMessage](message)" and "foo?.postMessage(message)"
        if member_expr.is_computed() || member_expr.optional() {
            return;
        }

        let comma_idx =
            Span::new(arg.span().end, call_expr.span.end).source_text(ctx.source_text()).find(',');
        let offset = comma_idx.unwrap_or(0) as u32;
        let target_span = Span::new(arg.span().end + offset, call_expr.span.end);
        ctx.diagnostic_with_suggestion(
            require_post_message_target_origin_diagnostic(target_span),
            |fixer| {
                let last_token = Span::new(call_expr.span.end - 1, call_expr.span.end);
                let text = match member_expr.object() {
                    Expression::Identifier(ident) => {
                        format!("{}.location.origin", ident.name.as_str())
                    }
                    _ => "self.location.origin".to_string(),
                };

                let replacement =
                    if comma_idx.is_some() { format!(" {text},") } else { format!(", {text}") };
                fixer.insert_text_before(&last_token, replacement)
            },
        );
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
        ("foo.postMessage(message )", "foo.postMessage(message , foo.location.origin)", None),
        (
            "window.postMessage(message,)",
            "window.postMessage(message, window.location.origin,)",
            None,
        ),
        (
            "window.postMessage(message,                 /** comments */  )",
            "window.postMessage(message,                 /** comments */   window.location.origin,)",
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
