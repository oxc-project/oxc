use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        parse_expect_jest_fn_call,
    },
};

// String constants to avoid repeated allocations
const EXPECT_STR: &str = "expect";
const FAIL_STR: &str = "fail";
const CATCH_STR: &str = "catch";

fn no_conditional_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected conditional expect")
        .with_help("Avoid calling `expect` conditionally")
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
    /// Jest only considers a test to have failed if it throws an error, meaning if calls to
    /// assertion functions like expect occur in conditional code such as a catch statement,
    /// tests can end up passing but not actually test anything. Additionally, conditionals
    /// tend to make tests more brittle and complex, as they increase the amount of mental
    /// thinking needed to understand what is actually being tested.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
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
    /// it('baz', async () => {
    ///   try {
    ///     await foo();
    ///   } catch (err) {
    ///     expect(err).toMatchObject({ code: 'MODULE_NOT_FOUND' });
    ///   }
    /// });
    ///
    /// it('throws an error', async () => {
    ///   await foo().catch(error => expect(error).toBeInstanceOf(error));
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// it('foo', () => {
    ///   expect(!value).toBe(false);
    /// });
    ///
    /// function getValue() {
    ///   if (process.env.FAIL) {
    ///     return 1;
    ///   }
    ///   return 2;
    /// }
    ///
    /// it('foo', () => {
    ///   expect(getValue()).toBe(2);
    /// });
    ///
    /// it('validates the request', () => {
    ///   try {
    ///     processRequest(request);
    ///   } catch { } finally {
    ///     expect(validRequest).toHaveBeenCalledWith(request);
    ///   }
    /// });
    ///
    /// it('throws an error', async () => {
    ///   await expect(foo).rejects.toThrow(Error);
    /// });
    /// ```
    NoConditionalExpect,
    jest,
    correctness
);

// To flag we encountered a conditional block/catch block when traversing the parents.
#[derive(Debug, Clone, Copy)]
struct InConditional(bool);

// To track if we're inside a Jest test context
#[derive(Debug, Clone, Copy)]
struct InJestTest(bool);

impl Rule for NoConditionalExpect {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        if let AstKind::CallExpression(call_expr) = node.kind() {
            // First try to parse as a Jest expect call
            if let Some(jest_fn_call) =
                parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
            {
                // Record visited nodes for avoid infinite loop.
                let mut visited = FxHashSet::default();

                // When first visiting the node, we assume it's not in a conditional block.
                let (has_condition_or_catch, in_jest_test) =
                    check_parents(node, &mut visited, InConditional(false), InJestTest(false), ctx);

                if matches!(has_condition_or_catch, InConditional(true)) {
                    // Check if we're in a Jest test context
                    if matches!(in_jest_test, InJestTest(true)) {
                        ctx.diagnostic(no_conditional_expect_diagnostic(jest_fn_call.head.span));
                    }
                }
                return;
            }

