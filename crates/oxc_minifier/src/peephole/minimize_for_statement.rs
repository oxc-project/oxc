use oxc_allocator::{ArenaBox, TakeIn};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `mangleFor`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9801>
    pub fn minimize_for_statement(for_stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Greedily pull leading `if (c) break;` guards into the for-test in one
        // pass — pulling one per pass would need a pass per guard and never
        // converge on a long run. Stop once the test's `&&` spine reaches the cap
        // so the run can't rebuild an expression deep enough to overflow. The cap
        // check is idempotent: a later pass re-enters with the test already at the
        // cap and pulls nothing, so any leftover guards stay in the body — correct
        // and terminal.
        loop {
            if for_stmt.test.as_ref().is_some_and(Self::logical_expression_count_exceeded) {
                return;
            }
            if !Self::pull_leading_break_into_for_test(for_stmt, ctx) {
                return;
            }
        }
    }

    /// Pull a single leading `if (c) break;` (or `if (x) y(); else break;`) guard
    /// from the loop body into the for-test, returning `true` when one was pulled.
    fn pull_leading_break_into_for_test(
        for_stmt: &mut ForStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        // Get the first statement in the loop
        let mut first = &for_stmt.body;
        if let Statement::BlockStatement(block_stmt) = first {
            if let Some(b) = block_stmt.body.first() {
                first = b;
            } else {
                return false;
            }
        }

        let Statement::IfStatement(if_stmt) = first else {
            return false;
        };
        // "for (;;) if (x) break;" => "for (; !x;) ;"
        // "for (; a;) if (x) break;" => "for (; a && !x;) ;"
        // "for (;;) if (x) break; else y();" => "for (; !x;) y();"
        // "for (; a;) if (x) break; else y();" => "for (; a && !x;) y();"
        if let Some(Statement::BreakStatement(break_stmt)) = if_stmt.consequent.get_one_child() {
            if break_stmt.label.is_some() {
                return false;
            }

            let span = for_stmt.body.span();
            let (first, body) = match for_stmt.body.take_in(ctx) {
                Statement::BlockStatement(mut block_stmt) => (
                    block_stmt.body.get_mut(0).unwrap().take_in(ctx),
                    Some(Statement::BlockStatement(block_stmt)),
                ),
                stmt => (stmt, None),
            };

            let Statement::IfStatement(mut if_stmt) = first else { unreachable!() };

            let expr = match if_stmt.test.take_in(ctx) {
                Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                    unary_expr.unbox().argument
                }
                e => Self::minimize_not(e.span(), e, ctx),
            };

            if let Some(test) = &mut for_stmt.test {
                let left = test.take_in(ctx);
                let mut logical_expr =
                    LogicalExpression::new(test.span(), left, LogicalOperator::And, expr, ctx);
                let new_test = Self::try_fold_and_or(&mut logical_expr, ctx).unwrap_or_else(|| {
                    Expression::LogicalExpression(ArenaBox::new_in(logical_expr, ctx))
                });
                ctx.replace_expression(test, new_test);
            } else {
                for_stmt.test = Some(expr);
            }

            let alternate = if_stmt.alternate.take();
            let new_body = Self::drop_first_statement(span, body, alternate, ctx);
            ctx.replace_statement(&mut for_stmt.body, new_body);
            return true;
        }
        // "for (;;) if (x) y(); else break;" => "for (; x;) y();"
        // "for (; a;) if (x) y(); else break;" => "for (; a && x;) y();"
        if let Some(Statement::BreakStatement(break_stmt)) =
            if_stmt.alternate.as_ref().and_then(|stmt| stmt.get_one_child())
        {
            if break_stmt.label.is_some() {
                return false;
            }

            let span = for_stmt.body.span();
            let (first, body) = match for_stmt.body.take_in(ctx) {
                Statement::BlockStatement(mut block_stmt) => (
                    block_stmt.body.get_mut(0).unwrap().take_in(ctx),
                    Some(Statement::BlockStatement(block_stmt)),
                ),
                stmt => (stmt, None),
            };

            let Statement::IfStatement(mut if_stmt) = first else { unreachable!() };

            let expr = if_stmt.test.take_in(ctx);

            if let Some(test) = &mut for_stmt.test {
                let left = test.take_in(ctx);
                let mut logical_expr =
                    LogicalExpression::new(test.span(), left, LogicalOperator::And, expr, ctx);
                let new_test = Self::try_fold_and_or(&mut logical_expr, ctx).unwrap_or_else(|| {
                    Expression::LogicalExpression(ArenaBox::new_in(logical_expr, ctx))
                });
                ctx.replace_expression(test, new_test);
            } else {
                for_stmt.test = Some(expr);
            }

            let consequent = if_stmt.consequent.take_in(ctx);
            let new_body = Self::drop_first_statement(span, body, Some(consequent), ctx);
            ctx.replace_statement(&mut for_stmt.body, new_body);
            return true;
        }
        false
    }

    fn drop_first_statement(
        span: Span,
        body: Option<Statement<'a>>,
        replace: Option<Statement<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        match body {
            Some(Statement::BlockStatement(mut block_stmt)) if !block_stmt.body.is_empty() => {
                if let Some(replace) = replace {
                    block_stmt.body[0] = replace;
                } else if block_stmt.body.len() == 2
                    && !Self::statement_cares_about_scope(&block_stmt.body[1])
                {
                    return block_stmt.body[1].take_in(ctx);
                } else {
                    block_stmt.body.remove(0);
                }
                Statement::BlockStatement(block_stmt)
            }
            _ => replace.unwrap_or_else(|| Statement::new_empty_statement(span, ctx)),
        }
    }
}
