#![expect(
    clippy::inline_always,
    reason = "thin wrappers, which should compile down to just the intrinsics they call"
)]

use std::arch::aarch64::*;

use crate::{
    cpu::Neon,
    simd::{
        arrays::{IntArray64, IntArray128},
        operand::Operand128,
    },
};

use super::Vec128;

/// Creation methods.
impl Vec128 {
    /// Create a [`Vec128`] wrapping a [`uint8x16_t`].
    ///
    /// Internal method only. `pub(super)` so `Vec256` methods can use it.
    #[inline(always)]
    pub(super) fn new(inner: uint8x16_t, cpu: Neon) -> Self {
        Self { inner, cpu }
    }

    /// Get [`Neon`] token from this [`Vec128`].
    #[inline(always)]
    pub fn cpu(self) -> Neon {
        self.cpu
    }

    /// Create a [`Vec128`] with all bits zero.
    #[inline(always)]
    pub fn zero(cpu: Neon) -> Self {
        Self::splat(0, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 16 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: Neon) -> Self {
        // SAFETY: `Neon` type guarantees platform supports Neon
        Self::new(unsafe { vdupq_n_u8(n) }, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 4 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: Neon) -> Self {
        // SAFETY: `Neon` type guarantees platform supports Neon
        Self::new(unsafe { vreinterpretq_u8_u32(vdupq_n_u32(n)) }, cpu)
    }
}

/// Loads and stores.
impl Vec128 {
    /// Read a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 16 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: Neon) -> Self {
        // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes.
        // `Neon` type guarantees platform supports Neon.
        Self::new(unsafe { vld1q_u8(ptr) }, cpu)
    }

    /// Read 8 bytes at pointer `ptr` into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 8 bytes.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn load_low_unaligned(ptr: *const u8, cpu: Neon) -> Self {
        // SAFETY: Caller guarantees `ptr` is valid for reading 8 bytes.
        // `Neon` type guarantees platform supports Neon.
        Self::new(unsafe { vcombine_u8(vld1_u8(ptr), vdup_n_u8(0)) }, cpu)
    }

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray128, cpu: Neon) -> Self {
        // SAFETY: `IntArray128` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 16 bytes
        unsafe { Self::load_unaligned(slice.as_ptr(), cpu) }
    }

    /// Read a slice of integers into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// `slice` can be any slice of integers which is 8 bytes in total
    /// (e.g. `&[u8; 8]`, `&[i32; 2]`).
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn read_low_unaligned(slice: &impl IntArray64, cpu: Neon) -> Self {
        // SAFETY: `IntArray64` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 8 bytes
        unsafe { Self::load_low_unaligned(slice.as_ptr(), cpu) }
    }

    /// Store a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 16 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        // This is all `vst1q_u8` does. Calling it would give the same instructions,
        // but it's a function which the compiler has to instantiate and then inline.
        // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes.
        unsafe { ptr.cast::<uint8x16_t>().write_unaligned(self.inner) }
    }

    /// Store the low 8 bytes of a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 8 bytes.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store_low_unaligned(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees `ptr` is valid for writing 8 bytes.
        // `Vec128` type guarantees platform supports Neon.
        unsafe { vst1_u8(ptr, vget_low_u8(self.inner)) }
    }
}

/// Bitwise ops.
impl Vec128 {
    /// Check if all the bits of this [`Vec128`] are zero.
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        // Neon has no equivalent of x86's `ptest`, so combine the 2 halves,
        // and test the result for 0
        (self.get_u64::<0>() | self.get_u64::<1>()) == 0
    }

    /// Invert all the bits of this [`Vec128`].
    #[inline(always)]
    pub fn not(self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vmvnq_u8(self.inner) }, self.cpu)
    }

    /// Shift each of the 16 bytes of this [`Vec128`] left by `BITS` bits.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn shl_u8<const BITS: i32>(self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vshlq_n_u8::<BITS>(self.inner) }, self.cpu)
    }

    /// Shift each of the 16 bytes of this [`Vec128`] right by `BITS` bits.
    #[inline(always)]
    pub fn shr_u8<const BITS: i32>(self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vshrq_n_u8::<BITS>(self.inner) }, self.cpu)
    }

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vandq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vorrq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { veorq_u8(self.inner, other.inner) }, self.cpu)
    }
}

/// Arithmetic ops.
impl Vec128 {
    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn add_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vaddq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vqaddq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vsubq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vqsubq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Add adjacent pairs of bytes in this [`Vec128`] and `other`, wrapping on overflow.
    ///
    /// The result is `[s0 + s1, s2 + s3, .., s14 + s15, o0 + o1, o2 + o3, .., o14 + o15]`,
    /// where `sN` and `oN` are the bytes at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn pairwise_add_u8(self, other: Self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vpaddq_u8(self.inner, other.inner) }, self.cpu)
    }
}

/// Comparison.
impl Vec128 {
    /// Compare every byte of this [`Vec128`] to either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    pub fn eq_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vceqq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand128<u8>) -> bool {
        self.xor(other).is_zero()
    }

    /// Check every byte of this [`Vec128`] against either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes share any set bits, otherwise `0x00`.
    #[inline(always)]
    pub fn test_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vtstq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    pub fn lt_i8(self, other: impl Operand128<i8>) -> Self {
        other.into_vec128(self.cpu).gt_i8(self)
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    pub fn gt_i8(self, other: impl Operand128<i8>) -> Self {
        let other = other.into_vec128(self.cpu);
        let (left, right) = (self.inner, other.inner);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        let result = unsafe { vcgtq_s8(vreinterpretq_s8_u8(left), vreinterpretq_s8_u8(right)) };
        Self::new(result, self.cpu)
    }
}

/// Bytewise ops.
impl Vec128 {
    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn shr_bytes<const BYTES: i32>(self) -> Self {
        let zero = Self::zero(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vextq_u8::<BYTES>(self.inner, zero.inner) }, self.cpu)
    }

    /// Rearrange the bytes of this [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// * If `control[i]` is 16 or more, `result[i]` is 0.
    /// * Otherwise, `result[i]` is `self[control[i]]`.
    ///
    /// Often used as a 16-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vqtbl1q_u8(self.inner, control.inner) }, self.cpu)
    }

    /// Combine the bits of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each bit of the result comes from `other` if the same bit of `mask` is set,
    /// otherwise from `self`.
    #[inline(always)]
    pub fn blend(self, other: impl Operand128<u8>, mask: Self) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vbslq_u8(mask.inner, other.inner, self.inner) }, self.cpu)
    }
}

/// `u64`-wise ops.
impl Vec128 {
    /// Get one of the 2 `u64`s of this [`Vec128`].
    ///
    /// `INDEX` is 0 for the low 8 bytes, or 1 for the high 8 bytes.
    #[inline(always)]
    pub fn get_u64<const INDEX: i32>(self) -> u64 {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        unsafe { vgetq_lane_u64::<INDEX>(vreinterpretq_u64_u8(self.inner)) }
    }
}
