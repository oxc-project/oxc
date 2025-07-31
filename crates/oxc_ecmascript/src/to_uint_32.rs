/// `ToUint32 ( argument )`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-touint32>
pub trait ToUint32 {
    fn to_uint_32(&self) -> u32;
}

impl ToUint32 for f64 {
    fn to_uint_32(&self) -> u32 {
        // 1. Let number be ? ToNumber(argument).
        let number = *self;

        // 2. If number is not finite or number is +0𝔽 or number is -0𝔽, return +0𝔽.
        if !number.is_finite() || number == 0.0 {
            return 0;
        }

        // 3. Let int be truncate(ℝ(number)).
        let int = number.trunc();

        // 4. Let int32bit be int modulo 2^32.
        let int32bit = int as i64 % (1_i64 << 32);

        // 5. Return 𝔽(int32bit).
        int32bit as u32
    }
}

impl ToUint32 for i32 {
    fn to_uint_32(&self) -> u32 {
        *self as u32
    }
}

impl ToUint32 for u32 {
    fn to_uint_32(&self) -> u32 {
        *self
    }
}

#[cfg(test)]
mod test {
    use super::ToUint32;

    #[test]
    fn test_to_uint_32() {
        // Test cases from ECMAScript specification and common scenarios
        assert_eq!(0.0_f64.to_uint_32(), 0);
        assert_eq!((-0.0_f64).to_uint_32(), 0);
        assert_eq!(f64::INFINITY.to_uint_32(), 0);
        assert_eq!(f64::NEG_INFINITY.to_uint_32(), 0);
        assert_eq!(f64::NAN.to_uint_32(), 0);
        
        assert_eq!(1.0_f64.to_uint_32(), 1);
        assert_eq!((-1.0_f64).to_uint_32(), 4294967295); // 2^32 - 1
        assert_eq!(4294967295.0_f64.to_uint_32(), 4294967295);
        assert_eq!(4294967296.0_f64.to_uint_32(), 0); // 2^32 wraps to 0
        assert_eq!(4294967297.0_f64.to_uint_32(), 1); // 2^32 + 1 wraps to 1
        
        // Test fractional values
        assert_eq!(1.5_f64.to_uint_32(), 1);
        assert_eq!((-1.5_f64).to_uint_32(), 4294967295);
        
        // Test large values
        assert_eq!(9007199254740992.0_f64.to_uint_32(), 0); // 2^53
        
        // Test integer types
        assert_eq!(42_i32.to_uint_32(), 42);
        assert_eq!((-1_i32).to_uint_32(), 4294967295);
        assert_eq!(42_u32.to_uint_32(), 42);
    }
}