use oxc_allocator::Vec as ArenaVec;
use oxc_ast::AstKind;
use oxc_cfg::{
    EdgeType, ErrorEdgeKind, InstructionKind,
    graph::{Direction, visit::EdgeRef},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
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
    pending
);

impl Rule for NoUselessReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ReturnStatement(ret) = node.kind() else {
            return;
        };

        if ret.argument.is_some() {
            return;
        }

        if Self::is_useless_return(node, ctx) {
            ctx.diagnostic(no_useless_return_diagnostic(ret.span));
        }
    }
}

/// Result of analyzing ancestor context for a return statement
enum AncestorAnalysis {
    /// Return is definitely not useless (e.g., inside loop or finally block)
    NotUseless,
    /// Return is at function end (is useless if no reachable code after)
    AtFunctionEnd,
    /// Return is not at function end (not useless)
    NotAtFunctionEnd,
}

impl NoUselessReturn {
    /// Check if a return statement is useless.
    /// A return is useless if:
    /// 1. It has no value
    /// 2. It's at the "end" of the function (last statement in all enclosing blocks)
    /// 3. There's no reachable code after it that would execute if removed
    /// 4. It's not inside a loop (where it serves as early exit)
    /// 5. It's not inside a finally block (where it overrides other returns)
    /// 6. It's not preventing switch fallthrough
    fn is_useless_return(return_node: &AstNode, ctx: &LintContext) -> bool {
        let return_span = return_node.kind().span();

        match Self::analyze_ancestors(return_node.id(), return_span, ctx) {
            AncestorAnalysis::NotUseless | AncestorAnalysis::NotAtFunctionEnd => false,
            AncestorAnalysis::AtFunctionEnd => {
                // Only check CFG if we're at function end
                !Self::has_reachable_code_after(return_node, ctx)
            }
        }
    }

