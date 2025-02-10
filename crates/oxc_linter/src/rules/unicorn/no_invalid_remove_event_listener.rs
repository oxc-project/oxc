use oxc_ast::{
    ast::{Argument, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_invalid_remove_event_listener_diagnostic(call_span: Span, arg_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid `removeEventListener` call.")
        .with_help("The listener argument should be a function reference.")
        .with_labels([
            call_span.label("`removeEventListener` called here."),
            arg_span.label("Invalid argument here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoInvalidRemoveEventListener;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// It warns when you use a non-function value as the second argument of `removeEventListener`.
    ///
    /// ### Why is this bad?
    ///
    /// The [`removeEventListener`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/removeEventListener) function must be called with a reference to the same function that was passed to [`addEventListener`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener). Calling `removeEventListener` with an inline function or the result of an inline `.bind()` call is indicative of an error, and won't actually remove the listener.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// el.removeEventListener('click', () => {});
    /// el.removeEventListener('click', function () {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// el.removeEventListener('click', handler);
    /// el.removeEventListener('click', handler.bind(this));
    /// ```
    NoInvalidRemoveEventListener,
    unicorn,
    correctness
);

impl Rule for NoInvalidRemoveEventListener {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let remove_event_listener_ident_span = match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                if v.property.name != "removeEventListener" {
                    return;
                }
                v.property.span
            }
            _ => return,
        };

        if member_expr.optional() {
            return;
        }

        if matches!(call_expr.arguments.first(), Some(Argument::SpreadElement(_))) {
            return;
        }

        let Some(listener) = call_expr.arguments.get(1) else {
            return;
        };

        if !matches!(
            listener,
            Argument::FunctionExpression(_)
                | Argument::ArrowFunctionExpression(_)
                | Argument::CallExpression(_)
        ) {
            return;
        }

        if let Argument::CallExpression(call_expr) = listener {
            match call_expr.callee.get_member_expr() {
                Some(MemberExpression::StaticMemberExpression(v)) => {
                    if v.property.name != "bind" {
                        return;
                    }
                }
                _ => {
                    return;
                }
            }
        }

        let listener_span = listener.span();
        let listener_span = if listener_span.size() > 20 {
            match listener {
                Argument::FunctionExpression(func_expr) => {
                    Span::new(func_expr.span.start, func_expr.params.span.end)
                }
                Argument::ArrowFunctionExpression(arrow_expr) => {
                    Span::new(arrow_expr.span.start, arrow_expr.body.span.start)
                }
                Argument::CallExpression(_) => listener_span,
                _ => unreachable!(),
            }
        } else {
            listener_span
        };

        ctx.diagnostic(no_invalid_remove_event_listener_diagnostic(
            remove_event_listener_ident_span,
            listener_span,
        ));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"new el.removeEventListener("click", () => {})"#,
        r#"el?.removeEventListener("click", () => {})"#,
        r#"el.removeEventListener?.("click", () => {})"#,
        r#"el.notRemoveEventListener("click", () => {})"#,
        r#"el[removeEventListener]("click", () => {})"#,
        r#"el.removeEventListener("click")"#,
        r"el.removeEventListener()",
        r"el.removeEventListener(() => {})",
        r#"el.removeEventListener(...["click", () => {}], () => {})"#,
        r#"el.removeEventListener(() => {}, "click")"#,
        r#"window.removeEventListener("click", bind())"#,
        r#"window.removeEventListener("click", handler.notBind())"#,
        r#"window.removeEventListener("click", handler[bind]())"#,
        r#"window.removeEventListener("click", handler.bind?.())"#,
        r#"window.removeEventListener("click", handler?.bind())"#,
        r"window.removeEventListener(handler)",
        r#"this.removeEventListener("click", getListener())"#,
        r#"el.removeEventListener("scroll", handler)"#,
        r#"el.removeEventListener("keydown", obj.listener)"#,
        r#"removeEventListener("keyup", () => {})"#,
        r#"removeEventListener("keydown", function () {})"#,
        "document.removeEventListener('keydown', keydownHandler)",
        "document.removeEventListener('keydown', this.keydownHandler)",
    ];

    let fail = vec![
        r#"window.removeEventListener("scroll", handler.bind(abc))"#,
        r#"window.removeEventListener("scroll", this.handler.bind(abc))"#,
        r#"window.removeEventListener("click", () => {})"#,
        r#"window.removeEventListener("keydown", function () {})"#,
        r#"el.removeEventListener("click", (e) => { e.preventDefault(); })"#,
        r#"el.removeEventListener("mouseover", fn.bind(abc))"#,
        r#"el.removeEventListener("mouseout", function (e) {})"#,
        r#"el.removeEventListener("mouseout", function (e) {}, true)"#,
        r#"el.removeEventListener("click", function (e) {}, ...moreArguments)"#,
        r"el.removeEventListener(() => {}, () => {}, () => {})",
        "document.removeEventListener('keydown', () => foo())",
        "document.removeEventListener('keydown', function () {})",
        // make sure that if the listener is a big one, we shorten the span.
        r#"
        element.removeEventListener("glider-refresh", event => {
            // $ExpectType GliderEvent<undefined>
            event;

            // $ExpectType boolean
            event.bubbles;

            event.target;

            if (event.target) {
                // $ExpectType Glider<HTMLElement> | undefined
                event.target._glider;
            }
        });
        "#,
        r#"
        element.removeEventListener("glider-refresh", function (event) {
            // $ExpectType GliderEvent<undefined>
            event;

            // $ExpectType boolean
            event.bubbles;

            event.target;

            if (event.target) {
                // $ExpectType Glider<HTMLElement> | undefined
                event.target._glider;
            }
        });
        "#,
    ];

    Tester::new(
        NoInvalidRemoveEventListener::NAME,
        NoInvalidRemoveEventListener::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
