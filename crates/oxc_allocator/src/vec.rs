// We're wrapping an existing implementation. Having those wrapper functions
// must incur no overhead, so we declare them `#[inline(always)]`.
#![allow(clippy::inline_always)]

//! Arena Vec.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/ast/src/arena.rs)

use core::fmt;
use std::{
    self,
    borrow::Cow,
    fmt::Debug,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    mem::ManuallyDrop,
    ops::{self, Index, RangeBounds},
    slice::SliceIndex,
};

use allocator_api2::alloc::Global;

#[cfg(any(feature = "serialize", test))]
use serde::{ser::SerializeSeq, Serialize, Serializer};

use crate::{Allocator, Box};

type VecImpl<'a, T> = bump_scope::BumpVec<'a, 'a, T>;

/// A `Vec` without [`Drop`], which stores its data in the arena allocator.
///
/// Should only be used for storing AST types.
///
/// Must NOT be used to store types which have a [`Drop`] implementation.
/// `T::drop` will NOT be called on the `Vec`'s contents when the `Vec` is dropped.
/// If `T` owns memory outside of the arena, this will be a memory leak.
///
/// Note: This is not a soundness issue, as Rust does not support relying on `drop`
/// being called to guarantee soundness.
pub struct Vec<'alloc, T>(ManuallyDrop<VecImpl<'alloc, T>>);

