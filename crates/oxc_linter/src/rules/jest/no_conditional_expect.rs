use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-conditional-expect): Unexpected conditional expect")]
#[diagnostic(severity(warning), help("Avoid calling `expect` conditionally`"))]
struct NoConditionalExpectDiagnostic(#[label] pub Span);

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
    //   await foo().catch(error => expect(error).toBeInstanceOf(error));
    // });
    /// ```
    NoConditionalExpect,
    restriction
);

impl Rule for NoConditionalExpect {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if !is_type_of_jest_fn_call(call_expr, node, ctx, &[JestFnKind::Expect]) {
                return;
            }

            let has_condition_or_catch = check_parents(node, ctx, false);
            if has_condition_or_catch {
                ctx.diagnostic(NoConditionalExpectDiagnostic(call_expr.span));
            }
        }
    }
}

fn check_parents<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>, in_conditional: bool) -> bool {
    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };

    match parent.kind() {
        AstKind::CallExpression(call_expr) => {
            if is_type_of_jest_fn_call(
                call_expr,
                parent,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                return in_conditional;
            }

            if let Expression::MemberExpression(member_expr) = &call_expr.callee {
                if member_expr.static_property_name() == Some("catch") {
                    return check_parents(parent, ctx, true);
                }
            }
        }
        AstKind::CatchClause(_)
        | AstKind::SwitchStatement(_)
        | AstKind::IfStatement(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::LogicalExpression(_) => return check_parents(parent, ctx, true),
        AstKind::Function(function) => {
            let Some(ident) = &function.id else {
                return false;
            };
            let symbol_table = ctx.semantic().symbols();
            let Some(symbol_id) = ident.symbol_id.get() else {
                return false;
            };

            return symbol_table.get_resolved_references(symbol_id).any(|reference| {
                let Some(parent) = ctx.nodes().parent_node(reference.node_id()) else {
                    return false;
                };

                check_parents(parent, ctx, in_conditional)
            });
        }
        AstKind::Program(_) => return false,
        _ => {}
    }

    check_parents(parent, ctx, in_conditional)
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
    ];

    Tester::new(NoConditionalExpect::NAME, pass, fail).test_and_snapshot();
}
