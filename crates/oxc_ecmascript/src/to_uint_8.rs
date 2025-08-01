//! 7.1.12 ToUint8 ( argument )
//! <https://tc39.es/ecma262/#sec-touint8>

use crate::to_integer_or_infinity::ToIntegerOrInfinity;

const TWO_TO_THE_8: f64 = 256.0;

/// `ToUint8` converts argument to one of 2^8 integer values in the range 0 to 255, inclusive.
pub trait ToUint8 {
    fn to_uint_8(&self) -> u8;
}

impl ToUint8 for f64 {
    fn to_uint_8(&self) -> u8 {
        // 1. Let number be ? ToNumber(argument).
        let number = *self;

        // 2. If number is not finite or number is +0𝔽 or number is -0𝔽, return +0𝔽.
        if !number.is_finite() || number == 0.0 {
            return 0;
        }

        // 3. Let int be the mathematical value whose sign is the sign of number and whose magnitude is floor(abs(ℝ(number))).
        let int = number.to_integer_or_infinity();

        // 4. Let int8bit be int modulo 2^8.
        let int8bit = int.rem_euclid(TWO_TO_THE_8);

        // 5. Return 𝔽(int8bit).
        int8bit as u8
    }
}

impl ToUint8 for i32 {
    fn to_uint_8(&self) -> u8 {
        (*self as f64).to_uint_8()
    }
}

impl ToUint8 for u32 {
    fn to_uint_8(&self) -> u8 {
        (*self as f64).to_uint_8()
    }
}

/// `ToUint8Clamp` converts argument to one of 2^8 integer values in the range 0 to 255, inclusive, by clamping.
pub trait ToUint8Clamp {
    fn to_uint_8_clamp(&self) -> u8;
}

impl ToUint8Clamp for f64 {
    fn to_uint_8_clamp(&self) -> u8 {
        // 1. Let number be ? ToNumber(argument).
        let number = *self;

        // 2. If number is NaN, return +0𝔽.
        if number.is_nan() {
            return 0;
        }

        // 3. If number ≤ 0, return +0𝔽.
        if number <= 0.0 {
            return 0;
        }

        // 4. If number ≥ 255, return 255𝔽.
        if number >= 255.0 {
            return 255;
        }

        // 5. Let f be floor(number).
        let f = number.floor();

        // 6. If f + 0.5 < number, return 𝔽(f + 1).
        if f + 0.5 < number {
            return (f + 1.0) as u8;
        }

        // 7. If number < f + 0.5, return 𝔽(f).
        if number < f + 0.5 {
            return f as u8;
        }

        // 8. If f is odd, return 𝔽(f + 1).
        if (f as u64) % 2 == 1 {
            return (f + 1.0) as u8;
        }

        // 9. Return 𝔽(f).
        f as u8
    }
}

impl ToUint8Clamp for i32 {
    fn to_uint_8_clamp(&self) -> u8 {
        (*self as f64).to_uint_8_clamp()
    }
}

impl ToUint8Clamp for u32 {
    fn to_uint_8_clamp(&self) -> u8 {
        (*self as f64).to_uint_8_clamp()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_uint_8() {
        // Basic cases
        assert_eq!(0.0.to_uint_8(), 0);
        assert_eq!(1.0.to_uint_8(), 1);
        assert_eq!(255.0.to_uint_8(), 255);
        assert_eq!(256.0.to_uint_8(), 0);
        assert_eq!(257.0.to_uint_8(), 1);
        assert_eq!((-1.0).to_uint_8(), 255);
        assert_eq!((-256.0).to_uint_8(), 0);
        assert_eq!((-257.0).to_uint_8(), 255);

        // Edge cases
        assert_eq!(f64::NAN.to_uint_8(), 0);
        assert_eq!(f64::INFINITY.to_uint_8(), 0);
        assert_eq!(f64::NEG_INFINITY.to_uint_8(), 0);
        assert_eq!(0.0.to_uint_8(), 0);
        assert_eq!((-0.0).to_uint_8(), 0);

        // Fractional values
        assert_eq!(1.7.to_uint_8(), 1);
        assert_eq!(1.2.to_uint_8(), 1);
        assert_eq!((-1.7).to_uint_8(), 255);
        assert_eq!((-1.2).to_uint_8(), 255);

        // Integer types
        assert_eq!(255_i32.to_uint_8(), 255);
        assert_eq!(256_i32.to_uint_8(), 0);
        assert_eq!(255_u32.to_uint_8(), 255);
    }

    #[test]
    fn test_to_uint_8_clamp() {
        // Basic cases
        assert_eq!(0.0.to_uint_8_clamp(), 0);
        assert_eq!(1.0.to_uint_8_clamp(), 1);
        assert_eq!(255.0.to_uint_8_clamp(), 255);
        assert_eq!(256.0.to_uint_8_clamp(), 255);
        assert_eq!(1000.0.to_uint_8_clamp(), 255);
        assert_eq!((-1.0).to_uint_8_clamp(), 0);
        assert_eq!((-1000.0).to_uint_8_clamp(), 0);

        // Edge cases
        assert_eq!(f64::NAN.to_uint_8_clamp(), 0);
        assert_eq!(f64::INFINITY.to_uint_8_clamp(), 255);
        assert_eq!(f64::NEG_INFINITY.to_uint_8_clamp(), 0);
        assert_eq!(0.0.to_uint_8_clamp(), 0);
        assert_eq!((-0.0).to_uint_8_clamp(), 0);

        // Banker's rounding cases (round half to even)
        assert_eq!(0.5.to_uint_8_clamp(), 0); // 0 is even, so round down
        assert_eq!(1.5.to_uint_8_clamp(), 2); // 1 is odd, so round up
        assert_eq!(2.5.to_uint_8_clamp(), 2); // 2 is even, so round down
        assert_eq!(3.5.to_uint_8_clamp(), 4); // 3 is odd, so round up

        // Other fractional values
        assert_eq!(1.3.to_uint_8_clamp(), 1);
        assert_eq!(1.7.to_uint_8_clamp(), 2);

        // Integer types
        assert_eq!(255_i32.to_uint_8_clamp(), 255);
        assert_eq!(1000_i32.to_uint_8_clamp(), 255);
        assert_eq!(255_u32.to_uint_8_clamp(), 255);
    }
}
