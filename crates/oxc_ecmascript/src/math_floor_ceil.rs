/// `Math.floor ( x )`
///
/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.floor>
pub trait MathFloor {
    fn math_floor(&self) -> f64;
}

impl MathFloor for f64 {
    fn math_floor(&self) -> f64 {
        // 1. Let n be ? ToNumber(x).
        let n = *self;

        // 2. If n is not finite or n is an integral Number, return n.
        if !n.is_finite() || n.fract() == 0.0 {
            return n;
        }

        // 3. If n < 1 and n > +0, return +0.
        if n < 1.0 && n > 0.0 {
            return 0.0;
        }

        // 4. If n is less than +0, return the largest (closest to +∞) integral Number value that is not greater than n.
        // 5. Return the largest (closest to +∞) integral Number value that is not greater than n.
        n.floor()
    }
}

impl MathFloor for i32 {
    fn math_floor(&self) -> f64 {
        *self as f64
    }
}

/// `Math.ceil ( x )`
///
/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.ceil>
pub trait MathCeil {
    fn math_ceil(&self) -> f64;
}

impl MathCeil for f64 {
    fn math_ceil(&self) -> f64 {
        // 1. Let n be ? ToNumber(x).
        let n = *self;

        // 2. If n is not finite or n is an integral Number, return n.
        if !n.is_finite() || n.fract() == 0.0 {
            return n;
        }

        // 3. If n < +0 and n > -1, return -0.
        if n < 0.0 && n > -1.0 {
            return -0.0;
        }

        // 4. If n is greater than +0, return the smallest (closest to -∞) integral Number value that is not less than n.
        // 5. Return the smallest (closest to -∞) integral Number value that is not less than n.
        n.ceil()
    }
}

impl MathCeil for i32 {
    fn math_ceil(&self) -> f64 {
        *self as f64
    }
}

#[cfg(test)]
mod test {
    use super::{MathCeil, MathFloor};

    #[test]
    fn test_math_floor() {
        // Positive numbers
        assert_eq!(5.3_f64.math_floor(), 5.0);
        assert_eq!(5.0_f64.math_floor(), 5.0);
        assert_eq!(0.9_f64.math_floor(), 0.0);
        assert_eq!(0.1_f64.math_floor(), 0.0);

        // Negative numbers
        assert_eq!((-5.3_f64).math_floor(), -6.0);
        assert_eq!((-5.0_f64).math_floor(), -5.0);
        assert_eq!((-0.1_f64).math_floor(), -1.0);
        assert_eq!((-0.9_f64).math_floor(), -1.0);

        // Special values
        assert_eq!(0.0_f64.math_floor(), 0.0);
        assert_eq!((-0.0_f64).math_floor(), -0.0);
        assert!(f64::NAN.math_floor().is_nan());
        assert_eq!(f64::INFINITY.math_floor(), f64::INFINITY);
        assert_eq!(f64::NEG_INFINITY.math_floor(), f64::NEG_INFINITY);

        // Integer input
        assert_eq!(5_i32.math_floor(), 5.0);
        assert_eq!((-5_i32).math_floor(), -5.0);
    }

    #[test]
    fn test_math_ceil() {
        // Positive numbers
        assert_eq!(5.3_f64.math_ceil(), 6.0);
        assert_eq!(5.0_f64.math_ceil(), 5.0);
        assert_eq!(0.9_f64.math_ceil(), 1.0);
        assert_eq!(0.1_f64.math_ceil(), 1.0);

        // Negative numbers
        assert_eq!((-5.3_f64).math_ceil(), -5.0);
        assert_eq!((-5.0_f64).math_ceil(), -5.0);
        assert_eq!((-0.1_f64).math_ceil(), -0.0);
        assert_eq!((-0.9_f64).math_ceil(), -0.0);

        // Special values
        assert_eq!(0.0_f64.math_ceil(), 0.0);
        assert_eq!((-0.0_f64).math_ceil(), -0.0);
        assert!(f64::NAN.math_ceil().is_nan());
        assert_eq!(f64::INFINITY.math_ceil(), f64::INFINITY);
        assert_eq!(f64::NEG_INFINITY.math_ceil(), f64::NEG_INFINITY);

        // Integer input
        assert_eq!(5_i32.math_ceil(), 5.0);
        assert_eq!((-5_i32).math_ceil(), -5.0);
    }
}
