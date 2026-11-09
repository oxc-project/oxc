//! CPU feature sets, and SIMD vector types which work with them.
//!
//! - CPU feature sets:
//!   - [`Avx2`], [`Neon`] and [`Generic`].
//!   - [`DynCpu`] which holds any of them.
//!   - [`Cpu`] trait which they all implement.
//!   - [`BASELINE`] const which is the most capable SIMD implementation
//!     statically known to exist on this CPU.
//! - SIMD vector types: [`Vec128`] and [`Vec256`].
//! - Vector data types: [`VecData128`] and [`VecData256`], to read/write SIMD vectors from/to.
//! - Macros which compile code for each CPU feature set: [`cpu_multiversion`] and [`cpu_version`].
//! - Macro for testing SIMD code `test_cpus` (with `testing` Cargo feature).
//!
//! The docs on the `cpu` and `simd` modules explain how they work.

// The SIMD code and its tests assume little-endian byte order
// (e.g. that a vector's `u64` lanes hold its bytes in little-endian order).
// The SIMD code is only tested in CI on little-endian platforms.
#![cfg(target_endian = "little")]

// The macros' output refers to this crate's items as `::oxc_simd::...`,
// so it works in other crates. This makes those paths work in this crate too.
extern crate self as oxc_simd;

pub use oxc_simd_macros::{cpu_multiversion, cpu_version};

#[cfg(any(test, feature = "testing"))]
pub use oxc_simd_macros::test_cpus;

mod cpu;
mod prefetch;
mod simd;

pub use cpu::{Avx2, BASELINE, Cpu, DynCpu, Generic, Neon, to_dyn};
pub use prefetch::prefetch;
pub use simd::{Vec128, Vec256, VecData128, VecData256};
