// Conversion methods are zero cost, and other methods just delegate, so marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use crate::assert_unchecked;

// # Implementation details
//
// `std::os::fd::BorrowedFd` is a wrapper around `std::os::fd::RawFd`, and is `#[repr(transparent)]`.
// `RawFd` is a type alias for `i32`. `i32` has same layout as `u32`.
// Therefore `BorrowedFd` has the same layout as `u32`.
// `NonZeroU32` is `#[repr(transparent)]`, and `ManuallyDrop` is also `#[repr(transparent)]`.
// Therefore `NonMaxU32` has same layout as `u32`.
//
// The feature that `BorrowedFd` brings is that it has a
// `#[rustc_layout_scalar_valid_range_end(0xFF_FF_FF_FE)]` attribute,
// meaning `u32::MAX` is an invalid bit pattern for the type. This invalid bit pattern forms a niche.
// <https://doc.rust-lang.org/stable/std/os/fd/struct.BorrowedFd.html>
//
// `BorrowedFd` and `OwnedFd` are the only public stable types in the standard library which exhibit
// this property. `OwnedFd` is not `Copy`, so we use `BorrowedFd`.
// We never use the `BorrowedFd` as an actual file descriptor, only utilize it for its layout.
//
// `BorrowedFd` is not `Drop`, but we wrap it in `ManuallyDrop` anyway, just to make sure.
//
// Because of the niche, `Option<NonMaxU32>` is 4 bytes.
//
// Unlike the `NonMaxU32` type from `nonmax` crate, this type has zero cost converting to and from `u32`.
// https://godbolt.org/z/cGaqhcco4
//
// `BorrowedFd` is only available on Unix-like platforms.
// We substitute a less efficient version on Windows, which does have a (small) conversion cost.

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    mem::{ManuallyDrop, transmute},
    os::fd::BorrowedFd,
};

/// [`NonZeroU32`] equivalent, except with the illegal value at the top of the range (`u32::MAX`).
///
/// [`NonMaxU32`] can represent any number from 0 to `u32::MAX - 1` inclusive.
///
/// `NonMaxU32` has a niche, so `Option<NonMaxU32>` is 4 bytes.
///
/// [`NonZeroU32`]: std::num::NonZeroU32
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NonMaxU32(ManuallyDrop<BorrowedFd<'static>>);

impl NonMaxU32 {
    /// Create [`NonMaxU32`] from [`u32`].
    ///
    /// Returns `None` if `n` is `u32::MAX`.
    #[inline(always)]
    pub const fn new(n: u32) -> Option<Self> {
        if n < u32::MAX {
            // SAFETY: We just checked `n < u32::MAX`
            Some(unsafe { Self::new_unchecked(n) })
        } else {
            None
        }
    }

    /// Create [`NonMaxU32`] from [`u32`], without checks.
    ///
    /// # SAFETY
    /// `n` must not be `u32::MAX`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(n: u32) -> Self {
        // SAFETY: Caller guarantees `n` is not `u32::MAX`.
        // See implementation details comment above.
        unsafe { transmute::<u32, Self>(n) }
    }

    /// Convert [`NonMaxU32`] to [`u32`].
    #[inline(always)]
    pub const fn get(self) -> u32 {
        // SAFETY: See implementation details comment above
        let n = unsafe { transmute::<Self, u32>(self) };

        // Make sure compiler understands return value cannot be `u32::MAX`.
        // This may aid it to make optimizations in some cases.
        // SAFETY: `NonMaxU32` cannot represent `u32::MAX`.
        unsafe { assert_unchecked!(n < u32::MAX) };

        n
    }
}

impl Eq for NonMaxU32 {}

impl PartialEq for NonMaxU32 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Ord for NonMaxU32 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl PartialOrd for NonMaxU32 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for NonMaxU32 {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

const _: () = {
    assert!(size_of::<BorrowedFd>() == 4);
    assert!(align_of::<BorrowedFd>() == 4);
    assert!(size_of::<NonMaxU32>() == 4);
    assert!(align_of::<NonMaxU32>() == 4);
    assert!(size_of::<u32>() == 4);
    assert!(align_of::<u32>() == 4);
    assert!(size_of::<Option<NonMaxU32>>() == 4);
    assert!(size_of::<Option<Option<NonMaxU32>>>() == 8);
};
