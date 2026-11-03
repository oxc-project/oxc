//! Methods of `Vec128<Neon>`, which wraps a [`uint8x16_t`].
//!
//! Implements [`Vec128Impl`], and has the methods which only Neon has.
//! The `neon` module docs list the methods which behave differently from AVX2.
//!
//! [`uint8x16_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16_t.html

#![expect(
    clippy::inline_always,
    reason = "thin wrappers, which should compile down to just the intrinsics they call"
)]

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::{
    cpu::{Neon, cpu_impl},
    simd::{
        Vec128, Vec128Impl, VecData128,
        arrays::{IntArray64, IntArray128},
        operand::Operand128,
    },
};

/// Type which `Vec128<Neon>` wraps.
///
/// On platforms other than aarch64, `Vec128<Neon>` can never be created,
/// and it wraps a [`VecData128`], for same size and alignment as `uint8x16_t`.
type Vec128Inner = cfg_select! {
    target_arch = "aarch64" => uint8x16_t,
    _ => VecData128,
};

#[cpu_impl(Neon)]
impl Vec128<Neon> {
    /// Create a [`Vec128`] wrapping a [`uint8x16_t`].
    ///
    /// Internal method only. `pub(super)` so `Vec256` methods can use it.
    ///
    /// [`uint8x16_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16_t.html
    #[inline(always)]
    pub(super) fn new(inner: Vec128Inner, cpu: Neon) -> Self {
        Self { inner, cpu }
    }
}

