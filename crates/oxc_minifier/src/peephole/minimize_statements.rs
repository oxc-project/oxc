use std::{iter, ops::ControlFlow};

use oxc_allocator::{Box, TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, IsLiteralValue, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_semantic::ScopeFlags;
use oxc_span::{ContentEq, GetSpan};
use oxc_traverse::Ancestor;

use crate::{ctx::Ctx, keep_var::KeepVar};

use super::PeepholeOptimizations;

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
    pub fn minimize_statements(stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut Ctx<'a, '_>) {
        let mut old_stmts = stmts.take_in(ctx.ast);
        let mut is_control_flow_dead = false;
        let mut keep_var = KeepVar::new(ctx.ast);
        for i in 0..old_stmts.len() {
            let stmt = old_stmts[i].take_in(ctx.ast);
            if is_control_flow_dead
                && !stmt.is_module_declaration()
                && !matches!(stmt.as_declaration(), Some(Declaration::FunctionDeclaration(_)))
            {
                keep_var.visit_statement(&stmt);
                continue;
            }
            if Self::minimize_statement(
                stmt,
                i,
                &mut old_stmts,
                stmts,
                &mut is_control_flow_dead,
                ctx,
            )
            .is_break()
            {
                break;
            }
        }
        if let Some(stmt) = keep_var.get_variable_declaration_statement()
            && let Some(stmt) = Self::remove_unused_variable_declaration(stmt, ctx)
        {
            stmts.push(stmt);
        }

        // Drop a trailing unconditional jump statement if applicable
        if let Some(last_stmt) = stmts.last() {
            match last_stmt {
                // "while (x) { y(); continue; }" => "while (x) { y(); }"
                Statement::ContinueStatement(s) if s.label.is_none() => {
                    let mut changed = false;
                    if let Some(
                        Ancestor::ForStatementBody(_)
                        | Ancestor::ForInStatementBody(_)
                        | Ancestor::ForOfStatementBody(_),
                    ) = ctx.ancestors().nth(1)
                    {
                        stmts.pop();
                        changed = true;
                    }
                    if changed {
                        ctx.state.changed = true;
                    }
                }
                // "function f() { x(); return; }" => "function f() { x(); }"
                Statement::ReturnStatement(s) if s.argument.is_none() => {
                    if let Ancestor::FunctionBodyStatements(_) = ctx.parent() {
                        stmts.pop();
                        ctx.state.changed = true;
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
                            ctx.state.changed = true;
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
                                ctx.ast.statement_return(right_span, Some(argument));
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

                            ctx.state.changed = true;
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
                                .unwrap_or_else(|| ctx.ast.void_0(left_span));
                            // "if (a) return a; return;" => "return a ? b : void 0;"
                            let mut right = last_return
                                .unbox()
                                .argument
                                .unwrap_or_else(|| ctx.ast.void_0(right_span));

                            // "if (!a) return b; return c;" => "return a ? c : b;"
                            if let Expression::UnaryExpression(unary_expr) = &mut prev_if.test
                                && unary_expr.operator.is_not()
                            {
                                prev_if.test = unary_expr.argument.take_in(ctx.ast);
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
                                    prev_if.test.take_in(ctx.ast),
                                    left,
                                    right,
                                    ctx,
                                )
                            };
                            let last_return_stmt =
                                ctx.ast.statement_return(right_span, Some(argument));
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
                            ctx.state.changed = true;
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
                            let last_throw_stmt = ctx.ast.statement_throw(right_span, argument);
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

                            ctx.state.changed = true;
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
                                prev_if.test = unary_expr.argument.take_in(ctx.ast);
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
                                    prev_if.test.take_in(ctx.ast),
                                    left,
                                    right,
                                    ctx,
                                )
                            };
                            let last_throw_stmt = ctx.ast.statement_throw(right_span, argument);
                            stmts.push(last_throw_stmt);
                        }
                        _ => break 'throw_loop,
                    }
                }
            }
        }
    }

    fn minimize_statement(
        stmt: Statement<'a>,
        i: usize,
        stmts: &mut Vec<'a, Statement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        is_control_flow_dead: &mut bool,

        ctx: &mut Ctx<'a, '_>,
    ) -> ControlFlow<()> {
        match stmt {
            Statement::EmptyStatement(_) => (),
            Statement::BreakStatement(s) => {
                *is_control_flow_dead = true;
                result.push(Statement::BreakStatement(s));
            }
            Statement::ContinueStatement(s) => {
                *is_control_flow_dead = true;
                result.push(Statement::ContinueStatement(s));
            }
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
                Self::handle_return_statement(ret_stmt, result, is_control_flow_dead, ctx);
            }
            Statement::ThrowStatement(throw_stmt) => {
                Self::handle_throw_statement(throw_stmt, result, is_control_flow_dead, ctx);
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
        ctx: &Ctx<'a, '_>,
    ) -> Expression<'a> {
        let a = a.take_in(ctx.ast);
        let b = b.take_in(ctx.ast);
        if let Expression::SequenceExpression(mut sequence_expr) = a {
            // `(a, b); c`
            sequence_expr.expressions.push(b);
            return Expression::SequenceExpression(sequence_expr);
        }
        let span = a.span();
        let exprs = if let Expression::SequenceExpression(sequence_expr) = b {
            // `a; (b, c)`
            ctx.ast.vec_from_iter(std::iter::once(a).chain(sequence_expr.unbox().expressions))
        } else {
            // `a; b`
            ctx.ast.vec_from_array([a, b])
        };
        ctx.ast.expression_sequence(span, exprs)
    }

    fn jump_stmts_look_the_same(left: &Statement<'a>, right: &Statement<'a>) -> bool {
        if left.is_jump_statement() && right.is_jump_statement() {
            return left.content_eq(right);
        }
        false
    }

    /// For variable declarations:
    /// * merge with the previous variable declarator if their kinds are the same
    /// * remove the variable declarator if it is unused
    /// * keep the initializer if it has side effects
    fn handle_variable_declaration(
        mut var_decl: Box<'a, VariableDeclaration<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if let Some(first_decl) = var_decl.declarations.first_mut()
            && let Some(first_decl_init) = first_decl.init.as_mut()
        {
            let changed = Self::substitute_single_use_symbol_in_statement(
                first_decl_init,
                result,
                ctx,
                false,
            );
            if changed {
                ctx.state.changed = true;
            }
        }
        if Self::substitute_single_use_symbol_within_declaration(
            var_decl.kind,
            &mut var_decl.declarations,
            ctx,
        ) {
            ctx.state.changed = true;
        }

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
            ctx.state.changed = true;
        }
        let VariableDeclaration { span, kind, declarations, declare, .. } = var_decl.unbox();
        for mut decl in declarations {
            if Self::should_remove_unused_declarator(&decl, ctx) {
                ctx.state.changed = true;
                if let Some(init) = decl.init.take()
                    && init.may_have_side_effects(ctx)
                {
                    result.push(ctx.ast.statement_expression(init.span(), init));
                }
            } else {
                if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last_mut()
                    && kind == prev_var_decl.kind
                {
                    prev_var_decl.declarations.push(decl);
                    continue;
                }
                let decls = ctx.ast.vec1(decl);
                let new_decl = ctx.ast.alloc_variable_declaration(span, kind, decls, declare);
                result.push(Statement::VariableDeclaration(new_decl));
            }
        }
    }

    fn handle_expression_statement(
        mut expr_stmt: Box<'a, ExpressionStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        let changed = Self::substitute_single_use_symbol_in_statement(
            &mut expr_stmt.expression,
            result,
            ctx,
            false,
        );
        if changed {
            ctx.state.changed = true;
        }

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
        {
            let a = &mut prev_expr_stmt.expression;
            let b = &mut expr_stmt.expression;
            expr_stmt.expression = Self::join_sequence(a, b, ctx);
            result.pop();
            ctx.state.changed = true;
        }
        // "var a; a = b();" => "var a = b();"
        match &mut expr_stmt.expression {
            Expression::AssignmentExpression(assign_expr) => {
                let merged = Self::merge_assignment_to_declaration(assign_expr, result, ctx);
                if merged {
                    ctx.state.changed = true;
                    return;
                }
            }
            Expression::SequenceExpression(sequence_expr) => {
                if result
                    .last()
                    .is_some_and(|stmt| matches!(stmt, Statement::VariableDeclaration(_)))
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
                            ctx.state.changed = true;
                            return;
                        }
                        Some(val) if val == sequence_len - 1 => {
                            // all elements are merged except for the last expression
                            let last_expr = sequence_expr.expressions.pop().unwrap();
                            result.push(ctx.ast.statement_expression(last_expr.span(), last_expr));
                            ctx.state.changed = true;
                            return;
                        }
                        Some(0) => {
                            // no elements are merged
                        }
                        Some(val) => {
                            sequence_expr.expressions.drain(0..val);
                            ctx.state.changed = true;
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
        result: &mut Vec<'a, Statement<'a>>,
        ctx: &Ctx<'a, '_>,
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
                    decl.init = Some(assign_expr.right.take_in(ctx.ast));
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

    fn handle_switch_statement(
        mut switch_stmt: Box<'a, SwitchStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        let changed = Self::substitute_single_use_symbol_in_statement(
            &mut switch_stmt.discriminant,
            result,
            ctx,
            false,
        );
        if changed {
            ctx.state.changed = true;
        }

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
        {
            let a = &mut prev_expr_stmt.expression;
            let b = &mut switch_stmt.discriminant;
            switch_stmt.discriminant = Self::join_sequence(a, b, ctx);
            result.pop();
            ctx.state.changed = true;
        }

        // Prune empty case clauses before a trailing default
        // e.g., `switch (x) { case 0: foo(); break; case 1: default: bar() }`
        // => `switch (x) { case 0: foo(); break; default: bar() }`
        // https://github.com/evanw/esbuild/commit/add452ed51333953dd38a26f28a775bb220ea2e9
        if let Some(last_case) = switch_stmt.cases.last()
            && last_case.test.is_none()
        {
            let default_idx = switch_stmt.cases.len() - 1;
            let mut first_empty_idx = default_idx;
            // Iterate backward through preceding cases
            while first_empty_idx > 0 {
                let case = &switch_stmt.cases[first_empty_idx - 1];
                // Only remove empty cases with primitive literal tests
                if case.consequent.is_empty()
                    && case.test.as_ref().is_some_and(Expression::is_literal)
                {
                    first_empty_idx -= 1;
                } else {
                    break;
                }
            }
            // If we found cases to remove, keep cases [0..first_empty_idx] + [default]
            if first_empty_idx < default_idx {
                let default_case = switch_stmt.cases.pop().unwrap();
                switch_stmt.cases.truncate(first_empty_idx);
                switch_stmt.cases.push(default_case);
                ctx.state.changed = true;
            }
        }

        result.push(Statement::SwitchStatement(switch_stmt));
    }

    fn handle_if_statement(
        i: usize,
        stmts: &mut Vec<'a, Statement<'a>>,
        mut if_stmt: Box<'a, IfStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) -> ControlFlow<()> {
        let changed =
            Self::substitute_single_use_symbol_in_statement(&mut if_stmt.test, result, ctx, false);
        if changed {
            ctx.state.changed = true;
        }

        // Absorb a previous expression statement
        if ctx.options().sequences {
            if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                let a = &mut prev_expr_stmt.expression;
                let b = &mut if_stmt.test;
                if_stmt.test = Self::join_sequence(a, b, ctx);
                result.pop();
                ctx.state.changed = true;
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
                        prev_if_stmt.test.take_in(ctx.ast),
                        if_stmt.test.take_in(ctx.ast),
                        ctx,
                    );
                    result.pop();
                    ctx.state.changed = true;
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
                            ctx.ast.vec_from_iter(iter::once(alternate).chain(drained_stmts))
                        } else {
                            ctx.ast.vec_from_iter(drained_stmts)
                        };

                        Self::minimize_statements(&mut body, ctx);
                        let span = if body.is_empty() {
                            if_stmt.consequent.span()
                        } else {
                            body[0].span()
                        };
                        let test = if_stmt.test.take_in(ctx.ast);
                        let mut test = Self::minimize_not(test.span(), test, ctx);
                        Self::minimize_expression_in_boolean_context(&mut test, ctx);
                        let consequent = if body.len() == 1 {
                            body.remove(0)
                        } else {
                            let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
                            let block_stmt =
                                ctx.ast.block_statement_with_scope_id(span, body, scope_id);
                            Statement::BlockStatement(ctx.ast.alloc(block_stmt))
                        };
                        let mut if_stmt = ctx.ast.if_statement(test.span(), test, consequent, None);
                        let if_stmt = Self::try_minimize_if(&mut if_stmt, ctx)
                            .unwrap_or_else(|| Statement::IfStatement(ctx.ast.alloc(if_stmt)));
                        result.push(if_stmt);
                        ctx.state.changed = true;
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
                                ctx.state.changed = true;
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
        mut ret_stmt: Box<'a, ReturnStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        is_control_flow_dead: &mut bool,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if let Some(ret_argument_expr) = &mut ret_stmt.argument {
            let changed = Self::substitute_single_use_symbol_in_statement(
                ret_argument_expr,
                result,
                ctx,
                false,
            );
            if changed {
                ctx.state.changed = true;
            }
        }

        if let Some(argument) = &mut ret_stmt.argument
            && argument.value_type(ctx) == ValueType::Undefined
            // `return undefined` has a different semantic in async generator function.
            && !ctx.is_closest_function_scope_an_async_generator()
        {
            ctx.state.changed = true;
            if argument.may_have_side_effects(ctx) {
                if ctx.options().sequences
                    && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
                {
                    let a = &mut prev_expr_stmt.expression;
                    prev_expr_stmt.expression = Self::join_sequence(a, argument, ctx);
                } else {
                    result.push(
                        ctx.ast.statement_expression(argument.span(), argument.take_in(ctx.ast)),
                    );
                }
            }
            ret_stmt.argument = None;
            result.push(Statement::ReturnStatement(ret_stmt));
            *is_control_flow_dead = true;
            return;
        }

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
            && let Some(argument) = &mut ret_stmt.argument
        {
            let a = &mut prev_expr_stmt.expression;
            *argument = Self::join_sequence(a, argument, ctx);
            result.pop();
            ctx.state.changed = true;
        }
        result.push(Statement::ReturnStatement(ret_stmt));
        *is_control_flow_dead = true;
    }

    fn handle_throw_statement(
        mut throw_stmt: Box<'a, ThrowStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        is_control_flow_dead: &mut bool,

        ctx: &mut Ctx<'a, '_>,
    ) {
        let changed = Self::substitute_single_use_symbol_in_statement(
            &mut throw_stmt.argument,
            result,
            ctx,
            false,
        );
        if changed {
            ctx.state.changed = true;
        }

        if ctx.options().sequences
            && let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut()
        {
            let a = &mut prev_expr_stmt.expression;
            let b = &mut throw_stmt.argument;
            throw_stmt.argument = Self::join_sequence(a, b, ctx);
            result.pop();
            ctx.state.changed = true;
        }
        result.push(Statement::ThrowStatement(throw_stmt));
        *is_control_flow_dead = true;
    }

    fn handle_for_statement(
        mut for_stmt: Box<'a, ForStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if let Some(init) = &mut for_stmt.init {
            match init {
                ForStatementInit::VariableDeclaration(var_decl) => {
                    if let Some(first_decl) = var_decl.declarations.first_mut()
                        && let Some(first_decl_init) = first_decl.init.as_mut()
                    {
                        let is_block_scoped_decl = !first_decl.kind.is_var();
                        let changed = Self::substitute_single_use_symbol_in_statement(
                            first_decl_init,
                            result,
                            ctx,
                            is_block_scoped_decl,
                        );
                        if changed {
                            ctx.state.changed = true;
                        }
                    }
                    if Self::substitute_single_use_symbol_within_declaration(
                        var_decl.kind,
                        &mut var_decl.declarations,
                        ctx,
                    ) {
                        ctx.state.changed = true;
                    }
                }
                match_expression!(ForStatementInit) => {
                    let init = init.to_expression_mut();
                    let changed =
                        Self::substitute_single_use_symbol_in_statement(init, result, ctx, false);
                    if changed {
                        ctx.state.changed = true;
                    }
                }
            }
        }

        if let Some(ForStatementInit::VariableDeclaration(var_decl)) = &mut for_stmt.init {
            let old_len = var_decl.declarations.len();
            var_decl.declarations.retain(|decl| {
                !Self::should_remove_unused_declarator(decl, ctx)
                    || decl.init.as_ref().is_some_and(|init| init.may_have_side_effects(ctx))
            });
            if old_len != var_decl.declarations.len() {
                if var_decl.declarations.is_empty() {
                    for_stmt.init = None;
                }
                ctx.state.changed = true;
            }
        }

        if ctx.options().sequences {
            match result.last_mut() {
                Some(Statement::ExpressionStatement(prev_expr_stmt)) => {
                    if let Some(init) = &mut for_stmt.init {
                        if let Some(init) = init.as_expression_mut() {
                            let a = &mut prev_expr_stmt.expression;
                            *init = Self::join_sequence(a, init, ctx);
                            result.pop();
                            ctx.state.changed = true;
                        }
                    } else {
                        for_stmt.init = Some(ForStatementInit::from(
                            prev_expr_stmt.expression.take_in(ctx.ast),
                        ));
                        result.pop();
                        ctx.state.changed = true;
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
                            result.pop();
                            ctx.state.changed = true;
                        }
                    } else if prev_var_decl.kind.is_var() {
                        let Some(Statement::VariableDeclaration(prev_var_decl)) = result.pop()
                        else {
                            unreachable!()
                        };
                        for_stmt.init = Some(ForStatementInit::VariableDeclaration(prev_var_decl));
                        ctx.state.changed = true;
                    }
                }
                _ => {}
            }
        }
        result.push(Statement::ForStatement(for_stmt));
    }

    fn handle_for_in_statement(
        mut for_in_stmt: Box<'a, ForInStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        // Annex B.3.5 allows initializers in non-strict mode
        // <https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-initializers-in-forin-statement-heads>
        // That is evaluated before the right hand side is evaluated. So, in that case, skip the single use substitution.
        if !matches!(&for_in_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if var_decl.has_init())
        {
            let is_block_scoped_decl = matches!(&for_in_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if !var_decl.kind.is_var());
            let changed = Self::substitute_single_use_symbol_in_statement(
                &mut for_in_stmt.right,
                result,
                ctx,
                is_block_scoped_decl,
            );
            if changed {
                ctx.state.changed = true;
            }
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
                    if !has_side_effectful_initializer {
                        let a = &mut prev_expr_stmt.expression;
                        for_in_stmt.right = Self::join_sequence(a, &mut for_in_stmt.right, ctx);
                        result.pop();
                        ctx.state.changed = true;
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
                            for_in_stmt.left = ForStatementLeft::VariableDeclaration(prev_var_decl);
                            ctx.state.changed = true;
                        }
                    }
                }
                _ => {}
            }
        }
        result.push(Statement::ForInStatement(for_in_stmt));
    }

    fn handle_for_of_statement(
        mut for_of_stmt: Box<'a, ForOfStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        let is_block_scoped_decl = matches!(&for_of_stmt.left, ForStatementLeft::VariableDeclaration(var_decl) if !var_decl.kind.is_var());
        let changed = Self::substitute_single_use_symbol_in_statement(
            &mut for_of_stmt.right,
            result,
            ctx,
            is_block_scoped_decl,
        );
        if changed {
            ctx.state.changed = true;
        }

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
                for_of_stmt.left = ForStatementLeft::VariableDeclaration(prev_var_decl);
                ctx.state.changed = true;
            }
        }
        result.push(Statement::ForOfStatement(for_of_stmt));
    }

    /// `appendIfOrLabelBodyPreservingScope`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9852>
    fn handle_block(
        result: &mut Vec<'a, Statement<'a>>,
        block_stmt: Box<'a, BlockStatement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        let keep_block = block_stmt.body.iter().any(Self::statement_cares_about_scope);
        if keep_block {
            result.push(Statement::BlockStatement(block_stmt));
        } else {
            result.append(&mut block_stmt.unbox().body);
            ctx.state.changed = true;
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
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &Ctx<'a, '_>,
        non_scoped_literal_only: bool,
    ) -> bool {
        if Self::keep_top_level_var_in_script_mode(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
        {
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
            if new_len == 0 {
                inlined = true;
                stmts.pop();
            } else if old_len != new_len {
                inlined = true;
                prev_var_decl.declarations.truncate(new_len);
                break;
            } else {
                break;
            }
        }
        inlined
    }

    fn substitute_single_use_symbol_within_declaration(
        kind: VariableDeclarationKind,
        declarations: &mut Vec<'a, VariableDeclarator<'a>>,
        ctx: &Ctx<'a, '_>,
    ) -> bool {
        if Self::keep_top_level_var_in_script_mode(ctx)
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
                declarations.drain(i - drop_count..i);
                i -= drop_count;
            }
            i += 1;
        }
        changed
    }

    /// Returns new length
    fn substitute_single_use_symbol_in_expression_from_declarators(
        target_expr: &mut Expression<'a>,
        declarators: &mut [VariableDeclarator<'a>],
        ctx: &Ctx<'a, '_>,
        non_scoped_literal_only: bool,
    ) -> usize {
        let last_non_inlined_index = declarators.iter_mut().rposition(|prev_decl| {
            let Some(prev_decl_init) = &mut prev_decl.init else {
                return true;
            };
            let BindingPattern::BindingIdentifier(prev_decl_id) = &prev_decl.id else {
                return true;
            };
            if ctx.is_expression_whose_name_needs_to_be_kept(prev_decl_init) {
                return true;
            }
            let Some(symbol_value) =
                ctx.state.symbol_values.get_symbol_value(prev_decl_id.symbol_id())
            else {
                return true;
            };
            // we should check whether it's exported by `symbol_value.exported`
            // because the variable might be exported with `export { foo }` rather than `export var foo`
            if symbol_value.exported
                || symbol_value.read_references_count > 1
                || symbol_value.write_references_count > 0
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
        ctx: &Ctx<'a, '_>,
    ) -> Option<bool> {
        match target_expr {
            Expression::Identifier(id) => {
                if id.name == search_for {
                    *target_expr = replacement.take_in(ctx.ast);
                    return Some(true);
                }
                // If the identifier is not a getter and the identifier is read-only,
                // we know that the value is same even if we reordered the expression.
                if let Some(symbol_id) = ctx.scoping().get_reference(id.reference_id()).symbol_id()
                    && !ctx.scoping().symbol_is_mutated(symbol_id)
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
                            Self::is_expression_that_reference_may_change(&member_expr.object, ctx)
                        }
                        AssignmentTarget::PrivateFieldExpression(member_expr) => {
                            Self::is_expression_that_reference_may_change(&member_expr.object, ctx)
                        }
                        AssignmentTarget::StaticMemberExpression(member_expr) => {
                            Self::is_expression_that_reference_may_change(&member_expr.object, ctx)
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
            Expression::CallExpression(call_expr) => {
                // Don't substitute something into a call target that could change "this"
                if !((replacement.is_member_expression()
                    || matches!(replacement, Expression::ChainExpression(_)))
                    && call_expr.callee.is_identifier_reference())
                {
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
            }
            Expression::NewExpression(new_expr) => {
                // Don't substitute something into a call target that could change "this"
                if !((replacement.is_member_expression()
                    || matches!(replacement, Expression::ChainExpression(_)))
                    && new_expr.callee.is_identifier_reference())
                {
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
                        ObjectPropertyKind::ObjectProperty(prop) => match prop.key {
                            PropertyKey::StaticIdentifier(_)
                            | PropertyKey::PrivateIdentifier(_) => {
                                if let Some(changed) =
                                    Self::substitute_single_use_symbol_in_expression(
                                        &mut prop.value,
                                        search_for,
                                        replacement,
                                        replacement_has_side_effect,
                                        ctx,
                                    )
                                {
                                    if prop.shorthand && prop.key.is_specific_id("__proto__") {
                                        // { __proto__ } -> { ['__proto__']: value }
                                        prop.computed = true;
                                        prop.key =
                                            PropertyKey::from(ctx.ast.expression_string_literal(
                                                prop.key.span(),
                                                "__proto__",
                                                None,
                                            ));
                                    }
                                    prop.shorthand = false;
                                    return Some(changed);
                                }
                            }
                            match_expression!(PropertyKey) => {
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
                        },
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
                let mut expr = Expression::from(chain_expr.expression.take_in(ctx.ast));
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
