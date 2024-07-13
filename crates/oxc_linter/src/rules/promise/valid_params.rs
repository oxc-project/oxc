use oxc_ast::{
    ast::{CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn valid_params_diagnostic(span0: Span, x0: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-promise(valid-params): {x0}")).with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct ValidParams;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the proper number of arguments are passed to Promise functions.
    ///
    /// ### Why is this bad?
    ///
    /// Calling a Promise function with the incorrect number of arguments can lead to unexpected
    /// behavior or hard to spot bugs.
    ///
    /// ### Example
    /// ```javascript
    /// Promise.resolve(1, 2)
    /// ```
    ValidParams,
    correctness,
);

fn is_promise(call_expr: &CallExpression) -> bool {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    let Some(prop_name) = member_expr.static_property_name() else {
        return false;
    };

    // hello.then(), hello.catch(), hello.finally()
    if matches!(prop_name, "then" | "catch" | "finally") {
        return true;
    }

    // foo().then(foo => {}), needed?
    if let Expression::CallExpression(inner_call_expr) = member_expr.object() {
        return is_promise(inner_call_expr);
    }

    if member_expr.object().is_specific_id("Promise")
        && matches!(
            prop_name,
            "resolve" | "reject" | "all" | "allSettled" | "race" | "any" | "withResolvers"
        )
    {
        return true;
    }

    false
}

impl Rule for ValidParams {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_promise(call_expr) {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let Some(prop_name) = member_expr.static_property_name() else {
            return;
        };

        let args_len = call_expr.arguments.len();

        match prop_name {
            "resolve" | "reject" => {
                if args_len > 1 {
                    ctx.diagnostic(valid_params_diagnostic(call_expr.span, &format!("Promise.{prop_name}() requires 0 or 1 arguments, but received {args_len}")));
                }
            }
            "then" => {
                if !(1..=2).contains(&args_len) {
                    ctx.diagnostic(valid_params_diagnostic(call_expr.span, &format!("Promise.{prop_name}() requires 1 or 2 arguments, but received {args_len}")));
                }
            }
            "race" | "all" | "allSettled" | "any" | "catch" | "finally" => {
                if args_len != 1 {
                    ctx.diagnostic(valid_params_diagnostic(
                        call_expr.span,
                        &format!(
                            "Promise.{prop_name}() requires 1 argument, but received {args_len}"
                        ),
                    ));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.resolve()",
        "Promise.resolve(1)",
        "Promise.resolve({})",
        "Promise.resolve(referenceToSomething)",
        "Promise.reject()",
        "Promise.reject(1)",
        "Promise.reject({})",
        "Promise.reject(referenceToSomething)",
        "Promise.reject(Error())",
        "Promise.race([])",
        "Promise.race(iterable)",
        "Promise.race([one, two, three])",
        "Promise.all([])",
        "Promise.all(iterable)",
        "Promise.all([one, two, three])",
        "Promise.allSettled([])",
        "Promise.allSettled(iterable)",
        "Promise.allSettled([one, two, three])",
        "Promise.any([])",
        "Promise.any(iterable)",
        "Promise.any([one, two, three])",
        "somePromise().then(success)",
        "somePromise().then(success, failure)",
        "promiseReference.then(() => {})",
        "promiseReference.then(() => {}, () => {})",
        "somePromise().catch(callback)",
        "somePromise().catch(err => {})",
        "promiseReference.catch(callback)",
        "promiseReference.catch(err => {})",
        "somePromise().finally(callback)",
        "somePromise().finally(() => {})",
        "promiseReference.finally(callback)",
        "promiseReference.finally(() => {})",
        "Promise.all([
			  Promise.resolve(1),
			  Promise.resolve(2),
			  Promise.reject(Error()),
			])
			  .then(console.log)
			  .catch(console.error)
			  .finally(console.log)
			",
    ];

    let fail = vec![
        "Promise.resolve(1, 2)",
        "Promise.resolve({}, function() {}, 1, 2, 3)",
        "Promise.reject(1, 2, 3)",
        "Promise.reject({}, function() {}, 1, 2)",
        "Promise.race(1, 2)",
        "Promise.race({}, function() {}, 1, 2, 3)",
        "Promise.all(1, 2, 3)",
        "Promise.all({}, function() {}, 1, 2)",
        "Promise.allSettled(1, 2, 3)",
        "Promise.allSettled({}, function() {}, 1, 2)",
        "Promise.any(1, 2, 3)",
        "Promise.any({}, function() {}, 1, 2)",
        "somePromise().then()",
        "somePromise().then(() => {}, () => {}, () => {})",
        "promiseReference.then()",
        "promiseReference.then(() => {}, () => {}, () => {})",
        "somePromise().catch()",
        "somePromise().catch(() => {}, () => {})",
        "promiseReference.catch()",
        "promiseReference.catch(() => {}, () => {})",
        "somePromise().finally()",
        "somePromise().finally(() => {}, () => {})",
        "promiseReference.finally()",
        "promiseReference.finally(() => {}, () => {})",
    ];

    Tester::new(ValidParams::NAME, pass, fail).test_and_snapshot();
}
