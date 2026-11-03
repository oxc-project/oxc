//! Methods of `Vec128<Avx2>`, which wraps an [`__m128i`].
//!
//! Implements [`Vec128Impl`], and has the methods which only AVX2 has.
//!
//! These are 128-bit operations, so the intrinsics are from SSE2 to SSE4.1,
//! which every CPU with AVX2 supports.
//!
//! [`__m128i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m128i.html

#![expect(
    clippy::inline_always,
    reason = "wrappers must inline to get the `target_feature`s of the function calling them"
)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::{
    cpu::{Avx2, cpu_impl},
    simd::{
        Vec128, Vec128Impl, VecData128,
        arrays::{IntArray64, IntArray128},
        operand::Operand128,
    },
};

/// Type which `Vec128<Avx2>` wraps.
///
/// On platforms other than x86_64, `Vec128<Avx2>` can never be created,
/// and it wraps a [`VecData128`], for same size and alignment as `__m128i`.
type Vec128Inner = cfg_select! {
    target_arch = "x86_64" => __m128i,
    _ => VecData128,
};

#[cpu_impl(Avx2)]
impl Vec128<Avx2> {
    /// Create a [`Vec128`] wrapping an [`__m128i`].
    ///
    /// Internal method only. `pub(super)` so `Vec256` methods can use it.
    ///
    /// [`__m128i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m128i.html
    #[inline(always)]
    pub(super) fn new(inner: Vec128Inner, cpu: Avx2) -> Self {
        Self { inner, cpu }
    }
}

#[cpu_impl(Avx2)]
impl Vec128Impl for Vec128<Avx2> {
    type Cpu = Avx2;
    type Inner = Vec128Inner;

    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec128`] with all bits zero.
    #[inline(always)]
    fn zero(cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_setzero_si128() }, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 16 times.
    #[inline(always)]
    fn splat(n: u8, cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_set1_epi8(n.cast_signed()) }, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 4 times.
    #[inline(always)]
    fn splat_u32(n: u32, cpu: Avx2) -> Self {
        // SAFETY: `Avx2` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_set1_epi32(n.cast_signed()) }, cpu)
    }

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get [`Avx2`] token from this [`Vec128`].
    #[inline(always)]
    fn cpu(self) -> Avx2 {
        self.cpu
    }

    // ----------------------------------------
    // Loads and stores
    // ----------------------------------------

    /// Read a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 16 bytes.
    /// * `ptr` must be aligned on 16.
    #[inline(always)]
    unsafe fn load(ptr: *const u8, cpu: Avx2) -> Self {
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
    unsafe fn load_unaligned(ptr: *const u8, cpu: Avx2) -> Self {
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
    unsafe fn load_low_unaligned(ptr: *const u8, cpu: Avx2) -> Self {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for reading 8 bytes.
        // `Avx2` type guarantees platform supports AVX2, which includes SSE2.
        Self::new(unsafe { _mm_loadl_epi64(ptr) }, cpu)
    }

    /// Read a [`Vec128`] from a [`VecData128`], which is aligned on 16.
    #[inline(always)]
    fn read(data: &VecData128, cpu: Avx2) -> Self {
        // SAFETY: `VecData128` guarantees `data.as_ptr()`
        // is valid for aligned reads of 16 bytes
        unsafe { Self::load(data.as_ptr(), cpu) }
    }

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    #[inline(always)]
    fn read_unaligned(slice: &impl IntArray128, cpu: Avx2) -> Self {
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
    fn read_low_unaligned(slice: &impl IntArray64, cpu: Avx2) -> Self {
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
    #[inline(always)]
    unsafe fn store(self, ptr: *mut u8) {
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
    unsafe fn store_unaligned(self, ptr: *mut u8) {
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
    unsafe fn store_low_unaligned(self, ptr: *mut u8) {
        #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't require alignment")]
        let ptr = ptr.cast::<__m128i>();

        // SAFETY: Caller guarantees `ptr` is valid for writing 8 bytes.
        // `Vec128` type guarantees platform supports AVX2, which includes SSE2.
        unsafe { _mm_storel_epi64(ptr, self.inner) }
    }

    /// Write a [`Vec128`] into a [`VecData128`], which is aligned on 16.
    #[inline(always)]
    fn write(self, dst: &mut VecData128) {
        // SAFETY: `VecData128` guarantees `dst.as_mut_ptr()`
        // is valid for aligned writes of 16 bytes
        unsafe { self.store(dst.as_mut_ptr()) }
    }

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec128`] are zero.
    #[inline(always)]
    fn is_zero(self) -> bool {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE4.1
        unsafe { _mm_testz_si128(self.inner, self.inner) != 0 }
    }

    /// Invert all the bits of this [`Vec128`].
    #[inline(always)]
    fn not(self) -> Self {
        self.xor(Self::splat(0xFF, self.cpu))
    }

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn and(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_and_si128(self.inner, other.inner) }, self.cpu)
    }

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn or(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_or_si128(self.inner, other.inner) }, self.cpu)
    }

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn xor(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_xor_si128(self.inner, other.inner) }, self.cpu)
    }

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    fn add_u8(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_add_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    #[inline(always)]
    fn saturating_add_u8(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_adds_epu8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    fn sub_u8(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_sub_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    #[inline(always)]
    fn saturating_sub_u8(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_subs_epu8(self.inner, other.inner) }, self.cpu)
    }

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec128`] to either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    fn eq_u8(self, other: impl Operand128<u8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_cmpeq_epi8(self.inner, other.inner) }, self.cpu)
    }

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn all_eq_u8(self, other: impl Operand128<u8, Avx2>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    fn lt_i8(self, other: impl Operand128<i8, Avx2>) -> Self {
        other.into_vec128(self.cpu).gt_i8(self)
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    fn gt_i8(self, other: impl Operand128<i8, Avx2>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        Self::new(unsafe { _mm_cmpgt_epi8(self.inner, other.inner) }, self.cpu)
    }

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    #[inline(always)]
    fn shr_bytes<const BYTES: i32>(self) -> Self {
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
    fn shuffle_u8s(self, control: Self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSSE3
        Self::new(unsafe { _mm_shuffle_epi8(self.inner, control.inner) }, self.cpu)
    }

    /// Combine the bytes of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the top bit of the same byte of `mask` is set,
    /// otherwise from `self`. Other bits of `mask` are ignored.
    #[inline(always)]
    fn blend(self, other: impl Operand128<u8, Avx2>, mask: Self) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE4.1
        Self::new(unsafe { _mm_blendv_epi8(self.inner, other.inner, mask.inner) }, self.cpu)
    }

    // ----------------------------------------
    // `u64`-wise ops
    // ----------------------------------------

    /// Get one of the 2 `u64`s of this [`Vec128`].
    ///
    /// `INDEX` is 0 for the low 8 bytes, or 1 for the high 8 bytes.
    #[inline(always)]
    fn get_u64<const INDEX: i32>(self) -> u64 {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE4.1
        unsafe { _mm_extract_epi64::<INDEX>(self.inner).cast_unsigned() }
    }
}

/// Methods which are specific to this architecture.
#[cpu_impl(Avx2)]
impl Vec128<Avx2> {
    /// Gather the top bit of each byte of this [`Vec128`] into a 16-bit mask.
    ///
    /// Bit N of the result is the top bit of byte N. Bits 16-31 of the result are zero.
    #[inline(always)]
    pub fn movemask(self) -> u32 {
        // SAFETY: `Vec128` type guarantees platform supports AVX2, which includes SSE2
        unsafe { _mm_movemask_epi8(self.inner).cast_unsigned() }
    }
}
