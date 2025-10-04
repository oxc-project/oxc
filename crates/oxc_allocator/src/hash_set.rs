//! A hash set without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
//!
//! See [`HashSet`] for more details.
//!
//! [`FxHasher`]: rustc_hash::FxHasher

// All methods which just delegate to `hashbrown::HashSet` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    hash::Hash,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use bumpalo::Bump;
use rustc_hash::FxBuildHasher;

// Re-export additional types from `hashbrown`
pub use hashbrown::hash_set::{
    Difference, Drain, Entry, ExtractIf, Intersection, IntoIter, Iter, SymmetricDifference, Union,
};

use crate::{Allocator, HashMap};

type FxHashSet<'alloc, T> = hashbrown::HashSet<T, FxBuildHasher, &'alloc Bump>;

/// A hash set without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
///
/// Just a thin wrapper around [`hashbrown::HashSet`], which disables the `Drop` implementation.
///
/// All APIs are the same, except create a [`HashSet`] with
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
#[derive(Debug)]
pub struct HashSet<'alloc, T>(ManuallyDrop<FxHashSet<'alloc, T>>);

/// SAFETY: Same as `HashMap`. See `HashMap`'s doc comment for details.
unsafe impl<T: Sync> Sync for HashSet<'_, T> {}

// TODO: `IntoIter` and other consuming iterators provided by `hashbrown` are `Drop`.
// Wrap them in `ManuallyDrop` to prevent that.

impl<'alloc, T> HashSet<'alloc, T> {
    /// Const assertion that `T` is not `Drop`.
    /// Must be referenced in all methods which create a `HashSet`.
    const ASSERT_T_IS_NOT_DROP: () = {
        assert!(!std::mem::needs_drop::<T>(), "Cannot create a HashSet<T> where T is a Drop type");
    };

    /// Creates an empty [`HashSet`]. It will be allocated with the given allocator.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate
    /// until it is first inserted into.
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let inner = FxHashSet::with_hasher_in(FxBuildHasher, allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Creates an empty [`HashSet`] with the specified capacity. It will be allocated with the given allocator.
    ///
    /// The hash set will be able to hold at least capacity elements without reallocating.
    /// If capacity is 0, the hash set will not allocate.
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let inner =
            FxHashSet::with_capacity_and_hasher_in(capacity, FxBuildHasher, allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Create a new [`HashSet`] whose elements are taken from an iterator and allocated in the given `allocator`.
    ///
    /// This is behaviorially identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self
    where
        T: Eq + Hash,
    {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let iter = iter.into_iter();

        // Use the iterator's lower size bound.
        // This follows `hashbrown::HashSet`'s `from_iter` implementation.
        //
        // This is a trade-off:
        // * Negative: If lower bound is too low, the `HashSet` may have to grow and reallocate during `for_each` loop.
        // * Positive: Avoids potential large over-allocation for iterators where upper bound may be a large over-estimate
        //   e.g. filter iterators.
        let capacity = iter.size_hint().0;
        let set = FxHashSet::with_capacity_and_hasher_in(capacity, FxBuildHasher, allocator.bump());
        // Wrap in `ManuallyDrop` *before* calling `for_each`, so compiler doesn't insert unnecessary code
        // to drop the `FxHashSet` in case of a panic in iterator's `next` method
        let mut set = ManuallyDrop::new(set);

        iter.for_each(|v| {
            set.insert(v);
        });

        Self(set)
    }

    /// Calling this method produces a compile-time panic.
    ///
    /// This method would be unsound, because [`HashSet`] is `Sync`, and the underlying allocator
    /// (`bumpalo::Bump`) is not `Sync`.
    ///
    /// This method exists only to block access as much as possible to the underlying
    /// `hashbrown::HashSet::allocator` method. That method can still be accessed via explicit `Deref`
    /// (`hash_set.deref().allocator()`), but that's unsound.
    ///
    /// We'll prevent access to it completely and remove this method as soon as we can.
    // TODO: Do that!
    #[expect(clippy::unused_self)]
    pub fn allocator(&self) -> &'alloc Bump {
        const { panic!("This method cannot be called") };
        unreachable!();
    }
}

// Provide access to all `hashbrown::HashSet`'s methods via deref
impl<'alloc, T> Deref for HashSet<'alloc, T> {
    type Target = FxHashSet<'alloc, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T> DerefMut for HashSet<'alloc, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut FxHashSet<'alloc, T> {
        &mut self.0
    }
}

impl<'alloc, T> IntoIterator for HashSet<'alloc, T> {
    type IntoIter = IntoIter<T, &'alloc Bump>;
    type Item = T;

    /// Creates a consuming iterator, that is, one that moves each value out of the set
    /// in arbitrary order.
    ///
    /// The set cannot be used after calling this.
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        // TODO: `hashbrown::hash_set::IntoIter` is `Drop`.
        // Wrap it in `ManuallyDrop` to prevent that.
        inner.into_iter()
    }
}

impl<'alloc, 'i, T> IntoIterator for &'i HashSet<'alloc, T> {
    type IntoIter = <&'i FxHashSet<'alloc, T> as IntoIterator>::IntoIter;
    type Item = &'i T;

    /// Creates an iterator over the values of a `HashSet` in arbitrary order.
    ///
    /// The iterator element type is `&'a T`.
    ///
    /// Return the same [`Iter`] struct as by the `iter` method on [`HashSet`].
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
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

// Note: `Index` and `Extend` are implemented via `Deref`

/// Convert `HashMap<T, ()>` to `HashSet<T>`.
///
/// This conversion is zero cost, as `HashSet<T>` is just a wrapper around `HashMap<T, ()>`.
impl<'alloc, T> From<HashMap<'alloc, T, ()>> for HashSet<'alloc, T> {
    #[inline(always)]
    fn from(map: HashMap<'alloc, T, ()>) -> Self {
        let inner_map = ManuallyDrop::into_inner(map.0);
        let inner_set = hashbrown::HashSet::from(inner_map);
        Self(ManuallyDrop::new(inner_set))
    }
}
