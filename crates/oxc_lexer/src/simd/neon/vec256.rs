#![expect(
    clippy::inline_always,
    reason = "thin wrappers, which should compile down to just the intrinsics they call"
)]

use std::arch::aarch64::*;

use crate::{
    cpu::Neon,
    simd::{arrays::IntArray256, operand::Operand256},
};

use super::{Vec128, Vec256};

/// Creation methods.
impl Vec256 {
    /// Create a [`Vec256`] wrapping a [`uint8x16x2_t`].
    ///
    /// Internal method only.
    #[inline(always)]
    fn new(inner: uint8x16x2_t, cpu: Neon) -> Self {
        Self { inner, cpu }
    }

    /// Create a [`Vec256`] by repeating `n` 32 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: Neon) -> Self {
        let half = Vec128::splat(n, cpu);
        Self::from_halves(half, half)
    }

    /// Create a [`Vec256`] by repeating `n` 8 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: Neon) -> Self {
        let half = Vec128::splat_u32(n, cpu);
        Self::from_halves(half, half)
    }

    /// Create a [`Vec256`] from its low and high 128 bits.
    #[inline(always)]
    pub fn from_halves(low: Vec128, high: Vec128) -> Self {
        Self::new(uint8x16x2_t(low.inner, high.inner), low.cpu)
    }

    /// Get the low and high 128 bits of this [`Vec256`].
    #[inline(always)]
    pub fn into_halves(self) -> (Vec128, Vec128) {
        (Vec128::new(self.inner.0, self.cpu), Vec128::new(self.inner.1, self.cpu))
    }
}

/// Loads.
impl Vec256 {
    /// Read a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 32 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: Neon) -> Self {
        // This is all `vld1q_u8_x2` does. Calling it would give the same instructions,
        // but it's a function which the compiler has to instantiate and then inline.
        // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
        // `Neon` type guarantees platform supports Neon.
        Self::new(unsafe { ptr.cast::<uint8x16x2_t>().read_unaligned() }, cpu)
    }

    /// Read a [`Vec256`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 32 bytes in total
    /// (e.g. `&[u8; 32]`, `&[i64; 4]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray256, cpu: Neon) -> Self {
        // SAFETY: `IntArray256` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 32 bytes
        unsafe { Self::load_unaligned(slice.as_ptr(), cpu) }
    }
}

/// Bitwise ops.
impl Vec256 {
    /// Invert all the bits of this [`Vec256`].
    #[inline(always)]
    pub fn not(self) -> Self {
        let (low, high) = self.into_halves();
        Self::from_halves(!low, !high)
    }

    /// AND this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand256<u8>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low & other_low, high & other_high)
    }

    /// OR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand256<u8>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low | other_low, high | other_high)
    }

    /// XOR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand256<u8>) -> Self {
        let (low, high) = self.into_halves();
        let (other_low, other_high) = other.into_vec256(self.cpu).into_halves();
        Self::from_halves(low ^ other_low, high ^ other_high)
    }
}

/// Bytewise ops.
impl Vec256 {
    /// Rearrange the bytes of this [`Vec256`] into a [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// * If `control[i]` is 32 or more, `result[i]` is 0.
    /// * Otherwise, `result[i]` is `self[control[i]]`.
    ///
    /// Often used as a 32-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Vec128) -> Vec128 {
        // SAFETY: `Vec256` type guarantees platform supports Neon
        Vec128::new(unsafe { vqtbl2q_u8(self.inner, control.inner) }, self.cpu)
    }
}
