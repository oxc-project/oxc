use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    jest_ast_util::{JestFnKind, JestGeneralFnKind, ParsedGeneralJestFnCall},
    rule::Rule
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-confusing-set-timeout)")]
#[diagnostic(severity(warning))]
struct NoGlobalSetTimeoutDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-confusing-set-timeout)")]
#[diagnostic(severity(warning))]
struct NoMultipleSetTimeoutsDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-confusing-set-timeout)")]
#[diagnostic(severity(warning))]
struct NoUnorderSetTimeoutDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoConfusingSetTimeout;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow confusing usages of jest.setTimeout
    ///
    /// ### Why is this bad?
    ///
    /// - being called anywhere other than in global scope
    /// - being called multiple times
    /// - being called after other Jest functions like hooks, `describe`, `test`, or `it`
    ///
    ///
    /// ### Example
    ///
    /// All of these are invalid case:
    /// ```javascript
    /// escribe('test foo', () => {
    ///   jest.setTimeout(1000);
    ///   it('test-description', () => {
    ///     // test logic;
    ///   });
    /// });
    ///
    /// describe('test bar', () => {
    ///   it('test-description', () => {
    ///     jest.setTimeout(1000);
    ///     // test logic;
    ///   });
    /// });
    ///
    /// test('foo-bar', () => {
    ///   jest.setTimeout(1000);
    /// });
    ///
    /// describe('unit test', () => {
    ///   beforeEach(() => {
    ///     jest.setTimeout(1000);
    ///   });
    /// });
    /// ```
    NoConfusingSetTimeout,
    restriction
);

impl Rule for NoConfusingSetTimeout {
    fn run_once(&self, ctx: &LintContext) {
        let mut should_emit_orded_set_timeout = false;
        let mut seen_jest_timeout = false;

        for node in ctx.nodes().iter() {
            let AstKind::CallExpression(call_expr) = node.kind() else {
                continue;
            };

            // let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, node, ctx) else {
            //     continue;
            // };

            // let ParsedGeneralJestFnCall { kind, .. } = jest_fn_call;
            // match kind {
            //     JestFnKind::General(gen_kind) => {
            //     },
            //     _ => {}
            // }

            // // if !is_jest_set_timeout(&jest_fn_call) {
            // //     should_emit_orded_set_timeout = true;
            // //     continue;
            // // }

            // if ctx.scopes().get_flags(node.scope_id()).is_top() {
            //     ctx.diagnostic(NoGlobalSetTimeoutDiagnostic(call_expr.span));
            // }

            // if should_emit_orded_set_timeout {
            //     ctx.diagnostic(NoUnorderSetTimeoutDiagnostic(call_expr.span));
            // }

            // if seen_jest_timeout {
            //     ctx.diagnostic(NoMultipleSetTimeoutsDiagnostic(call_expr.span));
            // } else {
            //     seen_jest_timeout = true;
            // }
        }
    }
}

fn is_jest_set_timeout(jest_fn_call: &ParsedGeneralJestFnCall) -> bool {
    let ParsedGeneralJestFnCall { kind, members, .. } = jest_fn_call;
    matches!(kind, JestFnKind::General(JestGeneralFnKind::Jest))
}

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                jest.setTimeout(1000);
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        // (
        //     "
        //         jest.setTimeout(1000);
        //         window.setTimeout(6000)
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('test foo', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import { handler } from 'dep/mod';
        //         jest.setTimeout(800);
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         function handler() {}
        //         jest.setTimeout(800);
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         const { handler } = require('dep/mod');
        //         jest.setTimeout(800);
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         jest.setTimeout(1000);
        //         window.setTimeout(60000);
        //     ",
        //     None,
        // ),
        // ("window.setTimeout(60000);", None),
        // ("setTimeout(1000);", None),
        // (
        //     "
        //         jest.setTimeout(1000);
        //         test('test case', () => {
        //             setTimeout(() => {
        //             Promise.resolv();
        //             }, 5000);
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         test('test case', () => {
        //             setTimeout(() => {
        //             Promise.resolv();
        //             }, 5000);
        //         });
        //     ",
        //     None,
        // ),
    ];

    let fail = vec![
        // (
        //     "
        //         jest.setTimeout(1000);
        //         setTimeout(1000);
        //         window.setTimeout(1000);
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //         jest.setTimeout(800);
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         describe('A', () => {
        //             jest.setTimeout(800);
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         describe('B', () => {
        //         it('B.1', async () => {
        //             await new Promise((resolve) => {
        //             jest.setTimeout(1000);
        //             setTimeout(resolve, 10000).unref();
        //             });
        //         });
        //         it('B.2', async () => {
        //             await new Promise((resolve) => { setTimeout(resolve, 10000).unref(); });
        //         });
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         test('test-suite', () => {
        //             jest.setTimeout(1000);
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //         jest.setTimeout(1000);
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import { jest } from '@jest/globals';
        //         {
        //            jest.setTimeout(800);
        //         }
        //         describe('A', () => {
        //             beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //             it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
        //         });
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         jest.setTimeout(800);
        //         jest.setTimeout(900);
        //     ",
        //     None
        // ),
        // (
        //     "
        //         expect(1 + 2).toEqual(3);
        //         jest.setTimeout(800);
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import { jest as Jest } from '@jest/globals';
        //         {
        //             Jest.setTimeout(800);
        //         }
        //     ",
        //     None,
        // ),
    ];

    Tester::new(NoConfusingSetTimeout::NAME, pass, fail).test_and_snapshot();
}
