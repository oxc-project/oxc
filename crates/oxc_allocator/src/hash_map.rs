//! A hash map without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
//!
//! See [`HashMap`] for more details.
//!
//! [`FxHasher`]: rustc_hash::FxHasher

// All methods which just delegate to `hashbrown::HashMap` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    hash::Hash,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use bumpalo::Bump;
use rustc_hash::FxBuildHasher;

// Re-export additional types from `hashbrown`
pub use hashbrown::{
    Equivalent, TryReserveError,
    hash_map::{
        Drain, Entry, EntryRef, ExtractIf, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys,
        OccupiedError, Values, ValuesMut,
    },
};

use crate::Allocator;

type FxHashMap<'alloc, K, V> = hashbrown::HashMap<K, V, FxBuildHasher, &'alloc Bump>;

/// A hash map without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
///
/// Just a thin wrapper around [`hashbrown::HashMap`], which disables the `Drop` implementation.
///
/// All APIs are the same, except create a [`HashMap`] with
/// either [`new_in`](HashMap::new_in) or [`with_capacity_in`](HashMap::with_capacity_in).
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`HashMap::new_in`] and all other methods which create
/// a [`HashMap`] will refuse to compile if either key or value is a [`Drop`] type.
///
/// [`FxHasher`]: rustc_hash::FxHasher
#[derive(Debug)]
pub struct HashMap<'alloc, K, V>(pub(crate) ManuallyDrop<FxHashMap<'alloc, K, V>>);

/// SAFETY: Even though `Bump` is not `Sync`, we can make `HashMap<K, V>` `Sync` if both `K` and `V`
/// are `Sync` because:
///
/// 1. No public methods allow access to the `&Bump` that `HashMap` contains (in `hashbrown::HashMap`),
///    so user cannot illegally obtain 2 `&Bump`s on different threads via `HashMap`.
///
/// 2. All internal methods which access the `&Bump` take a `&mut self`.
///    `&mut HashMap` cannot be transferred across threads, and nor can an owned `HashMap`
///    (`HashMap` is not `Send`).
///    Therefore these methods taking `&mut self` can be sure they're not operating on a `HashMap`
///    which has been moved across threads.
///
/// Note: `HashMap` CANNOT be `Send`, even if `K` and `V` are `Send`, because that would allow 2 `HashMap`s
/// on different threads to both allocate into same arena simultaneously. `Bump` is not thread-safe,
/// and this would be undefined behavior.
///
/// ### Soundness holes
///
/// This is not actually fully sound. There are 2 holes I (@overlookmotel) am aware of:
///
/// 1. `allocator` method, which does allow access to the `&Bump` that `HashMap` contains.
/// 2. `Clone` impl on `hashbrown::HashMap`, which may perform allocations in the arena, given only a
///    `&self` reference.
///
/// [`HashMap::allocator`] prevents accidental access to the underlying method of `hashbrown::HashMap`,
/// and `clone` called on a `&HashMap` clones the `&HashMap` reference, not the `HashMap` itself (harmless).
/// But both can be accessed via explicit `Deref` (`hash_map.deref().allocator()` or `hash_map.deref().clone()`),
/// so we don't have complete soundness.
///
/// To close these holes we need to remove `Deref` and `DerefMut` impls on `HashMap`, and instead add
/// methods to `HashMap` itself which pass on calls to the inner `hashbrown::HashMap`.
///
/// TODO: Fix these holes.
/// TODO: Remove any other methods that currently allow performing allocations with only a `&self` reference.
unsafe impl<K: Sync, V: Sync> Sync for HashMap<'_, K, V> {}

// TODO: `IntoIter`, `Drain`, and other consuming iterators provided by `hashbrown` are `Drop`.
// Wrap them in `ManuallyDrop` to prevent that.

impl<'alloc, K, V> HashMap<'alloc, K, V> {
    /// Const assertions that `K` and `V` are not `Drop`.
    /// Must be referenced in all methods which create a `HashMap`.
    const ASSERT_K_AND_V_ARE_NOT_DROP: () = {
        assert!(
            !std::mem::needs_drop::<K>(),
            "Cannot create a HashMap<K, V> where K is a Drop type"
        );
        assert!(
            !std::mem::needs_drop::<V>(),
            "Cannot create a HashMap<K, V> where V is a Drop type"
        );
    };

