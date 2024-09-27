//! This crate is a fork of `index_vec`, <https://github.com/thomcc/index_vec>
//! It helps with defining "newtype"-style wrappers around `usize` (or
//! other integers), `Vec<T>`, and `[T]` so that some additional type safety can
//! be gained at zero cost.
//!
//! ## Example / Overview
//! ```rust
//! use oxc_index::{IndexVec, IndexSlice, index_vec};
//!
//! oxc_index::define_index_type! {
//!     // Define StrIdx to use only 32 bits internally (you can use usize, u16,
//!     // and even u8).
//!     pub struct StrIdx = u32;
//!
//!     // The defaults are very reasonable, but this macro can let
//!     // you customize things quite a bit:
//!
//!     // By default, creating a StrIdx would check an incoming `usize against
//!     // `u32::max_value()`, as u32 is the wrapped index type. Lets imagine that
//!     // StrIdx has to interface with an external system that uses signed ints.
//!     // We can change the checking behavior to complain on i32::max_value()
//!     // instead:
//!     MAX_INDEX = i32::max_value() as usize;
//!
//!     // We can also disable checking all-together if we are more concerned with perf
//!     // than any overflow problems, or even do so, but only for debug builds: Quite
//!     // pointless here, but an okay example
//!     DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
//!
//!     // And more too, see this macro's docs for more info.
//! }
//!
//! // Create a vector which can be accessed using `StrIdx`s.
//! let mut strs: IndexVec<StrIdx, &'static str> = index_vec!["strs", "bar", "baz"];
//!
//! // l is a `StrIdx`
//! let l = strs.last_idx();
//! assert_eq!(strs[l], "baz");
//!
//! let new_i = strs.push("quux");
//! assert_eq!(strs[new_i], "quux");
//!
//! // The slice APIs are wrapped as well.
//! let s: &IndexSlice<StrIdx, [&'static str]> = &strs[StrIdx::new(1)..];
//! assert_eq!(s[0], "bar");
//!
//! // Indices are mostly interoperable with `usize`, and support
//! // a lot of what you might want to do to an index.
//!
//! // Comparison
//! assert_eq!(StrIdx::new(0), 0usize);
//!
//! // Addition
//! assert_eq!(StrIdx::new(0) + 1, 1usize);
//!
//! // Subtraction
//! assert_eq!(StrIdx::new(1) - 1, 0usize);
//!
//! // Wrapping
//! assert_eq!(StrIdx::new(5) % strs.len(), 1usize);
//! // ...
//! ```
//! ## Background
//!
//! The goal is to help with the pattern of using a `type FooIdx = usize` to
//! access a `Vec<Foo>` with something that can statically prevent using a
//! `FooIdx` in a `Vec<Bar>`. It's most useful if you have a bunch of indices
//! referring to different sorts of vectors.
//!
//! The code was originally based on `rustc`'s `IndexVec` code, however that has
//! been almost entirely rewritten (except for the cases where it's trivial,
//! e.g. the Vec wrapper).
//!
//! ## Other crates
//!
//! The [`indexed_vec`](https://crates.io/crates/indexed_vec) crate predates
//! this, and is a much closer copy of the code from `rustc`. Unfortunately,
//! this means it does not compile on stable.
//!
//! If you're looking for something further from a vec and closer to a map, you
//! might find [`handy`](https://crates.io/crates/handy),
//! [`slotmap`](https://crates.io/crates/slotmap), or
//! [`slab`](https://crates.io/crates/slab) to be closer what you want.
//!
//! ## FAQ
//!
//! #### Wouldn't `define_index_type` be better as a proc macro?
//!
//! Probably. It's not a proc macro because I tend to avoid them where possible
//! due to wanting to minimize compile times. If the issues around proc-macro
//! compile times are fixed, then I'll revisit this.
//!
//! I also may eventually add a proc-macro feature which is not required, but
//! avoids some of the grossness.
//!
//! #### Does `define_index_type` do too much?
//!
//! Possibly. It defines a type, implements a bunch of functions on it, and
//! quite a few traits. That said, it's intended to be a very painless journey
//! from `Vec<T>` + `usize` to `IndexVec<I, T>`. If it left it up to the
//! developer to do those things, it would be too annoying to be worth using.
//!
//! #### The syntax for the options in `define_index_type` is terrible.
//!
//! I'm open to suggestions.
//!
//! #### Does it support no_std?
//!
//! Yes, although it uses `extern crate alloc;`, of course.
//!
//! #### Does it support serde?
//!
//! Yes, but only if you turn on the `serialize` feature.
//!
//! #### What features are planned?
//!
//! Planned is a bit strong but here are the things I would find useful.
//!
//! - Support any remaining parts of the slice/vec api.
//! - Add typesafe wrappers for SmallVec/ArrayVec (behind a cargo `feature`, of
//!   course).
//! - Better syntax for the define_index_type macro (no concrete ideas).
//! - Allow the generated type to be a tuple struct, or use a specific field
//!   name.
//! - Allow use of indices for string types (the primary benefit here would
//!   probably be the ability to e.g. use u32 without too much pain rather than
//!   mixing up indices from different strings -- but you never know!)
//! - Allow index types such as NonZeroU32 and such, if it can be done sanely.
//! - ...
//!
#![allow(clippy::inline_always)]
#![allow(clippy::partialeq_ne_impl)]
#![no_std]
extern crate alloc;

