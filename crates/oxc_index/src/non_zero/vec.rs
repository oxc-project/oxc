use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::iter::{self, FromIterator};
use core::marker::PhantomData;
use core::slice;

#[cfg(feature = "serialize")]
use super::slice::NonZeroIndexBox;

use crate::{__internal_impl_partialeq, __internal_impl_partialeq2};

use super::{
    indexing::{NonZeroIdxRangeBounds, NonZeroIdxSliceIndex},
    slice::NonZeroIndexSlice,
    NonZeroIdx,
};

/// A Vec that only accepts indices of a specific non-zero type.
///
/// TODO
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonZeroIndexVec<I: NonZeroIdx, T> {
    /// Our wrapped Vec.
    pub raw: Vec<T>,
    _marker: PhantomData<fn(&I)>,
}

#[allow(unsafe_code)]
// SAFETY: Whether `NonZeroIndexVec` is `Send` depends only on the data,
// not the phantom data.
unsafe impl<I: NonZeroIdx, T> Send for NonZeroIndexVec<I, T> where T: Send {}

impl<I: NonZeroIdx, T: fmt::Debug> fmt::Debug for NonZeroIndexVec<I, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.raw, fmt)
    }
}

pub(crate) type NonZeroEnumerated<Iter, I, T> =
    iter::Map<iter::Enumerate<Iter>, fn((usize, T)) -> (I, T)>;

impl<I: NonZeroIdx, T> NonZeroIndexVec<I, T> {
    /// Construct a new NonZeroIndexVec.
    #[inline]
    pub fn new() -> Self {
        NonZeroIndexVec { raw: Vec::new(), _marker: PhantomData }
    }

    /// Construct a `NonZeroIndexVec` from a `Vec<T>`.
    ///
    /// Panics if it's length is too large for our index type.
    #[inline]
    pub fn from_vec(vec: Vec<T>) -> Self {
        // See if `I::from_usize` might be upset by this length.
        let _ = I::from_usize(vec.len());
        NonZeroIndexVec { raw: vec, _marker: PhantomData }
    }

