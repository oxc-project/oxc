//! Arena Vec.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/ast/src/arena.rs)

use std::{
    self,
    borrow::Cow,
    fmt::Debug,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    ops::{self, Index, RangeBounds},
    slice::SliceIndex,
};

use allocator_api2::alloc::Global;

use crate::{Allocator, Box};

type VecImpl<'a, T> = bump_scope::BumpVec<'a, 'a, T>;

/// A bump-allocated vector.
#[cfg_attr(any(feature = "serialize", test), derive(serde::Serialize))]
#[derive(Debug)]
pub struct Vec<'alloc, T>(VecImpl<'alloc, T>);

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
        Self(VecImpl::new_in(allocator))
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
        Self(VecImpl::with_capacity_in(capacity, allocator))
    }

    /// Create a new [`Vec`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorally identical to [`FromIterator::from_iter`].
    #[inline(always)]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self {
        Self(VecImpl::from_iter_in(iter, allocator))
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
    pub const fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// Equivalent to `&s[..]`.
    #[must_use]
    #[inline(always)]
    pub const fn as_slice(&self) -> &[T] {
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

    /// Appends an element to the back of a collection.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.0.push(value)
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
        self.0.insert(index, element)
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
    /// [`clear`]: BumpVec::clear
    /// [`drain`]: BumpVec::drain
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
        let ptr = self.0.into_fixed_vec().into_boxed_slice().into_raw();
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
        self.retain_mut(|elem| f(elem))
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
        self.0.retain(f)
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
    /// [`resize`]: BumpVec::resize
    /// [`truncate`]: BumpVec::truncate
    /// [`extend`]: BumpVec::extend
    /// [`clear`]: BumpVec::clear
    /// [`capacity`]: BumpVec::capacity
    #[inline(always)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.0.set_len(new_len);
    }
}

macro_rules! impl_slice_eq1 {
    ([$($($vars:tt)+)?] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
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

impl<'alloc, T> IntoIterator for Vec<'alloc, T> {
    type IntoIter = <VecImpl<'alloc, T> as IntoIterator>::IntoIter;
    type Item = T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Extend<T> for Vec<'_, T> {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
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
