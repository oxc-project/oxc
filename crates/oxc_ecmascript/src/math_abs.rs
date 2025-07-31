use num_bigint::BigInt;
use num_traits::Signed;

/// `Math.abs ( x )`
///
/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.abs>
pub trait MathAbs {
    type Output;
    fn math_abs(&self) -> Self::Output;
}

impl MathAbs for f64 {
    type Output = f64;
    
    fn math_abs(&self) -> Self::Output {
        // 1. Let n be ? ToNumber(x).
        let n = *self;
        
        // 2. If n is NaN, return NaN.
        if n.is_nan() {
            return f64::NAN;
        }
        
        // 3. If n is -0𝔽, return +0𝔽.
        if n == -0.0 {
            return 0.0;
        }
        
        // 4. If n is -∞𝔽, return +∞𝔽.
        if n == f64::NEG_INFINITY {
            return f64::INFINITY;
        }
        
        // 5. If n < +0𝔽, return -n.
        // 6. Return n.
        n.abs()
    }
}

impl MathAbs for i32 {
    type Output = f64;
    
    fn math_abs(&self) -> Self::Output {
        (*self as f64).math_abs()
    }
}

impl MathAbs for BigInt {
    type Output = BigInt;
    
    fn math_abs(&self) -> Self::Output {
        self.abs()
    }
}

#[cfg(test)]
mod test {
    use super::MathAbs;
    use num_bigint::BigInt;

    #[test]
    fn test_math_abs() {
        // Positive numbers
        assert_eq!(5.0_f64.math_abs(), 5.0);
        assert_eq!(0.0_f64.math_abs(), 0.0);
        
        // Negative numbers
        assert_eq!((-5.0_f64).math_abs(), 5.0);
        assert_eq!((-0.0_f64).math_abs(), 0.0);
        
        // Special values
        assert!(f64::NAN.math_abs().is_nan());
        assert_eq!(f64::INFINITY.math_abs(), f64::INFINITY);
        assert_eq!(f64::NEG_INFINITY.math_abs(), f64::INFINITY);
        
        // Fractional numbers
        assert_eq!((-3.14_f64).math_abs(), 3.14);
        assert_eq!(3.14_f64.math_abs(), 3.14);
        
        // Integer types
        assert_eq!(5_i32.math_abs(), 5.0);
        assert_eq!((-5_i32).math_abs(), 5.0);
        
        // BigInt
        let big_pos = BigInt::from(12345);
        let big_neg = BigInt::from(-12345);
        assert_eq!(big_pos.math_abs(), BigInt::from(12345));
        assert_eq!(big_neg.math_abs(), BigInt::from(12345));
    }
}