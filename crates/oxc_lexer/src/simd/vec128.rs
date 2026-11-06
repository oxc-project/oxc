//! [`Vec128`], a 128-bit SIMD vector which is generic over the CPU type.
//!
//! [`Vec128Impl`] holds the methods which every CPU type's [`Vec128`] has.
//! The inherent methods here are generic over the CPU type. Each calls the trait method
//! of the same name, so code can call the methods without the trait in scope.
//! Each CPU type implements the trait in its own module (`avx2` or `neon`),
//! along with the methods which only it has.
//!
//! The `!`, `&`, `|` and `^` operators for [`Vec128`] are here too.

#![expect(clippy::inline_always, reason = "wrappers must inline into the function calling them")]

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::{
    CpuSimd, VecData128,
    arrays::{IntArray64, IntArray128},
    operand::Operand128,
};

/// 128-bit SIMD vector.
///
/// Holds a `C`, so a vector can only exist if the CPU supports that feature set.
/// `C` is zero-sized, so [`Vec128`] is the same size as the type it wraps.
///
/// The methods which every CPU type has are in [`Vec128Impl`], and are also inherent methods.
/// The `avx2` and `neon` modules have the methods which only one CPU type has.
#[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
pub struct Vec128<C: CpuSimd> {
    pub(super) inner: <C::Vec128 as Vec128Impl>::Inner,
    pub(super) cpu: C,
}

// `Clone` and `Copy` are implemented by hand, because `#[derive(Copy)]` would add a bound
// requiring the type of `inner` to be `Copy`. Checking that bound requires
// `Vec128<C>: Vec128Impl`, which requires `Vec128<C>: Copy`, which is a cycle.
// `#[derive(Clone)]` has the same problem.
#[expect(clippy::expl_impl_clone_on_copy)]
impl<C: CpuSimd> Clone for Vec128<C> {
    /// Copy this [`Vec128`].
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: CpuSimd> Copy for Vec128<C> {}

/// Methods of [`Vec128`] which code that is generic over the CPU type can call.
///
/// Each CPU type's [`Vec128`] implements this trait.
/// Each method is also an inherent method of [`Vec128`],
/// which can be called without this trait in scope.
pub trait Vec128Impl: Copy {
    /// CPU feature set which this vector is for.
    type Cpu: CpuSimd;
    /// Type which this vector wraps.
    type Inner: Copy;

    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec128`] with all bits zero.
    fn zero(cpu: Self::Cpu) -> Self;

    /// Create a [`Vec128`] by repeating `n` 16 times.
    fn splat(n: u8, cpu: Self::Cpu) -> Self;

    /// Create a [`Vec128`] by repeating `n` 4 times.
    fn splat_u32(n: u32, cpu: Self::Cpu) -> Self;

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get the [`CpuSimd`] token from this [`Vec128`].
    fn cpu(self) -> Self::Cpu;

    // ----------------------------------------
    // Loads and stores
    // ----------------------------------------

    /// Read a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 16 bytes.
    /// * `ptr` must be aligned on 16.
    unsafe fn load(ptr: *const u8, cpu: Self::Cpu) -> Self;

    /// Read a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 16 bytes.
    unsafe fn load_unaligned(ptr: *const u8, cpu: Self::Cpu) -> Self;

    /// Read 8 bytes at pointer `ptr` into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 8 bytes.
    unsafe fn load_low_unaligned(ptr: *const u8, cpu: Self::Cpu) -> Self;

    /// Read a [`Vec128`] from a [`VecData128`], which is aligned on 16.
    fn read(data: &VecData128, cpu: Self::Cpu) -> Self;

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    fn read_unaligned(slice: &impl IntArray128, cpu: Self::Cpu) -> Self;

    /// Read a slice of integers into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// `slice` can be any slice of integers which is 8 bytes in total
    /// (e.g. `&[u8; 8]`, `&[i32; 2]`).
    fn read_low_unaligned(slice: &impl IntArray64, cpu: Self::Cpu) -> Self;

    /// Store a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for writing 16 bytes.
    /// * `ptr` must be aligned on 16.
    unsafe fn store(self, ptr: *mut u8);

    /// Store a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 16 bytes.
    unsafe fn store_unaligned(self, ptr: *mut u8);

    /// Store the low 8 bytes of a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 8 bytes.
    unsafe fn store_low_unaligned(self, ptr: *mut u8);

    /// Write a [`Vec128`] into a [`VecData128`], which is aligned on 16.
    fn write(self, dst: &mut VecData128);

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec128`] are zero.
    fn is_zero(self) -> bool;

