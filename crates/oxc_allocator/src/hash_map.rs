//! A hash map without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
//!
//! See [`HashMap`] for more details.
//!
//! [`FxHasher`]: rustc_hash::FxHasher

// All methods which just delegate to `hashbrown::HashMap` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    hash::{BuildHasher, Hash},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use rustc_hash::FxBuildHasher;

use crate::bump::Bump;

// Re-export additional types from `hashbrown`
pub use hashbrown::{
    Equivalent, TryReserveError,
    hash_map::{
        Drain, Entry, EntryRef, ExtractIf, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys,
        OccupiedError, Values, ValuesMut,
    },
};

use crate::Allocator;

type InnerHashMap<'alloc, K, V, H> = hashbrown::HashMap<K, V, H, &'alloc Bump>;

/// A hash map without `Drop`, that stores data in arena allocator.
///
/// Just a thin wrapper around [`hashbrown::HashMap`], which disables the `Drop` implementation.
///
/// All APIs are the same, except create a [`HashMapImpl`] with
/// either [`new_in`](HashMapImpl::new_in) or [`with_capacity_in`](HashMapImpl::with_capacity_in).
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`HashMapImpl::new_in`] and all other methods which create
/// a [`HashMapImpl`] will refuse to compile if either key or value is a [`Drop`] type.
pub struct HashMapImpl<'alloc, K, V, H: BuildHasher>(
    pub(crate) ManuallyDrop<InnerHashMap<'alloc, K, V, H>>,
);

impl<K, V, H: BuildHasher> std::fmt::Debug for HashMapImpl<'_, K, V, H>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

/// A hash map without `Drop`, that uses [`FxHasher`] to hash keys, and stores data in arena allocator.
///
/// This is a type alias for [`HashMapImpl`] with [`FxBuildHasher`].
///
/// [`FxHasher`]: rustc_hash::FxHasher
pub type HashMap<'alloc, K, V> = HashMapImpl<'alloc, K, V, FxBuildHasher>;

/// SAFETY: Even though `Bump` is not `Sync`, we can make `HashMapImpl<K, V, H>` `Sync` if both `K` and `V`
/// are `Sync` because:
///
/// 1. No public methods allow access to the `&Bump` that `HashMapImpl` contains (in `hashbrown::HashMap`),
///    so user cannot illegally obtain 2 `&Bump`s on different threads via `HashMapImpl`.
///
/// 2. All internal methods which access the `&Bump` take a `&mut self`.
///    `&mut HashMapImpl` cannot be transferred across threads, and nor can an owned `HashMapImpl`
///    (`HashMapImpl` is not `Send`).
///    Therefore these methods taking `&mut self` can be sure they're not operating on a `HashMapImpl`
///    which has been moved across threads.
///
/// Note: `HashMapImpl` CANNOT be `Send`, even if `K` and `V` are `Send`, because that would allow 2 `HashMapImpl`s
/// on different threads to both allocate into same arena simultaneously. `Bump` is not thread-safe,
/// and this would be undefined behavior.
///
/// ### Soundness holes
///
/// This is not actually fully sound. There are 2 holes I (@overlookmotel) am aware of:
///
/// 1. `allocator` method, which does allow access to the `&Bump` that `HashMapImpl` contains.
/// 2. `Clone` impl on `hashbrown::HashMap`, which may perform allocations in the arena, given only a
///    `&self` reference.
///
/// [`HashMapImpl::allocator`] prevents accidental access to the underlying method of `hashbrown::HashMap`,
/// and `clone` called on a `&HashMapImpl` clones the `&HashMapImpl` reference, not the `HashMapImpl` itself (harmless).
/// But both can be accessed via explicit `Deref` (`hash_map.deref().allocator()` or `hash_map.deref().clone()`),
/// so we don't have complete soundness.
///
/// To close these holes we need to remove `Deref` and `DerefMut` impls on `HashMapImpl`, and instead add
/// methods to `HashMapImpl` itself which pass on calls to the inner `hashbrown::HashMap`.
///
/// TODO: Fix these holes.
/// TODO: Remove any other methods that currently allow performing allocations with only a `&self` reference.
unsafe impl<K: Sync, V: Sync, H: BuildHasher> Sync for HashMapImpl<'_, K, V, H> {}

// TODO: `IntoIter`, `Drain`, and other consuming iterators provided by `hashbrown` are `Drop`.
// Wrap them in `ManuallyDrop` to prevent that.

