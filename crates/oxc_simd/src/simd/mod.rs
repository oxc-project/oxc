//! SIMD vector types.
//!
//! # Vector types
//!
//! [`Vec128`] and [`Vec256`] wrap a platform's SIMD vector types and intrinsics in a safe API.
//!
//! They're generic over a CPU feature set `C` which implements [`CpuSimd`] ([`Avx2`] or [`Neon`]).
//! The CPU feature set types ([`Avx2`] and [`Neon`]) can only be created when the CPU is proven
//! to support these features. Vector types hold a `C: CpuSimd`, so a [`Vec128<Avx2>`]
//! or [`Vec256<Avx2>`] can only be created on a system which supports AVX2. Ditto for Neon.
//!
//! # Methods
//!
//! - Methods which load or store via a raw pointer are unsafe.
//!   The caller must ensure the pointer is valid.
//! - All other methods are safe, including `read*` and `write` methods,
//!   which read from and write to a reference.
//!
//! Some methods only exist for one CPU, because the other CPU has no matching instruction:
//!
//! - AVX2 only: [`movemask`] (Neon has no equivalent of x86's `pmovmskb`) and [`shuffle_u32s`].
//! - Neon only: [`shl_u8`], [`shr_u8`], [`pairwise_add_u8`], [`sum_u8`], [`nibble_mask`]
//!   and [`test_u8`].
//!
//! Most methods do the same on every CPU type, but these differ in behavior:
//!
//! - [`Vec128::shuffle_u8s`]:
//!   - AVX2: Control byte with the top bit set gives 0, and otherwise only its low 4 bits count.
//!   - Neon: Control byte of 16 or more gives 0.
//! - [`Vec256::shuffle_u8s`]:
//!   - AVX2: 2 separate lookups in 16-entry tables, with a 32-byte control and 32-byte result.
//!     Control bytes work the same as AVX2's [`Vec128::shuffle_u8s`].
//!   - Neon: 1 lookup in a 32-entry table, with a 16-byte control and 16-byte result.
//!     Control byte of 32 or more gives 0.
//! - [`Vec128::blend`] and [`Vec256::blend`]:
//!   - AVX2: Selects whole bytes, using the top bit of each byte of `mask`.
//!   - Neon: Selects each bit separately, using the same bit of `mask`.
//!
//! # Compilation
//!
//! The vector types and all their methods exist on every platform, so code which uses them
//! compiles on any platform. Internally, vector methods use the [`cpu_select!`] macro,
//! which chooses the method's implementation depending on the vector's feature set.
//! The macro gates that code with `#[cfg(target_arch = "...")]`, so it only compiles
//! on platforms which provide the required SIMD types and intrinsics.
//!
//! All code in the rest of this crate can be compiled, checked, and linted on any platform,
//! and Rust Analyzer can see all the code for all platforms "as one" on any dev machine.
//! Platform-specific code which has to be gated by `#[cfg(target_arch = "...")]` is confined
//! to just this module and the [`cpu`] module.
//!
//! # Performance
//!
//! All methods are `#[inline(always)]`, but the intrinsics they call are only inlined into
//! a function which enables the same target features. Without the attribute, code is still sound,
//! but every vector operation is a function call, passing the vectors via memory.
//!
//! Therefore functions which use these types must be compiled with AVX2 / Neon enabled.
//! This means they should have a `#[target_feature(enable = "avx2")]` /
//! `#[target_feature(enable = "neon")]` attribute, or be inlined into functions which do.
//! The attribute is essential on x86_64 (AVX2), but less important on aarch64 (Neon),
//! as every aarch64 target with `std` enables `neon` by default.
//!
//! `#[cpu_version]` and `#[cpu_multiversion]` macros can be used to ensure functions
//! are annotated with the right `#[target_feature(...)]` attributes.
//!
//! # Files in this directory
//!
//! [`Vec128`] and [`Vec256`] are defined in the [`vec128`] and [`vec256`] modules, along with
//! the methods which every SIMD feature set has, and the `!`, `&`, `|` and `^` operators.
//! The [`avx2`] and [`neon`] modules add methods which are exclusive to one CPU feature set
//! (e.g. [`movemask`] on x86_64).
//!
//! The other modules in this directory contain:
//!
//! - [`data`]: [`VecData128`] and [`VecData256`] - aligned byte arrays to read into vectors.
//! - [`arrays`]: traits for arrays of integers which vectors can read without alignment.
//! - [`macros`]: [`cpu_select!`] and [`unused!`] macros, used for defining methods.
//! - [`operand`]: traits which let an operation take either a vector or a scalar.
//!
//! [`Avx2`]: crate::cpu::Avx2
//! [`Neon`]: crate::cpu::Neon
//! [`movemask`]: Vec256::movemask
//! [`shuffle_u32s`]: Vec256::shuffle_u32s
//! [`shl_u8`]: Vec128::shl_u8
//! [`shr_u8`]: Vec128::shr_u8
//! [`pairwise_add_u8`]: Vec128::pairwise_add_u8
//! [`test_u8`]: Vec128::test_u8
//! [`sum_u8`]: Vec128::sum_u8
//! [`nibble_mask`]: Vec128::nibble_mask
//! [`cpu_select!`]: macros::cpu_select
//! [`cpu`]: crate::cpu
//! [`unused!`]: macros::unused

