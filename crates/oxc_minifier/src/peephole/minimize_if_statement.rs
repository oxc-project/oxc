use oxc_ast::ast::*;

use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::TraverseCtx;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `MangleIf`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser.go#L9860>
    pub fn try_minimize_if(
        &mut self,
        if_stmt: &mut IfStatement<'a>,
        traverse_ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        self.wrap_to_avoid_ambiguous_else(if_stmt, traverse_ctx);
        let ctx = Ctx(traverse_ctx);
        if let Statement::ExpressionStatement(expr_stmt) = &mut if_stmt.consequent {
            if if_stmt.alternate.is_none() {
                let (op, e) = match &mut if_stmt.test {
                    // "if (!a) b();" => "a || b();"
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        (LogicalOperator::Or, &mut unary_expr.argument)
                    }
                    // "if (a) b();" => "a && b();"
                    e => (LogicalOperator::And, e),
                };
                let a = ctx.ast.move_expression(e);
                let b = ctx.ast.move_expression(&mut expr_stmt.expression);
                let expr = Self::join_with_left_associative_op(if_stmt.span, op, a, b, ctx);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            } else if let Some(Statement::ExpressionStatement(alternate_expr_stmt)) =
                &mut if_stmt.alternate
            {
                // "if (a) b(); else c();" => "a ? b() : c();"
                let test = ctx.ast.move_expression(&mut if_stmt.test);
                let consequent = ctx.ast.move_expression(&mut expr_stmt.expression);
                let alternate = ctx.ast.move_expression(&mut alternate_expr_stmt.expression);
                let expr =
                    self.minimize_conditional(if_stmt.span, test, consequent, alternate, ctx);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            }
        } else if Self::is_statement_empty(&if_stmt.consequent) {
            if if_stmt.alternate.is_none()
                || if_stmt.alternate.as_ref().is_some_and(Self::is_statement_empty)
            {
                // "if (a) {}" => "a;"
                let expr = ctx.ast.move_expression(&mut if_stmt.test);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            } else if let Some(Statement::ExpressionStatement(expr_stmt)) = &mut if_stmt.alternate {
                let (op, e) = match &mut if_stmt.test {
                    // "if (!a) {} else b();" => "a && b();"
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        (LogicalOperator::And, &mut unary_expr.argument)
                    }
                    // "if (a) {} else b();" => "a || b();"
                    e => (LogicalOperator::Or, e),
                };
                let a = ctx.ast.move_expression(e);
                let b = ctx.ast.move_expression(&mut expr_stmt.expression);
                let expr = Self::join_with_left_associative_op(if_stmt.span, op, a, b, ctx);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            } else if let Some(stmt) = &mut if_stmt.alternate {
                // "yes" is missing and "no" is not missing (and is not an expression)
                match &mut if_stmt.test {
                    // "if (!a) {} else return b;" => "if (a) return b;"
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        if_stmt.test = ctx.ast.move_expression(&mut unary_expr.argument);
                        if_stmt.consequent = ctx.ast.move_statement(stmt);
                        if_stmt.alternate = None;
                        self.mark_current_function_as_changed();
                    }
                    // "if (a) {} else return b;" => "if (!a) return b;"
                    _ => {
                        if_stmt.test = Self::minimize_not(
                            if_stmt.test.span(),
                            ctx.ast.move_expression(&mut if_stmt.test),
                            ctx,
                        );
                        if_stmt.consequent = ctx.ast.move_statement(stmt);
                        if_stmt.alternate = None;
                        self.mark_current_function_as_changed();
                    }
                }
            }
        } else {
            // "yes" is not missing (and is not an expression)
            if let Some(alternate) = &mut if_stmt.alternate {
                // "yes" is not missing (and is not an expression) and "no" is not missing
                if let Expression::UnaryExpression(unary_expr) = &mut if_stmt.test {
                    if unary_expr.operator.is_not() {
                        // "if (!a) return b; else return c;" => "if (a) return c; else return b;"
                        if_stmt.test = ctx.ast.move_expression(&mut unary_expr.argument);
                        std::mem::swap(&mut if_stmt.consequent, alternate);
                        self.wrap_to_avoid_ambiguous_else(if_stmt, traverse_ctx);
                        self.mark_current_function_as_changed();
                    }
                }
                // "if (a) return b; else {}" => "if (a) return b;" is handled by remove_dead_code
            } else {
                // "no" is missing
                if let Statement::IfStatement(if2_stmt) = &mut if_stmt.consequent {
                    if if2_stmt.alternate.is_none() {
                        // "if (a) if (b) return c;" => "if (a && b) return c;"
                        let a = ctx.ast.move_expression(&mut if_stmt.test);
                        let b = ctx.ast.move_expression(&mut if2_stmt.test);
                        if_stmt.test = Self::join_with_left_associative_op(
                            if_stmt.test.span(),
                            LogicalOperator::And,
                            a,
                            b,
                            ctx,
                        );
                        if_stmt.consequent = ctx.ast.move_statement(&mut if2_stmt.consequent);
                        self.mark_current_function_as_changed();
                    }
                }
            }
        }
        None
    }

    /// Wrap to avoid ambiguous else.
    /// `if (foo) if (bar) baz else quaz` ->  `if (foo) { if (bar) baz else quaz }`
    fn wrap_to_avoid_ambiguous_else(
        &mut self,
        if_stmt: &mut IfStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Statement::IfStatement(if2) = &mut if_stmt.consequent {
            if if2.consequent.is_jump_statement() && if2.alternate.is_some() {
                let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
                if_stmt.consequent = Statement::BlockStatement(ctx.ast.alloc(
                    ctx.ast.block_statement_with_scope_id(
                        if_stmt.consequent.span(),
                        ctx.ast.vec1(ctx.ast.move_statement(&mut if_stmt.consequent)),
                        scope_id,
                    ),
                ));
                self.mark_current_function_as_changed();
            }
        }
    }

    fn is_statement_empty(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::BlockStatement(block_stmt) if block_stmt.body.is_empty() => true,
            Statement::EmptyStatement(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tester::test;

    #[test]
    fn test_minimize_if() {
        test(
            "function writeInteger(int) {
                if (int >= 0)
                    if (int <= 0xffffffff) return this.u32(int);
                    else if (int > -0x80000000) return this.n32(int);
            }",
            "function writeInteger(int) {
                if (int >= 0) {
                    if (int <= 4294967295) return this.u32(int);
                    if (int > -2147483648) return this.n32(int);
                }
            }",
        );

        test(
            "function bar() {
              if (!x) {
                return null;
              } else if (y) {
                return foo;
              } else if (z) {
                return bar;
              }
            }",
            "function bar() {
              if (x) {
                if (y)
                  return foo;
                if (z)
                  return bar;
              } else return null;
            }",
        );

        test(
            "function f() {
              if (foo)
                if (bar) return X;
                else return Y;
              return Z;
            }",
            "function f() {
              return foo ? bar ? X : Y : Z;
            }",
        );

        test(
            "function _() {
                if (currentChar === '\\n')
                    return pos + 1;
                else if (currentChar !== ' ' && currentChar !== '\\t')
                    return pos + 1;
            }",
            "function _() {
                if (currentChar === '\\n' || currentChar !== ' ' && currentChar !== '\\t')
                    return pos + 1;
            }",
        );
    }
}
