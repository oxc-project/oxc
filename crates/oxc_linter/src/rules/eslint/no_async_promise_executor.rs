use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_async_promise_executor_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Promise executor functions should not be `async`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncPromiseExecutor;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow using an async function as a Promise executor
    ///
    /// ### Why is this bad?
    /// The `new Promise` constructor accepts an executor function as an argument,
    /// which has `resolve` and `reject` parameters that can be used to control the state of the created Promise.
    /// For example:
    ///
    /// ### Example
    /// ```javascript
    /// const result = new Promise(function executor(resolve, reject) {
    ///   readFile('foo.txt', function(err, result) {
    ///     if (err) {
    ///       reject(err);
    ///     } else {
    ///       resolve(result);
    ///     }
    ///   });
    /// });
    /// ```
    ///
    /// The executor function can also be an `async function`. However, this is usually a mistake, for a few reasons:
    ///
    /// - If an async executor function throws an error, the error will be lost and won’t cause the newly-constructed `Promise` to reject.This could make it difficult to debug and handle some errors.
    /// - If a Promise executor function is using `await`, this is usually a sign that it is not actually necessary to use the `new Promise` constructor, or the scope of the `new Promise` constructor can be reduced.
    NoAsyncPromiseExecutor,
    correctness
);

impl Rule for NoAsyncPromiseExecutor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expression) = node.kind() else {
            return;
        };
        if !new_expression.callee.is_specific_id("Promise") {
            return;
        }
        let Some(expression) = new_expression.arguments.first().and_then(Argument::as_expression)
        else {
            return;
        };
        let mut span = match expression.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) if arrow.r#async => arrow.span,
            Expression::FunctionExpression(func) if func.r#async => func.span,
            _ => return,
        };

        span.end = span.start + 5;

        ctx.diagnostic(no_async_promise_executor_diagnostic(span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new Promise((resolve, reject) => {})", None),
        ("new Promise((resolve, reject) => {}, async function unrelated() {})", None),
        ("new Foo(async (resolve, reject) => {})", None),
    ];

    let fail = vec![
        ("new Promise(async function foo(resolve, reject) {})", None),
        ("new Promise(async (resolve, reject) => {})", None),
        ("new Promise(((((async () => {})))))", None),
    ];

    Tester::new(NoAsyncPromiseExecutor::NAME, pass, fail).test_and_snapshot();
}
