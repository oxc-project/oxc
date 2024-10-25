use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        is_type_of_jest_fn_call, parse_expect_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn no_conditional_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected conditional expect")
        .with_help("Avoid calling `expect` conditionally`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConditionalExpect;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents the use of expect in conditional blocks, such as ifs & catch(s).
    /// This includes using expect in callbacks to functions named catch, which are assumed to be promises.
    ///
    /// ### Why is this bad?
    ///
    /// Jest only considers a test to have failed if it throws an error, meaning if calls to assertion functions like expect occur in conditional code such as a catch statement, tests can end up passing but not actually test anything.
    /// Additionally, conditionals tend to make tests more brittle and complex, as they increase the amount of mental thinking needed to understand what is actually being tested.
    ///
    /// ### Example
    /// ```javascript
    /// it('foo', () => {
    ///   doTest && expect(1).toBe(2);
    /// });
    ///
    /// it('bar', () => {
    ///   if (!skipTest) {
    ///     expect(1).toEqual(2);
    ///   }
    /// });
    ///
    /// it('throws an error', async () => {
    ///   await foo().catch(error => expect(error).toBeInstanceOf(error));
    /// });
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/no-conditional-expect.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-conditional-expect": "error"
    ///   }
    /// }
    /// ```
    NoConditionalExpect,
    correctness
);

// To flag we encountered a conditional block/catch block when traversing the parents.
#[derive(Debug, Clone, Copy)]
struct InConditional(bool);

impl Rule for NoConditionalExpect {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        // Record visited nodes for avoid infinite loop.
        let mut visited = FxHashSet::default();

        // When first visiting the node, we assume it's not in a conditional block.
        let has_condition_or_catch = check_parents(node, &mut visited, InConditional(false), ctx);
        if matches!(has_condition_or_catch, InConditional(true)) {
            ctx.diagnostic(no_conditional_expect_diagnostic(jest_fn_call.head.span));
        }
    }
}

fn check_parents<'a>(
    node: &AstNode<'a>,
    visited: &mut FxHashSet<NodeId>,
    in_conditional: InConditional,
    ctx: &LintContext<'a>,
) -> InConditional {
    // if the node is already visited, we should return `false` to avoid infinite loop.
    if !visited.insert(node.id()) {
        return InConditional(false);
    }

    let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
        return InConditional(false);
    };

    match parent_node.kind() {
        AstKind::CallExpression(call_expr) => {
            let jest_node = PossibleJestNode { node: parent_node, original: None };

            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                return in_conditional;
            }

            if let Some(member_expr) = call_expr.callee.as_member_expression() {
                if member_expr.static_property_name() == Some("catch") {
                    return check_parents(parent_node, visited, InConditional(true), ctx);
                }
            }
        }
        AstKind::CatchClause(_)
        | AstKind::SwitchStatement(_)
        | AstKind::IfStatement(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::AwaitExpression(_)
        | AstKind::LogicalExpression(_) => {
            return check_parents(parent_node, visited, InConditional(true), ctx);
        }
        AstKind::Function(function) => {
            let Some(ident) = &function.id else {
                return InConditional(false);
            };
            let symbol_table = ctx.semantic().symbols();
            let Some(symbol_id) = ident.symbol_id.get() else {
                return InConditional(false);
            };

            // Consider cases like:
            // ```javascript
            // function foo() {
            //   foo()
            // }
            // ````
            // To avoid infinite loop, we need to check if the function is already visited when
            // call `check_parents`.
            let boolean = symbol_table.get_resolved_references(symbol_id).any(|reference| {
                let Some(parent) = ctx.nodes().parent_node(reference.node_id()) else {
                    return false;
                };
                matches!(check_parents(parent, visited, in_conditional, ctx), InConditional(true))
            });
            return InConditional(boolean);
        }
        AstKind::Program(_) => return InConditional(false),
        _ => {}
    }

    check_parents(parent_node, visited, in_conditional, ctx)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
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
    ];

    let mut fail = vec![
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

    let pass_vitest = vec![
        "
            it('foo', () => {
                process.env.FAIL && setNum(1);

                expect(num).toBe(2);
            });
        ",
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
        "
            it('foo', () => {
                const num = process.env.FAIL ? 1 : 2;

                expect(num).toBe(2);
            });
        ",
        "
            function getValue() {
                return process.env.FAIL ? 1 : 2
            }

            it('foo', () => {
                expect(getValue()).toBe(2);
            });
        ",
    ];

    let fail_vitest = vec![
        "
            it('foo', () => {
                something && expect(something).toHaveBeenCalled();
            })
        ",
        "
            it('foo', () => {
                a || (b && expect(something).toHaveBeenCalled());
            })
        ",
        "
            it.each``('foo', () => {
                something || expect(something).toHaveBeenCalled();
            });
        ",
        "
            it.each()('foo', () => {
                something || expect(something).toHaveBeenCalled();
            })
        ",
        "
            function getValue() {
                something || expect(something).toHaveBeenCalled();
            }
            it('foo', getValue);
        ",
        "
            it('foo', () => {
                something ? expect(something).toHaveBeenCalled() : noop();
            })
        ",
        "
            function getValue() {
                something ? expect(something).toHaveBeenCalled() : noop();
            }

            it('foo', getValue);
        ",
        "
            it('foo', () => {
                something ? noop() : expect(something).toHaveBeenCalled();
            })
        ",
    ];

    pass.extend(pass_vitest.into_iter().map(|x| (x, None)));
    fail.extend(fail_vitest.into_iter().map(|x| (x, None)));

    Tester::new(NoConditionalExpect::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