impl<'alloc, K, V, H: BuildHasher + Default> HashMapImpl<'alloc, K, V, H> {
    /// Const assertions that `K` and `V` are not `Drop`.
    /// Must be referenced in all methods which create a `HashMapImpl`.
    const ASSERT_K_AND_V_ARE_NOT_DROP: () = {
        assert!(
            !std::mem::needs_drop::<K>(),
            "Cannot create a HashMapImpl<K, V, H> where K is a Drop type"
        );
        assert!(
            !std::mem::needs_drop::<V>(),
            "Cannot create a HashMapImpl<K, V, H> where V is a Drop type"
        );
    };

    /// Creates an empty [`HashMapImpl`]. It will be allocated with the given allocator.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate
    /// until it is first inserted into.
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_K_AND_V_ARE_NOT_DROP };

        let inner = InnerHashMap::with_hasher_in(H::default(), allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Creates an empty [`HashMapImpl`] with the specified capacity. It will be allocated with the given allocator.
    ///
    /// The hash map will be able to hold at least capacity elements without reallocating.
    /// If capacity is 0, the hash map will not allocate.
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_K_AND_V_ARE_NOT_DROP };

        let inner =
            InnerHashMap::with_capacity_and_hasher_in(capacity, H::default(), allocator.bump());
        Self(ManuallyDrop::new(inner))
    }

    /// Create a new [`HashMapImpl`] whose elements are taken from an iterator and
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
        // * Negative: If lower bound is too low, the `HashMapImpl` may have to grow and reallocate during `for_each` loop.
        // * Positive: Avoids potential large over-allocation for iterators where upper bound may be a large over-estimate
        //   e.g. filter iterators.
        let capacity = iter.size_hint().0;
        let map =
            InnerHashMap::with_capacity_and_hasher_in(capacity, H::default(), allocator.bump());
        // Wrap in `ManuallyDrop` *before* calling `for_each`, so compiler doesn't insert unnecessary code
        // to drop the `InnerHashMap` in case of a panic in iterator's `next` method
        let mut map = ManuallyDrop::new(map);

        iter.for_each(|(k, v)| {
            map.insert(k, v);
        });

        Self(map)
    }
}

impl<'alloc, K, V, H: BuildHasher> HashMapImpl<'alloc, K, V, H> {
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
    /// This method would be unsound, because [`HashMapImpl`] is `Sync`, and the underlying allocator
    /// (`Bump`) is not `Sync`.
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
impl<'alloc, K, V, H: BuildHasher> Deref for HashMapImpl<'alloc, K, V, H> {
    type Target = InnerHashMap<'alloc, K, V, H>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, K, V, H: BuildHasher> DerefMut for HashMapImpl<'alloc, K, V, H> {
    #[inline]
    fn deref_mut(&mut self) -> &mut InnerHashMap<'alloc, K, V, H> {
        &mut self.0
    }
}

impl<'alloc, K, V, H: BuildHasher> IntoIterator for HashMapImpl<'alloc, K, V, H> {
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

impl<'alloc, 'i, K, V, H: BuildHasher> IntoIterator for &'i HashMapImpl<'alloc, K, V, H> {
    type IntoIter = <&'i InnerHashMap<'alloc, K, V, H> as IntoIterator>::IntoIter;
    type Item = (&'i K, &'i V);

    /// Creates an iterator over the entries of a `HashMapImpl` in arbitrary order.
    ///
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// Return the same [`Iter`] struct as by the `iter` method on [`HashMapImpl`].
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'alloc, 'i, K, V, H: BuildHasher> IntoIterator for &'i mut HashMapImpl<'alloc, K, V, H> {
    type IntoIter = <&'i mut InnerHashMap<'alloc, K, V, H> as IntoIterator>::IntoIter;
    type Item = (&'i K, &'i mut V);

    /// Creates an iterator over the entries of a `HashMapImpl` in arbitrary order
    /// with mutable references to the values.
    ///
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// Return the same [`IterMut`] struct as by the `iter_mut` method on [`HashMapImpl`].
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V, H: BuildHasher> PartialEq for HashMapImpl<'_, K, V, H>
where
    K: Eq + Hash,
    V: PartialEq,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, H: BuildHasher> Eq for HashMapImpl<'_, K, V, H>
where
    K: Eq + Hash,
    V: Eq,
{
}

// Note: `Index` and `Extend` are implemented via `Deref`
