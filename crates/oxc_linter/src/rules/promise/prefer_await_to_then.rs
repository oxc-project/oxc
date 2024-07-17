use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

fn prefer_wait_to_then_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-promise(prefer-await-to-then): Prefer await to then()/catch()/finally()",
    )
    .with_label(span0)
}

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct PreferAwaitToThen;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `await` to `then()`/`catch()`/`finally()` for reading Promise values
    ///
    /// ### Why is this bad?
    ///
    /// Async/await syntax can be seen as more readable.
    ///
    /// ### Example
    /// ```javascript
    /// myPromise.then(doSomething)
    /// ```
    PreferAwaitToThen,
    style,
);

fn is_inside_yield_or_await(node: &AstNode) -> bool {
    matches!(node.kind(), AstKind::YieldExpression(_) | AstKind::AwaitExpression(_))
}

impl Rule for PreferAwaitToThen {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let Some(prop_name) = member_expr.static_property_name() else {
            return;
        };

        if !matches!(prop_name, "then" | "catch" | "finally") {
            return;
        }

        // Await statements cannot be added to the top level scope
        if ctx.scopes().get_flags(node.scope_id()).is_top() {
            return;
        }

        // Already inside a yield or await
        if ctx
            .nodes()
            .ancestors(node.id())
            .any(|node_id| is_inside_yield_or_await(ctx.nodes().get_node(node_id)))
        {
            return;
        }

        ctx.diagnostic(prefer_wait_to_then_diagnostic(call_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "async function hi() { await thing() }",
        "async function hi() { await thing().then() }",
        "async function hi() { await thing().catch() }",
        "a = async () => (await something())",
        "a = async () => {
			      try { await something() } catch (error) { somethingElse() }
			    }",
        "something().then(async () => await somethingElse())",
        "function foo() { hey.somethingElse(x => {}) }",
        "const isThenable = (obj) => {
			      return obj && typeof obj.then === 'function';
			    };",
        "function isThenable(obj) {
			      return obj && typeof obj.then === 'function';
			    }",
    ];

    let fail = vec![
        "function foo() { hey.then(x => {}) }",
        "function foo() { hey.then(function() { }).then() }",
        "function foo() { hey.then(function() { }).then(x).catch() }",
        "async function a() { hey.then(function() { }).then(function() { }) }",
        "function foo() { hey.catch(x => {}) }",
        "function foo() { hey.finally(x => {}) }",
    ];

    Tester::new(PreferAwaitToThen::NAME, pass, fail).test_and_snapshot();
}
