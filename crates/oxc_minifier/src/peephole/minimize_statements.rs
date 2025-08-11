use std::{iter, ops::ControlFlow};

use oxc_allocator::{Box, TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_semantic::ScopeId;
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
    pub fn minimize_statements(&self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut Ctx<'a, '_>) {
        let mut result: Vec<'a, Statement<'a>> = ctx.ast.vec_with_capacity(stmts.len());
        let mut is_control_flow_dead = false;
        let mut keep_var = KeepVar::new(ctx.ast);
        let mut new_stmts = stmts.take_in(ctx.ast);
        for i in 0..new_stmts.len() {
            let stmt = new_stmts[i].take_in(ctx.ast);
            if is_control_flow_dead
                && !stmt.is_module_declaration()
                && !matches!(stmt.as_declaration(), Some(Declaration::FunctionDeclaration(_)))
            {
                keep_var.visit_statement(&stmt);
                continue;
            }
            if self
                .minimize_statement(
                    stmt,
                    i,
                    &mut new_stmts,
                    &mut result,
                    &mut is_control_flow_dead,
                    ctx,
                )
                .is_break()
            {
                break;
            }
        }
        if let Some(stmt) = keep_var.get_variable_declaration_statement() {
            result.push(stmt);
        }

        // Drop a trailing unconditional jump statement if applicable
        if let Some(last_stmt) = result.last() {
            match last_stmt {
                // "while (x) { y(); continue; }" => "while (x) { y(); }"
                Statement::ContinueStatement(s) if s.label.is_none() => {
                    let mut changed = false;
                    if let Some(Ancestor::ForStatementBody(_)) = ctx.ancestors().nth(1) {
                        result.pop();
                        changed = true;
                    }
                    if changed {
                        ctx.state.changed = true;
                    }
                }
                // "function f() { x(); return; }" => "function f() { x(); }"
                Statement::ReturnStatement(s) if s.argument.is_none() => {
                    if let Ancestor::FunctionBodyStatements(_) = ctx.parent() {
                        result.pop();
                        ctx.state.changed = true;
                    }
                }
                _ => {}
            }
        }

        // Merge certain statements in reverse order
        if result.len() >= 2 && ctx.options().sequences {
            if let Some(Statement::ReturnStatement(_)) = result.last() {
                'return_loop: while result.len() >= 2 {
                    let prev_index = result.len() - 2;
                    let prev_stmt = &result[prev_index];
                    match prev_stmt {
                        Statement::ExpressionStatement(_) => {
                            if let Some(Statement::ReturnStatement(last_return)) = result.last() {
                                if last_return.argument.is_none() {
                                    break 'return_loop;
                                }
                            }
                            ctx.state.changed = true;
                            // "a(); return b;" => "return a(), b;"
                            let last_stmt = result.pop().unwrap();
                            let Statement::ReturnStatement(mut last_return) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = result.pop().unwrap();
                            let Statement::ExpressionStatement(mut expr_stmt) = prev_stmt else {
                                unreachable!()
                            };
                            let b = last_return.argument.as_mut().unwrap();
                            let argument = Self::join_sequence(&mut expr_stmt.expression, b, ctx);
                            let right_span = last_return.span;
                            let last_return_stmt =
                                ctx.ast.statement_return(right_span, Some(argument));
                            result.push(last_return_stmt);
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
                            let last_stmt = result.pop().unwrap();
                            let Statement::ReturnStatement(last_return) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = result.pop().unwrap();
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
                            if let Expression::UnaryExpression(unary_expr) = &mut prev_if.test {
                                if unary_expr.operator.is_not() {
                                    prev_if.test = unary_expr.argument.take_in(ctx.ast);
                                    std::mem::swap(&mut left, &mut right);
                                }
                            }

                            let argument = if let Expression::SequenceExpression(sequence_expr) =
                                &mut prev_if.test
                            {
                                // "if (a, b) return c; return d;" => "return a, b ? c : d;"
                                let test = sequence_expr.expressions.pop().unwrap();
                                let mut b =
                                    self.minimize_conditional(prev_if.span, test, left, right, ctx);
                                Self::join_sequence(&mut prev_if.test, &mut b, ctx)
                            } else {
                                // "if (a) return b; return c;" => "return a ? b : c;"
                                self.minimize_conditional(
                                    prev_if.span,
                                    prev_if.test.take_in(ctx.ast),
                                    left,
                                    right,
                                    ctx,
                                )
                            };
                            let last_return_stmt =
                                ctx.ast.statement_return(right_span, Some(argument));
                            result.push(last_return_stmt);
                        }
                        _ => break 'return_loop,
                    }
                }
            } else if let Some(Statement::ThrowStatement(_)) = result.last() {
                'throw_loop: while result.len() >= 2 {
                    let prev_index = result.len() - 2;
                    let prev_stmt = &result[prev_index];
                    match prev_stmt {
                        Statement::ExpressionStatement(_) => {
                            ctx.state.changed = true;
                            // "a(); throw b;" => "throw a(), b;"
                            let last_stmt = result.pop().unwrap();
                            let Statement::ThrowStatement(mut last_throw) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = result.pop().unwrap();
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
                            result.push(last_throw_stmt);
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
                            let last_stmt = result.pop().unwrap();
                            let Statement::ThrowStatement(last_throw) = last_stmt else {
                                unreachable!()
                            };
                            let prev_stmt = result.pop().unwrap();
                            let Statement::IfStatement(prev_if) = prev_stmt else { unreachable!() };
                            let mut prev_if = prev_if.unbox();
                            let Statement::ThrowStatement(prev_throw) = prev_if.consequent else {
                                unreachable!()
                            };

                            let right_span = last_throw.span;
                            let mut left = prev_throw.unbox().argument;
                            let mut right = last_throw.unbox().argument;

                            // "if (!a) throw b; throw c;" => "throw a ? c : b;"
                            if let Expression::UnaryExpression(unary_expr) = &mut prev_if.test {
                                if unary_expr.operator.is_not() {
                                    prev_if.test = unary_expr.argument.take_in(ctx.ast);
                                    std::mem::swap(&mut left, &mut right);
                                }
                            }

                            let argument = if let Expression::SequenceExpression(sequence_expr) =
                                &mut prev_if.test
                            {
                                // "if (a, b) throw c; throw d;" => "throw a, b ? c : d;"
                                let test = sequence_expr.expressions.pop().unwrap();
                                let mut b =
                                    self.minimize_conditional(prev_if.span, test, left, right, ctx);
                                Self::join_sequence(&mut prev_if.test, &mut b, ctx)
                            } else {
                                // "if (a) throw b; throw c;" => "throw a ? b : c;"
                                self.minimize_conditional(
                                    prev_if.span,
                                    prev_if.test.take_in(ctx.ast),
                                    left,
                                    right,
                                    ctx,
                                )
                            };
                            let last_throw_stmt = ctx.ast.statement_throw(right_span, argument);
                            result.push(last_throw_stmt);
                        }
                        _ => break 'throw_loop,
                    }
                }
            }
        }

        *stmts = result;
    }

    fn minimize_statement(
        &self,
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
                self.handle_variable_declaration(var_decl, result, ctx);
            }
            Statement::ExpressionStatement(expr_stmt) => {
                self.handle_expression_statement(expr_stmt, result, ctx);
            }
            Statement::SwitchStatement(switch_stmt) => {
                self.handle_switch_statement(switch_stmt, result, ctx);
            }
            Statement::IfStatement(if_stmt) => {
                if self.handle_if_statement(i, stmts, if_stmt, result, ctx).is_break() {
                    return ControlFlow::Break(());
                }
            }
            Statement::ReturnStatement(ret_stmt) => {
                self.handle_return_statement(ret_stmt, result, is_control_flow_dead, ctx);
            }
            Statement::ThrowStatement(throw_stmt) => {
                self.handle_throw_statement(throw_stmt, result, is_control_flow_dead, ctx);
            }
            Statement::ForStatement(for_stmt) => {
                self.handle_for_statement(for_stmt, result, ctx);
            }
            Statement::ForInStatement(for_in_stmt) => {
                self.handle_for_in_statement(for_in_stmt, result, ctx);
            }
            Statement::ForOfStatement(for_of_stmt) => {
                self.handle_for_of_statement(for_of_stmt, result, ctx);
            }
            Statement::BlockStatement(block_stmt) => self.handle_block(result, block_stmt, ctx),
            stmt => result.push(stmt),
        }
        ControlFlow::Continue(())
    }

    fn join_sequence(
        a: &mut Expression<'a>,
        b: &mut Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
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
        &self,
        var_decl: Box<'a, VariableDeclaration<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        // If `join_vars` is off, but there are unused declarators ... just join them to make our code simpler.
        if !ctx.options().join_vars
            && var_decl.declarations.iter().all(|d| !Self::should_remove_unused_declarator(d, ctx))
        {
            result.push(Statement::VariableDeclaration(var_decl));
            return;
        }

        if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last() {
            if var_decl.kind == prev_var_decl.kind {
                ctx.state.changed = true;
            }
        }
        let VariableDeclaration { span, kind, declarations, declare } = var_decl.unbox();
        for mut decl in declarations {
            if Self::should_remove_unused_declarator(&decl, ctx) {
                ctx.state.changed = true;
                if let Some(init) = decl.init.take() {
                    if init.may_have_side_effects(ctx) {
                        result.push(ctx.ast.statement_expression(init.span(), init));
                    }
                }
            } else {
                if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last_mut() {
                    if kind == prev_var_decl.kind {
                        prev_var_decl.declarations.push(decl);
                        continue;
                    }
                }
                let decls = ctx.ast.vec1(decl);
                let new_decl = ctx.ast.alloc_variable_declaration(span, kind, decls, declare);
                result.push(Statement::VariableDeclaration(new_decl));
            }
        }
    }

    fn handle_expression_statement(
        &self,
        mut expr_stmt: Box<'a, ExpressionStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if ctx.options().sequences {
            if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                let a = &mut prev_expr_stmt.expression;
                let b = &mut expr_stmt.expression;
                expr_stmt.expression = Self::join_sequence(a, b, ctx);
                result.pop();
                ctx.state.changed = true;
            }
        }
        result.push(Statement::ExpressionStatement(expr_stmt));
    }

    fn handle_switch_statement(
        &self,
        mut switch_stmt: Box<'a, SwitchStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if ctx.options().sequences {
            if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                let a = &mut prev_expr_stmt.expression;
                let b = &mut switch_stmt.discriminant;
                switch_stmt.discriminant = Self::join_sequence(a, b, ctx);
                result.pop();
                ctx.state.changed = true;
            }
        }
        result.push(Statement::SwitchStatement(switch_stmt));
    }

    #[expect(clippy::cast_possible_truncation)]
    fn handle_if_statement(
        &self,
        i: usize,
        stmts: &mut Vec<'a, Statement<'a>>,
        mut if_stmt: Box<'a, IfStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) -> ControlFlow<()> {
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
                if let Some(Statement::IfStatement(prev_if_stmt)) = result.last_mut() {
                    if prev_if_stmt.alternate.is_none()
                        && Self::jump_stmts_look_the_same(
                            &prev_if_stmt.consequent,
                            &if_stmt.consequent,
                        )
                    {
                        // "if (a) break c; if (b) break c;" => "if (a || b) break c;"
                        // "if (a) continue c; if (b) continue c;" => "if (a || b) continue c;"
                        // "if (a) return c; if (b) return c;" => "if (a || b) return c;"
                        // "if (a) throw c; if (b) throw c;" => "if (a || b) throw c;"
                        if_stmt.test = self.join_with_left_associative_op(
                            if_stmt.test.span(),
                            LogicalOperator::Or,
                            prev_if_stmt.test.take_in(ctx.ast),
                            if_stmt.test.take_in(ctx.ast),
                            ctx,
                        );
                        result.pop();
                        ctx.state.changed = true;
                    }
                }

                let mut optimize_implicit_jump = false;
                // "while (x) { if (y) continue; z(); }" => "while (x) { if (!y) z(); }"
                // "while (x) { if (y) continue; else z(); w(); }" => "while (x) { if (!y) { z(); w(); } }" => "for (; x;) !y && (z(), w());"
                if ctx.ancestors().nth(1).is_some_and(Ancestor::is_for_statement) {
                    if let Statement::ContinueStatement(continue_stmt) = &if_stmt.consequent {
                        if continue_stmt.label.is_none() {
                            optimize_implicit_jump = true;
                        }
                    }
                }

                // "let x = () => { if (y) return; z(); };" => "let x = () => { if (!y) z(); };"
                // "let x = () => { if (y) return; else z(); w(); };" => "let x = () => { if (!y) { z(); w(); } };" => "let x = () => { !y && (z(), w()); };"
                if ctx.parent().is_function_body() {
                    if let Statement::ReturnStatement(return_stmt) = &if_stmt.consequent {
                        if return_stmt.argument.is_none() {
                            optimize_implicit_jump = true;
                        }
                    }
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
                    if let Some(alternate) = &if_stmt.alternate {
                        if Self::statement_cares_about_scope(alternate) {
                            can_move_branch_condition_outside_scope = false;
                        }
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

                        self.minimize_statements(&mut body, ctx);
                        let span = if body.is_empty() {
                            if_stmt.consequent.span()
                        } else {
                            body[0].span()
                        };
                        let test = if_stmt.test.take_in(ctx.ast);
                        let mut test = self.minimize_not(test.span(), test, ctx);
                        self.try_fold_expr_in_boolean_context(&mut test, ctx);
                        let consequent = if body.len() == 1 {
                            body.remove(0)
                        } else {
                            let scope_id = ScopeId::new(ctx.scoping().scopes_len() as u32);
                            let block_stmt =
                                ctx.ast.block_statement_with_scope_id(span, body, scope_id);
                            Statement::BlockStatement(ctx.ast.alloc(block_stmt))
                        };
                        let mut if_stmt = ctx.ast.if_statement(test.span(), test, consequent, None);
                        let if_stmt = self
                            .try_minimize_if(&mut if_stmt, ctx)
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
                        if let Some(Statement::IfStatement(if_stmt)) = result.last_mut() {
                            if if_stmt.consequent.is_jump_statement() {
                                if let Some(stmt) = if_stmt.alternate.take() {
                                    if let Statement::BlockStatement(block_stmt) = stmt {
                                        self.handle_block(result, block_stmt, ctx);
                                    } else {
                                        result.push(stmt);
                                        ctx.state.changed = true;
                                    }
                                    continue;
                                }
                            }
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
        &self,
        mut ret_stmt: Box<'a, ReturnStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        is_control_flow_dead: &mut bool,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if ctx.options().sequences {
            if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                if let Some(argument) = &mut ret_stmt.argument {
                    let a = &mut prev_expr_stmt.expression;
                    *argument = Self::join_sequence(a, argument, ctx);
                    result.pop();
                    ctx.state.changed = true;
                }
            }
        }
        result.push(Statement::ReturnStatement(ret_stmt));
        *is_control_flow_dead = true;
    }

    fn handle_throw_statement(
        &self,
        mut throw_stmt: Box<'a, ThrowStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        is_control_flow_dead: &mut bool,

        ctx: &mut Ctx<'a, '_>,
    ) {
        if ctx.options().sequences {
            if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                let a = &mut prev_expr_stmt.expression;
                let b = &mut throw_stmt.argument;
                throw_stmt.argument = Self::join_sequence(a, b, ctx);
                result.pop();
                ctx.state.changed = true;
            }
        }
        result.push(Statement::ThrowStatement(throw_stmt));
        *is_control_flow_dead = true;
    }

    fn handle_for_statement(
        &self,
        mut for_stmt: Box<'a, ForStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
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
                        if prev_var_decl.kind.is_var() {
                            if let ForStatementInit::VariableDeclaration(var_decl) = init {
                                if var_decl.kind.is_var() {
                                    var_decl
                                        .declarations
                                        .splice(0..0, prev_var_decl.declarations.drain(..));
                                    result.pop();
                                    ctx.state.changed = true;
                                }
                            }
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
        &self,
        mut for_in_stmt: Box<'a, ForInStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,

        ctx: &mut Ctx<'a, '_>,
    ) {
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
                        if let Some(prev_var_decl_item) = prev_var_decl_no_init_item {
                            if let BindingPatternKind::BindingIdentifier(decl_id) =
                                &prev_var_decl_item.id.kind
                            {
                                if id.name == decl_id.name {
                                    let Some(Statement::VariableDeclaration(prev_var_decl)) =
                                        result.pop()
                                    else {
                                        unreachable!()
                                    };
                                    for_in_stmt.left =
                                        ForStatementLeft::VariableDeclaration(prev_var_decl);
                                    ctx.state.changed = true;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        result.push(Statement::ForInStatement(for_in_stmt));
    }

    fn handle_for_of_statement(
        &self,
        mut for_of_stmt: Box<'a, ForOfStatement<'a>>,
        result: &mut Vec<'a, Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        // "var a; for (a of b) c" => "for (var a of b) c"
        if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last_mut() {
            if let ForStatementLeft::AssignmentTargetIdentifier(id) = &for_of_stmt.left {
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
                if let Some(prev_var_decl_item) = prev_var_decl_no_init_item {
                    if let BindingPatternKind::BindingIdentifier(decl_id) =
                        &prev_var_decl_item.id.kind
                    {
                        if id.name == decl_id.name {
                            let Some(Statement::VariableDeclaration(prev_var_decl)) = result.pop()
                            else {
                                unreachable!()
                            };
                            for_of_stmt.left = ForStatementLeft::VariableDeclaration(prev_var_decl);
                            ctx.state.changed = true;
                        }
                    }
                }
            }
        }
        result.push(Statement::ForOfStatement(for_of_stmt));
    }

    /// `appendIfOrLabelBodyPreservingScope`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9852>
    fn handle_block(
        &self,
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
}
