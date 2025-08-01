//! 7.2.11 SameValue ( x, y )
//! 7.2.12 SameValueZero ( x, y )
//! <https://tc39.es/ecma262/#sec-samevalue>
//! <https://tc39.es/ecma262/#sec-samevaluezero>

/// `SameValue` determines whether two ECMAScript language values are the same value.
/// This is more strict than == and === as it distinguishes +0 from -0 and considers NaN equal to itself.
pub fn same_value(x: f64, y: f64) -> bool {
    // 1. If Type(x) is different from Type(y), return false.
    // (We only handle f64 here, so types are always the same)
    
    // 2. If Type(x) is Number, then
    //    a. Return ! Number::sameValue(x, y).
    number_same_value(x, y)
}

/// `SameValueZero` is like SameValue but treats +0 and -0 as the same.
pub fn same_value_zero(x: f64, y: f64) -> bool {
    // 1. If Type(x) is different from Type(y), return false.
    // (We only handle f64 here, so types are always the same)
    
    // 2. If Type(x) is Number, then
    //    a. Return ! Number::sameValueZero(x, y).
    number_same_value_zero(x, y)
}

/// Number::sameValue ( x, y )
/// <https://tc39.es/ecma262/#sec-numeric-types-number-samevalue>
fn number_same_value(x: f64, y: f64) -> bool {
    // 1. If x is NaN and y is NaN, return true.
    if x.is_nan() && y.is_nan() {
        return true;
    }
    
    // 2. If x is +0𝔽 and y is -0𝔽, return false.
    if x == 0.0 && y == 0.0 && x.is_sign_positive() && y.is_sign_negative() {
        return false;
    }
    
    // 3. If x is -0𝔽 and y is +0𝔽, return false.
    if x == 0.0 && y == 0.0 && x.is_sign_negative() && y.is_sign_positive() {
        return false;
    }
    
    // 4. If x is the same Number value as y, return true.
    // 5. Return false.
    x == y
}

/// Number::sameValueZero ( x, y )
/// <https://tc39.es/ecma262/#sec-numeric-types-number-samevaluezero>
fn number_same_value_zero(x: f64, y: f64) -> bool {
    // 1. If x is NaN and y is NaN, return true.
    if x.is_nan() && y.is_nan() {
        return true;
    }
    
    // 2. If x is +0𝔽 and y is -0𝔽, return true.
    if x == 0.0 && y == 0.0 {
        return true;
    }
    
    // 3. If x is -0𝔽 and y is +0𝔽, return true.
    if x == 0.0 && y == 0.0 {
        return true;
    }
    
    // 4. If x is the same Number value as y, return true.
    // 5. Return false.
    x == y
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_same_value() {
        // Normal equality cases
        assert!(same_value(1.0, 1.0));
        assert!(same_value(0.0, 0.0));
        assert!(same_value(-1.0, -1.0));
        assert!(!same_value(1.0, 2.0));
        assert!(!same_value(0.0, 1.0));
        
        // NaN cases - SameValue considers NaN equal to itself
        assert!(same_value(f64::NAN, f64::NAN));
        assert!(!same_value(f64::NAN, 0.0));
        assert!(!same_value(0.0, f64::NAN));
        
        // Zero sign distinction - SameValue distinguishes +0 from -0
        assert!(same_value(0.0, 0.0)); // +0 === +0
        assert!(same_value(-0.0, -0.0)); // -0 === -0
        assert!(!same_value(0.0, -0.0)); // +0 !== -0
        assert!(!same_value(-0.0, 0.0)); // -0 !== +0
        
        // Infinity cases
        assert!(same_value(f64::INFINITY, f64::INFINITY));
        assert!(same_value(f64::NEG_INFINITY, f64::NEG_INFINITY));
        assert!(!same_value(f64::INFINITY, f64::NEG_INFINITY));
        assert!(!same_value(f64::INFINITY, 1.0));
    }

    #[test]
    fn test_same_value_zero() {
        // Normal equality cases
        assert!(same_value_zero(1.0, 1.0));
        assert!(same_value_zero(0.0, 0.0));
        assert!(same_value_zero(-1.0, -1.0));
        assert!(!same_value_zero(1.0, 2.0));
        assert!(!same_value_zero(0.0, 1.0));
        
        // NaN cases - SameValueZero considers NaN equal to itself
        assert!(same_value_zero(f64::NAN, f64::NAN));
        assert!(!same_value_zero(f64::NAN, 0.0));
        assert!(!same_value_zero(0.0, f64::NAN));
        
        // Zero sign cases - SameValueZero treats +0 and -0 as equal
        assert!(same_value_zero(0.0, 0.0)); // +0 === +0
        assert!(same_value_zero(-0.0, -0.0)); // -0 === -0
        assert!(same_value_zero(0.0, -0.0)); // +0 === -0 (difference from SameValue)
        assert!(same_value_zero(-0.0, 0.0)); // -0 === +0 (difference from SameValue)
        
        // Infinity cases
        assert!(same_value_zero(f64::INFINITY, f64::INFINITY));
        assert!(same_value_zero(f64::NEG_INFINITY, f64::NEG_INFINITY));
        assert!(!same_value_zero(f64::INFINITY, f64::NEG_INFINITY));
        assert!(!same_value_zero(f64::INFINITY, 1.0));
    }

    #[test]
    fn test_number_same_value() {
        // Basic tests for the internal function
        assert!(number_same_value(1.0, 1.0));
        assert!(number_same_value(f64::NAN, f64::NAN));
        assert!(!number_same_value(0.0, -0.0));
        assert!(!number_same_value(-0.0, 0.0));
    }

    #[test]
    fn test_number_same_value_zero() {
        // Basic tests for the internal function
        assert!(number_same_value_zero(1.0, 1.0));
        assert!(number_same_value_zero(f64::NAN, f64::NAN));
        assert!(number_same_value_zero(0.0, -0.0));
        assert!(number_same_value_zero(-0.0, 0.0));
    }
}