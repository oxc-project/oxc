use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_promise, AstNode};

fn zero_or_one_argument_required_diagnostic(
    span: Span,
    prop_name: &str,
    args_len: usize,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Promise.{prop_name}() requires 0 or 1 arguments, but received {args_len}"
    ))
    .with_label(span)
}

fn one_or_two_argument_required_diagnostic(
    span: Span,
    prop_name: &str,
    args_len: usize,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Promise.{prop_name}() requires 1 or 2 arguments, but received {args_len}"
    ))
    .with_label(span)
}

fn one_argument_required_diagnostic(span: Span, prop_name: &str, args_len: usize) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Promise.{prop_name}() requires 1 argument, but received {args_len}"
    ))
    .with_label(span)
}

fn valid_params_diagnostic(span: Span, x0: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(x0.to_string()).with_label(span)
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Promise.resolve(1, 2)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// Promise.resolve(1)
    /// ```
    ValidParams,
    promise,
    correctness,
);

impl Rule for ValidParams {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(prop_name) = is_promise(call_expr) else {
            return;
        };

        let args_len = call_expr.arguments.len();

        match prop_name.as_str() {
            "resolve" | "reject" => {
                if args_len > 1 {
                    ctx.diagnostic(zero_or_one_argument_required_diagnostic(
                        call_expr.span,
                        &prop_name,
                        args_len,
                    ));
                }
            }
            "then" => {
                if args_len != 1 && args_len != 2 {
                    ctx.diagnostic(one_or_two_argument_required_diagnostic(
                        call_expr.span,
                        &prop_name,
                        args_len,
                    ));
                    ctx.diagnostic(valid_params_diagnostic(call_expr.span, &format!("Promise.{prop_name}() requires 1 or 2 arguments, but received {args_len}")));
                }
            }
            "race" | "all" | "allSettled" | "any" | "catch" | "finally" => {
                if args_len != 1 {
                    ctx.diagnostic(one_argument_required_diagnostic(
                        call_expr.span,
                        &prop_name,
                        args_len,
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

    Tester::new(ValidParams::NAME, ValidParams::PLUGIN, pass, fail).test_and_snapshot();
}
