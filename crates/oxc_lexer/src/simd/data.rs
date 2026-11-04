//! Byte arrays which are aligned for SIMD vectors.
//!
//! [`VecData128`] is 16 bytes aligned on 16, and [`VecData256`] is 32 bytes aligned on 32.
//! Their `read` methods read them into a [`Vec128`] or [`Vec256`] with an aligned load,
//! and the vectors' `write` methods write back to them with an aligned store.
//!
//! Use them for lookup tables and other data which is read into vectors.
//! A read of aligned data never crosses a cache line.
//!
//! LLVM usually turns a read of a `const` table into a constant of its own, which it aligns itself.
//! So alignment matters most for `static` tables, which are indexed at runtime.
//!
//! Both types deref to a byte array, so can be indexed like an array, but not in const contexts.
//! To build a table in a `const` or `static`, use `as_bytes_mut` or `as_u32s_mut` instead.
//!
//! [`Vec128`]: super::Vec128
//! [`Vec256`]: super::Vec256

#![expect(clippy::inline_always, reason = "Trivial methods should be inlined")]

use std::{
    ops::{Deref, DerefMut},
    ptr,
};

use super::{
    CpuSimd, Vec128Impl, Vec256Impl,
    arrays::{IntArray128, IntArray256},
};

/// 16 bytes, aligned on 16, for reading into a [`Vec128`] with an aligned load.
///
/// Derefs to `[u8; 16]`, so can be indexed like an array, except in const contexts.
///
/// [`Vec128`]: super::Vec128
#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct VecData128([u8; 16]);

impl VecData128 {
    /// Create a [`VecData128`] from an array of bytes.
    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Create a [`VecData128`] from an array of `u32`s, each in native byte order.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub const fn from_u32s(values: [u32; 4]) -> Self {
        let mut bytes = [[0; 4]; 4];
        let mut i = 0;
        while i < 4 {
            bytes[i] = values[i].to_ne_bytes();
            i += 1;
        }
        Self(*bytes.as_flattened().first_chunk().unwrap())
    }

    /// Get a reference to the bytes of this [`VecData128`].
    ///
    /// Same as [`Deref`], but can be used in const contexts.
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Get a mutable reference to the bytes of this [`VecData128`].
    ///
    /// Same as [`DerefMut`], but can be used in const contexts.
    #[inline(always)]
    pub const fn as_bytes_mut(&mut self) -> &mut [u8; 16] {
        &mut self.0
    }

    /// Get a reference to the contents of this [`VecData128`], as `u32`s in native byte order.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub const fn as_u32s(&self) -> &[u32; 4] {
        // SAFETY: `VecData128` is 16 bytes, aligned on 16. `[u32; 4]` is 16 bytes, aligned on 4.
        // Any bit pattern is valid for both types.
        unsafe { &*ptr::from_ref(self).cast::<[u32; 4]>() }
    }

    /// Get a mutable reference to the contents of this [`VecData128`], as `u32`s in native byte order.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub const fn as_u32s_mut(&mut self) -> &mut [u32; 4] {
        // SAFETY: `VecData128` is 16 bytes, aligned on 16. `[u32; 4]` is 16 bytes, aligned on 4.
        // Any bit pattern is valid for both types.
        unsafe { &mut *ptr::from_mut(self).cast::<[u32; 4]>() }
    }

    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for aligned reads of 16 bytes.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub const fn as_ptr(&self) -> *const u8 {
        self.0.as_slice().as_ptr()
    }

    /// Get a `*mut u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for aligned writes of 16 bytes.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    /// Read this [`VecData128`] into a [`Vec128`]. Same as [`Vec128::read`].
    ///
    /// [`Vec128`]: super::Vec128
    /// [`Vec128::read`]: super::Vec128::read
    #[inline(always)]
    pub fn read<C: CpuSimd>(&self, cpu: C) -> C::Vec128 {
        C::Vec128::read(self, cpu)
    }
}

