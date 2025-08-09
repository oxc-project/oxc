/// `Math.round ( x )`
///
/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.round>
pub trait MathRound {
    fn math_round(&self) -> f64;
}

impl MathRound for f64 {
    fn math_round(&self) -> f64 {
        // 1. Let n be ? ToNumber(x).
        let n = *self;

        // 2. If n is not finite or n is an integral Number, return n.
        if !n.is_finite() || n.fract() == 0.0 {
            return n;
        }

        // 3. If n < 0.5 and n > +0, return +0.
        if n < 0.5 && n > 0.0 {
            return 0.0;
        }

        // 4. If n < +0 and n ≥ -0.5, return -0.
        if n < 0.0 && n >= -0.5 {
            return -0.0;
        }

        // 5. Return the integral Number closest to n, preferring the Number closer to +∞ in the case of a tie.
        // We need to implement this manually since Rust's round() rounds away from zero
        let truncated = n.trunc();
        let fractional = n - truncated;

        if fractional.abs() < 0.5 {
            // Closer to truncated value
            truncated
        } else if fractional.abs() > 0.5 {
            // Closer to next integer
            if n > 0.0 { truncated + 1.0 } else { truncated - 1.0 }
        } else {
            // Exactly halfway - prefer the number closer to +∞
            if fractional > 0.0 {
                // Positive halfway case: round up
                truncated + 1.0
            } else {
                // Negative halfway case: round up (towards positive infinity)
                truncated
            }
        }
    }
}

impl MathRound for i32 {
    fn math_round(&self) -> f64 {
        *self as f64
    }
}

/// `Math.max ( ...args )`
///
/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.max>
pub trait MathMax {
    fn math_max(&self, other: f64) -> f64;
}

impl MathMax for f64 {
    fn math_max(&self, other: f64) -> f64 {
        let a = *self;
        let b = other;

        // If either value is NaN, return NaN
        if a.is_nan() || b.is_nan() {
            return f64::NAN;
        }

        // Handle positive and negative zero correctly
        if a == 0.0 && b == 0.0 {
            if a.is_sign_positive() || b.is_sign_positive() {
                return 0.0; // +0
            } else {
                return -0.0; // -0
            }
        }

        a.max(b)
    }
}

/// Helper function for Math.max with multiple arguments
pub fn math_max_values(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NEG_INFINITY;
    }

    let mut result = values[0];
    for &value in &values[1..] {
        result = result.math_max(value);
    }
    result
}

/// `Math.min ( ...args )`
///
/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.min>
pub trait MathMin {
    fn math_min(&self, other: f64) -> f64;
}

impl MathMin for f64 {
    fn math_min(&self, other: f64) -> f64 {
        let a = *self;
        let b = other;

        // If either value is NaN, return NaN
        if a.is_nan() || b.is_nan() {
            return f64::NAN;
        }

        // Handle positive and negative zero correctly
        if a == 0.0 && b == 0.0 {
            if a.is_sign_negative() || b.is_sign_negative() {
                return -0.0; // -0
            } else {
                return 0.0; // +0
            }
        }

        a.min(b)
    }
}

/// Helper function for Math.min with multiple arguments
pub fn math_min_values(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::INFINITY;
    }

    let mut result = values[0];
    for &value in &values[1..] {
        result = result.math_min(value);
    }
    result
}

#[cfg(test)]
mod test {
    use super::{MathMax, MathMin, MathRound, math_max_values, math_min_values};

    #[test]
    fn test_math_round() {
        // Positive numbers
        assert_eq!(5.3_f64.math_round(), 5.0);
        assert_eq!(5.5_f64.math_round(), 6.0); // Round half up
        assert_eq!(5.7_f64.math_round(), 6.0);
        assert_eq!(0.4_f64.math_round(), 0.0);
        assert_eq!(0.5_f64.math_round(), 1.0);

        // Negative numbers
        assert_eq!((-5.3_f64).math_round(), -5.0);
        assert_eq!((-5.5_f64).math_round(), -5.0); // Round half away from zero
        assert_eq!((-5.7_f64).math_round(), -6.0);
        assert_eq!((-0.4_f64).math_round(), -0.0);
        assert_eq!((-0.5_f64).math_round(), -0.0);

        // Special values
        assert_eq!(0.0_f64.math_round(), 0.0);
        assert_eq!((-0.0_f64).math_round(), -0.0);
        assert!(f64::NAN.math_round().is_nan());
        assert_eq!(f64::INFINITY.math_round(), f64::INFINITY);
        assert_eq!(f64::NEG_INFINITY.math_round(), f64::NEG_INFINITY);

        // Integer input
        assert_eq!(5_i32.math_round(), 5.0);
        assert_eq!((-5_i32).math_round(), -5.0);
    }

    #[test]
    fn test_math_max() {
        // Basic functionality
        assert_eq!(5.0_f64.math_max(3.0), 5.0);
        assert_eq!(3.0_f64.math_max(5.0), 5.0);
        assert_eq!(5.0_f64.math_max(5.0), 5.0);

        // With negative numbers
        assert_eq!((-3.0_f64).math_max(-5.0), -3.0);
        assert_eq!((-5.0_f64).math_max(-3.0), -3.0);
        assert_eq!(5.0_f64.math_max(-3.0), 5.0);

        // With NaN
        assert!(5.0_f64.math_max(f64::NAN).is_nan());
        assert!(f64::NAN.math_max(5.0).is_nan());

        // With infinity
        assert_eq!(5.0_f64.math_max(f64::INFINITY), f64::INFINITY);
        assert_eq!(f64::NEG_INFINITY.math_max(5.0), 5.0);

        // Zero handling
        assert_eq!(0.0_f64.math_max(-0.0), 0.0);
        assert_eq!((-0.0_f64).math_max(0.0), 0.0);

        // Multiple values helper
        assert_eq!(math_max_values(&[1.0, 3.0, 2.0]), 3.0);
        assert_eq!(math_max_values(&[]), f64::NEG_INFINITY);
    }

    #[test]
    fn test_math_min() {
        // Basic functionality
        assert_eq!(5.0_f64.math_min(3.0), 3.0);
        assert_eq!(3.0_f64.math_min(5.0), 3.0);
        assert_eq!(5.0_f64.math_min(5.0), 5.0);

        // With negative numbers
        assert_eq!((-3.0_f64).math_min(-5.0), -5.0);
        assert_eq!((-5.0_f64).math_min(-3.0), -5.0);
        assert_eq!(5.0_f64.math_min(-3.0), -3.0);

        // With NaN
        assert!(5.0_f64.math_min(f64::NAN).is_nan());
        assert!(f64::NAN.math_min(5.0).is_nan());

        // With infinity
        assert_eq!(5.0_f64.math_min(f64::INFINITY), 5.0);
        assert_eq!(f64::NEG_INFINITY.math_min(5.0), f64::NEG_INFINITY);

        // Zero handling
        assert_eq!(0.0_f64.math_min(-0.0), -0.0);
        assert_eq!((-0.0_f64).math_min(0.0), -0.0);

        // Multiple values helper
        assert_eq!(math_min_values(&[1.0, 3.0, 2.0]), 1.0);
        assert_eq!(math_min_values(&[]), f64::INFINITY);
    }
}
