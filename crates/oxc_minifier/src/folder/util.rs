use std::cmp::Ordering;

use num_bigint::BigInt;

use super::tri::Tri;

use crate::compressor::ast_util::{is_exact_int64, NumberValue};

/// ported from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L1250)
#[allow(clippy::cast_possible_truncation)]
pub fn bigint_less_than_number(
    bigint_value: &BigInt,
    number_value: &NumberValue,
    invert: Tri,
    will_negative: bool,
) -> Tri {
    // if invert is false, then the number is on the right in tryAbstractRelationalComparison
    // if it's true, then the number is on the left
    match number_value {
        NumberValue::NaN => Tri::for_boolean(will_negative),
        NumberValue::PositiveInfinity => Tri::True.xor(invert),
        NumberValue::NegativeInfinity => Tri::False.xor(invert),
        NumberValue::Number(num) => {
            if let Some(Ordering::Equal | Ordering::Greater) =
                num.abs().partial_cmp(&2_f64.powi(53))
            {
                Tri::Unknown
            } else {
                let number_as_bigint = BigInt::from(*num as i64);

                match bigint_value.cmp(&number_as_bigint) {
                    Ordering::Less => Tri::True.xor(invert),
                    Ordering::Greater => Tri::False.xor(invert),
                    Ordering::Equal => {
                        if is_exact_int64(*num) {
                            Tri::False
                        } else {
                            Tri::for_boolean(num.is_sign_positive()).xor(invert)
                        }
                    }
                }
            }
        }
    }
}
