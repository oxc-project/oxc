use std::{
    cmp::Ordering,
    fmt, hash,
    ops::{self, Deref},
};

use oxc_ast::BigInt;

use crate::{JsFrom, JsInto};

use super::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Numeric {
    Number(Number),
    BigInt(BigInt),
}

impl From<Numeric> for Value<'static> {
    fn from(value: Numeric) -> Self {
        match value {
            Numeric::Number(n) => Value::Number(n),
            Numeric::BigInt(n) => Value::BigInt(n),
        }
    }
}

/// ## [6.1.6.1 The Number Type](https://262.ecma-international.org/15.0/index.html#sec-ecmascript-language-types-number-type)
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Number(f64); // TODO: add i32 variant, like V8's SMI?

impl Deref for Number {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Number {
    pub const ZERO: Number = Number(0.0);
    pub const NEG_ZERO: Number = Number(-0.0);
    pub const ONE: Number = Number(1.0);
    /// Not-a-Number.
    ///
    /// Do not compare values with this, instead use [`Number::is_nan`].
    pub const NAN: Number = Number(f64::NAN);
    pub const INFINITY: Number = Number(f64::INFINITY);
    pub const NEG_INFINITY: Number = Number(f64::NEG_INFINITY);

    #[inline]
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns `true` if this value is NaN.
    #[inline]
    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    #[inline]
    pub fn float(f: f64) -> Self {
        Self(f)
    }

    /// Returns `true` if this [`Number`] is _not_ +/- infinity or [NaN](`Self::NAN`).
    #[inline]
    pub fn is_finite(self) -> bool {
        !self.is_nan() && self != Self::INFINITY && self != Self::NEG_INFINITY
    }

    /// Returns `true` if this [`Number`] is +/- zero.
    #[inline]
    pub fn is_zero(self) -> bool {
        self == Self::ZERO || self == Self::NEG_ZERO
    }

    /// Calls `f` when `self` is a valid number, or returns [`Self::NAN`] when
    /// `self` is NaN.
    fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        // it would be nice to have access to the `unlikely` intrinsic...
        if self.is_nan() {
            Self::NAN
        } else {
            f(self)
        }
    }
}

// 6.1.6.1.1 unaryMinus(x)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-unaryMinus

impl ops::Neg for Number {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.map(|n| Self(-n.0))
    }
}

// 6.1.6.1.2 bitwiseNOT(x)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-bitwiseNOT
// TODO

// 6.1.6.1.3 Number::exponentiate ( base, exponent )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-exponentiate
// TODO

// 6.1.6.1.4 Number::multiply ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-multiply
impl ops::Mul for Number {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        if rhs.is_nan() || rhs.is_nan() {
            return Self::NAN;
        }
        Self(self.0 * rhs.0)
    }
}

// 6.1.6.1.5 Number::divide (`x`, `y`)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-divide
impl ops::Div for Number {
    type Output = Self;
    fn div(/* x */ self, y: Self) -> Self::Output {
        // TODO: ensure this is correct w.r.t 6.1.6.1.5
        // https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-divide
        // 1. If x is NaN or y is NaN, return NaN.
        if self.is_nan() || y.is_nan() {
            return Self::NAN;
        }
        // 2. if x is +∞ or -∞,
        if self.is_infinite() {
            // a. if y is either +∞ or -∞, return NAN
            if y.is_infinite() {
                return Self::NAN;
            }
            // b. is y is +0 or y > +0, return x
            if y >= Self::ZERO && y != Self::NEG_ZERO {
                debug_assert_ne!(y, Self::NEG_ZERO);
                return self;
            }
            // c. Return -x (NOTE: skip Number neg impl, use f64 directly)
            return Self(-self.0);
        }

        // 3. if y is +∞, then
        if y == Self::INFINITY {
            // a. if x is +0 or x > +0, return +0. Otherwise, return -0
            if self >= Self::ZERO {
                return Self::ZERO;
            }
            return Self::NEG_ZERO;
        } else if y == Self::NEG_INFINITY {
            // 4. if y is -∞, then
            // a. if x is +0 or x > +0, return -0. Otherwise, return +0
            if self >= Self::ZERO {
                return Self::NEG_ZERO;
            }
            return Self::ZERO;
        }
        // 5. if x is either +0 or -0, then
        if self.is_zero() {
            // a. if y is either +0 or -0, return NaN
            if y.is_zero() {
                return Self::NAN;
            }
            // b. if y > +0, return x
            if y > Self::ZERO {
                return self;
            }
            // c. return -x
            return Self(-self.0);
        }

        // 6. if y is +0, then
        if y == Self::ZERO {
            // a. if x > +0, return +∞. Otherwise, return -∞
            if self > Self::ZERO {
                return Self::INFINITY;
            } else {
                return Self::NEG_INFINITY;
            }
        }

        // 7. if y is -0, then
        if y == Self::NEG_ZERO {
            // a. if x > +0, return -∞. Otherwise, return +∞
            if self > Self::ZERO {
                return Self::NEG_INFINITY;
            } else {
                return Self::INFINITY;
            }
        }

        // 8. Return F( R(x) / R(y) )
        Self(self.0 / y.0)
    }
}

