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