impl<'alloc, T> Vec<'alloc, T> {
    /// Constructs a new, empty `Vec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let arena = Allocator::default();
    ///
    /// let mut vec: Vec<i32> = Vec::new_in(&arena);
    /// assert!(vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(VecImpl::new_in(allocator)))
    }

    /// Constructs a new, empty `Vec<T>` with at least the specified capacity
    /// with the provided allocator.
    ///
    /// The vector will be able to hold at least `capacity` elements without
    /// reallocating. This method is allowed to allocate for more elements than
    /// `capacity`. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// minimum *capacity* specified, the vector will have a zero *length*.
    ///
    /// For `Vec<T>` where `T` is a zero-sized type, there will be no allocation
    /// and the capacity will always be `usize::MAX`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let arena = Allocator::default();
    ///
    /// let mut vec = Vec::with_capacity_in(10, &arena);
    ///
    /// // The vector contains no items, even though it has capacity for more
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // These are all done without reallocating...
    /// for i in 0..10 {
    ///     vec.push(i);
    /// }
    /// assert_eq!(vec.len(), 10);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // ...but this may make the vector reallocate
    /// vec.push(11);
    /// assert_eq!(vec.len(), 11);
    /// assert!(vec.capacity() >= 11);
    ///
    /// // A vector of a zero-sized type will always over-allocate, since no
    /// // allocation is necessary
    /// let vec_units = Vec::<()>::with_capacity_in(10, &arena);
    /// assert_eq!(vec_units.capacity(), usize::MAX);
    /// ```
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(VecImpl::with_capacity_in(capacity, allocator)))
    }

    /// Create a new [`Vec`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorally identical to [`FromIterator::from_iter`].
    #[inline(always)]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(VecImpl::from_iter_in(iter, allocator)))
    }

    /// Returns the total number of elements the vector can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// let vec = Vec::<i32>::with_capacity_in(2048, &allocator);
    /// assert!(vec.capacity() >= 2048);
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// Equivalent to `&s[..]`.
    #[must_use]
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Extracts a mutable slice containing the entire vector.
    ///
    /// Equivalent to `&mut s[..]`.
    #[must_use]
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    /// Returns a raw pointer to the slice, or a dangling raw pointer
    /// valid for zero sized reads.
    #[must_use]
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    /// Returns an unsafe mutable pointer to slice, or a dangling
    /// raw pointer valid for zero sized reads.
    #[must_use]
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    /// Appends an element to the back of a collection.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    /// Inserts an element at position `index` within the vector, shifting all elements after it to the right.
    ///
    /// # Panics
    /// Panics if `index > len`.
    ///
    /// # Examples
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// vec.insert(1, 4);
    /// assert_eq!(vec, [1, 4, 2, 3]);
    /// vec.insert(4, 5);
    /// assert_eq!(vec, [1, 4, 2, 3, 5]);
    /// ```
    #[inline(always)]
    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, element);
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// Note: Because this shifts over the remaining elements, it has a
    /// worst-case performance of *O*(*n*). If you don't need the order of elements
    /// to be preserved, use [`swap_remove`] instead.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    ///
    /// [`swap_remove`]: Self::swap_remove
    ///
    /// # Examples
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
    /// assert_eq!(v.remove(1), 2);
    /// assert_eq!(v, [1, 3]);
    /// ```
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> T {
        self.0.remove(index)
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is *O*(1).
    /// If you need to preserve the element order, use [`remove`] instead.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    ///
    /// [`remove`]: Self::remove
    /// # Examples
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// let mut v = Vec::from_iter_in(["foo", "bar", "baz", "qux"], &allocator);
    ///
    /// assert_eq!(v.swap_remove(1), "bar");
    /// assert_eq!(v, ["foo", "qux", "baz"]);
    ///
    /// assert_eq!(v.swap_remove(0), "foo");
    /// assert_eq!(v, ["baz", "qux"]);
    /// ```
    #[inline(always)]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.0.swap_remove(index)
    }

    /// Shortens the vector, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater than the vector's current length, this has no
    /// effect.
    ///
    /// The [`drain`] method can emulate `truncate`, but causes the excess
    /// elements to be returned instead of dropped.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    ///
    /// # Examples
    ///
    /// Truncating a five element vector to two elements:
    ///
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// #
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4, 5], &allocator);
    /// vec.truncate(2);
    /// assert_eq!(vec, [1, 2]);
    /// ```
    ///
    /// No truncation occurs when `len` is greater than the vector's current
    /// length:
    ///
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// #
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// vec.truncate(8);
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    ///
    /// Truncating when `len == 0` is equivalent to calling the [`clear`]
    /// method.
    ///
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// #
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// vec.truncate(0);
    /// assert_eq!(vec, []);
    /// ```
    ///
    /// [`clear`]: Self::clear
    /// [`drain`]: Self::drain
    #[inline(always)]
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }

    /// Clears the vector, removing all values.
    ///
    /// # Examples
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4, 5], &allocator);
    /// vec.clear();
    /// assert!(vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Converts the vector into [`Box<[T]>`][owned slice].
    ///
    /// Any excess capacity the vector has will not be included in the slice.
    /// The excess memory will be leaked in the arena (i.e. not reused by another allocation).
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    /// let mut v = Vec::with_capacity_in(10, &allocator);
    /// v.extend([1, 2, 3]);
    /// let b = v.into_boxed_slice();
    ///
    /// assert_eq!(&*b, &[1, 2, 3]);
    /// assert_eq!(b.len(), 3);
    /// ```
    ///
    /// [owned slice]: Box
    #[inline(always)]
    pub fn into_boxed_slice(self) -> Box<'alloc, [T]> {
        // By first calling `into_fixed_vec` we don't shrink the allocation.
        let ptr = ManuallyDrop::into_inner(self.0).into_fixed_vec().into_boxed_slice().into_raw();
        // SAFETY: `ptr` points to a valid slice `[T]`.
        // Lifetime of returned `Box<'alloc, [T]>` is same as lifetime of consumed `Vec<'alloc, T>`,
        // so data in the `Box` must be valid for its lifetime.
        // `Vec` uniquely owned the data, and we have consumed the `Vec`, so the new `Box` has
        // unique ownership of the data (no aliasing).
        // `ptr` was created from a `&mut [T]`.
        unsafe { Box::from_non_null(ptr) }
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// let mut vec2 = Vec::from_iter_in([4, 5, 6], &allocator);
    /// vec.append(&mut vec2);
    /// assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(vec2, []);
    /// ```
    #[inline(always)]
    pub fn append(&mut self, other: &mut Self) {
        self.reserve(other.len());

        // SAFETY:
        // We have allocated enough space for the additional elements from `other`.
        unsafe {
            other.as_ptr().copy_to_nonoverlapping(self.as_mut_ptr().add(self.len()), other.len());

            let self_len = self.len();
            self.set_len(self_len + other.len());

            other.set_len(0);
        }
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// let mut vec2 = Vec::from_iter_in([4, 5, 6], &allocator);
    /// vec.prepend(&mut vec2);
    /// assert_eq!(vec, [4, 5, 6, 1, 2, 3]);
    /// assert_eq!(vec2, []);
    /// ```
    #[inline(always)]
    pub fn prepend(&mut self, other: &mut Self) {
        self.reserve(other.len());

        // SAFETY:
        // We have allocated enough space for the additional elements from `other`.
        unsafe {
            // copy existing content forward to make space
            self.as_mut_ptr().copy_to(self.as_mut_ptr().add(other.len()), self.len());

            // copy other
            other.as_ptr().copy_to_nonoverlapping(self.as_mut_ptr(), other.len());

            let self_len = self.len();
            self.set_len(self_len + other.len());

            other.set_len(0);
        }
    }

    /// Removes the specified range from the vector in bulk, returning all
    /// removed elements as an iterator. If the iterator is dropped before
    /// being fully consumed, it drops the remaining removed elements.
    ///
    /// The returned iterator keeps a mutable borrow on the vector to optimize
    /// its implementation.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`](core::mem::forget), for example), the vector may have lost and leaked
    /// elements arbitrarily, including elements outside the range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxc_allocator::{ Allocator, Vec };
    /// # let allocator = Allocator::default();
    /// let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
    /// let u = Vec::from_iter_in(v.drain(1..), &allocator);
    /// assert_eq!(v, [1]);
    /// assert_eq!(u, [2, 3]);
    ///
    /// // A full range clears the vector, like `clear()` does
    /// v.drain(..);
    /// assert_eq!(v, []);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        Drain(self.0.drain(range))
    }

    /// Creates a splicing iterator that replaces the specified range in the vector
    /// with the given `replace_with` iterator and yields the removed items.
    /// `replace_with` does not need to be the same length as `range`.
    ///
    /// `range` is removed even if the iterator is not consumed until the end.
    ///
    /// It is unspecified how many elements are removed from the vector
    /// if the `Splice` value is leaked.
    ///
    /// The input iterator `replace_with` is only consumed when the `Splice` value is dropped.
    ///
    /// This is optimal if:
    ///
    /// * The tail (elements in the vector after `range`) is empty,
    /// * or `replace_with` yields fewer or equal elements than `range`’s length
    /// * or the lower bound of its `size_hint()` is exact.
    ///
    /// Otherwise, a temporary vector is allocated and the tail is moved twice.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator: Allocator = Allocator::default();
    /// let mut v = Vec::from_iter_in([1, 2, 3, 4], &allocator);
    /// let new = [7, 8, 9];
    /// let u = Vec::from_iter_in(v.splice(1..3, new), &allocator);
    /// assert_eq!(v, &[1, 7, 8, 9, 4]);
    /// assert_eq!(u, &[2, 3]);
    /// ```
    #[inline(always)]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        Splice(self.0.splice(range, replace_with))
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4], &allocator);
    ///
    /// vec.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    ///
    /// assert_eq!(vec, [2, 3, 4]);
    /// ```
    #[inline(always)]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|elem| f(elem));
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4], &allocator);
    ///
    /// vec.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    ///
    /// assert_eq!(vec, [2, 3, 4]);
    /// ```
    #[inline(always)]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.0.retain(f);
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `Vec<T>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1], &allocator);
    /// vec.reserve(10);
    /// assert!(vec.capacity() >= 11);
    /// ```
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Reserves the minimum capacity for at least `additional` more elements to
    /// be inserted in the given `Vec<T>`. Unlike [`reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`reserve`] if future insertions are expected.
    ///
    /// [`reserve`]: Vec::reserve
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{ Allocator, Vec };
    /// let allocator = Allocator::default();
    /// let mut vec = Vec::from_iter_in([1], &allocator);
    /// vec.reserve_exact(10);
    /// assert!(vec.capacity() >= 11);
    /// ```
    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// This is a low-level operation that maintains none of the normal
    /// invariants of the type. Normally changing the length of a vector
    /// is done using one of the safe operations instead, such as
    /// [`resize`], [`truncate`], [`extend`], or [`clear`].
    ///
    /// # Safety
    /// - `new_len` must be less than or equal to the [`capacity`].
    /// - The elements at `old_len..new_len` must be initialized.
    ///
    /// [`resize`]: Self::resize
    /// [`truncate`]: Self::truncate
    /// [`extend`]: Self::extend
    /// [`clear`]: Self::clear
    /// [`capacity`]: Self::capacity
    #[inline(always)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.0.set_len(new_len);
    }

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with the result of
    /// calling the closure `f`. The return values from `f` will end up
    /// in the `Vec` in the order they have been generated.
    ///
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// This method uses a closure to create new values on every push. If
    /// you'd rather [`Clone`] a given value, use [`Vec::resize`]. If you
    /// want to use the [`Default`] trait to generate values, you can
    /// pass [`Default::default`] as the second argument.
    #[inline(always)]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.0.resize_with(new_len, f);
    }
}