    /// Analyze ancestors in a single pass to determine:
    /// 1. If return is in a special context (loop, finally, switch fallthrough)
    /// 2. If return is at the "end" of the function
    fn analyze_ancestors(
        return_node_id: NodeId,
        return_span: Span,
        ctx: &LintContext,
    ) -> AncestorAnalysis {
        let nodes = ctx.nodes();
        let mut current_span = return_span;

        for ancestor_id in nodes.ancestor_ids(return_node_id) {
            let ancestor_kind = nodes.kind(ancestor_id);

            match ancestor_kind {
                AstKind::FunctionBody(body) => {
                    return if Self::is_span_in_last_statement(&body.statements, current_span) {
                        AncestorAnalysis::AtFunctionEnd
                    } else {
                        AncestorAnalysis::NotAtFunctionEnd
                    };
                }

                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    return AncestorAnalysis::NotAtFunctionEnd;
                }

                AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_) => {
                    return AncestorAnalysis::NotUseless;
                }

                AstKind::BlockStatement(block) => {
                    let parent_kind = nodes.parent_kind(ancestor_id);
                    if let AstKind::TryStatement(try_stmt) = parent_kind
                        && try_stmt.finalizer.as_ref().is_some_and(|f| f.span == block.span)
                    {
                        return AncestorAnalysis::NotUseless;
                    }
                    if !Self::is_span_in_last_statement(&block.body, current_span) {
                        return AncestorAnalysis::NotAtFunctionEnd;
                    }
                    current_span = block.span;
                }

                AstKind::SwitchCase(case) => {
                    let parent_kind = nodes.parent_kind(ancestor_id);
                    if let AstKind::SwitchStatement(switch_stmt) = parent_kind
                        && let Some((idx, _)) =
                            switch_stmt.cases.iter().enumerate().find(|(_, c)| c.span == case.span)
                    {
                        let is_last_case = idx == switch_stmt.cases.len() - 1;

                        if !is_last_case
                            && case.consequent.last().is_some_and(|last_stmt| {
                                last_stmt.span().contains_inclusive(current_span)
                            })
                        {
                            let subsequent_cases_empty = switch_stmt
                                .cases
                                .iter()
                                .skip(idx + 1)
                                .all(|c| c.consequent.is_empty());

                            if !subsequent_cases_empty {
                                return AncestorAnalysis::NotUseless;
                            }
                        }
                    }
                    current_span = case.span;
                }

                AstKind::IfStatement(if_stmt) => {
                    let in_consequent = if_stmt.consequent.span().contains_inclusive(current_span);
                    let in_alternate = if_stmt
                        .alternate
                        .as_ref()
                        .is_some_and(|alt| alt.span().contains_inclusive(current_span));

                    if in_consequent || in_alternate {
                        current_span = if_stmt.span;
                    }
                }

                AstKind::TryStatement(try_stmt) => {
                    let in_try = try_stmt.block.span.contains_inclusive(current_span);
                    let in_catch = try_stmt
                        .handler
                        .as_ref()
                        .is_some_and(|h| h.span.contains_inclusive(current_span));

                    if in_try || in_catch {
                        current_span = try_stmt.span;
                    }
                }

                _ => {
                    current_span = ancestor_kind.span();
                }
            }
        }

        AncestorAnalysis::NotAtFunctionEnd
    }

    /// Check if removing this return would make code after it reachable.
    /// Returns true if the return prevents reachable code from executing.
    fn has_reachable_code_after(return_node: &AstNode, ctx: &LintContext) -> bool {
        let cfg = ctx.cfg();
        let graph = cfg.graph();
        let return_block_id = ctx.nodes().cfg_id(return_node.id());

        for edge in graph.edges_directed(return_block_id, Direction::Outgoing) {
            let dominated = matches!(
                edge.weight(),
                EdgeType::Normal | EdgeType::Jump | EdgeType::Error(ErrorEdgeKind::Explicit)
            );

            if dominated {
                let target = edge.target();
                let target_block = cfg.basic_block(target);

                if target_block.is_unreachable() {
                    continue;
                }

                let has_meaningful_code = target_block.instructions().iter().any(|instr| {
                    !matches!(
                        instr.kind,
                        InstructionKind::ImplicitReturn | InstructionKind::Unreachable
                    )
                });

                if has_meaningful_code {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a span is contained in the last statement of a statement list
    #[inline]
    fn is_span_in_last_statement(
        statements: &ArenaVec<oxc_ast::ast::Statement>,
        span: Span,
    ) -> bool {
        statements.last().is_some_and(|last| last.span().contains_inclusive(span))
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
        // Labeled break - return prevents reaching code after the label
        "function foo() { label: { if (x) { return; } break label; } doSomething(); }",
        // Try with throw - return in catch prevents code after try
        "function foo() { try { throw new Error(); } catch (e) { return; } doSomething(); }",
        // Nested try-catch where return in inner catch prevents outer code
        "function foo() { try { try { throw 1; } catch (e) { return; } } catch (e) { } doSomething(); }",
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
        // Consecutive returns (second is at function end and is useless)
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
        // Deeply nested if statements - useless return at deep level
        "function foo() { if (a) { if (b) { if (c) { return; } } } }",
        // Labeled statement with useless return
        "function foo() { label: { return; } }",
        // With statement (deprecated but valid)
        "function foo() { with (obj) { return; } }",
        // Getter with useless return
        "const obj = { get foo() { return; } }",
        // Setter with useless return
        "const obj = { set foo(val) { return; } }",
    ];

    // Note: ESLint has additional test cases that require `parserOptions: { ecmaFeatures: { globalReturn: true } }`
    // These test global scope return statements (e.g., `return;` or `if (foo) { return; }`).
    // oxc does not support globalReturn, so these tests are not included.
    // Examples from ESLint:
    // - "foo(); return;" // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } } }
    // - "if (foo) { bar(); return; } else { baz(); }" // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } } }

    Tester::new(NoUselessReturn::NAME, NoUselessReturn::PLUGIN, pass, fail).test_and_snapshot();
}
