use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_unneeded_async_expect_function::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoUnneededAsyncExpectFunction;

declare_oxc_lint!(
    NoUnneededAsyncExpectFunction,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.39.0",
);

impl Rule for NoUnneededAsyncExpectFunction {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        "expect.hasAssertions()",
        "
                it('pass', async () => {
                  expect();
                })
                ",
        "
                it('pass', async () => {
                  await expect(doSomethingAsync()).rejects.toThrow();
                })
                ",
        "
                it('pass', async () => {
                  await expect(doSomethingAsync(1, 2)).resolves.toBe(1);
                })
                ",
        "
                it('pass', async () => {
                  await expect(async () => {
                    await doSomethingAsync();
                    await doSomethingTwiceAsync(1, 2);
                  }).rejects.toThrow();
                })
                ",
        "
                it('pass', async () => {
                  await expect(async () => {
                    doSomethingAsync();
                  }).rejects.toThrow();
                })
                ",
        "
                it('pass', async () => {
                  await expect(async () => {
                    const a = 1;
                    await doSomethingAsync(a);
                  }).rejects.toThrow();
                })
                ",
        "
                it('pass for non-async expect', async () => {
                  await expect(() => {
                    doSomethingSync(a);
                  }).rejects.toThrow();
                })
                ",
        "
                it('pass for await in expect', async () => {
                  await expect(await doSomethingAsync()).rejects.toThrow();
                })
                ",
        "
                it('pass for different matchers', async () => {
                  await expect(await doSomething()).not.toThrow();
                  await expect(await doSomething()).toHaveLength(2);
                  await expect(await doSomething()).toHaveReturned();
                  await expect(await doSomething()).not.toHaveBeenCalled();
                  await expect(await doSomething()).not.toBeDefined();
                  await expect(await doSomething()).toEqual(2);
                })
                ",
        "
                it('pass for using await within for-loop', async () => {
                  const b = [async () => Promise.resolve(1), async () => Promise.reject(2)];
                  await expect(async() => {
                    for (const a of b) {
                      await b();
                    }
                  }).rejects.toThrow();
                })
                ",
        "
                it('pass for using await within array', async () => {
                  await expect(async() => [await Promise.reject(2)]).rejects.toThrow(2);
                })
                ",
        "
                import { expect as pleaseExpect } from '@jest/globals';
                it('pass', async () => {
                await pleaseExpect(doSomethingAsync()).rejects.toThrow();
                })",
    ];

    let fail = vec![
        "
                  it('should be fixed', async () => {
                    await expect(async () => {
                      await doSomethingAsync();
                    }).rejects.toThrow();
                  })
                  ",
        "
                  it('should be fixed', async () => {
                    await expect(async () => await doSomethingAsync()).rejects.toThrow();
                  })
                  ",
        "
                  it('should be fixed', async () => {
                    await expect(async function () {
                      await doSomethingAsync();
                    }).rejects.toThrow();
                  })
                  ",
        "
                    it('should be fixed for async arrow function', async () => {
                      await expect(async () => {
                        await doSomethingAsync(1, 2);
                      }).rejects.toThrow();
                    })
                  ",
        "
                    it('should be fixed for async normal function', async () => {
                      await expect(async function () {
                        await doSomethingAsync(1, 2);
                      }).rejects.toThrow();
                    })
                  ",
        "
                    it('should be fixed for Promise.all', async () => {
                      await expect(async function () {
                        await Promise.all([doSomethingAsync(1, 2), doSomethingAsync()]);
                      }).rejects.toThrow();
                    })
                  ",
        "
                    it('should be fixed for async ref to expect', async () => {
                      const a = async () => { await doSomethingAsync() };
                      await expect(async () => {
                        await a();
                      }).rejects.toThrow();
                    })
                  ",
    ];

    let fix = vec![
        (
            "
                  it('should be fixed', async () => {
                    await expect(async () => {
                      await doSomethingAsync();
                    }).rejects.toThrow();
                  })
                  ",
            "
                  it('should be fixed', async () => {
                    await expect(doSomethingAsync()).rejects.toThrow();
                  })
                  ",
            None,
        ),
        (
            "
                  it('should be fixed', async () => {
                    await expect(async () => await doSomethingAsync()).rejects.toThrow();
                  })
                  ",
            "
                  it('should be fixed', async () => {
                    await expect(doSomethingAsync()).rejects.toThrow();
                  })
                  ",
            None,
        ),
        (
            "
                  it('should be fixed', async () => {
                    await expect(async function () {
                      await doSomethingAsync();
                    }).rejects.toThrow();
                  })
                  ",
            "
                  it('should be fixed', async () => {
                    await expect(doSomethingAsync()).rejects.toThrow();
                  })
                  ",
            None,
        ),
        (
            "
                    it('should be fixed for async arrow function', async () => {
                      await expect(async () => {
                        await doSomethingAsync(1, 2);
                      }).rejects.toThrow();
                    })
                  ",
            "
                    it('should be fixed for async arrow function', async () => {
                      await expect(doSomethingAsync(1, 2)).rejects.toThrow();
                    })
                  ",
            None,
        ),
        (
            "
                    it('should be fixed for async normal function', async () => {
                      await expect(async function () {
                        await doSomethingAsync(1, 2);
                      }).rejects.toThrow();
                    })
                  ",
            "
                    it('should be fixed for async normal function', async () => {
                      await expect(doSomethingAsync(1, 2)).rejects.toThrow();
                    })
                  ",
            None,
        ),
        (
            "
                    it('should be fixed for Promise.all', async () => {
                      await expect(async function () {
                        await Promise.all([doSomethingAsync(1, 2), doSomethingAsync()]);
                      }).rejects.toThrow();
                    })
                  ",
            "
                    it('should be fixed for Promise.all', async () => {
                      await expect(Promise.all([doSomethingAsync(1, 2), doSomethingAsync()])).rejects.toThrow();
                    })
                  ",
            None,
        ),
        (
            "
                    it('should be fixed for async ref to expect', async () => {
                      const a = async () => { await doSomethingAsync() };
                      await expect(async () => {
                        await a();
                      }).rejects.toThrow();
                    })
                  ",
            "
                    it('should be fixed for async ref to expect', async () => {
                      const a = async () => { await doSomethingAsync() };
                      await expect(a()).rejects.toThrow();
                    })
                  ",
            None,
        ),
    ];

    let pass_vitest = vec![
        "
                import { expect as pleaseExpect } from 'vitest';
                it('pass', async () => {
                  await pleaseExpect(doSomethingAsync()).rejects.toThrow();
                })
                ",
    ];

    pass.extend(pass_vitest);

    Tester::new(
        NoUnneededAsyncExpectFunction::NAME,
        NoUnneededAsyncExpectFunction::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