// 6.1.6.1.6 Number::remainder ( n, d )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-remainder
impl ops::Rem for Number {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        // 1. If n is NaN or d is NaN, return NaN.
        // 2. If n is either +∞𝔽 or -∞𝔽, return NaN.
        // 3. If d is either +∞𝔽 or -∞𝔽, return n.
        // 4. If d is either +0𝔽 or -0𝔽, return NaN.
        // 5. If n is either +0𝔽 or -0𝔽, return n.
        // 6. Assert: n and d are finite and non-zero.
        // 7. Let quotient be ℝ(n) / ℝ(d).
        // 8. Let q be truncate(quotient).
        // 9. Let r be ℝ(n) - (ℝ(d) × q).
        // 10. If r = 0 and n < -0𝔽, return -0𝔽.
        // 11. Return 𝔽(r).

        // 1. If n is NaN or d is NaN, return NaN.
        // 2. If n is either +∞𝔽 or -∞𝔽, return NaN.
        if self.is_nan() || rhs.is_nan() || self.is_infinite() {
            return Self::NAN;
        }

        // 3. If d is either +∞𝔽 or -∞𝔽, return n.
        if rhs.is_infinite() {
            return self;
        }

        // 4. If d is either +0𝔽 or -0𝔽, return NaN.
        if rhs.is_zero() {
            return Self::NAN;
        }

        // 5. If n is either +0𝔽 or -0𝔽, return n.
        if self.is_zero() {
            return self;
        }

        // 6. Assert: n and d are finite and non-zero.
        debug_assert!(self.is_finite() && !self.is_zero());
        debug_assert!(rhs.is_finite() && !rhs.is_zero());
        let q = (self.0 / rhs.0).trunc();
        let r = self.0 - (rhs.0 * q);
        if r == 0.0 && self.is_sign_negative() {
            Self::NEG_ZERO
        } else {
            Self(r)
        }
    }
}

// 6.1.6.1.7 Number::add (`x`, `y`)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-add
impl ops::Add for Number {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

// 6.1.6.1.8 Number::subtract (`x`, `y`)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-subtract
impl ops::Sub for Number {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

// 6.1.6.1.9 Number::leftShift (`x`, `y`)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-subtract
impl ops::Shl for Number {
    type Output = Self;

    /// Performs the `<<` operation.
    /// ## References
    /// - [6.1.6.1.9 Number::leftShift (`x`, `y`)](https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-leftShift)
    fn shl(self, rhs: Self) -> Self::Output {
        // 1. Let lnum be ! ToInt32(x).
        let lnum: i32 = self.into_js();
        // 2. Let rnum be ! ToUint32(y).
        let rnum: u32 = rhs.into_js();
        // 3. Let shiftCount be ℝ(rnum) modulo 32.
        let shift_count = rnum % 32;
        // 4. Return the result of performing a sign-extending right shift of
        //    lnum by shiftCount bits. The most significant bit is propagated.
        //    The mathematical value of the result is exactly representable as a
        //    32-bit two's complement bit string.
        Self(f64::from(lnum << shift_count))
    }
}
// 6.1.6.1.10 Number::signedRightShift ( `x`, `y` )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-signedRightShift
// TODO

// 6.1.6.1.11 Number::unsignedRightShift ( `x`, `y` )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-unsignedRightShift
impl ops::Shr for Number {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
// 6.1.6.1.12 Number::lessThan ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-lessThan
impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 1. If x is NaN or y is NaN, return undefined.
        // 2. if y is Nan< return undefined
        if self.is_nan() || other.is_nan() {
            return None;
        }
        // 3. is x is y, return false
        if self.0 == other.0 {
            return Some(Ordering::Equal);
        }
        // 4. if x is +0 and y is -0, return false
        // 5. if x is -0 and y is +0, return true
        if self.is_zero() && other.is_zero() {
            return Some(Ordering::Equal);
        }

