use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::valid_describe_callback::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct ValidDescribeCallback;

declare_oxc_lint!(
    ValidDescribeCallback,
    jest,
    correctness,
    docs = DOCUMENTATION,
    version = "0.0.8",
);

impl Rule for ValidDescribeCallback {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe.each([1, 2, 3])('%s', (a, b) => {});", None),
        ("describe('foo', function() {})", None),
        ("describe('foo', () => {})", None),
        ("describe(`foo`, () => {})", None),
        ("xdescribe('foo\', () => {})", None),
        ("fdescribe('foo', () => {})", None),
        ("describe.only('foo', () => {})", None),
        ("describe.skip('foo', () => {})", None),
        (
            "
            describe('foo', () => {
                it('bar', () => {
                    return Promise.resolve(42).then(value => {
                        expect(value).toBe(42)
                    })
                })
            })
            ",
            None,
        ),
        (
            "
            describe('foo', () => {
                it('bar', async () => {
                    expect(await Promise.resolve(42)).toBe(42)
                })
            })
            ",
            None,
        ),
        ("if (hasOwnProperty(obj, key)) {}", None),
        (
            "
            describe.each`
                foo  | foe
                ${'1'} | ${'2'}
            `('$something', ({ foo, foe }) => {});
            ",
            None,
        ),
    ];

    let fail = vec![
        ("describe.each()()", None),
        ("describe['each']()()", None),
        ("describe.each(() => {})()", None),
        ("describe.each(() => {})('foo')", None),
        ("describe.each()(() => {})", None),
        ("describe['each']()(() => {})", None),
        ("describe.each('foo')(() => {})", None),
        ("describe.only.each('foo')(() => {})", None),
        ("describe(() => {})", None),
        ("describe('foo')", None),
        ("describe('foo', 'foo2')", None),
        ("describe()", None),
        ("describe('foo', async () => {})", None),
        ("describe('foo', async function () {})", None),
        ("xdescribe('foo', async function () {})", None),
        ("fdescribe('foo', async function () {})", None),
        (
            "
            import { fdescribe } from '@jest/globals';
            fdescribe('foo', async function () {})
            ",
            None,
        ),
        ("describe.only('foo', async function () {})", None),
        ("describe.skip('foo', async function () {})", None),
        (
            "
            describe('sample case', () => {
                it('works', () => {
                    expect(true).toEqual(true);
                });
                describe('async', async () => {
                    await new Promise(setImmediate);
                    it('breaks', () => {
                        throw new Error('Fail');
                    });
                });
            });
            ",
            None,
        ),
        (
            "
            describe('foo', function () {
                return Promise.resolve().then(() => {
                    it('breaks', () => {
                        throw new Error('Fail')
                    })
                })
            })
            ",
            None,
        ),
        (
            "
            describe('foo', () => {
                return Promise.resolve().then(() => {
                    it('breaks', () => {
                        throw new Error('Fail')
                    })
                })
                describe('nested', () => {
                    return Promise.resolve().then(() => {
                        it('breaks', () => {
                            throw new Error('Fail')
                        })
                    })
                })
            })
            ",
            None,
        ),
        (
            "
            describe('foo', async () => {
                await something()
                it('does something')
                describe('nested', () => {
                    return Promise.resolve().then(() => {
                        it('breaks', () => {
                            throw new Error('Fail')
                        })
                    })
                })
            })
            ",
            None,
        ),
        ("describe('foo', () => test('bar', () => {})) ", None),
        ("describe('foo', done => {})", None),
        ("describe('foo', function (done) {})", None),
        ("describe('foo', function (one, two, three) {})", None),
        ("describe('foo', async function (done) {})", None),
    ];

    Tester::new(ValidDescribeCallback::NAME, ValidDescribeCallback::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
