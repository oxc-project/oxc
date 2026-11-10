//! Extension of SIMD vector types for AVX2.
//!
//! [`Vec128<Avx2>`] wraps an [`__m128i`], and [`Vec256<Avx2>`] wraps an [`__m256i`].
//!
//! The methods which every CPU type has are in `simd`'s [`vec128`] and [`vec256`] modules.
//! This module implements [`CpuSimd`] and [`CpuSimdPrivate`] for [`Avx2`],
//! and implements the vector methods which are exclusive to AVX2.
//!
//! [`vec128`]: super::vec128
//! [`vec256`]: super::vec256
//! [`Avx2`]: crate::cpu::Avx2
//! [`__m128i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m128i.html
//! [`__m256i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m256i.html

// These methods must be inlined to get the `target_feature`s of the functions calling them,
// otherwise the intrinsics also can't be inlined into them - 2 function calls for each method
#![expect(clippy::inline_always)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::cpu::Avx2;

use super::{
    CpuSimd, CpuSimdPrivate, Vec128, Vec256, Vec256Inner,
    macros::{cpu_select, unused},
};

#[cfg_attr(target_arch = "x86_64", expect(unused_imports))]
use super::VecData256;

#[cfg(test)]
mod tests;

impl CpuSimd for Avx2 {}

/// A [`Vec256<Avx2>`] stores an [`__m256i`].
///
/// [`__m256i`] only exists on x86_64. On other platforms, [`Vec256<Avx2>`] stores
/// a [`VecData256`] instead, which is the same size and alignment (32 bytes, aligned on 32).
/// So [`Vec256<Avx2>`] has the same layout on every platform.
///
/// On x86_64, [`Vec256Inner`] is [`__m256i`] too, so the conversions are no-ops.
/// On other platforms, a [`Vec256<Avx2>`] can never be created, so the conversion methods
/// are unreachable.
///
/// [`__m256i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m256i.html
impl CpuSimdPrivate for Avx2 {
    type Vec256Storage = cfg_select! {
        target_arch = "x86_64" => __m256i,
        _ => VecData256,
    };

    #[inline(always)]
    fn vec256_storage(inner: Vec256Inner) -> Self::Vec256Storage {
        cfg_select! {
            target_arch = "x86_64" => inner,
            _ => unused!(inner),
        }
    }

    #[inline(always)]
    fn vec256_inner(storage: Self::Vec256Storage) -> Vec256Inner {
        cfg_select! {
            target_arch = "x86_64" => storage,
            _ => unused!(storage),
        }
    }
}

/// [`Vec128`] methods which are specific to AVX2.
///
/// These are 128-bit operations, so the intrinsics are from SSE2 to SSE4.1,
/// which every CPU with AVX2 supports.
impl Vec128<Avx2> {
    /// Gather the top bit of each byte of this [`Vec128`] into a 16-bit mask.
    ///
    /// Bit N of the result is the top bit of byte N. Bits 16-31 of the result are zero.
    #[inline(always)]
    pub fn movemask(self) -> u32 {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Vec128<Avx2>` type guarantees platform supports AVX2,
                // which includes SSE2
                unsafe { _mm_movemask_epi8(self.inner()).cast_unsigned() }
            }
            _ => unused!(self)
        })
    }
}

/// [`Vec256`] methods which are specific to AVX2.
///
/// Some AVX2 instructions treat a 256-bit vector as 2 separate 128-bit halves
/// (e.g. `shuffle_u8s`, `unpack_lo_u32`). See the docs for those methods.
impl Vec256<Avx2> {
    /// Gather the top bit of each byte of this [`Vec256`] into a 32-bit mask.
    ///
    /// Bit N of the result is the top bit of byte N.
    #[inline(always)]
    pub fn movemask(self) -> u32 {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Vec256<Avx2>` type guarantees platform supports AVX2
                unsafe { _mm256_movemask_epi8(self.inner()).cast_unsigned() }
            }
            _ => unused!(self)
        })
    }

    /// Rearrange the bytes of this [`Vec256`].
    ///
    /// The low and high 128 bits are treated as 2 separate 16-byte vectors,
    /// so a byte can only move within its own half.
    ///
    /// In each half, for each byte `i`:
    ///
    /// - If the top bit of `control[i]` is set, `result[i]` is 0.
    /// - Otherwise, `result[i]` is `self[control[i] & 0x0F]`.
    ///
    /// Often used as a 16-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Vec256<Avx2>` type guarantees platform supports AVX2
                Self::new(unsafe { _mm256_shuffle_epi8(self.inner(), control.inner()) }, self.cpu())
            }
            _ => unused!(self, control)
        })
    }

    /// Rearrange the 8 `u32`s of this [`Vec256`].
    ///
    /// For each `u32` `i`, `result[i]` is `self[indices[i] & 7]`.
    ///
    /// Unlike [`shuffle_u8s`], values can move between the low and high 128 bits.
    ///
    /// [`shuffle_u8s`]: Self::shuffle_u8s
    #[inline(always)]
    pub fn shuffle_u32s(self, indices: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Vec256<Avx2>` type guarantees platform supports AVX2
                Self::new(
                    unsafe { _mm256_permutevar8x32_epi32(self.inner(), indices.inner()) },
                    self.cpu(),
                )
            }
            _ => unused!(self, indices)
        })
    }
}
