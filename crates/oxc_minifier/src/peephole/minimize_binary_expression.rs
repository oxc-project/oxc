use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::IsInt32OrUint32;
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
