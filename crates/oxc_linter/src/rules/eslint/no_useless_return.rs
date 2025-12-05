use oxc_ast::AstKind;
use oxc_cfg::{
    EdgeType, InstructionKind,
    graph::{Direction, visit::EdgeRef},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary return statement.")
        .with_help("Remove this redundant `return` statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows redundant return statements.
    ///
    /// ### Why is this bad?
    ///
    /// A `return;` statement with nothing after it is redundant, and has no effect
    /// on the runtime behavior of a function. This can be confusing, so it's better
    /// to disallow these redundant statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function foo() { return; }
    ///
    /// function bar() {
    ///     doSomething();
    ///     return;
    /// }
    ///
    /// function baz() {
    ///     if (condition) {
    ///         doSomething();
    ///         return;
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function foo() { return 5; }
    ///
    /// function bar() {
    ///     if (condition) {
    ///         return;
    ///     }
    ///     doSomething();
    /// }
    ///
    /// function baz() {
    ///     return doSomething();
    /// }
    /// ```
    NoUselessReturn,
    eslint,
    pedantic,
    pending // TODO: implement auto-fix
);

impl Rule for NoUselessReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // We only care about return statements without a value
        let AstKind::ReturnStatement(ret) = node.kind() else {
            return;
        };

        // If the return has a value, it's not useless
        if ret.argument.is_some() {
            return;
        }

        // Check if this return statement is useless
        if Self::is_useless_return(node, ctx) {
            ctx.diagnostic(no_useless_return_diagnostic(ret.span));
        }
    }
}

