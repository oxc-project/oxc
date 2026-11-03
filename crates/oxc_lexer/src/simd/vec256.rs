//! [`Vec256`], a 256-bit SIMD vector which is generic over the CPU type.
//!
//! [`Vec256Impl`] holds the methods which every CPU type's [`Vec256`] has.
//! The inherent methods here are generic over the CPU type. Each calls the trait method
//! of the same name, so code can call the methods without the trait in scope.
//! Each CPU type implements the trait in its own module (`avx2` or `neon`),
//! along with the methods which only it has.
//!
//! The `!`, `&`, `|` and `^` operators for [`Vec256`] are here too.

#![expect(clippy::inline_always, reason = "wrappers must inline into the function calling them")]

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::{CpuSimd, Vec128, VecData256, arrays::IntArray256, operand::Operand256};

/// 256-bit SIMD vector.
///
/// Holds a `C`, so a vector can only exist if the CPU supports that feature set.
/// `C` is zero-sized, so [`Vec256`] is the same size as the type it wraps.
///
/// The methods which every CPU type has are in [`Vec256Impl`], and are also inherent methods.
/// The `avx2` and `neon` modules have the methods which only one CPU type has.
pub struct Vec256<C: CpuSimd> {
    pub(super) inner: <C::Vec256 as Vec256Impl>::Inner,
    pub(super) cpu: C,
}

// `Clone` and `Copy` are implemented by hand, because `#[derive(Copy)]` would add a bound
// requiring the type of `inner` to be `Copy`. Checking that bound requires
// `Vec256<C>: Vec256Impl`, which requires `Vec256<C>: Copy`, which is a cycle.
// `#[derive(Clone)]` has the same problem.
#[expect(clippy::expl_impl_clone_on_copy)]
impl<C: CpuSimd> Clone for Vec256<C> {
    /// Copy this [`Vec256`].
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: CpuSimd> Copy for Vec256<C> {}

/// Methods of [`Vec256`] which code that is generic over the CPU type can call.
///
/// Each CPU type's [`Vec256`] implements this trait.
/// Each method is also an inherent method of [`Vec256`],
/// which can be called without this trait in scope.
pub trait Vec256Impl: Copy {
    /// CPU feature set which this vector is for.
    type Cpu: CpuSimd;
    /// Type which this vector wraps.
    type Inner: Copy;

    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec256`] with all bits zero.
    fn zero(cpu: Self::Cpu) -> Self;

    /// Create a [`Vec256`] by repeating `n` 32 times.
    fn splat(n: u8, cpu: Self::Cpu) -> Self;

    /// Create a [`Vec256`] by repeating `n` 8 times.
    fn splat_u32(n: u32, cpu: Self::Cpu) -> Self;

    /// Create a [`Vec256`] by repeating `v` twice.
    fn splat_v128(v: Vec128<Self::Cpu>) -> Self;

    /// Create a [`Vec256`] by zero-extending the low 8 bytes of `v` to 8 `u32`s.
    fn widen_u8_to_u32(v: Vec128<Self::Cpu>) -> Self;

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get the [`CpuSimd`] token from this [`Vec256`].
    fn cpu(self) -> Self::Cpu;

    // ----------------------------------------
    // Loads and stores
    // ----------------------------------------

    /// Read a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 32 bytes.
    /// * `ptr` must be aligned on 32.
    unsafe fn load(ptr: *const u8, cpu: Self::Cpu) -> Self;

    /// Read a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 32 bytes.
    unsafe fn load_unaligned(ptr: *const u8, cpu: Self::Cpu) -> Self;

    /// Read a [`Vec256`] from a [`VecData256`], which is aligned on 32.
    fn read(data: &VecData256, cpu: Self::Cpu) -> Self;

    /// Read a [`Vec256`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 32 bytes in total
    /// (e.g. `&[u8; 32]`, `&[i64; 4]`).
    fn read_unaligned(slice: &impl IntArray256, cpu: Self::Cpu) -> Self;

    /// Store a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for writing 32 bytes.
    /// * `ptr` must be aligned on 32.
    unsafe fn store(self, ptr: *mut u8);

    /// Store a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 32 bytes.
    unsafe fn store_unaligned(self, ptr: *mut u8);

    /// Write a [`Vec256`] into a [`VecData256`], which is aligned on 32.
    fn write(self, dst: &mut VecData256);

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec256`] are zero.
    fn is_zero(self) -> bool;

    /// Invert all the bits of this [`Vec256`].
    fn not(self) -> Self;

