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
//! Each feature set implements the traits in its own module,
//! `avx2` on x86_64 and `neon` on aarch64, along with the methods which only it has.
//! Other platforms have no vector types.
//!
//! The other modules:
//!
//! - `data` has [`VecData128`] and [`VecData256`], aligned byte arrays to read into vectors.
//! - `arrays` has traits for arrays of integers which vectors can read without alignment.
//! - `operand` has traits which let an operation take either a vector or a scalar.

#[cfg(target_arch = "x86_64")]
mod avx2;
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod arrays;
#[cfg(any(test, target_arch = "x86_64", target_arch = "aarch64"))]
mod data;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod operand;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod vec128;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod vec256;

#[cfg(any(test, target_arch = "x86_64", target_arch = "aarch64"))]
pub use data::{VecData128, VecData256};
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub use vec128::Vec128;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use vec128::Vec128Impl;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub use vec256::Vec256;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use vec256::Vec256Impl;

#[cfg(all(test, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod tests;

/// CPU feature set which has SIMD vector types.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub trait CpuSimd: Copy {
    /// [`Vec128`] for this CPU feature set.
    type Vec128: Vec128Impl<Cpu = Self>;
    /// [`Vec256`] for this CPU feature set.
    type Vec256: Vec256Impl<Cpu = Self>;
}

#[cfg(target_arch = "x86_64")]
impl CpuSimd for crate::cpu::Avx2 {
    type Vec128 = Vec128<Self>;
    type Vec256 = Vec256<Self>;
}

#[cfg(target_arch = "aarch64")]
impl CpuSimd for crate::cpu::Neon {
    type Vec128 = Vec128<Self>;
    type Vec256 = Vec256<Self>;
}
