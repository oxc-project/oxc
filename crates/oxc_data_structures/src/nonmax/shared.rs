// All methods are either zero cost, very cheap, or delegate, so marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    convert::TryFrom,
    error::Error,
    fmt::{self, Display},
};

use super::NonMaxU32;

impl NonMaxU32 {
    /// [`NonMaxU32`] with the value zero (0).
    pub const ZERO: Self = {
        // SAFETY: 0 is a valid value
        unsafe { Self::new_unchecked(0) }
    };

    /// [`NonMaxU32`] with the maximum value zero (`u32::MAX - 1`).
    pub const MAX: Self = {
        // SAFETY: `u32::MAX - 1` is a valid value
        unsafe { Self::new_unchecked(u32::MAX - 1) }
    };
}

impl Default for NonMaxU32 {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<NonMaxU32> for u32 {
    #[inline(always)]
    fn from(value: NonMaxU32) -> u32 {
        value.get()
    }
}

impl TryFrom<u32> for NonMaxU32 {
    type Error = TryFromU32Error;

    #[inline(always)]
    fn try_from(n: u32) -> Result<Self, TryFromU32Error> {
        // Note: Conversion from `Option` to `Result` here is zero-cost
        Self::new(n).ok_or(TryFromU32Error(()))
    }
}

macro_rules! impl_fmt {
    ($Trait:ident) => {
        impl fmt::$Trait for NonMaxU32 {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::$Trait::fmt(&self.get(), f)
            }
        }
    };
}

impl_fmt!(Debug);
impl_fmt!(Display);
impl_fmt!(Binary);
impl_fmt!(Octal);
impl_fmt!(LowerHex);
impl_fmt!(UpperHex);

/// Error type for failed conversion from [`u32`] to [`NonMaxU32`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TryFromU32Error(());

impl Error for TryFromU32Error {}

impl Display for TryFromU32Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        "Out of range conversion to `NonMaxU32` attempted".fmt(fmt)
    }
}
