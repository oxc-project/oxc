#![expect(
    clippy::inline_always,
    reason = "wrappers must inline to get the `target_feature`s of the function calling them"
)]

use std::arch::x86_64::*;

use crate::{
    cpu::Avx2,
    simd::{arrays::IntArray256, operand::Operand256},
};

use super::{Vec128, Vec256};

/// Creation methods.
impl Vec256 {
    /// Create a [`Vec256`] wrapping an [`__m256i`].
    ///
    /// Internal method only.
    #[inline(always)]
    fn new(inner: __m256i, cpu: Avx2) -> Self {
        Self { inner, cpu }
    }

    /// Get [`Avx2`] token from this [`Vec256`].
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn cpu(self) -> Avx2 {
        self.cpu
    }

    /// Create a [`Vec256`] with all bits zero.
    #[inline(always)]
    pub fn zero(cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_setzero_si256() }, cpu)
    }

    /// Create a [`Vec256`] by repeating `n` 32 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_set1_epi8(n.cast_signed()) }, cpu)
    }

    /// Create a [`Vec256`] by repeating `n` 8 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_set1_epi32(n.cast_signed()) }, cpu)
    }

    /// Create a [`Vec256`] by repeating `v` twice.
    #[inline(always)]
    pub fn splat_v128(v: Vec128) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_broadcastsi128_si256(v.inner) }, v.cpu)
    }

    /// Create a [`Vec256`] by zero-extending the low 8 bytes of `v` to 8 `u32`s.
    #[inline(always)]
    pub fn widen_u8_to_u32(v: Vec128) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_cvtepu8_epi32(v.inner) }, v.cpu)
    }
}

/// Loads and stores.
impl Vec256 {
    /// Read a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 32 bytes.
    /// * `ptr` must be aligned on 32.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn load(ptr: *const u8, cpu: Avx2) -> Self {
        #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
        let ptr = ptr.cast::<__m256i>();

        // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes, and aligned.
        // `Avx2` type guarantees platform supports AVX2.
        Self::new(unsafe { _mm256_load_si256(ptr) }, cpu)
    }

    /// Read a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 32 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: Avx2) -> Self {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m256i>();

        // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
        // `Avx2` type guarantees platform supports AVX2.
        Self::new(unsafe { _mm256_loadu_si256(ptr) }, cpu)
    }

    /// Read a [`Vec256`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 32 bytes in total
    /// (e.g. `&[u8; 32]`, `&[i64; 4]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray256, cpu: Avx2) -> Self {
        // SAFETY: `IntArray256` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 32 bytes
        unsafe { Self::load_unaligned(slice.as_ptr(), cpu) }
    }

    /// Store a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for writing 32 bytes.
    /// * `ptr` must be aligned on 32.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store(self, ptr: *mut u8) {
        #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
        let ptr = ptr.cast::<__m256i>();

        // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes, and aligned.
        // `Vec256` type guarantees platform supports AVX2.
        unsafe { _mm256_store_si256(ptr, self.inner) }
    }

    /// Store a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 32 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m256i>();

        // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes.
        // `Vec256` type guarantees platform supports AVX2.
        unsafe { _mm256_storeu_si256(ptr, self.inner) }
    }
}

/// Bitwise ops.
impl Vec256 {
    /// Gather the top bit of each byte of this [`Vec256`] into a 32-bit mask.
    ///
    /// Bit N of the result is the top bit of byte N.
    #[inline(always)]
    pub fn movemask(self) -> u32 {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        unsafe { _mm256_movemask_epi8(self.inner).cast_unsigned() }
    }

    /// Check if all the bits of this [`Vec256`] are zero.
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        unsafe { _mm256_testz_si256(self.inner, self.inner) != 0 }
    }

    /// Invert all the bits of this [`Vec256`].
    #[inline(always)]
    pub fn not(self) -> Self {
        self.xor(Self::splat(0xFF, self.cpu))
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] left by `BITS` bits.
    #[inline(always)]
    pub fn shl_u16<const BITS: i32>(self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_slli_epi16::<BITS>(self.inner) }, self.cpu)
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] right by `BITS` bits.
    #[inline(always)]
    pub fn shr_u16<const BITS: i32>(self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_srli_epi16::<BITS>(self.inner) }, self.cpu)
    }

    /// AND this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_and_si256(self.inner, other.inner) }, self.cpu)
    }

    /// OR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_or_si256(self.inner, other.inner) }, self.cpu)
    }

    /// XOR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_xor_si256(self.inner, other.inner) }, self.cpu)
    }
}

