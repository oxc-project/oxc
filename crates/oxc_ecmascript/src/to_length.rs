//! 7.1.17 ToLength ( argument )
//! <https://tc39.es/ecma262/#sec-tolength>

use crate::to_integer_or_infinity::ToIntegerOrInfinity;

const MAX_SAFE_INTEGER: f64 = 9007199254740991.0; // 2^53 - 1

/// `ToLength` converts argument to a non-negative integer, suitable for use as the length of an array-like object.
pub trait ToLength {
    fn to_length(&self) -> f64;
}

impl ToLength for f64 {
    fn to_length(&self) -> f64 {
        // 1. Let len be ? ToIntegerOrInfinity(argument).
        let len = self.to_integer_or_infinity();

        // 2. If len ≤ 0, return +0𝔽.
        if len <= 0.0 {
            return 0.0;
        }

        // 3. Return 𝔽(min(len, 2^53 - 1)).
        len.min(MAX_SAFE_INTEGER)
    }
}

impl ToLength for i32 {
    fn to_length(&self) -> f64 {
        (*self as f64).to_length()
    }
}

impl ToLength for u32 {
    fn to_length(&self) -> f64 {
        (*self as f64).to_length()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_length() {
        // Basic cases
        assert_eq!(0.0.to_length(), 0.0);
        assert_eq!(1.0.to_length(), 1.0);
        assert_eq!(5.0.to_length(), 5.0);
        assert_eq!(100.0.to_length(), 100.0);
        assert_eq!(MAX_SAFE_INTEGER.to_length(), MAX_SAFE_INTEGER);

        // Negative numbers should return 0
        assert_eq!((-1.0).to_length(), 0.0);
        assert_eq!((-100.0).to_length(), 0.0);
        assert_eq!(f64::NEG_INFINITY.to_length(), 0.0);

        // Values greater than MAX_SAFE_INTEGER should be clamped
        assert_eq!((MAX_SAFE_INTEGER + 1.0).to_length(), MAX_SAFE_INTEGER);
        assert_eq!((MAX_SAFE_INTEGER * 2.0).to_length(), MAX_SAFE_INTEGER);
        assert_eq!(f64::INFINITY.to_length(), MAX_SAFE_INTEGER);

        // Edge cases
        assert_eq!(f64::NAN.to_length(), 0.0);
        assert_eq!(0.0.to_length(), 0.0);
        assert_eq!((-0.0).to_length(), 0.0);

        // Fractional values (should be truncated towards zero)
        assert_eq!(1.7.to_length(), 1.0);
        assert_eq!(1.2.to_length(), 1.0);
        assert_eq!((-1.7).to_length(), 0.0); // Negative, so becomes 0
        assert_eq!((-1.2).to_length(), 0.0); // Negative, so becomes 0

        // Integer types
        assert_eq!(100_i32.to_length(), 100.0);
        assert_eq!((-5_i32).to_length(), 0.0);
        assert_eq!(100_u32.to_length(), 100.0);
    }
}
