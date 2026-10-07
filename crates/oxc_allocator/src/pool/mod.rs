use std::{mem::ManuallyDrop, ops::Deref};

use crate::Allocator;

mod standard;
use standard::StandardAllocatorPool;

// Fixed size allocators are only supported on 64-bit little-endian platforms at present.
// They are only enabled if `fixed_size` Cargo feature is enabled.
//
// Note: Importing the `fixed_size` module would cause a compilation error on 32-bit systems.
#[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
mod fixed_size;
#[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
use fixed_size::FixedSizeAllocatorPool;
#[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
pub use fixed_size::{FixedSizeAllocatorMetadata, free_fixed_size_allocator};

/// A thread-safe pool for reusing [`Allocator`] instances to reduce allocation overhead.
///
/// Uses either:
/// 1. Standard allocators - suitable for general use.
/// 2. Fixed-size allocators - compatible with raw transfer.
///
/// Standard allocator pool is created by [`AllocatorPool::new`].
/// Fixed-size allocator pool is created by [`AllocatorPool::new_fixed_size`].
///
/// Fixed-size allocators are only supported on 64-bit little-endian platforms at present,
/// and require the `fixed_size` Cargo feature to be enabled.
#[repr(transparent)]
pub struct AllocatorPool(AllocatorPoolInner);

/// Inner type of [`AllocatorPool`], holding either a standard or fixed-size allocator pool.
enum AllocatorPoolInner {
    Standard(StandardAllocatorPool),
    #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
    FixedSize(FixedSizeAllocatorPool),
}

impl AllocatorPool {
    /// Create a new [`AllocatorPool`] for use across the specified number of threads,
    /// which uses standard allocators.
    pub fn new(thread_count: usize) -> AllocatorPool {
        Self(AllocatorPoolInner::Standard(StandardAllocatorPool::new(thread_count)))
    }

    /// Create a new [`AllocatorPool`] for use across the specified number of threads,
    /// which uses fixed-size allocators (suitable for raw transfer).
    #[cfg(feature = "fixed_size")]
    pub fn new_fixed_size(thread_count: usize) -> AllocatorPool {
        #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
        {
            Self(AllocatorPoolInner::FixedSize(FixedSizeAllocatorPool::new(thread_count)))
        }

        #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
        {
            let _thread_count = thread_count; // Avoid unused vars lint warning
            panic!("Fixed size allocators are only supported on 64-bit little-endian platforms");
        }
    }

    /// Retrieve an [`Allocator`] from the pool, or create a new one if the pool is empty.
    ///
    /// Returns an [`AllocatorGuard`] that gives access to the allocator.
    ///
    /// # Panics
    ///
    /// * Panics if the underlying mutex is poisoned.
    /// * Panics if a new allocator needs to be created but memory allocation fails.
    pub fn get(&self) -> AllocatorGuard<'_> {
        let allocator = match &self.0 {
            AllocatorPoolInner::Standard(pool) => pool.get(),
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            AllocatorPoolInner::FixedSize(pool) => pool.get(),
        };

        AllocatorGuard { allocator: ManuallyDrop::new(allocator), pool: self }
    }

    /// Number of allocators this pool was constructed with.
    ///
    /// For a fixed-size pool this is the number of arenas that were actually created
    /// (on Windows this can be less than the requested `thread_count`).
    pub fn len(&self) -> usize {
        match &self.0 {
            AllocatorPoolInner::Standard(pool) => pool.len(),
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            AllocatorPoolInner::FixedSize(pool) => pool.len(),
        }
    }

    /// `true` if this pool was constructed with no allocators.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Buffer ids of every allocator in this pool.
    ///
    /// Empty for a standard pool, which does not stamp buffer ids.
    ///
    /// Callers use this to tell JS to drop its cached raw-transfer views before the pool is dropped.
    pub fn buffer_ids(&self) -> &[u32] {
        match &self.0 {
            AllocatorPoolInner::Standard(_) => &[],
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            AllocatorPoolInner::FixedSize(pool) => pool.buffer_ids(),
        }
    }

    /// `true` if this pool uses fixed-size raw-transfer allocators.
    pub fn is_fixed_size(&self) -> bool {
        match &self.0 {
            AllocatorPoolInner::Standard(_) => false,
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            AllocatorPoolInner::FixedSize(_) => true,
        }
    }

    /// Add an [`Allocator`] to the pool.
    ///
    /// The `Allocator` is reset by this method, so it's ready to be re-used.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    fn add(&self, allocator: Allocator) {
        // SAFETY: This method is only called from `AllocatorGuard::drop`.
        // `AllocatorGuard`s are only created by `AllocatorPool::get`, so the `Allocator` must have
        // been created by this pool. Therefore, it is the correct type for the pool.
        unsafe {
            match &self.0 {
                AllocatorPoolInner::Standard(pool) => pool.add(allocator),
                #[cfg(all(
                    feature = "fixed_size",
                    target_pointer_width = "64",
                    target_endian = "little"
                ))]
                AllocatorPoolInner::FixedSize(pool) => pool.add(allocator),
            }
        }
    }
}

