use oxc_ast::ast::Expression;
use oxc_ast::{
    ast::{Argument, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-unicorn(no-invalid-remove-event-listener): Invalid `removeEventListener` call."
)]
#[diagnostic(severity(warning), help("The listener argument should be a function reference."))]
struct NoInvalidRemoveEventListenerDiagnostic(#[label] pub Span);

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
    /// ```javascript
    /// // Bad
    /// el.removeEventListener('click', () => {});
    /// el.removeEventListener('click', function () {});
    ///
    /// // Good
    /// el.removeEventListener('click', handler);
    /// el.removeEventListener('click', handler.bind(this));
    /// ```
    NoInvalidRemoveEventListener,
    correctness
);

impl Rule for NoInvalidRemoveEventListener {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };

        match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                if v.property.name != "removeEventListener" {
                    return;
                }
            }
            _ => return,
        }

        if member_expr.optional() {
            return;
        }

        if matches!(call_expr.arguments.get(0), Some(Argument::SpreadElement(_))) {
            return;
        }

        let Some(Argument::Expression(listener)) = call_expr.arguments.get(1) else { return };

        if !matches!(
            listener,
            Expression::FunctionExpression(_)
                | Expression::ArrowExpression(_)
                | Expression::CallExpression(_)
        ) {
            return;
        }

        if let Expression::CallExpression(call_expr) = listener {
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

        ctx.diagnostic(NoInvalidRemoveEventListenerDiagnostic(listener.span()));
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
    ];

    Tester::new_without_config(NoInvalidRemoveEventListener::NAME, pass, fail).test_and_snapshot();
}