        // 6. if x is +∞, return false
        // 7. if y is +∞, return true
        // 8. if y is -∞, return false
        // 9. if x is -∞, return true
        // 10. Assert: x and y are finite.
        // 11. If R(x) < R(y), return true. Otherwise, return false.
        self.0.partial_cmp(&other.0)
    }
}

// 6.1.6.1.13 Number::equal ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-equal
// derived by PartialEq

// 6.1.6.1.14 Number::sameValue ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-sameValue
impl Number {
    /// 6.1.6.1.14 Number::sameValue (`x`, `y`)
    ///
    /// > The abstract operation Number::sameValue takes arguments `x` (a [`Number`]) and `y` (a
    /// [`Number`]) and returns a Boolean.
    pub fn same_value(&self, y: &Self) -> bool {
        // 1. if ix is Nan and y is NaN, return true
        if self.is_nan() && y.is_nan() {
            return true;
        }
        // 2. if x is +0 and y is -0, return false
        // 3. if x is -0 and y is +0, return false
        if self.is_zero() && y.is_zero() {
            return self.0.is_sign_positive() == y.0.is_sign_positive();
        }
        // 4. if x is y, return true
        // 5. if x is NaN, return false
        self.0 == y.0
    }
}

// 6.1.6.1.15 Number::sameValueZero(`x`, `y`)
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-sameValueZero
impl Number {
    /// 6.1.6.1.15 Number::sameValueZero(`x`, `y`)
    ///
    /// > The abstract operation Number::sameValueZero takes arguments `x` (a [`Number`]) and `y`
    /// (a [`Number`]) and returns a Boolean.
    ///
    /// ## References
    /// - [ECMAScript Standard](https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-sameValueZero)
    pub fn same_value_zero(&self, y: &Self) -> bool {
        // 1. If x is NaN and y is NaN, return true.
        if self.is_nan() && y.is_nan() {
            return true;
        }
        // 2. If x is +0𝔽 and y is -0𝔽, return true.
        // 3. If x is -0𝔽 and y is +0𝔽, return true.
        if self.is_zero() && y.is_zero() {
            return true;
        }

        // 4. If x is y, return true.
        // 5. Return false.
        self.0 == y.0
    }
}

// 6.1.6.1.16 NumberBitwiseOp ( op, x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numberbitwiseop
macro_rules! number_bitwise_op {
    ($op:tt, $x:expr, $y:expr) => {{
        #[cfg(debug_assertions)]
        {
            assert!(!Number::is_nan($x));
            assert!(!Number::is_nan($y));
        }

        let x = i32::from_js($x);
        let y = i32::from_js($y);
        Number::from(x $op y)
    }};
}

// 6.1.6.1.17 Number::bitwiseAND ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-bitwiseAND
impl ops::BitAnd for Number {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        number_bitwise_op!(&, self, rhs)
    }
}

// 6.1.6.1.18 Number::bitwiseXOR ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-bitwiseXOR
impl ops::BitXor for Number {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        number_bitwise_op!(^, self, rhs)
    }
}

// 6.1.6.1.19 Number::bitwiseOR ( x, y )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-bitwiseOR
impl ops::BitOr for Number {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        number_bitwise_op!(|, self, rhs)
    }
}

// 6.1.6.1.20 Number::toString ( x, radix )
// https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-toString
// TODO

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Self(f64::from(value))
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<Number> for f64 {
    fn from(value: Number) -> Self {
        value.0
    }
}

impl JsFrom<f64> for Number {
    #[inline]
    fn from_js(value: f64) -> Self {
        Self(value)
    }
}

impl JsFrom<Number> for f64 {
    #[inline]
    fn from_js(value: Number) -> Self {
        value.0
    }
}

