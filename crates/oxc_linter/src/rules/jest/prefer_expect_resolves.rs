use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::prefer_expect_resolves::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PreferExpectResolves;

declare_oxc_lint!(PreferExpectResolves, jest, style, fix, docs = DOCUMENTATION, version = "0.2.14",);

impl Rule for PreferExpectResolves {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
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
        (
            "it('is true', async () => { expect(await mockTaskManager.runSoon).toHaveBeenCalledTimes(1); });",
            "it('is true', async () => { await expect(mockTaskManager.runSoon).resolves.toHaveBeenCalledTimes(1); });",
            None,
        ),
    ];

    Tester::new(PreferExpectResolves::NAME, PreferExpectResolves::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
