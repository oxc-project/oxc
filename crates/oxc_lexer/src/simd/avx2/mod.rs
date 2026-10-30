//! SIMD vector types for AVX2.
//!
//! [`Vec256`] wraps an [`__m256i`], and [`Vec128`] wraps an [`__m128i`].
//!
//! Each vector holds an [`Avx2`], so a vector can only exist if the CPU supports AVX2.
//! That removes the need for the caller of a vector's methods to promise it does.
//!
//! * Methods which load or store via a raw pointer are unsafe.
//!   The caller must ensure the pointer is valid.
//! * All other methods are safe, including `read_*` methods which read from a reference.
//!
//! [`Avx2`] is zero-sized, so a vector is the same size as the type it wraps.
//!
//! # Performance
//!
//! A function which uses these types must have `#[target_feature(enable = "avx2")]`,
//! or be inlined into a function which does.
//!
//! The methods are `#[inline(always)]`, but the intrinsics they call are only inlined into
//! a function which enables the same target features. Without the attribute, code is still sound,
//! but every vector operation is a function call, passing the vectors via memory.
//!
//! [`Avx2`]: crate::cpu::Avx2
//! [`__m128i`]: std::arch::x86_64::__m128i
//! [`__m256i`]: std::arch::x86_64::__m256i

use crate::cpu::Avx2;

mod vec128;
mod vec256;

#[cfg(test)]
mod tests;

/// [`Vec128`](super::Vec128) for AVX2.
pub type Vec128 = super::Vec128<Avx2>;

/// [`Vec256`](super::Vec256) for AVX2.
pub type Vec256 = super::Vec256<Avx2>;
