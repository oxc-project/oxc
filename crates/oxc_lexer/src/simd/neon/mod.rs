//! SIMD vector types for Neon.
//!
//! `Vec128<Neon>` wraps a [`uint8x16_t`], and `Vec256<Neon>` wraps a [`uint8x16x2_t`].
//! Neon registers are 128 bits, so a `Vec256<Neon>` is a pair of registers.
//!
//! On other platforms, they can never be created, and wrap a [`VecData128`] / pair of
//! [`VecData128`]s, which have the same sizes and alignments. Their method bodies are only
//! compiled on aarch64, but code using the types can be compiled and linted on any platform.
//!
//! Each vector holds a [`Neon`], so a vector can only exist if the CPU supports Neon.
//! That removes the need for the caller of a vector's methods to promise it does.
//!
//! * Methods which load or store via a raw pointer are unsafe.
//!   The caller must ensure the pointer is valid.
//! * All other methods are safe, including `read*` and `write` methods,
//!   which read from and write to a reference.
//!
//! [`Neon`] is zero-sized, so a vector is the same size as the type it wraps.
//!
//! # Differences from AVX2
//!
//! Methods have the same names as the methods of `Vec128<Avx2>` and `Vec256<Avx2>`
//! which do the same job, but these differ in behavior:
//!
//! * [`shuffle_u8s`] reads all 8 bits of each control byte.
//!   On AVX2, a control byte with the top bit set gives 0, and otherwise only its low 4 bits count.
//! * `blend` selects each bit separately. On AVX2, it selects whole bytes, using their top bits.
//! * [`Vec256::shuffle_u8s`] is 1 lookup in a 32-entry table, with a 16-byte control.
//!   On AVX2, it's 2 separate lookups in 16-entry tables, with a 32-byte control.
//! * `load`, `store`, `read` and `write` use the same instructions as their unaligned versions.
//!   Neon has no separate aligned load and store instructions.
//!
//! Some methods only exist for one CPU, because the other CPU has no matching instruction:
//!
//! * AVX2 only: `movemask` (Neon has no equivalent of x86's `pmovmskb`) and `Vec256::shuffle_u32s`.
//! * Neon only: `shl_u8`, `shr_u8`, `pairwise_add_u8` and `test_u8`.
//!
//! # Performance
//!
//! The methods are `#[inline(always)]`, but the intrinsics they call are only inlined into
//! a function which enables the same target features. Every aarch64 target with `std`
//! enables `neon` by default, so in practice they're inlined everywhere.
//!
//! [`Neon`]: crate::cpu::Neon
//! [`Vec256::shuffle_u8s`]: super::Vec256::shuffle_u8s
//! [`shuffle_u8s`]: super::Vec128::shuffle_u8s
//! [`uint8x16_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16_t.html
//! [`uint8x16x2_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16x2_t.html
//! [`VecData128`]: super::VecData128

mod vec128;
mod vec256;

#[cfg(test)]
mod tests;
