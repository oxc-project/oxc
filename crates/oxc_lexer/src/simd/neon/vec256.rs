//! Methods of `Vec256<Neon>`, which wraps a [`uint8x16x2_t`].
//!
//! Implements [`Vec256Impl`], and has the methods which only Neon has.
//!
//! Neon registers are 128 bits, so a `Vec256<Neon>` is a pair of registers,
//! and its bitwise methods work on each half separately.
//! It only has the few methods which Neon code needs, far fewer than `Vec256<Avx2>`.
//!
//! [`uint8x16x2_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16x2_t.html

#![expect(
    clippy::inline_always,
    reason = "thin wrappers, which should compile down to just the intrinsics they call"
)]

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::{
    cpu::{Neon, cpu_impl},
    simd::{Vec128, Vec256, Vec256Impl, VecData256, arrays::IntArray256, operand::Operand256},
};

/// Type which `Vec256<Neon>` wraps.
///
/// On platforms other than aarch64, `Vec256<Neon>` can never be created, and it wraps
/// a pair of [`VecData128`]s, for same size and alignment as `uint8x16x2_t`.
///
/// [`VecData128`]: crate::simd::VecData128
type Vec256Inner = cfg_select! {
    target_arch = "aarch64" => uint8x16x2_t,
    _ => [crate::simd::VecData128; 2],
};

#[cpu_impl(Neon)]
impl Vec256<Neon> {
    /// Create a [`Vec256`] wrapping a [`uint8x16x2_t`].
    ///
    /// Internal method only.
    ///
    /// [`uint8x16x2_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16x2_t.html
    #[inline(always)]
    fn new(inner: Vec256Inner, cpu: Neon) -> Self {
        Self { inner, cpu }
    }
}

