use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, IsInt32OrUint32},
    side_effects::MayHaveSideEffects,
};
use oxc_syntax::operator::BinaryOperator;

use crate::TraverseCtx;
use crate::peephole::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_bitwise_binary_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::BinaryExpression(bin_expr) = expr else { return };
        if !bin_expr.operator.is_bitwise() {
            return;
        }

        match bin_expr.operator {
            BinaryOperator::BitwiseAnd => {
                // u32::MAX -> 2^32-1 -> 4_294_967_295
                if bin_expr.right.is_number_value(4_294_967_295.0)
                    && let Expression::NumericLiteral(n) = &mut bin_expr.right
                {
                    // `a & 0xffffffff` -> `a | 0`
                    ctx.notice_change();
                    n.value = 0.0;
                    n.raw = None;
                    bin_expr.operator = BinaryOperator::BitwiseOR;
                } else if bin_expr.left.is_number_value(4_294_967_295.0)
                    && let Expression::NumericLiteral(n) = &mut bin_expr.left
                {
                    // `0xffffffff & a` -> `0 | a`
                    ctx.notice_change();
                    n.value = 0.0;
                    n.raw = None;
                    bin_expr.operator = BinaryOperator::BitwiseOR;
                }
            }
            BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight | BinaryOperator::BitwiseXOR
                if bin_expr.right.is_number_0() =>
            {
                // `a << 0` -> `a | 0`
                // `a >> 0` -> `a | 0`
                // `a ^ 0` -> `a | 0`
                ctx.notice_change();
                bin_expr.operator = BinaryOperator::BitwiseOR;
            }
            _ => {}
        }

        Self::remove_redundant_bitwise_or_zero(bin_expr, ctx);

        if bin_expr.operator == BinaryOperator::BitwiseOR {
            if bin_expr.right.is_number_0() && Self::is_bitwise_int(&bin_expr.left, ctx) {
                // `(a OP b) | 0` -> `a OP b`
                ctx.replace_expression_with(expr, Self::unwrap_left_from_binary_expr);
            } else if bin_expr.left.is_number_0() && Self::is_bitwise_int(&bin_expr.right, ctx) {
                // `0 | (a OP b)` -> `a OP b`
                ctx.replace_expression_with(expr, Self::unwrap_right_from_binary_expr);
            }
        }
    }

    /// Removes `| 0` from an operand of another bitwise operator.
    ///
    /// - `(a | 0) OP b` -> `a OP b`
    /// - `b OP (a | 0)` -> `b OP a`
    ///
    /// Bitwise operators reduce their number operands modulo 2^32, so applying
    /// `ToInt32` first is redundant. `b` must be a number because, for a BigInt `a`,
    /// `a | 0` throws but `a OP b` succeeds when `b` is also a BigInt.
    ///
    /// In `(a | 0) OP b`, `b` is evaluated after `a` has been converted. Removing
    /// `| 0` moves that conversion after `b`, so `b` must not have side effects.
    fn remove_redundant_bitwise_or_zero(e: &mut BinaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if Self::is_bitwise_or_zero(&e.left)
            && e.right.value_type(ctx).is_number()
            && !e.right.may_have_side_effects(ctx)
        {
            ctx.replace_expression_with(&mut e.left, Self::unwrap_bitwise_or_zero);
        }
        if Self::is_bitwise_or_zero(&e.right) && e.left.value_type(ctx).is_number() {
            ctx.replace_expression_with(&mut e.right, Self::unwrap_bitwise_or_zero);
        }
    }

    fn is_bitwise_or_zero(expr: &Expression<'a>) -> bool {
        matches!(
            expr,
            Expression::BinaryExpression(e)
                if e.operator == BinaryOperator::BitwiseOR
                    && (e.left.is_number_0() || e.right.is_number_0())
        )
    }

    fn unwrap_bitwise_or_zero(e: Expression<'a>, _ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let Expression::BinaryExpression(e) = e else { unreachable!() };
        let e = e.unbox();
        if e.right.is_number_0() { e.left } else { e.right }
    }

    fn is_bitwise_int(expr: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        let Expression::BinaryExpression(e) = expr else { return false };
        e.operator != BinaryOperator::ShiftRightZeroFill && e.is_int32_or_uint32(ctx)
    }

    fn unwrap_left_from_binary_expr(
        e: Expression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Expression::BinaryExpression(e) = e else { unreachable!() };
        e.unbox().left
    }

    fn unwrap_right_from_binary_expr(
        e: Expression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Expression::BinaryExpression(e) = e else { unreachable!() };
        e.unbox().right
    }
}