use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    vec,
    vec::Vec,
};
use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    fmt::Debug,
    hash::Hash,
    iter::{self, FromIterator},
    marker::PhantomData,
    ops::Range,
    slice,
};
mod idxslice;
mod indexing;
pub use idxslice::{IndexBox, IndexSlice};
pub use indexing::{IdxRangeBounds, IdxSliceIndex};
#[cfg(feature = "rayon")]
pub use rayon_impl::*;
#[cfg(feature = "rayon")]
mod rayon_impl;

#[macro_use]
mod macros;

/// Represents a wrapped value convertible to and from a `usize`.
///
/// Generally you implement this via the [`define_index_type!`] macro, rather
/// than manually implementing it.
///
/// # Overflow
///
/// `Idx` impls are allowed to be smaller than `usize`, which means converting
/// `usize` to an `Idx` implementation might have to handle overflow.
///
/// The way overflow is handled is up to the implementation of `Idx`, but it's
/// generally panicking, unless it was turned off via the
/// `DISABLE_MAX_INDEX_CHECK` option in [`define_index_type!`]. If you need more
/// subtle handling than this, then you're on your own (or, well, either handle
/// it earlier, or pick a bigger index type).
///
/// Note: I'm open for suggestions on how to handle this case, but do not want
/// the typical cases (E.g. Idx is a newtyped `usize` or `u32`), to become more
/// complex.
pub trait Idx: Copy + 'static + Ord + Debug + Hash {
    /// Construct an Index from a `usize`. This is equivalent to `From<usize>`.
    ///
    /// Note that this will panic if `idx` does not fit (unless checking has
    /// been disabled, as mentioned above). Also note that `Idx` implementations
    /// are free to define what "fit" means as they desire.
    fn from_usize(idx: usize) -> Self;

    /// Get the underlying index. This is equivalent to `Into<usize>`
    fn index(self) -> usize;
}

/// A macro equivalent to the stdlib's `vec![]`, but producing an `IndexVec`.
#[macro_export]
macro_rules! index_vec {
    ($($tokens:tt)*) => {
        $crate::IndexVec::from_vec(vec![$($tokens)*])
    }
}

/// A macro similar to the stdlib's `vec![]`, but producing an
/// `Box<IndexSlice<I, [T]>>` (That is, an `IndexBox<I, [T]>`).
#[macro_export]
macro_rules! index_box {
    ($($tokens:tt)*) => {
        $crate::IndexVec::from_vec(vec![$($tokens)*]).into_boxed_slice()
    }
}

/// A Vec that only accepts indices of a specific type.
///
/// This is a thin wrapper around `Vec`, to the point where the backing vec is a
/// public property (called `raw`). This is in part because I know this API is
/// not a complete mirror of Vec's (patches welcome). In the worst case, you can
/// always do what you need to the Vec itself.
///
/// Note that this implements Deref/DerefMut to [`IndexSlice`], and so all the
/// methods on IndexSlice are available as well. See it's documentation for some
/// further information.
///
/// The following extensions to the Vec APIs are added (in addition to the ones
/// mentioned in IndexSlice's documentation):
///
/// - [`IndexVec::next_idx`], [`IndexSlice::last_idx`] give the next and most
///   recent index returned by `push`.
/// - [`IndexVec::push`] returns the index the item was inserted at.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexVec<I: Idx, T> {
    /// Our wrapped Vec.
    pub raw: Vec<T>,
    _marker: PhantomData<fn(&I)>,
}

