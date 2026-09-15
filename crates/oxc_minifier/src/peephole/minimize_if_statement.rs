use oxc_allocator::TakeIn;
use oxc_ast::ast::*;

use oxc_semantic::ScopeFlags;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `MangleIf`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser.go#L9860>
    pub fn try_minimize_if(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };

        // Flip empty consequent so the rest of the function can assume consequent is non-empty.
        if Self::is_statement_empty(&if_stmt.consequent) {
            if if_stmt.alternate.is_none() {
                // `if (a) {}` => `a;`
                ctx.replace_statement_with(stmt, |stmt, ctx| {
                    let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                    let IfStatement { mut test, span, .. } = if_stmt.unbox();
                    Self::remove_unused_expression(&mut test, ctx);
                    Statement::new_expression_statement(span, test, ctx)
                });
                return;
            }
            let new_consequent = if_stmt.alternate.take().unwrap();

            if let Statement::ExpressionStatement(expr_stmt) = new_consequent {
                ctx.replace_statement_with(stmt, |stmt, ctx| {
                    let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                    let IfStatement { test, span, .. } = if_stmt.unbox();
                    let (op, a) = match test {
                        // `if (!a); else b();` => `a && b();`
                        Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                            (LogicalOperator::And, unary_expr.unbox().argument)
                        }
                        // `if (a); else b();` => `a || b();`
                        e => (LogicalOperator::Or, e),
                    };
                    let b = expr_stmt.unbox().expression;
                    let expr = Self::join_with_left_associative_op(span, op, a, b, ctx);
                    Statement::new_expression_statement(span, expr, ctx)
                });
                return;
            }

            // `if (!a) {} else x;` => `if (a) x;`
            // `if (a)  {} else x;` => `if (!a) x;`
            ctx.replace_expression_with(&mut if_stmt.test, |old, ctx| {
                Self::minimize_not(old.span(), old, ctx, true)
            });
            ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
        }

        // Consequent is non-empty from here on.

        if let Some(alternate) = &if_stmt.alternate {
            if matches!(&if_stmt.consequent, Statement::ExpressionStatement(_)) {
                if matches!(alternate, Statement::ExpressionStatement(_)) {
                    // `if (a) b(); else c();` => `a ? b() : c();`
                    ctx.replace_statement_with(stmt, |stmt, ctx| {
                        let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                        let IfStatement { test, consequent, alternate, span, .. } = if_stmt.unbox();
                        let Statement::ExpressionStatement(a) = consequent else { unreachable!() };
                        let Statement::ExpressionStatement(b) = alternate.unwrap() else {
                            unreachable!()
                        };
                        let a = a.unbox().expression;
                        let b = b.unbox().expression;
                        let expr = Self::minimize_conditional(span, test, a, b, ctx);
                        Statement::new_expression_statement(span, expr, ctx)
                    });
                    return;
                }
            } else {
                // Normalize: move the `!` out of the test by swapping branches.
                // Avoid swapping when alternate is an `if` — that risks a worse chain.
                // `if (!a) return b; else return c;` => `if (a) return c; else return b;`
                if !matches!(alternate, Statement::IfStatement(_))
                    && let Expression::UnaryExpression(unary_expr) = &if_stmt.test
                    && unary_expr.operator.is_not()
                {
                    ctx.replace_expression_with(&mut if_stmt.test, Self::unwrap_unary);
                    let if_mut = if_stmt.as_mut();
                    let Some(alternate) = &mut if_mut.alternate else { unreachable!() };
                    std::mem::swap(&mut if_mut.consequent, alternate);
                }
            }
        } else if matches!(&if_stmt.consequent, Statement::ExpressionStatement(_)) {
            ctx.replace_statement_with(stmt, |stmt, ctx| {
                let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                let IfStatement { test, consequent, span, .. } = if_stmt.unbox();
                let Statement::ExpressionStatement(b) = consequent else { unreachable!() };
                let (op, a) = match test {
                    // `if (!a) b();` => `a || b();`
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        (LogicalOperator::Or, unary_expr.unbox().argument)
                    }
                    // `if (a)  b();` => `a && b();`
                    e => (LogicalOperator::And, e),
                };
                let b = b.unbox().expression;
                let expr = Self::join_with_left_associative_op(span, op, a, b, ctx);
                Statement::new_expression_statement(span, expr, ctx)
            });
            return;
        } else if let Statement::IfStatement(if2_stmt) = &mut if_stmt.consequent
            && if2_stmt.alternate.is_none()
        {
            // `if (a) if (b) x;` => `if (a && b) x;`
            let IfStatement { test, consequent, .. } = if2_stmt.take_in(ctx);
            ctx.replace_expression_with(&mut if_stmt.test, |a, ctx| {
                Self::join_with_left_associative_op(test.span(), LogicalOperator::And, a, test, ctx)
            });
            ctx.replace_statement(&mut if_stmt.consequent, consequent);
        }

        Self::wrap_to_avoid_ambiguous_else(if_stmt, ctx);
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