impl<T> Vec<'_, T>
where
    T: Clone,
{
    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `value`.
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// This method requires `T` to implement [`Clone`],
    /// in order to be able to clone the passed value.
    /// If you need more flexibility (or want to rely on [`Default`] instead of
    /// [`Clone`]), use [`resize_with`].
    /// If you only need to resize to a smaller size, use [`truncate`].
    ///
    /// [`resize_with`]: Self::resize_with
    /// [`truncate`]: Self::truncate
    #[inline(always)]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.0.resize(new_len, value);
    }
}

impl<'alloc, T: Debug> Debug for Vec<'alloc, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = &*self.0;
        f.debug_tuple("Vec").field(inner).finish()
    }
}

#[cfg(any(feature = "serialize", test))]
impl<'alloc, T> Serialize for Vec<'alloc, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

macro_rules! impl_slice_eq1 {
    ([$($($vars:tt)+)?] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        #[allow(clippy::partialeq_ne_impl)]
        impl<$($($vars)+,)? T, U> PartialEq<$rhs> for $lhs
        where
            T: PartialEq<U>,
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    }
}

impl_slice_eq1! { ['t, 'u] Vec<'t, T>, Vec<'u, U> }
impl_slice_eq1! { ['t] Vec<'t, T>, &[U] }
impl_slice_eq1! { [] Vec<'_, T>, &mut [U] }
impl_slice_eq1! { [] &[T], Vec<'_, U> }
impl_slice_eq1! { [] &mut [T], Vec<'_, U> }
impl_slice_eq1! { [] Vec<'_, T>, [U] }
impl_slice_eq1! { [] [T], Vec<'_, U>  }
impl_slice_eq1! { ['t] Cow<'_, [T]>, Vec<'t, U> where T: Clone }
impl_slice_eq1! { ['t, const N: usize] Vec<'t, T>, [U; N] }
impl_slice_eq1! { ['t, const N: usize] Vec<'t, T>, &[U; N] }

/// A draining iterator for `Vec<T>`.
///
/// This `struct` is created by [`Vec::drain`].
/// See its documentation for more.
///
/// # Example
///
/// ```
/// # use oxc_allocator::{ Allocator, Vec };
/// # let allocator = Allocator::default();
/// let mut v = Vec::from_iter_in([0, 1, 2], &allocator);
/// let iter: oxc_allocator::vec::Drain<'_, _> = v.drain(..);
/// ```
pub struct Drain<'a, T>(bump_scope::owned_slice::Drain<'a, T>);

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.0.next_back()
    }
}

impl<T> ExactSizeIterator for Drain<'_, T> {}

impl<T> FusedIterator for Drain<'_, T> {}

/// A splicing iterator for `Vec`.
///
/// This struct is created by [`Vec::splice()`].
/// See its documentation for more.
///
/// # Example
///
/// ```
/// # use oxc_allocator::{ Allocator, Vec };
/// # let allocator = Allocator::default();
/// let mut v = Vec::from_iter_in([0, 1, 2], &allocator);
/// let new = [7, 8];
/// let iter: oxc_allocator::vec::Splice<'_, _> = v.splice(1.., new);
/// ```
pub struct Splice<'a, I: Iterator>(bump_scope::bump_vec::Splice<'a, I, Global>);

impl<I: Iterator> Iterator for Splice<'_, I> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'alloc, T> Eq for Vec<'alloc, T> where T: Eq {}

impl<'alloc, T> ops::Deref for Vec<'alloc, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T> ops::DerefMut for Vec<'alloc, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

/// An iterator that moves out of a vector.
///
/// This `struct` is created by the `into_iter` method on [`Vec`]
/// (provided by the [`IntoIterator`] trait).
///
/// # Example
///
/// ```
/// # use oxc_allocator::{ Allocator, Vec };
/// let v = Vec::from_iter_in([0, 1, 2], &allocator);
/// let iter: std::vec::IntoIter<_> = v.into_iter();
/// ```
pub struct IntoIter<'alloc, T>(bump_scope::bump_vec::IntoIter<'alloc, 'alloc, T>);

impl<T> IntoIter<'_, T> {
    /// Returns the remaining items of this iterator as a slice.
    #[must_use]
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Returns the remaining items of this iterator as a mutable slice.
    #[must_use]
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }
}

impl<T> Iterator for IntoIter<'_, T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.0.count()
    }
}

impl<T> DoubleEndedIterator for IntoIter<'_, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<'_, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> FusedIterator for IntoIter<'_, T> {}

impl<'alloc, T> IntoIterator for Vec<'alloc, T> {
    type IntoIter = IntoIter<'alloc, T>;
    type Item = T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        // TODO: `allocator_api2::vec::Vec::IntoIter` is `Drop`.
        // Wrap it in `ManuallyDrop` to prevent that.
        IntoIter(inner.into_iter())
    }
}

impl<'alloc, T> IntoIterator for &'alloc Vec<'alloc, T> {
    type IntoIter = std::slice::Iter<'alloc, T>;
    type Item = &'alloc T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for Vec<'_, T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

// Unused right now.
// impl<'alloc, T> ops::IndexMut<usize> for Vec<'alloc, T> {
// fn index_mut(&mut self, index: usize) -> &mut Self::Output {
// self.0.index_mut(index)
// }
// }

impl<'alloc, T: Hash> Hash for Vec<'alloc, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for e in self.0.iter() {
            e.hash(state);
        }
    }
}

