//! Arena String.
//!
//! See [`String`] for more details.

// All methods which just delegate to `bumpalo::collections::String` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use bumpalo::collections::String as BumpaloString;
use simdutf8::basic::from_utf8;
pub use simdutf8::basic::Utf8Error;

use crate::{Allocator, Vec};

/// Arena String.
///
/// UTF-8 encoded, growable string. Identical to [`std::string::String`] except that it stores
/// string contents in arena allocator.
//
// We wrap the inner `BumpaloString` in `ManuallyDrop` to make `String` non-Drop.
// `bumpalo::collections::String` is a wrapper around `bumpalo::collections::Vec<u8>`.
// Even though a `Vec<u8>` cannot require dropping (because `u8` is not `Drop`), `Vec<u8>` still has
// a `Drop` impl, which means `bumpalo::collections::String` is `Drop` too.
// We want to make it clear to compiler that `String` doesn't require dropping, so it doesn't
// produce pointless "drop guard" code to handle dropping a `String` in case of a panic.
#[derive(PartialOrd, Eq, Ord)]
pub struct String<'alloc>(ManuallyDrop<BumpaloString<'alloc>>);

impl<'alloc> String<'alloc> {
    /// Creates a new empty [`String`].
    ///
    /// Given that the `String` is empty, this will not allocate any initial
    /// buffer. While that means that this initial operation is very
    /// inexpensive, it may cause excessive allocation later when you add
    /// data. If you have an idea of how much data the `String` will hold,
    /// consider the [`with_capacity_in`] method to prevent excessive
    /// re-allocation.
    ///
    /// [`with_capacity_in`]: String::with_capacity_in
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> String<'alloc> {
        Self(ManuallyDrop::new(BumpaloString::new_in(allocator.bump())))
    }

    /// Creates a new empty [`String`] with specified capacity.
    ///
    /// `String`s have an internal buffer to hold their data. The capacity is
    /// the length of that buffer, and can be queried with the `capacity`
    /// method. This method creates an empty `String`, but one with an initial
    /// buffer that can hold `capacity` bytes. This is useful when you may be
    /// appending a bunch of data to the `String`, reducing the number of
    /// reallocations it needs to do.
    ///
    /// If the given capacity is `0`, no allocation will occur, and this method
    /// is identical to the [`new_in`] method.
    ///
    /// [`capacity`]: String::capacity
    /// [`new_in`]: String::new_in
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> String<'alloc> {
        Self(ManuallyDrop::new(BumpaloString::with_capacity_in(capacity, allocator.bump())))
    }

    /// Construct a new [`String`] from a string slice.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, String};
    ///
    /// let allocator = Allocator::default();
    ///
    /// let s = String::from_str_in("hello", &allocator);
    /// assert_eq!(s, "hello");
    /// ```
    #[inline(always)]
    pub fn from_str_in(s: &str, allocator: &'alloc Allocator) -> String<'alloc> {
        Self(ManuallyDrop::new(BumpaloString::from_str_in(s, allocator.bump())))
    }

    /// Convert `Vec<u8>` into [`String`].
    ///
    /// # Errors
    /// Returns [`Err`] if the `Vec` does not comprise a valid UTF-8 string.
    pub fn from_utf8(bytes: Vec<'alloc, u8>) -> Result<String<'alloc>, Utf8Error> {
        // Check vec comprises a valid UTF-8 string.
        from_utf8(&bytes)?;
        // SAFETY: We just checked it's a valid UTF-8 string
        let s = unsafe { Self::from_utf8_unchecked(bytes) };
        Ok(s)
    }

    /// Convert `Vec<u8>` into [`String`], without checking bytes comprise a valid UTF-8 string.
    ///
    /// Does not copy the contents of the `Vec`, converts in place. This is a zero-cost operation.
    ///
    /// # SAFETY
    /// Caller must ensure this `Vec<u8>` comprises a valid UTF-8 string.
    //
    // `#[inline(always)]` because this is a no-op at runtime
    #[expect(clippy::missing_safety_doc, clippy::unnecessary_safety_comment)]
    #[inline(always)]
    pub unsafe fn from_utf8_unchecked(bytes: Vec<'alloc, u8>) -> String<'alloc> {
        // Cannot use `bumpalo::String::from_utf8_unchecked` because it takes a `bumpalo::collections::Vec`,
        // and our inner `Vec` type is `allocator_api2::vec::Vec`.
        // SAFETY: Conversion is safe because both types store data in arena in same way.
        // Lifetime of returned `String` is same as lifetime of original `Vec<u8>`.
        let inner = ManuallyDrop::into_inner(bytes.0);
        let (ptr, len, capacity, bump) = inner.into_raw_parts_with_alloc();
        Self(ManuallyDrop::new(BumpaloString::from_raw_parts_in(ptr, len, capacity, bump)))
    }

    /// Creates a new [`String`] from a length, capacity, and pointer.
    ///
    /// # SAFETY
    ///
    /// This is highly unsafe, due to the number of invariants that aren't checked:
    ///
    /// * The memory at `ptr` needs to have been previously allocated by the same [`Allocator`].
    /// * `length` needs to be less than or equal to `capacity`.
    /// * `capacity` needs to be the correct value.
    ///
    /// Violating these may cause problems like corrupting the allocator's internal data structures.
    ///
    /// The ownership of `ptr` is effectively transferred to the `String` which may then deallocate,
    /// reallocate or change the contents of memory pointed to by the pointer at will. Ensure that
    /// nothing else uses the pointer after calling this function.
    ///
    /// # Examples
    /// ```
    /// use std::mem;
    /// use oxc_allocator::{Allocator, String};
    ///
    /// let allocator = Allocator::default();
    ///
    /// unsafe {
    ///     let mut s = String::from_str_in("hello", &allocator);
    ///     let ptr = s.as_mut_ptr();
    ///     let len = s.len();
    ///     let capacity = s.capacity();
    ///
    ///     let s = String::from_raw_parts_in(ptr, len, capacity, &allocator);
    ///
    ///     assert_eq!(s, "hello");
    /// }
    /// ```
    #[expect(clippy::missing_safety_doc, clippy::unnecessary_safety_comment)]
    #[inline(always)]
    pub unsafe fn from_raw_parts_in(
        buf: *mut u8,
        length: usize,
        capacity: usize,
        allocator: &'alloc Allocator,
    ) -> String<'alloc> {
        // SAFETY: Safety conditions of this method are the same as `BumpaloString`'s method
        let inner = BumpaloString::from_raw_parts_in(buf, length, capacity, allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Convert this `String<'alloc>` into an `&'alloc str`. This is analogous to
    /// [`std::string::String::into_boxed_str`].
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, String};
    ///
    /// let allocator = Allocator::default();
    ///
    /// let s = String::from_str_in("foo", &allocator);
    /// assert_eq!(s.into_bump_str(), "foo");
    /// ```
    #[inline(always)]
    pub fn into_bump_str(self) -> &'alloc str {
        let inner = ManuallyDrop::into_inner(self.0);
        inner.into_bump_str()
    }
}

// Provide access to all `bumpalo::String`'s methods via deref
impl<'alloc> Deref for String<'alloc> {
    type Target = BumpaloString<'alloc>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc> DerefMut for String<'alloc> {
    #[inline]
    fn deref_mut(&mut self) -> &mut BumpaloString<'alloc> {
        &mut self.0
    }
}

impl PartialEq for String<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

// `impl_eq!` macro copied from `bumpalo`
macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        impl<'a, 'alloc> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        impl<'a, 'alloc> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    };
}

impl_eq! { String<'alloc>, str }
impl_eq! { String<'alloc>, &'a str }
impl_eq! { std::borrow::Cow<'a, str>, String<'alloc> }
impl_eq! { std::string::String, String<'alloc> }

impl Display for String<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for String<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Hash for String<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}
