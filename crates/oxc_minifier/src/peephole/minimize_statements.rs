use oxc_allocator::Vec;
use oxc_ast::{ast::*, Visit};
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_span::{cmp::ContentEq, GetSpan};
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
    pub fn minimize_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: Ctx<'a, '_>) {
        let mut result: Vec<'a, Statement<'a>> = ctx.ast.vec_with_capacity(stmts.len());
        let mut is_control_flow_dead = false;
        let mut keep_var = KeepVar::new(ctx.ast);
        for stmt in ctx.ast.vec_from_iter(stmts.drain(..)) {
            if is_control_flow_dead
                && !stmt.is_module_declaration()
                && !matches!(stmt.as_declaration(), Some(Declaration::FunctionDeclaration(_)))
            {
                keep_var.visit_statement(&stmt);
                continue;
            }
            self.minimize_statement(stmt, &mut result, &mut is_control_flow_dead, ctx);
        }
        if let Some(stmt) = keep_var.get_variable_declaration_statement() {
            result.push(stmt);
        }

        // Drop a trailing unconditional jump statement if applicable
        if let Some(last_stmt) = result.last() {
            match last_stmt {
                // "while (x) { y(); continue; }" => "while (x) { y(); }"
                Statement::ContinueStatement(s) if s.label.is_none() => {
                    if let Some(Ancestor::ForStatementBody(_)) = ctx.ancestors().nth(1) {
                        result.pop();
                        self.mark_current_function_as_changed();
                    }
                }
                // "function f() { x(); return; }" => "function f() { x(); }"
                Statement::ReturnStatement(s) if s.argument.is_none() => {
                    if let Ancestor::FunctionBodyStatements(_) = ctx.parent() {
                        result.pop();
                        self.mark_current_function_as_changed();
                    }
                }
                _ => {}
            }
        }

        // Merge certain statements in reverse order
        if result.len() >= 2 {
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
                            self.mark_current_function_as_changed();
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

                            self.mark_current_function_as_changed();
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
                                    prev_if.test =
                                        ctx.ast.move_expression(&mut unary_expr.argument);
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
                                let mut expr = self.minimize_conditional(
                                    prev_if.span,
                                    ctx.ast.move_expression(&mut prev_if.test),
                                    left,
                                    right,
                                    ctx,
                                );
                                self.minimize_conditions_exit_expression(&mut expr, ctx);
                                expr
                            };
                            let last_return_stmt =
                                ctx.ast.statement_return(right_span, Some(argument));
                            result.push(last_return_stmt);
                        }
                        _ => break 'return_loop,
                    }
                }
            }
        }

        *stmts = result;
    }

    fn minimize_statement(
        &mut self,
        stmt: Statement<'a>,
        result: &mut Vec<'a, Statement<'a>>,
        is_control_flow_dead: &mut bool,
        ctx: Ctx<'a, '_>,
    ) {
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
            Statement::VariableDeclaration(mut var_decl) => {
                if let Some(Statement::VariableDeclaration(prev_var_decl)) = result.last_mut() {
                    if var_decl.kind == prev_var_decl.kind {
                        var_decl.declarations.splice(0..0, prev_var_decl.declarations.drain(..));
                        result.pop();
                        self.mark_current_function_as_changed();
                    }
                }
                result.push(Statement::VariableDeclaration(var_decl));
            }
            Statement::ExpressionStatement(mut expr_stmt) => {
                if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                    let a = &mut prev_expr_stmt.expression;
                    let b = &mut expr_stmt.expression;
                    expr_stmt.expression = Self::join_sequence(a, b, ctx);
                    result.pop();
                    self.mark_current_function_as_changed();
                }
                result.push(Statement::ExpressionStatement(expr_stmt));
            }
            Statement::SwitchStatement(mut switch_stmt) => {
                if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                    let a = &mut prev_expr_stmt.expression;
                    let b = &mut switch_stmt.discriminant;
                    switch_stmt.discriminant = Self::join_sequence(a, b, ctx);
                    result.pop();
                    self.mark_current_function_as_changed();
                }
                result.push(Statement::SwitchStatement(switch_stmt));
            }
            Statement::IfStatement(mut if_stmt) => {
                if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                    let a = &mut prev_expr_stmt.expression;
                    let b = &mut if_stmt.test;
                    if_stmt.test = Self::join_sequence(a, b, ctx);
                    result.pop();
                    self.mark_current_function_as_changed();
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
                            if_stmt.test = Self::join_with_left_associative_op(
                                if_stmt.test.span(),
                                LogicalOperator::Or,
                                ctx.ast.move_expression(&mut prev_if_stmt.test),
                                ctx.ast.move_expression(&mut if_stmt.test),
                                ctx,
                            );
                            result.pop();
                            self.mark_current_function_as_changed();
                        }
                    }

                    if if_stmt.alternate.is_some() {
                        // "if (a) return b; else if (c) return d; else return e;" => "if (a) return b; if (c) return d; return e;"
                        result.push(Statement::IfStatement(if_stmt));
                        loop {
                            if let Some(Statement::IfStatement(if_stmt)) = result.last_mut() {
                                if if_stmt.consequent.is_jump_statement() {
                                    if let Some(stmt) = if_stmt.alternate.take() {
                                        result.push(stmt);
                                        self.mark_current_function_as_changed();
                                        continue;
                                    }
                                }
                            }
                            break;
                        }
                        return;
                    }
                }

                result.push(Statement::IfStatement(if_stmt));
            }
            Statement::ReturnStatement(mut ret_stmt) => {
                if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                    if let Some(argument) = &mut ret_stmt.argument {
                        let a = &mut prev_expr_stmt.expression;
                        *argument = Self::join_sequence(a, argument, ctx);
                        result.pop();
                        self.mark_current_function_as_changed();
                    }
                }
                result.push(Statement::ReturnStatement(ret_stmt));
                *is_control_flow_dead = true;
            }
            Statement::ThrowStatement(mut throw_stmt) => {
                if let Some(Statement::ExpressionStatement(prev_expr_stmt)) = result.last_mut() {
                    let a = &mut prev_expr_stmt.expression;
                    let b = &mut throw_stmt.argument;
                    throw_stmt.argument = Self::join_sequence(a, b, ctx);
                    result.pop();
                    self.mark_current_function_as_changed();
                }
                result.push(Statement::ThrowStatement(throw_stmt));
                *is_control_flow_dead = true;
            }
            Statement::ForStatement(mut for_stmt) => {
                match result.last_mut() {
                    Some(Statement::ExpressionStatement(prev_expr_stmt)) => {
                        if let Some(init) = &mut for_stmt.init {
                            if let Some(init) = init.as_expression_mut() {
                                let a = &mut prev_expr_stmt.expression;
                                *init = Self::join_sequence(a, init, ctx);
                                result.pop();
                                self.mark_current_function_as_changed();
                            }
                        } else {
                            for_stmt.init = Some(ForStatementInit::from(
                                ctx.ast.move_expression(&mut prev_expr_stmt.expression),
                            ));
                            result.pop();
                            self.mark_current_function_as_changed();
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
                                        self.mark_current_function_as_changed();
                                    }
                                }
                            }
                        } else if prev_var_decl.kind.is_var() {
                            let var_decl = ctx.ast.move_variable_declaration(prev_var_decl);
                            for_stmt.init = Some(ForStatementInit::VariableDeclaration(
                                ctx.ast.alloc(var_decl),
                            ));
                            result.pop();
                            self.mark_current_function_as_changed();
                        }
                    }
                    _ => {}
                }
                result.push(Statement::ForStatement(for_stmt));
            }
            Statement::ForInStatement(mut for_in_stmt) => {
                match result.last_mut() {
                    // "a; for (var b in c) d" => "for (var b in a, c) d"
                    Some(Statement::ExpressionStatement(prev_expr_stmt)) => {
                        // Annex B.3.5 allows initializers in non-strict mode
                        // <https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-initializers-in-forin-statement-heads>
                        // If there's a side-effectful initializer, we should not move the previous statement inside.
                        let has_side_effectful_initializer = {
                            if let ForStatementLeft::VariableDeclaration(var_decl) =
                                &for_in_stmt.left
                            {
                                if var_decl.declarations.len() == 1 {
                                    // only var can have a initializer
                                    var_decl.kind.is_var()
                                        && var_decl.declarations[0].init.as_ref().is_some_and(
                                            |init| ctx.expression_may_have_side_effects(init),
                                        )
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
                            self.mark_current_function_as_changed();
                        }
                    }
                    // "var a; for (a in b) c" => "for (var a in b) c"
                    Some(Statement::VariableDeclaration(prev_var_decl)) => {
                        if let ForStatementLeft::AssignmentTargetIdentifier(id) = &for_in_stmt.left
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
                            if let Some(prev_var_decl_item) = prev_var_decl_no_init_item {
                                if let BindingPatternKind::BindingIdentifier(decl_id) =
                                    &prev_var_decl_item.id.kind
                                {
                                    if id.name == decl_id.name {
                                        for_in_stmt.left =
                                            ForStatementLeft::VariableDeclaration(ctx.ast.alloc(
                                                ctx.ast.move_variable_declaration(prev_var_decl),
                                            ));
                                        result.pop();
                                        self.mark_current_function_as_changed();
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                result.push(Statement::ForInStatement(for_in_stmt));
            }
            Statement::ForOfStatement(mut for_of_stmt) => {
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
                                    for_of_stmt.left =
                                        ForStatementLeft::VariableDeclaration(ctx.ast.alloc(
                                            ctx.ast.move_variable_declaration(prev_var_decl),
                                        ));
                                    result.pop();
                                    self.mark_current_function_as_changed();
                                }
                            }
                        }
                    }
                }
                result.push(Statement::ForOfStatement(for_of_stmt));
            }
            stmt => result.push(stmt),
        }
    }

    fn join_sequence(
        a: &mut Expression<'a>,
        b: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Expression<'a> {
        let a = ctx.ast.move_expression(a);
        let b = ctx.ast.move_expression(b);
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
}