    /// AND this [`Vec256`] with either another [`Vec256`] or a `u8`.
    fn and(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// OR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    fn or(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// XOR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    fn xor(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// Shift each of the 16 `u16`s of this [`Vec256`] left by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    fn shl_u16<const BITS: i32>(self) -> Self;

    /// Shift each of the 16 `u16`s of this [`Vec256`] right by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    fn shr_u16<const BITS: i32>(self) -> Self;

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// wrapping on overflow.
    fn add_u8(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// stopping at 255.
    fn saturating_add_u8(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// wrapping on overflow.
    fn sub_u8(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// stopping at 0.
    fn saturating_sub_u8(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// Add either another [`Vec256`] or a `u32` to every `u32` of this [`Vec256`],
    /// wrapping on overflow.
    fn add_u32(self, other: impl Operand256<u32, Self::Cpu>) -> Self;

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec256`] to either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    fn eq_u8(self, other: impl Operand256<u8, Self::Cpu>) -> Self;

    /// Check if every byte of this [`Vec256`] equals either another [`Vec256`] or a `u8`.
    fn all_eq_u8(self, other: impl Operand256<u8, Self::Cpu>) -> bool;

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    fn lt_i8(self, other: impl Operand256<i8, Self::Cpu>) -> Self;

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    fn gt_i8(self, other: impl Operand256<i8, Self::Cpu>) -> Self;

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Combine the bytes of this [`Vec256`] and either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the same byte of `mask` is `0xFF`,
    /// or from `self` if it's `0x00`.
    /// Other values in `mask` give different results on different CPUs.
    fn blend(self, other: impl Operand256<u8, Self::Cpu>, mask: Self) -> Self;

    // ----------------------------------------
    // `u32`-wise ops
    // ----------------------------------------

    /// Interleave the low 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s0, o0, s1, o1, s4, o4, s5, o5]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    fn unpack_lo_u32(self, other: Self) -> Self;

    /// Interleave the high 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s2, o2, s3, o3, s6, o6, s7, o7]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    fn unpack_hi_u32(self, other: Self) -> Self;

    // ----------------------------------------
    // Operations on 128-bit halves
    // ----------------------------------------

    /// Create a [`Vec256`] from its low and high 128 bits.
    fn from_halves(low: Vec128<Self::Cpu>, high: Vec128<Self::Cpu>) -> Self;

    /// Get the low and high 128 bits of this [`Vec256`].
    fn into_halves(self) -> (Vec128<Self::Cpu>, Vec128<Self::Cpu>);

    /// Get a [`Vec256`] made of the low 128 bits of this [`Vec256`],
    /// followed by the low 128 bits of `other`.
    fn join_lo_halves(self, other: Self) -> Self;

    /// Get a [`Vec256`] made of the high 128 bits of this [`Vec256`],
    /// followed by the high 128 bits of `other`.
    fn join_hi_halves(self, other: Self) -> Self;
}

/// Methods which are generic over the CPU type.
///
/// Each calls the [`Vec256Impl`] method of the same name, which each CPU type implements.
/// Calling them doesn't need [`Vec256Impl`] in scope.
impl<C: CpuSimd> Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec256`] with all bits zero.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn zero(cpu: C) -> Self {
        <Self as Vec256Impl>::zero(cpu)
    }

    /// Create a [`Vec256`] by repeating `n` 32 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: C) -> Self {
        <Self as Vec256Impl>::splat(n, cpu)
    }

    /// Create a [`Vec256`] by repeating `n` 8 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: C) -> Self {
        <Self as Vec256Impl>::splat_u32(n, cpu)
    }

    /// Create a [`Vec256`] by repeating `v` twice.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn splat_v128(v: Vec128<C>) -> Self {
        <Self as Vec256Impl>::splat_v128(v)
    }

    /// Create a [`Vec256`] by zero-extending the low 8 bytes of `v` to 8 `u32`s.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn widen_u8_to_u32(v: Vec128<C>) -> Self {
        <Self as Vec256Impl>::widen_u8_to_u32(v)
    }

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get the [`CpuSimd`] token from this [`Vec256`].
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn cpu(self) -> C {
        <Self as Vec256Impl>::cpu(self)
    }

    // ----------------------------------------
    // Loads and stores
    // ----------------------------------------

