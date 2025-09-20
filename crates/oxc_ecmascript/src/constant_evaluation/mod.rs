mod binary_expressions;
mod call_expr;
mod call_expressions;
mod equality_comparison;
mod identifiers;
mod is_int32_or_uint32;
mod is_literal_value;
mod logical_expressions;
mod member_expressions;
mod traits;
mod unary_expressions;
mod url_encoding;
mod value;
mod value_type;

pub use is_int32_or_uint32::IsInt32OrUint32;
pub use is_literal_value::IsLiteralValue;
pub use traits::{ConstantEvaluation, ConstantEvaluationCtx};
pub use value::ConstantValue;
pub use value_type::{DetermineValueType, ValueType};

use std::borrow::Cow;

use oxc_ast::ast::*;

use crate::{ToBigInt, ToBoolean, ToJsString as ToJsStringTrait, ToNumber};

impl<'a> ConstantEvaluation<'a> for Expression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        let result = match target_ty {
            Some(ValueType::Boolean) => self.to_boolean(ctx).map(ConstantValue::Boolean),
            Some(ValueType::Number) => self.to_number(ctx).map(ConstantValue::Number),
            Some(ValueType::BigInt) => self.to_big_int(ctx).map(ConstantValue::BigInt),
            Some(ValueType::String) => self.to_js_string(ctx).map(ConstantValue::String),
            _ => None,
        };
        if result.is_some() {
            return result;
        }

        match self {
            Expression::BinaryExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::LogicalExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::UnaryExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::Identifier(ident) => ident.evaluate_value_to(ctx, target_ty),
            Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
            Expression::NullLiteral(_) => Some(ConstantValue::Null),
            Expression::BooleanLiteral(lit) => Some(ConstantValue::Boolean(lit.value)),
            Expression::BigIntLiteral(lit) => lit.to_big_int(ctx).map(ConstantValue::BigInt),
            Expression::StringLiteral(lit) => {
                Some(ConstantValue::String(Cow::Borrowed(lit.value.as_str())))
            }
            Expression::StaticMemberExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::ComputedMemberExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::CallExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::SequenceExpression(e) => {
                // For sequence expression, the value is the value of the RHS.
                e.expressions.last().and_then(|e| e.evaluate_value_to(ctx, target_ty))
            }
            _ => None,
        }
    }
}

// Re-export the public API for backwards compatibility
pub use binary_expressions::binary_operation_evaluate_value;
