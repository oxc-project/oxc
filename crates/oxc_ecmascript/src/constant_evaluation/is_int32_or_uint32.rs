use num_traits::ToPrimitive;
use oxc_ast::ast::*;

use crate::GlobalContext;

use super::DetermineValueType;

pub trait IsInt32OrUint32<'a> {
    /// Whether the value of the expression is a int32 or uint32.
    /// If this method returns `true`, we know that the value cannot be NaN or Infinity.
    ///
    /// - true means it is int32 or uint32.
    /// - false means it is neither int32 nor uint32, or it is unknown.
    ///
    /// Based on <https://github.com/evanw/esbuild/blob/v0.25.0/internal/js_ast/js_ast_helpers.go#L950>
    fn is_int32_or_uint32(&self, ctx: &impl GlobalContext<'a>) -> bool;
}

impl<'a> IsInt32OrUint32<'a> for Expression<'a> {
    fn is_int32_or_uint32(&self, ctx: &impl GlobalContext<'a>) -> bool {
        match self {
            Expression::NumericLiteral(n) => n.is_int32_or_uint32(ctx),
            Expression::UnaryExpression(e) => e.is_int32_or_uint32(ctx),
            Expression::BinaryExpression(e) => e.is_int32_or_uint32(ctx),
            Expression::LogicalExpression(e) => e.is_int32_or_uint32(ctx),
            Expression::ConditionalExpression(e) => {
                e.consequent.is_int32_or_uint32(ctx) && e.alternate.is_int32_or_uint32(ctx)
            }
            Expression::SequenceExpression(e) => {
                e.expressions.last().is_some_and(|e| e.is_int32_or_uint32(ctx))
            }
            Expression::ParenthesizedExpression(e) => e.expression.is_int32_or_uint32(ctx),
            _ => false,
        }
    }
}

impl<'a> IsInt32OrUint32<'a> for NumericLiteral<'a> {
    fn is_int32_or_uint32(&self, _ctx: &impl GlobalContext<'a>) -> bool {
        self.value.fract() == 0.0
            && (self.value.to_i32().is_some() || self.value.to_u32().is_some())
    }
}

impl<'a> IsInt32OrUint32<'a> for UnaryExpression<'a> {
    fn is_int32_or_uint32(&self, ctx: &impl GlobalContext<'a>) -> bool {
        match self.operator {
            UnaryOperator::BitwiseNot => self.value_type(ctx).is_number(),
            UnaryOperator::UnaryPlus => self.argument.is_int32_or_uint32(ctx),
            _ => false,
        }
    }
}

impl<'a> IsInt32OrUint32<'a> for BinaryExpression<'a> {
    fn is_int32_or_uint32(&self, ctx: &impl GlobalContext<'a>) -> bool {
        match self.operator {
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR => self.value_type(ctx).is_number(),
            BinaryOperator::ShiftRightZeroFill => true,
            _ => false,
        }
    }
}

impl<'a> IsInt32OrUint32<'a> for LogicalExpression<'a> {
    fn is_int32_or_uint32(&self, ctx: &impl GlobalContext<'a>) -> bool {
        match self.operator {
            LogicalOperator::And | LogicalOperator::Or => {
                self.left.is_int32_or_uint32(ctx) && self.right.is_int32_or_uint32(ctx)
            }
            LogicalOperator::Coalesce => self.left.is_int32_or_uint32(ctx),
        }
    }
}
