//! SIMD vector types.
//!
//! [`Vec128`] and [`Vec256`] wrap a platform's SIMD vector types and intrinsics in a safe API.
//! They're generic over a CPU feature set `C` which implements [`CpuSimd`] (`Avx2` or `Neon`).
//! A vector holds a `C`, so it can only exist if the CPU supports that feature set.
//! So only the methods which load or store via a raw pointer are unsafe.
//!
//! [`Vec128`] and [`Vec256`] are in the `vec128` and `vec256` modules, along with the
//! [`Vec128Impl`] and [`Vec256Impl`] traits, which hold the methods which every feature set has,
//! and the `!`, `&`, `|` and `^` operators.
//! Each feature set implements the traits in its own module, `avx2` or `neon`,
//! along with the methods which only it has.
//!
//! The vector types and all their methods exist on every platform, so code which uses them
//! compiles everywhere. Each feature set's method bodies are only compiled on its own platform
//! (`avx2` on x86_64, `neon` on aarch64). Elsewhere, the feature set's type is uninhabited,
//! so its vectors can never be created, and its methods are `unreachable!()`.
//! See [`cpu_impl`].
//!
//! The other modules:
//!
//! - `data` has [`VecData128`] and [`VecData256`], aligned byte arrays to read into vectors.
//! - `arrays` has traits for arrays of integers which vectors can read without alignment.
//! - `operand` has traits which let an operation take either a vector or a scalar.
//! - `prefetch` has a function which asks the CPU to fetch memory into the cache.
//!
//! [`cpu_impl`]: crate::cpu::cpu_impl

use crate::cpu::{Avx2, Neon};

mod arrays;
mod avx2;
mod data;
mod neon;
mod operand;
mod prefetch;
mod vec128;
mod vec256;

pub use data::{VecData128, VecData256};
pub use prefetch::prefetch;
pub use vec128::Vec128;
pub use vec256::Vec256;

use vec128::Vec128Impl;
use vec256::Vec256Impl;

#[cfg(test)]
mod tests;

/// CPU feature set which has SIMD vector types.
pub trait CpuSimd: Copy {
    /// [`Vec128`] for this CPU feature set.
    type Vec128: Vec128Impl<Cpu = Self>;
    /// [`Vec256`] for this CPU feature set.
    type Vec256: Vec256Impl<Cpu = Self>;
}

impl CpuSimd for Avx2 {
    type Vec128 = Vec128<Avx2>;
    type Vec256 = Vec256<Avx2>;
}

impl CpuSimd for Neon {
    type Vec128 = Vec128<Neon>;
    type Vec256 = Vec256<Neon>;
}
