use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `mangleFor`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9801>
    pub fn minimize_for_statement(for_stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Get the first statement in the loop
        let mut first = &for_stmt.body;
        if let Some(block_stmt) = first.as_block_statement() {
            if let Some(b) = block_stmt.body.first() {
                first = b;
            } else {
                return;
            }
        }

        let Some(if_stmt) = first.as_if_statement() else {
            return;
        };
        // "for (;;) if (x) break;" => "for (; !x;) ;"
        // "for (; a;) if (x) break;" => "for (; a && !x;) ;"
        // "for (;;) if (x) break; else y();" => "for (; !x;) y();"
        // "for (; a;) if (x) break; else y();" => "for (; a && !x;) y();"
        if let Some(one_child) = if_stmt.consequent.get_one_child()
            && let Some(break_stmt) = one_child.as_break_statement()
        {
            if break_stmt.label.is_some() {
                return;
            }

            let span = for_stmt.body.span();
            let first_stmt;
            let body;
            if for_stmt.body.is_block_statement() {
                let mut block_stmt = for_stmt.body.take_in(ctx.ast).into_block_statement();
                first_stmt = block_stmt.body.get_mut(0).unwrap().take_in(ctx.ast);
                body = Some(Statement::block_statement(block_stmt));
            } else {
                first_stmt = for_stmt.body.take_in(ctx.ast);
                body = None;
            }

            let mut if_stmt_inner = first_stmt.into_if_statement().unbox();

            let expr = if let Some(unary_expr) = if_stmt_inner.test.as_unary_expression()
                && unary_expr.operator.is_not()
            {
                if_stmt_inner.test.into_unary_expression().unbox().argument
            } else {
                let e = if_stmt_inner.test.take_in(ctx.ast);
                Self::minimize_not(e.span(), e, ctx)
            };

            if let Some(test) = &mut for_stmt.test {
                let left = test.take_in(ctx.ast);
                let mut logical_expr =
                    ctx.ast.logical_expression(test.span(), left, LogicalOperator::And, expr);
                *test = Self::try_fold_and_or(&mut logical_expr, ctx)
                    .unwrap_or_else(|| Expression::logical_expression(ctx.ast.alloc(logical_expr)));
            } else {
                for_stmt.test = Some(expr);
            }

            let alternate = if_stmt_inner.alternate.take();
            for_stmt.body = Self::drop_first_statement(span, body, alternate, ctx);
            ctx.state.changed = true;
            return;
        }
        // "for (;;) if (x) y(); else break;" => "for (; x;) y();"
        // "for (; a;) if (x) y(); else break;" => "for (; a && x;) y();"
        if let Some(alt) = &if_stmt.alternate
            && let Some(one_child) = alt.get_one_child()
            && let Some(break_stmt) = one_child.as_break_statement()
        {
            if break_stmt.label.is_some() {
                return;
            }

            let span = for_stmt.body.span();
            let first_stmt;
            let body;
            if for_stmt.body.is_block_statement() {
                let mut block_stmt = for_stmt.body.take_in(ctx.ast).into_block_statement();
                first_stmt = block_stmt.body.get_mut(0).unwrap().take_in(ctx.ast);
                body = Some(Statement::block_statement(block_stmt));
            } else {
                first_stmt = for_stmt.body.take_in(ctx.ast);
                body = None;
            }

            let mut if_stmt_inner = first_stmt.into_if_statement().unbox();

            let expr = if_stmt_inner.test.take_in(ctx.ast);

            if let Some(test) = &mut for_stmt.test {
                let left = test.take_in(ctx.ast);
                let mut logical_expr =
                    ctx.ast.logical_expression(test.span(), left, LogicalOperator::And, expr);
                *test = Self::try_fold_and_or(&mut logical_expr, ctx)
                    .unwrap_or_else(|| Expression::logical_expression(ctx.ast.alloc(logical_expr)));
            } else {
                for_stmt.test = Some(expr);
            }

            let consequent = if_stmt_inner.consequent.take_in(ctx.ast);
            for_stmt.body = Self::drop_first_statement(span, body, Some(consequent), ctx);
            ctx.state.changed = true;
        }
    }

    fn drop_first_statement(
        span: Span,
        body: Option<Statement<'a>>,
        replace: Option<Statement<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        if let Some(body) = body {
            if body.is_block_statement() {
                let mut block_stmt = body.into_block_statement();
                if !block_stmt.body.is_empty() {
                    if let Some(replace) = replace {
                        block_stmt.body[0] = replace;
                    } else if block_stmt.body.len() == 2
                        && !Self::statement_cares_about_scope(&block_stmt.body[1])
                    {
                        return block_stmt.body[1].take_in(ctx.ast);
                    } else {
                        block_stmt.body.remove(0);
                    }
                    return Statement::block_statement(block_stmt);
                }
            }
        }
        replace.unwrap_or_else(|| ctx.ast.statement_empty(span))
    }
}
