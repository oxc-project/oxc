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
        // Flip empty consequent so the rest of the function can assume consequent is non-empty.
        if Self::is_statement_empty(&if_stmt.consequent) {
            if if_stmt.alternate.is_none() {
                // `if (a) {}` => `a;`
                let mut expr = if_stmt.test.take_in(ctx);
                Self::remove_unused_expression(&mut expr, ctx);
                return Some(Statement::new_expression_statement(if_stmt.span, expr, ctx));
            }
            let mut new_consequent = if_stmt.alternate.take().unwrap();

            if let Statement::ExpressionStatement(expr_stmt) = &mut new_consequent {
                let (op, a) = match &mut if_stmt.test {
                    // `if (!a); else b();` => `a && b();`
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        (LogicalOperator::And, unary_expr.argument.take_in(ctx))
                    }
                    // `if (a); else b();` => `a || b();`
                    e => (LogicalOperator::Or, e.take_in(ctx)),
                };
                let b = expr_stmt.expression.take_in(ctx);
                let expr = Self::join_with_left_associative_op(if_stmt.span, op, a, b, ctx);
                return Some(Statement::new_expression_statement(if_stmt.span, expr, ctx));
            }

            // `if (!a) {} else x;` => `if (a) x;`
            // `if (a)  {} else x;` => `if (!a) x;`
            ctx.replace_expression_with(&mut if_stmt.test, |old, ctx| {
                Self::minimize_not(old.span(), old, ctx, true)
            });
            ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
        }

        // Consequent is non-empty from here on.

        if let Some(alternate) = &mut if_stmt.alternate {
            if let Statement::ExpressionStatement(expr_stmt) = &mut if_stmt.consequent {
                if let Statement::ExpressionStatement(alternate_expr_stmt) = alternate {
                    // `if (a) b(); else c();` => `a ? b() : c();`
                    let test = if_stmt.test.take_in(ctx);
                    let consequent = expr_stmt.expression.take_in(ctx);
                    let alternate = alternate_expr_stmt.expression.take_in(ctx);
                    let expr =
                        Self::minimize_conditional(if_stmt.span, test, consequent, alternate, ctx);
                    return Some(Statement::new_expression_statement(if_stmt.span, expr, ctx));
                }
            } else {
                // Normalize: move the `!` out of the test by swapping branches.
                // Avoid swapping when alternate is an `if` — that risks a worse chain.
                // `if (!a) return b; else return c;` => `if (a) return c; else return b;`
                if !matches!(alternate, Statement::IfStatement(_))
                    && let Expression::UnaryExpression(unary_expr) = &mut if_stmt.test
                    && unary_expr.operator.is_not()
                {
                    ctx.replace_expression_with(&mut if_stmt.test, Self::unwrap_unary);
                    std::mem::swap(&mut if_stmt.consequent, alternate);
                }
            }
        } else if let Statement::ExpressionStatement(expr_stmt) = &mut if_stmt.consequent {
            let (op, a) = match &mut if_stmt.test {
                // `if (!a) b();` => `a || b();`
                Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                    (LogicalOperator::Or, unary_expr.argument.take_in(ctx))
                }
                // `if (a)  b();` => `a && b();`
                e => (LogicalOperator::And, e.take_in(ctx)),
            };
            let b = expr_stmt.expression.take_in(ctx);
            let expr = Self::join_with_left_associative_op(if_stmt.span, op, a, b, ctx);
            return Some(Statement::new_expression_statement(if_stmt.span, expr, ctx));
        } else if let Statement::IfStatement(if2_stmt) = &mut if_stmt.consequent
            && if2_stmt.alternate.is_none()
        {
            // `if (a) if (b) x;` => `if (a && b) x;`
            let a = if_stmt.test.take_in(ctx);
            let b = if2_stmt.test.take_in(ctx);
            let new_test = Self::join_with_left_associative_op(
                if_stmt.test.span(),
                LogicalOperator::And,
                a,
                b,
                ctx,
            );
            let new_consequent = if2_stmt.consequent.take_in(ctx);
            ctx.replace_expression(&mut if_stmt.test, new_test);
            ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
        }

        Self::wrap_to_avoid_ambiguous_else(if_stmt, ctx);
        None
    }

    /// Wrap to avoid ambiguous else.
    /// `if (foo) if (bar) baz else quaz` ->  `if (foo) { if (bar) baz else quaz }`
    fn wrap_to_avoid_ambiguous_else(if_stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::IfStatement(if2) = &if_stmt.consequent
            && if2.alternate.is_some()
        {
            let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
            let new_consequent = Statement::new_block_statement_with_scope_id(
                if_stmt.consequent.span(),
                [if_stmt.consequent.take_in(ctx)],
                scope_id,
                ctx,
            );
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
