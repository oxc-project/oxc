//! Traits for the right-hand side of a vector operation, which can be a vector or a scalar.
//!
//! e.g. `v.and(w)` ANDs 2 vectors, and `v.and(0x0f)` ANDs every byte of `v` with `0x0f`.
//! A scalar is repeated to fill a vector.

#![expect(
    clippy::inline_always,
    reason = "conversions call vector methods, which must inline into the function calling them"
)]

use super::{CpuSimd, Vec128, Vec128Impl, Vec256, Vec256Impl};

/// Right-hand side of a [`Vec128`] operation.
///
/// Either another [`Vec128`], or a scalar of type `T` which is repeated to fill a [`Vec128`].
///
/// `T` is a parameter so that only 1 integer type implements `Operand128<T, C>` for each `T`.
/// That's what allows integer literals as operands without a suffix (e.g. `v & 0x0f`).
/// If an operation accepted both `u8` and `i8`, a literal's type would be ambiguous,
/// so it would fall back to `i32`, and wouldn't compile.
///
/// `C` is the CPU feature set of the [`Vec128`].
pub trait Operand128<T, C: CpuSimd> {
    /// Convert to a [`Vec128`].
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    fn into_vec128(self, cpu: C) -> Vec128<C>;
}

impl<T, C: CpuSimd> Operand128<T, C> for Vec128<C> {
    #[inline(always)]
    fn into_vec128(self, _cpu: C) -> Vec128<C> {
        self
    }
}

impl<C: CpuSimd> Operand128<u8, C> for u8
where
    Vec128<C>: Vec128Impl<Cpu = C>,
{
    #[inline(always)]
    fn into_vec128(self, cpu: C) -> Vec128<C> {
        Vec128::splat(self, cpu)
    }
}

impl<C: CpuSimd> Operand128<i8, C> for i8
where
    Vec128<C>: Vec128Impl<Cpu = C>,
{
    #[inline(always)]
    fn into_vec128(self, cpu: C) -> Vec128<C> {
        Vec128::splat(self.cast_unsigned(), cpu)
    }
}

impl<C: CpuSimd> Operand128<u32, C> for u32
where
    Vec128<C>: Vec128Impl<Cpu = C>,
{
    #[inline(always)]
    fn into_vec128(self, cpu: C) -> Vec128<C> {
        Vec128::splat_u32(self, cpu)
    }
}

/// Right-hand side of a [`Vec256`] operation.
///
/// Either another [`Vec256`], or a scalar of type `T` which is repeated to fill a [`Vec256`].
///
/// `T` is a parameter so that only 1 integer type implements `Operand256<T, C>` for each `T`.
/// That's what allows integer literals as operands without a suffix (e.g. `v & 0x0f`).
/// If an operation accepted both `u8` and `i8`, a literal's type would be ambiguous,
/// so it would fall back to `i32`, and wouldn't compile.
///
/// `C` is the CPU feature set of the [`Vec256`].
pub trait Operand256<T, C: CpuSimd> {
    /// Convert to a [`Vec256`].
    #[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    fn into_vec256(self, cpu: C) -> Vec256<C>;
}

impl<T, C: CpuSimd> Operand256<T, C> for Vec256<C> {
    #[inline(always)]
    fn into_vec256(self, _cpu: C) -> Vec256<C> {
        self
    }
}

impl<C: CpuSimd> Operand256<u8, C> for u8
where
    Vec256<C>: Vec256Impl<Cpu = C>,
{
    #[inline(always)]
    fn into_vec256(self, cpu: C) -> Vec256<C> {
        Vec256::splat(self, cpu)
    }
}

impl<C: CpuSimd> Operand256<i8, C> for i8
where
    Vec256<C>: Vec256Impl<Cpu = C>,
{
    #[inline(always)]
    fn into_vec256(self, cpu: C) -> Vec256<C> {
        Vec256::splat(self.cast_unsigned(), cpu)
    }
}

impl<C: CpuSimd> Operand256<u32, C> for u32
where
    Vec256<C>: Vec256Impl<Cpu = C>,
{
    #[inline(always)]
    fn into_vec256(self, cpu: C) -> Vec256<C> {
        Vec256::splat_u32(self, cpu)
    }
}
