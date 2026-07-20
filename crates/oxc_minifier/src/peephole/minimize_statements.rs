use std::{iter, ops::ControlFlow};

use crate::generated::ancestor::Ancestor;
use oxc_allocator::{ArenaBox, ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, IsLiteralValue, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_semantic::ScopeFlags;
use oxc_span::{ContentEq, GetSpan, GetSpanMut};
use oxc_syntax::symbol::SymbolId;

use crate::{TraverseCtx, keep_var::KeepVar};

use super::PeepholeOptimizations;

/// `false` when dropping `stmt` produces a byte-identical AST — a `var`
/// with no initializers, which `KeepVar` re-emits unchanged at the end of
/// the block. Flagging such an identity drop as a real change would
/// oscillate the peephole fixed-point loop forever.
///
/// A TS type annotation disqualifies the identity classification: `KeepVar`'s
/// re-emit strips annotations, and an annotation can hold resolved references
/// (computed keys in a type literal) that must go through the drop walk.
fn dead_drop_mutates_ast(stmt: &Statement<'_>) -> bool {
    !matches!(stmt, Statement::VariableDeclaration(decl)
        if decl.kind.is_var()
            && decl.declarations.iter().all(|d| d.init.is_none() && d.type_annotation.is_none()))
}

impl<'a> PeepholeOptimizations {
    /// `mangleStmts`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L8788>
    ///
    /// See also
    ///
    /// ## Statement Fusion
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/StatementFusion.java>
    ///
    /// ## Collapse variable declarations
    /// `var a; var b = 1; var c = 2` => `var a, b = 1; c = 2`
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/CollapseVariableDeclarations.java>
    ///
    /// ## Collapse into for statements:
    /// `var a = 0; for(;a<0;a++) {}` => `for(var a = 0;a<0;a++) {}`
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/Denormalize.java>
    ///
    /// ## MinimizeExitPoints:
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/MinimizeExitPoints.java>
    pub fn minimize_statements(stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let mut old_stmts = stmts.take_in(ctx);
        let mut is_control_flow_dead = false;
        let mut keep_var = KeepVar::new();
        let mut identity_drops = 0u32;
        for i in 0..old_stmts.len() {
            let stmt = old_stmts[i].take_in(ctx);
            if is_control_flow_dead
                && !stmt.is_module_declaration()
                && !matches!(stmt.as_declaration(), Some(Declaration::FunctionDeclaration(_)))
            {
                // Harvest any `var` bindings so they re-emit at the end of
                // the block (see `keep_var.get_variable_declaration_statement`
                // below).
                keep_var.visit_statement(&stmt);
                // Re-flag the peephole loop so the next iteration runs with
                // refreshed reference counts and the unused-declarator pass
                // can remove bindings that this drop just orphaned (e.g.
                // `var x = {}` after dropping `module.exports = x;`).
                if dead_drop_mutates_ast(&stmt) {
                    ctx.drop_statement(&stmt);
                } else {
                    identity_drops += 1;
                }
                continue; // drop: `stmt` is intentionally not pushed into `stmts`.
            }
            if Self::minimize_statement(stmt, i, &mut old_stmts, stmts, ctx).is_break() {
                break;
            }
            // A statement that never completes normally — a direct jump, a
            // kept block ending in a jump, an if/else or try/catch where
            // every branch jumps — makes the rest of the list unreachable.
            // https://github.com/rolldown/rolldown/issues/10184
            if !is_control_flow_dead
                && stmts.last().is_some_and(Self::statement_never_completes_normally)
            {
                is_control_flow_dead = true;
            }
        }
        if let Some(stmt) = keep_var.get_variable_declaration_statement(&ctx.ast) {
            match Self::remove_unused_variable_declaration(stmt, ctx) {
                // Multiple identity-dropped `var x;`s coalesce into a single
                // `var x, y;`. The individual drops looked byte-identical, but
                // the combined re-emit is a real AST change — re-flag so the
                // fixed-point loop doesn't terminate one iteration early.
                Some(stmt) => {
                    stmts.push(stmt);
                    if identity_drops > 1 {
                        ctx.notice_change();
                    }
                }
                // The harvested `var` was entirely unused; the net effect of
                // drop + re-hoist + remove is removing the declaration, even
                // though every individual drop was classified as identity.
                None => ctx.notice_change(),
            }
        }

        // Drop a trailing unconditional jump statement if applicable
        if let Some(last_stmt) = stmts.last() {
            match last_stmt {
                // "while (x) { y(); continue; }" => "while (x) { y(); }"
                Statement::ContinueStatement(s) if s.label.is_none() => {
                    if matches!(
                        ctx.ancestors().nth(1),
                        Some(
                            Ancestor::ForStatementBody(_)
                                | Ancestor::ForInStatementBody(_)
                                | Ancestor::ForOfStatementBody(_),
                        )
                    ) {
                        let dropped = stmts.pop().unwrap();
                        ctx.drop_statement(&dropped);
                    }
                }
                // "function f() { x(); return; }" => "function f() { x(); }"
                Statement::ReturnStatement(s) if s.argument.is_none() => {
                    if let Ancestor::FunctionBodyStatements(_) = ctx.parent() {
                        let dropped = stmts.pop().unwrap();
                        ctx.drop_statement(&dropped);
                    }
                }
                _ => {}
            }
        }

        // Merge certain statements in reverse order
        if stmts.len() >= 2 && ctx.options().sequences {
            if let Some(Statement::ReturnStatement(_)) = stmts.last() {
                'return_loop: while stmts.len() >= 2 {
                    let prev_index = stmts.len() - 2;
                    let prev_stmt = &stmts[prev_index];
                    match prev_stmt {
                        Statement::ExpressionStatement(_) => {
                            if let Some(Statement::ReturnStatement(last_return)) = stmts.last()
                                && last_return.argument.is_none()
                            {
                                break 'return_loop;
                            }
                            ctx.notice_change();
                            // "a(); return b;" => "return a(), b;"
                            let last_stmt = stmts.pop().unwrap();
                            let Statement::ReturnStatement(mut last_return) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = stmts.pop().unwrap();
                            let Statement::ExpressionStatement(mut expr_stmt) = prev_stmt else {
                                unreachable!()
                            };
                            let b = last_return.argument.as_mut().unwrap();
                            let argument = Self::join_sequence(&mut expr_stmt.expression, b, ctx);
                            let right_span = last_return.span;
                            let last_return_stmt =
                                Statement::new_return_statement(right_span, Some(argument), ctx);
                            stmts.push(last_return_stmt);
                        }
                        // Merge the last two statements
                        Statement::IfStatement(if_stmt) => {
                            // The previous statement must be an if statement with no else clause
                            if if_stmt.alternate.is_some() {
                                break 'return_loop;
                            }
                            // The then clause must be a return
                            let Statement::ReturnStatement(_) = &if_stmt.consequent else {
                                break 'return_loop;
                            };
                            if let Some(Statement::ReturnStatement(last_return)) = stmts.last()
                                && let Some(arg) = &last_return.argument
                                && Self::conditional_expression_count_exceeded(arg)
                            {
                                break 'return_loop;
                            }

                            ctx.notice_change();
                            let last_stmt = stmts.pop().unwrap();
                            let Statement::ReturnStatement(last_return) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = stmts.pop().unwrap();
                            let Statement::IfStatement(prev_if) = prev_stmt else { unreachable!() };
                            let mut prev_if = prev_if.unbox();
                            let Statement::ReturnStatement(prev_return) = prev_if.consequent else {
                                unreachable!()
                            };

                            let left_span = prev_return.span;
                            let right_span = last_return.span;
                            // "if (a) return; return b;" => "return a ? void 0 : b;"
                            let mut left = prev_return
                                .unbox()
                                .argument
                                .unwrap_or_else(|| Expression::new_void_0(left_span, ctx));
                            // "if (a) return a; return;" => "return a ? b : void 0;"
                            let mut right = last_return
                                .unbox()
                                .argument
                                .unwrap_or_else(|| Expression::new_void_0(right_span, ctx));

                            // "if (!a) return b; return c;" => "return a ? c : b;"
                            if let Expression::UnaryExpression(unary_expr) = &mut prev_if.test
                                && unary_expr.operator.is_not()
                            {
                                prev_if.test = unary_expr.argument.take_in(ctx);
                                std::mem::swap(&mut left, &mut right);
                            }

                            let argument = if let Expression::SequenceExpression(sequence_expr) =
                                &mut prev_if.test
                            {
                                // "if (a, b) return c; return d;" => "return a, b ? c : d;"
                                let test = sequence_expr.expressions.pop().unwrap();
                                let mut b = Self::minimize_conditional(
                                    prev_if.span,
                                    test,
                                    left,
                                    right,
                                    ctx,
                                );
                                Self::join_sequence(&mut prev_if.test, &mut b, ctx)
                            } else {
                                // "if (a) return b; return c;" => "return a ? b : c;"
                                Self::minimize_conditional(
                                    prev_if.span,
                                    prev_if.test.take_in(ctx),
                                    left,
                                    right,
                                    ctx,
                                )
                            };
                            let last_return_stmt =
                                Statement::new_return_statement(right_span, Some(argument), ctx);
                            stmts.push(last_return_stmt);
                        }
                        _ => break 'return_loop,
                    }
                }
            } else if let Some(Statement::ThrowStatement(_)) = stmts.last() {
                'throw_loop: while stmts.len() >= 2 {
                    let prev_index = stmts.len() - 2;
                    let prev_stmt = &stmts[prev_index];
                    match prev_stmt {
                        Statement::ExpressionStatement(_) => {
                            ctx.notice_change();
                            // "a(); throw b;" => "throw a(), b;"
                            let last_stmt = stmts.pop().unwrap();
                            let Statement::ThrowStatement(mut last_throw) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = stmts.pop().unwrap();
                            let Statement::ExpressionStatement(mut expr_stmt) = prev_stmt else {
                                unreachable!()
                            };
                            let argument = Self::join_sequence(
                                &mut expr_stmt.expression,
                                &mut last_throw.argument,
                                ctx,
                            );
                            let right_span = last_throw.span;
                            let last_throw_stmt =
                                Statement::new_throw_statement(right_span, argument, ctx);
                            stmts.push(last_throw_stmt);
                        }
                        // Merge the last two statements
                        Statement::IfStatement(if_stmt) => {
                            // The previous statement must be an if statement with no else clause
                            if if_stmt.alternate.is_some() {
                                break 'throw_loop;
                            }
                            // The then clause must be a throw
                            let Statement::ThrowStatement(_) = &if_stmt.consequent else {
                                break 'throw_loop;
                            };
                            if let Some(Statement::ThrowStatement(last_throw)) = stmts.last()
                                && Self::conditional_expression_count_exceeded(&last_throw.argument)
                            {
                                break 'throw_loop;
                            }

                            ctx.notice_change();
                            let last_stmt = stmts.pop().unwrap();
                            let Statement::ThrowStatement(last_throw) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = stmts.pop().unwrap();
                            let Statement::IfStatement(prev_if) = prev_stmt else { unreachable!() };
                            let mut prev_if = prev_if.unbox();
                            let Statement::ThrowStatement(prev_throw) = prev_if.consequent else {
                                unreachable!()
                            };

                            let right_span = last_throw.span;
                            let mut left = prev_throw.unbox().argument;
                            let mut right = last_throw.unbox().argument;

                            // "if (!a) throw b; throw c;" => "throw a ? c : b;"
                            if let Expression::UnaryExpression(unary_expr) = &mut prev_if.test
                                && unary_expr.operator.is_not()
                            {
                                prev_if.test = unary_expr.argument.take_in(ctx);
                                std::mem::swap(&mut left, &mut right);
                            }

                            let argument = if let Expression::SequenceExpression(sequence_expr) =
                                &mut prev_if.test
                            {
                                // "if (a, b) throw c; throw d;" => "throw a, b ? c : d;"
                                let test = sequence_expr.expressions.pop().unwrap();
                                let mut b = Self::minimize_conditional(
                                    prev_if.span,
                                    test,
                                    left,
                                    right,
                                    ctx,
                                );
                                Self::join_sequence(&mut prev_if.test, &mut b, ctx)
                            } else {
                                // "if (a) throw b; throw c;" => "throw a ? b : c;"
                                Self::minimize_conditional(
                                    prev_if.span,
                                    prev_if.test.take_in(ctx),
                                    left,
                                    right,
                                    ctx,
                                )
                            };
                            let last_throw_stmt =
                                Statement::new_throw_statement(right_span, argument, ctx);
                            stmts.push(last_throw_stmt);
                        }
                        _ => break 'throw_loop,
                    }
                }
            }
        }
    }

    /// Some parsers cannot parse long conditional expressions.
    /// See <https://bugzilla.mozilla.org/show_bug.cgi?id=2033215>
    fn conditional_expression_count_exceeded(expr: &Expression<'a>) -> bool {
        let mut depth = 0u16;
        let mut current = expr;
        while let Expression::ConditionalExpression(c) = current {
            depth += 1;
            if depth == 500 {
                return true;
            }
            current = &c.alternate;
        }
        false
    }

    fn minimize_statement(
        stmt: Statement<'a>,
        i: usize,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ControlFlow<()> {
        match stmt {
            Statement::EmptyStatement(_) => (),
            Statement::VariableDeclaration(var_decl) => {
                Self::handle_variable_declaration(var_decl, result, ctx);
            }
            Statement::ExpressionStatement(expr_stmt) => {
                Self::handle_expression_statement(expr_stmt, result, ctx);
            }
            Statement::SwitchStatement(switch_stmt) => {
                Self::handle_switch_statement(switch_stmt, result, ctx);
            }
            Statement::IfStatement(if_stmt) => {
                if Self::handle_if_statement(i, stmts, if_stmt, result, ctx).is_break() {
                    return ControlFlow::Break(());
                }
            }
            Statement::ReturnStatement(ret_stmt) => {
                Self::handle_return_statement(ret_stmt, result, ctx);
            }
            Statement::ThrowStatement(throw_stmt) => {
                Self::handle_throw_statement(throw_stmt, result, ctx);
            }
            Statement::ForStatement(for_stmt) => {
                Self::handle_for_statement(for_stmt, result, ctx);
            }
            Statement::ForInStatement(for_in_stmt) => {
                Self::handle_for_in_statement(for_in_stmt, result, ctx);
            }
            Statement::ForOfStatement(for_of_stmt) => {
                Self::handle_for_of_statement(for_of_stmt, result, ctx);
            }
            Statement::BlockStatement(block_stmt) => Self::handle_block(result, block_stmt, ctx),
            stmt => result.push(stmt),
        }
        ControlFlow::Continue(())
    }

    fn join_sequence(
        a: &mut Expression<'a>,
        b: &mut Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let a = a.take_in(ctx);
        let b = b.take_in(ctx);
        if let Expression::SequenceExpression(mut sequence_expr) = a {
            // `(a, b); c`
            sequence_expr.expressions.push(b);
            return Expression::SequenceExpression(sequence_expr);
        }
        let span = a.span();
        let exprs = if let Expression::SequenceExpression(sequence_expr) = b {
            // `a; (b, c)`
            ArenaVec::from_iter_in(std::iter::once(a).chain(sequence_expr.unbox().expressions), ctx)
        } else {
            // `a; b`
            ArenaVec::from_array_in([a, b], ctx)
        };
        Expression::new_sequence_expression(span, exprs, ctx)
    }

    fn jump_stmts_look_the_same(left: &Statement<'a>, right: &Statement<'a>) -> bool {
        if left.is_jump_statement() && right.is_jump_statement() {
            return left.content_eq(right);
        }
        false
    }

    /// Whether control never falls through to the statement after this one in
    /// the same statement list (terser's `aborts`).
    ///
    /// Any abrupt completion qualifies, whatever its target: a `break` or
    /// `continue` transfers control past the end of the enclosing statement
    /// list, never to the next statement in it.
    ///
    /// Conservative on purpose:
    /// - A labeled statement completes normally when its body breaks out of
    ///   its own label (`a: { break a; }`), so it never counts.
    /// - Loops and switches count as completing normally; `while (true)`
    ///   without `break` is handled by loop-specific folds instead.
    fn statement_never_completes_normally(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_) => true,
            Statement::BlockStatement(block) => Self::block_never_completes_normally(block),
            Statement::IfStatement(if_stmt) => {
                if_stmt.alternate.as_ref().is_some_and(Self::statement_never_completes_normally)
                    && Self::statement_never_completes_normally(&if_stmt.consequent)
            }
            Statement::TryStatement(try_stmt) => {
                // A finalizer that aborts overrides however the other
                // blocks complete. Otherwise the try block must abort, and
                // so must the catch block when present (an exception
                // thrown before the try block's jump lands there).
                try_stmt.finalizer.as_ref().is_some_and(|f| Self::block_never_completes_normally(f))
                    || (Self::block_never_completes_normally(&try_stmt.block)
                        && try_stmt
                            .handler
                            .as_ref()
                            .is_none_or(|h| Self::block_never_completes_normally(&h.body)))
            }
            _ => false,
        }
    }

    fn block_never_completes_normally(block: &BlockStatement<'a>) -> bool {
        // A minimized dead zone keeps only hoisting survivors after the jump:
        // `function` declarations and the initializer-less `var` stub
        // re-emitted by `KeepVar`. Their bindings initialize at scope entry
        // and nothing after an aborting statement ever runs, so skip them
        // from the back before testing how the block completes.
        block
            .body
            .iter()
            .rev()
            .find(|stmt| match stmt {
                Statement::FunctionDeclaration(_) => false,
                Statement::VariableDeclaration(decl) => {
                    !(decl.kind.is_var() && decl.declarations.iter().all(|d| d.init.is_none()))
                }
                _ => true,
            })
            .is_some_and(Self::statement_never_completes_normally)
    }

    /// For variable declarations:
    /// * merge with the previous variable declarator if their kinds are the same
    /// * remove the variable declarator if it is unused
    /// * keep the initializer if it has side effects
    fn handle_variable_declaration(
        mut var_decl: ArenaBox<'a, VariableDeclaration<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(first_decl) = var_decl.declarations.first_mut()
            && let Some(first_decl_init) = first_decl.init.as_mut()
        {
            Self::substitute_single_use_symbol_in_statement(first_decl_init, result, ctx, false);
        }
        Self::substitute_single_use_symbol_within_declaration(
            var_decl.kind,
            &mut var_decl.declarations,
            ctx,
        );

        // If `join_vars` is off, but there are unused declarators ... just join them to make our code simpler.
        if !ctx.options().join_vars
            && var_decl.declarations.iter().all(|d| !Self::should_remove_unused_declarator(d, ctx))
        {
            result.push(Statement::VariableDeclaration(var_decl));
            return;
        }

        if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last()
            && var_decl.kind == prev_var_decl.kind
        {
            ctx.notice_change();
        }
        let VariableDeclaration { span, kind, declarations, declare, .. } = var_decl.unbox();
        for mut decl in declarations {
            if Self::should_remove_unused_declarator(&decl, ctx) {
                // `init` is `mut` because `remove_unused_expression` rewrites
                // it in place (peeling pure-call wrappers, etc). It is taken
                // out first because it may survive as an expression statement
                // — the declarator walk below must not mark its refs dead.
                if let Some(mut init) = decl.init.take() {
                    if Self::remove_unused_expression(&mut init, ctx) {
                        ctx.drop_expression(&init);
                    } else {
                        result.push(Statement::new_expression_statement(init.span(), init, ctx));
                    }
                }
                // Walk the rest of the dropped declarator (binding pattern +
                // TS type annotation, which can contain references). Also
                // records the mutation for the fixed-point loop driver.
                ctx.drop_variable_declarator(&decl);
            } else {
                if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last_mut()
                    && kind == prev_var_decl.kind
                {
                    prev_var_decl.declarations.push(decl);
                    continue;
                }
                let new_decl = VariableDeclaration::boxed(span, kind, [decl], declare, ctx);
                result.push(Statement::VariableDeclaration(new_decl));
            }
        }
    }

    /// Whether an expression is or contains a `super()` call at the top level
    /// (i.e., in a sequence expression, but not nested inside conditionals/functions).
    fn expression_contains_super_call(expr: &Expression<'a>) -> bool {
        match expr {
            _ if expr.is_super_call_expression() => true,
            Expression::SequenceExpression(seq) => {
                seq.expressions.iter().any(Expression::is_super_call_expression)
            }
            _ => false,
        }
    }

    fn handle_expression_statement(
        mut expr_stmt: ArenaBox<'a, ExpressionStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::substitute_single_use_symbol_in_statement(
            &mut expr_stmt.expression,
            result,
            ctx,
            false,
        );

        // In a derived constructor, `this` after an unconditional `super()` is safe to drop.
        // Walk backwards through preceding sibling statements looking for `super()`.
        // Only consider top-level expression statements — `super()` inside `if`/loops
        // is conditional and doesn't guarantee `this` is initialized.
        if matches!(expr_stmt.expression, Expression::ThisExpression(_))
            && Self::this_is_inside_derived_constructor(ctx)
            && result.iter().rev().any(|stmt| {
                matches!(
                    stmt,
                    Statement::ExpressionStatement(prev)
                        if Self::expression_contains_super_call(&prev.expression)
                )
            })
        {
            let dropped = Statement::ExpressionStatement(expr_stmt);
            ctx.drop_statement(&dropped);
            return;
        }

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
        {
            let a = &mut prev_expr_stmt.expression;
            let b = &mut expr_stmt.expression;
            expr_stmt.expression = Self::join_sequence(a, b, ctx);
            let dropped = result.pop().unwrap();
            ctx.drop_statement(&dropped);
        }
        // "var a; a = b();" => "var a = b();"
        match &mut expr_stmt.expression {
            Expression::AssignmentExpression(assign_expr) => {
                let merged = Self::merge_assignment_to_declaration(assign_expr, result, ctx);
                if merged {
                    let dropped = Statement::ExpressionStatement(expr_stmt);
                    ctx.drop_statement(&dropped);
                    return;
                }
            }
            Expression::SequenceExpression(sequence_expr)
                if result
                    .last()
                    .is_some_and(|stmt| matches!(stmt, Statement::VariableDeclaration(_))) =>
            {
                let first_non_merged_index =
                    sequence_expr.expressions.iter_mut().position(|expr| {
                        if let Expression::AssignmentExpression(assign_expr) = expr {
                            !Self::merge_assignment_to_declaration(assign_expr, result, ctx)
                        } else {
                            true
                        }
                    });
                let sequence_len = sequence_expr.expressions.len();
                match first_non_merged_index {
                    None => {
                        // all elements are merged
                        let dropped = Statement::ExpressionStatement(expr_stmt);
                        ctx.drop_statement(&dropped);
                        return;
                    }
                    Some(val) if val == sequence_len - 1 => {
                        // all elements are merged except for the last expression
                        let last_expr = sequence_expr.expressions.pop().unwrap();
                        result.push(Statement::new_expression_statement(
                            last_expr.span(),
                            last_expr,
                            ctx,
                        ));
                        let dropped = Statement::ExpressionStatement(expr_stmt);
                        ctx.drop_statement(&dropped);
                        return;
                    }
                    Some(0) => {
                        // no elements are merged
                    }
                    Some(val) => {
                        for dropped in sequence_expr.expressions.drain(0..val) {
                            ctx.drop_expression(&dropped);
                        }
                    }
                }
            }
            _ => {}
        }

        result.push(Statement::ExpressionStatement(expr_stmt));
    }

    fn merge_assignment_to_declaration(
        assign_expr: &mut AssignmentExpression<'a>,
        result: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        if assign_expr.operator != AssignmentOperator::Assign {
            return false;
        }
        let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign_expr.left else {
            return false;
        };
        let Some(Statement::VariableDeclaration(var_decl)) = result.last_mut() else {
            return false;
        };
        if !matches!(&var_decl.kind, VariableDeclarationKind::Var | VariableDeclarationKind::Let) {
            return false;
        }
        for decl in var_decl.declarations.iter_mut().rev() {
            let BindingPattern::BindingIdentifier(kind) = &decl.id else {
                break;
            };
            if kind.name == id.name {
                if decl.init.is_none()
                    && (decl.kind == VariableDeclarationKind::Var
                        || assign_expr.right.is_literal_value(true, ctx))
                {
                    // "var a; a = b();" => "var a = b();"
                    decl.init = Some(assign_expr.right.take_in(ctx));
                    return true;
                }
                // Note it is not possible to compress like:
                // - "var a = b(); a = c();" => "var a = (b(), c());"
                //   This is not possible as we need to consider cases when `c()` accesses `a`
                // - "var a = 1; a = b();" => "var a = b();"
                //   This is not possible as we need to consider cases when `b()` accesses `a`
                // - "let a; a = foo(a);" => "let a = foo(a);"
                //   This is not possible as TDZ error would be introduced
                break;
            }
            // should not move assignment above variables with initializer to keep the execution order
            if decl.init.is_some() {
                break;
            }
            // should not move assignment above other variables for let
            // this could cause TDZ errors (e.g. `let a, b; b = a;`)
            if decl.kind == VariableDeclarationKind::Let {
                break;
            }
        }
        false
    }

    fn is_switch_case_removable(stmt: &SwitchCase, allow_break: bool) -> bool {
        let is_empty = if stmt.consequent.len() == 1 {
            match stmt.consequent.last() {
                Some(Statement::EmptyStatement(_)) => true,
                Some(Statement::BreakStatement(break_stmt)) => {
                    allow_break && break_stmt.label.is_none()
                }
                _ => false,
            }
        } else {
            stmt.consequent.is_empty()
        };

        is_empty && stmt.test.as_ref().is_none_or(Expression::is_literal)
    }

    fn handle_switch_statement(
        mut switch_stmt: ArenaBox<'a, SwitchStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::substitute_single_use_symbol_in_statement(
            &mut switch_stmt.discriminant,
            result,
            ctx,
            false,
        );

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
        {
            let a = &mut prev_expr_stmt.expression;
            let b = &mut switch_stmt.discriminant;
            switch_stmt.discriminant = Self::join_sequence(a, b, ctx);
            let dropped = result.pop().unwrap();
            ctx.drop_statement(&dropped);
        }

        // Remove empty case clauses that don't affect behavior.
        // Handles fall-through semantics: remove empty cases before default or at end (if no default).
        // e.g., `switch(x){ case 0: foo(); break; case 1: default: bar() }`
        // => `switch(x){ case 0: foo(); break; default: bar() }`
        // https://github.com/evanw/esbuild/commit/add452ed51333953dd38a26f28a775bb220ea2e9
        let case_count = switch_stmt.cases.len();
        if case_count == 1 {
            // Remove sole case if empty and has no side-effect test
            if Self::is_switch_case_removable(&switch_stmt.cases[0], true) {
                ctx.drop_switch_case(&switch_stmt.cases.pop().unwrap());
            }
        } else if case_count > 1 {
            // Determine the range [0, end] to check for removable cases.
            // 1. default exists and is empty: check the full switch.
            // 2. default exists and is non-removable and last: check only cases before that default.
            // 3. default exists, is non-removable, and is not last: skip this optimization (`end = 0`).
            // 4. no default case: check the full switch and allow a trailing unlabeled `break`.
            let default_pos = switch_stmt.cases.iter().rposition(SwitchCase::is_default_case);
            let (end, allow_break) = if let Some(default_pos) = default_pos {
                if Self::is_switch_case_removable(&switch_stmt.cases[default_pos], true) {
                    (case_count, true)
                } else if default_pos == case_count - 1 {
                    (default_pos, false)
                } else {
                    (0, false)
                }
            } else {
                (case_count, true)
            };

            if end > 0 {
                // Last non-removable case index in [0, end]. Returns None if all cases are removable.
                let last_non_removable_case_before_end = switch_stmt.cases[..end]
                    .iter()
                    .rposition(|case| !Self::is_switch_case_removable(case, allow_break));

                // Calculate the start of the removable suffix.
                // 1. next case after last non-removable: remove from pos + 1
                // 2. no non-removable case: all cases are removable, start from 0
                let start = match last_non_removable_case_before_end {
                    Some(pos) => pos + 1,
                    None => 0,
                };

                // Remove the removable suffix if any
                if start < end && default_pos.is_none_or(|pos| pos >= start) {
                    for removed_case in switch_stmt.cases.drain(start..end) {
                        ctx.drop_switch_case(&removed_case);
                    }
                }
            }
        }

        if switch_stmt.cases.is_empty() {
            result.push(Statement::new_expression_statement(
                switch_stmt.span,
                switch_stmt.discriminant.take_in(ctx),
                ctx,
            ));
            return;
        } else if let Some(last_case) = switch_stmt.cases.last_mut()
            && let Some(Statement::BreakStatement(last_break)) = last_case.consequent.last()
            && last_break.label.is_none()
        {
            let dropped = last_case.consequent.pop().unwrap();
            ctx.drop_statement(&dropped);
        }

        result.push(Statement::SwitchStatement(switch_stmt));
    }

    fn handle_if_statement(
        i: usize,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        mut if_stmt: ArenaBox<'a, IfStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) -> ControlFlow<()> {
        Self::substitute_single_use_symbol_in_statement(&mut if_stmt.test, result, ctx, false);

        // Absorb a previous expression statement
        if ctx.options().sequences {
            if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                let a = &mut prev_expr_stmt.expression;
                let b = &mut if_stmt.test;
                if_stmt.test = Self::join_sequence(a, b, ctx);
                let dropped = result.pop().unwrap();
                ctx.drop_statement(&dropped);
            }

            if if_stmt.consequent.is_jump_statement() {
                // Absorb a previous if statement
                if let Some(Statement::IfStatement(prev_if_stmt)) = result.last_mut()
                    && prev_if_stmt.alternate.is_none()
                    && Self::jump_stmts_look_the_same(&prev_if_stmt.consequent, &if_stmt.consequent)
                {
                    // "if (a) break c; if (b) break c;" => "if (a || b) break c;"
                    // "if (a) continue c; if (b) continue c;" => "if (a || b) continue c;"
                    // "if (a) return c; if (b) return c;" => "if (a || b) return c;"
                    // "if (a) throw c; if (b) throw c;" => "if (a || b) throw c;"
                    if_stmt.test = Self::join_with_left_associative_op(
                        if_stmt.test.span(),
                        LogicalOperator::Or,
                        prev_if_stmt.test.take_in(ctx),
                        if_stmt.test.take_in(ctx),
                        ctx,
                    );
                    let dropped = result.pop().unwrap();
                    ctx.drop_statement(&dropped);
                }

                let mut optimize_implicit_jump = false;
                // "while (x) { if (y) continue; z(); }" => "while (x) { if (!y) z(); }"
                // "while (x) { if (y) continue; else z(); w(); }" => "while (x) { if (!y) { z(); w(); } }" => "for (; x;) !y && (z(), w());"
                if ctx.ancestors().nth(1).is_some_and(|v| {
                    v.is_for_statement() || v.is_for_in_statement() || v.is_for_of_statement()
                }) && let Statement::ContinueStatement(continue_stmt) = &if_stmt.consequent
                    && continue_stmt.label.is_none()
                {
                    optimize_implicit_jump = true;
                }

                // "let x = () => { if (y) return; z(); };" => "let x = () => { if (!y) z(); };"
                // "let x = () => { if (y) return; else z(); w(); };" => "let x = () => { if (!y) { z(); w(); } };" => "let x = () => { !y && (z(), w()); };"
                if ctx.parent().is_function_body()
                    && let Statement::ReturnStatement(return_stmt) = &if_stmt.consequent
                    && return_stmt.argument.is_none()
                {
                    optimize_implicit_jump = true;
                }
                if optimize_implicit_jump {
                    // Don't do this transformation if the branch condition could
                    // potentially access symbols declared later on on this scope below.
                    // If so, inverting the branch condition and nesting statements after
                    // this in a block would break that access which is a behavior change.
                    //
                    //   // This transformation is incorrect
                    //   if (a()) return; function a() {}
                    //   if (!a()) { function a() {} }
                    //
                    //   // This transformation is incorrect
                    //   if (a(() => b)) return; let b;
                    //   if (a(() => b)) { let b; }
                    //
                    let mut can_move_branch_condition_outside_scope = true;
                    if let Some(alternate) = &if_stmt.alternate
                        && Self::statement_cares_about_scope(alternate)
                    {
                        can_move_branch_condition_outside_scope = false;
                    }
                    if let Some(stmts) = stmts.get(i + 1..) {
                        for stmt in stmts {
                            if Self::statement_cares_about_scope(stmt) {
                                can_move_branch_condition_outside_scope = false;
                                break;
                            }
                        }
                    }

                    if can_move_branch_condition_outside_scope {
                        let drained_stmts = stmts.drain(i + 1..);
                        let mut body = if let Some(alternate) = if_stmt.alternate.take() {
                            ArenaVec::from_iter_in(iter::once(alternate).chain(drained_stmts), ctx)
                        } else {
                            ArenaVec::from_iter_in(drained_stmts, ctx)
                        };

                        Self::minimize_statements(&mut body, ctx);
                        let span = if body.is_empty() {
                            if_stmt.consequent.span()
                        } else {
                            body[0].span()
                        };
                        let test = if_stmt.test.take_in(ctx);
                        let mut test = Self::minimize_not(test.span(), test, ctx);
                        Self::minimize_expression_in_boolean_context(&mut test, ctx);
                        let consequent = if body.len() == 1 {
                            body.remove(0)
                        } else {
                            let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
                            Statement::new_block_statement_with_scope_id(span, body, scope_id, ctx)
                        };
                        let mut if_stmt =
                            IfStatement::new(test.span(), test, consequent, None, ctx);
                        let if_stmt =
                            Self::try_minimize_if(&mut if_stmt, ctx).unwrap_or_else(|| {
                                Statement::IfStatement(ArenaBox::new_in(if_stmt, ctx))
                            });
                        result.push(if_stmt);
                        ctx.notice_change();
                        return ControlFlow::Break(());
                    }
                }

                if if_stmt.alternate.is_some() {
                    // "if (a) return b; else if (c) return d; else return e;" => "if (a) return b; if (c) return d; return e;"
                    result.push(Statement::IfStatement(if_stmt));
                    loop {
                        if let Some(Statement::IfStatement(if_stmt)) = result.last_mut()
                            && if_stmt.consequent.is_jump_statement()
                            && let Some(stmt) = if_stmt.alternate.take()
                        {
                            if let Statement::BlockStatement(block_stmt) = stmt {
                                Self::handle_block(result, block_stmt, ctx);
                            } else {
                                result.push(stmt);
                                ctx.notice_change();
                            }
                            continue;
                        }
                        break;
                    }
                    return ControlFlow::Continue(());
                }
            }
        }

        result.push(Statement::IfStatement(if_stmt));
        ControlFlow::Continue(())
    }

    fn handle_return_statement(
        mut ret_stmt: ArenaBox<'a, ReturnStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(ret_argument_expr) = &mut ret_stmt.argument {
            Self::substitute_single_use_symbol_in_statement(ret_argument_expr, result, ctx, false);
        }

        if let Some(argument) = &mut ret_stmt.argument
            && argument.value_type(ctx) == ValueType::Undefined
            // `return undefined` has a different semantic in async generator function.
            && !ctx.is_closest_function_scope_an_async_generator()
        {
            if argument.may_have_side_effects(ctx) {
                if ctx.options().sequences
                    && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
                {
                    let a = &mut prev_expr_stmt.expression;
                    prev_expr_stmt.expression = Self::join_sequence(a, argument, ctx);
                } else {
                    result.push(Statement::new_expression_statement(
                        argument.span(),
                        argument.take_in(ctx),
                        ctx,
                    ));
                }
            }
            if let Some(old) = ret_stmt.argument.take() {
                ctx.drop_expression(&old);
            }
            result.push(Statement::ReturnStatement(ret_stmt));
            return;
        }

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
            && let Some(argument) = &mut ret_stmt.argument
        {
            let a = &mut prev_expr_stmt.expression;
            let new_arg = Self::join_sequence(a, argument, ctx);
            ctx.replace_expression(argument, new_arg);
            result.pop();
        }
        result.push(Statement::ReturnStatement(ret_stmt));
    }

    fn handle_throw_statement(
        mut throw_stmt: ArenaBox<'a, ThrowStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::substitute_single_use_symbol_in_statement(
            &mut throw_stmt.argument,
            result,
            ctx,
            false,
        );

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
        {
            let a = &mut prev_expr_stmt.expression;
            let b = &mut throw_stmt.argument;
            throw_stmt.argument = Self::join_sequence(a, b, ctx);
            let dropped = result.pop().unwrap();
            ctx.drop_statement(&dropped);
        }
        result.push(Statement::ThrowStatement(throw_stmt));
    }

    fn handle_for_statement(
        mut for_stmt: ArenaBox<'a, ForStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(init) = &mut for_stmt.init {
            match init {
                ForStatementInit::VariableDeclaration(var_decl) => {
                    if let Some(first_decl) = var_decl.declarations.first_mut()
                        && let Some(first_decl_init) = first_decl.init.as_mut()
                    {
                        let is_block_scoped_decl = !first_decl.kind.is_var();
                        Self::substitute_single_use_symbol_in_statement(
                            first_decl_init,
                            result,
                            ctx,
                            is_block_scoped_decl,
                        );
                    }
                    Self::substitute_single_use_symbol_within_declaration(
                        var_decl.kind,
                        &mut var_decl.declarations,
                        ctx,
                    );
                }
                match_expression!(ForStatementInit) => {
                    let init = init.to_expression_mut();
                    Self::substitute_single_use_symbol_in_statement(init, result, ctx, false);
                }
            }
        }

        if let Some(ForStatementInit::VariableDeclaration(var_decl)) = &mut for_stmt.init {
            let old_len = var_decl.declarations.len();
            var_decl.declarations.retain_mut(|decl| {
                let should_keep = !Self::should_remove_unused_declarator(decl, ctx)
                    || decl
                        .init
                        .as_ref()
                        .is_some_and(|init| Self::has_side_effects_or_preserved_iife(init, ctx));
                if !should_keep {
                    // Same leak hazard as `remove_unused_variable_declaration`:
                    // the `retain` silently drops the declarator, so its refs
                    // (init and TS type annotation) need an explicit walk to
                    // reach `PassChanges`.
                    ctx.drop_variable_declarator(decl);
                }
                should_keep
            });
            if old_len != var_decl.declarations.len() {
                if var_decl.declarations.is_empty() {
                    for_stmt.init = None;
                }
                ctx.notice_change();
            }
        }

        if ctx.options().sequences {
            match result.last_mut() {
                Some(Statement::ExpressionStatement(prev_expr_stmt)) => {
                    if let Some(init) = &mut for_stmt.init {
                        if let Some(init) = init.as_expression_mut() {
                            let a = &mut prev_expr_stmt.expression;
                            let new_init = Self::join_sequence(a, init, ctx);
                            ctx.replace_expression(init, new_init);
                            let dropped = result.pop().unwrap();
                            ctx.drop_statement(&dropped);
                        }
                    } else {
                        for_stmt.init =
                            Some(ForStatementInit::from(prev_expr_stmt.expression.take_in(ctx)));
                        let dropped = result.pop().unwrap();
                        ctx.drop_statement(&dropped);
                    }
                }
                Some(Statement::VariableDeclaration(prev_var_decl)) => {
                    if let Some(init) = &mut for_stmt.init {
                        if prev_var_decl.kind.is_var()
                            && let ForStatementInit::VariableDeclaration(var_decl) = init
                            && var_decl.kind.is_var()
                        {
                            var_decl
                                .declarations
                                .splice(0..0, prev_var_decl.declarations.drain(..));
                            let dropped = result.pop().unwrap();
                            ctx.drop_statement(&dropped);
                        }
                    } else if prev_var_decl.kind.is_var() {
                        let Some(Statement::VariableDeclaration(prev_var_decl)) = result.pop()
                        else {
                            unreachable!()
                        };
                        for_stmt.init = Some(ForStatementInit::VariableDeclaration(prev_var_decl));
                        ctx.notice_change();
                    }
                }
                _ => {}
            }
        }
        result.push(Statement::ForStatement(for_stmt));
    }

    fn handle_for_in_statement(
        mut for_in_stmt: ArenaBox<'a, ForInStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,

        ctx: &mut TraverseCtx<'a>,
    ) {
        // Annex B.3.5 allows initializers in non-strict mode
        // <https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-initializers-in-forin-statement-heads>
        // That is evaluated before the right hand side is evaluated. So, in that case, skip the single use substitution.
        if !matches!(&for_in_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if var_decl.has_init())
        {
            let is_block_scoped_decl = matches!(&for_in_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if !var_decl.kind.is_var());
            Self::substitute_single_use_symbol_in_statement(
                &mut for_in_stmt.right,
                result,
                ctx,
                is_block_scoped_decl,
            );
        }

        if ctx.options().sequences {
            match result.last_mut() {
                // "a; for (var b in c) d" => "for (var b in a, c) d"
                Some(Statement::ExpressionStatement(prev_expr_stmt)) => {
                    // Annex B.3.5 allows initializers in non-strict mode
                    // <https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-initializers-in-forin-statement-heads>
                    // If there's a side-effectful initializer, we should not move the previous statement inside.
                    let has_side_effectful_initializer = {
                        if let ForStatementLeft::VariableDeclaration(var_decl) = &for_in_stmt.left {
                            if var_decl.declarations.len() == 1 {
                                // only var can have a initializer
                                var_decl.kind.is_var()
                                    && var_decl.declarations[0]
                                        .init
                                        .as_ref()
                                        .is_some_and(|init| init.may_have_side_effects(ctx))
                            } else {
                                // the spec does not allow multiple declarations though
                                true
                            }
                        } else {
                            false
                        }
                    };
                    // Only allow inlining when the for-in variable is declared with `var`.
                    // Block-scoped declarations (let/const) can cause variable shadowing issues
                    // where the inlined expression might reference a variable with the same name
                    // as the for-in variable, but after inlining, it would incorrectly refer to
                    // the shadowed for-in variable instead.
                    // See: https://github.com/oxc-project/oxc/issues/18650
                    let is_block_scoped = matches!(&for_in_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if !var_decl.kind.is_var());
                    if !has_side_effectful_initializer && !is_block_scoped {
                        let a = &mut prev_expr_stmt.expression;
                        for_in_stmt.right = Self::join_sequence(a, &mut for_in_stmt.right, ctx);
                        let dropped = result.pop().unwrap();
                        ctx.drop_statement(&dropped);
                    }
                }
                // "var a; for (a in b) c" => "for (var a in b) c"
                Some(Statement::VariableDeclaration(prev_var_decl)) => {
                    if let ForStatementLeft::AssignmentTargetIdentifier(id) = &for_in_stmt.left {
                        let prev_var_decl_no_init_item = {
                            if prev_var_decl.kind.is_var()
                                && prev_var_decl.declarations.len() == 1
                                && prev_var_decl.declarations[0].init.is_none()
                            {
                                Some(&prev_var_decl.declarations[0])
                            } else {
                                None
                            }
                        };
                        if let Some(prev_var_decl_item) = prev_var_decl_no_init_item
                            && let BindingPattern::BindingIdentifier(decl_id) =
                                &prev_var_decl_item.id
                            && id.name == decl_id.name
                        {
                            let Some(Statement::VariableDeclaration(prev_var_decl)) = result.pop()
                            else {
                                unreachable!()
                            };
                            let new_left = ForStatementLeft::VariableDeclaration(prev_var_decl);
                            ctx.replace_for_statement_left(&mut for_in_stmt.left, new_left);
                        }
                    }
                }
                _ => {}
            }
        }
        result.push(Statement::ForInStatement(for_in_stmt));
    }

    fn handle_for_of_statement(
        mut for_of_stmt: ArenaBox<'a, ForOfStatement<'a>>,
        result: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let is_block_scoped_decl = matches!(&for_of_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if !var_decl.kind.is_var());
        Self::substitute_single_use_symbol_in_statement(
            &mut for_of_stmt.right,
            result,
            ctx,
            is_block_scoped_decl,
        );

        // "var a; for (a of b) c" => "for (var a of b) c"
        if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last_mut()
            && let ForStatementLeft::AssignmentTargetIdentifier(id) = &for_of_stmt.left
        {
            let prev_var_decl_no_init_item = {
                if prev_var_decl.kind.is_var()
                    && prev_var_decl.declarations.len() == 1
                    && prev_var_decl.declarations[0].init.is_none()
                {
                    Some(&prev_var_decl.declarations[0])
                } else {
                    None
                }
            };
            if let Some(prev_var_decl_item) = prev_var_decl_no_init_item
                && let BindingPattern::BindingIdentifier(decl_id) = &prev_var_decl_item.id
                && id.name == decl_id.name
            {
                let Some(Statement::VariableDeclaration(prev_var_decl)) = result.pop() else {
                    unreachable!()
                };
                let new_left = ForStatementLeft::VariableDeclaration(prev_var_decl);
                ctx.replace_for_statement_left(&mut for_of_stmt.left, new_left);
            }
        }
        result.push(Statement::ForOfStatement(for_of_stmt));
    }

    /// `appendIfOrLabelBodyPreservingScope`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9852>
    fn handle_block(
        result: &mut ArenaVec<'a, Statement<'a>>,
        block_stmt: ArenaBox<'a, BlockStatement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let keep_block = block_stmt.body.iter().any(Self::statement_cares_about_scope);
        if keep_block {
            result.push(Statement::BlockStatement(block_stmt));
        } else {
            result.append(&mut block_stmt.unbox().body);
            ctx.notice_change();
        }
    }

    /// `statementCaresAboutScope`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9767>
    pub fn statement_cares_about_scope(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::BlockStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::ExpressionStatement(_)
            | Statement::IfStatement(_)
            | Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::DoWhileStatement(_)
            | Statement::WhileStatement(_)
            | Statement::WithStatement(_)
            | Statement::TryStatement(_)
            | Statement::SwitchStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::LabeledStatement(_) => false,
            Statement::VariableDeclaration(decl) => !decl.kind.is_var(),
            _ => true,
        }
    }

    /// Inline single-use variable declarations where possible:
    /// ```js
    /// // before
    /// let x = fn();
    /// return x.y();
    ///
    /// // after
    /// return fn().y();
    /// ```
    ///
    /// part of `mangleStmts`: <https://github.com/evanw/esbuild/blob/v0.25.9/internal/js_parser/js_parser.go#L9111-L9189>
    /// `substituteSingleUseSymbolInStmt`: <https://github.com/evanw/esbuild/blob/v0.25.9/internal/js_parser/js_parser.go#L9583>
    fn substitute_single_use_symbol_in_statement(
        expr_in_stmt: &mut Expression<'a>,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
        non_scoped_literal_only: bool,
    ) -> bool {
        if Self::is_script_root_scope(ctx) || ctx.current_scope_flags().contains_direct_eval() {
            return false;
        }

        let mut inlined = false;
        while let Some(Statement::VariableDeclaration(prev_var_decl)) = stmts.last_mut() {
            if prev_var_decl.kind.is_using() {
                break;
            }
            let old_len = prev_var_decl.declarations.len();
            let new_len = Self::substitute_single_use_symbol_in_expression_from_declarators(
                expr_in_stmt,
                &mut prev_var_decl.declarations,
                ctx,
                non_scoped_literal_only,
            );
            // The inlined declarators' inits were already taken out by the
            // substitution, but the discarded declarators themselves still
            // need a drop walk — their TS type annotations can hold resolved
            // references (computed keys in a type literal).
            if new_len == 0 {
                inlined = true;
                let dropped = stmts.pop().unwrap();
                ctx.drop_statement(&dropped);
            } else if old_len != new_len {
                inlined = true;
                for decl in prev_var_decl.declarations.drain(new_len..) {
                    ctx.drop_variable_declarator(&decl);
                }
                break;
            } else {
                break;
            }
        }
        inlined
    }

    fn substitute_single_use_symbol_within_declaration(
        kind: VariableDeclarationKind,
        declarations: &mut ArenaVec<'a, VariableDeclarator<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        if Self::is_script_root_scope(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
            || kind.is_using()
        {
            return false;
        }

        let mut changed = false;
        let mut i = 1;
        while i < declarations.len() {
            let (prev_decls, [decl, ..]) = declarations.split_at_mut(i) else { unreachable!() };
            let Some(decl_init) = &mut decl.init else {
                i += 1;
                continue;
            };
            let old_len = prev_decls.len();
            let new_len = Self::substitute_single_use_symbol_in_expression_from_declarators(
                decl_init, prev_decls, ctx, false,
            );
            if old_len != new_len {
                changed = true;
                let drop_count = old_len - new_len;
                // Same drop-walk requirement as the truncate above: the
                // drained declarators' type annotations can hold references.
                for decl in declarations.drain(i - drop_count..i) {
                    ctx.drop_variable_declarator(&decl);
                }
                i -= drop_count;
            }
            i += 1;
        }
        changed
    }

    /// Returns new length.
    ///
    /// CONTRACT: the consumed suffix `declarators[new_len..]` is the caller's
    /// to discard, and the caller must route each discarded declarator through
    /// `ctx.drop_variable_declarator` (or an enclosing `drop_statement`) — the
    /// inlined inits are already taken out, but binding patterns and TS type
    /// annotations can still hold resolved references that would otherwise
    /// leak past the incremental scoping refresh.
    fn substitute_single_use_symbol_in_expression_from_declarators(
        target_expr: &mut Expression<'a>,
        declarators: &mut [VariableDeclarator<'a>],
        ctx: &mut TraverseCtx<'a>,
        non_scoped_literal_only: bool,
    ) -> usize {
        let last_non_inlined_index = declarators.iter_mut().rposition(|prev_decl| {
            let Some(prev_decl_init) = &mut prev_decl.init else {
                return true;
            };
            let BindingPattern::BindingIdentifier(prev_decl_id) = &prev_decl.id else {
                return true;
            };
            // Don't inline `var e` inside `catch (e) { ... }`. Removing the var declarator
            // would lose the function-scoped hoisting that `var` provides. The catch parameter
            // and the var share one symbol (with CatchVariable flag) due to the redeclaration
            // semantics in https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
            if ctx.scoping().symbol_flags(prev_decl_id.symbol_id()).is_catch_variable() {
                return true;
            }
            if ctx.is_expression_whose_name_needs_to_be_kept(prev_decl_init) {
                return true;
            }
            let Some(symbol_value) = ctx.state.symbols.value(prev_decl_id.symbol_id()) else {
                return true;
            };
            // Implicitly observable bindings remain live independently of
            // their resolved-reference count.
            // An `export { foo }` specifier also contributes a reference, but
            // consult the shared metadata explicitly for consistency with the
            // other count-based consumers.
            if ctx.state.symbols.is_implicitly_observable(prev_decl_id.symbol_id())
                || symbol_value.references.has_multiple_reads()
                || symbol_value.references.has_writes()
            {
                return true;
            }
            if non_scoped_literal_only && !prev_decl_init.is_literal_value(false, ctx) {
                return true;
            }
            let replaced = Self::substitute_single_use_symbol_in_expression(
                target_expr,
                &prev_decl_id.name,
                prev_decl_init,
                prev_decl_init.may_have_side_effects(ctx),
                ctx,
            );
            if replaced != Some(true) {
                return true;
            }
            false
        });
        match last_non_inlined_index {
            None => 0,
            Some(last_non_inlined_index) => last_non_inlined_index + 1,
        }
    }

    /// Whether reordering this read before a side-effecting replacement could
    /// observe `symbol_id` in its Temporal Dead Zone.
    ///
    /// The hazard is a block-scoped binding (`let`/`const`/`using`/`class`/`enum`)
    /// closed over from an enclosing function: the function can suspend at an
    /// `await`/`yield` while outer code initializes the binding, so the earlier
    /// (reordered) read hits the TDZ where the original would not. A same-function
    /// binding can't be initialized mid-suspension, so it stays inlinable.
    ///
    /// <https://github.com/rolldown/rolldown/issues/9959>
    fn is_tdz_closed_over_read(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        ctx.scoping().symbol_flags(symbol_id).is_block_scoped()
            && Self::read_crosses_function_boundary(
                ctx.current_scope_id(),
                ctx.scoping().symbol_scope_id(symbol_id),
                ctx,
            )
    }

    /// Whether reordering a side-effecting replacement past this member
    /// assignment-target object is unsafe. The object is evaluated before the
    /// replacement, so it is unsafe if its reference may change, or if it reads
    /// a closed-over lexical that could be in its TDZ (e.g. `v.x = await f()`
    /// reading `v` before the await). See [`Self::is_tdz_closed_over_read`].
    fn member_object_blocks_reorder(object: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        Self::is_expression_that_reference_may_change(object, ctx)
            || matches!(object, Expression::Identifier(id)
                if ctx
                    .scoping()
                    .get_reference(id.reference_id())
                    .symbol_id()
                    .is_some_and(|symbol_id| Self::is_tdz_closed_over_read(symbol_id, ctx)))
    }

    /// Returns Some(true) when the expression is successfully replaced.
    /// Returns Some(false) when the expression is not replaced, and cannot try the subsequent expressions.
    /// Return None when the expression is not replaced, and can try the subsequent expressions.
    ///
    /// `substituteSingleUseSymbolInExpr`: <https://github.com/evanw/esbuild/blob/v0.25.9/internal/js_parser/js_parser.go#L9642>
    fn substitute_single_use_symbol_in_expression(
        target_expr: &mut Expression<'a>,
        search_for: &str,
        replacement: &mut Expression<'a>,
        replacement_has_side_effect: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<bool> {
        match target_expr {
            Expression::Identifier(id) => {
                if id.name == search_for {
                    // Preserve the span of the target identifier so that comments
                    // attached to it (via `attached_to`) remain correctly associated
                    // with the replacement expression.
                    // https://github.com/rolldown/rolldown/issues/8248
                    let target_span = target_expr.span();
                    let mut new_expr = replacement.take_in(ctx);
                    // Span fix-up on the still-owned new node before the slot replacement, not the slot replacement itself.
                    *new_expr.span_mut() = target_span;
                    ctx.replace_expression(target_expr, new_expr);
                    return Some(true);
                }
                // If the identifier is not a getter and the identifier is read-only,
                // we know that the value is same even if we reordered the expression.
                //
                // But a lexical binding that is closed over from an enclosing
                // function/module scope may still be in its Temporal Dead Zone
                // when this function runs (e.g. the function is called before the
                // binding's declaration executes). Reordering its read earlier —
                // in particular before a side-effecting replacement such as an
                // `await` — can surface a `ReferenceError` that the original order
                // avoids. https://github.com/rolldown/rolldown/issues/9959
                if let Some(symbol_id) = ctx.scoping().get_reference(id.reference_id()).symbol_id()
                    && !Self::is_symbol_mutated(symbol_id, ctx)
                    && !Self::is_tdz_closed_over_read(symbol_id, ctx)
                {
                    return None;
                }
            }
            Expression::AwaitExpression(await_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut await_expr.argument,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(argument) = &mut yield_expr.argument
                    && let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        argument,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    )
                {
                    return Some(changed);
                }
            }
            Expression::ImportExpression(import_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut import_expr.source,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }

                // The "import()" expression has side effects but the side effects are
                // always asynchronous so there is no way for the side effects to modify
                // the replacement value. So it's ok to reorder the replacement value
                // past the "import()" expression assuming everything else checks out.
                if !replacement_has_side_effect && !import_expr.source.may_have_side_effects(ctx) {
                    return None;
                }
            }
            Expression::UnaryExpression(unary_expr) => {
                if unary_expr.operator != UnaryOperator::Delete
                    && let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        &mut unary_expr.argument,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    )
                {
                    return Some(changed);
                }
            }
            Expression::StaticMemberExpression(member_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut member_expr.object,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
            }
            Expression::BinaryExpression(binary_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut binary_expr.left,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut binary_expr.right,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
            }
            Expression::AssignmentExpression(assign_expr) => {
                if assign_expr.left.may_have_side_effects(ctx) {
                    // Do not reorder past a side effect in an assignment target, as that may
                    // change the replacement value. For example, "fn()" may change "a" here:
                    // ```js
                    // let a = 1;
                    // foo[fn()] = a;
                    // ```
                    return Some(false);
                }
                if assign_expr.operator != AssignmentOperator::Assign && replacement_has_side_effect
                {
                    // If this is a read-modify-write assignment and the replacement has side
                    // effects, don't reorder it past the assignment target. The assignment
                    // target is being read so it may be changed by the side effect. For
                    // example, "fn()" may change "foo" here:
                    // ```js
                    // let a = fn();
                    // foo += a;
                    // ```
                    return Some(false);
                }
                if replacement_has_side_effect {
                    // If the assignment target may depend on side effects of the replacement,
                    // don't reorder it past the assignment target. The non-last part of the
                    // assignment target is evaluated before the assignment evaluation so that
                    // part may be changed by the side effect. For example, "fn()" may change
                    // "foo" here:
                    // ```js
                    // let a = fn();
                    // foo.bar = a;
                    // ```
                    let may_depend_on_side_effect = match &assign_expr.left {
                        AssignmentTarget::AssignmentTargetIdentifier(_) => false,
                        AssignmentTarget::ComputedMemberExpression(member_expr) => {
                            Self::member_object_blocks_reorder(&member_expr.object, ctx)
                        }
                        AssignmentTarget::PrivateFieldExpression(member_expr) => {
                            Self::member_object_blocks_reorder(&member_expr.object, ctx)
                        }
                        AssignmentTarget::StaticMemberExpression(member_expr) => {
                            Self::member_object_blocks_reorder(&member_expr.object, ctx)
                        }
                        AssignmentTarget::ArrayAssignmentTarget(_)
                        | AssignmentTarget::ObjectAssignmentTarget(_)
                        | AssignmentTarget::TSAsExpression(_)
                        | AssignmentTarget::TSNonNullExpression(_)
                        | AssignmentTarget::TSSatisfiesExpression(_)
                        | AssignmentTarget::TSTypeAssertion(_) => true,
                    };
                    if may_depend_on_side_effect {
                        return Some(false);
                    }
                }
                // If we get here then it should be safe to attempt to substitute the
                // replacement past the left operand into the right operand.
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut assign_expr.right,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
            }
            Expression::LogicalExpression(logical_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut logical_expr.left,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
                // Do not substitute our unconditionally-executed value into a branch
                // unless the value itself has no side effects
                if !replacement_has_side_effect
                    && let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        &mut logical_expr.right,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    )
                {
                    return Some(changed);
                }
            }
            Expression::PrivateInExpression(private_in_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut private_in_expr.right,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
            }
            Expression::ConditionalExpression(cond_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut cond_expr.test,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
                // Do not substitute our unconditionally-executed value into a branch
                // unless the value itself has no side effects
                if !replacement_has_side_effect {
                    // Unlike other branches in this function such as "a && b" or "a?.[b]",
                    // the "a ? b : c" form has potential code evaluation along both control
                    // flow paths. Handle this by allowing substitution into either branch.
                    // Side effects in one branch should not prevent the substitution into
                    // the other branch.

                    let consequent_changed = Self::substitute_single_use_symbol_in_expression(
                        &mut cond_expr.consequent,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    );
                    if consequent_changed == Some(true) {
                        return consequent_changed;
                    }
                    let alternate_changed = Self::substitute_single_use_symbol_in_expression(
                        &mut cond_expr.alternate,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    );
                    if alternate_changed == Some(true) {
                        return alternate_changed;
                    }
                    // Side effects in either branch should stop us from continuing to try to
                    // substitute the replacement after the control flow branches merge again.
                    if consequent_changed == Some(false) || alternate_changed == Some(false) {
                        return Some(false);
                    }
                }
            }
            Expression::ComputedMemberExpression(member_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut member_expr.object,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
                // Do not substitute our unconditionally-executed value into a branch
                // unless the value itself has no side effects
                if (!replacement_has_side_effect || !member_expr.optional)
                    && let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        &mut member_expr.expression,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    )
                {
                    return Some(changed);
                }
            }
            Expression::PrivateFieldExpression(private_field_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut private_field_expr.object,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
            }
            Expression::CallExpression(call_expr)
                // Don't substitute something into a call target that could change "this"
                if !((replacement.is_member_expression()
                    || matches!(replacement, Expression::ChainExpression(_)))
                    && call_expr.callee.is_identifier_reference())
                => {
                    if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        &mut call_expr.callee,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    ) {
                        return Some(changed);
                    }

                    // Do not substitute our unconditionally-executed value into a branch
                    // unless the value itself has no side effects
                    if !replacement_has_side_effect || !call_expr.optional {
                        for arg in &mut call_expr.arguments {
                            match arg {
                                Argument::SpreadElement(spread_elem) => {
                                    if let Some(changed) =
                                        Self::substitute_single_use_symbol_in_expression(
                                            &mut spread_elem.argument,
                                            search_for,
                                            replacement,
                                            replacement_has_side_effect,
                                            ctx,
                                        )
                                    {
                                        return Some(changed);
                                    }
                                    // spread element may have sideeffects
                                    return Some(false);
                                }
                                match_expression!(Argument) => {
                                    if let Some(changed) =
                                        Self::substitute_single_use_symbol_in_expression(
                                            arg.to_expression_mut(),
                                            search_for,
                                            replacement,
                                            replacement_has_side_effect,
                                            ctx,
                                        )
                                    {
                                        return Some(changed);
                                    }
                                }
                            }
                        }
                    }
                }
            Expression::NewExpression(new_expr)
                // Don't substitute something into a call target that could change "this"
                if !((replacement.is_member_expression()
                    || matches!(replacement, Expression::ChainExpression(_)))
                    && new_expr.callee.is_identifier_reference())
                => {
                    if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        &mut new_expr.callee,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    ) {
                        return Some(changed);
                    }

                    for arg in &mut new_expr.arguments {
                        match arg {
                            Argument::SpreadElement(spread_elem) => {
                                if let Some(changed) =
                                    Self::substitute_single_use_symbol_in_expression(
                                        &mut spread_elem.argument,
                                        search_for,
                                        replacement,
                                        replacement_has_side_effect,
                                        ctx,
                                    )
                                {
                                    return Some(changed);
                                }
                                // spread element may have sideeffects
                                return Some(false);
                            }
                            match_expression!(Argument) => {
                                if let Some(changed) =
                                    Self::substitute_single_use_symbol_in_expression(
                                        arg.to_expression_mut(),
                                        search_for,
                                        replacement,
                                        replacement_has_side_effect,
                                        ctx,
                                    )
                                {
                                    return Some(changed);
                                }
                            }
                        }
                    }
                }
            Expression::ArrayExpression(array_expr) => {
                for elem in &mut array_expr.elements {
                    match elem {
                        ArrayExpressionElement::SpreadElement(spread_elem) => {
                            if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                                &mut spread_elem.argument,
                                search_for,
                                replacement,
                                replacement_has_side_effect,
                                ctx,
                            ) {
                                return Some(changed);
                            }
                            // spread element may have sideeffects
                            return Some(false);
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        match_expression!(ArrayExpressionElement) => {
                            if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                                elem.to_expression_mut(),
                                search_for,
                                replacement,
                                replacement_has_side_effect,
                                ctx,
                            ) {
                                return Some(changed);
                            }
                        }
                    }
                }
            }
            Expression::ObjectExpression(obj_expr) => {
                for prop in &mut obj_expr.properties {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(prop) => {
                            if prop.computed {
                                if let Some(changed) =
                                    Self::substitute_single_use_symbol_in_expression(
                                        prop.key.to_expression_mut(),
                                        search_for,
                                        replacement,
                                        replacement_has_side_effect,
                                        ctx,
                                    )
                                {
                                    return Some(changed);
                                }
                                // Stop now because computed keys have side effects
                                return Some(false);
                            }

                            if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                                &mut prop.value,
                                search_for,
                                replacement,
                                replacement_has_side_effect,
                                ctx,
                            ) {
                                if prop.shorthand && prop.key.is_specific_id("__proto__") {
                                    // { __proto__ } -> { ['__proto__']: value }
                                    prop.computed = true;
                                    prop.key = PropertyKey::new_string_literal(prop.key.span(), "__proto__", None, ctx);
                                }
                                prop.shorthand = false;
                                return Some(changed);
                            }
                        }
                        ObjectPropertyKind::SpreadProperty(prop) => {
                            if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                                &mut prop.argument,
                                search_for,
                                replacement,
                                replacement_has_side_effect,
                                ctx,
                            ) {
                                return Some(changed);
                            }
                            // Stop now because spread properties have side effects
                            return Some(false);
                        }
                    }
                }
            }
            Expression::TaggedTemplateExpression(tagged_expr) => {
                if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                    &mut tagged_expr.tag,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                ) {
                    return Some(changed);
                }
                for elem in &mut tagged_expr.quasi.expressions {
                    if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        elem,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    ) {
                        return Some(changed);
                    }
                }
            }
            Expression::TemplateLiteral(template_literal) => {
                for elem in &mut template_literal.expressions {
                    if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        elem,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    ) {
                        return Some(changed);
                    }
                }
            }
            Expression::ChainExpression(chain_expr) => {
                let mut expr = Expression::from(chain_expr.expression.take_in(ctx));
                let changed = Self::substitute_single_use_symbol_in_expression(
                    &mut expr,
                    search_for,
                    replacement,
                    replacement_has_side_effect,
                    ctx,
                );
                chain_expr.expression = expr.into_chain_element().expect("should be chain element");
                return changed;
            }
            Expression::SequenceExpression(sequence_expr) => {
                for expr in &mut sequence_expr.expressions {
                    if let Some(changed) = Self::substitute_single_use_symbol_in_expression(
                        expr,
                        search_for,
                        replacement,
                        replacement_has_side_effect,
                        ctx,
                    ) {
                        return Some(changed);
                    }
                }
            }
            _ => {}
        }

        // If both the replacement and this expression have no observable side
        // effects, then we can reorder the replacement past this expression
        //
        // ```js
        // // Substitution is ok
        // let replacement = 123;
        // return x + replacement;
        //
        // // Substitution is not ok because "fn()" may change "x"
        // let replacement = fn();
        // return x + replacement;
        //
        // // Substitution is not ok because "x == x" may change "x" due to "valueOf()" evaluation
        // let replacement = [x];
        // return (x == x) + replacement;
        // ```
        if !replacement_has_side_effect && !target_expr.may_have_side_effects(ctx) {
            return None;
        }

        // We can always reorder past literal values
        if replacement.is_literal_value(true, ctx) || target_expr.is_literal_value(true, ctx) {
            return None;
        }

        // Otherwise we should stop trying to substitute past this point
        Some(false)
    }
}