// SAFETY: `VecData128` contains 16 initialized bytes
unsafe impl IntArray128 for VecData128 {
    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for aligned reads of 16 bytes.
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self.as_ptr()
    }
}

impl Deref for VecData128 {
    type Target = [u8; 16];

    /// Get a reference to the bytes of this [`VecData128`].
    #[inline(always)]
    fn deref(&self) -> &[u8; 16] {
        self.as_bytes()
    }
}

impl DerefMut for VecData128 {
    /// Get a mutable reference to the bytes of this [`VecData128`].
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8; 16] {
        self.as_bytes_mut()
    }
}

/// 32 bytes, aligned on 32, for reading into a [`Vec256`] with an aligned load.
///
/// Derefs to `[u8; 32]`, so can be indexed like an array, except in const contexts.
///
/// [`Vec256`]: super::Vec256
#[derive(Clone, Copy)]
#[repr(C, align(32))]
pub struct VecData256([u8; 32]);

impl VecData256 {
    /// Create a [`VecData256`] from an array of bytes.
    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a [`VecData256`] from an array of `u32`s, each in native byte order.
    #[inline(always)]
    pub const fn from_u32s(values: [u32; 8]) -> Self {
        let mut bytes = [[0; 4]; 8];
        let mut i = 0;
        while i < 8 {
            bytes[i] = values[i].to_ne_bytes();
            i += 1;
        }
        Self(*bytes.as_flattened().first_chunk().unwrap())
    }

    /// Get a reference to the bytes of this [`VecData256`].
    ///
    /// Same as [`Deref`], but can be used in const contexts.
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get a mutable reference to the bytes of this [`VecData256`].
    ///
    /// Same as [`DerefMut`], but can be used in const contexts.
    #[inline(always)]
    pub const fn as_bytes_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }

    /// Get a reference to the contents of this [`VecData256`], as `u32`s in native byte order.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub const fn as_u32s(&self) -> &[u32; 8] {
        // SAFETY: `VecData256` is 32 bytes, aligned on 32. `[u32; 8]` is 32 bytes, aligned on 4.
        // Any bit pattern is valid for both types.
        unsafe { &*ptr::from_ref(self).cast::<[u32; 8]>() }
    }

    /// Get a mutable reference to the contents of this [`VecData256`], as `u32`s in native byte order.
    #[inline(always)]
    pub const fn as_u32s_mut(&mut self) -> &mut [u32; 8] {
        // SAFETY: `VecData256` is 32 bytes, aligned on 32. `[u32; 8]` is 32 bytes, aligned on 4.
        // Any bit pattern is valid for both types.
        unsafe { &mut *ptr::from_mut(self).cast::<[u32; 8]>() }
    }

    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for aligned reads of 32 bytes.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub const fn as_ptr(&self) -> *const u8 {
        self.0.as_slice().as_ptr()
    }

    /// Get a `*mut u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for aligned writes of 32 bytes.
    #[cfg_attr(not(any(test, target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    /// Get the low and high 16 bytes of this [`VecData256`], as 2 [`VecData128`]s.
    #[inline(always)]
    pub const fn as_halves(&self) -> &[VecData128; 2] {
        // SAFETY: `VecData256` is 32 bytes, aligned on 32. `[VecData128; 2]` is 32 bytes, aligned on 16.
        // Both types are just bytes, so any bit pattern is valid for both.
        unsafe { &*ptr::from_ref(self).cast::<[VecData128; 2]>() }
    }

    /// Read this [`VecData256`] into a [`Vec256`]. Same as [`Vec256::read`].
    ///
    /// [`Vec256`]: super::Vec256
    /// [`Vec256::read`]: super::Vec256::read
    #[inline(always)]
    pub fn read<C: CpuSimd>(&self, cpu: C) -> C::Vec256 {
        C::Vec256::read(self, cpu)
    }
}

