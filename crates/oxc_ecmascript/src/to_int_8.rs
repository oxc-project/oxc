//! 7.1.10 ToInt8 ( argument )
//! <https://tc39.es/ecma262/#sec-toint8>

use crate::to_integer_or_infinity::ToIntegerOrInfinity;

const TWO_TO_THE_8: f64 = 256.0;

/// `ToInt8` converts argument to one of 2^8 integer values in the range -128 to 127, inclusive.
pub trait ToInt8 {
    fn to_int_8(&self) -> i8;
}

impl ToInt8 for f64 {
    fn to_int_8(&self) -> i8 {
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
        
        // 5. If int8bit ≥ 2^7, return 𝔽(int8bit - 2^8); otherwise return 𝔽(int8bit).
        if int8bit >= 128.0 {
            (int8bit - TWO_TO_THE_8) as i8
        } else {
            int8bit as i8
        }
    }
}

impl ToInt8 for i32 {
    fn to_int_8(&self) -> i8 {
        (*self as f64).to_int_8()
    }
}

impl ToInt8 for u32 {
    fn to_int_8(&self) -> i8 {
        (*self as f64).to_int_8()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_int_8() {
        // Basic cases
        assert_eq!(0.0.to_int_8(), 0);
        assert_eq!(1.0.to_int_8(), 1);
        assert_eq!(127.0.to_int_8(), 127);
        assert_eq!(128.0.to_int_8(), -128);
        assert_eq!(255.0.to_int_8(), -1);
        assert_eq!(256.0.to_int_8(), 0);
        assert_eq!((-1.0).to_int_8(), -1);
        assert_eq!((-128.0).to_int_8(), -128);
        assert_eq!((-129.0).to_int_8(), 127);
        
        // Edge cases
        assert_eq!(f64::NAN.to_int_8(), 0);
        assert_eq!(f64::INFINITY.to_int_8(), 0);
        assert_eq!(f64::NEG_INFINITY.to_int_8(), 0);
        assert_eq!(0.0.to_int_8(), 0);
        assert_eq!((-0.0).to_int_8(), 0);
        
        // Fractional values
        assert_eq!(1.7.to_int_8(), 1);
        assert_eq!(1.2.to_int_8(), 1);
        assert_eq!((-1.7).to_int_8(), -1);
        assert_eq!((-1.2).to_int_8(), -1);
        
        // Large values that wrap around
        assert_eq!(1000.0.to_int_8(), -24); // 1000 % 256 = 232, 232 - 256 = -24
        assert_eq!((-1000.0).to_int_8(), 24); // -1000 % 256 = 24
        
        // Integer types
        assert_eq!(127_i32.to_int_8(), 127);
        assert_eq!(128_i32.to_int_8(), -128);
        assert_eq!(255_u32.to_int_8(), -1);
    }
}