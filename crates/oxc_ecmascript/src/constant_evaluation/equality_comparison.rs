use oxc_ast::ast::{Expression, NumberBase};

use super::{ConstantEvaluation, ConstantEvaluationCtx, DetermineValueType, ValueType};

/// <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
pub(super) fn abstract_equality_comparison<'a>(
    ctx: &impl ConstantEvaluationCtx<'a>,
    left_expr: &Expression<'a>,
    right_expr: &Expression<'a>,
) -> Option<bool> {
    let left = left_expr.value_type(ctx);
    let right = right_expr.value_type(ctx);
    if left != ValueType::Undetermined && right != ValueType::Undetermined {
        if left == right {
            return strict_equality_comparison(ctx, left_expr, right_expr);
        }
        if matches!(
            (left, right),
            (ValueType::Null, ValueType::Undefined) | (ValueType::Undefined, ValueType::Null)
        ) {
            return Some(true);
        }

        if matches!((left, right), (ValueType::Number, ValueType::String))
            || matches!(right, ValueType::Boolean)
        {
            if let Some(num) = right_expr.evaluate_value_to_number(ctx) {
                let number_literal_expr = ctx.ast().expression_numeric_literal(
                    oxc_span::SPAN,
                    num,
                    None,
                    if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                );
                return abstract_equality_comparison(ctx, left_expr, &number_literal_expr);
            }
            return None;
        }

        if matches!((left, right), (ValueType::String, ValueType::Number))
            || matches!(left, ValueType::Boolean)
        {
            if let Some(num) = left_expr.evaluate_value_to_number(ctx) {
                let number_literal_expr = ctx.ast().expression_numeric_literal(
                    oxc_span::SPAN,
                    num,
                    None,
                    if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                );
                return abstract_equality_comparison(ctx, &number_literal_expr, right_expr);
            }
            return None;
        }

        if matches!(left, ValueType::BigInt) || matches!(right, ValueType::BigInt) {
            let left_bigint = left_expr.evaluate_value_to_bigint(ctx);
            let right_bigint = right_expr.evaluate_value_to_bigint(ctx);
            if let (Some(l_big), Some(r_big)) = (left_bigint, right_bigint) {
                return Some(l_big.eq(&r_big));
            }
        }

        if matches!(left, ValueType::String | ValueType::Number | ValueType::BigInt)
            && matches!(right, ValueType::Object)
        {
            return None;
        }

        if matches!(left, ValueType::Object)
            && matches!(right, ValueType::String | ValueType::Number | ValueType::BigInt)
        {
            return None;
        }

        return Some(false);
    }
    None
}

/// <https://tc39.es/ecma262/#sec-strict-equality-comparison>
#[expect(clippy::float_cmp)]
pub(super) fn strict_equality_comparison<'a>(
    ctx: &impl ConstantEvaluationCtx<'a>,
    left_expr: &Expression<'a>,
    right_expr: &Expression<'a>,
) -> Option<bool> {
    let left = left_expr.value_type(ctx);
    let right = right_expr.value_type(ctx);
    if !left.is_undetermined() && !right.is_undetermined() {
        // Strict equality can only be true for values of the same type.
        if left != right {
            return Some(false);
        }
        return match left {
            ValueType::Number => {
                let lnum = left_expr.get_side_free_number_value(ctx)?;
                let rnum = right_expr.get_side_free_number_value(ctx)?;
                if lnum.is_nan() || rnum.is_nan() {
                    return Some(false);
                }
                Some(lnum == rnum)
            }
            ValueType::String => {
                let left = left_expr.get_side_free_string_value(ctx)?;
                let right = right_expr.get_side_free_string_value(ctx)?;
                Some(left == right)
            }
            ValueType::Undefined | ValueType::Null => Some(true),
            ValueType::Boolean => {
                let left = left_expr.evaluate_value_to_boolean(ctx)?;
                let right = right_expr.evaluate_value_to_boolean(ctx)?;
                Some(left == right)
            }
            ValueType::BigInt => {
                let left = left_expr.get_side_free_bigint_value(ctx)?;
                let right = right_expr.get_side_free_bigint_value(ctx)?;
                Some(left == right)
            }
            ValueType::Object => None,
            ValueType::Undetermined => unreachable!(),
        };
    }

    // Then, try to evaluate based on the value of the expression.
    // There's only one special case:
    // Any strict equality comparison against NaN returns false.
    if left_expr.is_nan() || right_expr.is_nan() {
        return Some(false);
    }
    None
}