#[cpu_impl(Neon)]
impl Vec128Impl for Vec128<Neon> {
    type Cpu = Neon;
    type Inner = Vec128Inner;

    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec128`] with all bits zero.
    #[inline(always)]
    fn zero(cpu: Neon) -> Self {
        Self::splat(0, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 16 times.
    #[inline(always)]
    fn splat(n: u8, cpu: Neon) -> Self {
        // SAFETY: `Neon` type guarantees platform supports Neon
        Self::new(unsafe { vdupq_n_u8(n) }, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 4 times.
    #[inline(always)]
    fn splat_u32(n: u32, cpu: Neon) -> Self {
        // SAFETY: `Neon` type guarantees platform supports Neon
        Self::new(unsafe { vreinterpretq_u8_u32(vdupq_n_u32(n)) }, cpu)
    }

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get [`Neon`] token from this [`Vec128`].
    #[inline(always)]
    fn cpu(self) -> Neon {
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
    unsafe fn load(ptr: *const u8, cpu: Neon) -> Self {
        // Neon has no separate aligned load instruction, so this is the same as `load_unaligned`.
        // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes.
        unsafe { Self::load_unaligned(ptr, cpu) }
    }

    /// Read a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 16 bytes.
    #[inline(always)]
    unsafe fn load_unaligned(ptr: *const u8, cpu: Neon) -> Self {
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
    #[inline(always)]
    unsafe fn load_low_unaligned(ptr: *const u8, cpu: Neon) -> Self {
        // SAFETY: Caller guarantees `ptr` is valid for reading 8 bytes.
        // `Neon` type guarantees platform supports Neon.
        Self::new(unsafe { vcombine_u8(vld1_u8(ptr), vdup_n_u8(0)) }, cpu)
    }

    /// Read a [`Vec128`] from a [`VecData128`], which is aligned on 16.
    #[inline(always)]
    fn read(data: &VecData128, cpu: Neon) -> Self {
        // Neon has no separate aligned load instruction, so this is the same as `read_unaligned`
        Self::read_unaligned(data, cpu)
    }

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    #[inline(always)]
    fn read_unaligned(slice: &impl IntArray128, cpu: Neon) -> Self {
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
    fn read_low_unaligned(slice: &impl IntArray64, cpu: Neon) -> Self {
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
        // Neon has no separate aligned store instruction, so this is the same as `store_unaligned`.
        // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes.
        unsafe { self.store_unaligned(ptr) }
    }

    /// Store a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 16 bytes.
    #[inline(always)]
    unsafe fn store_unaligned(self, ptr: *mut u8) {
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
    #[inline(always)]
    unsafe fn store_low_unaligned(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees `ptr` is valid for writing 8 bytes.
        // `Vec128` type guarantees platform supports Neon.
        unsafe { vst1_u8(ptr, vget_low_u8(self.inner)) }
    }

    /// Write a [`Vec128`] into a [`VecData128`], which is aligned on 16.
    ///
    /// Neon has no separate aligned store instruction, so this does the same as [`store_unaligned`].
    ///
    /// [`store_unaligned`]: Self::store_unaligned
    #[inline(always)]
    fn write(self, dst: &mut VecData128) {
        // SAFETY: `VecData128` guarantees `dst.as_mut_ptr()` is valid for writes of 16 bytes
        unsafe { self.store_unaligned(dst.as_mut_ptr()) }
    }

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec128`] are zero.
    #[inline(always)]
    fn is_zero(self) -> bool {
        // Neon has no equivalent of x86's `ptest`, so combine the 2 halves,
        // and test the result for 0
        (self.get_u64::<0>() | self.get_u64::<1>()) == 0
    }

    /// Invert all the bits of this [`Vec128`].
    #[inline(always)]
    fn not(self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vmvnq_u8(self.inner) }, self.cpu)
    }

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn and(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vandq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn or(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vorrq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn xor(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { veorq_u8(self.inner, other.inner) }, self.cpu)
    }

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    fn add_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vaddq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    #[inline(always)]
    fn saturating_add_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vqaddq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    fn sub_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vsubq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    #[inline(always)]
    fn saturating_sub_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vqsubq_u8(self.inner, other.inner) }, self.cpu)
    }

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec128`] to either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    fn eq_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vceqq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    #[inline(always)]
    fn all_eq_u8(self, other: impl Operand128<u8, Neon>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    fn lt_i8(self, other: impl Operand128<i8, Neon>) -> Self {
        other.into_vec128(self.cpu).gt_i8(self)
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    fn gt_i8(self, other: impl Operand128<i8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        let (left, right) = (self.inner, other.inner);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        let result = unsafe { vcgtq_s8(vreinterpretq_s8_u8(left), vreinterpretq_s8_u8(right)) };
        Self::new(result, self.cpu)
    }

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    #[inline(always)]
    fn shr_bytes<const BYTES: i32>(self) -> Self {
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
    fn shuffle_u8s(self, control: Self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vqtbl1q_u8(self.inner, control.inner) }, self.cpu)
    }

    /// Combine the bits of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each bit of the result comes from `other` if the same bit of `mask` is set,
    /// otherwise from `self`.
    #[inline(always)]
    fn blend(self, other: impl Operand128<u8, Neon>, mask: Self) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vbslq_u8(mask.inner, other.inner, self.inner) }, self.cpu)
    }

    // ----------------------------------------
    // `u64`-wise ops
    // ----------------------------------------

    /// Get one of the 2 `u64`s of this [`Vec128`].
    ///
    /// `INDEX` is 0 for the low 8 bytes, or 1 for the high 8 bytes.
    #[inline(always)]
    fn get_u64<const INDEX: i32>(self) -> u64 {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        unsafe { vgetq_lane_u64::<INDEX>(vreinterpretq_u64_u8(self.inner)) }
    }
}

/// Methods which are specific to this architecture.
#[cpu_impl(Neon)]
impl Vec128<Neon> {
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

    /// Add adjacent pairs of bytes in this [`Vec128`] and `other`, wrapping on overflow.
    ///
    /// The result is `[s0 + s1, s2 + s3, .., s14 + s15, o0 + o1, o2 + o3, .., o14 + o15]`,
    /// where `sN` and `oN` are the bytes at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn pairwise_add_u8(self, other: Self) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vpaddq_u8(self.inner, other.inner) }, self.cpu)
    }

    /// Check every byte of this [`Vec128`] against either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes share any set bits, otherwise `0x00`.
    #[inline(always)]
    pub fn test_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        let other = other.into_vec128(self.cpu);
        // SAFETY: `Vec128` type guarantees platform supports Neon
        Self::new(unsafe { vtstq_u8(self.inner, other.inner) }, self.cpu)
    }
}
