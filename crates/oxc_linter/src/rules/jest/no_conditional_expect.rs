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
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        parse_expect_jest_fn_call,
    },
};

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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) => {
                // Only check top-level functions (not nested ones) since those are the ones
                // that could be passed by reference to Jest tests
                if is_top_level_function(node, ctx) {
                    if let Some(func_body) = &func.body {
                        if function_used_in_jest_tests(node, ctx) {
                            check_function_body_for_conditional_expects(func_body, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        if let AstKind::CallExpression(call_expr) = node.kind() {
            println!("[DEBUG] Found CallExpression, checking if it's an expect call");

            let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
            else {
                println!("[DEBUG] Not an expect call, returning early");
                return;
            };

            // Check if this is expect.fail, which should be allowed in conditional contexts
            let is_expect_fail = if let Some(matcher) = jest_fn_call.matcher() {
                if let Some(name) = matcher.name() { name == "fail" } else { false }
            } else {
                false
            };

            if is_expect_fail {
                println!("[DEBUG] Found expect.fail, checking conditional context");
            }

            println!("[DEBUG] Found expect call: {:?}", jest_fn_call.head.span);

            // Record visited nodes for avoid infinite loop.
            let mut visited = FxHashSet::default();

            // When first visiting the node, we assume it's not in a conditional block.
            let (has_condition_or_catch, in_jest_test) = check_parents(
                node,
                &mut visited,
                InConditional(false),
                InJestTest(false),
                ctx,
                possible_jest_node,
            );

            println!(
                "[DEBUG] check_parents result: {:?}, in_jest_test: {:?}",
                has_condition_or_catch, in_jest_test
            );

            if matches!(has_condition_or_catch, InConditional(true)) {
                // Check if we're in a Jest test context
                if matches!(in_jest_test, InJestTest(true)) {
                    println!("[DEBUG] Found conditional expect in Jest test, creating diagnostic");
                    ctx.diagnostic(no_conditional_expect_diagnostic(jest_fn_call.head.span));
                } else {
                    // For standalone functions, check if this is expect.fail, which should be allowed
                    let is_expect_fail = if let Some(matcher) = jest_fn_call.matcher() {
                        if let Some(name) = matcher.name() { name == "fail" } else { false }
                    } else {
                        false
                    };

                    if is_expect_fail {
                        println!(
                            "[DEBUG] Found expect.fail in conditional context in standalone function, allowing it"
                        );
                    } else {
                        println!(
                            "[DEBUG] Conditional expect not in Jest test context, no diagnostic needed"
                        );
                    }
                }
            } else {
                println!("[DEBUG] No conditional context found, no diagnostic needed");
            }
        } else {
            println!("[DEBUG] Node is not a CallExpression: {:?}", node.kind());
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

fn function_used_in_jest_tests<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Get the function name if it's a named function
    let func_name = match func_node.kind() {
        AstKind::Function(func) => func.id.as_ref().map(|id| id.name.as_str()),
        _ => return false,
    };

    let Some(name) = func_name else {
        println!("[DEBUG] Function has no name, skipping");
        return false;
    };

    println!("[DEBUG] Checking if function '{}' is used in Jest tests", name);

    // Search for Jest test calls that reference this function
    let mut is_used_in_jest = false;

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
                    if let Some(expr) = arg.as_expression() {
                        if let oxc_ast::ast::Expression::Identifier(ident) = expr {
                            if ident.name == name {
                                is_used_in_jest = true;
                                break;
                            }
                        }
                    }
                }
                if is_used_in_jest {
                    break;
                }
            }
        }
    }

    is_used_in_jest
}

fn check_function_body_for_conditional_expects<'a>(
    func_body: &oxc_allocator::Box<'_, oxc_ast::ast::FunctionBody<'a>>,
    ctx: &LintContext<'a>,
) {
    // Walk through the function body looking for expect calls in conditional contexts
    for stmt in &func_body.statements {
        check_statement_for_conditional_expects(stmt, ctx);
    }
}

