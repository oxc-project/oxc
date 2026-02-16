use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn no_await_in_promise_methods_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Promise in `Promise.{method_name}()` should not be awaited."))
        .with_help("Remove the `await`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAwaitInPromiseMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using `await` in `Promise` method parameters
    ///
    /// ### Why is this bad?
    ///
    /// Using `await` on promises passed as arguments to `Promise.all()`,
    /// `Promise.allSettled()`, `Promise.any()`, or `Promise.race()` is likely a
    /// mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async function foo() {
    ///     Promise.all([await promise, anotherPromise]);
    ///     Promise.allSettled([await promise, anotherPromise]);
    ///     Promise.any([await promise, anotherPromise]);
    ///     Promise.race([await promise, anotherPromise]);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function foo() {
    ///     Promise.all([promise, anotherPromise]);
    ///     Promise.allSettled([promise, anotherPromise]);
    ///     Promise.any([promise, anotherPromise]);
    ///     Promise.race([promise, anotherPromise]);
    /// }
    /// ```
    NoAwaitInPromiseMethods,
    unicorn,
    correctness,
    suggestion
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

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if member_expr.optional() || member_expr.is_computed() {
            return;
        }

        let Some(first_argument) = call_expr.arguments[0].as_expression() else {
            return;
        };
        let first_argument = first_argument.without_parentheses();
        let Expression::ArrayExpression(first_argument_array_expr) = first_argument else {
            return;
        };

        for element in &first_argument_array_expr.elements {
            if let Some(element_expr) = element.as_expression()
                && let Expression::AwaitExpression(await_expr) = element_expr.without_parentheses()
            {
                let property_name =
                    member_expr.static_property_name().expect("callee is a static property");
                let await_keyword_span = Span::sized(await_expr.span.start, 5);

                ctx.diagnostic_with_suggestion(
                    no_await_in_promise_methods_diagnostic(await_keyword_span, property_name),
                    |fixer| {
                        let source = fixer.source_text();
                        let after_await = &source[(await_keyword_span.end as usize)..];

                        let trailing_spaces = after_await
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .map(|c| {
                                u32::try_from(c.len_utf8())
                                    .expect("Failed to convert char len usize to u32")
                            })
                            .sum();

                        fixer.delete_range(await_keyword_span.expand_right(trailing_spaces))
                    },
                );
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
        "Promise.all?.([await promise])",
        "Promise?.all([await promise])",
        "Promise.notListedMethod([await promise])",
        "NotPromise.all([await promise])",
        "Promise.all([(await promise, 0)])",
        "new Promise.all([await promise])",
        "globalThis.Promise.all([await promise])",
        r#"Promise["all"]([await promise])"#,
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

    let fix = vec![
        ("Promise.all([await promise])", "Promise.all([promise])"),
        ("Promise.allSettled([await promise])", "Promise.allSettled([promise])"),
        ("Promise.any([await promise])", "Promise.any([promise])"),
        ("Promise.race([await promise])", "Promise.race([promise])"),
        ("Promise.all([, await promise])", "Promise.all([, promise])"),
        ("Promise.all([await promise,])", "Promise.all([promise,])"),
        ("Promise.all([await promise],)", "Promise.all([promise],)"),
        ("Promise.all([await (0, promise)],)", "Promise.all([(0, promise)],)"),
        ("Promise.all([await (( promise ))])", "Promise.all([(( promise ))])"),
        ("Promise.all([await await promise])", "Promise.all([await promise])"),
        (
            "Promise.all([...foo, await promise1, await promise2])",
            "Promise.all([...foo, promise1, promise2])",
        ),
        ("Promise.all([await /* comment*/ promise])", "Promise.all([/* comment*/ promise])"),
    ];

    Tester::new(NoAwaitInPromiseMethods::NAME, NoAwaitInPromiseMethods::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .expect_fix(fix)
        .test_and_snapshot();
}
