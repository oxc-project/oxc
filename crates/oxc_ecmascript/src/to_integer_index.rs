use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

/// Convert a value to u32 if the value can be an integer index and can be represented with u32.
///
/// Note that the max value of u32 is `2^32 - 1`, which is smaller than the max value of
/// safe integer `2^53 - 1`.
///
/// If the value is a non-negative safe integer, it can be an integer index.
/// <https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-object-type>
pub trait ToIntegerIndex {
    fn to_integer_index(self) -> Option<u32>;
}

impl ToIntegerIndex for f64 {
    fn to_integer_index(self) -> Option<u32> {
        if self.fract() != 0.0 || self < 0.0 {
            return None;
        }
        self.to_u32()
    }
}

impl ToIntegerIndex for BigInt {
    fn to_integer_index(self) -> Option<u32> {
        if self < BigInt::zero() {
            return None;
        }
        self.to_u32()
    }
}