use crate::cpu::Cpu;

mod arrays;
mod avx2;
mod data;
mod macros;
mod neon;
mod operand;
mod vec128;
mod vec256;

pub use data::{VecData128, VecData256};
pub use vec128::Vec128;
pub use vec256::Vec256;

use vec256::Vec256Inner;

#[cfg(test)]
mod tests;

/// CPU feature set which has SIMD vector types.
///
/// Vector methods choose their code by CPU type, using [`cpu_select!`].
/// So code for a feature set only runs for vectors of that feature set's type.
///
/// On each platform, only one of the types which implement this trait can exist -
/// [`Avx2`] on x86_64, and [`Neon`] on aarch64. The other is uninhabited.
/// On other platforms, neither can exist.
///
/// Vector types' size and alignment reflect their native underlying types (e.g. `__m256i`).
/// e.g. on platforms other than x86_64, `Vec256<Avx2>` stores a stand-in with
/// the same size and alignment as `__m256i` (see [`CpuSimdPrivate::Vec256Storage`]).
///
/// This trait is sealed, so code outside the `simd` module can't implement it on other types.
///
/// It requires [`Cpu`], so only the types in the `cpu` module, which can't be forged,
/// can implement it. So holding a `C: CpuSimd` proves that the CPU supports the feature set.
///
/// [`Avx2`]: crate::cpu::Avx2
/// [`Neon`]: crate::cpu::Neon
/// [`cpu_select!`]: macros::cpu_select
#[expect(private_bounds)]
pub trait CpuSimd: CpuSimdPrivate + Cpu {}

/// Private trait to guarantee that only code within this module can implement [`CpuSimd`].
///
/// It also holds the type which a [`Vec256`] stores for each CPU type, and the conversions
/// between it and the platform's SIMD type - all of which is private to this module.
///
/// Conversion methods are no-ops. This conversion infrastructure is required just to ensure
/// that [`Vec256<Avx2>`] and [`Vec256<Neon>`] always have the same layout, no matter what
/// platform the code is compiled on. So e.g. structs containing them always show the correct
/// size and alignment in IDEs, and assertions about the size and alignment of types work
/// on any platform.
trait CpuSimdPrivate {
    /// Type which a `Vec256` of this CPU type stores.
    ///
    /// On its own platform, it's the platform's SIMD type ([`Vec256Inner`]).
    /// On other platforms, it's a stand-in with the same size and alignment,
    /// so a [`Vec256<Avx2>`] or [`Vec256<Neon>`] has the same layout on every platform.
    type Vec256Storage: Copy;

    /// Convert the platform's SIMD type to a [`Vec256Storage`].
    ///
    /// [`Vec256Storage`]: Self::Vec256Storage
    fn vec256_storage(inner: Vec256Inner) -> Self::Vec256Storage;

    /// Convert a [`Vec256Storage`] to the platform's SIMD type.
    ///
    /// [`Vec256Storage`]: Self::Vec256Storage
    fn vec256_inner(storage: Self::Vec256Storage) -> Vec256Inner;
}