    /// Read a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 32 bytes.
    /// * `ptr` must be aligned on 32.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn load(ptr: *const u8, cpu: C) -> Self {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec256Impl>::load(ptr, cpu) }
    }

    /// Read a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 32 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: C) -> Self {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec256Impl>::load_unaligned(ptr, cpu) }
    }

    /// Read a [`Vec256`] from a [`VecData256`], which is aligned on 32.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn read(data: &VecData256, cpu: C) -> Self {
        <Self as Vec256Impl>::read(data, cpu)
    }

    /// Read a [`Vec256`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 32 bytes in total
    /// (e.g. `&[u8; 32]`, `&[i64; 4]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray256, cpu: C) -> Self {
        <Self as Vec256Impl>::read_unaligned(slice, cpu)
    }

    /// Store a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for writing 32 bytes.
    /// * `ptr` must be aligned on 32.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec256Impl>::store(self, ptr) }
    }

    /// Store a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 32 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec256Impl>::store_unaligned(self, ptr) }
    }

    /// Write a [`Vec256`] into a [`VecData256`], which is aligned on 32.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn write(self, dst: &mut VecData256) {
        <Self as Vec256Impl>::write(self, dst);
    }

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec256`] are zero.
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        <Self as Vec256Impl>::is_zero(self)
    }

    /// Invert all the bits of this [`Vec256`].
    #[inline(always)]
    pub fn not(self) -> Self {
        <Self as Vec256Impl>::not(self)
    }

    /// AND this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::and(self, other)
    }

    /// OR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::or(self, other)
    }

    /// XOR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::xor(self, other)
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] left by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn shl_u16<const BITS: i32>(self) -> Self {
        <Self as Vec256Impl>::shl_u16::<BITS>(self)
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] right by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn shr_u16<const BITS: i32>(self) -> Self {
        <Self as Vec256Impl>::shr_u16::<BITS>(self)
    }

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn add_u8(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::add_u8(self, other)
    }

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// stopping at 255.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::saturating_add_u8(self, other)
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::sub_u8(self, other)
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// stopping at 0.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::saturating_sub_u8(self, other)
    }

    /// Add either another [`Vec256`] or a `u32` to every `u32` of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn add_u32(self, other: impl Operand256<u32, C>) -> Self {
        <Self as Vec256Impl>::add_u32(self, other)
    }

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec256`] to either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn eq_u8(self, other: impl Operand256<u8, C>) -> Self {
        <Self as Vec256Impl>::eq_u8(self, other)
    }

    /// Check if every byte of this [`Vec256`] equals either another [`Vec256`] or a `u8`.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand256<u8, C>) -> bool {
        <Self as Vec256Impl>::all_eq_u8(self, other)
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn lt_i8(self, other: impl Operand256<i8, C>) -> Self {
        <Self as Vec256Impl>::lt_i8(self, other)
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn gt_i8(self, other: impl Operand256<i8, C>) -> Self {
        <Self as Vec256Impl>::gt_i8(self, other)
    }

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Combine the bytes of this [`Vec256`] and either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the same byte of `mask` is `0xFF`,
    /// or from `self` if it's `0x00`.
    /// Other values in `mask` give different results on different CPUs.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn blend(self, other: impl Operand256<u8, C>, mask: Self) -> Self {
        <Self as Vec256Impl>::blend(self, other, mask)
    }

    // ----------------------------------------
    // `u32`-wise ops
    // ----------------------------------------

    /// Interleave the low 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s0, o0, s1, o1, s4, o4, s5, o5]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn unpack_lo_u32(self, other: Self) -> Self {
        <Self as Vec256Impl>::unpack_lo_u32(self, other)
    }

    /// Interleave the high 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s2, o2, s3, o3, s6, o6, s7, o7]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn unpack_hi_u32(self, other: Self) -> Self {
        <Self as Vec256Impl>::unpack_hi_u32(self, other)
    }

    // ----------------------------------------
    // Operations on 128-bit halves
    // ----------------------------------------

    /// Create a [`Vec256`] from its low and high 128 bits.
    #[cfg_attr(all(not(test), target_arch = "x86_64"), expect(dead_code))]
    #[inline(always)]
    pub fn from_halves(low: Vec128<C>, high: Vec128<C>) -> Self {
        <Self as Vec256Impl>::from_halves(low, high)
    }

    /// Get the low and high 128 bits of this [`Vec256`].
    #[cfg_attr(all(not(test), target_arch = "x86_64"), expect(dead_code))]
    #[inline(always)]
    pub fn into_halves(self) -> (Vec128<C>, Vec128<C>) {
        <Self as Vec256Impl>::into_halves(self)
    }

    /// Get a [`Vec256`] made of the low 128 bits of this [`Vec256`],
    /// followed by the low 128 bits of `other`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn join_lo_halves(self, other: Self) -> Self {
        <Self as Vec256Impl>::join_lo_halves(self, other)
    }

    /// Get a [`Vec256`] made of the high 128 bits of this [`Vec256`],
    /// followed by the high 128 bits of `other`.
    #[cfg_attr(all(not(test), target_arch = "aarch64"), expect(dead_code))]
    #[inline(always)]
    pub fn join_hi_halves(self, other: Self) -> Self {
        <Self as Vec256Impl>::join_hi_halves(self, other)
    }
}

// Operators `!`, `&`, `|`, `^`, `&=`, `|=` and `^=`, which call `not`, `and`, `or` and `xor`.
// As with those methods, the right-hand side can be a vector or a `u8`, so `v & 0x0f` works.

impl<C: CpuSimd> Not for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Vec256::not(self)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitAnd<T> for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self {
        self.and(rhs)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitAndAssign<T> for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.and(rhs);
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitOr<T> for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self {
        self.or(rhs)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitOrAssign<T> for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.or(rhs);
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitXor<T> for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self {
        self.xor(rhs)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitXorAssign<T> for Vec256<C>
where
    Self: Vec256Impl<Cpu = C>,
{
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.xor(rhs);
    }
}
