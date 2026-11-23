use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{DetermineValueType, IsInt32OrUint32};
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
    /// `ToInt32` first is redundant. `b` must be a number because `a | 0` throws for
    /// a BigInt `a`, while every operator except `>>>` accepts two BigInts.
    ///
    /// In `(a | 0) OP b`, `a | 0` can throw before `b` is evaluated. Removing
    /// `| 0` delays that conversion until after `b`, so require `b` to be a
    /// numeric literal: evaluating it cannot throw or run user code.
    fn remove_redundant_bitwise_or_zero(e: &mut BinaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if e.right.is_number()
            && let Some(unwrap) = Self::bitwise_or_zero_unwrapper(&e.left)
        {
            ctx.replace_expression_with(&mut e.left, unwrap);
        }
        if let Some(unwrap) = Self::bitwise_or_zero_unwrapper(&e.right)
            && e.left.value_type(ctx).is_number()
        {
            ctx.replace_expression_with(&mut e.right, unwrap);
        }
    }

    fn bitwise_or_zero_unwrapper(
        expr: &Expression<'a>,
    ) -> Option<fn(Expression<'a>, &mut TraverseCtx<'a>) -> Expression<'a>> {
        let Expression::BinaryExpression(e) = expr else { return None };
        if e.operator != BinaryOperator::BitwiseOR {
            None
        } else if e.right.is_number_0() {
            Some(Self::unwrap_left_from_binary_expr)
        } else if e.left.is_number_0() {
            Some(Self::unwrap_right_from_binary_expr)
        } else {
            None
        }
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