impl<T> Extend<T> for Vec<'_, T> {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

#[cfg(test)]
mod test {
    use super::Vec;
    use crate::{Allocator, Box};

    #[test]
    fn vec_with_capacity() {
        let allocator = Allocator::default();
        let v: Vec<i32> = Vec::with_capacity_in(10, &allocator);
        assert!(v.is_empty());
    }

    #[test]
    fn vec_debug() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = format!("{v:?}");
        assert_eq!(v, "Vec([\"x\"])");
    }

    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = serde_json::to_string(&v).unwrap();
        assert_eq!(v, "[\"x\"]");
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }

    #[test]
    fn vec_to_boxed_slice() {
        let allocator = Allocator::default();
        let mut v = Vec::with_capacity_in(10, &allocator);
        v.extend([1, 2, 3]);

        let b = v.into_boxed_slice();
        // Check return value is an `oxc_allocator::Box`, not an `allocator_api2::boxed::Box`
        let b: Box<[u8]> = b;

        assert_eq!(&*b, &[1, 2, 3]);
        // Check length of slice is equal to what `v.len()` was, not `v.capacity()`
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn append() {
        let allocator = Allocator::default();

        {
            let mut v = Vec::new_in(&allocator);
            let mut other = Vec::<i32>::new_in(&allocator);
            v.append(&mut other);
            assert!(v.is_empty());
            assert!(other.is_empty());
        }

        {
            let mut v = Vec::new_in(&allocator);
            let mut other = Vec::from_iter_in([1, 2, 3], &allocator);
            v.append(&mut other);
            assert_eq!(v.as_slice(), &[1, 2, 3]);
            assert!(other.is_empty());
        }

        {
            let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
            let mut other = Vec::new_in(&allocator);
            v.append(&mut other);
            assert_eq!(v.as_slice(), &[1, 2, 3]);
            assert!(other.is_empty());
        }

        {
            let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
            let mut other = Vec::from_iter_in([4, 5, 6], &allocator);
            v.append(&mut other);
            assert_eq!(v.as_slice(), &[1, 2, 3, 4, 5, 6]);
            assert!(other.is_empty());
        }
    }

    #[test]
    fn prepend() {
        let allocator = Allocator::default();

        {
            let mut v = Vec::new_in(&allocator);
            let mut other = Vec::<i32>::new_in(&allocator);
            v.prepend(&mut other);
            assert!(v.is_empty());
            assert!(other.is_empty());
        }

        {
            let mut v = Vec::new_in(&allocator);
            let mut other = Vec::from_iter_in([1, 2, 3], &allocator);
            v.prepend(&mut other);
            assert_eq!(v.as_slice(), &[1, 2, 3]);
            assert!(other.is_empty());
        }

        {
            let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
            let mut other = Vec::new_in(&allocator);
            v.prepend(&mut other);
            assert_eq!(v.as_slice(), &[1, 2, 3]);
            assert!(other.is_empty());
        }

        {
            let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
            let mut other = Vec::from_iter_in([4, 5, 6], &allocator);
            v.prepend(&mut other);
            assert_eq!(v.as_slice(), &[4, 5, 6, 1, 2, 3]);
            assert!(other.is_empty());
        }
    }
}
