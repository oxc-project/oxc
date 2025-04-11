use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn avoid_new_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid creating new promises").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AvoidNew;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow creating promises with `new Promise()`.
    ///
    /// ### Why is this bad?
    ///
    /// Many cases that use `new Promise()` could be refactored to use an
    /// `async` function. `async` is considered more idiomatic in modern JavaScript.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() {
    ///     return new Promise((resolve, reject) => { /* ... */ });
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function foo() {
    ///     // ...
    /// }
    /// const bar = await Promise.all([baz(), bang()]);
    /// ```
    AvoidNew,
    promise,
    style,
);

impl Rule for AvoidNew {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = &expr.callee else {
            return;
        };

        if ident.name == "Promise"
            && ctx.scoping().root_unresolved_references().contains_key(ident.name.as_str())
        {
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
        "var Promise = a; new Promise()",
    ];

    let fail = vec![
        "var x = new Promise(function (x, y) {})",
        "new Promise()",
        "Thing(new Promise(() => {}))",
    ];

    Tester::new(AvoidNew::NAME, AvoidNew::PLUGIN, pass, fail).test_and_snapshot();
}
