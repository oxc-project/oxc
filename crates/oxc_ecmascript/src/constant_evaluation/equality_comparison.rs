use oxc_ast::ast::{Expression, NumberBase};

use super::{ConstantEvaluation, DetermineValueType, ValueType};

/// <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
pub(super) fn abstract_equality_comparison<'a>(
    c: &impl ConstantEvaluation<'a>,
    left_expr: &Expression<'a>,
    right_expr: &Expression<'a>,
) -> Option<bool> {
    let left = left_expr.value_type(c);
    let right = right_expr.value_type(c);
    if left != ValueType::Undetermined && right != ValueType::Undetermined {
        if left == right {
            return strict_equality_comparison(c, left_expr, right_expr);
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
            if let Some(num) = c.get_side_free_number_value(right_expr) {
                let number_literal_expr = c.ast().expression_numeric_literal(
                    oxc_span::SPAN,
                    num,
                    None,
                    if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                );
                return abstract_equality_comparison(c, left_expr, &number_literal_expr);
            }
            return None;
        }

        if matches!((left, right), (ValueType::String, ValueType::Number))
            || matches!(left, ValueType::Boolean)
        {
            if let Some(num) = c.get_side_free_number_value(left_expr) {
                let number_literal_expr = c.ast().expression_numeric_literal(
                    oxc_span::SPAN,
                    num,
                    None,
                    if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                );
                return abstract_equality_comparison(c, &number_literal_expr, right_expr);
            }
            return None;
        }

        if matches!(left, ValueType::BigInt) || matches!(right, ValueType::BigInt) {
            let left_bigint = c.get_side_free_bigint_value(left_expr);
            let right_bigint = c.get_side_free_bigint_value(right_expr);
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
    c: &impl ConstantEvaluation<'a>,
    left_expr: &Expression<'a>,
    right_expr: &Expression<'a>,
) -> Option<bool> {
    let left = left_expr.value_type(c);
    let right = right_expr.value_type(c);
    if !left.is_undetermined() && !right.is_undetermined() {
        // Strict equality can only be true for values of the same type.
        if left != right {
            return Some(false);
        }
        return match left {
            ValueType::Number => {
                let lnum = c.get_side_free_number_value(left_expr)?;
                let rnum = c.get_side_free_number_value(right_expr)?;
                if lnum.is_nan() || rnum.is_nan() {
                    return Some(false);
                }
                Some(lnum == rnum)
            }
            ValueType::String => {
                let left = c.get_side_free_string_value(left_expr)?;
                let right = c.get_side_free_string_value(right_expr)?;
                Some(left == right)
            }
            ValueType::Undefined | ValueType::Null => Some(true),
            ValueType::Boolean => {
                let left = c.get_boolean_value(left_expr)?;
                let right = c.get_boolean_value(right_expr)?;
                Some(left == right)
            }
            ValueType::BigInt => {
                let left = c.get_side_free_bigint_value(left_expr)?;
                let right = c.get_side_free_bigint_value(right_expr)?;
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
