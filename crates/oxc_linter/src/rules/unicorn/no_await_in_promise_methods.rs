use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::outermost_paren_parent, context::LintContext, rule::Rule};

fn no_await_in_promise_methods_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Promise in `Promise.{method_name}()` should not be awaited."))
        .with_help("Remove the `await`")
        .with_label(span)
}

fn nested_await_in_promise_methods_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Avoid `await` inside `Promise.{method_name}()` arguments to prevent delaying the call."
    ))
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
    /// mistake and delays starting the other promises concurrently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async function foo() {
    ///     Promise.all([await promise, anotherPromise]);
    ///     Promise.all([(await promise).foo, anotherPromise]);
    ///     Promise.all([condition ? await promise : fallbackPromise, anotherPromise]);
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
    ///     Promise.all([promise.then(value => value.foo), anotherPromise]);
    ///     Promise.all([condition ? promise : fallbackPromise, anotherPromise]);
    ///     Promise.allSettled([promise, anotherPromise]);
    ///     Promise.any([promise, anotherPromise]);
    ///     Promise.race([promise, anotherPromise]);
    /// }
    /// ```
    NoAwaitInPromiseMethods,
    unicorn,
    correctness,
    recommended,
    conditional_suggestion,
    version = "0.2.18",
    short_description = "Disallow using `await` in `Promise` method parameters.",
);

impl Rule for NoAwaitInPromiseMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AwaitExpression(await_expr) = node.kind() else {
            return;
        };

        let Some((property_name, is_direct_element)) = find_enclosing_promise_method(node, ctx)
        else {
            return;
        };

        let await_keyword_span = Span::sized(await_expr.span.start, 5);
        if !is_direct_element {
            ctx.diagnostic(nested_await_in_promise_methods_diagnostic(
                await_keyword_span,
                property_name,
            ));
            return;
        }

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

fn promise_method_name<'a>(
    array_node: &AstNode<'a>,
    array_span: Span,
    ctx: &LintContext<'a>,
) -> Option<&'static str> {
    if let AstKind::CallExpression(call_expr) = outermost_paren_parent(array_node, ctx)?.kind()
        && !call_expr.optional
        && call_expr.arguments.len() == 1
        && let Some(first_arg) = call_expr.arguments[0].as_expression()
        && first_arg.without_parentheses().span() == array_span
        && let Expression::StaticMemberExpression(member_expr) =
            call_expr.callee.without_parentheses()
        && !member_expr.optional
        && let Expression::Identifier(obj) = member_expr.object.without_parentheses()
        && obj.name == "Promise"
    {
        match member_expr.property.name.as_str() {
            "all" => Some("all"),
            "allSettled" => Some("allSettled"),
            "any" => Some("any"),
            "race" => Some("race"),
            _ => None,
        }
    } else {
        None
    }
}