    /// Creates an empty [`HashMap`]. It will be allocated with the given allocator.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate
    /// until it is first inserted into.
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_K_AND_V_ARE_NOT_DROP };

        let inner = FxHashMap::with_hasher_in(FxBuildHasher, allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Creates an empty [`HashMap`] with the specified capacity. It will be allocated with the given allocator.
    ///
    /// The hash map will be able to hold at least capacity elements without reallocating.
    /// If capacity is 0, the hash map will not allocate.
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_K_AND_V_ARE_NOT_DROP };

        let inner =
            FxHashMap::with_capacity_and_hasher_in(capacity, FxBuildHasher, allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Create a new [`HashMap`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorally identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = (K, V)>>(
        iter: I,
        allocator: &'alloc Allocator,
    ) -> Self
    where
        K: Eq + Hash,
    {
        const { Self::ASSERT_K_AND_V_ARE_NOT_DROP };

        let iter = iter.into_iter();

        // Use the iterator's lower size bound.
        // This follows `hashbrown::HashMap`'s `from_iter` implementation.
        //
        // This is a trade-off:
        // * Negative: If lower bound is too low, the `HashMap` may have to grow and reallocate during `for_each` loop.
        // * Positive: Avoids potential large over-allocation for iterators where upper bound may be a large over-estimate
        //   e.g. filter iterators.
        let capacity = iter.size_hint().0;
        let map = FxHashMap::with_capacity_and_hasher_in(capacity, FxBuildHasher, allocator.bump());
        // Wrap in `ManuallyDrop` *before* calling `for_each`, so compiler doesn't insert unnecessary code
        // to drop the `FxHashMap` in case of a panic in iterator's `next` method
        let mut map = ManuallyDrop::new(map);

        iter.for_each(|(k, v)| {
            map.insert(k, v);
        });

        Self(map)
    }

    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    ///
    /// The map cannot be used after calling this. The iterator element type is `K`.
    #[inline(always)]
    pub fn into_keys(self) -> IntoKeys<K, V, &'alloc Bump> {
        let inner = ManuallyDrop::into_inner(self.0);
        inner.into_keys()
    }

    /// Creates a consuming iterator visiting all the values in arbitrary order.
    ///
    /// The map cannot be used after calling this. The iterator element type is `V`.
    #[inline(always)]
    pub fn into_values(self) -> IntoValues<K, V, &'alloc Bump> {
        let inner = ManuallyDrop::into_inner(self.0);
        inner.into_values()
    }

    /// Calling this method produces a compile-time panic.
    ///
    /// This method would be unsound, because [`HashMap`] is `Sync`, and the underlying allocator
    /// (`bumpalo::Bump`) is not `Sync`.
    ///
    /// This method exists only to block access as much as possible to the underlying
    /// `hashbrown::HashMap::allocator` method. That method can still be accessed via explicit `Deref`
    /// (`hash_map.deref().allocator()`), but that's unsound.
    ///
    /// We'll prevent access to it completely and remove this method as soon as we can.
    // TODO: Do that!
    #[expect(clippy::unused_self)]
    pub fn allocator(&self) -> &'alloc Bump {
        const { panic!("This method cannot be called") };
        unreachable!();
    }
}

// Provide access to all `hashbrown::HashMap`'s methods via deref
impl<'alloc, K, V> Deref for HashMap<'alloc, K, V> {
    type Target = FxHashMap<'alloc, K, V>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, K, V> DerefMut for HashMap<'alloc, K, V> {
    #[inline]
    fn deref_mut(&mut self) -> &mut FxHashMap<'alloc, K, V> {
        &mut self.0
    }
}

impl<'alloc, K, V> IntoIterator for HashMap<'alloc, K, V> {
    type IntoIter = IntoIter<K, V, &'alloc Bump>;
    type Item = (K, V);

    /// Creates a consuming iterator, that is, one that moves each key-value pair out of the map
    /// in arbitrary order.
    ///
    /// The map cannot be used after calling this.
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        // TODO: `hashbrown::hash_map::IntoIter` is `Drop`.
        // Wrap it in `ManuallyDrop` to prevent that.
        inner.into_iter()
    }
}

impl<'alloc, 'i, K, V> IntoIterator for &'i HashMap<'alloc, K, V> {
    type IntoIter = <&'i FxHashMap<'alloc, K, V> as IntoIterator>::IntoIter;
    type Item = (&'i K, &'i V);

    /// Creates an iterator over the entries of a `HashMap` in arbitrary order.
    ///
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// Return the same [`Iter`] struct as by the `iter` method on [`HashMap`].
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'alloc, 'i, K, V> IntoIterator for &'i mut HashMap<'alloc, K, V> {
    type IntoIter = <&'i mut FxHashMap<'alloc, K, V> as IntoIterator>::IntoIter;
    type Item = (&'i K, &'i mut V);

    /// Creates an iterator over the entries of a `HashMap` in arbitrary order
    /// with mutable references to the values.
    ///
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// Return the same [`IterMut`] struct as by the `iter_mut` method on [`HashMap`].
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V> PartialEq for HashMap<'_, K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V> Eq for HashMap<'_, K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

// Note: `Index` and `Extend` are implemented via `Deref`