    /// Invert all the bits of this [`Vec128`].
    fn not(self) -> Self;

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    fn and(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    fn or(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    fn xor(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// wrapping on overflow.
    fn add_u8(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    fn saturating_add_u8(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    fn sub_u8(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    fn saturating_sub_u8(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec128`] to either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    fn eq_u8(self, other: impl Operand128<u8, Self::Cpu>) -> Self;

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    fn all_eq_u8(self, other: impl Operand128<u8, Self::Cpu>) -> bool;

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    fn lt_i8(self, other: impl Operand128<i8, Self::Cpu>) -> Self;

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    fn gt_i8(self, other: impl Operand128<i8, Self::Cpu>) -> Self;

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    fn shr_bytes<const BYTES: i32>(self) -> Self;

    /// Rearrange the bytes of this [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// - If `control[i]` is less than 16, `result[i]` is `self[control[i]]`.
    /// - If the top bit of `control[i]` is set, `result[i]` is 0.
    /// - Otherwise, `result[i]` depends on the CPU.
    ///
    /// Often used as a 16-entry lookup table, with `self` as the table.
    fn shuffle_u8s(self, control: Self) -> Self;

    /// Combine the bytes of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the same byte of `mask` is `0xFF`,
    /// or from `self` if it's `0x00`.
    /// Other values in `mask` give different results on different CPUs.
    fn blend(self, other: impl Operand128<u8, Self::Cpu>, mask: Self) -> Self;

    // ----------------------------------------
    // `u64`-wise ops
    // ----------------------------------------

    /// Get one of the 2 `u64`s of this [`Vec128`].
    ///
    /// `INDEX` is 0 for the low 8 bytes, or 1 for the high 8 bytes.
    fn get_u64<const INDEX: i32>(self) -> u64;
}

/// Methods which are generic over the CPU type.
///
/// Each calls the [`Vec128Impl`] method of the same name, which each CPU type implements.
/// Calling them doesn't need [`Vec128Impl`] in scope.
impl<C: CpuSimd> Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    // ----------------------------------------
    // Creation methods
    // ----------------------------------------

    /// Create a [`Vec128`] with all bits zero.
    #[cfg_attr(not(any(test, target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn zero(cpu: C) -> Self {
        <Self as Vec128Impl>::zero(cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 16 times.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn splat(n: u8, cpu: C) -> Self {
        <Self as Vec128Impl>::splat(n, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 4 times.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: C) -> Self {
        <Self as Vec128Impl>::splat_u32(n, cpu)
    }

    // ----------------------------------------
    // Accessors
    // ----------------------------------------

    /// Get the [`CpuSimd`] token from this [`Vec128`].
    #[inline(always)]
    pub fn cpu(self) -> C {
        <Self as Vec128Impl>::cpu(self)
    }

    // ----------------------------------------
    // Loads and stores
    // ----------------------------------------

    /// Read a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for reading 16 bytes.
    /// * `ptr` must be aligned on 16.
    #[cfg_attr(not(any(test, target_arch = "x86_64")), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn load(ptr: *const u8, cpu: C) -> Self {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec128Impl>::load(ptr, cpu) }
    }

    /// Read a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 16 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: C) -> Self {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec128Impl>::load_unaligned(ptr, cpu) }
    }

    /// Read 8 bytes at pointer `ptr` into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 8 bytes.
    #[inline(always)]
    pub unsafe fn load_low_unaligned(ptr: *const u8, cpu: C) -> Self {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec128Impl>::load_low_unaligned(ptr, cpu) }
    }

    /// Read a [`Vec128`] from a [`VecData128`], which is aligned on 16.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn read(data: &VecData128, cpu: C) -> Self {
        <Self as Vec128Impl>::read(data, cpu)
    }

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray128, cpu: C) -> Self {
        <Self as Vec128Impl>::read_unaligned(slice, cpu)
    }

    /// Read a slice of integers into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// `slice` can be any slice of integers which is 8 bytes in total
    /// (e.g. `&[u8; 8]`, `&[i32; 2]`).
    #[inline(always)]
    pub fn read_low_unaligned(slice: &impl IntArray64, cpu: C) -> Self {
        <Self as Vec128Impl>::read_low_unaligned(slice, cpu)
    }

    /// Store a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be valid for writing 16 bytes.
    /// * `ptr` must be aligned on 16.
    #[cfg_attr(not(any(test, target_arch = "x86_64")), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec128Impl>::store(self, ptr) }
    }

    /// Store a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 16 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec128Impl>::store_unaligned(self, ptr) }
    }

    /// Store the low 8 bytes of a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 8 bytes.
    #[inline(always)]
    pub unsafe fn store_low_unaligned(self, ptr: *mut u8) {
        // SAFETY: Caller guarantees the same requirements as the trait method has
        unsafe { <Self as Vec128Impl>::store_low_unaligned(self, ptr) }
    }

    /// Write a [`Vec128`] into a [`VecData128`], which is aligned on 16.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn write(self, dst: &mut VecData128) {
        <Self as Vec128Impl>::write(self, dst);
    }

    // ----------------------------------------
    // Bitwise ops
    // ----------------------------------------

    /// Check if all the bits of this [`Vec128`] are zero.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        <Self as Vec128Impl>::is_zero(self)
    }

    /// Invert all the bits of this [`Vec128`].
    #[inline(always)]
    pub fn not(self) -> Self {
        <Self as Vec128Impl>::not(self)
    }

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::and(self, other)
    }

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::or(self, other)
    }

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::xor(self, other)
    }

    // ----------------------------------------
    // Arithmetic ops
    // ----------------------------------------

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[cfg_attr(not(any(test, target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn add_u8(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::add_u8(self, other)
    }

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    #[cfg_attr(not(any(test, target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::saturating_add_u8(self, other)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::sub_u8(self, other)
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::saturating_sub_u8(self, other)
    }

    // ----------------------------------------
    // Comparison
    // ----------------------------------------

    /// Compare every byte of this [`Vec128`] to either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    pub fn eq_u8(self, other: impl Operand128<u8, C>) -> Self {
        <Self as Vec128Impl>::eq_u8(self, other)
    }

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand128<u8, C>) -> bool {
        <Self as Vec128Impl>::all_eq_u8(self, other)
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    pub fn lt_i8(self, other: impl Operand128<i8, C>) -> Self {
        <Self as Vec128Impl>::lt_i8(self, other)
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn gt_i8(self, other: impl Operand128<i8, C>) -> Self {
        <Self as Vec128Impl>::gt_i8(self, other)
    }

    // ----------------------------------------
    // Bytewise ops
    // ----------------------------------------

    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    #[inline(always)]
    pub fn shr_bytes<const BYTES: i32>(self) -> Self {
        <Self as Vec128Impl>::shr_bytes::<BYTES>(self)
    }

    /// Rearrange the bytes of this [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// - If `control[i]` is less than 16, `result[i]` is `self[control[i]]`.
    /// - If the top bit of `control[i]` is set, `result[i]` is 0.
    /// - Otherwise, `result[i]` depends on the CPU.
    ///
    /// Often used as a 16-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Self) -> Self {
        <Self as Vec128Impl>::shuffle_u8s(self, control)
    }

    /// Combine the bytes of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the same byte of `mask` is `0xFF`,
    /// or from `self` if it's `0x00`.
    /// Other values in `mask` give different results on different CPUs.
    #[inline(always)]
    pub fn blend(self, other: impl Operand128<u8, C>, mask: Self) -> Self {
        <Self as Vec128Impl>::blend(self, other, mask)
    }

    // ----------------------------------------
    // `u64`-wise ops
    // ----------------------------------------

    /// Get one of the 2 `u64`s of this [`Vec128`].
    ///
    /// `INDEX` is 0 for the low 8 bytes, or 1 for the high 8 bytes.
    #[inline(always)]
    pub fn get_u64<const INDEX: i32>(self) -> u64 {
        <Self as Vec128Impl>::get_u64::<INDEX>(self)
    }
}

// Operators `!`, `&`, `|`, `^`, `&=`, `|=` and `^=`, which call `not`, `and`, `or` and `xor`.
// As with those methods, the right-hand side can be a vector or a `u8`, so `v & 0x0f` works.

impl<C: CpuSimd> Not for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Vec128::not(self)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitAnd<T> for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self {
        self.and(rhs)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitAndAssign<T> for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.and(rhs);
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitOr<T> for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self {
        self.or(rhs)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitOrAssign<T> for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.or(rhs);
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitXor<T> for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self {
        self.xor(rhs)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitXorAssign<T> for Vec128<C>
where
    Self: Vec128Impl<Cpu = C>,
{
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.xor(rhs);
    }
}