impl JsFrom<Number> for i32 {
    /// ## References
    /// - `ToInt32(x)`: <https://262.ecma-international.org/15.0/index.html#sec-toint32>
    /// - Rust casting behavior: <https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics>
    #[allow(clippy::cast_possible_truncation)]
    fn from_js(val: Number) -> Self {
        let two_32: f64 = f64::powf(2.0, 32.0);
        let two_31: f64 = f64::powf(2.0, 31.0);

        match val {
            // 2. If number is not finite or number is either +0𝔽 or -0𝔽, return +0𝔽.
            Number::ZERO | Number::INFINITY | Number::NEG_INFINITY => 0,
            _ => {
                // 3. Let int be truncate(ℝ(number)).
                let int = f64::trunc(val.into());
                // 4. Let int32bit be int modulo 2**32.
                let int32bit = int % two_32;
                // 5. If int32bit ≥ 2**31, return 𝔽(int32bit - 2**32);  otherwise return 𝔽(int32bit).

                if int32bit >= two_31 {
                    (int32bit - two_32) as i32
                } else {
                    int32bit as i32
                }
            }
        }
    }
}

impl JsFrom<Number> for u32 {
    /// ## References
    /// - `ToUint32(x)`: <https://262.ecma-international.org/15.0/index.html#sec-touint32>
    /// - Rust casting behavior: <https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics>
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn from_js(val: Number) -> Self {
        if val.is_infinite() || val == Number::ZERO {
            return 0;
        }
        let int = f64::trunc(val.into());
        let int32bit = int % f64::powf(2.0, 32.0);
        int32bit as u32
    }
}

impl hash::Hash for Number {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ops::*;

    const INFINITIES: [Number; 2] = [Number::INFINITY, Number::NEG_INFINITY];

    // NOTE: NAN is never equal to itself and cannot be used in assert_eq!. Use
    // is_nan() instead.

    // https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-unaryMinus
    #[test]
    fn test_number_unary_minus() {
        assert!(Number::NAN.neg().is_nan());
        assert_eq!(-Number::INFINITY, Number::NEG_INFINITY);
        assert_eq!(-Number::NEG_INFINITY, Number::INFINITY);
        assert_eq!(-Number::ZERO, Number::NEG_ZERO);
        assert_eq!(-Number::NEG_ZERO, Number::ZERO);
        assert_eq!(-Number::new(1.0), Number::new(-1.0));
    }

    // 6.1.6.1.4 Number::multiply ( x, y )
    // https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-multiply
    #[test]
    fn test_number_mul() {
        assert!((Number::NAN * Number::NAN).is_nan());
        assert!((Number::INFINITY * Number::ZERO).is_nan());
        assert!((Number::ZERO * Number::INFINITY).is_nan());
        assert_eq!(Number::INFINITY * Number::INFINITY, Number::INFINITY);
        assert_eq!(Number::ZERO * Number::new(1.0), Number::ZERO);
    }

    // 6.1.6.1.5 Number::divide ( x, y )
    #[test]
    fn test_number_div() {
        // 1. if x is NaN or y is NaN, return NaN
        assert!((Number::NAN / Number::NAN).is_nan());
        assert!((Number::ONE / Number::NAN).is_nan());
        assert!((Number::NAN / Number::ONE).is_nan());

        // 2. if x is +∞ or -∞,
        //   a. if y is either +∞ or -∞, return NAN
        //   b. if y is +0 or y > +0, return x
        //   c. Return -x
        for x in INFINITIES {
            assert!((x / Number::INFINITY).is_nan());
            assert!((x / Number::NEG_INFINITY).is_nan());
            assert_eq!(x / Number::ZERO, x, "{x:?} / Number::ZERO != {x:?}");
            assert_eq!(x / Number::ONE, x);
            assert_eq!(x / Number::new(-1.0), -x);
        }

        // 3. if y is +∞ or -∞, then
        //   a. If x is +0 or x > +0, return +0. Otherwise, return -0
        for y in INFINITIES {
            assert_eq!(Number::ZERO / y, Number::ZERO);
            assert_eq!(Number::ONE / y, Number::ZERO);
            assert_eq!(Number::NEG_ZERO / y, Number::NEG_ZERO);
            assert_eq!(Number::new(-1.0) / y, Number::NEG_ZERO);
        }

        assert_eq!(Number::INFINITY / Number::ZERO, Number::NAN);

        assert_eq!(Number::INFINITY / Number::ZERO, Number::NAN);
        assert_eq!(Number::ZERO / Number::INFINITY, Number::ZERO);
        assert_eq!(Number::INFINITY / Number::INFINITY, Number::NAN);
        assert_eq!(Number::ZERO / Number::new(1.0), Number::ZERO);
    }

