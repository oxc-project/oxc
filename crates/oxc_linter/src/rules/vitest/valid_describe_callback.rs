use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::valid_describe_callback::{ValidDescribeCallbackOptions, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct ValidDescribeCallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule validates that the second parameter of a `describe()` function is a
    /// callback function. This callback function:
    /// - should not contain any parameters
    /// - should not contain any `return` statements
    ///
    /// Vitest supports async `describe()` callbacks, so this rule allows them.
    ///
    /// ### Why is this bad?
    ///
    /// Using an improper `describe()` callback function can lead to unexpected test
    /// errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // Callback function parameters are not allowed
    /// describe('myFunction()', done => {
    ///   // ...
    /// });
    ///
    /// // Returning a value from a describe block is not allowed
    /// describe('myFunction', () =>
    ///   it('returns a truthy value', () => {
    ///     expect(myFunction()).toBeTruthy();
    /// }));
    /// ```
    ValidDescribeCallback,
    vitest,
    correctness,
    version = "0.0.8",
);

impl Rule for ValidDescribeCallback {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx, ValidDescribeCallbackOptions::VITEST);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
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

    let mut fail = vec![
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

    let pass_vitest = vec![
        ("describe.each([1, 2, 3])(\"%s\", (a, b) => {});", None),
        ("describe.for([1, 2, 3])(\"%s\", (a, b) => {});", None),
        ("describe(\"foo\", function() {})", None),
        ("describe(\"foo\", () => {})", None),
        ("describe(`foo`, () => {})", None),
        ("xdescribe(\"foo\", () => {})", None),
        ("fdescribe(\"foo\", () => {})", None),
        ("describe.only(\"foo\", () => {})", None),
        ("describe.skip(\"foo\", () => {})", None),
        ("describe.todo(\"runPrettierFormat\");", None),
        (
            "
                import { describe } from 'vitest';
                describe.todo(\"runPrettierFormat\");
            ",
            None,
        ),
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
        ("describe(\"foo\", async () => {})", None),
        ("describe(\"foo\", async function () {})", None),
        ("xdescribe(\"foo\", async function () {})", None),
        ("fdescribe(\"foo\", async function () {})", None),
        ("describe.only(\"foo\", async function () {})", None),
        ("describe.skip(\"foo\", async function () {})", None),
        (
            "
                describe('sample case', () => {
                    it('works', () => {
                        expect(true).toEqual(true);
                    });
                    describe('async', async () => {
                        await new Promise(setImmediate);
                        it('works', () => {
                            expect(true).toEqual(true);
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('sample case', () => {
                    it('works', () => {
                        expect(true).toEqual(true);
                    });
                    describe('async', () => {
                        it('works', async () => {
                            await new Promise(setImmediate);
                            expect(true).toEqual(true);
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('FixtureTest', async () => {
                    const files = collectTsFiles('./tests/fixtures');
                    for (const file of files) {
                        const { config } = await import(file);

                        test(file, async () => {
                            await runFixtureTest(config);
                        });
                    }
                });
            ",
            None,
        ),
        (
            "
                describe('foo', { only: true }, () => {
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
                if (hasOwnProperty(obj, key)) {
                }
            ",
            None,
        ),
        (
            "
                describe.each`
                    foo  | foe
                    ${1} | ${2}
                `('$something', ({ foo, foe }) => {});
            ",
            None,
        ),
    ];

    let fail_vitest = vec![
        ("describe.each()()", None),
        ("describe[\"each\"]()()", None),
        ("describe.each(() => {})()", None),
        ("describe.each(() => {})(\"foo\")", None),
        ("describe.each()(() => {})", None),
        ("describe[\"each\"]()(() => {})", None),
        ("describe.each(\"foo\")(() => {})", None),
        ("describe.only.each(\"foo\")(() => {})", None),
        ("describe(() => {})", None),
        ("describe(\"foo\")", None),
        ("describe(\"foo\", \"foo2\")", None),
        ("describe()", None),
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
        (
            "
                describe('foo', () =>
                    test('bar', () => {})
                )
            ",
            None,
        ),
        (
            "
                describe('foo', { only: true }, () =>
                    test('bar', () => {})
                )
            ",
            None,
        ),
        (
            "
                describe('foo', { only: true }, () => {
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
        ("describe(\"foo\", done => {})", None),
        ("describe(\"foo\", function (done) {})", None),
        ("describe(\"foo\", function (one, two, three) {})", None),
        ("describe(\"foo\", async function (done) {})", None),
        ("describe(\"foo\", { only: true }, done => {})", None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(ValidDescribeCallback::NAME, ValidDescribeCallback::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