    /// Construct an NonZeroIndexVec that can hold at least `capacity` items before
    /// reallocating. See [`Vec::with_capacity`].
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        NonZeroIndexVec { raw: Vec::with_capacity(capacity), _marker: PhantomData }
    }

    /// Similar to `self.into_iter().enumerate()` but with indices of `I` and
    /// not `usize`.
    #[inline(always)]
    pub fn into_iter_enumerated(self) -> NonZeroEnumerated<vec::IntoIter<T>, I, T> {
        self.raw.into_iter().enumerate().map(|(i, t)| (I::from_usize(i), t))
    }

    /// Creates a splicing iterator that replaces the specified range in the
    /// vector with the given `replace_with` iterator and yields the removed
    /// items. See [`Vec::splice`]
    #[inline]
    pub fn splice<R, It>(
        &mut self,
        range: R,
        replace_with: It,
    ) -> vec::Splice<<It as IntoIterator>::IntoIter>
    where
        It: IntoIterator<Item = T>,
        R: NonZeroIdxRangeBounds<I>,
    {
        self.raw.splice(range.into_range(), replace_with)
    }

    /// Similar to `self.drain(r).enumerate()` but with indices of `I` and not
    /// `usize`.
    #[inline]
    pub fn drain_enumerated<R: NonZeroIdxRangeBounds<I>>(
        &mut self,
        range: R,
    ) -> NonZeroEnumerated<vec::Drain<'_, T>, I, T> {
        self.raw.drain(range.into_range()).enumerate().map(|(i, t)| (I::from_usize(i), t))
    }

    /// Gives the next index that will be assigned when `push` is
    /// called.
    #[inline]
    pub fn next_idx(&self) -> I {
        I::from_usize(self.len())
    }

    /// Get a the storage as a `&[T]`
    #[inline(always)]
    pub fn as_raw_slice(&self) -> &[T] {
        &self.raw
    }

    /// Get a the storage as a `&mut [T]`
    #[inline(always)]
    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        &mut self.raw
    }

    /// Equivalent to accessing our `raw` field, but as a function.
    #[inline(always)]
    pub fn as_vec(&self) -> &Vec<T> {
        &self.raw
    }

    /// Equivalent to accessing our `raw` field mutably, but as a function, if
    /// that's what you'd prefer.
    #[inline(always)]
    pub fn as_mut_vec(&mut self) -> &mut Vec<T> {
        &mut self.raw
    }

    /// Push a new item onto the vector, and return it's index.
    #[inline]
    pub fn push(&mut self, d: T) -> I {
        let idx = I::from_usize(self.len());
        self.raw.push(d);
        idx
    }

    /// Pops the last item off, returning it. See [`Vec::pop`].
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    /// Converts the vector into an owned IdxSlice, dropping excess capacity.
    #[inline]
    pub fn into_boxed_slice(self) -> alloc::boxed::Box<NonZeroIndexSlice<I, [T]>> {
        let b = self.raw.into_boxed_slice();
        // SAFETY: `NonZeroIndexSlice` is a thin wrapper around `[T]` with the added marker for the index.
        #[allow(unsafe_code)]
        unsafe {
            Box::from_raw(Box::into_raw(b) as *mut NonZeroIndexSlice<I, [T]>)
        }
    }

    /// Return an iterator that removes the items from the requested range. See
    /// [`Vec::drain`].
    ///
    /// See also [`NonZeroIndexVec::drain_enumerated`], which gives you indices (of the
    /// correct type) as you iterate.
    #[inline]
    pub fn drain<R: NonZeroIdxRangeBounds<I>>(&mut self, range: R) -> vec::Drain<'_, T> {
        self.raw.drain(range.into_range())
    }

    /// Shrinks the capacity of the vector as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit();
    }

    /// Shortens the vector, keeping the first `len` elements and dropping
    /// the rest. See [`Vec::truncate`]
    #[inline]
    pub fn truncate(&mut self, a: usize) {
        self.raw.truncate(a);
    }

    /// Clear our vector. See [`Vec::clear`].
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
    }

    /// Reserve capacity for `c` more elements. See [`Vec::reserve`]
    #[inline]
    pub fn reserve(&mut self, c: usize) {
        self.raw.reserve(c);
    }

    /// Get a ref to the item at the provided index, or None for out of bounds.
    #[inline]
    pub fn get<J: NonZeroIdxSliceIndex<I, T>>(&self, index: J) -> Option<&J::Output> {
        index.get(self.as_slice())
    }

    /// Get a mut ref to the item at the provided index, or None for out of
    /// bounds
    #[inline]
    pub fn get_mut<J: NonZeroIdxSliceIndex<I, T>>(&mut self, index: J) -> Option<&mut J::Output> {
        index.get_mut(self.as_mut_slice())
    }

    /// Resize ourselves in-place to `new_len`. See [`Vec::resize`].
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.raw.resize(new_len, value);
    }

    /// Resize ourselves in-place to `new_len`. See [`Vec::resize_with`].
    #[inline]
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, f: F) {
        self.raw.resize_with(new_len, f);
    }

    /// Moves all the elements of `other` into `Self`, leaving `other` empty.
    /// See [`Vec::append`].
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        self.raw.append(&mut other.raw);
    }

    /// Splits the collection into two at the given index. See
    /// [`Vec::split_off`].
    #[inline]
    #[must_use]
    pub fn split_off(&mut self, idx: I) -> Self {
        Self::from_vec(self.raw.split_off(idx.index()))
    }

    /// Remove the item at `index`. See [`Vec::remove`].
    #[inline]
    pub fn remove(&mut self, index: I) -> T {
        self.raw.remove(index.index())
    }

    /// Remove the item at `index` without maintaining order. See
    /// [`Vec::swap_remove`].
    #[inline]
    pub fn swap_remove(&mut self, index: I) -> T {
        self.raw.swap_remove(index.index())
    }

    /// Insert an item at `index`. See [`Vec::insert`].
    #[inline]
    pub fn insert(&mut self, index: I, element: T) {
        self.raw.insert(index.index(), element);
    }

    /// Append all items in the slice to the end of our vector.
    ///
    /// See [`Vec::extend_from_slice`].
    #[inline]
    pub fn extend_from_slice(&mut self, other: &NonZeroIndexSlice<I, [T]>)
    where
        T: Clone,
    {
        self.raw.extend_from_slice(&other.raw);
    }

    /// Forwards to the `Vec::retain` implementation.
    #[inline]
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.raw.retain(f);
    }

    /// Forwards to the `Vec::dedup_by_key` implementation.
    #[inline]
    pub fn dedup_by_key<F: FnMut(&mut T) -> K, K: PartialEq>(&mut self, key: F) {
        self.raw.dedup_by_key(key);
    }

    /// Forwards to the `Vec::dedup` implementation.
    #[inline]
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.raw.dedup();
    }

    /// Forwards to the `Vec::dedup_by` implementation.
    #[inline]
    pub fn dedup_by<F: FnMut(&mut T, &mut T) -> bool>(&mut self, same_bucket: F) {
        self.raw.dedup_by(same_bucket);
    }

    /// Get a NonZeroIndexSlice over this vector. See `as_raw_slice` for converting to
    /// a `&[T]` (or access `self.raw`).
    #[inline(always)]
    pub fn as_slice(&self) -> &NonZeroIndexSlice<I, [T]> {
        NonZeroIndexSlice::new(&self.raw)
    }

    /// Get a mutable NonZeroIndexSlice over this vector. See `as_raw_slice_mut` for
    /// converting to a `&mut [T]` (or access `self.raw`).
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut NonZeroIndexSlice<I, [T]> {
        NonZeroIndexSlice::new_mut(&mut self.raw)
    }
}

