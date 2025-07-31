/// `ToUint16 ( argument )`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-touint16>
pub trait ToUint16 {
    fn to_uint_16(&self) -> u16;
}

impl ToUint16 for f64 {
    fn to_uint_16(&self) -> u16 {
        // 1. Let number be ? ToNumber(argument).
        let number = *self;

        // 2. If number is not finite or number is +0𝔽 or number is -0𝔽, return +0𝔽.
        if !number.is_finite() || number == 0.0 {
            return 0;
        }

        // 3. Let int be truncate(ℝ(number)).
        let int = number.trunc();

        // 4. Let int16bit be int modulo 2^16.
        let int16bit = int as i32 % (1_i32 << 16);

        // 5. Return 𝔽(int16bit).
        int16bit as u16
    }
}

impl ToUint16 for i32 {
    fn to_uint_16(&self) -> u16 {
        *self as u16
    }
}

impl ToUint16 for u32 {
    fn to_uint_16(&self) -> u16 {
        *self as u16
    }
}

impl ToUint16 for u16 {
    fn to_uint_16(&self) -> u16 {
        *self
    }
}

#[cfg(test)]
mod test {
    use super::ToUint16;

    #[test]
    fn test_to_uint_16() {
        // Test cases from ECMAScript specification and common scenarios
        assert_eq!(0.0_f64.to_uint_16(), 0);
        assert_eq!((-0.0_f64).to_uint_16(), 0);
        assert_eq!(f64::INFINITY.to_uint_16(), 0);
        assert_eq!(f64::NEG_INFINITY.to_uint_16(), 0);
        assert_eq!(f64::NAN.to_uint_16(), 0);
        
        assert_eq!(1.0_f64.to_uint_16(), 1);
        assert_eq!((-1.0_f64).to_uint_16(), 65535); // 2^16 - 1
        assert_eq!(65535.0_f64.to_uint_16(), 65535);
        assert_eq!(65536.0_f64.to_uint_16(), 0); // 2^16 wraps to 0
        assert_eq!(65537.0_f64.to_uint_16(), 1); // 2^16 + 1 wraps to 1
        
        // Test fractional values
        assert_eq!(1.5_f64.to_uint_16(), 1);
        assert_eq!((-1.5_f64).to_uint_16(), 65535);
        
        // Test integer types
        assert_eq!(42_i32.to_uint_16(), 42);
        assert_eq!((-1_i32).to_uint_16(), 65535);
        assert_eq!(42_u32.to_uint_16(), 42);
        assert_eq!(42_u16.to_uint_16(), 42);
    }
}