#[cpu_impl(Neon)]
impl Vec256Impl for Vec256<Neon> {
    type Cpu = Neon;
    type Inner = Vec256Inner;

    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec256`] with all bits zero.
    #[inline(always)]
    fn zero(cpu: Neon) -> Self {
        Self::splat(0, cpu)
    }

    /// Create a [`Vec256`] by repeating `n` 32 times.
    #[inline(always)]
    fn splat(n: u8, cpu: Neon) -> Self {
        let half = Vec128::splat(n, cpu);
        Self::from_halves(half, half)
    }

    /// Create a [`Vec256`] by repeating `n` 8 times.
    #[inline(always)]
    fn splat_u32(n: u32, cpu: Neon) -> Self {
        let half = Vec128::splat_u32(n, cpu);
        Self::from_halves(half, half)
    }

    /// Create a [`Vec256`] by repeating `v` twice.
    #[inline(always)]
    fn splat_v128(v: Vec128<Neon>) -> Self {
        Self::from_halves(v, v)
    }

    /// Create a [`Vec256`] by zero-extending the low 8 bytes of `v` to 8 `u32`s.
    #[inline(always)]
    fn widen_u8_to_u32(v: Vec128<Neon>) -> Self {
        // SAFETY: `Vec128` type guarantees platform supports Neon
        let inner = unsafe {
            let u16s = vmovl_u8(vget_low_u8(v.inner));
            let low = vmovl_u16(vget_low_u16(u16s));
            let high = vmovl_u16(vget_high_u16(u16s));
            uint8x16x2_t(vreinterpretq_u8_u32(low), vreinterpretq_u8_u32(high))
        };
        Self::new(inner, v.cpu)
    }

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get [`Neon`] token from this [`Vec256`].
    #[inline(always)]
    fn cpu(self) -> Neon {
        self.cpu
    }

    // ----------------------------------------
    // Loads and stores
    // ----------------------------------------

    /// Read a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 32 bytes.
    /// * `ptr` must be aligned on 32.
    #[inline(always)]
    unsafe fn load(ptr: *const u8, cpu: Neon) -> Self {
        // Neon has no separate aligned load instruction, so this is the same as `load_unaligned`.
        // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
        unsafe { Self::load_unaligned(ptr, cpu) }
    }

    /// Read a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 32 bytes.
    #[inline(always)]
    unsafe fn load_unaligned(ptr: *const u8, cpu: Neon) -> Self {
        // This is all `vld1q_u8_x2` does. Calling it would give the same instructions,
        // but it's a function which the compiler has to instantiate and then inline.
        // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
        // `Neon` type guarantees platform supports Neon.
        Self::new(unsafe { ptr.cast::<uint8x16x2_t>().read_unaligned() }, cpu)
    }

    /// Read a [`Vec256`] from a [`VecData256`], which is aligned on 32.
    #[inline(always)]
    fn read(data: &VecData256, cpu: Neon) -> Self {
        // Neon has no separate aligned load instruction, so this is the same as `read_unaligned`
        Self::read_unaligned(data, cpu)
    }

    /// Read a [`Vec256`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 32 bytes in total
    /// (e.g. `&[u8; 32]`, `&[i64; 4]`).
    #[inline(always)]
    fn read_unaligned(slice: &impl IntArray256, cpu: Neon) -> Self {
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
    #[inline(always)]
    unsafe fn store(self, ptr: *mut u8) {
        // Neon has no separate aligned store instruction, so this is the same as `store_unaligned`.
        // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes.
        unsafe { self.store_unaligned(ptr) }
    }

    /// Store a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 32 bytes.
    #[inline(always)]
    unsafe fn store_unaligned(self, ptr: *mut u8) {
        // This is all `vst1q_u8_x2` does. Calling it would give the same instructions,
        // but it's a function which the compiler has to instantiate and then inline.
        // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes.
        unsafe { ptr.cast::<uint8x16x2_t>().write_unaligned(self.inner) }
    }

    /// Write a [`Vec256`] into a [`VecData256`], which is aligned on 32.
    #[inline(always)]
    fn write(self, dst: &mut VecData256) {
        // Neon has no separate aligned store instruction, so this is the same as `store_unaligned`.
        // SAFETY: `VecData256` guarantees `dst.as_mut_ptr()` is valid for writes of 32 bytes.
        unsafe { self.store_unaligned(dst.as_mut_ptr()) }
    }

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec256`] are zero.
    #[inline(always)]
    fn is_zero(self) -> bool {
        let (low, high) = self.into_halves();
        (low | high).is_zero()
    }

    /// Invert all the bits of this [`Vec256`].
    #[inline(always)]
    fn not(self) -> Self {
        let (low, high) = self.into_halves();
        Self::from_halves(!low, !high)
    }

    /// AND this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    fn and(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low & other_low, high & other_high)
    }

    /// OR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    fn or(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low | other_low, high | other_high)
    }

    /// XOR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    fn xor(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low ^ other_low, high ^ other_high)
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] left by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[inline(always)]
    fn shl_u16<const BITS: i32>(self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports Neon
        let inner = unsafe {
            let low = vshlq_n_u16::<BITS>(vreinterpretq_u16_u8(self.inner.0));
            let high = vshlq_n_u16::<BITS>(vreinterpretq_u16_u8(self.inner.1));
            uint8x16x2_t(vreinterpretq_u8_u16(low), vreinterpretq_u8_u16(high))
        };
        Self::new(inner, self.cpu)
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] right by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[inline(always)]
    fn shr_u16<const BITS: i32>(self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports Neon
        let inner = unsafe {
            let low = vshrq_n_u16::<BITS>(vreinterpretq_u16_u8(self.inner.0));
            let high = vshrq_n_u16::<BITS>(vreinterpretq_u16_u8(self.inner.1));
            uint8x16x2_t(vreinterpretq_u8_u16(low), vreinterpretq_u8_u16(high))
        };
        Self::new(inner, self.cpu)
    }

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[inline(always)]
    fn add_u8(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.add_u8(other_low), high.add_u8(other_high))
    }

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// stopping at 255.
    #[inline(always)]
    fn saturating_add_u8(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.saturating_add_u8(other_low), high.saturating_add_u8(other_high))
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[inline(always)]
    fn sub_u8(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.sub_u8(other_low), high.sub_u8(other_high))
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// stopping at 0.
    #[inline(always)]
    fn saturating_sub_u8(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.saturating_sub_u8(other_low), high.saturating_sub_u8(other_high))
    }

    /// Add either another [`Vec256`] or a `u32` to every `u32` of this [`Vec256`],
    /// wrapping on overflow.
    #[inline(always)]
    fn add_u32(self, other: impl Operand256<u32, Neon>) -> Self {
        let other = other.into_vec256(self.cpu);
        // SAFETY: `Vec256` type guarantees platform supports Neon
        let inner = unsafe {
            let low =
                vaddq_u32(vreinterpretq_u32_u8(self.inner.0), vreinterpretq_u32_u8(other.inner.0));
            let high =
                vaddq_u32(vreinterpretq_u32_u8(self.inner.1), vreinterpretq_u32_u8(other.inner.1));
            uint8x16x2_t(vreinterpretq_u8_u32(low), vreinterpretq_u8_u32(high))
        };
        Self::new(inner, self.cpu)
    }

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec256`] to either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    fn eq_u8(self, other: impl Operand256<u8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.eq_u8(other_low), high.eq_u8(other_high))
    }

    /// Check if every byte of this [`Vec256`] equals either another [`Vec256`] or a `u8`.
    #[inline(always)]
    fn all_eq_u8(self, other: impl Operand256<u8, Neon>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    fn lt_i8(self, other: impl Operand256<i8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.lt_i8(other_low), high.lt_i8(other_high))
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    fn gt_i8(self, other: impl Operand256<i8, Neon>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low.gt_i8(other_low), high.gt_i8(other_high))
    }

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Combine the bits of this [`Vec256`] and either another [`Vec256`] or a `u8`.
    ///
    /// Each bit of the result comes from `other` if the same bit of `mask` is set,
    /// otherwise from `self`.
    #[inline(always)]
    fn blend(self, other: impl Operand256<u8, Neon>, mask: Self) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        let (mask_low, mask_high) = mask.into_halves();
        Self::from_halves(low.blend(other_low, mask_low), high.blend(other_high, mask_high))
    }

    // ----------------------------------------
    // `u32`-wise ops
    // ----------------------------------------

    /// Interleave the low 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s0, o0, s1, o1, s4, o4, s5, o5]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    fn unpack_lo_u32(self, other: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports Neon
        let inner = unsafe {
            let low =
                vzip1q_u32(vreinterpretq_u32_u8(self.inner.0), vreinterpretq_u32_u8(other.inner.0));
            let high =
                vzip1q_u32(vreinterpretq_u32_u8(self.inner.1), vreinterpretq_u32_u8(other.inner.1));
            uint8x16x2_t(vreinterpretq_u8_u32(low), vreinterpretq_u8_u32(high))
        };
        Self::new(inner, self.cpu)
    }

    /// Interleave the high 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s2, o2, s3, o3, s6, o6, s7, o7]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    fn unpack_hi_u32(self, other: Self) -> Self {
        // SAFETY: `Vec256` type guarantees platform supports Neon
        let inner = unsafe {
            let low =
                vzip2q_u32(vreinterpretq_u32_u8(self.inner.0), vreinterpretq_u32_u8(other.inner.0));
            let high =
                vzip2q_u32(vreinterpretq_u32_u8(self.inner.1), vreinterpretq_u32_u8(other.inner.1));
            uint8x16x2_t(vreinterpretq_u8_u32(low), vreinterpretq_u8_u32(high))
        };
        Self::new(inner, self.cpu)
    }

    // ----------------------------------------
    // Operations on 128-bit halves
    // ----------------------------------------

    /// Create a [`Vec256`] from its low and high 128 bits.
    #[inline(always)]
    fn from_halves(low: Vec128<Neon>, high: Vec128<Neon>) -> Self {
        Self::new(uint8x16x2_t(low.inner, high.inner), low.cpu)
    }

    /// Get the low and high 128 bits of this [`Vec256`].
    #[inline(always)]
    fn into_halves(self) -> (Vec128<Neon>, Vec128<Neon>) {
        (Vec128::new(self.inner.0, self.cpu), Vec128::new(self.inner.1, self.cpu))
    }

    /// Get a [`Vec256`] made of the low 128 bits of this [`Vec256`],
    /// followed by the low 128 bits of `other`.
    #[inline(always)]
    fn join_lo_halves(self, other: Self) -> Self {
        Self::from_halves(self.into_halves().0, other.into_halves().0)
    }

    /// Get a [`Vec256`] made of the high 128 bits of this [`Vec256`],
    /// followed by the high 128 bits of `other`.
    #[inline(always)]
    fn join_hi_halves(self, other: Self) -> Self {
        Self::from_halves(self.into_halves().1, other.into_halves().1)
    }
}

/// Methods which are specific to this architecture.
#[cpu_impl(Neon)]
impl Vec256<Neon> {
    /// Rearrange the bytes of this [`Vec256`] into a [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// * If `control[i]` is 32 or more, `result[i]` is 0.
    /// * Otherwise, `result[i]` is `self[control[i]]`.
    ///
    /// Often used as a 32-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Vec128<Neon>) -> Vec128<Neon> {
        // SAFETY: `Vec256` type guarantees platform supports Neon
        Vec128::new(unsafe { vqtbl2q_u8(self.inner, control.inner) }, self.cpu)
    }
}
