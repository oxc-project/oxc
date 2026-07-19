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
        let mut unary =
            Expression::new_unary_expression(span, UnaryOperator::LogicalNot, expr, ctx);
        Self::minimize_unary(&mut unary, ctx);
        unary
    }

    /// `MaybeSimplifyNot`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L73>
    pub fn minimize_unary(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Some(e) = expr.as_unary_expression_mut() else { return };
        if !e.operator.is_not() {
            return;
        }
        Self::minimize_expression_in_boolean_context(&mut e.argument, ctx);
        match e.argument.kind_mut() {
            // `!!true` -> `true`
            // `!!false` -> `false`
            ExpressionKindMut::UnaryExpression(e)
                if e.operator.is_not() && e.argument.value_type(ctx).is_boolean() =>
            {
                let new_expr = e.argument.take_in(ctx);
                ctx.replace_expression(expr, new_expr);
            }
            // `!(a == b)` => `a != b`
            // `!(a != b)` => `a == b`
            // `!(a === b)` => `a !== b`
            // `!(a !== b)` => `a === b`
            ExpressionKindMut::BinaryExpression(binary_expr)
                if binary_expr.operator.is_equality() =>
            {
                binary_expr.operator = binary_expr.operator.equality_inverse_operator().unwrap();
                let new_expr = e.argument.take_in(ctx);
                ctx.replace_expression(expr, new_expr);
            }
            // `!(a == b || c == d)` => `a != b && c != d`
            // `!(a == b && c == d)` => `a != b || c != d`
            // De Morgan's law, only when every comparison in the `&&`/`||` chain
            // inverts its operator in place (equality operators; relational ones
            // are unsound under NaN) and inversion does not add parentheses.
            // The fold is exact and involutive: a later `minimize_not` restores
            // the original chain at no cost, so shapes that consume the `!` for
            // free (branch swaps, `!!` collapses) are unaffected.
            ExpressionKindMut::LogicalExpression(logical_expr)
                if Self::de_morgan_paren_delta(logical_expr).is_some_and(|delta| delta <= 0) =>
            {
                Self::de_morgan_invert_logical(logical_expr);
                let new_expr = e.argument.take_in(ctx);
                ctx.replace_expression(expr, new_expr);
            }
            // "!(a, b)" => "a, !b"
            ExpressionKindMut::SequenceExpression(sequence_expr) => {
                if let Some(last_expr) = sequence_expr.expressions.last_mut() {
                    let new_last =
                        Self::minimize_not(last_expr.span(), last_expr.take_in(ctx), ctx);
                    ctx.replace_expression(last_expr, new_last);
                    let new_expr = e.argument.take_in(ctx);
                    ctx.replace_expression(expr, new_expr);
                }
            }
            _ => {}
        }
    }

    /// Character delta from parentheses added or removed by De Morgan's law
    /// (flipping `&&` <-> `||` changes which nested operands need parens), or
    /// `None` if some operand cannot invert its operator in place.
    fn de_morgan_paren_delta(e: &LogicalExpression<'a>) -> Option<i32> {
        if !matches!(e.operator, LogicalOperator::And | LogicalOperator::Or) {
            return None;
        }
        let mut delta = 0;
        for side in [&e.left, &e.right] {
            match side.kind() {
                ExpressionKind::BinaryExpression(b) if b.operator.is_equality() => {}
                ExpressionKind::LogicalExpression(child) => {
                    delta += Self::de_morgan_paren_delta(child)?;
                    // `&&` under `||` prints bare but its inversion (`||` under
                    // `&&`) needs parens; the reverse drops parens.
                    match (e.operator, child.operator) {
                        (LogicalOperator::Or, LogicalOperator::And) => delta += 2,
                        (LogicalOperator::And, LogicalOperator::Or) => delta -= 2,
                        _ => {}
                    }
                }
                _ => return None,
            }
        }
        Some(delta)
    }

    /// Apply De Morgan's law in place. Only called on chains approved by
    /// [`Self::de_morgan_paren_delta`].
    fn de_morgan_invert_logical(e: &mut LogicalExpression<'a>) {
        e.operator = if e.operator == LogicalOperator::And {
            LogicalOperator::Or
        } else {
            LogicalOperator::And
        };
        Self::de_morgan_invert(&mut e.left);
        Self::de_morgan_invert(&mut e.right);
    }

    fn de_morgan_invert(expr: &mut Expression<'a>) {
        match expr.kind_mut() {
            ExpressionKindMut::BinaryExpression(e) => {
                e.operator = e.operator.equality_inverse_operator().unwrap();
            }
            ExpressionKindMut::LogicalExpression(e) => Self::de_morgan_invert_logical(e),
            _ => unreachable!(),
        }
    }
}
