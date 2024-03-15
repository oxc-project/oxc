use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::Fix,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_expect_jest_fn_call, ParsedExpectFnCall,
        PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-expect-resolves): Prefer `await expect(...).resolves` over `expect(await ...)` syntax.")]
#[diagnostic(severity(warning), help("Use `await expect(...).resolves` instead"))]
struct ExpectResolves(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferExpectResolves;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When working with promises, there are two primary ways you can test the resolved
    /// value:
    /// 1. use the `resolve` modifier on `expect`
    /// (`await expect(...).resolves.<matcher>` style)
    /// 2. `await` the promise and assert against its result
    /// (`expect(await ...).<matcher>` style)
    ///
    /// While the second style is arguably less dependent on `jest`, if the promise
    /// rejects it will be treated as a general error, resulting in less predictable
    /// behaviour and output from `jest`.
    ///
    /// Additionally, favoring the first style ensures consistency with its `rejects`
    /// counterpart, as there is no way of "awaiting" a rejection.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // valid
    /// it('passes', async () => {
    ///     await expect(someValue()).resolves.toBe(true);
    /// });
    /// it('is true', async () => {
    ///     const myPromise = Promise.resolve(true);
    ///
    ///     await expect(myPromise).resolves.toBe(true);
    /// });
    ///
    /// it('errors', async () => {
    ///     await expect(Promise.reject(new Error('oh noes!'))).rejects.toThrowError(
    ///         'oh noes!',
    ///     );
    /// });
    ///
    /// // invalid
    /// it('passes', async () => {
    ///     expect(await someValue()).toBe(true);
    /// });
    /// it('is true', async () => {
    ///     const myPromise = Promise.resolve(true);
    ///     expect(await myPromise).toBe(true);
    /// });
    /// ```
    PreferExpectResolves,
    style,
);

impl Rule for PreferExpectResolves {
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            Self::run(possible_jest_node, ctx);
        }
    }
}

impl PreferExpectResolves {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
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
        let Argument::Expression(Expression::AwaitExpression(await_expr)) = argument else {
            return;
        };
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        ctx.diagnostic_with_fix(ExpectResolves(await_expr.span), || {
            let content = Self::build_code(&jest_expect_fn_call, call_expr, ident.span, ctx);
            Fix::new(content, call_expr.span)
        });
    }

    fn build_code<'a>(
        jest_expect_fn_call: &ParsedExpectFnCall,
        call_expr: &CallExpression<'a>,
        ident_span: Span,
        ctx: &LintContext<'a>,
    ) -> String {
        let mut formatter = ctx.codegen();
        let first = call_expr.arguments.first().unwrap();
        let Argument::Expression(Expression::AwaitExpression(await_expr)) = first else {
            return formatter.into_source_text();
        };

        let offset = match &await_expr.argument {
            Expression::CallExpression(call_expr) => call_expr.span.start - ident_span.end,
            Expression::Identifier(promise_ident) => promise_ident.span.start - ident_span.end,
            _ => 0,
        };

        let arg_span = Span::new(
            call_expr.span.start + (ident_span.end - ident_span.start) + offset,
            call_expr.span.end,
        );

        formatter.print_str(b"await");
        formatter.print_hard_space();
        formatter.print_str(jest_expect_fn_call.local.as_bytes());
        formatter.print(b'(');
        formatter.print_str(arg_span.source_text(ctx.source_text()).as_bytes());
        formatter.print_str(b".resolves");
        formatter.into_source_text()
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

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
                    await expect(someValue(),).resolves.toBe(true);
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

    Tester::new(PreferExpectResolves::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
