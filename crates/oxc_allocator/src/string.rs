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
    ptr,
};

use bumpalo::collections::String as BumpaloString;
pub use simdutf8::basic::Utf8Error;
use simdutf8::basic::from_utf8;

use oxc_data_structures::assert_unchecked;

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
    #[inline(always)]
    pub unsafe fn from_utf8_unchecked(bytes: Vec<'alloc, u8>) -> String<'alloc> {
        // Cannot use `bumpalo::String::from_utf8_unchecked` because it takes a `bumpalo::collections::Vec`,
        // and our inner `Vec` type is our own `crate::vec2::Vec`.

        // Wrap `bytes` in `ManuallyDrop` to prevent its memory getting freed when `bytes`
        // goes out of scope at end of this function.
        // This shouldn't actually be required as `Vec` is already non-`Drop`,
        // but `ManuallyDrop` has no runtime cost, so it doesn't hurt to make sure.
        let mut bytes = ManuallyDrop::new(bytes);

        let ptr = bytes.as_mut_ptr();
        let len = bytes.len();
        let capacity = bytes.capacity();
        let bump = bytes.bump();
        // SAFETY: Conversion is safe because both types store data in arena in same way.
        // Lifetime of returned `String` is same as lifetime of original `Vec<u8>`.
        unsafe {
            Self(ManuallyDrop::new(BumpaloString::from_raw_parts_in(ptr, len, capacity, bump)))
        }
    }

    /// Create a new [`String`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, String};
    ///
    /// let allocator = Allocator::default();
    /// let string = String::from_strs_array_in(["hello", "world"], &allocator);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `usize::MAX`.
    #[inline]
    pub fn from_strs_array_in<const N: usize>(
        strings: [&str; N],
        allocator: &'alloc Allocator,
    ) -> String<'alloc> {
        // Calculate total length of all the strings concatenated.
        //
        // We have to use `checked_add` here to guard against additions wrapping around
        // if some of the input `&str`s are very long, or there's many of them.
        //
        // However, `&str`s have max length of `isize::MAX`.
        // https://users.rust-lang.org/t/does-str-reliably-have-length-isize-max/126777
        // Use `assert_unchecked!` to communicate this invariant to compiler, which allows it to
        // optimize out the overflow checks where some of `strings` are static, so their size is known.
        //
        // e.g. `String::from_strs_array_in(["__vite_ssr_import_", str, "__"])`, for example,
        // requires no checks at all, because the static parts have total length of 20 bytes,
        // and `str` has max length of `isize::MAX`. `isize::MAX as usize + 20` cannot overflow `usize`.
        // Compiler can see that, and removes the overflow check.
        // https://godbolt.org/z/MGh44Yz5d
        #[expect(clippy::checked_conversions)]
        let total_len = strings.iter().fold(0usize, |total_len, s| {
            let len = s.len();
            // SAFETY: `&str`s have maximum length of `isize::MAX`
            unsafe { assert_unchecked!(len <= (isize::MAX as usize)) };
            total_len.checked_add(len).unwrap()
        });

        // Create actual `String` in a separate function, to ensure that `from_strs_array_in`
        // is inlined, so that compiler has knowledge to remove the overflow checks above.
        // When some of `strings` are static, this function is usually only a single instruction.
        // Compiler can choose whether or not to inline `from_strs_array_with_total_len`.
        // SAFETY: `total_len` has been calculated correctly above
        unsafe { Self::from_strs_array_with_total_len_in(strings, total_len, allocator) }
    }

    /// Create a new [`String`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`, with provided `total_len`.
    ///
    /// # SAFETY
    /// `total_len` must be the total length of all `strings` concatenated.
    unsafe fn from_strs_array_with_total_len_in<const N: usize>(
        strings: [&str; N],
        total_len: usize,
        allocator: &'alloc Allocator,
    ) -> String<'alloc> {
        // Create a `Vec<u8>` with sufficient capacity to contain all the `strings` concatenated.
        // Note: If `total_len == 0`, this does not allocate, and `vec.as_mut_ptr()` is a dangling pointer.
        let mut vec = Vec::with_capacity_in(total_len, allocator);

        let mut dst = vec.as_mut_ptr();
        for str in strings {
            let src = str.as_ptr();
            let len = str.len();

            // SAFETY:
            // `src` is obtained from a `&str` with length `len`, so is valid for reading `len` bytes.
            // `dst` is within bounds of `vec`'s allocation. So is `dst + len`.
            // `u8` has no alignment requirements, so `src` and `dst` are sufficiently aligned.
            // No overlapping, because we're copying from an existing `&str` to a newly allocated buffer.
            //
            // If `str` is empty, `len` is 0.
            // If `total_len == 0`, `dst` is a dangling pointer, *not* valid for read or write of a `u8`.
            // `copy_nonoverlapping` requires that `src` and `dst` must both
            // "be valid for reads of `count * size_of::<T>()` bytes".
            // However, safety docs for `std::ptr` (https://doc.rust-lang.org/std/ptr/index.html#safety)
            // state that:
            // "For memory accesses of size zero, *every* pointer is valid, including the null pointer".
            // So we do not need any special handling of zero-size `&str`s to satisfy safety constraints.
            unsafe { ptr::copy_nonoverlapping(src, dst, len) };

            // SAFETY: We allocated sufficient capacity in `vec` for all the strings concatenated.
            // So `dst.add(len)` cannot go out of bounds.
            //
            // If `total_len` is 0, `dst` may be an invalid dangling pointer and adding to it would be UB.
            // But if that is the case, `len` must be 0 too.
            // Docs for `*mut T::add` preface the safety requirements for being in bounds and provenance
            // with "If the computed offset is non-zero". So they don't apply in the case where `len == 0`.
            // So we do not need any special handling of zero-size `&str`s to satisfy safety constraints.
            dst = unsafe { dst.add(len) };
        }

        debug_assert_eq!(dst as usize - vec.as_ptr() as usize, total_len);

        // SAFETY: We have written `total_len` bytes to `vec`'s backing memory
        unsafe { vec.set_len(total_len) };

        // SAFETY: All the data added to `vec` has been `&str`s, so the contents of `vec` is guaranteed
        // to be a valid UTF-8 string
        unsafe { String::from_utf8_unchecked(vec) }
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
    /// * The region of memory starting at `ptr` and spanning `length` bytes must contain a valid
    ///   UTF-8 string.
    ///
    /// Violating these may cause problems like corrupting the allocator's internal data structures.
    ///
    /// The ownership of `ptr` is effectively transferred to the `String` which may then deallocate,
    /// reallocate or change the contents of memory pointed to by the pointer at will. Ensure that
    /// nothing else uses the pointer after calling this function.
    ///
    /// # Examples
    /// ```
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
    #[inline(always)]
    pub unsafe fn from_raw_parts_in(
        buf: *mut u8,
        length: usize,
        capacity: usize,
        allocator: &'alloc Allocator,
    ) -> String<'alloc> {
        // SAFETY: Safety conditions of this method are the same as `BumpaloString`'s method
        let inner =
            unsafe { BumpaloString::from_raw_parts_in(buf, length, capacity, allocator.bump()) };
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

    /// Set length of [`String`].
    ///
    /// # SAFETY
    ///
    /// * `new_len` must be less than or equal to [`capacity()`].
    /// * If `new_len` > `self.len()`, the bytes at `self.len()..new_len` must be initialized.
    /// * `new_len` must be on a UTF-8 character boundary
    ///   i.e. the first `new_len` bytes of the `String`'s buffer must comprise a valid UTF-8 string.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, String};
    /// let allocator = Allocator::new();
    ///
    /// let mut s = String::from_str_in("foobar", &allocator);
    /// unsafe { s.set_len(3) };
    /// assert_eq!(s, "foo");
    /// ```
    ///
    /// [`capacity()`]: String#capacity
    pub unsafe fn set_len(&mut self, new_len: usize) {
        // SAFETY: Safety requirements satisfy `bumpalo::collections::Vec`'s safety requirements.
        // Caller guarantees `new_len` is on a UTF-8 character boundary.
        unsafe { self.as_mut_vec().set_len(new_len) }
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

#[cfg(test)]
mod test {
    use crate::{Allocator, String};

    #[test]
    fn string_from_array_len_1() {
        let allocator = Allocator::default();
        let string = String::from_strs_array_in(["hello"], &allocator);
        assert_eq!(string, "hello");
    }

    #[test]
    fn string_from_array_len_2() {
        let allocator = Allocator::default();
        let string = String::from_strs_array_in(["hello", "world!"], &allocator);
        assert_eq!(string, "helloworld!");
    }

    #[test]
    fn string_from_array_len_3() {
        let hello = "hello";
        let world = std::string::String::from("world");
        let allocator = Allocator::default();
        let string = String::from_strs_array_in([hello, &world, "!"], &allocator);
        assert_eq!(string, "helloworld!");
    }

    #[test]
    fn string_from_empty_array() {
        let allocator = Allocator::default();
        let string = String::from_strs_array_in([], &allocator);
        assert_eq!(string, "");
    }

    #[test]
    fn string_from_array_of_empty_strs() {
        let allocator = Allocator::default();
        let string = String::from_strs_array_in(["", "", ""], &allocator);
        assert_eq!(string, "");
    }

    #[test]
    fn string_from_array_containing_some_empty_strs() {
        let allocator = Allocator::default();
        let string = String::from_strs_array_in(["", "hello", ""], &allocator);
        assert_eq!(string, "hello");
    }
}