            // If not a Jest expect call, check if it's a regular expect call that might be in a function used by Jest
            if let Expression::Identifier(ident) = &call_expr.callee {
                if ident.name == EXPECT_STR {
                    // Check if this expect call is inside a function that's used in Jest tests
                    if is_expect_in_jest_function_context(node, ctx) {
                        // Record visited nodes for avoid infinite loop.
                        let mut visited = FxHashSet::default();

                        // When first visiting the node, we assume it's not in a conditional block.
                        let (has_condition_or_catch, _) = check_parents(
                            node,
                            &mut visited,
                            InConditional(false),
                            InJestTest(false),
                            ctx,
                        );

                        if matches!(has_condition_or_catch, InConditional(true)) {
                            // Check if this expect call is in a finally block, which should be allowed
                            if !is_in_finally_block_simple(node, ctx) {
                                ctx.diagnostic(no_conditional_expect_diagnostic(ident.span));
                            }
                        }
                    }
                }
            }
            // Also check member expressions like expect().toBe() or expect.fail()
            else if let Expression::StaticMemberExpression(member) = &call_expr.callee {
                if let Expression::Identifier(ident) = &member.object {
                    if ident.name == EXPECT_STR {
                        // Check if this is expect.fail, which should be allowed
                        if member.property.name == FAIL_STR {
                            return; // Don't flag expect.fail
                        }

                        // Check if this expect call is inside a function that's used in Jest tests
                        if is_expect_in_jest_function_context(node, ctx) {
                            // Record visited nodes for avoid infinite loop.
                            let mut visited = FxHashSet::default();

                            // When first visiting the node, we assume it's not in a conditional block.
                            let (has_condition_or_catch, _) = check_parents(
                                node,
                                &mut visited,
                                InConditional(false),
                                InJestTest(false),
                                ctx,
                            );

                            if matches!(has_condition_or_catch, InConditional(true)) {
                                ctx.diagnostic(no_conditional_expect_diagnostic(ident.span));
                            }
                        }
                    }
                }
                // Also check chained calls like expect().toBe()
                else if let Expression::CallExpression(inner_call) = &member.object {
                    if let Expression::Identifier(ident) = &inner_call.callee {
                        if ident.name == EXPECT_STR {
                            // Check if this expect call is inside a function that's used in Jest tests
                            if is_expect_in_jest_function_context(node, ctx) {
                                // Record visited nodes for avoid infinite loop.
                                let mut visited = FxHashSet::default();

                                // When first visiting the node, we assume it's not in a conditional block.
                                let (has_condition_or_catch, _) = check_parents(
                                    node,
                                    &mut visited,
                                    InConditional(false),
                                    InJestTest(false),
                                    ctx,
                                );

                                if matches!(has_condition_or_catch, InConditional(true)) {
                                    // Check if this expect call is in a finally block, which should be allowed
                                    if !is_in_finally_block_simple(node, ctx) {
                                        ctx.diagnostic(no_conditional_expect_diagnostic(
                                            ident.span,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_top_level_function<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Check if the parent is a Program node (top-level) or a ExportNamedDeclaration/ExportDefaultDeclaration
    let parent_node = ctx.nodes().parent_node(func_node.id());
    matches!(
        parent_node.kind(),
        AstKind::Program(_)
            | AstKind::ExportNamedDeclaration(_)
            | AstKind::ExportDefaultDeclaration(_)
    )
}

fn is_expect_in_jest_function_context<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Walk up the AST to find if this expect call is inside a function that's used in Jest tests
    let mut current_node = node;

    loop {
        let parent_node = ctx.nodes().parent_node(current_node.id());

        match parent_node.kind() {
            AstKind::Function(_func) => {
                // Check if this is a top-level function
                if is_top_level_function(parent_node, ctx) {
                    // Check if this function is used in Jest tests
                    return function_used_in_jest_tests(parent_node, ctx);
                }
                current_node = parent_node;
            }
            AstKind::Program(_) => {
                // Reached the top level, not in a function used by Jest
                return false;
            }
            _ => {
                current_node = parent_node;
            }
        }
    }
}

fn is_in_finally_block_simple<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Simple check: walk up the AST and see if we encounter a TryStatement with a finalizer
    let mut current_node = node;

    loop {
        let parent_node = ctx.nodes().parent_node(current_node.id());

        match parent_node.kind() {
            AstKind::TryStatement(try_stmt) => {
                // If this try statement has a finalizer, we're in a finally block
                return try_stmt.finalizer.is_some();
            }
            AstKind::Program(_) => {
                // Reached the top level, not in a finally block
                return false;
            }
            _ => {
                current_node = parent_node;
            }
        }
    }
}

fn function_used_in_jest_tests<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Get the function name if it's a named function
    let func_name = match func_node.kind() {
        AstKind::Function(func) => func.id.as_ref().map(|id| id.name.as_str()),
        _ => return false,
    };

    let Some(name) = func_name else {
        return false;
    };

    // Walk through all nodes to find Jest test calls
    for node in ctx.semantic().nodes().iter() {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            // Check if this is a Jest test function call
            let jest_node = PossibleJestNode { node, original: None };
            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                // Check if any argument is an identifier that matches our function name
                for arg in &call_expr.arguments {
                    if let Some(Expression::Identifier(ident)) = arg.as_expression() {
                        if ident.name == name {
                            return true; // Early return when found
                        }
                    }
                }
            }
        }
    }

    false
}

fn check_parents<'a>(
    node: &AstNode<'a>,
    visited: &mut FxHashSet<NodeId>,
    in_conditional: InConditional,
    in_jest_test: InJestTest,
    ctx: &LintContext<'a>,
) -> (InConditional, InJestTest) {
    // if the node is already visited, we should return `false` to avoid infinite loop.
    if !visited.insert(node.id()) {
        return (InConditional(false), in_jest_test);
    }

    let parent_node = ctx.nodes().parent_node(node.id());

    match parent_node.kind() {
        AstKind::CallExpression(call_expr) => {
            let jest_node = PossibleJestNode { node: parent_node, original: None };

            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                return (in_conditional, InJestTest(true));
            }

            // Optimized catch detection
            if let Some(member_expr) = call_expr.callee.as_member_expression() {
                if member_expr.static_property_name() == Some(CATCH_STR) {
                    return check_parents(
                        parent_node,
                        visited,
                        InConditional(true),
                        in_jest_test,
                        ctx,
                    );
                }
            }
        }
        AstKind::BlockStatement(_) => {
            // Continue checking without marking as conditional
            return check_parents(parent_node, visited, in_conditional, in_jest_test, ctx);
        }
        AstKind::CatchClause(_)
        | AstKind::SwitchStatement(_)
        | AstKind::IfStatement(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::LogicalExpression(_)
        | AstKind::Function(_) => {
            // Continue checking but mark that we're in a conditional context
            return check_parents(parent_node, visited, InConditional(true), in_jest_test, ctx);
        }
        AstKind::Program(_) => {
            return (in_conditional, in_jest_test);
        }
        _ => {}
    }

    check_parents(parent_node, visited, in_conditional, in_jest_test, ctx)
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
        (
            "it('throws an error', async () => {
                await expect(foo).rejects.toThrow(Error);
            });",
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

    Tester::new(NoConditionalExpect::NAME, NoConditionalExpect::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
