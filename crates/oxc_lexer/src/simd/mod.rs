#[cfg(target_arch = "x86_64")]
use crate::cpu::Avx2 as CpuType;
#[cfg(target_arch = "aarch64")]
use crate::cpu::Neon as CpuType;

#[cfg(target_arch = "x86_64")]
mod avx2;
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod arrays;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod bit_traits;
#[cfg(any(test, target_arch = "x86_64", target_arch = "aarch64"))]
mod data;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod operand;

#[cfg(any(test, target_arch = "x86_64", target_arch = "aarch64"))]
pub use data::{VecData128, VecData256};

#[cfg(all(test, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod tests;

/// CPU feature set which has SIMD vector types.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub trait CpuSimd: Copy {
    /// [`Vec128`] for this CPU feature set.
    type Vec128: Vec128Impl<Cpu = Self>;
    /// [`Vec256`] for this CPU feature set.
    type Vec256: Vec256Impl<Cpu = Self>;

    /// Type which a [`Vec128`] wraps.
    type Vec128Inner: Copy;
    /// Type which a [`Vec256`] wraps.
    type Vec256Inner: Copy;
}

#[cfg(target_arch = "x86_64")]
impl CpuSimd for crate::cpu::Avx2 {
    type Vec128 = Vec128<Self>;
    type Vec256 = Vec256<Self>;
    type Vec128Inner = std::arch::x86_64::__m128i;
    type Vec256Inner = std::arch::x86_64::__m256i;
}

#[cfg(target_arch = "aarch64")]
impl CpuSimd for crate::cpu::Neon {
    type Vec128 = Vec128<Self>;
    type Vec256 = Vec256<Self>;
    type Vec128Inner = std::arch::aarch64::uint8x16_t;
    type Vec256Inner = std::arch::aarch64::uint8x16x2_t;
}

/// Methods of [`Vec128`] which code that is generic over the CPU type can call.
///
/// Each method is also an inherent method of [`Vec128`], which can be called without this trait in scope.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub trait Vec128Impl {
    /// CPU feature set which this vector is for.
    type Cpu: CpuSimd;

    /// Read a [`Vec128`] from a [`VecData128`].
    fn read(data: &VecData128, cpu: Self::Cpu) -> Self;
}

/// Methods of [`Vec256`] which code that is generic over the CPU type can call.
///
/// Each method is also an inherent method of [`Vec256`], which can be called without this trait in scope.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub trait Vec256Impl {
    /// CPU feature set which this vector is for.
    type Cpu: CpuSimd;

    /// Read a [`Vec256`] from a [`VecData256`].
    fn read(data: &VecData256, cpu: Self::Cpu) -> Self;
}

/// 128-bit SIMD vector.
///
/// Holds a `C`, so a vector can only exist if the CPU supports that feature set.
/// `C` is zero-sized, so [`Vec128`] is the same size as the type it wraps.
///
/// The methods are in the `avx2` and `neon` modules.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[derive(Clone, Copy)]
pub struct Vec128<C: CpuSimd> {
    inner: C::Vec128Inner,
    cpu: C,
}

/// 256-bit SIMD vector.
///
/// Holds a `C`, so a vector can only exist if the CPU supports that feature set.
/// `C` is zero-sized, so [`Vec256`] is the same size as the type it wraps.
///
/// The methods are in the `avx2` and `neon` modules.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[derive(Clone, Copy)]
pub struct Vec256<C: CpuSimd> {
    inner: C::Vec256Inner,
    cpu: C,
}
