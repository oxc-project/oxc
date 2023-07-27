use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("removeEventListener() should be called with a correct listener")]
#[diagnostic(
    severity(warning),
    help(
        "This 'removeEventListener()' call does nothing because a newly created function is passed. Consider using the exact function instance that was added at the 'addEventListener()' call"
    )
)]
struct BadRemoveEventListenerDiagnostic(#[label] pub Span);

/// `https://deepscan.io/docs/rules/bad-remove-event-listener`
#[derive(Debug, Default, Clone)]
pub struct BadRemoveEventListener;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether a newly created function is passed to `removeEventListener`.
    ///
    /// ### Example
    /// ```javascript
    /// document.removeEventListener('keydown', function () {})
    /// ```
    BadRemoveEventListener,
    correctness
);

impl Rule for BadRemoveEventListener {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Some(member) = call_expr.callee.get_member_expr() else { return };
        if member.static_property_name() == Some("removeEventListener")
            && matches!(call_expr.arguments.get(1), Some(Argument::Expression(expr)) if expr.is_function())
        {
            ctx.diagnostic(BadRemoveEventListenerDiagnostic(call_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("document.removeEventListener('keydown', keydownHandler)", None),
        ("document.removeEventListener('keydown', this.keydownHandler)", None),
    ];

    let fail = vec![
        ("document.removeEventListener('keydown', () => foo())", None),
        ("document.removeEventListener('keydown', function () {})", None),
    ];

    Tester::new(BadRemoveEventListener::NAME, pass, fail).test_and_snapshot();
}
