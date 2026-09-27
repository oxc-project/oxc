use oxc_ast::ast::Expression;
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

        // `(0 | a) OP b` or `(a | 0) OP b` -> `a OP b`
        Self::try_replace_or_number_0(&mut bin_expr.left, ctx);
        // `a OP (0 | b)` or `a OP (b | 0)` -> `a OP b`
        Self::try_replace_or_number_0(&mut bin_expr.right, ctx);

        if bin_expr.operator == BinaryOperator::BitwiseOR {
            if bin_expr.right.is_number_0()
                && matches!(&bin_expr.left, Expression::BinaryExpression(e) if matches!(
                    e.operator,
                    BinaryOperator::ShiftLeft
                        | BinaryOperator::ShiftRight
                        | BinaryOperator::BitwiseOR
                        | BinaryOperator::BitwiseXOR
                        | BinaryOperator::BitwiseAnd
                ))
            {
                // `(a OP b) | 0` -> `a OP b`
                ctx.replace_expression_with(expr, Self::unwrap_left_from_binary_expr);
            } else if bin_expr.left.is_number_0()
                && matches!(&bin_expr.right, Expression::BinaryExpression(e) if matches!(
                    e.operator,
                    BinaryOperator::ShiftLeft
                        | BinaryOperator::ShiftRight
                        | BinaryOperator::BitwiseOR
                        | BinaryOperator::BitwiseXOR
                        | BinaryOperator::BitwiseAnd
                ))
            {
                // `0 | (a OP b)` -> `a OP b`
                ctx.replace_expression_with(expr, Self::unwrap_right_from_binary_expr);
            }
        }
    }

    fn try_replace_or_number_0(bin_expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::BinaryExpression(e) = &bin_expr
            && e.operator == BinaryOperator::BitwiseOR
        {
            if e.left.is_number_0() {
                // `(0 | a)` -> `a`
                ctx.replace_expression_with(bin_expr, Self::unwrap_right_from_binary_expr);
            } else if e.right.is_number_0() {
                // `(a | 0)` -> `a`
                ctx.replace_expression_with(bin_expr, Self::unwrap_left_from_binary_expr);
            }
        }
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
