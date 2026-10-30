#[cfg(target_arch = "x86_64")]
pub mod avx2;
#[cfg(target_arch = "aarch64")]
pub mod neon;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod arrays;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod bit_traits;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod operand;

#[cfg(all(test, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod tests;

#[cfg(target_arch = "x86_64")]
use crate::cpu::Avx2 as CpuType;
#[cfg(target_arch = "aarch64")]
use crate::cpu::Neon as CpuType;

/// CPU feature set which has SIMD vector types.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub trait CpuSimd: Copy {
    /// Type which a [`Vec128`] wraps.
    type Vec128Inner: Copy;
    /// Type which a [`Vec256`] wraps.
    type Vec256Inner: Copy;
}

#[cfg(target_arch = "x86_64")]
impl CpuSimd for crate::cpu::Avx2 {
    type Vec128Inner = std::arch::x86_64::__m128i;
    type Vec256Inner = std::arch::x86_64::__m256i;
}

#[cfg(target_arch = "aarch64")]
impl CpuSimd for crate::cpu::Neon {
    type Vec128Inner = std::arch::aarch64::uint8x16_t;
    type Vec256Inner = std::arch::aarch64::uint8x16x2_t;
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
