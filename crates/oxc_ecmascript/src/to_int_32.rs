/// Converts a 64-bit floating point number to an `i32` according to the [`ToInt32`][ToInt32] algorithm.
///
/// [ToInt32]: https://tc39.es/ecma262/#sec-toint32
///
/// This is copied from [Boa](https://github.com/boa-dev/boa/blob/61567687cf4bfeca6bd548c3e72b6965e74b2461/core/engine/src/builtins/number/conversions.rs)
pub trait ToInt32 {
    fn to_int_32(&self) -> i32;
}

impl ToInt32 for f64 {
    #[allow(clippy::float_cmp, clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn to_int_32(&self) -> i32 {
        const SIGN_MASK: u64 = 0x8000_0000_0000_0000;
        const EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;
        const SIGNIFICAND_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
        const HIDDEN_BIT: u64 = 0x0010_0000_0000_0000;
        const PHYSICAL_SIGNIFICAND_SIZE: i32 = 52; // Excludes the hidden bit.
        const SIGNIFICAND_SIZE: i32 = 53;

        const EXPONENT_BIAS: i32 = 0x3FF + PHYSICAL_SIGNIFICAND_SIZE;
        const DENORMAL_EXPONENT: i32 = -EXPONENT_BIAS + 1;

        fn is_denormal(number: f64) -> bool {
            (number.to_bits() & EXPONENT_MASK) == 0
        }

        fn exponent(number: f64) -> i32 {
            if is_denormal(number) {
                return DENORMAL_EXPONENT;
            }

            let d64 = number.to_bits();
            let biased_e = ((d64 & EXPONENT_MASK) >> PHYSICAL_SIGNIFICAND_SIZE) as i32;

            biased_e - EXPONENT_BIAS
        }

        fn significand(number: f64) -> u64 {
            let d64 = number.to_bits();
            let significand = d64 & SIGNIFICAND_MASK;

            if is_denormal(number) {
                significand
            } else {
                significand + HIDDEN_BIT
            }
        }

        fn sign(number: f64) -> i64 {
            if (number.to_bits() & SIGN_MASK) == 0 {
                1
            } else {
                -1
            }
        }

        let number = *self;

        if number.is_finite() && number <= f64::from(i32::MAX) && number >= f64::from(i32::MIN) {
            let i = number as i32;
            if f64::from(i) == number {
                return i;
            }
        }

        let exponent = exponent(number);
        let bits = if exponent < 0 {
            if exponent <= -SIGNIFICAND_SIZE {
                return 0;
            }

            significand(number) >> -exponent
        } else {
            if exponent > 31 {
                return 0;
            }

            (significand(number) << exponent) & 0xFFFF_FFFF
        };

        (sign(number) * (bits as i64)) as i32
    }
}