// SAFETY: Whether `IndexVec` is `Send` depends only on the data,
// not the phantom data.
unsafe impl<I: Idx, T> Send for IndexVec<I, T> where T: Send {}

impl<I: Idx, T: fmt::Debug> fmt::Debug for IndexVec<I, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.raw, fmt)
    }
}
type Enumerated<Iter, I, T> = iter::Map<iter::Enumerate<Iter>, fn((usize, T)) -> (I, T)>;

impl<I: Idx, T> IndexVec<I, T> {
    /// Construct a new IndexVec.
    #[inline]
    pub fn new() -> Self {
        IndexVec { raw: Vec::new(), _marker: PhantomData }
    }

    /// Construct a `IndexVec` from a `Vec<T>`.
    ///
    /// Panics if it's length is too large for our index type.
    #[inline]
    pub fn from_vec(vec: Vec<T>) -> Self {
        // See if `I::from_usize` might be upset by this length.
        let _ = I::from_usize(vec.len());
        IndexVec { raw: vec, _marker: PhantomData }
    }

    /// Construct an IndexVec that can hold at least `capacity` items before
    /// reallocating. See [`Vec::with_capacity`].
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        IndexVec { raw: Vec::with_capacity(capacity), _marker: PhantomData }
    }

    /// Similar to `self.into_iter().enumerate()` but with indices of `I` and
    /// not `usize`.
    #[inline(always)]
    pub fn into_iter_enumerated(self) -> Enumerated<vec::IntoIter<T>, I, T> {
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
        R: IdxRangeBounds<I>,
    {
        self.raw.splice(range.into_range(), replace_with)
    }

    /// Similar to `self.drain(r).enumerate()` but with indices of `I` and not
    /// `usize`.
    #[inline]
    pub fn drain_enumerated<R: IdxRangeBounds<I>>(
        &mut self,
        range: R,
    ) -> Enumerated<vec::Drain<'_, T>, I, T> {
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

    /// Converts the vector into an owned [`IndexSlice`], dropping excess capacity.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<IndexSlice<I, [T]>> {
        let b = self.raw.into_boxed_slice();
        // SAFETY: `IndexSlice` is a thin wrapper around `[T]` with the added marker for the index.
        unsafe { Box::from_raw(Box::into_raw(b) as *mut IndexSlice<I, [T]>) }
    }

    /// Return an iterator that removes the items from the requested range. See
    /// [`Vec::drain`].
    ///
    /// See also [`IndexVec::drain_enumerated`], which gives you indices (of the
    /// correct type) as you iterate.
    #[inline]
    pub fn drain<R: IdxRangeBounds<I>>(&mut self, range: R) -> vec::Drain<'_, T> {
        self.raw.drain(range.into_range())
    }

    /// Shrinks the capacity of the vector as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit();
    }

    /// Shrinks the capacity of the vector with a lower bound.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.raw.shrink_to(min_capacity);
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
    pub fn get<J: IdxSliceIndex<I, T>>(&self, index: J) -> Option<&J::Output> {
        index.get(self.as_slice())
    }

    /// Get a mut ref to the item at the provided index, or None for out of
    /// bounds
    #[inline]
    pub fn get_mut<J: IdxSliceIndex<I, T>>(&mut self, index: J) -> Option<&mut J::Output> {
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
    pub fn extend_from_slice(&mut self, other: &IndexSlice<I, [T]>)
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

    /// Get a IndexSlice over this vector. See `as_raw_slice` for converting to
    /// a `&[T]` (or access `self.raw`).
    #[inline(always)]
    pub fn as_slice(&self) -> &IndexSlice<I, [T]> {
        IndexSlice::new(&self.raw)
    }

    /// Get a mutable IndexSlice over this vector. See `as_raw_slice_mut` for
    /// converting to a `&mut [T]` (or access `self.raw`).
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut IndexSlice<I, [T]> {
        IndexSlice::new_mut(&mut self.raw)
    }
}

impl<I: Idx, T> Default for IndexVec<I, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Idx, T> Extend<T> for IndexVec<I, T> {
    #[inline]
    fn extend<J: IntoIterator<Item = T>>(&mut self, iter: J) {
        self.raw.extend(iter);
    }
}

impl<'a, I: Idx, T: 'a + Copy> Extend<&'a T> for IndexVec<I, T> {
    #[inline]
    fn extend<J: IntoIterator<Item = &'a T>>(&mut self, iter: J) {
        self.raw.extend(iter);
    }
}

