//! Bitwise operators for [`Vec128`] and [`Vec256`].
//!
//! `!v`, `v & x`, `v | x` and `v ^ x` call the vector's `not`, `and`, `or` and `xor` methods,
//! and so do `&=`, `|=` and `^=`. As with those methods, `x` can be a vector or a `u8`,
//! so `v & 0x0f` works.
//!
//! The operators are only implemented for vectors of `CpuType`,
//! which is the CPU feature set with vector types on this platform.

#![expect(
    clippy::inline_always,
    reason = "operators call vector methods, which must inline into the function calling them"
)]

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::{
    CpuType, Vec128, Vec256,
    operand::{Operand128, Operand256},
};

// Vec128

impl Not for Vec128<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Vec128::not(self)
    }
}

impl<T: Operand128<u8>> BitAnd<T> for Vec128<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self {
        self.and(rhs)
    }
}

impl<T: Operand128<u8>> BitAndAssign<T> for Vec128<CpuType> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.and(rhs);
    }
}

impl<T: Operand128<u8>> BitOr<T> for Vec128<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self {
        self.or(rhs)
    }
}

impl<T: Operand128<u8>> BitOrAssign<T> for Vec128<CpuType> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.or(rhs);
    }
}

impl<T: Operand128<u8>> BitXor<T> for Vec128<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self {
        self.xor(rhs)
    }
}

impl<T: Operand128<u8>> BitXorAssign<T> for Vec128<CpuType> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.xor(rhs);
    }
}

// Vec256

impl Not for Vec256<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Vec256::not(self)
    }
}

impl<T: Operand256<u8>> BitAnd<T> for Vec256<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self {
        self.and(rhs)
    }
}

impl<T: Operand256<u8>> BitAndAssign<T> for Vec256<CpuType> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.and(rhs);
    }
}

impl<T: Operand256<u8>> BitOr<T> for Vec256<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self {
        self.or(rhs)
    }
}

impl<T: Operand256<u8>> BitOrAssign<T> for Vec256<CpuType> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.or(rhs);
    }
}

impl<T: Operand256<u8>> BitXor<T> for Vec256<CpuType> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self {
        self.xor(rhs)
    }
}

impl<T: Operand256<u8>> BitXorAssign<T> for Vec256<CpuType> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.xor(rhs);
    }
}