impl<I: NonZeroIdx, T> Default for NonZeroIndexVec<I, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I: NonZeroIdx, T> Extend<T> for NonZeroIndexVec<I, T> {
    #[inline]
    fn extend<J: IntoIterator<Item = T>>(&mut self, iter: J) {
        self.raw.extend(iter);
    }
}

impl<'a, I: NonZeroIdx, T: 'a + Copy> Extend<&'a T> for NonZeroIndexVec<I, T> {
    #[inline]
    fn extend<J: IntoIterator<Item = &'a T>>(&mut self, iter: J) {
        self.raw.extend(iter);
    }
}

impl<I: NonZeroIdx, T> FromIterator<T> for NonZeroIndexVec<I, T> {
    #[inline]
    fn from_iter<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = T>,
    {
        NonZeroIndexVec { raw: FromIterator::from_iter(iter), _marker: PhantomData }
    }
}

impl<I: NonZeroIdx, T> IntoIterator for NonZeroIndexVec<I, T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> vec::IntoIter<T> {
        self.raw.into_iter()
    }
}

impl<'a, I: NonZeroIdx, T> IntoIterator for &'a NonZeroIndexVec<I, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> slice::Iter<'a, T> {
        self.raw.iter()
    }
}

impl<'a, I: NonZeroIdx, T> IntoIterator for &'a mut NonZeroIndexVec<I, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.raw.iter_mut()
    }
}

impl<I: NonZeroIdx, T> From<NonZeroIndexVec<I, T>> for Box<NonZeroIndexSlice<I, [T]>> {
    #[inline]
    fn from(src: NonZeroIndexVec<I, T>) -> Self {
        src.into_boxed_slice()
    }
}

impl<I: NonZeroIdx, T> From<Box<NonZeroIndexSlice<I, [T]>>> for NonZeroIndexVec<I, T> {
    #[inline]
    fn from(src: Box<NonZeroIndexSlice<I, [T]>>) -> Self {
        src.into_vec()
    }
}

impl<'a, I: NonZeroIdx, T> From<Cow<'a, NonZeroIndexSlice<I, [T]>>> for NonZeroIndexVec<I, T>
where
    NonZeroIndexSlice<I, [T]>: ToOwned<Owned = NonZeroIndexVec<I, T>>,
{
    #[inline]
    fn from(s: Cow<'a, NonZeroIndexSlice<I, [T]>>) -> NonZeroIndexVec<I, T> {
        s.into_owned()
    }
}

impl<'a, I: NonZeroIdx, T: Clone> From<&'a NonZeroIndexSlice<I, [T]>> for NonZeroIndexVec<I, T> {
    #[inline]
    fn from(src: &'a NonZeroIndexSlice<I, [T]>) -> Self {
        src.to_owned()
    }
}
impl<'a, I: NonZeroIdx, T: Clone> From<&'a mut NonZeroIndexSlice<I, [T]>>
    for NonZeroIndexVec<I, T>
{
    #[inline]
    fn from(src: &'a mut NonZeroIndexSlice<I, [T]>) -> Self {
        src.to_owned()
    }
}

impl<I: NonZeroIdx, T> From<Vec<T>> for NonZeroIndexVec<I, T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self { raw: v, _marker: PhantomData }
    }
}

impl<I: NonZeroIdx, T: Clone> Clone for NonZeroIndexVec<I, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { raw: self.raw.clone(), _marker: PhantomData }
    }
    #[inline]
    fn clone_from(&mut self, o: &Self) {
        self.raw.clone_from(&o.raw);
    }
}

impl<I: NonZeroIdx, A> AsRef<[A]> for NonZeroIndexVec<I, A> {
    #[inline]
    fn as_ref(&self) -> &[A] {
        &self.raw
    }
}

impl<I: NonZeroIdx, A> AsMut<[A]> for NonZeroIndexVec<I, A> {
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        &mut self.raw
    }
}

impl<I: NonZeroIdx, A> AsRef<NonZeroIndexSlice<I, [A]>> for NonZeroIndexVec<I, A> {
    #[inline]
    fn as_ref(&self) -> &NonZeroIndexSlice<I, [A]> {
        NonZeroIndexSlice::new(&self.raw)
    }
}

