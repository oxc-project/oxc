use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn avoid_new_promise_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid creating new promises").with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct AvoidNew;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow creating new promises outside of utility libs.
    ///
    /// ### Why is this bad?
    ///
    /// If you dislike the new promise style promises.
    ///
    /// ### Example
    /// ```javascript
    /// new Promise((resolve, reject) => { ... });
    /// ```
    AvoidNew,
    restriction,
);

impl Rule for AvoidNew {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = &expr.callee else {
            return;
        };

        if ident.name == "Promise" && ctx.semantic().is_reference_to_global_variable(ident) {
            ctx.diagnostic(avoid_new_promise_diagnostic(expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.resolve()",
        "Promise.reject()",
        "Promise.all()",
        "new Horse()",
        "new PromiseLikeThing()",
        "new Promise.resolve()",
    ];

    let fail = vec![
        "var x = new Promise(function (x, y) {})",
        "new Promise()",
        "Thing(new Promise(() => {}))",
    ];

    Tester::new(AvoidNew::NAME, pass, fail).test_and_snapshot();
}
