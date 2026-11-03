//! Traits for the right-hand side of a vector operation, which can be a vector or a scalar.
//!
//! e.g. `v.and(w)` ANDs 2 vectors, and `v.and(0x0f)` ANDs every byte of `v` with `0x0f`.
//! A scalar is repeated to fill a vector.
//!
//! The traits only cover vectors of `CpuType`,
//! which is the CPU feature set with vector types on this platform.

#![expect(
    clippy::inline_always,
    reason = "conversions call vector methods, which must inline into the function calling them"
)]

use super::{CpuType, Vec128, Vec256};

/// Right-hand side of a [`Vec128`] operation.
///
/// Either another [`Vec128`], or a scalar of type `T` which is repeated to fill a [`Vec128`].
///
/// `T` is a parameter so that only 1 integer type implements `Operand128<T>` for each `T`.
/// That's what allows integer literals as operands without a suffix (e.g. `v & 0x0f`).
/// If an operation accepted both `u8` and `i8`, a literal's type would be ambiguous,
/// so it would fall back to `i32`, and wouldn't compile.
pub trait Operand128<T> {
    /// Convert to a [`Vec128`].
    fn into_vec128(self, cpu: CpuType) -> Vec128<CpuType>;
}

impl<T> Operand128<T> for Vec128<CpuType> {
    #[inline(always)]
    fn into_vec128(self, _cpu: CpuType) -> Vec128<CpuType> {
        self
    }
}

impl Operand128<u8> for u8 {
    #[inline(always)]
    fn into_vec128(self, cpu: CpuType) -> Vec128<CpuType> {
        Vec128::splat(self, cpu)
    }
}

impl Operand128<i8> for i8 {
    #[inline(always)]
    fn into_vec128(self, cpu: CpuType) -> Vec128<CpuType> {
        Vec128::splat(self.cast_unsigned(), cpu)
    }
}

impl Operand128<u32> for u32 {
    #[inline(always)]
    fn into_vec128(self, cpu: CpuType) -> Vec128<CpuType> {
        Vec128::splat_u32(self, cpu)
    }
}

/// Right-hand side of a [`Vec256`] operation.
///
/// Either another [`Vec256`], or a scalar of type `T` which is repeated to fill a [`Vec256`].
///
/// `T` is a parameter so that only 1 integer type implements `Operand256<T>` for each `T`.
/// That's what allows integer literals as operands without a suffix (e.g. `v & 0x0f`).
/// If an operation accepted both `u8` and `i8`, a literal's type would be ambiguous,
/// so it would fall back to `i32`, and wouldn't compile.
pub trait Operand256<T> {
    /// Convert to a [`Vec256`].
    fn into_vec256(self, cpu: CpuType) -> Vec256<CpuType>;
}

impl<T> Operand256<T> for Vec256<CpuType> {
    #[inline(always)]
    fn into_vec256(self, _cpu: CpuType) -> Vec256<CpuType> {
        self
    }
}

impl Operand256<u8> for u8 {
    #[inline(always)]
    fn into_vec256(self, cpu: CpuType) -> Vec256<CpuType> {
        Vec256::splat(self, cpu)
    }
}

impl Operand256<i8> for i8 {
    #[inline(always)]
    fn into_vec256(self, cpu: CpuType) -> Vec256<CpuType> {
        Vec256::splat(self.cast_unsigned(), cpu)
    }
}

impl Operand256<u32> for u32 {
    #[inline(always)]
    fn into_vec256(self, cpu: CpuType) -> Vec256<CpuType> {
        Vec256::splat_u32(self, cpu)
    }
}
