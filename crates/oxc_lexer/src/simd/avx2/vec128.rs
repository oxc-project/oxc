#![expect(
    clippy::inline_always,
    reason = "wrappers must inline to get the `target_feature`s of the function calling them"
)]

use std::arch::x86_64::*;

use crate::{
    cpu::Avx2,
    simd::{
        arrays::{IntArray64, IntArray128},
        operand::Operand128,
    },
};

use super::Vec128;

/// Creation methods.
impl Vec128 {
    /// Create a [`Vec128`] wrapping an [`__m128i`].
    ///
    /// Internal method only.
    #[inline(always)]
    fn new(inner: __m128i, cpu: Avx2) -> Self {
        Self { inner, cpu }
    }

    /// Get [`Avx2`] token from this [`Vec128`].
    #[expect(dead_code)]
    #[inline(always)]
    pub fn cpu(self) -> Avx2 {
        self.cpu
    }

    /// Create a [`Vec128`] with all bits zero.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn zero(cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_setzero_si128() }, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 16 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_set1_epi8(n.cast_signed()) }, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 4 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_set1_epi32(n.cast_signed()) }, cpu)
    }
}

/// Loads and stores.
impl Vec128 {
    /// Read a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 16 bytes.
    /// * `ptr` must be aligned on 16.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn load(ptr: *const u8, cpu: Avx2) -> Self {
        #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes, and aligned.
        // `Avx2` type guarantees platform supports AVX2, which includes SSE2.
        Self::new(unsafe { _mm_load_si128(ptr) }, cpu)
    }

    /// Read a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 16 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: Avx2) -> Self {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes.
        // `Avx2` type guarantees platform supports AVX2, which includes SSE2.
        Self::new(unsafe { _mm_loadu_si128(ptr) }, cpu)
    }

    /// Read 8 bytes at pointer `ptr` into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 8 bytes.
    #[inline(always)]
    pub unsafe fn load_low_unaligned(ptr: *const u8, cpu: Avx2) -> Self {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for reading 8 bytes.
        // `Avx2` type guarantees platform supports AVX2, which includes SSE2.
        Self::new(unsafe { _mm_loadl_epi64(ptr) }, cpu)
    }

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray128, cpu: Avx2) -> Self {
        // SAFETY: `IntArray128` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 16 bytes
        unsafe { Self::load_unaligned(slice.as_ptr(), cpu) }
    }

    /// Read a slice of integers into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// `slice` can be any slice of integers which is 8 bytes in total
    /// (e.g. `&[u8; 8]`, `&[i32; 2]`).
    #[inline(always)]
    pub fn read_low_unaligned(slice: &impl IntArray64, cpu: Avx2) -> Self {
        // SAFETY: `IntArray64` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 8 bytes
        unsafe { Self::load_low_unaligned(slice.as_ptr(), cpu) }
    }

    /// Store a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for writing 16 bytes.
    /// * `ptr` must be aligned on 16.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store(self, ptr: *mut u8) {
        #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes, and aligned.
        // `Vec128` type guarantees platform supports AVX2, which includes SSE2.
        unsafe { _mm_store_si128(ptr, self.inner) }
    }

    /// Store a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 16 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes.
        // `Vec128` type guarantees platform supports AVX2, which includes SSE2.
        unsafe { _mm_storeu_si128(ptr, self.inner) }
    }

    /// Store the low 8 bytes of a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 8 bytes.
    #[inline(always)]
    pub unsafe fn store_low_unaligned(self, ptr: *mut u8) {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for writing 8 bytes.
        // `Vec128` type guarantees platform supports AVX2, which includes SSE2.
        unsafe { _mm_storel_epi64(ptr, self.inner) }
    }
}

/// Bitwise ops.
impl Vec128 {
    /// Gather the top bit of each byte of this [`Vec128`] into a 16-bit mask.
    ///
    /// Bit N of the result is the top bit of byte N. Bits 16-31 of the result are zero.
    #[inline(always)]
    pub fn movemask(self) -> u32 {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        unsafe { _mm_movemask_epi8(self.inner).cast_unsigned() }
    }

    /// Check if all the bits of this [`Vec128`] are zero.
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE4.1
        unsafe { _mm_testz_si128(self.inner, self.inner) != 0 }
    }

    /// Invert all the bits of this [`Vec128`].
    #[inline(always)]
    pub fn not(self) -> Self {
        self.xor(Self::splat(0xFF, self.cpu))
    }

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_and_si128(self.inner, other.inner) }, self.cpu)
    }

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_or_si128(self.inner, other.inner) }, self.cpu)
    }

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_xor_si128(self.inner, other.inner) }, self.cpu)
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
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_add_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_adds_epu8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_sub_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand128<u8>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_subs_epu8(self.inner, other.inner) }, self.cpu)
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
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_cmpeq_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand128<u8>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[cfg_attr(not(test), expect(dead_code))]
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
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_cmpgt_epi8(self.inner, other.inner) }, self.cpu)
    }
}

/// Bytewise ops.
impl Vec128 {
    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    #[inline(always)]
    pub fn shr_bytes<const BYTES: i32>(self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_srli_si128::<BYTES>(self.inner) }, self.cpu)
    }

    /// Rearrange the bytes of this [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// * If the top bit of `control[i]` is set, `result[i]` is 0.
    /// * Otherwise, `result[i]` is `self[control[i] & 0x0F]`.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSSE3
        Self::new(unsafe { _mm_shuffle_epi8(self.inner, control.inner) }, self.cpu)
    }

    /// Combine the bytes of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the top bit of the same byte of `mask` is set,
    /// otherwise from `self`. Other bits of `mask` are ignored.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn blend(self, other: impl Operand128<u8>, mask: Self) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE4.1
        Self::new(unsafe { _mm_blendv_epi8(self.inner, other.inner, mask.inner) }, self.cpu)
    }
}
