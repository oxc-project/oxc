use std::borrow::Cow;

use oxc_ast::ast::*;

use crate::ToInt32;

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, ValueType};
use super::value_type::DetermineValueType;

impl<'a> ConstantEvaluation<'a> for UnaryExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.operator {
            UnaryOperator::Typeof => {
                let arg_ty = self.argument.value_type(ctx);
                let s = match arg_ty {
                    ValueType::BigInt => "bigint",
                    ValueType::Number => "number",
                    ValueType::String => "string",
                    ValueType::Boolean => "boolean",
                    ValueType::Undefined => "undefined",
                    ValueType::Null => "object",
                    _ => match &self.argument {
                        Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                            "object"
                        }
                        Expression::ClassExpression(_)
                        | Expression::FunctionExpression(_)
                        | Expression::ArrowFunctionExpression(_) => "function",
                        _ => return None,
                    },
                };
                Some(ConstantValue::String(Cow::Borrowed(s)))
            }
            UnaryOperator::Void => Some(ConstantValue::Undefined),
            UnaryOperator::LogicalNot => {
                self.argument.evaluate_value_to_boolean(ctx).map(|b| !b).map(ConstantValue::Boolean)
            }
            UnaryOperator::UnaryPlus => {
                self.argument.evaluate_value_to_number(ctx).map(ConstantValue::Number)
            }
            UnaryOperator::UnaryNegation => match self.argument.value_type(ctx) {
                ValueType::BigInt => self
                    .argument
                    .evaluate_value_to_bigint(ctx)
                    .map(|v| -v)
                    .map(ConstantValue::BigInt),
                ValueType::Number => self
                    .argument
                    .evaluate_value_to_number(ctx)
                    .map(|v| if v.is_nan() { v } else { -v })
                    .map(ConstantValue::Number),
                ValueType::Undefined => Some(ConstantValue::Number(f64::NAN)),
                ValueType::Null => Some(ConstantValue::Number(-0.0)),
                _ => None,
            },
            UnaryOperator::BitwiseNot => match self.argument.value_type(ctx) {
                ValueType::BigInt => self
                    .argument
                    .evaluate_value_to_bigint(ctx)
                    .map(|v| !v)
                    .map(ConstantValue::BigInt),
                #[expect(clippy::cast_lossless)]
                _ => self
                    .argument
                    .evaluate_value_to_number(ctx)
                    .map(|v| (!v.to_int_32()) as f64)
                    .map(ConstantValue::Number),
            },
            UnaryOperator::Delete => None,
        }
    }
}