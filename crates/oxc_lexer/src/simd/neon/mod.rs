//! SIMD vector types for Neon.
//!
//! [`Vec128`] wraps a [`uint8x16_t`], and [`Vec256`] wraps a [`uint8x16x2_t`].
//!
//! Each vector holds a [`Neon`], so a vector can only exist if the CPU supports Neon.
//! That removes the need for the caller of a vector's methods to promise it does.
//!
//! * Methods which load or store via a raw pointer are unsafe.
//!   The caller must ensure the pointer is valid.
//! * All other methods are safe, including `read_*` methods which read from a reference.
//!
//! [`Neon`] is zero-sized, so a vector is the same size as the type it wraps.
//!
//! # Differences from AVX2
//!
//! Methods have the same names as the methods of the AVX2 `Vec128` which do the same job,
//! but these differ in behavior:
//!
//! * [`shuffle_u8s`] reads all 8 bits of each control byte.
//!   On AVX2, a control byte with the top bit set gives 0, and otherwise only its low 4 bits count.
//! * [`blend`] selects each bit separately. On AVX2, it selects whole bytes, using their top bits.
//! * [`Vec256::shuffle_u8s`] is 1 lookup in a 32-entry table, with a 16-byte control.
//!   On AVX2, it's 2 separate lookups in 16-entry tables, with a 32-byte control.
//!
//! The AVX2 `Vec128` also has some methods which this type doesn't:
//!
//! * `movemask`. Neon has no equivalent of x86's `pmovmskb`, so it would be several instructions.
//! * Aligned `load` and `store`. Aligned loads and stores have no advantage on Neon.
//!
//! # Performance
//!
//! The methods are `#[inline(always)]`, but the intrinsics they call are only inlined into
//! a function which enables the same target features. Every aarch64 target with `std`
//! enables `neon` by default, so in practice they're inlined everywhere.
//!
//! [`Neon`]: crate::cpu::Neon
//! [`blend`]: Vec128::blend
//! [`shuffle_u8s`]: Vec128::shuffle_u8s
//! [`uint8x16_t`]: std::arch::aarch64::uint8x16_t
//! [`uint8x16x2_t`]: std::arch::aarch64::uint8x16x2_t

use crate::cpu::Neon;

mod vec128;
mod vec256;

#[cfg(test)]
mod tests;

/// [`Vec128`](super::Vec128) for Neon.
pub type Vec128 = super::Vec128<Neon>;

/// [`Vec256`](super::Vec256) for Neon.
///
/// Neon registers are 128 bits, so this is a pair of registers.
pub type Vec256 = super::Vec256<Neon>;
