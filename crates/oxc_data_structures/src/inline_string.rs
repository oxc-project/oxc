//! Fixed capacity string, stored on the stack.

use std::{
    fmt::{self, Display},
    ops::{Add, AddAssign, Deref},
};

use crate::assert_unchecked;

/// Short inline string.
///
/// `CAPACITY` determines the maximum length of the string.
///
/// `Len` type is the type used to store string length.
/// It can be `u8`, `u16`, `u32`, or `usize`. On 64-bit platforms, it can also be `u64`.
///
/// To make this type maximally efficient, select a `CAPACITY` value and `Len` type
/// which make the total byte size of `InlineString<CAPACITY, Len>` a multiple of 8.
///
/// Failure to do this results in the type containing padding bytes, which makes operations on it
/// more expensive. Invalid combinations of `CAPACITY` and `Len` will produce an error at compile time.
///
/// Generally, you want to use the largest `Len` type you can while allowing your required maximum
/// capacity, but without growing the size of the type. Then increase `CAPACITY` to fill any spare bytes.
///
/// Examples of valid `CAPACITY` / `Len` combinations:
///
/// * `FixedSizeString<7, u8>` = 8 bytes
/// * `FixedSizeString<8, usize>` = 16 bytes (on 64-bit platforms)
/// * `FixedSizeString<12, u32>` = 16 bytes
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InlineString<const CAPACITY: usize, Len: UnsignedInt> {
    bytes: [u8; CAPACITY],
    len: Len,
    // Aligned on 8 on 64-bit platforms, aligned on 4 on 32-bit platforms
    _align: [usize; 0],
}

impl<const CAPACITY: usize, Len: UnsignedInt> InlineString<CAPACITY, Len> {
    const ASSERTS: () = {
        assert!(CAPACITY <= Len::MAX_USIZE, "`CAPACITY` is too large for `Len`");
        assert!(
            (CAPACITY + size_of::<Len>()) % size_of::<usize>() == 0,
            "`CAPACITY + size_of::<Len>()` is not a multiple of `size_of::<usize>()`"
        );
    };

    /// Create an empty [`InlineString`].
    #[inline]
    pub fn new() -> Self {
        const { Self::ASSERTS };

        Self { bytes: [0; CAPACITY], len: Len::ZERO, _align: [] }
    }

    /// Create [`InlineString`] from `&str`.
    ///
    /// # Panics
    /// Panics if `s.len() > CAPACITY`.
    #[expect(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        const { Self::ASSERTS };

        let mut bytes = [0; CAPACITY];
        let slice = &mut bytes[..s.len()];
        slice.copy_from_slice(s.as_bytes());
        Self { bytes, len: Len::from_usize(s.len()), _align: [] }
    }

    /// Push a byte to the string.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// * String is already full to capacity.
    /// * `byte` is >= 128 (not an ASCII character).
    #[inline]
    pub fn push(&mut self, byte: u8) {
        assert!(self.len.to_usize() < CAPACITY);
        assert!(byte.is_ascii());

        // SAFETY: We just checked the safety constraints
        unsafe { self.push_unchecked(byte) }
    }

    /// Push a byte to the string, without checks.
    ///
    /// # SAFETY
    /// * Must not push more than `CAPACITY` bytes.
    /// * `byte` must be < 128 (an ASCII character).
    #[inline]
    pub unsafe fn push_unchecked(&mut self, byte: u8) {
        debug_assert!(self.len.to_usize() < CAPACITY);
        debug_assert!(byte.is_ascii());

        // SAFETY: Caller guarantees not pushing more than `CAPACITY` bytes, so `len` is in bounds
        unsafe { *self.bytes.get_unchecked_mut(self.len.to_usize()) = byte };
        self.len += Len::from_usize(1);
    }

    /// Get length of string as `Len`.
    #[inline]
    pub fn len(&self) -> Len {
        self.len
    }

    /// Get length of string as `usize`.
    #[inline]
    pub fn len_usize(&self) -> usize {
        self.len.to_usize()
    }

    /// Get if string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == Len::ZERO
    }

    /// Get string as `&str` slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: If safety conditions of `push_unchecked` have been upheld,
        // `self.len <= CAPACITY`, and contents of slice of `bytes` is a valid UTF-8 string
        unsafe {
            assert_unchecked!(self.len.to_usize() <= CAPACITY);
            let slice = &self.bytes[..self.len.to_usize()];
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Get string as `&mut str` slice.
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        // SAFETY: If safety conditions of `push_unchecked` have been upheld,
        // `self.len <= CAPACITY`, and contents of slice of `bytes` is a valid UTF-8 string
        unsafe {
            assert_unchecked!(self.len.to_usize() <= CAPACITY);
            let slice = &mut self.bytes[..self.len.to_usize()];
            std::str::from_utf8_unchecked_mut(slice)
        }
    }
}

impl<const CAPACITY: usize, Len: UnsignedInt> Default for InlineString<CAPACITY, Len> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAPACITY: usize, Len: UnsignedInt> Deref for InlineString<CAPACITY, Len> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAPACITY: usize, Len: UnsignedInt> Display for InlineString<CAPACITY, Len> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Trait for type which can be `len` field.
///
/// Private trait to prevent code outside this module implementing this trait on other types.
#[expect(private_bounds)]
pub trait UnsignedInt: Copy + PartialEq + Eq + Add + AddAssign + Sealed {
    /// Zero.
    const ZERO: Self;

    /// Maximum value for this type as a `usize`.
    const MAX_USIZE: usize;

    /// Convert to `usize`.
    fn to_usize(self) -> usize;

    /// Convert from `usize`.
    fn from_usize(n: usize) -> Self;
}

/// Trait which isn't public, and is a bound of `UnsignedInt`.
/// Prevents code outside this module implementing `UnsignedInt` on any other types.
trait Sealed {}

macro_rules! impl_unsigned_int {
    ($ty:ident) => {
        #[allow(clippy::cast_possible_truncation, clippy::allow_attributes)]
        impl UnsignedInt for $ty {
            const ZERO: Self = 0;

            const MAX_USIZE: usize = $ty::MAX as usize;

            #[inline(always)]
            fn to_usize(self) -> usize {
                self as usize
            }

            #[inline(always)]
            fn from_usize(n: usize) -> Self {
                n.try_into().unwrap()
            }
        }

        impl Sealed for $ty {}
    };
}

impl_unsigned_int!(u8);
impl_unsigned_int!(u16);
impl_unsigned_int!(u32);
// Only implement for `u64` on 64-bit systems so that conversion from this type to `usize`
// cannot truncate value
#[cfg(target_pointer_width = "64")]
impl_unsigned_int!(u64);
impl_unsigned_int!(usize);
