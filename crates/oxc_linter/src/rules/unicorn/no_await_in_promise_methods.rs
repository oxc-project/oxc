use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{
    AstNode,
    ast_util::{is_method_call, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
};

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
    /// Disallow using `await` in `Promise` method parameters.
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
    recommended,
    suggestion,
    version = "0.2.18",
    short_description = "Disallow using `await` in `Promise` method parameters.",
);

impl Rule for NoAwaitInPromiseMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AwaitExpression(await_expr) = node.kind() else {
            return;
        };
        let Some(array_node) = outermost_paren_parent(node, ctx) else {
            return;
        };
        let AstKind::ArrayExpression(array_expr) = array_node.kind() else {
            return;
        };
        let Some(call_node) = outermost_paren_parent(array_node, ctx) else {
            return;
        };
        let AstKind::CallExpression(call_expr) = call_node.kind() else {
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
        if first_argument.without_parentheses().span() != array_expr.span {
            return;
        }
        let property_name = member_expr
            .static_property_name()
            .and_then(JSStr::as_str)
            .expect("callee is a static property");
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
        "Promise.all([...await promise])",
        "Promise.all([[await promise]])",
        "Promise.all([fn(await promise)])",
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
        ("Promise.all(([(await promise)]))", "Promise.all(([(promise)]))"),
    ];

    Tester::new(NoAwaitInPromiseMethods::NAME, NoAwaitInPromiseMethods::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .expect_fix(fix)
        .test_and_snapshot();
}
