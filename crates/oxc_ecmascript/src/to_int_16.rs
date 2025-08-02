//! 7.1.11 ToInt16 ( argument )
//! <https://tc39.es/ecma262/#sec-toint16>

use crate::to_integer_or_infinity::ToIntegerOrInfinity;

const TWO_TO_THE_16: f64 = 65536.0;

/// `ToInt16` converts argument to one of 2^16 integer values in the range -32768 to 32767, inclusive.
pub trait ToInt16 {
    fn to_int_16(&self) -> i16;
}

impl ToInt16 for f64 {
    fn to_int_16(&self) -> i16 {
        // 1. Let number be ? ToNumber(argument).
        let number = *self;

        // 2. If number is not finite or number is +0𝔽 or number is -0𝔽, return +0𝔽.
        if !number.is_finite() || number == 0.0 {
            return 0;
        }

        // 3. Let int be the mathematical value whose sign is the sign of number and whose magnitude is floor(abs(ℝ(number))).
        let int = number.to_integer_or_infinity();

        // 4. Let int16bit be int modulo 2^16.
        let int16bit = int.rem_euclid(TWO_TO_THE_16);

        // 5. If int16bit ≥ 2^15, return 𝔽(int16bit - 2^16); otherwise return 𝔽(int16bit).
        if int16bit >= 32768.0 { (int16bit - TWO_TO_THE_16) as i16 } else { int16bit as i16 }
    }
}

impl ToInt16 for i32 {
    fn to_int_16(&self) -> i16 {
        (*self as f64).to_int_16()
    }
}

impl ToInt16 for u32 {
    fn to_int_16(&self) -> i16 {
        (*self as f64).to_int_16()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_int_16() {
        // Basic cases
        assert_eq!(0.0.to_int_16(), 0);
        assert_eq!(1.0.to_int_16(), 1);
        assert_eq!(32767.0.to_int_16(), 32767);
        assert_eq!(32768.0.to_int_16(), -32768);
        assert_eq!(65535.0.to_int_16(), -1);
        assert_eq!(65536.0.to_int_16(), 0);
        assert_eq!((-1.0).to_int_16(), -1);
        assert_eq!((-32768.0).to_int_16(), -32768);
        assert_eq!((-32769.0).to_int_16(), 32767);

        // Edge cases
        assert_eq!(f64::NAN.to_int_16(), 0);
        assert_eq!(f64::INFINITY.to_int_16(), 0);
        assert_eq!(f64::NEG_INFINITY.to_int_16(), 0);
        assert_eq!(0.0.to_int_16(), 0);
        assert_eq!((-0.0).to_int_16(), 0);

        // Fractional values
        assert_eq!(1.7.to_int_16(), 1);
        assert_eq!(1.2.to_int_16(), 1);
        assert_eq!((-1.7).to_int_16(), -1);
        assert_eq!((-1.2).to_int_16(), -1);

        // Large values that wrap around
        assert_eq!(100000.0.to_int_16(), -31072); // 100000 % 65536 = 34464, 34464 - 65536 = -31072
        assert_eq!((-100000.0).to_int_16(), 31072); // -100000 % 65536 = 31072

        // Integer types
        assert_eq!(32767_i32.to_int_16(), 32767);
        assert_eq!(32768_i32.to_int_16(), -32768);
        assert_eq!(65535_u32.to_int_16(), -1);
    }
}