// SAFETY: `VecData256` contains 32 initialized bytes
unsafe impl IntArray256 for VecData256 {
    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for aligned reads of 32 bytes.
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self.as_ptr()
    }
}

impl Deref for VecData256 {
    type Target = [u8; 32];

    /// Get a reference to the bytes of this [`VecData256`].
    #[inline(always)]
    fn deref(&self) -> &[u8; 32] {
        self.as_bytes()
    }
}

impl DerefMut for VecData256 {
    /// Get a mutable reference to the bytes of this [`VecData256`].
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8; 32] {
        self.as_bytes_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use super::*;

    #[test]
    fn vec_data128() {
        assert_eq!((size_of::<VecData128>(), align_of::<VecData128>()), (16, 16));

        let bytes: [u8; 16] = array::from_fn(|i| u8::try_from(i).unwrap() * 3);
        let mut data = VecData128::from_bytes(bytes);
        assert_eq!(*data.as_bytes(), bytes);
        assert_eq!(data[5], 15);

        data[5] = 0xFF;
        assert_eq!(data.as_bytes()[5], 0xFF);

        data.as_bytes_mut()[6] = 0xFE;
        assert_eq!(data.as_bytes()[6], 0xFE);

        data.as_u32s_mut()[2] = 0x8234_5678;
        assert_eq!(data.as_bytes()[8..12], 0x8234_5678u32.to_ne_bytes());

        let values = [1, 0x8000_0000, 3, u32::MAX];
        let data = VecData128::from_u32s(values);
        assert_eq!(data[..], *values.map(u32::to_ne_bytes).as_flattened());
        assert_eq!(*data.as_u32s(), values);
    }

    #[test]
    fn vec_data128_pointers() {
        let mut data = VecData128::from_bytes([0; 16]);
        let start = ptr::from_ref(&data).cast::<u8>();
        assert_eq!(start.addr() % 16, 0);
        assert_eq!(data.as_ptr(), start);
        assert_eq!(IntArray128::as_ptr(&data), start);
        assert_eq!(data.as_mut_ptr().cast_const(), start);
    }

    #[test]
    fn vec_data256() {
        assert_eq!((size_of::<VecData256>(), align_of::<VecData256>()), (32, 32));

        let bytes: [u8; 32] = array::from_fn(|i| u8::try_from(i).unwrap() * 3);
        let mut data = VecData256::from_bytes(bytes);
        assert_eq!(*data.as_bytes(), bytes);
        assert_eq!(data[20], 60);

        let [low, high] = data.as_halves();
        assert_eq!(*low.as_bytes(), bytes[..16]);
        assert_eq!(*high.as_bytes(), bytes[16..]);

        data[20] = 0xFF;
        assert_eq!(data.as_bytes()[20], 0xFF);

        data.as_bytes_mut()[21] = 0xFE;
        assert_eq!(data.as_bytes()[21], 0xFE);

        data.as_u32s_mut()[6] = 0x8234_5678;
        assert_eq!(data.as_bytes()[24..28], 0x8234_5678u32.to_ne_bytes());

        let values = [1, 2, 3, 0x8000_0000, 5, 6, 7, u32::MAX];
        let data = VecData256::from_u32s(values);
        assert_eq!(data[..], *values.map(u32::to_ne_bytes).as_flattened());
        assert_eq!(*data.as_u32s(), values);
    }

    #[test]
    fn vec_data256_pointers() {
        let mut data = VecData256::from_bytes([0; 32]);
        let start = ptr::from_ref(&data).cast::<u8>();
        assert_eq!(start.addr() % 32, 0);
        assert_eq!(data.as_ptr(), start);
        assert_eq!(IntArray256::as_ptr(&data), start);
        assert_eq!(data.as_mut_ptr().cast_const(), start);

        let [low, high] = data.as_halves();
        assert_eq!(low.as_ptr(), start);
        assert_eq!(high.as_ptr(), start.wrapping_add(16));
    }
}
