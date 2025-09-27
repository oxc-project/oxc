//! A hash set without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
//!
//! See [`HashSet`] for more details.
//!
//! [`FxHasher`]: rustc_hash::FxHasher

// All methods which just delegate to `HashMap` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{hash::Hash, mem::ManuallyDrop};

use bumpalo::Bump;

use crate::{
    Allocator,
    hash_map::{HashMap, IntoKeys, Keys},
};

/// A hash set without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
///
/// Just a thin wrapper around [`HashMap<T, ()>`], which provides set semantics.
///
/// All APIs are similar to `std::collections::HashSet`, except create a [`HashSet`] with
/// either [`new_in`](HashSet::new_in) or [`with_capacity_in`](HashSet::with_capacity_in).
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`HashSet::new_in`] and all other methods which create
/// a [`HashSet`] will refuse to compile if the key is a [`Drop`] type.
///
/// [`FxHasher`]: rustc_hash::FxHasher
pub struct HashSet<'alloc, T>(ManuallyDrop<HashMap<'alloc, T, ()>>);

/// SAFETY: `HashSet` is `Sync` if `T` is `Sync` because it's just a wrapper around `HashMap<T, ()>`,
/// which is already `Sync` when `T` is `Sync`.
unsafe impl<T: Sync> Sync for HashSet<'_, T> {}

impl<'alloc, T> HashSet<'alloc, T> {
    /// Creates an empty [`HashSet`]. It will be allocated with the given allocator.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate
    /// until it is first inserted into.
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(HashMap::new_in(allocator)))
    }

    /// Creates an empty [`HashSet`] with the specified capacity. It will be allocated with the given allocator.
    ///
    /// The hash set will be able to hold at least capacity elements without reallocating.
    /// If capacity is 0, the hash set will not allocate.
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(HashMap::with_capacity_in(capacity, allocator)))
    }

    /// Create a new [`HashSet`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorially identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self
    where
        T: Eq + Hash,
    {
        Self(ManuallyDrop::new(HashMap::from_iter_in(iter.into_iter().map(|k| (k, ())), allocator)))
    }

    /// Returns the number of elements the set can hold without reallocating.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `&'a T`.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.keys()
    }

    /// Returns the number of elements in the set.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Clears the set, removing all values.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns `true` if the set contains a value.
    #[inline(always)]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ?Sized,
    {
        self.0.contains_key(value)
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the set did not previously contain this value, `true` is returned.
    /// - If the set already contained this value, `false` is returned.
    #[inline(always)]
    pub fn insert(&mut self, value: T) -> bool
    where
        T: Eq + Hash,
    {
        self.0.insert(value, ()).is_none()
    }

    /// Removes a value from the set. Returns whether the value was present in the set.
    #[inline(always)]
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ?Sized,
    {
        self.0.remove(value).is_some()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `HashSet`. The collection may reserve more space to speculatively
    /// avoid frequent reallocations.
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize)
    where
        T: Eq + Hash,
    {
        self.0.reserve(additional);
    }

    /// Visits the values representing the difference, i.e., the values that are in `self` but not in `other`.
    #[inline]
    pub fn difference<'a>(&'a self, other: &'a HashSet<'alloc, T>) -> impl Iterator<Item = &'a T>
    where
        T: Eq + Hash,
    {
        self.iter().filter(|&v| !other.contains(v))
    }
}

impl<'alloc, T> IntoIterator for HashSet<'alloc, T> {
    type IntoIter = IntoKeys<T, (), &'alloc Bump>;
    type Item = T;

    /// Creates a consuming iterator, that is, one that moves each value out of the set
    /// in arbitrary order.
    ///
    /// The set cannot be used after calling this.
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        inner.into_keys()
    }
}

impl<'alloc, 'i, T> IntoIterator for &'i HashSet<'alloc, T> {
    type IntoIter = Keys<'i, T, ()>;
    type Item = &'i T;

    /// Creates an iterator over the values of a `HashSet` in arbitrary order.
    ///
    /// The iterator element type is `&'a T`.
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.keys()
    }
}

impl<T> PartialEq for HashSet<'_, T>
where
    T: Eq + Hash,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for HashSet<'_, T> where T: Eq + Hash {}
