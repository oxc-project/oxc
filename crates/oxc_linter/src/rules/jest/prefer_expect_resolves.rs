use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn expect_resolves(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `await expect(...).resolves` over `expect(await ...)` syntax.")
        .with_help("Use `await expect(...).resolves` instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferExpectResolves;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `await expect(...).resolves` over `expect(await ...)` when testing
    /// promises.
    ///
    /// ### Why is this bad?
    ///
    /// When working with promises, there are two primary ways you can test the
    /// resolved value:
    /// 1. use the `resolve` modifier on `expect`
    /// (`await expect(...).resolves.<matcher>` style)
    /// 2. `await` the promise and assert against its result
    /// (`expect(await ...).<matcher>` style)
    ///
    /// While the second style is arguably less dependent on `jest`, if the
    /// promise rejects it will be treated as a general error, resulting in less
    /// predictable behaviour and output from `jest`.
    ///
    /// Additionally, favoring the first style ensures consistency with its
    /// `rejects` counterpart, as there is no way of "awaiting" a rejection.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// it('passes', async () => {
    ///     expect(await someValue()).toBe(true);
    /// });
    /// it('is true', async () => {
    ///     const myPromise = Promise.resolve(true);
    ///     expect(await myPromise).toBe(true);
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// it('passes', async () => {
    ///     await expect(someValue()).resolves.toBe(true);
    /// });
    /// it('is true', async () => {
    ///     const myPromise = Promise.resolve(true);
    ///
    ///     await expect(myPromise).resolves.toBe(true);
    /// });
    /// it('errors', async () => {
    ///     await expect(Promise.reject(new Error('oh noes!'))).rejects.toThrowError(
    ///         'oh noes!',
    ///     );
    /// });
    /// ```
    PreferExpectResolves,
    jest,
    style,
    fix
);

impl Rule for PreferExpectResolves {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_expect_fn_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Some(Expression::CallExpression(call_expr)) = jest_expect_fn_call.head.parent else {
            return;
        };
        let Some(argument) = call_expr.arguments.first() else {
            return;
        };
        let Argument::AwaitExpression(await_expr) = argument else {
            return;
        };
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };
        ctx.diagnostic_with_fix(expect_resolves(await_expr.span), |fixer| {
            let offset = match &await_expr.argument {
                Expression::CallExpression(call_expr) => call_expr.span.start - ident.span.end,
                Expression::Identifier(promise_ident) => promise_ident.span.start - ident.span.end,
                _ => 0,
            };
            let arg_span = Span::new(
                call_expr.span.start + (ident.span.end - ident.span.start) + offset,
                await_expr.span.end,
            );
            let local = jest_expect_fn_call.local.as_ref();
            let argument = fixer.source_range(arg_span);
            let mut code = String::with_capacity(local.len() + argument.len() + 17);
            code.push_str("await ");
            code.push_str(local);
            code.push('(');
            code.push_str(argument);
            code.push_str(").resolves");
            fixer.replace(call_expr.span, code)
        });
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

    let pass = vec![
        ("expect.hasAssertions()", None),
        (
            "
                it('passes', async () => {
                    await expect(someValue()).resolves.toBe(true);
                });
            ",
            None,
        ),
        (
            "
                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    await expect(myPromise).resolves.toBe(true);
                });
            ",
            None,
        ),
        (
            "
                it('errors', async () => {
                    await expect(Promise.reject(new Error('oh noes!'))).rejects.toThrowError(
                        'oh noes!',
                    );
                });
            ",
            None,
        ),
        ("expect().nothing();", None),
    ];

    let fail = vec![
        (
            "
                it('passes', async () => {
                    expect(await someValue(),).toBe(true);
                });
            ",
            None,
        ),
        (
            "
                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    expect(await myPromise).toBe(true);
                });
            ",
            None,
        ),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';

                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    pleaseExpect(await myPromise).toBe(true);
                });
            ",
            None,
        ),
    ];

    let fix = vec![
        (
            "
                it('passes', async () => {
                    expect(await someValue(),).toBe(true);
                });
            ",
            "
                it('passes', async () => {
                    await expect(someValue()).resolves.toBe(true);
                });
            ",
            None,
        ),
        (
            "
                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    expect(await myPromise).toBe(true);
                });
            ",
            "
                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    await expect(myPromise).resolves.toBe(true);
                });
            ",
            None,
        ),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';

                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    pleaseExpect(await myPromise).toBe(true);
                });
            ",
            "
                import { expect as pleaseExpect } from '@jest/globals';

                it('is true', async () => {
                    const myPromise = Promise.resolve(true);
                    await pleaseExpect(myPromise).resolves.toBe(true);
                });
            ",
            None,
        ),
    ];

    Tester::new(PreferExpectResolves::NAME, PreferExpectResolves::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