impl NoUselessReturn {
    /// Check if a return statement is useless.
    fn is_useless_return(return_node: &AstNode, ctx: &LintContext) -> bool {
        let return_node_id = return_node.id();

        // Check if this return is inside a special control structure
        for ancestor_id in ctx.nodes().ancestor_ids(return_node_id) {
            let ancestor = ctx.nodes().get_node(ancestor_id);
            match ancestor.kind() {
                // Stop at function boundary
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    break;
                }
                // Return inside loops is not useless - it's for early exit
                AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_) => {
                    return false;
                }
                // Return in finally block is never useless - it overrides other returns
                AstKind::BlockStatement(block) => {
                    // Check if this block is a finally block by looking at its parent
                    let parent_id = ctx.nodes().parent_id(ancestor.id());
                    let parent = ctx.nodes().get_node(parent_id);
                    if let AstKind::TryStatement(try_stmt) = parent.kind()
                        && try_stmt.finalizer.as_ref().is_some_and(|f| f.span == block.span)
                    {
                        return false;
                    }
                }
                // Check switch case - return prevents fallthrough when it's contained in the
                // last statement of a non-last case (either directly or inside if/else),
                // AND the subsequent cases have meaningful code.
                // If there's code after the return's containing block (like break), or
                // all subsequent cases are empty, then it doesn't prevent meaningful fallthrough.
                AstKind::SwitchCase(case) => {
                    let parent_id = ctx.nodes().parent_id(ancestor.id());
                    let parent = ctx.nodes().get_node(parent_id);
                    if let AstKind::SwitchStatement(switch_stmt) = parent.kind() {
                        // Find index of this case
                        let mut case_idx = None;
                        for (i, c) in switch_stmt.cases.iter().enumerate() {
                            if c.span == case.span {
                                case_idx = Some(i);
                                break;
                            }
                        }

                        if let Some(idx) = case_idx {
                            // If NOT the last case
                            if idx < switch_stmt.cases.len() - 1 {
                                let return_span =
                                    ctx.nodes().get_node(return_node_id).kind().span();
                                // Check if return is contained in the last statement of the case
                                if let Some(last_stmt) = case.consequent.last() {
                                    if last_stmt.span().contains_inclusive(return_span) {
                                        // Return is in the last statement - check if subsequent
                                        // cases have meaningful code (not all empty)
                                        let subsequent_cases_empty = switch_stmt
                                            .cases
                                            .iter()
                                            .skip(idx + 1)
                                            .all(|c| c.consequent.is_empty());

                                        if !subsequent_cases_empty {
                                            // Subsequent cases have code - return prevents fallthrough
                                            return false;
                                        }
                                        // All subsequent cases are empty - return is useless
                                    }
                                }
                                // Otherwise, there's more code after the return (like break),
                                // so continue checking - the return might be useless
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Use CFG to check if there's unreachable code after this return
        // If there is unreachable code, the return is NOT useless (it's preventing that code)
        if Self::has_unreachable_code_after(return_node, ctx) {
            return false;
        }

        // Check if the return is at the "end" of the function
        Self::is_at_function_end(return_node_id, ctx)
    }

    /// Check if there's unreachable code after the return statement
    fn has_unreachable_code_after(return_node: &AstNode, ctx: &LintContext) -> bool {
        let cfg = ctx.cfg();
        let graph = cfg.graph();
        let return_block_id = ctx.nodes().cfg_id(return_node.id());

        // Check outgoing edges from the return block
        for edge in graph.edges_directed(return_block_id, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Normal | EdgeType::Jump => {
                    let target = edge.target();
                    let target_block = cfg.basic_block(target);

                    // Check if target has meaningful code (not just implicit return)
                    for instr in target_block.instructions() {
                        if !matches!(
                            instr.kind,
                            InstructionKind::ImplicitReturn | InstructionKind::Unreachable
                        ) {
                            // There's code after this return - check if it's unreachable
                            if !target_block.is_unreachable() {
                                // Code is reachable - this means something would execute
                                // after the return if it were removed
                                // But wait - if there's code AFTER the return in the same block,
                                // that code IS unreachable, so the return is not useless
                                return true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Check if the return statement is at the "end" of the function
    fn is_at_function_end(return_node_id: oxc_semantic::NodeId, ctx: &LintContext) -> bool {
        let mut current_id = return_node_id;

        for ancestor_id in ctx.nodes().ancestor_ids(return_node_id) {
            let ancestor = ctx.nodes().get_node(ancestor_id);

            match ancestor.kind() {
                // Reached function boundary - check if current node is last in body
                AstKind::FunctionBody(body) => {
                    return Self::is_last_in_statements(&body.statements, current_id, ctx);
                }
                // For if statements, continue checking if the if is at the end
                AstKind::IfStatement(if_stmt) => {
                    let current_span = ctx.nodes().get_node(current_id).kind().span();

                    // Check if return is in consequent or alternate
                    let in_consequent = if_stmt.consequent.span().contains_inclusive(current_span);

                    let in_alternate = if_stmt
                        .alternate
                        .as_ref()
                        .is_some_and(|alt| alt.span().contains_inclusive(current_span));

                    if in_consequent || in_alternate {
                        current_id = ancestor_id;
                    }
                }
                // Block statement - check if current is last
                AstKind::BlockStatement(block) => {
                    if !Self::is_last_in_statements(&block.body, current_id, ctx) {
                        return false;
                    }
                    current_id = ancestor_id;
                }
                // Try statement
                AstKind::TryStatement(try_stmt) => {
                    let current_span = ctx.nodes().get_node(current_id).kind().span();

                    let in_try = try_stmt.block.span.contains_inclusive(current_span);

                    let in_catch = try_stmt
                        .handler
                        .as_ref()
                        .is_some_and(|h| h.span.contains_inclusive(current_span));

                    if in_try || in_catch {
                        current_id = ancestor_id;
                    }
                }
                // Reaching a function boundary for nested functions
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    return false;
                }
                // For switch cases, we already handled the "not last case" check above
                // If we're in the last case, continue to check if switch is at end
                _ => {
                    current_id = ancestor_id;
                }
            }
        }

        false
    }

    /// Check if a node is the last statement in a list
    fn is_last_in_statements(
        statements: &oxc_allocator::Vec<oxc_ast::ast::Statement>,
        node_id: oxc_semantic::NodeId,
        ctx: &LintContext,
    ) -> bool {
        let Some(last) = statements.last() else {
            return false;
        };

        let node_span = ctx.nodes().get_node(node_id).kind().span();

        // Check if node is contained in the last statement
        last.span().contains_inclusive(node_span)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // ESLint valid test cases
    // https://github.com/eslint/eslint/blob/main/tests/lib/rules/no-useless-return.js
    let pass = vec![
        // Basic cases - return with value
        "function foo() { return 5; }",
        "function foo() { return null; }",
        "function foo() { return doSomething(); }",
        // if-else with code after
        "
        function foo() {
            if (bar) {
                doSomething();
                return;
            } else {
                doSomethingElse();
            }
            qux();
        }
        ",
        // switch with multiple cases - return prevents fallthrough
        "
        function foo() {
            switch (bar) {
                case 1:
                    doSomething();
                    return;
                default:
                    doSomethingElse();
            }
        }
        ",
        "
        function foo() {
            switch (bar) {
                default:
                    doSomething();
                    return;
                case 1:
                    doSomethingElse();
            }
        }
        ",
        "
        function foo() {
            switch (bar) {
                case 1:
                    if (a) {
                        doSomething();
                        return;
                    } else {
                        doSomething();
                        return;
                    }
                default:
                    doSomethingElse();
            }
        }
        ",
        // for loop
        "
        function foo() {
            for (var foo = 0; foo < 10; foo++) {
                return;
            }
        }
        ",
        // for-in loop
        "
        function foo() {
            for (var foo in bar) {
                return;
            }
        }
        ",
        // for-of loop
        "function foo() { for (var foo of bar) return; }",
        // try-finally (return in finally can override)
        "
        function foo() {
            try {
                return 5;
            } finally {
                return;
            }
        }
        ",
        // try-catch with code after
        "
        function foo() {
            try {
                bar();
                return;
            } catch (err) {}
            baz();
        }
        ",
        "
        function foo() {
            if (something) {
                try {
                    bar();
                    return;
                } catch (err) {}
            }
            baz();
        }
        ",
        // unreachable code after return
        "
        function foo() {
            return;
            doSomething();
        }
        ",
        "
			              function foo() {
			                for (var foo of bar) return;
			              }
			            ", // { "ecmaVersion": 6 },
        // arrow functions
        "() => { if (foo) return; bar(); }",
        "() => 5",
        "() => { return; doSomething(); }",
        "if (foo) { return; } doSomething();", // {				"parserOptions": { "ecmaFeatures": { "globalReturn": true } },			},
        "
        function foo() {
            if (bar) return;
            return baz;
        }
        ",
        "
        function foo() {
            if (bar) {
                return;
            }
            return baz;
        }
        ",
        "
        function foo() {
            if (bar) baz();
            else return;
            return 5;
        }
        ",
        // unreachable while after return
        "
        function foo() {
            return;
            while (foo) return;
            foo;
        }
        ",
        // try-finally with code after
        "
			          try {
			            throw new Error('foo');
			            while (false);
			          } catch (err) {}
			        ",
        r#"
			          function foo(arg) {
			            throw new Error("Debugging...");
			            if (!arg) {
			              return;
			            }
			            console.log(arg);
			          }
			        "#,
        "
        function foo() {
            try {
                bar();
                return;
            } finally {
                baz();
            }
            qux();
        }
        ",
        // Empty function
        "function foo() { }",
        // while loop
        "function foo() { while (true) { return; } }",
        // do-while loop
        "function foo() { do { return; } while (true); }",
    ];

    // ESLint invalid test cases
    let fail = vec![
        // Simple useless return
        "function foo() { return; }",
        "function foo() { doSomething(); return; }",
        // if-else where both branches or if itself is last
        "function foo() { if (condition) { bar(); return; } else { baz(); } }",
        "function foo() { if (foo) return; }",
        // Multiple useless returns
        "
        function foo() {
            if (foo) {
                return;
            }
            return;
        }
        ",
        // Switch fallthrough cases - return is useless when in last executed case
        "function foo() { switch (bar) { case 1: doSomething(); default: doSomethingElse(); return; } }",
        "function foo() { switch (bar) { default: doSomething(); case 1: doSomething(); return; } }",
        // Switch with if+break patterns
        "function foo() { switch (bar) { case 1: if (a) { doSomething(); return; } break; default: doSomethingElse(); } }",
        "function foo() { switch (bar) { case 1: if (a) { doSomething(); return; } else { doSomething(); } break; default: doSomethingElse(); } }",
        "function foo() { switch (bar) { case 1: if (a) { doSomething(); return; } default: } }",
        // try-catch (useless return in catch)
        "function foo() { try {} catch (err) { return; } }",
        // try with useless return, catch has return value
        "
        function foo() {
            try {
                foo();
                return;
            } catch (err) {
                return 5;
            }
        }
        ",
        // if inside try with useless return
        "
        function foo() {
            if (something) {
                try {
                    bar();
                    return;
                } catch (err) {}
            }
        }
        ",
        // try with useless return, catch has other code
        "
        function foo() {
            try {
                return;
            } catch (err) {
                foo();
            }
        }
        ",
        // try with useless return and finally
        "
        function foo() {
            try {
                return;
            } finally {
                bar();
            }
        }
        ",
        // nested try-catch
        "
        function foo() {
            try {
                bar();
            } catch (e) {
                try {
                    baz();
                    return;
                } catch (e) {
                    qux();
                }
            }
        }
        ",
        // try-finally then useless return
        "
        function foo() {
            try {} finally {}
            return;
        }
        ",
        // nested function in finally with useless return
        "
        function foo() {
            try {
                return 5;
            } finally {
                function bar() {
                    return;
                }
            }
        }
        ",
        // Arrow function
        "() => { return; }",
        // Consecutive returns (first is useless as unreachable follows)
        "function foo() { return; return; }",
        // Nested functions
        "function foo() { function bar() { return; } }",
        // Arrow functions with body
        "const foo = () => { return; }",
        "const foo = () => { doSomething(); return; }",
        // Method
        "const obj = { foo() { return; } }",
        // Class method
        "class Foo { bar() { return; } }",
    ];

    // Note: ESLint has additional test cases that require `parserOptions: { ecmaFeatures: { globalReturn: true } }`
    // These test global scope return statements (e.g., `return;` or `if (foo) { return; }`).
    // oxc does not support globalReturn, so these tests are not included.
    // Examples from ESLint:
    // - "foo(); return;" // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } } }
    // - "if (foo) { bar(); return; } else { baz(); }" // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } } }

    Tester::new(NoUselessReturn::NAME, NoUselessReturn::PLUGIN, pass, fail).test_and_snapshot();
}
