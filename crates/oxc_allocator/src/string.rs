use std::{
    mem,
    ops::{Deref, DerefMut},
    ptr, str,
};

use crate::{Allocator, Vec};

/// A bump-allocated string.
pub struct String<'a> {
    vec: Vec<'a, u8>,
}

impl<'a> String<'a> {
    /// Constructs a new empty `String`.
    #[inline]
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self { vec: Vec::new_in(allocator) }
    }

    /// Constructs a `String` from a `&str`.
    #[inline]
    pub fn from_str_in(s: &str, allocator: &'a Allocator) -> Self {
        // code taken from `bumpalo::collections::String::from_str_in`
        let len = s.len();
        let mut t = String::with_capacity_in(len, allocator);
        // SAFETY:
        // * `src` is valid for reads of `s.len()` bytes by virtue of being an allocated `&str`.
        // * `dst` is valid for writes of `s.len()` bytes as `String::with_capacity_in(s.len(), bump)`
        //   above guarantees that.
        // * Alignment is not relevant as `u8` has no alignment requirements.
        // * Source and destination ranges cannot overlap as we just reserved the destination
        //   range from the bump.
        unsafe { ptr::copy_nonoverlapping(s.as_ptr(), t.vec.as_mut_ptr(), len) };
        // SAFETY: We reserved sufficent capacity for the string above.
        // The elements at `0..len` were initialized by `copy_nonoverlapping` above.
        unsafe { t.vec.set_len(len) };
        t
    }

    /// Constructs a new empty `String` with the specified capacity.
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'a Allocator) -> Self {
        Self { vec: Vec::with_capacity_in(capacity, allocator) }
    }

    /// Converts a `String` into a `&str`.
    #[inline]
    pub fn into_bump_str(self) -> &'a str {
        #![allow(
            clippy::undocumented_unsafe_blocks,
            clippy::missing_transmute_annotations,
            clippy::forget_non_drop
        )]
        // code taken from `bumpalo::collections::String::into_bump_str`
        let s = unsafe {
            let s = self.as_str();
            mem::transmute(s)
        };
        mem::forget(self);
        s
    }

    /// Appends a given string slice to the end of this string.
    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.vec.extend_from_slice_copy(string.as_bytes());
    }

    /// Appends a given `char` to the end of this string.
    #[inline]
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.vec.push(ch as u8),
            _ => self.vec.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    /// Extracts a string slice containing the entire `String`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self
    }
}

impl Deref for String<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            str::from_utf8_unchecked(&self.vec)
        }
    }
}

impl DerefMut for String<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            str::from_utf8_unchecked_mut(&mut self.vec)
        }
    }
}

// code taken from `bumpalo::collections::Vec`
impl<'alloc> Vec<'alloc, u8> {
    /// Copies all elements in the slice `other` and appends them to the `Vec`.
    ///
    /// Note that this function is same as [`extend_from_slice`] except that it is optimized for
    /// slices of types that implement the `Copy` trait. If and when Rust gets specialization
    /// this function will likely be deprecated (but still available).
    pub fn extend_from_slice_copy(&mut self, other: &[u8]) {
        // Reserve space in the Vec for the values to be added
        self.reserve(other.len());

        // Copy values into the space that was just reserved
        // SAFETY:
        // * `self` has enough capacity to store `other.len()` more items as `self.reserve(other.len())`
        //   above guarantees that.
        // * Source and destination data ranges cannot overlap as we just reserved the destination
        //   range from the bump.
        unsafe {
            self.extend_from_slice_copy_unchecked(other);
        }
    }

    /// Helper method to copy all of the items in `other` and append them to the end of `self`.
    ///
    /// SAFETY:
    ///   * The caller is responsible for:
    ///       * calling [`reserve`](Self::reserve) beforehand to guarantee that there is enough
    ///         capacity to store `other.len()` more items.
    ///       * guaranteeing that `self` and `other` do not overlap.
    unsafe fn extend_from_slice_copy_unchecked(&mut self, other: &[u8]) {
        let old_len = self.len();
        debug_assert!(old_len + other.len() <= self.capacity());

        // SAFETY:
        // * `src` is valid for reads of `other.len()` values by virtue of being a `&[T]`.
        // * `dst` is valid for writes of `other.len()` bytes because the caller of this
        //   method is required to `reserve` capacity to store at least `other.len()` items
        //   beforehand.
        // * Because `src` is a `&[T]` and dst is a `&[T]` within the `Vec<T>`,
        //   `copy_nonoverlapping`'s alignment requirements are met.
        // * Caller is required to guarantee that the source and destination ranges cannot overlap
        unsafe {
            let src = other.as_ptr();
            let dst = self.as_mut_ptr().add(old_len);
            ptr::copy_nonoverlapping(src, dst, other.len());
            self.set_len(old_len + other.len());
        }
    }
}
