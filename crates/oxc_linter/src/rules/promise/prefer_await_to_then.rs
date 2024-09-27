use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

fn prefer_wait_to_then_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer await to then()/catch()/finally()").with_label(span)
}

use crate::{context::LintContext, rule::Rule, utils::is_promise, AstNode};

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() { hey.then(x => {}) }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function hi() { await thing() }
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

        if is_promise(call_expr).is_none() {
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
        // <https://github.com/tc39/proposal-top-level-await>
        // Top level await is allowed now, so comment this out
        // "something().then(async () => await somethingElse())",
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
        "something().then(async () => await somethingElse())",
    ];

    Tester::new(PreferAwaitToThen::NAME, pass, fail).test_and_snapshot();
}
