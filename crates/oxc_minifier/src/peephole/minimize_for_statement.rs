use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `mangleFor`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9801>
    pub fn minimize_for_statement(for_stmt: &mut ForStatement<'a>, ctx: &mut Ctx<'a, '_>) {
        // Get the first statement in the loop
        let mut first = &for_stmt.body;
        if let Statement::BlockStatement(block_stmt) = first {
            if let Some(b) = block_stmt.body.first() {
                first = b;
            } else {
                return;
            }
        }

        let Statement::IfStatement(if_stmt) = first else {
            return;
        };
        // "for (;;) if (x) break;" => "for (; !x;) ;"
        // "for (; a;) if (x) break;" => "for (; a && !x;) ;"
        // "for (;;) if (x) break; else y();" => "for (; !x;) y();"
        // "for (; a;) if (x) break; else y();" => "for (; a && !x;) y();"
        if let Some(Statement::BreakStatement(break_stmt)) = if_stmt.consequent.get_one_child() {
            if break_stmt.label.is_some() {
                return;
            }

            let span = for_stmt.body.span();
            let (first, body) = match for_stmt.body.take_in(ctx.ast) {
                Statement::BlockStatement(mut block_stmt) => (
                    block_stmt.body.get_mut(0).unwrap().take_in(ctx.ast),
                    Some(Statement::BlockStatement(block_stmt)),
                ),
                stmt => (stmt, None),
            };

            let Statement::IfStatement(mut if_stmt) = first else { unreachable!() };

            let expr = match if_stmt.test.take_in(ctx.ast) {
                Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                    unary_expr.unbox().argument
                }
                e => Self::minimize_not(e.span(), e, ctx),
            };

            if let Some(test) = &mut for_stmt.test {
                let left = test.take_in(ctx.ast);
                let mut logical_expr =
                    ctx.ast.logical_expression(test.span(), left, LogicalOperator::And, expr);
                *test = Self::try_fold_and_or(&mut logical_expr, ctx)
                    .unwrap_or_else(|| Expression::LogicalExpression(ctx.ast.alloc(logical_expr)));
            } else {
                for_stmt.test = Some(expr);
            }

            let alternate = if_stmt.alternate.take();
            for_stmt.body = Self::drop_first_statement(span, body, alternate, ctx);
            ctx.state.changed = true;
            return;
        }
        // "for (;;) if (x) y(); else break;" => "for (; x;) y();"
        // "for (; a;) if (x) y(); else break;" => "for (; a && x;) y();"
        if let Some(Statement::BreakStatement(break_stmt)) =
            if_stmt.alternate.as_ref().and_then(|stmt| stmt.get_one_child())
        {
            if break_stmt.label.is_some() {
                return;
            }

            let span = for_stmt.body.span();
            let (first, body) = match for_stmt.body.take_in(ctx.ast) {
                Statement::BlockStatement(mut block_stmt) => (
                    block_stmt.body.get_mut(0).unwrap().take_in(ctx.ast),
                    Some(Statement::BlockStatement(block_stmt)),
                ),
                stmt => (stmt, None),
            };

            let Statement::IfStatement(mut if_stmt) = first else { unreachable!() };

            let expr = if_stmt.test.take_in(ctx.ast);

            if let Some(test) = &mut for_stmt.test {
                let left = test.take_in(ctx.ast);
                let mut logical_expr =
                    ctx.ast.logical_expression(test.span(), left, LogicalOperator::And, expr);
                *test = Self::try_fold_and_or(&mut logical_expr, ctx)
                    .unwrap_or_else(|| Expression::LogicalExpression(ctx.ast.alloc(logical_expr)));
            } else {
                for_stmt.test = Some(expr);
            }

            let consequent = if_stmt.consequent.take_in(ctx.ast);
            for_stmt.body = Self::drop_first_statement(span, body, Some(consequent), ctx);
            ctx.state.changed = true;
        }
    }

    fn drop_first_statement(
        span: Span,
        body: Option<Statement<'a>>,
        replace: Option<Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Statement<'a> {
        match body {
            Some(Statement::BlockStatement(mut block_stmt)) if !block_stmt.body.is_empty() => {
                if let Some(replace) = replace {
                    block_stmt.body[0] = replace;
                } else if block_stmt.body.len() == 2
                    && !Self::statement_cares_about_scope(&block_stmt.body[1])
                {
                    return block_stmt.body[1].take_in(ctx.ast);
                } else {
                    block_stmt.body.remove(0);
                }
                Statement::BlockStatement(block_stmt)
            }
            _ => replace.unwrap_or_else(|| ctx.ast.statement_empty(span)),
        }
    }
}
