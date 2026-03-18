use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, IsInt32OrUint32};
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Simplify syntax when we know it's used inside a boolean context, e.g. `if (boolean_context) {}`.
    ///
    /// `SimplifyBooleanExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2059>
    pub fn minimize_expression_in_boolean_context(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match expr.kind_mut() {
            // "!!a" => "a"
            ExpressionKindMut::UnaryExpression(u1) if u1.operator.is_not() => {
                if let Some(u2) = u1.argument.as_unary_expression_mut()
                    && u2.operator.is_not()
                {
                    let mut e = u2.argument.take_in(ctx.ast);
                    Self::minimize_expression_in_boolean_context(&mut e, ctx);
                    *expr = e;
                    ctx.state.changed = true;
                }
            }
            Expression::binary_expression(e)
                if e.operator.is_equality()
                    && e.right.as_numeric_literal().is_some_and(|lit| lit.value == 0.0)
                    && e.left.is_int32_or_uint32(ctx) =>
            {
                let argument = e.left.take_in(ctx.ast);
                *expr = if matches!(
                    e.operator,
                    BinaryOperator::StrictInequality | BinaryOperator::Inequality
                ) {
                    // `if ((a | b) !== 0)` -> `if (a | b);`
                    argument
                } else {
                    // `if ((a | b) === 0);", "if (!(a | b));")`
                    ctx.ast.expression_unary(e.span, UnaryOperator::LogicalNot, argument)
                };
                ctx.state.changed = true;
            }
            // "if (!!a && !!b)" => "if (a && b)"
            ExpressionKindMut::LogicalExpression(e) if e.operator.is_and() => {
                Self::minimize_expression_in_boolean_context(&mut e.left, ctx);
                Self::minimize_expression_in_boolean_context(&mut e.right, ctx);
                // "if (anything && truthyNoSideEffects)" => "if (anything)"
                if e.right.get_side_free_boolean_value(ctx) == Some(true) {
                    *expr = e.left.take_in(ctx.ast);
                    ctx.state.changed = true;
                }
            }
            // "if (!!a ||!!b)" => "if (a || b)"
            ExpressionKindMut::LogicalExpression(e) if e.operator == LogicalOperator::Or => {
                Self::minimize_expression_in_boolean_context(&mut e.left, ctx);
                Self::minimize_expression_in_boolean_context(&mut e.right, ctx);
                // "if (anything || falsyNoSideEffects)" => "if (anything)"
                if e.right.get_side_free_boolean_value(ctx) == Some(false) {
                    *expr = e.left.take_in(ctx.ast);
                    ctx.state.changed = true;
                }
            }
            ExpressionKindMut::ConditionalExpression(e) => {
                // "if (a ? !!b : !!c)" => "if (a ? b : c)"
                Self::minimize_expression_in_boolean_context(&mut e.consequent, ctx);
                Self::minimize_expression_in_boolean_context(&mut e.alternate, ctx);
                if let Some(boolean) = e.consequent.get_side_free_boolean_value(ctx) {
                    let right = e.alternate.take_in(ctx.ast);
                    let left = e.test.take_in(ctx.ast);
                    let span = e.span;
                    let (op, left) = if boolean {
                        // "if (anything1 ? truthyNoSideEffects : anything2)" => "if (anything1 || anything2)"
                        (LogicalOperator::Or, left)
                    } else {
                        // "if (anything1 ? falsyNoSideEffects : anything2)" => "if (!anything1 && anything2)"
                        (LogicalOperator::And, Self::minimize_not(left.span(), left, ctx))
                    };
                    *expr = Self::join_with_left_associative_op(span, op, left, right, ctx);
                    ctx.state.changed = true;
                    return;
                }
                if let Some(boolean) = e.alternate.get_side_free_boolean_value(ctx) {
                    let left = e.test.take_in(ctx.ast);
                    let right = e.consequent.take_in(ctx.ast);
                    let span = e.span;
                    let (op, left) = if boolean {
                        // "if (anything1 ? anything2 : truthyNoSideEffects)" => "if (!anything1 || anything2)"
                        (LogicalOperator::Or, Self::minimize_not(left.span(), left, ctx))
                    } else {
                        // "if (anything1 ? anything2 : falsyNoSideEffects)" => "if (anything1 && anything2)"
                        (LogicalOperator::And, left)
                    };
                    *expr = Self::join_with_left_associative_op(span, op, left, right, ctx);
                    ctx.state.changed = true;
                }
            }
            ExpressionKindMut::SequenceExpression(seq_expr) => {
                if let Some(last) = seq_expr.expressions.last_mut() {
                    Self::minimize_expression_in_boolean_context(last, ctx);
                }
            }
            _ => {}
        }
    }
}