impl<I: NonZeroIdx, A> AsMut<NonZeroIndexSlice<I, [A]>> for NonZeroIndexVec<I, A> {
    #[inline]
    fn as_mut(&mut self) -> &mut NonZeroIndexSlice<I, [A]> {
        NonZeroIndexSlice::new_mut(&mut self.raw)
    }
}

impl<I: NonZeroIdx, A> core::ops::Deref for NonZeroIndexVec<I, A> {
    type Target = NonZeroIndexSlice<I, [A]>;
    #[inline]
    fn deref(&self) -> &NonZeroIndexSlice<I, [A]> {
        NonZeroIndexSlice::new(&self.raw)
    }
}

impl<I: NonZeroIdx, A> core::ops::DerefMut for NonZeroIndexVec<I, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut NonZeroIndexSlice<I, [A]> {
        NonZeroIndexSlice::new_mut(&mut self.raw)
    }
}

impl<I: NonZeroIdx, T> Borrow<NonZeroIndexSlice<I, [T]>> for NonZeroIndexVec<I, T> {
    #[inline]
    fn borrow(&self) -> &NonZeroIndexSlice<I, [T]> {
        self.as_slice()
    }
}

impl<I: NonZeroIdx, T> BorrowMut<NonZeroIndexSlice<I, [T]>> for NonZeroIndexVec<I, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut NonZeroIndexSlice<I, [T]> {
        self.as_mut_slice()
    }
}

__internal_impl_partialeq! { NonZeroIndexVec<I, A>, Vec<B> }
__internal_impl_partialeq! { NonZeroIndexVec<I, A>, &'b [B] }
__internal_impl_partialeq! { NonZeroIndexVec<I, A>, &'b mut [B] }

__internal_impl_partialeq2! { NonZeroIndexVec<I, A>, &'b NonZeroIndexSlice<J, [B]> }
__internal_impl_partialeq2! { NonZeroIndexVec<I, A>, &'b mut NonZeroIndexSlice<J, [B]> }

__internal_impl_partialeq! { &'a NonZeroIndexSlice<I, [A]>, Vec<B> }
__internal_impl_partialeq! { &'a mut NonZeroIndexSlice<I, [A]>, Vec<B> }

__internal_impl_partialeq! { NonZeroIndexSlice<I, [A]>, &'b [B] }
__internal_impl_partialeq! { NonZeroIndexSlice<I, [A]>, &'b mut [B] }

__internal_impl_partialeq2! { &'a NonZeroIndexSlice<I, [A]>, NonZeroIndexVec<J, B> }
__internal_impl_partialeq2! { &'a mut NonZeroIndexSlice<I, [A]>, NonZeroIndexVec<J, B> }

__internal_impl_partialeq2! { NonZeroIndexSlice<I, [A]>, &'a NonZeroIndexSlice<J, [B]> }
__internal_impl_partialeq2! { NonZeroIndexSlice<I, [A]>, &'a mut NonZeroIndexSlice<J, [B]> }

macro_rules! array_impls {
    ($($N: expr)+) => {$(
        __internal_impl_partialeq! { NonZeroIndexVec<I, A>, [B; $N] }
        __internal_impl_partialeq! { NonZeroIndexVec<I, A>, &'b [B; $N] }
        __internal_impl_partialeq! { NonZeroIndexSlice<I, [A]>, [B; $N] }
        __internal_impl_partialeq! { NonZeroIndexSlice<I, [A]>, &'b [B; $N] }
        // __internal_impl_partialeq! { &'a NonZeroIndexSlice<I, [A]>, [B; $N] }
        // __internal_impl_partialeq! { &'a NonZeroIndexSlice<I, [A]>, &'b [B; $N] }
    )+};
}

array_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

#[inline(never)]
#[cold]
#[doc(hidden)]
pub fn __max_check_fail(u: usize, max: usize) -> ! {
    panic!("index_vec index overflow: {} is outside the range [0, {})", u, max,)
}

#[cfg(feature = "serialize")]
impl<I: NonZeroIdx, T: serde::ser::Serialize> serde::ser::Serialize for NonZeroIndexVec<I, T> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, I: NonZeroIdx, T: serde::de::Deserialize<'de>> serde::de::Deserialize<'de>
    for NonZeroIndexVec<I, T>
{
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::deserialize(deserializer).map(Self::from_vec)
    }
}

#[cfg(feature = "serialize")]
impl<I: NonZeroIdx, T: serde::ser::Serialize> serde::ser::Serialize for NonZeroIndexBox<I, T> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, I: NonZeroIdx, T: serde::de::Deserialize<'de>> serde::de::Deserialize<'de>
    for NonZeroIndexBox<I, [T]>
{
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Box::<[T]>::deserialize(deserializer).map(Into::into)
    }
}