/// Arithmetic ops.
impl Vec256 {
    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn add_u8(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_add_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// stopping at 255.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_adds_epu8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_sub_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// stopping at 0.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_subs_epu8(self.inner, other.inner) }, self.cpu)
    }

    /// Add either another [`Vec256`] or a `u32` to every `u32` of this [`Vec256`],
    /// wrapping on overflow.
    #[inline(always)]
    pub fn add_u32(self, other: impl Operand256<u32>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_add_epi32(self.inner, other.inner) }, self.cpu)
    }
}

/// Comparison.
impl Vec256 {
    /// Compare every byte of this [`Vec256`] to either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    pub fn eq_u8(self, other: impl Operand256<u8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_cmpeq_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Check if every byte of this [`Vec256`] equals either another [`Vec256`] or a `u8`.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand256<u8>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    pub fn lt_i8(self, other: impl Operand256<i8>) -> Self {
        other.into_vec256(self.cpu).gt_i8(self)
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    pub fn gt_i8(self, other: impl Operand256<i8>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_cmpgt_epi8(self.inner, other.inner) }, self.cpu)
    }
}

/// Bytewise ops.
impl Vec256 {
    /// Rearrange the bytes of this [`Vec256`].
    ///
    /// The low and high 128 bits are treated as 2 separate 16-byte vectors,
    /// so a byte can only move within its own half.
    ///
    /// In each half, for each byte `i`:
    ///
    /// * If the top bit of `control[i]` is set, `result[i]` is 0.
    /// * Otherwise, `result[i]` is `self[control[i] & 0x0F]`.
    ///
    /// Often used as a 16-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_shuffle_epi8(self.inner, control.inner) }, self.cpu)
    }

    /// Combine the bytes of this [`Vec256`] and either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the top bit of the same byte of `mask` is set,
    /// otherwise from `self`. Other bits of `mask` are ignored.
    #[inline(always)]
    pub fn blend(self, other: impl Operand256<u8>, mask: Self) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_blendv_epi8(self.inner, other.inner, mask.inner) }, self.cpu)
    }
}

/// `u32`-wise ops.
impl Vec256 {
    /// Rearrange the 8 `u32`s of this [`Vec256`].
    ///
    /// For each `u32` `i`, `result[i]` is `self[indices[i] & 7]`.
    ///
    /// Unlike [`shuffle_u8s`], values can move between the low and high 128 bits.
    ///
    /// [`shuffle_u8s`]: Self::shuffle_u8s
    #[inline(always)]
    pub fn shuffle_u32s(self, indices: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_permutevar8x32_epi32(self.inner, indices.inner) }, self.cpu)
    }

    /// Interleave the low 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s0, o0, s1, o1, s4, o4, s5, o5]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn unpack_lo_u32(self, other: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_unpacklo_epi32(self.inner, other.inner) }, self.cpu)
    }

    /// Interleave the high 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s2, o2, s3, o3, s6, o6, s7, o7]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn unpack_hi_u32(self, other: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_unpackhi_epi32(self.inner, other.inner) }, self.cpu)
    }
}

/// Operations on 128-bit halves.
impl Vec256 {
    /// Get a [`Vec256`] made of the low 128 bits of this [`Vec256`],
    /// followed by the low 128 bits of `other`.
    #[inline(always)]
    pub fn join_lo_halves(self, other: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_permute2x128_si256::<0x20>(self.inner, other.inner) }, self.cpu)
    }

    /// Get a [`Vec256`] made of the high 128 bits of this [`Vec256`],
    /// followed by the high 128 bits of `other`.
    #[inline(always)]
    pub fn join_hi_halves(self, other: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports AVX2
        Self::new(unsafe { _mm256_permute2x128_si256::<0x31>(self.inner, other.inner) }, self.cpu)
    }
}
