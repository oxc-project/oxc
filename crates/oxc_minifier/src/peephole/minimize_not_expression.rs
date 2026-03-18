use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::DetermineValueType;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_not(
        span: Span,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut unary = ctx.ast.expression_unary(span, UnaryOperator::LogicalNot, expr);
        Self::minimize_unary(&mut unary, ctx);
        unary
    }

    /// `MaybeSimplifyNot`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L73>
    pub fn minimize_unary(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::UnaryExpression(e) = expr else { return };
        if !e.operator.is_not() {
            return;
        }
        Self::minimize_expression_in_boolean_context(&mut e.argument, ctx);
        match &mut e.argument {
            // `!!true` -> `true`
            // `!!false` -> `false`
            Expression::UnaryExpression(e)
                if e.operator.is_not() && e.argument.value_type(ctx).is_boolean() =>
            {
                *expr = e.argument.take_in(ctx.ast);
                ctx.state.changed = true;
            }
            // `!(a == b)` => `a != b`
            // `!(a != b)` => `a == b`
            // `!(a === b)` => `a !== b`
            // `!(a !== b)` => `a === b`
            Expression::BinaryExpression(binary_expr) if binary_expr.operator.is_equality() => {
                binary_expr.operator = binary_expr.operator.equality_inverse_operator().unwrap();
                *expr = e.argument.take_in(ctx.ast);
                ctx.state.changed = true;
            }
            // "!(a, b)" => "a, !b"
            Expression::SequenceExpression(sequence_expr) => {
                if let Some(last_expr) = sequence_expr.expressions.last_mut() {
                    *last_expr =
                        Self::minimize_not(last_expr.span(), last_expr.take_in(ctx.ast), ctx);
                    *expr = e.argument.take_in(ctx.ast);
                    ctx.state.changed = true;
                }
            }
            _ => {}
        }
    }
}
