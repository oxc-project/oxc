//! Methods for creating [`Vec`]s, [`Box`]es, [`Ident`]s, and [`Str`]s.
//!
//! TODO: These should not be methods on `AstBuilder` - call sites should use `Vec::with_capacity_in` etc instead.
//! But that requires `Vec::with_capacity_in` to take an `&A: &GetAllocator` instead of `&Allocator`.
//! These methods are a temporary measure to avoid having to make that change now.

use std::borrow::Cow;

use oxc_allocator::{Box, FromIn, Vec};
use oxc_str::{Ident, Str};

use super::AstBuilder;

impl<'a> AstBuilder<'a> {
    /// Move a value into the memory arena.
    #[inline]
    pub fn alloc<T>(&self, value: T) -> Box<'a, T> {
        Box::new_in(value, self.allocator)
    }

    /// Create a new empty [`Vec`] that stores its elements in the memory arena.
    #[inline]
    pub fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    /// Create a new empty [`Vec`] that stores its elements in the memory arena.
    /// Enough memory will be pre-allocated to store at least `capacity`
    /// elements.
    #[inline]
    pub fn vec_with_capacity<T>(&self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    /// Create a new arena-allocated [`Vec`] initialized with a single element.
    #[inline]
    pub fn vec1<T>(&self, value: T) -> Vec<'a, T> {
        Vec::from_value_in(value, self.allocator)
    }

    /// Collect an iterator into a new arena-allocated [`Vec`].
    #[inline]
    pub fn vec_from_iter<T, I: IntoIterator<Item = T>>(&self, iter: I) -> Vec<'a, T> {
        Vec::from_iter_in(iter, self.allocator)
    }

    /// Create [`Vec`] from a fixed-size array.
    ///
    /// This is preferable to `vec_from_iter` where source is an array, as size is statically known,
    /// and compiler is more likely to construct the values directly in arena, rather than constructing
    /// on stack and then copying to arena.
    #[inline]
    pub fn vec_from_array<T, const N: usize>(&self, array: [T; N]) -> Vec<'a, T> {
        Vec::from_array_in(array, self.allocator)
    }

    /// Allocate an [`Ident`] from a string slice.
    #[inline]
    pub fn ident(&self, value: &str) -> Ident<'a> {
        Ident::from_in(value, self.allocator)
    }

    /// Allocate an [`Ident`] from an array of string slices.
    #[inline]
    pub fn ident_from_strs_array<const N: usize>(&self, strings: [&str; N]) -> Ident<'a> {
        Ident::from_strs_array_in(strings, self.allocator)
    }

    /// Convert a [`Cow<'a, str>`] to an [`Ident<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Ident` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Ident`.
    #[inline]
    pub fn ident_from_cow(&self, value: &Cow<'a, str>) -> Ident<'a> {
        Ident::from_cow_in(value, self.allocator)
    }

    /// Allocate a [`Str`] from a string slice.
    #[inline]
    pub fn str(&self, value: &str) -> Str<'a> {
        Str::from_in(value, self.allocator)
    }

    /// Allocate a [`Str`] from an array of string slices.
    #[inline]
    pub fn str_from_strs_array<const N: usize>(&self, strings: [&str; N]) -> Str<'a> {
        Str::from_strs_array_in(strings, self.allocator)
    }

    /// Convert a [`Cow<'a, str>`] to a [`Str<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns a `Str` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Str`.
    #[inline]
    pub fn str_from_cow(&self, value: &Cow<'a, str>) -> Str<'a> {
        Str::from_cow_in(value, self.allocator)
    }
}
