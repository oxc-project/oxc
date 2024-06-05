use oxc_ast::{
    ast::{CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;

use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::is_method_call, context::LintContext, fixer::Fix, rule::Rule, AstNode};

fn no_single_promise_in_promise_methods_diagnostic(span0: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-unicorn(no-single-promise-in-promise-methods): Wrapping single-element array with `Promise.{x1}()` is unnecessary."))
        .with_help("Either use the value directly, or switch to `Promise.resolve(…)`.")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct NoSinglePromiseInPromiseMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow passing single-element arrays to Promise methods
    ///
    /// ### Why is this bad?
    ///
    /// Passing a single-element array to `Promise.all()`, `Promise.any()`, or `Promise.race()` is likely a mistake.
    ///
    ///
    /// ### Example
    ///
    /// Bad
    /// ```js
    /// const foo = await Promise.all([promise]);
    /// const foo = await Promise.any([promise]);
    /// const foo = await Promise.race([promise]);
    /// const promise = Promise.all([nonPromise]);
    /// ```
    ///
    /// Good
    /// ```js
    /// const foo = await promise;
    /// const promise = Promise.resolve(nonPromise);
    /// const foo = await Promise.all(promises);
    /// const foo = await Promise.any([promise, anotherPromise]);
    /// const [{ value: foo, reason: error }] = await Promise.allSettled([promise]);
    /// ```
    ///
    NoSinglePromiseInPromiseMethods,
    correctness
);

impl Rule for NoSinglePromiseInPromiseMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_promise_method_with_single_argument(call_expr) {
            return;
        }
        let Some(first_argument) = call_expr.arguments[0].as_expression() else {
            return;
        };
        let first_argument = first_argument.without_parenthesized();
        let Expression::ArrayExpression(first_argument_array_expr) = first_argument else {
            return;
        };

        if first_argument_array_expr.elements.len() != 1 {
            return;
        }

        let first = &first_argument_array_expr.elements[0];
        if !first.is_expression() {
            return;
        }

        let info = call_expr
            .callee
            .get_member_expr()
            .expect("callee is a member expression")
            .static_property_info()
            .expect("callee is a static property");

        let diagnostic = no_single_promise_in_promise_methods_diagnostic(info.0, info.1);
        ctx.diagnostic_with_fix(diagnostic, || {
            let elem_text = first.span().source_text(ctx.source_text());

            let is_directly_in_await = ctx
                .semantic()
                .nodes()
                // get first non-parenthesis parent node
                .iter_parents(node.id())
                .skip(1) // first node is the call expr
                .find(|parent| !matches!(parent.kind(), AstKind::ParenthesizedExpression(_)))
                // check if it's an `await ...` expression
                .is_some_and(|parent| matches!(parent.kind(), AstKind::AwaitExpression(_)));

            let call_span = call_expr.span;

            if is_directly_in_await {
                Fix::new(elem_text, call_span)
            } else {
                Fix::new(format!("Promise.resolve({elem_text})"), call_span)
            }
        });
    }
}

fn is_promise_method_with_single_argument(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&["Promise"]), Some(&["all", "any", "race"]), Some(1), Some(1))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.all([promise, anotherPromise])",
        "Promise.all(notArrayLiteral)",
        "Promise.all([...promises])",
        "Promise.any([promise, anotherPromise])",
        "Promise.race([promise, anotherPromise])",
        "Promise.notListedMethod([promise])",
        "Promise[all]([promise])",
        "Promise.all([,])",
        "NotPromise.all([promise])",
        "Promise.all(...[promise])",
        "Promise.all([promise], extraArguments)",
        "Promise.all()",
        "new Promise.all([promise])",
        "globalThis.Promise.all([promise])",
        "Promise.allSettled([promise])",
    ];

    let fail = vec![
        "await Promise.all([(0, promise)])",
        "async function * foo() {await Promise.all([yield promise])}",
        "async function * foo() {await Promise.all([yield* promise])}",
        "await Promise.all([() => promise,],)",
        "await Promise.all([a ? b : c,],)",
        "await Promise.all([x ??= y,],)",
        "await Promise.all([x ||= y,],)",
        "await Promise.all([x &&= y,],)",
        "await Promise.all([x |= y,],)",
        "await Promise.all([x ^= y,],)",
        "await Promise.all([x ??= y,],)",
        "await Promise.all([x ||= y,],)",
        "await Promise.all([x &&= y,],)",
        "await Promise.all([x | y,],)",
        "await Promise.all([x ^ y,],)",
        "await Promise.all([x & y,],)",
        "await Promise.all([x !== y,],)",
        "await Promise.all([x == y,],)",
        "await Promise.all([x in y,],)",
        "await Promise.all([x >>> y,],)",
        "await Promise.all([x + y,],)",
        "await Promise.all([x / y,],)",
        "await Promise.all([x ** y,],)",
        "await Promise.all([promise,],)",
        "await Promise.all([getPromise(),],)",
        "await Promise.all([promises[0],],)",
        "await Promise.all([await promise])",
        "await Promise.any([promise])",
        "await Promise.race([promise])",
        "await Promise.all([new Promise(() => {})])",
        "+await Promise.all([+1])",
        "
        await Promise.all([(x,y)])
        [0].toString()
		",
        "Promise.all([promise,],)",
        "
		foo
		Promise.all([(0, promise),],)
		",
        "
        foo
        Promise.all([[array][0],],)
		",
        "Promise.all([promise]).then()",
        "Promise.all([1]).then()",
        "Promise.all([1.]).then()",
        "Promise.all([.1]).then()",
        "Promise.all([(0, promise)]).then()",
        "const _ = () => Promise.all([ a ?? b ,],)",
        "Promise.all([ {a} = 1 ,],)",
        "Promise.all([ function () {} ,],)",
        "Promise.all([ class {} ,],)",
        "Promise.all([ new Foo ,],).then()",
        "Promise.all([ new Foo ,],).toString",
        "foo(Promise.all([promise]))",
        "Promise.all([promise]).foo = 1",
        "Promise.all([promise])[0] ||= 1",
        "Promise.all([undefined]).then()",
        "Promise.all([null]).then()",
    ];

    let fix = vec![
        ("Promise.all([null]).then()", "Promise.resolve(null).then()", None),
        ("let x = await Promise.all([foo()])", "let x = await foo()", None),
        ("let x = await (Promise.all([foo()]))", "let x = await (foo())", None),
    ];

    Tester::new(NoSinglePromiseInPromiseMethods::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
