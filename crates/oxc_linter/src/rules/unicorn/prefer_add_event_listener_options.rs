use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_add_event_listener_options_diagnostic(span: Span, capture: bool) -> OxcDiagnostic {
    if capture {
        OxcDiagnostic::warn("Prefer `{capture: true}` over `true`.")
            .with_help("Replace `true` with `{capture: true}`.")
            .with_label(span)
    } else {
        OxcDiagnostic::warn("Prefer `{capture: false}` over `false`.")
            .with_help("Replace `false` with `{capture: false}`.")
            .with_label(span)
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferAddEventListenerOptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers an options object over a boolean third argument in `addEventListener()`.
    ///
    /// ### Why is this bad?
    ///
    /// The boolean argument is the legacy `useCapture` flag. `{capture: true}` says the same thing and can be extended with `once`, `passive`, or `signal`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// window.addEventListener("click", listener, true);
    /// window.addEventListener("click", listener, false);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// window.addEventListener("click", listener, {capture: true});
    /// window.addEventListener("click", listener);
    /// ```
    PreferAddEventListenerOptions,
    unicorn,
    pedantic,
    fix,
    version = "1.86.0",
    short_description = "Prefer an options object over a boolean in `addEventListener()`.",
);

impl Rule for PreferAddEventListenerOptions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional || call_expr.arguments.len() != 3 {
            return;
        }

        // `addEventListener("click", ...args, true)` is not a boolean useCapture argument.
        if call_expr.arguments.iter().any(Argument::is_spread) {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        if member_expr.optional() || member_expr.is_computed() {
            return;
        }
        if !matches!(member_expr.static_property_name(), Some("addEventListener")) {
            return;
        }

        let Some(options) = call_expr.arguments[2].as_expression() else {
            return;
        };
        // Parentheses are their own nodes here. Unwrap only those, so `true as boolean` stays ignored.
        let Expression::BooleanLiteral(literal) = options.without_parentheses() else {
            return;
        };

        let replacement = if literal.value { "{capture: true}" } else { "{capture: false}" };
        ctx.diagnostic_with_fix(
            prefer_add_event_listener_options_diagnostic(literal.span, literal.value),
            |fixer| fixer.replace(literal.span, replacement),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"window.addEventListener("click", listener)"#,
        r#"window.addEventListener("click", listener, {capture: true})"#,
        r#"window.addEventListener("click", listener, {capture: false})"#,
        r#"window.addEventListener("click", listener, {passive: true})"#,
        r#"window.addEventListener("click", listener, {once: true})"#,
        r#"window.addEventListener("click", listener, {signal})"#,
        r#"window.addEventListener("click", listener, options)"#,
        r#"window.addEventListener("click", listener, capture)"#,
        r#"window.addEventListener("click", listener, Boolean(value))"#,
        r#"window.addEventListener("click", listener, condition ? true : false)"#,
        r#"window.addEventListener("click", listener, undefined)"#,
        r#"window.addEventListener("click", listener, null)"#,
        r#"window["addEventListener"]("click", listener, true)"#,
        r#"window?.addEventListener("click", listener, true)"#,
        r#"window.addEventListener?.("click", listener, true)"#,
        r#"window.addEventListener("click", ...arguments_, true)"#,
    ];

    let fail = vec![
        r#"window.addEventListener("click", listener, true)"#,
        r#"window.addEventListener("click", listener, false)"#,
        r#"window.addEventListener("click", listener, (true))"#,
        r#"window.addEventListener("click", () => {}, true)"#,
        r#"window.addEventListener("click", function () {}, false)"#,
        r#"document.body.addEventListener("click", listener, true)"#,
        r#"(window).addEventListener("click", listener, false)"#,
        r#"window.addEventListener("click", listener, /* useCapture */ true)"#,
        r#"window.addEventListener("click", listener, true /* useCapture */)"#,
        r#"window.addEventListener(
	"click",
	listener,
	true
)"#,
    ];

    let fix = vec![
        (
            r#"window.addEventListener("click", listener, true)"#,
            r#"window.addEventListener("click", listener, {capture: true})"#,
        ),
        (
            r#"window.addEventListener("click", listener, false)"#,
            r#"window.addEventListener("click", listener, {capture: false})"#,
        ),
        (
            r#"window.addEventListener("click", listener, (true))"#,
            r#"window.addEventListener("click", listener, ({capture: true}))"#,
        ),
        (
            r#"window.addEventListener("click", () => {}, true)"#,
            r#"window.addEventListener("click", () => {}, {capture: true})"#,
        ),
        (
            r#"window.addEventListener("click", function () {}, false)"#,
            r#"window.addEventListener("click", function () {}, {capture: false})"#,
        ),
        (
            r#"document.body.addEventListener("click", listener, true)"#,
            r#"document.body.addEventListener("click", listener, {capture: true})"#,
        ),
        (
            r#"(window).addEventListener("click", listener, false)"#,
            r#"(window).addEventListener("click", listener, {capture: false})"#,
        ),
        (
            r#"window.addEventListener("click", listener, /* useCapture */ true)"#,
            r#"window.addEventListener("click", listener, /* useCapture */ {capture: true})"#,
        ),
        (
            r#"window.addEventListener("click", listener, true /* useCapture */)"#,
            r#"window.addEventListener("click", listener, {capture: true} /* useCapture */)"#,
        ),
        (
            r#"window.addEventListener(
	"click",
	listener,
	true
)"#,
            r#"window.addEventListener(
	"click",
	listener,
	{capture: true}
)"#,
        ),
    ];

    Tester::new(
        PreferAddEventListenerOptions::NAME,
        PreferAddEventListenerOptions::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
