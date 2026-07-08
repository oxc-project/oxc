//! Utility functions used by the expansion of the [`multi_vec!`] macro.
//!
//! [`multi_vec!`]: super::multi_vec

use std::{
    ptr::{self, NonNull},
    slice,
};

use oxc_index::{Idx, IndexSlice};

/// Create an [`IndexSlice`] from a pointer and length.
///
/// # SAFETY
///
/// Same requirements as [`slice::from_raw_parts`]: `ptr` must be valid for reads of
/// `len` consecutive initialized values of `T` (dangling but aligned is sufficient if
/// `len == 0`), which must not be mutated for the duration of lifetime `'v`.
#[inline]
pub unsafe fn index_slice_from_raw_parts<'v, I: Idx, T>(
    ptr: NonNull<u8>,
    len: usize,
) -> &'v IndexSlice<I, [T]> {
    // SAFETY: Caller guarantees `slice::from_raw_parts`'s requirements
    IndexSlice::from_slice(unsafe { slice::from_raw_parts(ptr.cast::<T>().as_ptr(), len) })
}

/// Create a mutable [`IndexSlice`] from a pointer and length.
///
/// # SAFETY
///
/// Same requirements as [`slice::from_raw_parts_mut`]: `ptr` must be valid for reads
/// and writes of `len` consecutive initialized values of `T` (dangling but aligned is
/// sufficient if `len == 0`), which must not be accessed through any other pointer for
/// the duration of lifetime `'v`.
#[inline]
pub unsafe fn index_slice_from_raw_parts_mut<'v, I: Idx, T>(
    ptr: NonNull<u8>,
    len: usize,
) -> &'v mut IndexSlice<I, [T]> {
    // SAFETY: Caller guarantees `slice::from_raw_parts_mut`'s requirements
    IndexSlice::from_slice_mut(unsafe { slice::from_raw_parts_mut(ptr.cast::<T>().as_ptr(), len) })
}

/// Drop the first `len` values of a field array, in place.
///
/// If a value's `Drop` panics, the remaining values in the array are leaked (not dropped).
///
/// # SAFETY
///
/// * `ptr` must be aligned for `T`, and valid for reads and writes of `len` consecutive
///   values of `T`, all initialized. (Dangling but aligned is sufficient if `len == 0`
///   or `T` is zero-sized.)
/// * The values must not be used after this call.
#[inline]
pub unsafe fn drop_column<T>(ptr: NonNull<u8>, len: usize) {
    // SAFETY: Caller guarantees `ptr` is aligned and valid for reads and writes of `len`
    // initialized values of `T`, which are never used again
    unsafe { ptr::drop_in_place(ptr::slice_from_raw_parts_mut(ptr.cast::<T>().as_ptr(), len)) }
}
