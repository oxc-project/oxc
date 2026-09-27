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

        if let Expression::BinaryExpression(e) = &bin_expr.left
            && e.operator == BinaryOperator::BitwiseOR
        {
            if e.left.is_number_0() {
                // `(0 | a) OP b` -> `a OP b`
                ctx.replace_expression_with(
                    &mut bin_expr.left,
                    Self::unwrap_right_from_binary_expr,
                );
            } else if e.right.is_number_0() {
                // `(a | 0) OP b` -> `a OP b`
                ctx.replace_expression_with(&mut bin_expr.left, Self::unwrap_left_from_binary_expr);
            }
        } else if let Expression::BinaryExpression(e) = &bin_expr.right
            && e.operator == BinaryOperator::BitwiseOR
        {
            if e.left.is_number_0() {
                // `a OP (0 | b)` -> `a OP b`
                ctx.replace_expression_with(
                    &mut bin_expr.right,
                    Self::unwrap_right_from_binary_expr,
                );
            } else if e.right.is_number_0() {
                // `a OP (b | 0)` -> `a OP b`
                ctx.replace_expression_with(
                    &mut bin_expr.right,
                    Self::unwrap_left_from_binary_expr,
                );
            }
        }

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

    pub fn unwrap_left_from_binary_expr(
        e: Expression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Expression::BinaryExpression(e) = e else { unreachable!() };
        e.unbox().left
    }

    pub fn unwrap_right_from_binary_expr(
        e: Expression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Expression::BinaryExpression(e) = e else { unreachable!() };
        e.unbox().right
    }
}