impl<I: Idx, T> FromIterator<T> for IndexVec<I, T> {
    #[inline]
    fn from_iter<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = T>,
    {
        IndexVec { raw: FromIterator::from_iter(iter), _marker: PhantomData }
    }
}

impl<I: Idx, T> IntoIterator for IndexVec<I, T> {
    type IntoIter = vec::IntoIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> vec::IntoIter<T> {
        self.raw.into_iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a IndexVec<I, T> {
    type IntoIter = slice::Iter<'a, T>;
    type Item = &'a T;

    #[inline]
    fn into_iter(self) -> slice::Iter<'a, T> {
        self.raw.iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a mut IndexVec<I, T> {
    type IntoIter = slice::IterMut<'a, T>;
    type Item = &'a mut T;

    #[inline]
    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.raw.iter_mut()
    }
}

impl<I: Idx, T> From<IndexVec<I, T>> for Box<IndexSlice<I, [T]>> {
    #[inline]
    fn from(src: IndexVec<I, T>) -> Self {
        src.into_boxed_slice()
    }
}

impl<I: Idx, T> From<Box<IndexSlice<I, [T]>>> for IndexVec<I, T> {
    #[inline]
    fn from(src: Box<IndexSlice<I, [T]>>) -> Self {
        src.into_vec()
    }
}

impl<'a, I: Idx, T> From<Cow<'a, IndexSlice<I, [T]>>> for IndexVec<I, T>
where
    IndexSlice<I, [T]>: ToOwned<Owned = IndexVec<I, T>>,
{
    #[inline]
    fn from(s: Cow<'a, IndexSlice<I, [T]>>) -> IndexVec<I, T> {
        s.into_owned()
    }
}

impl<'a, I: Idx, T: Clone> From<&'a IndexSlice<I, [T]>> for IndexVec<I, T> {
    #[inline]
    fn from(src: &'a IndexSlice<I, [T]>) -> Self {
        src.to_owned()
    }
}
impl<'a, I: Idx, T: Clone> From<&'a mut IndexSlice<I, [T]>> for IndexVec<I, T> {
    #[inline]
    fn from(src: &'a mut IndexSlice<I, [T]>) -> Self {
        src.to_owned()
    }
}

impl<I: Idx, T> From<Vec<T>> for IndexVec<I, T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self { raw: v, _marker: PhantomData }
    }
}

impl<I: Idx, T: Clone> Clone for IndexVec<I, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { raw: self.raw.clone(), _marker: PhantomData }
    }

    #[inline]
    fn clone_from(&mut self, o: &Self) {
        self.raw.clone_from(&o.raw);
    }
}

impl<I: Idx, A> AsRef<[A]> for IndexVec<I, A> {
    #[inline]
    fn as_ref(&self) -> &[A] {
        &self.raw
    }
}

impl<I: Idx, A> AsMut<[A]> for IndexVec<I, A> {
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        &mut self.raw
    }
}

impl<I: Idx, A> AsRef<IndexSlice<I, [A]>> for IndexVec<I, A> {
    #[inline]
    fn as_ref(&self) -> &IndexSlice<I, [A]> {
        IndexSlice::new(&self.raw)
    }
}

impl<I: Idx, A> AsMut<IndexSlice<I, [A]>> for IndexVec<I, A> {
    #[inline]
    fn as_mut(&mut self) -> &mut IndexSlice<I, [A]> {
        IndexSlice::new_mut(&mut self.raw)
    }
}

impl<I: Idx, A> core::ops::Deref for IndexVec<I, A> {
    type Target = IndexSlice<I, [A]>;

    #[inline]
    fn deref(&self) -> &IndexSlice<I, [A]> {
        IndexSlice::new(&self.raw)
    }
}

impl<I: Idx, A> core::ops::DerefMut for IndexVec<I, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut IndexSlice<I, [A]> {
        IndexSlice::new_mut(&mut self.raw)
    }
}

impl<I: Idx, T> Borrow<IndexSlice<I, [T]>> for IndexVec<I, T> {
    #[inline]
    fn borrow(&self) -> &IndexSlice<I, [T]> {
        self.as_slice()
    }
}

impl<I: Idx, T> BorrowMut<IndexSlice<I, [T]>> for IndexVec<I, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut IndexSlice<I, [T]> {
        self.as_mut_slice()
    }
}