    // 6.1.6.1.6 Number::remainder ( n, d )
    // https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-remainder
    #[test]
    fn test_number_rem() {
        // 1. If n is NaN or d is NaN, return NaN.
        assert!((Number::NAN % Number::NAN).is_nan());

        // 2. If n is either +∞ or -∞, return NaN.
        for inf in INFINITIES {
            assert!((inf % Number::NAN).is_nan());
            assert!((inf % Number::new(1.0)).is_nan());
            assert!((inf % Number::ZERO).is_nan());
            assert!((inf % Number::INFINITY).is_nan());
            assert!((inf % Number::NEG_INFINITY).is_nan());
        }
        // 3. if d is either +∞ or -∞, return n.
        for inf in INFINITIES {
            assert_eq!(Number::ZERO % inf, Number::ZERO);
            assert_eq!(Number::ONE % inf, Number::ONE);
            assert_eq!(Number::new(5.0) % inf, Number::new(5.0));
            assert_eq!(Number::new(-5.0) % inf, Number::new(-5.0));
        }
    }

    // 6.1.6.1.12 Number::lessThan(`x`, `y`)`
    #[test]
    fn test_number_less_than() {}

    // 6.1.6.1.13 Number::equal(`x`, `y`)
    #[test]
    fn test_number_eq() {
        // 1. If x is NaN, return false
        // 2. If y is NaN, return false
        assert_ne!(Number::NAN, Number::NAN);
        // nan on lhs
        assert_ne!(Number::NAN, Number::ONE);
        assert_ne!(Number::NAN, Number::ZERO);
        assert_ne!(Number::NAN, Number::INFINITY);
        assert_ne!(Number::NAN, Number::new(5.0));
        assert_ne!(Number::NAN, Number::new(-5.0));
        // nan on rhs
        assert_ne!(Number::ONE, Number::NAN);
        assert_ne!(Number::ZERO, Number::NAN);
        assert_ne!(Number::INFINITY, Number::NAN);
        assert_ne!(Number::new(5.0), Number::NAN);
        assert_ne!(Number::new(-5.0), Number::NAN);

        // 3. if x is y, return true
        assert_eq!(Number::ZERO, Number::ZERO);
        assert_eq!(Number::NEG_ZERO, Number::NEG_ZERO);
        assert_eq!(Number::ONE, Number::ONE);
        assert_eq!(Number::INFINITY, Number::INFINITY);
        assert_eq!(Number::NEG_INFINITY, Number::NEG_INFINITY);
        assert_eq!(Number::new(5.0), Number::new(5.0));
        assert_eq!(Number::new(-5.0), Number::new(-5.0));
        assert_eq!(Number::new(0.01), Number::new(0.01));

        // 4. if x is +_0 and y is -0, return true
        // 5. if x is -0 and y is +0, return true
        assert_eq!(Number::ZERO, Number::NEG_ZERO);
        assert_eq!(Number::NEG_ZERO, Number::ZERO);

        // 6. return false
        assert_ne!(Number::ZERO, Number::ONE);
        assert_ne!(Number::ZERO, Number::INFINITY);
        assert_ne!(Number::ZERO, Number::NEG_INFINITY);
        assert_ne!(Number::ZERO, Number::new(5.0));
        assert_ne!(Number::ZERO, Number::new(-5.0));

        assert_ne!(Number::ONE, Number::INFINITY);
        assert_ne!(Number::ONE, Number::NEG_INFINITY);
        assert_ne!(Number::ONE, Number::new(5.0));
        assert_ne!(Number::ONE, Number::new(-5.0));

        assert_ne!(Number::INFINITY, Number::NEG_INFINITY);
        assert_ne!(Number::INFINITY, Number::ZERO);
        assert_ne!(Number::INFINITY, Number::NEG_ZERO);
        assert_ne!(Number::INFINITY, Number::new(5.0));
        assert_ne!(Number::INFINITY, Number::new(-5.0));
    }
}
