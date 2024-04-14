use oxc_ast::{
    ast::{Argument, ArrayExpressionElement, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-await-in-promise-methods): Promise in `Promise.{1}()` should not be awaited.")]
#[diagnostic(severity(warning), help("Remove the `await`"))]
struct NoAwaitInPromiseMethodsDiagnostic(#[label] pub Span, String);

#[derive(Debug, Default, Clone)]
pub struct NoAwaitInPromiseMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using `await` in `Promise` method parameters
    ///
    /// ### Why is this bad?
    ///
    /// Using `await` on promises passed as arguments to `Promise.all()`, `Promise.allSettled()`, `Promise.any()`, or `Promise.race()` is likely a mistake.
    ///
    /// ### Example
    /// Bad
    ///
    /// ```js
    /// Promise.all([await promise, anotherPromise]);
    /// Promise.allSettled([await promise, anotherPromise]);
    /// Promise.any([await promise, anotherPromise]);
    /// Promise.race([await promise, anotherPromise]);
    /// ```
    ///
    /// Good
    ///
    /// ```js
    /// Promise.all([promise, anotherPromise]);
    /// Promise.allSettled([promise, anotherPromise]);
    /// Promise.any([promise, anotherPromise]);
    /// Promise.race([promise, anotherPromise]);
    /// ```
    NoAwaitInPromiseMethods,
    correctness
);

impl Rule for NoAwaitInPromiseMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(
            call_expr,
            Some(&["Promise"]),
            Some(&["all", "allSettled", "any", "race"]),
            Some(1),
            Some(1),
        ) {
            return;
        }

        let Some(Argument::Expression(first_argument)) = call_expr.arguments.first() else {
            return;
        };
        let first_argument = first_argument.without_parenthesized();
        let Expression::ArrayExpression(first_argument_array_expr) = first_argument else {
            return;
        };

        for element in &first_argument_array_expr.elements {
            if let ArrayExpressionElement::Expression(element_expr) = element {
                if let Expression::AwaitExpression(await_expr) =
                    element_expr.without_parenthesized()
                {
                    let property_name = call_expr
                        .callee
                        .get_member_expr()
                        .expect("callee is a member expression")
                        .static_property_name()
                        .expect("callee is a static property");

                    ctx.diagnostic(NoAwaitInPromiseMethodsDiagnostic(
                        Span::new(await_expr.span.start, await_expr.span.start + 5),
                        property_name.to_string(),
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.all([promise1, promise2, promise3, promise4])",
        "Promise.allSettled([promise1, promise2, promise3, promise4])",
        "Promise.any([promise1, promise2, promise3, promise4])",
        "Promise.race([promise1, promise2, promise3, promise4])",
        "Promise.all(...[await promise])",
        "Promise.all([await promise], extraArguments)",
        "Promise.all()",
        "Promise.all(notArrayExpression)",
        "Promise.all([,])",
        "Promise[all]([await promise])",
        "Promise.notListedMethod([await promise])",
        "NotPromise.all([await promise])",
        "Promise.all([(await promise, 0)])",
        "new Promise.all([await promise])",
        "globalThis.Promise.all([await promise])",
    ];

    let fail = vec![
        "Promise.all([await promise])",
        "Promise.allSettled([await promise])",
        "Promise.any([await promise])",
        "Promise.race([await promise])",
        "Promise.all([, await promise])",
        "Promise.all([await promise,])",
        "Promise.all([await promise],)",
        "Promise.all([await (0, promise)],)",
        "Promise.all([await (( promise ))])",
        "Promise.all([await await promise])",
        "Promise.all([...foo, await promise1, await promise2])",
        "Promise.all([await /* comment*/ promise])",
    ];

    Tester::new(NoAwaitInPromiseMethods::NAME, pass, fail).test_and_snapshot();
}
