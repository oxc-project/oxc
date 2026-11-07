//! Extension of SIMD vector types for Neon.
//!
//! [`Vec128<Neon>`] wraps a [`uint8x16_t`], and [`Vec256<Neon>`] wraps a [`uint8x16x2_t`].
//! Neon registers are 128 bits, so a [`Vec256<Neon>`] is a pair of SIMD registers.
//!
//! The methods which every CPU type has are in `simd`'s [`vec128`] and [`vec256`] modules.
//! This module implements [`CpuSimd`] and [`CpuSimdPrivate`] for [`Neon`],
//! and implements the vector methods which are exclusive to Neon.
//!
//! [`vec128`]: super::vec128
//! [`vec256`]: super::vec256
//! [`Neon`]: crate::cpu::Neon
//! [`uint8x16_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16_t.html
//! [`uint8x16x2_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16x2_t.html

// These methods must be inlined to get the `target_feature`s of the functions calling them,
// otherwise the intrinsics also can't be inlined into them - 2 function calls for each method
#![expect(clippy::inline_always)]

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::cpu::Neon;

use super::{
    CpuSimd, CpuSimdPrivate, Vec128, Vec256, Vec256Inner,
    macros::{cpu_select, unused},
    operand::Operand128,
};

#[cfg_attr(target_arch = "aarch64", expect(unused_imports))]
use super::VecData128;

#[cfg(test)]
mod tests;

impl CpuSimd for Neon {}

/// A [`Vec256<Neon>`] stores a [`uint8x16x2_t`].
///
/// [`uint8x16x2_t`] only exists on aarch64. On other platforms, [`Vec256<Neon>`] stores a pair
/// of [`VecData128`]s instead, which is the same size and alignment (32 bytes, aligned on 16).
/// So [`Vec256<Neon>`] has the same layout on every platform.
///
/// On aarch64, [`Vec256Inner`] is [`uint8x16x2_t`] too, so the conversions are no-ops.
/// On other platforms, a [`Vec256<Neon>`] can never be created, so the conversion methods
/// are unreachable.
///
/// [`uint8x16x2_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16x2_t.html
impl CpuSimdPrivate for Neon {
    type Vec256Storage = cfg_select! {
        target_arch = "aarch64" => uint8x16x2_t,
        _ => [VecData128; 2],
    };

    #[inline(always)]
    fn vec256_storage(inner: Vec256Inner) -> Self::Vec256Storage {
        cfg_select! {
            target_arch = "aarch64" => inner,
            _ => unused!(inner),
        }
    }

    #[inline(always)]
    fn vec256_inner(storage: Self::Vec256Storage) -> Vec256Inner {
        cfg_select! {
            target_arch = "aarch64" => storage,
            _ => unused!(storage),
        }
    }
}

/// [`Vec128`] methods which are specific to Neon.
impl Vec128<Neon> {
    /// Shift each of the 16 bytes of this [`Vec128`] left by `BITS` bits.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn shl_u8<const BITS: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Neon => {
                // SAFETY: `Vec128<Neon>` type guarantees platform supports Neon
                Self::new(unsafe { vshlq_n_u8::<BITS>(self.inner()) }, self.cpu())
            }
            _ => unused!(self)
        })
    }

    /// Shift each of the 16 bytes of this [`Vec128`] right by `BITS` bits.
    #[inline(always)]
    pub fn shr_u8<const BITS: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Neon => {
                // SAFETY: `Vec128<Neon>` type guarantees platform supports Neon
                Self::new(unsafe { vshrq_n_u8::<BITS>(self.inner()) }, self.cpu())
            }
            _ => unused!(self)
        })
    }

    /// Add adjacent pairs of bytes in this [`Vec128`] and `other`, wrapping on overflow.
    ///
    /// The result is `[s0 + s1, s2 + s3, .., s14 + s15, o0 + o1, o2 + o3, .., o14 + o15]`,
    /// where `sN` and `oN` are the bytes at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn pairwise_add_u8(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Neon => {
                // SAFETY: `Vec128<Neon>` type guarantees platform supports Neon
                Self::new(unsafe { vpaddq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Check every byte of this [`Vec128`] against either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes share any set bits, otherwise `0x00`.
    #[inline(always)]
    pub fn test_u8(self, other: impl Operand128<u8, Neon>) -> Self {
        cpu_select!(self.cpu(), {
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Vec128<Neon>` type guarantees platform supports Neon
                Self::new(unsafe { vtstq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }
}

/// [`Vec256`] methods which are specific to Neon.
impl Vec256<Neon> {
    /// Rearrange the bytes of this [`Vec256`] into a [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// - If `control[i]` is 32 or more, `result[i]` is 0.
    /// - Otherwise, `result[i]` is `self[control[i]]`.
    ///
    /// Often used as a 32-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Vec128<Neon>) -> Vec128<Neon> {
        cpu_select!(self.cpu(), {
            Neon => {
                // SAFETY: `Vec256<Neon>` type guarantees platform supports Neon
                Vec128::new(unsafe { vqtbl2q_u8(self.inner(), control.inner()) }, self.cpu())
            }
            _ => unused!(self, control)
        })
    }
}
