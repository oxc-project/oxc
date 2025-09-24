use std::cmp::Ordering;

use num_bigint::BigInt;
use num_traits::FromPrimitive;
use oxc_ast::ast::Expression;

use crate::{
    ToBigInt, ToJsString,
    constant_evaluation::{
        ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, DetermineValueType,
    },
};

pub fn is_less_than<'a>(
    ctx: &impl ConstantEvaluationCtx<'a>,
    x: &Expression<'a>,
    y: &Expression<'a>,
) -> Option<ConstantValue<'a>> {
    // a. Let px be ? ToPrimitive(x, NUMBER).
    // b. Let py be ? ToPrimitive(y, NUMBER).
    let px = x.value_type(ctx);
    let py = y.value_type(ctx);

    // If the operands are not primitives, `ToPrimitive` is *not* a noop.
    if px.is_undetermined() || px.is_object() || py.is_undetermined() || py.is_object() {
        return None;
    }

    // 3. If px is a String and py is a String, then
    if px.is_string() && py.is_string() {
        let left_string = x.to_js_string(ctx)?;
        let right_string = y.to_js_string(ctx)?;
        return Some(ConstantValue::Boolean(
            left_string.encode_utf16().cmp(right_string.encode_utf16()) == Ordering::Less,
        ));
    }

    // a. If px is a BigInt and py is a String, then
    if px.is_bigint() && py.is_string() {
        use crate::StringToBigInt;
        let ny = y.to_js_string(ctx)?.as_ref().string_to_big_int();
        let Some(ny) = ny else { return Some(ConstantValue::Undefined) };
        return Some(ConstantValue::Boolean(x.to_big_int(ctx)? < ny));
    }
    // b. If px is a String and py is a BigInt, then
    if px.is_string() && py.is_bigint() {
        use crate::StringToBigInt;
        let nx = x.to_js_string(ctx)?.as_ref().string_to_big_int();
        let Some(nx) = nx else { return Some(ConstantValue::Undefined) };
        return Some(ConstantValue::Boolean(nx < y.to_big_int(ctx)?));
    }

    // Both operands are primitives here.
    // ToNumeric returns a BigInt if the operand is a BigInt. Otherwise, it returns a Number.
    let nx_is_number = !px.is_bigint();
    let ny_is_number = !py.is_bigint();

    // f. If SameType(nx, ny) is true, then
    //   i. If nx is a Number, then
    if nx_is_number && ny_is_number {
        let left_num = x.evaluate_value_to_number(ctx)?;
        if left_num.is_nan() {
            return Some(ConstantValue::Undefined);
        }
        let right_num = y.evaluate_value_to_number(ctx)?;
        if right_num.is_nan() {
            return Some(ConstantValue::Undefined);
        }
        return Some(ConstantValue::Boolean(left_num < right_num));
    }
    //   ii. Else,
    if px.is_bigint() && py.is_bigint() {
        return Some(ConstantValue::Boolean(x.to_big_int(ctx)? < y.to_big_int(ctx)?));
    }

    let nx = x.evaluate_value_to_number(ctx);
    let ny = y.evaluate_value_to_number(ctx);

    // h. If nx or ny is NaN, return undefined.
    if nx_is_number && nx.is_some_and(f64::is_nan) || ny_is_number && ny.is_some_and(f64::is_nan) {
        return Some(ConstantValue::Undefined);
    }

    // i. If nx is -∞𝔽 or ny is +∞𝔽, return true.
    if nx_is_number && nx.is_some_and(|n| n == f64::NEG_INFINITY)
        || ny_is_number && ny.is_some_and(|n| n == f64::INFINITY)
    {
        return Some(ConstantValue::Boolean(true));
    }
    // j. If nx is +∞𝔽 or ny is -∞𝔽, return false.
    if nx_is_number && nx.is_some_and(|n| n == f64::INFINITY)
        || ny_is_number && ny.is_some_and(|n| n == f64::NEG_INFINITY)
    {
        return Some(ConstantValue::Boolean(false));
    }

    // k. If ℝ(nx) < ℝ(ny), return true; otherwise return false.
    if px.is_bigint() {
        let nx = x.to_big_int(ctx)?;
        let ny = y.evaluate_value_to_number(ctx)?;
        return compare_bigint_and_f64(&nx, ny)
            .map(|ord| ConstantValue::Boolean(ord == Ordering::Less));
    }
    if py.is_bigint() {
        let ny = y.to_big_int(ctx)?;
        let nx = x.evaluate_value_to_number(ctx)?;
        return compare_bigint_and_f64(&ny, nx)
            .map(|ord| ConstantValue::Boolean(ord.reverse() == Ordering::Less));
    }

    None
}

fn compare_bigint_and_f64(x: &BigInt, y: f64) -> Option<Ordering> {
    let ny = BigInt::from_f64(y)?;

    let raw_ord = x.cmp(&ny);
    if raw_ord == Ordering::Equal {
        let fract_ord = 0.0f64.partial_cmp(&y.fract()).expect("both should be finite");
        Some(fract_ord)
    } else {
        Some(raw_ord)
    }
}

#[cfg(test)]
mod test {
    use super::compare_bigint_and_f64;
    use num_bigint::BigInt;
    use std::cmp::Ordering;

    #[test]
    fn test_compare_bigint_and_f64() {
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::NAN), None);
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::INFINITY), None);
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::NEG_INFINITY), None);

        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 0.0), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(0), 0.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), 0.0), Some(Ordering::Less));

        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 0.9), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 1.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 1.1), Some(Ordering::Less));

        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -1.1), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -1.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -0.9), Some(Ordering::Less));
    }
}