fn find_enclosing_promise_method<'a>(
    await_node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<(&'static str, bool)> {
    let mut is_direct_element = true;
    for parent in ctx.nodes().ancestors(await_node.id()) {
        match parent.kind() {
            AstKind::Program(_)
            | AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::AwaitExpression(_) => return None,
            AstKind::ParenthesizedExpression(_) => {}
            AstKind::ArrayExpression(array_expr) => {
                if let Some(method_name) = promise_method_name(parent, array_expr.span, ctx) {
                    return Some((method_name, is_direct_element));
                }
                is_direct_element = false;
            }
            _ => {
                is_direct_element = false;
            }
        }
    }
    None
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
        "new Promise.all([await promise])",
        "globalThis.Promise.all([await promise])",
        r#"Promise["all"]([await promise])"#,
        "Promise.all([(async () => await promise)()])",
        "Promise.all([(async () => (await fetch(url)).json())(), ...otherPromises])",
        "Promise.all([async function () { await promise }])",
        "Promise.all([async function * () { await promise }])",
        "Promise.all([foo.then(async value => await value)])",
        "Promise.all([class { async foo() { await bar } }])",
        "Promise.all([class { static async foo() { await bar } }])",
        "Promise.all([class { foo = async () => await bar }])",
        "Promise.all([{async foo() { await bar }}])",
        "Promise.all([new Promise(async resolve => resolve(await promise))])",
        "Promise.all([promise1, promise2].map(async promise => (await promise).foo))",
        "await Promise.all([promise])",
        "Promise.all([await promise].map(foo))",
        "Promise.all(await promises)",
        "Promise.all([promise]).then(await foo)",
        "foo([await promise])",
        "foo([(await promise).bar])",
        "(Promise?.all)([await promise])",
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
        "Promise.all([(await promise, 0)])",
        "Promise.all([(await promise).foo, anotherPromise])",
        "Promise.all([condition ? await promise : fallbackPromise, anotherPromise])",
        "Promise.all([foo && await promise, anotherPromise])",
        "Promise.all([foo ?? await bar])",
        "Promise.all([fn(await promise)])",
        "Promise.all([...await promise])",
        "Promise.all([{foo: await promise}])",
        "Promise.all([[await promise]])",
        "Promise.all([(await promise1) + (await promise2)])",
        "Promise.all([await (await promise).foo])",
        "Promise.all([await foo(await promise)])",
        "Promise.all([Promise.all([await promise]), anotherPromise])",
        "Promise.all([await Promise.all([await promise])])",
        "Promise.all([async () => {}, (await promise).foo])",
        "Promise.all([(await promise)?.foo, anotherPromise])",
        "Promise.all([`${await promise}`, anotherPromise])",
        "Promise.all([tag`${await promise}`, anotherPromise])",
        "Promise.all([{[await key]: value}])",
        "Promise.all([{async [await key]() {}}])",
        "Promise.all([class { [await key]() {} }])",
        "Promise.all([class extends (await Base) {}])",
        "Promise.all([foo([await promise])])",
        "Promise.all([Promise.all(await promises), anotherPromise])",
        "Promise.allSettled([(await promise).foo, anotherPromise])",
        "Promise.any([(await promise).foo, anotherPromise])",
        "Promise.race([(await promise).foo, anotherPromise])",
        "Promise.all([await promise1, (await promise2).foo])",
        "Promise.all([((await promise)).foo])",
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
        (
            "Promise.all([(await promise).foo, anotherPromise])",
            "Promise.all([(await promise).foo, anotherPromise])",
        ),
        (
            "Promise.all([condition ? await promise : fallbackPromise, anotherPromise])",
            "Promise.all([condition ? await promise : fallbackPromise, anotherPromise])",
        ),
        (
            "Promise.all([await promise1, (await promise2).foo])",
            "Promise.all([promise1, (await promise2).foo])",
        ),
    ];

    Tester::new(NoAwaitInPromiseMethods::NAME, NoAwaitInPromiseMethods::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .expect_fix(fix)
        .test_and_snapshot();

    let tsx_pass = vec![
        "async function f() { Promise.all([(async () => (await promise as Foo).bar)()]); }",
        "async function f() { (Promise as any).all([await promise]); }",
    ];
    let tsx_fail = vec![
        "async function f() { Promise.all([(await promise as Foo).bar]); }",
        "async function f() { Promise.all([(await promise)!]); }",
        "async function f() { Promise.all([await promise!]); }",
        "async function f() { Promise.all([await promise as Foo]); }",
        "async function f() { Promise.all([await promise satisfies Foo]); }",
        "async function f() { Promise.all([<div>{await promise}</div>, anotherPromise]); }",
        "async function f() { Promise.all([<Foo bar={await promise} />, anotherPromise]); }",
    ];
    let tsx_fix = vec![
        (
            "async function f() { Promise.all([await promise!]); }",
            "async function f() { Promise.all([promise!]); }",
        ),
        (
            "async function f() { Promise.all([await promise as Foo]); }",
            "async function f() { Promise.all([await promise as Foo]); }",
        ),
        (
            "async function f() { Promise.all([(await promise)!]); }",
            "async function f() { Promise.all([(await promise)!]); }",
        ),
    ];
    Tester::new(NoAwaitInPromiseMethods::NAME, NoAwaitInPromiseMethods::PLUGIN, tsx_pass, tsx_fail)
        .expect_fix(tsx_fix)
        .test();
}
