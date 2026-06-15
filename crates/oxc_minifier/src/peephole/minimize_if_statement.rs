use oxc_allocator::TakeIn;
use oxc_ast::ast::*;

use oxc_semantic::ScopeFlags;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `MangleIf`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser.go#L9860>
    pub fn try_minimize_if(
        if_stmt: &mut IfStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        Self::wrap_to_avoid_ambiguous_else(if_stmt, ctx);
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
                let a = e.take_in(ctx.ast);
                let b = expr_stmt.expression.take_in(ctx.ast);
                let expr = Self::join_with_left_associative_op(if_stmt.span, op, a, b, ctx);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            } else if let Some(Statement::ExpressionStatement(alternate_expr_stmt)) =
                &mut if_stmt.alternate
            {
                // "if (a) b(); else c();" => "a ? b() : c();"
                let test = if_stmt.test.take_in(ctx.ast);
                let consequent = expr_stmt.expression.take_in(ctx.ast);
                let alternate = alternate_expr_stmt.expression.take_in(ctx.ast);
                let expr =
                    Self::minimize_conditional(if_stmt.span, test, consequent, alternate, ctx);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            }
        } else if Self::is_statement_empty(&if_stmt.consequent) {
            if if_stmt.alternate.is_none()
                || if_stmt.alternate.as_ref().is_some_and(Self::is_statement_empty)
            {
                // "if (a) {}" => "a;"
                let mut expr = if_stmt.test.take_in(ctx.ast);
                Self::remove_unused_expression(&mut expr, ctx);
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
                let a = e.take_in(ctx.ast);
                let b = expr_stmt.expression.take_in(ctx.ast);
                let expr = Self::join_with_left_associative_op(if_stmt.span, op, a, b, ctx);
                return Some(ctx.ast.statement_expression(if_stmt.span, expr));
            } else if let Some(stmt) = &mut if_stmt.alternate {
                // "yes" is missing and "no" is not missing (and is not an expression)
                match &mut if_stmt.test {
                    // "if (!a) {} else return b;" => "if (a) return b;"
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        let new_test = unary_expr.argument.take_in(ctx.ast);
                        let new_consequent = stmt.take_in(ctx.ast);
                        ctx.replace_expression(&mut if_stmt.test, new_test);
                        ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
                        if_stmt.alternate = None;
                    }
                    // "if (a) {} else return b;" => "if (!a) return b;"
                    _ => {
                        let new_test = Self::minimize_not(
                            if_stmt.test.span(),
                            if_stmt.test.take_in(ctx.ast),
                            ctx,
                        );
                        let new_consequent = stmt.take_in(ctx.ast);
                        ctx.replace_expression(&mut if_stmt.test, new_test);
                        ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
                        if_stmt.alternate = None;
                        Self::try_minimize_if(if_stmt, ctx);
                    }
                }
            }
        } else {
            // "yes" is not missing (and is not an expression)
            if let Some(alternate) = &mut if_stmt.alternate {
                // "yes" is not missing (and is not an expression) and "no" is not missing
                if !matches!(alternate, Statement::IfStatement(_))
                    && let Expression::UnaryExpression(unary_expr) = &mut if_stmt.test
                    && unary_expr.operator.is_not()
                {
                    // "if (!a) return b; else return c;" => "if (a) return c; else return b;"
                    let new_test = unary_expr.argument.take_in(ctx.ast);
                    ctx.replace_expression(&mut if_stmt.test, new_test);
                    std::mem::swap(&mut if_stmt.consequent, alternate);
                    Self::wrap_to_avoid_ambiguous_else(if_stmt, ctx);
                }
                // "if (!a) {} else if (b) {}" => "if (!a) {} if (b) {}" is handled by minimize_statements
                // "if (a) return b; else {}" => "if (a) return b;" is handled by remove_dead_code
            } else {
                // "no" is missing
                if let Statement::IfStatement(if2_stmt) = &mut if_stmt.consequent
                    && if2_stmt.alternate.is_none()
                {
                    // "if (a) if (b) return c;" => "if (a && b) return c;"
                    let a = if_stmt.test.take_in(ctx.ast);
                    let b = if2_stmt.test.take_in(ctx.ast);
                    let new_test = Self::join_with_left_associative_op(
                        if_stmt.test.span(),
                        LogicalOperator::And,
                        a,
                        b,
                        ctx,
                    );
                    let new_consequent = if2_stmt.consequent.take_in(ctx.ast);
                    ctx.replace_expression(&mut if_stmt.test, new_test);
                    ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
                }
            }
        }
        None
    }

    /// Wrap to avoid ambiguous else.
    /// `if (foo) if (bar) baz else quaz` ->  `if (foo) { if (bar) baz else quaz }`
    fn wrap_to_avoid_ambiguous_else(if_stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::IfStatement(if2) = &mut if_stmt.consequent
            && if2.consequent.is_jump_statement()
            && if2.alternate.is_some()
        {
            let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
            let new_consequent =
                Statement::BlockStatement(ctx.ast.alloc(ctx.ast.block_statement_with_scope_id(
                    if_stmt.consequent.span(),
                    ctx.ast.vec1(if_stmt.consequent.take_in(ctx.ast)),
                    scope_id,
                )));
            ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
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
