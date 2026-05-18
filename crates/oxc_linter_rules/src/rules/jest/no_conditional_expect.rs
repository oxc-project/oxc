use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_conditional_expect::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoConditionalExpect;

declare_oxc_lint!(NoConditionalExpect, jest, correctness, docs = DOCUMENTATION, version = "0.0.12",);

impl Rule for NoConditionalExpect {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(possible_jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                it('foo', () => {
                    expect(1).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    expect(!true).toBe(false);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    process.env.FAIL && setNum(1);
                    expect(num).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                function getValue() {
                    let num = 2;
                    process.env.FAIL && setNum(1);
                    return num;
                }
                it('foo', () => {
                expect(getValue()).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    process.env.FAIL || setNum(1);
                    expect(num).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                function getValue() {
                    let num = 2;
                    process.env.FAIL || setNum(1);
                    return num;
                }
                it('foo', () => {
                    expect(getValue()).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    const num = process.env.FAIL ? 1 : 2;
                    expect(num).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                function getValue() {
                    return process.env.FAIL ? 1 : 2
                }

                it('foo', () => {
                    expect(getValue()).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    let num;

                    switch(process.env.FAIL) {
                        case true:
                        num = 1;
                        break;
                        case false:
                        num = 2;
                        break;
                    }

                    expect(num).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                function getValue() {
                    switch(process.env.FAIL) {
                    case true:
                        return 1;
                    case false:
                        return 2;
                    }
                }

                it('foo', () => {
                    expect(getValue()).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    let num = 2;

                    if(process.env.FAIL) {
                        num = 1;
                    }

                    expect(num).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                function getValue() {
                    if(process.env.FAIL) {
                        return 1;
                    }
                    return 2;
                }

                it('foo', () => {
                    expect(getValue()).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    try {
                        // do something
                    } catch {
                        // ignore errors
                    } finally {
                        expect(something).toHaveBeenCalled();
                    }
                });
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    try {
                        // do something
                    } catch {
                        // ignore errors
                    }

                    expect(something).toHaveBeenCalled();
                });
            ",
            None,
        ),
        (
            "
                function getValue() {
                    try {
                        // do something
                    } catch {
                        // ignore errors
                    } finally {
                        expect(something).toHaveBeenCalled();
                    }
                }
                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                function getValue() {
                    try {
                        process.env.FAIL.toString();

                        return 1;
                    } catch {
                        return 2;
                    }
                }

                it('foo', () => {
                    expect(getValue()).toBe(2);
                });
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    try {
                        await Promise.resolve().then(() => {
                        throw new Error('oh noes!');
                        });
                    } catch {
                        // ignore errors
                    } finally {
                        expect(something).toHaveBeenCalled();
                    }
                });
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    await doSomething().catch(error => error);

                    expect(error).toBeInstanceOf(Error);
                });
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    try {
                        await Promise.resolve().then(() => {
                            throw new Error('oh noes!');
                        });
                    } catch {
                        // ignore errors
                    }

                    expect(something).toHaveBeenCalled();
                });
            ",
            None,
        ),
        (
            "function verifyVNodeTree(vnode) {
                    if (vnode._nextDom) {
                        expect.fail('vnode should not have _nextDom:' + vnode._nextDom);
                    }

                    if (vnode._children) {
                        for (let child of vnode._children) {
                            if (child) {
                                verifyVNodeTree(child);
                            }
                        }
                    }
                }",
            None,
        ),
        (
            "it('throws an error', async () => {
                await expect(foo).rejects.toThrow(Error);
            });",
            None,
        ),
    ];

    let fail = vec![
        (
            "
                it('foo', () => {
                    something && expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    a || b && expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    (a || b) && expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    a || (b && expect(something).toHaveBeenCalled());
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    a && b && expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    a && b || expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    (a && b) || expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    something && expect(something).toHaveBeenCalled();
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    something || expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it.each``('foo', () => {
                    something || expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it.each()('foo', () => {
                    something || expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    something || expect(something).toHaveBeenCalled();
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    something ? expect(something).toHaveBeenCalled() : noop();
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    something ? expect(something).toHaveBeenCalled() : noop();
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    something ? noop() : expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it.each``('foo', () => {
                    something ? noop() : expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                it.each()('foo', () => {
                    something ? noop() : expect(something).toHaveBeenCalled();
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    something ? noop() : expect(something).toHaveBeenCalled();
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    switch(something) {
                    case 'value':
                        break;
                    default:
                        expect(something).toHaveBeenCalled();
                    }
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    switch(something) {
                    case 'value':
                        expect(something).toHaveBeenCalled();
                    default:
                        break;
                    }
                })
            ",
            None,
        ),
        (
            "
                it.each``('foo', () => {
                    switch(something) {
                    case 'value':
                        expect(something).toHaveBeenCalled();
                    default:
                        break;
                    }
                })
            ",
            None,
        ),
        (
            "
                it.each()('foo', () => {
                    switch(something) {
                    case 'value':
                        expect(something).toHaveBeenCalled();
                    default:
                        break;
                    }
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    switch(something) {
                    case 'value':
                        break;
                    default:
                        expect(something).toHaveBeenCalled();
                    }
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    if(doSomething) {
                        expect(something).toHaveBeenCalled();
                    }
                })
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    if(!doSomething) {
                        // do nothing
                    } else {
                        expect(something).toHaveBeenCalled();
                    }
                })
            ",
            None,
        ),
        (
            "
                it.each``('foo', () => {
                    if(!doSomething) {
                        // do nothing
                    } else {
                        expect(something).toHaveBeenCalled();
                    }
                })
            ",
            None,
        ),
        (
            "
                it.each()('foo', () => {
                    if(!doSomething) {
                        // do nothing
                    } else {
                        expect(something).toHaveBeenCalled();
                    }
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    if(doSomething) {
                        expect(something).toHaveBeenCalled();
                    }
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                function getValue() {
                    if(!doSomething) {
                    // do nothing
                    } else {
                    expect(something).toHaveBeenCalled();
                    }
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('foo', () => {
                    try {

                    } catch (err) {
                    expect(err).toMatch('Error');
                    }
                })
            ",
            None,
        ),
        (
            "
                it.each``('foo', () => {
                    try {

                    } catch (err) {
                    expect(err).toMatch('Error');
                    }
                })
            ",
            None,
        ),
        (
            "
                it.each()('foo', () => {
                    try {

                    } catch (err) {
                        expect(err).toMatch('Error');
                    }
                })
            ",
            None,
        ),
        (
            "
                it.skip.each``('foo', () => {
                    try {

                    } catch (err) {
                        expect(err).toMatch('Error');
                    }
                })
            ",
            None,
        ),
        (
            "
                it.skip.each()('foo', () => {
                    try {

                    } catch (err) {
                        expect(err).toMatch('Error');
                    }
                })
            ",
            None,
        ),
        (
            "
                function getValue() {
                    try {
                    // do something
                    } catch {
                    expect(something).toHaveBeenCalled();
                    }
                }

                it('foo', getValue);
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    await Promise.resolve()
                    .then(() => { throw new Error('oh noes!'); })
                    .catch(error => expect(error).toBeInstanceOf(Error));
                });
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    await Promise.resolve()
                    .then(() => { throw new Error('oh noes!'); })
                    .catch(error => expect(error).toBeInstanceOf(Error))
                    .then(() => { throw new Error('oh noes!'); })
                    .catch(error => expect(error).toBeInstanceOf(Error))
                    .then(() => { throw new Error('oh noes!'); })
                    .catch(error => expect(error).toBeInstanceOf(Error));
                });
            ",
            None,
        ),
        (
            "
			        it('works', async () => {
			          await Promise.resolve()
			            .catch(error => expect(error).toBeInstanceOf(Error))
			            .catch(error => expect(error).toBeInstanceOf(Error))
			            .catch(error => expect(error).toBeInstanceOf(Error));
			        });
			      ",
            None,
        ),
        (
            "
                it('works', async () => {
                    await Promise.resolve()
                    .catch(error => expect(error).toBeInstanceOf(Error))
                    .then(() => { throw new Error('oh noes!'); })
                    .then(() => { throw new Error('oh noes!'); })
                    .then(() => { throw new Error('oh noes!'); });
                });
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    await somePromise
                    .then(() => { throw new Error('oh noes!'); })
                    .catch(error => expect(error).toBeInstanceOf(Error));
                });
            ",
            None,
        ),
        (
            "
                it('works', async () => {
                    await somePromise.catch(error => expect(error).toBeInstanceOf(Error));
                });
            ",
            None,
        ),
        (
            "
            it('works', async () => {
                verifyVNodeTree(vnode);

                function verifyVNodeTree(vnode) {
                    if (vnode._nextDom) {
                        expect.fail('vnode should not have _nextDom:' + vnode._nextDom);
                    }

                    if (vnode._children) {
                        for (let child of vnode._children) {
                            if (child) {
                                    verifyVNodeTree(child);
                            }
                        }
                    }
                }
            });
            ",
            None,
        ),
    ];

    Tester::new(NoConditionalExpect::NAME, NoConditionalExpect::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
