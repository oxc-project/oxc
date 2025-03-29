// All methods are very cheap, so marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{cmp::Ordering, num::NonZeroU32};

/// [`NonZeroU32`] equivalent, except with the illegal value at the top of the range (`u32::MAX`).
///
/// [`NonMaxU32`] can represent any number from 0 to `u32::MAX - 1` inclusive.
///
/// `NonMaxU32` has a niche, so `Option<NonMaxU32>` is 4 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NonMaxU32(NonZeroU32);

impl NonMaxU32 {
    /// Create [`NonMaxU32`] from [`u32`].
    ///
    /// Returns `None` if `n` is `u32::MAX`.
    #[inline(always)]
    pub const fn new(n: u32) -> Option<Self> {
        match NonZeroU32::new(n ^ u32::MAX) {
            Some(non_zero) => Some(Self(non_zero)),
            None => None,
        }
    }

    /// Create [`NonMaxU32`] from [`u32`], without checks.
    ///
    /// # SAFETY
    /// `n` must not be `u32::MAX`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(n: u32) -> Self {
        // SAFETY: Caller guarantees `n` is not `u32::MAX`.
        let non_zero = unsafe { NonZeroU32::new_unchecked(n ^ u32::MAX) };
        Self(non_zero)
    }

    /// Convert [`NonMaxU32`] to [`u32`].
    #[inline(always)]
    pub const fn get(self) -> u32 {
        self.0.get() ^ u32::MAX
    }
}

impl Ord for NonMaxU32 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl PartialOrd for NonMaxU32 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const _: () = {
    assert!(size_of::<NonMaxU32>() == 4);
    assert!(align_of::<NonMaxU32>() == 4);
    assert!(size_of::<u32>() == 4);
    assert!(align_of::<u32>() == 4);
    assert!(size_of::<Option<NonMaxU32>>() == 4);
    assert!(size_of::<Option<Option<NonMaxU32>>>() == 8);
};
