use num_traits::Zero;
use oxc_ast::ast::*;

use crate::{
    ToInt32, ToUint32,
    is_less_than::is_less_than,
    to_numeric::ToNumeric,
    to_primitive::ToPrimitive,
};

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, ValueType};
use super::value_type::DetermineValueType;
use super::equality_comparison::{abstract_equality_comparison, strict_equality_comparison};

impl<'a> ConstantEvaluation<'a> for BinaryExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        // FIXME: skipped for now to avoid performance regression, can be removed
        if target_ty == Some(ValueType::Boolean) {
            return None;
        }

        binary_operation_evaluate_value_to(self.operator, &self.left, &self.right, ctx, target_ty)
    }
}

pub fn binary_operation_evaluate_value<'a, Ctx: ConstantEvaluationCtx<'a>>(
    operator: BinaryOperator,
    left: &Expression<'a>,
    right: &Expression<'a>,
    ctx: &Ctx,
) -> Option<ConstantValue<'a>> {
    binary_operation_evaluate_value_to(operator, left, right, ctx, None)
}

fn binary_operation_evaluate_value_to<'a>(
    operator: BinaryOperator,
    left: &Expression<'a>,
    right: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
    _target_ty: Option<ValueType>,
) -> Option<ConstantValue<'a>> {
    match operator {
        BinaryOperator::Addition => {
            let left_to_primitive = left.to_primitive(ctx);
            let right_to_primitive = right.to_primitive(ctx);
            if left_to_primitive.is_string() == Some(true)
                || right_to_primitive.is_string() == Some(true)
            {
                let lval = left.evaluate_value_to_string(ctx)?;
                let rval = right.evaluate_value_to_string(ctx)?;
                return Some(ConstantValue::String(lval + rval));
            }
            let left_to_numeric_type = left_to_primitive.to_numeric(ctx);
            let right_to_numeric_type = right_to_primitive.to_numeric(ctx);
            if left_to_numeric_type.is_number() || right_to_numeric_type.is_number() {
                let lval = left.evaluate_value_to_number(ctx)?;
                let rval = right.evaluate_value_to_number(ctx)?;
                return Some(ConstantValue::Number(lval + rval));
            }
            if left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint() {
                let lval = left.evaluate_value_to_bigint(ctx)?;
                let rval = right.evaluate_value_to_bigint(ctx)?;
                return Some(ConstantValue::BigInt(lval + rval));
            }
            None
        }
        BinaryOperator::Subtraction => {
            let lval = left.evaluate_value_to_number(ctx)?;
            let rval = right.evaluate_value_to_number(ctx)?;
            Some(ConstantValue::Number(lval - rval))
        }
        BinaryOperator::Division => {
            let lval = left.evaluate_value_to_number(ctx)?;
            let rval = right.evaluate_value_to_number(ctx)?;
            Some(ConstantValue::Number(lval / rval))
        }
        BinaryOperator::Remainder => {
            let lval = left.evaluate_value_to_number(ctx)?;
            let rval = right.evaluate_value_to_number(ctx)?;
            Some(ConstantValue::Number(if rval.is_zero() { f64::NAN } else { lval % rval }))
        }
        BinaryOperator::Multiplication => {
            let lval = left.evaluate_value_to_number(ctx)?;
            let rval = right.evaluate_value_to_number(ctx)?;
            Some(ConstantValue::Number(lval * rval))
        }
        BinaryOperator::Exponential => {
            let lval = left.evaluate_value_to_number(ctx)?;
            let rval = right.evaluate_value_to_number(ctx)?;
            let result = lval.powf(rval);
            // For now, ignore the result if it large or has a decimal part
            // so that the output does not become bigger than the input.
            if result.is_finite() && (result.fract() != 0.0 || result.log10() > 4.0) {
                return None;
            }
            Some(ConstantValue::Number(result))
        }
        BinaryOperator::ShiftLeft => {
            let left = left.evaluate_value_to_number(ctx)?;
            let right = right.evaluate_value_to_number(ctx)?;
            let left = left.to_int_32();
            let right = right.to_uint_32() & 31;
            Some(ConstantValue::Number(f64::from(left << right)))
        }
        BinaryOperator::ShiftRight => {
            let left = left.evaluate_value_to_number(ctx)?;
            let right = right.evaluate_value_to_number(ctx)?;
            let left = left.to_int_32();
            let right = right.to_uint_32() & 31;
            Some(ConstantValue::Number(f64::from(left >> right)))
        }
        BinaryOperator::ShiftRightZeroFill => {
            let left = left.evaluate_value_to_number(ctx)?;
            let right = right.evaluate_value_to_number(ctx)?;
            let left = left.to_uint_32();
            let right = right.to_uint_32() & 31;
            Some(ConstantValue::Number(f64::from(left >> right)))
        }
        BinaryOperator::LessThan => is_less_than(ctx, left, right).map(|value| match value {
            ConstantValue::Undefined => ConstantValue::Boolean(false),
            _ => value,
        }),
        BinaryOperator::GreaterThan => is_less_than(ctx, right, left).map(|value| match value {
            ConstantValue::Undefined => ConstantValue::Boolean(false),
            _ => value,
        }),
        BinaryOperator::LessEqualThan => is_less_than(ctx, right, left).map(|value| match value {
            ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                ConstantValue::Boolean(false)
            }
            ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
            _ => unreachable!(),
        }),
        BinaryOperator::GreaterEqualThan => {
            is_less_than(ctx, left, right).map(|value| match value {
                ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                    ConstantValue::Boolean(false)
                }
                ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
                _ => unreachable!(),
            })
        }
        BinaryOperator::BitwiseAnd => {
            if left.value_type(ctx).is_bigint() && right.value_type(ctx).is_bigint() {
                let left_biginit = left.evaluate_value_to_bigint(ctx)?;
                let right_bigint = right.evaluate_value_to_bigint(ctx)?;
                return Some(ConstantValue::BigInt(left_biginit & right_bigint));
            }
            let left_int = left.evaluate_value_to_number(ctx)?.to_int_32();
            let right_int = right.evaluate_value_to_number(ctx)?.to_int_32();
            Some(ConstantValue::Number(f64::from(left_int & right_int)))
        }
        BinaryOperator::BitwiseOR => {
            if left.value_type(ctx).is_bigint() && right.value_type(ctx).is_bigint() {
                let left_biginit = left.evaluate_value_to_bigint(ctx)?;
                let right_bigint = right.evaluate_value_to_bigint(ctx)?;
                return Some(ConstantValue::BigInt(left_biginit | right_bigint));
            }
            let left_int = left.evaluate_value_to_number(ctx)?.to_int_32();
            let right_int = right.evaluate_value_to_number(ctx)?.to_int_32();
            Some(ConstantValue::Number(f64::from(left_int | right_int)))
        }
        BinaryOperator::BitwiseXOR => {
            if left.value_type(ctx).is_bigint() && right.value_type(ctx).is_bigint() {
                let left_biginit = left.evaluate_value_to_bigint(ctx)?;
                let right_bigint = right.evaluate_value_to_bigint(ctx)?;
                return Some(ConstantValue::BigInt(left_biginit ^ right_bigint));
            }
            let left_int = left.evaluate_value_to_number(ctx)?.to_int_32();
            let right_int = right.evaluate_value_to_number(ctx)?.to_int_32();
            Some(ConstantValue::Number(f64::from(left_int ^ right_int)))
        }
        BinaryOperator::Instanceof => {
            if let Expression::Identifier(right_ident) = right {
                let name = right_ident.name.as_str();
                if matches!(name, "Object" | "Number" | "Boolean" | "String")
                    && ctx.is_global_reference(right_ident)
                {
                    let left_ty = left.value_type(ctx);
                    if left_ty.is_undetermined() {
                        return None;
                    }
                    return Some(ConstantValue::Boolean(
                        name == "Object" && left.value_type(ctx).is_object(),
                    ));
                }
            }
            None
        }
        BinaryOperator::StrictEquality => {
            let value = strict_equality_comparison(ctx, left, right)?;
            Some(ConstantValue::Boolean(value))
        }
        BinaryOperator::StrictInequality => {
            let value = strict_equality_comparison(ctx, left, right)?;
            Some(ConstantValue::Boolean(!value))
        }
        BinaryOperator::Equality => {
            let value = abstract_equality_comparison(ctx, left, right)?;
            Some(ConstantValue::Boolean(value))
        }
        BinaryOperator::Inequality => {
            let value = abstract_equality_comparison(ctx, left, right)?;
            Some(ConstantValue::Boolean(!value))
        }
        BinaryOperator::In => None,
    }
}