fn check_statement_for_conditional_expects<'a>(
    stmt: &oxc_ast::ast::Statement<'a>,
    ctx: &LintContext<'a>,
) {
    check_statement_for_conditional_expects_with_context(stmt, ctx, false);
}

fn check_statement_for_conditional_expects_with_context<'a>(
    stmt: &oxc_ast::ast::Statement<'a>,
    ctx: &LintContext<'a>,
    in_conditional: bool,
) {
    use oxc_ast::ast::Statement;

    match stmt {
        Statement::IfStatement(if_stmt) => {
            // Check consequent and alternate for expect calls (these are conditional)
            check_statement_for_conditional_expects_with_context(&if_stmt.consequent, ctx, true);
            if let Some(alternate) = &if_stmt.alternate {
                check_statement_for_conditional_expects_with_context(alternate, ctx, true);
            }
        }
        Statement::SwitchStatement(switch_stmt) => {
            // Check all switch cases for expect calls (these are conditional)
            for case in &switch_stmt.cases {
                for stmt in &case.consequent {
                    check_statement_for_conditional_expects_with_context(stmt, ctx, true);
                }
            }
        }
        Statement::BlockStatement(block) => {
            for stmt in &block.body {
                check_statement_for_conditional_expects_with_context(stmt, ctx, in_conditional);
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            check_expression_for_conditional_expects(&expr_stmt.expression, ctx, in_conditional);
        }
        Statement::TryStatement(try_stmt) => {
            // Check catch clause for expect calls (these are conditional)
            if let Some(catch_clause) = &try_stmt.handler {
                for stmt in &catch_clause.body.body {
                    check_statement_for_conditional_expects_with_context(stmt, ctx, true);
                }
            }
        }
        _ => {}
    }
}

fn check_expression_for_conditional_expects<'a>(
    expr: &oxc_ast::ast::Expression<'a>,
    ctx: &LintContext<'a>,
    in_conditional: bool,
) {
    use oxc_ast::ast::Expression;

    match expr {
        Expression::LogicalExpression(logical) => {
            // Both sides of logical expressions are conditional
            check_expression_for_conditional_expects(&logical.left, ctx, true);
            check_expression_for_conditional_expects(&logical.right, ctx, true);
        }
        Expression::ConditionalExpression(conditional) => {
            // Both consequent and alternate are conditional
            check_expression_for_conditional_expects(&conditional.consequent, ctx, true);
            check_expression_for_conditional_expects(&conditional.alternate, ctx, true);
        }
        Expression::CallExpression(call) => {
            // Check if this is an expect call
            if let Expression::Identifier(ident) = &call.callee {
                if ident.name == "expect" && in_conditional {
                    println!("[DEBUG] Found conditional expect in function: {:?}", ident.span);
                    ctx.diagnostic(no_conditional_expect_diagnostic(ident.span));
                }
            }
            // Also check member expressions like expect().toBe() or expect.fail()
            else if let Expression::StaticMemberExpression(member) = &call.callee {
                if let Expression::Identifier(ident) = &member.object {
                    if ident.name == "expect" && in_conditional {
                        // Check if this is expect.fail, which should be allowed
                        if member.property.name == "fail" {
                            println!(
                                "[DEBUG] Found expect.fail in conditional context, allowing it"
                            );
                            return; // Don't flag expect.fail
                        }
                        println!(
                            "[DEBUG] Found conditional expect method call in function: {:?}",
                            ident.span
                        );
                        ctx.diagnostic(no_conditional_expect_diagnostic(ident.span));
                    }
                }
                // Also check chained calls like expect().toBe()
                else if let Expression::CallExpression(inner_call) = &member.object {
                    if let Expression::Identifier(ident) = &inner_call.callee {
                        if ident.name == "expect" && in_conditional {
                            ctx.diagnostic(no_conditional_expect_diagnostic(ident.span));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn check_parents<'a>(
    node: &AstNode<'a>,
    visited: &mut FxHashSet<NodeId>,
    in_conditional: InConditional,
    in_jest_test: InJestTest,
    ctx: &LintContext<'a>,
    jest_node: &PossibleJestNode<'a, '_>,
) -> (InConditional, InJestTest) {
    println!(
        "[DEBUG] check_parents called with node_id: {:?}, in_conditional: {:?}",
        node.id(),
        in_conditional
    );

    // if the node is already visited, we should return `false` to avoid infinite loop.
    if !visited.insert(node.id()) {
        println!("[DEBUG] Node already visited, returning InConditional(false)");
        return (InConditional(false), in_jest_test);
    }

    let parent_node = ctx.nodes().parent_node(node.id());
    println!("[DEBUG] Parent node kind: {:?}", parent_node.kind());

    match parent_node.kind() {
        AstKind::CallExpression(call_expr) => {
            println!("[DEBUG] Parent is CallExpression");
            let jest_node = PossibleJestNode { node: parent_node, original: None };

            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                println!(
                    "[DEBUG] Parent is a Jest test function, returning current in_conditional: {:?}",
                    in_conditional
                );
                return (in_conditional, InJestTest(true));
            }

            if let Some(member_expr) = call_expr.callee.as_member_expression() {
                if member_expr.static_property_name() == Some("catch") {
                    println!("[DEBUG] Found catch method call, marking as conditional");
                    return check_parents(
                        parent_node,
                        visited,
                        InConditional(true),
                        in_jest_test,
                        ctx,
                        &jest_node,
                    );
                }
            }
        }
        AstKind::CatchClause(_) => {
            println!("[DEBUG] Found CatchClause, marking as conditional");
            // Continue checking but mark that we're in a conditional context
            return check_parents(
                parent_node,
                visited,
                InConditional(true),
                in_jest_test,
                ctx,
                jest_node,
            );
        }
        AstKind::SwitchStatement(_) => {
            println!("[DEBUG] Found SwitchStatement, marking as conditional");
            return check_parents(
                parent_node,
                visited,
                InConditional(true),
                in_jest_test,
                ctx,
                jest_node,
            );
        }
        AstKind::IfStatement(_) => {
            println!("[DEBUG] Found IfStatement, marking as conditional");
            return check_parents(
                parent_node,
                visited,
                InConditional(true),
                in_jest_test,
                ctx,
                jest_node,
            );
        }
        AstKind::ConditionalExpression(_) => {
            println!("[DEBUG] Found ConditionalExpression, marking as conditional");
            return check_parents(
                parent_node,
                visited,
                InConditional(true),
                in_jest_test,
                ctx,
                jest_node,
            );
        }
        AstKind::LogicalExpression(_) => {
            println!("[DEBUG] Found LogicalExpression, marking as conditional");
            return check_parents(
                parent_node,
                visited,
                InConditional(true),
                in_jest_test,
                ctx,
                jest_node,
            );
        }
        AstKind::Function(_) => {
            println!("[DEBUG] Found Function, checking if it's passed to Jest test");
            // For now, let's preserve the conditional status and continue checking
            // The function detection will be handled separately
            println!("[DEBUG] Function found, preserving conditional status");
            return check_parents(
                parent_node,
                visited,
                in_conditional,
                in_jest_test,
                ctx,
                jest_node,
            );
        }
        AstKind::Program(_) => {
            println!(
                "[DEBUG] Reached Program level, returning current in_conditional: {:?}",
                in_conditional
            );
            return (in_conditional, in_jest_test);
        }
        _ => {
            println!("[DEBUG] Other node type: {:?}, continuing check", parent_node.kind());
        }
    }

    println!(
        "[DEBUG] Continuing to check parent with current in_conditional: {:?}",
        in_conditional
    );
    check_parents(parent_node, visited, in_conditional, in_jest_test, ctx, jest_node)
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