macro_rules! impl_partialeq {
    ($Lhs: ty, $Rhs: ty) => {
        impl<'a, 'b, A, B, I: Idx> PartialEq<$Rhs> for $Lhs
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &$Rhs) -> bool {
                self[..] == other[..]
            }

            #[inline]
            fn ne(&self, other: &$Rhs) -> bool {
                self[..] != other[..]
            }
        }
    };
}

macro_rules! impl_partialeq2 {
    ($Lhs: ty, $Rhs: ty) => {
        impl<'a, 'b, A, B, I: Idx, J: Idx> PartialEq<$Rhs> for $Lhs
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &$Rhs) -> bool {
                self.raw[..] == other.raw[..]
            }

            #[inline]
            fn ne(&self, other: &$Rhs) -> bool {
                self.raw[..] != other.raw[..]
            }
        }
    };
}

impl_partialeq! { IndexVec<I, A>, Vec<B> }
impl_partialeq! { IndexVec<I, A>, &'b [B] }
impl_partialeq! { IndexVec<I, A>, &'b mut [B] }

impl_partialeq2! { IndexVec<I, A>, &'b IndexSlice<J, [B]> }
impl_partialeq2! { IndexVec<I, A>, &'b mut IndexSlice<J, [B]> }

impl_partialeq! { &'a IndexSlice<I, [A]>, Vec<B> }
impl_partialeq! { &'a mut IndexSlice<I, [A]>, Vec<B> }

impl_partialeq! { IndexSlice<I, [A]>, &'b [B] }
impl_partialeq! { IndexSlice<I, [A]>, &'b mut [B] }

impl_partialeq2! { &'a IndexSlice<I, [A]>, IndexVec<J, B> }
impl_partialeq2! { &'a mut IndexSlice<I, [A]>, IndexVec<J, B> }

impl_partialeq2! { IndexSlice<I, [A]>, &'a IndexSlice<J, [B]> }
impl_partialeq2! { IndexSlice<I, [A]>, &'a mut IndexSlice<J, [B]> }

macro_rules! array_impls {
    ($($N: expr)+) => {$(
        impl_partialeq! { IndexVec<I, A>, [B; $N] }
        impl_partialeq! { IndexVec<I, A>, &'b [B; $N] }
        impl_partialeq! { IndexSlice<I, [A]>, [B; $N] }
        impl_partialeq! { IndexSlice<I, [A]>, &'b [B; $N] }
        // impl_partialeq! { &'a IndexSlice<I, [A]>, [B; $N] }
        // impl_partialeq! { &'a IndexSlice<I, [A]>, &'b [B; $N] }
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
impl<I: Idx, T: serde::ser::Serialize> serde::ser::Serialize for IndexVec<I, T> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, I: Idx, T: serde::de::Deserialize<'de>> serde::de::Deserialize<'de> for IndexVec<I, T> {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::deserialize(deserializer).map(Self::from_vec)
    }
}

#[cfg(feature = "serialize")]
impl<I: Idx, T: serde::ser::Serialize> serde::ser::Serialize for IndexBox<I, T> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, I: Idx, T: serde::de::Deserialize<'de>> serde::de::Deserialize<'de> for IndexBox<I, [T]> {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Box::<[T]>::deserialize(deserializer).map(Into::into)
    }
}

#[cfg(test)]
#[allow(clippy::legacy_numeric_constants)]
mod test {
    use super::*;

    define_index_type! {
        pub struct TestIdx = u32;
    }

    #[test]
    fn test_resize() {
        let mut v = IndexVec::<TestIdx, u32>::with_capacity(10);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());

        v.push(1);
        assert_eq!(v.len(), 1);

        v.resize(5, 1);
        assert_eq!(v.len(), 5);
        assert_eq!(v.as_slice(), &[1, 1, 1, 1, 1]);

        v.shrink_to_fit();
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn test_push_pop() {
        let mut v = IndexVec::<TestIdx, u32>::new();
        v.push(1);
        assert_eq!(v.pop(), Some(1));
    }

    #[test]
    fn test_clear() {
        let mut v: IndexVec<TestIdx, u32> = [1, 2, 3].into_iter().collect();
        assert_eq!(v.len(), 3);

        v.clear();
        assert_eq!(v.len(), 0);
        assert_eq!(v.as_slice(), &[]);
        assert_eq!(v, IndexVec::<TestIdx, u32>::new());
    }
}