/// A guard object representing exclusive access to an [`Allocator`] from the pool.
///
/// On drop, the `Allocator` is reset and returned to the pool.
pub struct AllocatorGuard<'alloc_pool> {
    allocator: ManuallyDrop<Allocator>,
    pool: &'alloc_pool AllocatorPool,
}

impl Deref for AllocatorGuard<'_> {
    type Target = Allocator;

    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

impl Drop for AllocatorGuard<'_> {
    /// Return [`Allocator`] back to the pool.
    fn drop(&mut self) {
        // SAFETY: After taking ownership of the `Allocator`, we do not touch the `ManuallyDrop` again
        let allocator = unsafe { ManuallyDrop::take(&mut self.allocator) };
        self.pool.add(allocator);
    }
}

#[cfg(all(test, feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
mod buffer_id_tests {
    use std::mem::size_of;

    use rustc_hash::FxHashSet;

    use super::*;

    #[test]
    fn buffer_id_two_pools_are_distinct() {
        let pool_a = AllocatorPool::new_fixed_size(2);
        let pool_b = AllocatorPool::new_fixed_size(2);
        assert_eq!(pool_a.len(), 2);
        assert_eq!(pool_b.len(), 2);
        assert!(pool_a.is_fixed_size());
        assert!(pool_b.is_fixed_size());

        let a0 = pool_a.get();
        let a1 = pool_a.get();
        let b0 = pool_b.get();
        let b1 = pool_b.get();

        assert!(a0.is_fixed_size());
        assert!(a1.is_fixed_size());
        assert!(b0.is_fixed_size());
        assert!(b1.is_fixed_size());

        // SAFETY: these allocators came from `new_fixed_size` pools.
        let ids = unsafe {
            [
                a0.fixed_size_buffer_id(),
                a1.fixed_size_buffer_id(),
                b0.fixed_size_buffer_id(),
                b1.fixed_size_buffer_id(),
            ]
        };
        assert_eq!(FxHashSet::from_iter(ids).len(), 4);
        assert_eq!(size_of::<FixedSizeAllocatorMetadata>(), 8);
    }

    #[test]
    fn buffer_id_new_fixed_size_len() {
        assert_eq!(AllocatorPool::new_fixed_size(2).len(), 2);
    }

    #[test]
    fn buffer_id_standard_pool_is_not_fixed_size() {
        let pool = AllocatorPool::new(2);
        assert_eq!(pool.len(), 2);
        assert!(!pool.is_fixed_size());
        assert!(!pool.get().is_fixed_size());
        assert!(pool.buffer_ids().is_empty());
    }

    /// The language server forgets a folder's buffers by id, so `buffer_ids` has to list exactly
    /// the ids of the arenas the pool hands out — no more, no fewer.
    #[test]
    fn buffer_ids_match_the_arenas_the_pool_hands_out() {
        let pool = AllocatorPool::new_fixed_size(3);
        let expected = pool.buffer_ids().iter().copied().collect::<FxHashSet<_>>();
        assert_eq!(expected.len(), pool.len());

        let guards = [pool.get(), pool.get(), pool.get()];
        // SAFETY: these allocators came from a `new_fixed_size` pool.
        let actual = unsafe {
            guards.iter().map(|guard| guard.fixed_size_buffer_id()).collect::<FxHashSet<_>>()
        };
        assert_eq!(actual, expected);
    }
}
