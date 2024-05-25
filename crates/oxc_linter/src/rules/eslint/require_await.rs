use oxc_ast::{
    ast::{Expression, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct RequireAwait;

fn require_await_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(require-await): Async function has no 'await' expression.")
        .with_labels([span0.into()])
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow async functions which have no await expression.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// async function foo() {
    ///   doSomething();
    /// }
    /// ```
    RequireAwait,
    pedantic,
);

impl Rule for RequireAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::FunctionBody(body) = node.kind() {
            if body.is_empty() {
                return;
            }

            let Some(parent) = ctx.nodes().parent_kind(node.id()) else {
                return;
            };

            match parent {
                AstKind::Function(func) => {
                    if func.r#async && !func.generator && !has_await(&body.statements) {
                        ctx.diagnostic(require_await_diagnostic(func.span));
                    }
                }
                AstKind::ArrowFunctionExpression(func) => {
                    if func.r#async && !has_await(&body.statements) {
                        ctx.diagnostic(require_await_diagnostic(func.span));
                    }
                }
                _ => {}
            }
        }
    }
}

fn has_await<'a>(statements: &'a [Statement<'a>]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::ExpressionStatement(expr) => {
            matches!(expr.expression, Expression::AwaitExpression(_))
        }
        Statement::ForOfStatement(f) => f.r#await,
        _ => false,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "async function foo() { await doSomething() }",
        "(async function() { await doSomething() })",
        "async () => { await doSomething() }",
        "async () => await doSomething()",
        "({ async foo() { await doSomething() } })",
        "class A { async foo() { await doSomething() } }",
        "(class { async foo() { await doSomething() } })",
        "async function foo() { await (async () => { await doSomething() }) }",
        "async function foo() {}",
        "async () => {}",
        "function foo() { doSomething() }",
        "async function foo() { for await (x of xs); }",
        "await foo()",
        "
			                for await (let num of asyncIterable) {
			                    console.log(num);
			                }
			            ",
        "async function* run() { yield * anotherAsyncGenerator() }",
        "async function* run() {
			                await new Promise(resolve => setTimeout(resolve, 100));
			                yield 'Hello';
			                console.log('World');
			            }
			            ",
        "async function* run() { }",
        "const foo = async function *(){}",
        r#"const foo = async function *(){ console.log("bar") }"#,
        r#"async function* run() { console.log("bar") }"#,
    ];

    let fail = vec![
        "async function foo() { doSomething() }",
        "(async function() { doSomething() })",
        "async () => { doSomething() }",
        "async () => doSomething()",
        "({ async foo() { doSomething() } })",
        "class A { async foo() { doSomething() } }",
        "(class { async foo() { doSomething() } })",
        "(class { async ''() { doSomething() } })",
        "async function foo() { async () => { await doSomething() } }",
        "async function foo() { await (async () => { doSomething() }) }",
    ];

    Tester::new(RequireAwait::NAME, pass, fail).test_and_snapshot();
}
