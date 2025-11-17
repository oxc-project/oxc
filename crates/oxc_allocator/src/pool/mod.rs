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
/// Supports three configurations:
/// 1. Standard allocators only - suitable for general use ([`AllocatorPool::new`]).
/// 2. Fixed-size allocators only - compatible with raw transfer ([`AllocatorPool::new_fixed_size`]).
/// 3. Dual mode with both types - allows choosing allocator type at runtime ([`AllocatorPool::new_dual`]).
///
/// The dual mode is useful when some allocations need raw transfer (e.g., files to be linted with JS plugins)
/// while others don't (e.g., dependency files only parsed for module resolution).
///
/// Fixed-size allocators are only supported on 64-bit little-endian platforms at present,
/// and require the `fixed_size` Cargo feature to be enabled.
pub struct AllocatorPool {
    standard: StandardAllocatorPool,
    #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
    fixed_size: FixedSizeAllocatorPool,
}

impl AllocatorPool {
    /// Create a new [`AllocatorPool`] for use across the specified number of threads,
    /// which uses standard allocators only.
    pub fn new(thread_count: usize) -> AllocatorPool {
        Self {
            standard: StandardAllocatorPool::new(thread_count),
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            fixed_size: FixedSizeAllocatorPool::new(0),
        }
    }

    /// Create a new [`AllocatorPool`] for use across the specified number of threads,
    /// which uses fixed-size allocators only (suitable for raw transfer).
    #[cfg(feature = "fixed_size")]
    pub fn new_fixed_size(thread_count: usize) -> AllocatorPool {
        #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
        {
            Self {
                standard: StandardAllocatorPool::new(0), // Not used, but need to initialize
                fixed_size: FixedSizeAllocatorPool::new(thread_count),
            }
        }

        #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
        {
            let _thread_count = thread_count; // Avoid unused vars lint warning
            panic!("Fixed size allocators are only supported on 64-bit little-endian platforms");
        }
    }

    /// Create a new [`AllocatorPool`] for use across the specified number of threads,
    /// which contains both standard and fixed-size allocators.
    ///
    /// This allows choosing the allocator type at runtime via [`AllocatorPool::get_maybe_fixed_size`].
    /// Useful when some allocations need raw transfer (e.g., files to be linted with JS plugins)
    /// while others don't (e.g., dependency files only parsed for module resolution).
    #[cfg(feature = "fixed_size")]
    pub fn new_dual(thread_count: usize) -> AllocatorPool {
        #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
        {
            Self {
                standard: StandardAllocatorPool::new(thread_count),
                fixed_size: FixedSizeAllocatorPool::new(thread_count),
            }
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
    /// If the pool was created with [`AllocatorPool::new_fixed_size`], returns a fixed-size allocator.
    /// Otherwise, returns a standard allocator.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    pub fn get(&self) -> AllocatorGuard<'_> {
        if cfg!(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))
        {
            let allocator = self.fixed_size.get();
            return AllocatorGuard {
                allocator: ManuallyDrop::new(allocator),
                is_fixed_size: true,
                pool: self,
            };
        }

        let allocator = self.standard.get();
        AllocatorGuard {
            allocator: ManuallyDrop::new(allocator),
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            is_fixed_size: false,
            pool: self,
        }
    }

    /// Retrieve an [`Allocator`] from the pool, choosing between standard and fixed-size allocators.
    ///
    /// Returns an [`AllocatorGuard`] that gives access to the allocator.
    ///
    /// # Parameters
    ///
    /// * `fixed_size` - If `true`, returns a fixed-size allocator (suitable for raw transfer).
    ///   If `false`, returns a standard allocator.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The underlying mutex is poisoned.
    /// - `fixed_size` is `true` but the pool was not created with [`AllocatorPool::new_dual`] or [`AllocatorPool::new_fixed_size`].
    #[cfg(feature = "fixed_size")]
    pub fn get_maybe_fixed_size(&self, fixed_size: bool) -> AllocatorGuard<'_> {
        #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
        {
            if fixed_size {
                let allocator = self.fixed_size.get();
                return AllocatorGuard {
                    allocator: ManuallyDrop::new(allocator),
                    is_fixed_size: true,
                    pool: self,
                };
            }
        }

        #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
        if fixed_size {
            panic!("Fixed size allocators are only supported on 64-bit little-endian platforms");
        }

        let allocator = self.standard.get();
        AllocatorGuard {
            allocator: ManuallyDrop::new(allocator),
            #[cfg(all(
                feature = "fixed_size",
                target_pointer_width = "64",
                target_endian = "little"
            ))]
            is_fixed_size: false,
            pool: self,
        }
    }

    /// Add an [`Allocator`] to the pool.
    ///
    /// The `Allocator` is reset by this method, so it's ready to be re-used.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
    fn add(&self, allocator: Allocator, is_fixed_size: bool) {
        // SAFETY: This method is only called from `AllocatorGuard::drop`.
        // `AllocatorGuard`s are only created by `AllocatorPool::get` or `AllocatorPool::get_maybe_fixed_size`,
        // so the `Allocator` must have been created by this pool. Therefore, it is the correct type for the pool.
        unsafe {
            if is_fixed_size {
                self.fixed_size.add(allocator);
            } else {
                self.standard.add(allocator);
            }
        }
    }

    /// Add an [`Allocator`] to the pool (for platforms without fixed-size support).
    #[cfg(not(all(
        feature = "fixed_size",
        target_pointer_width = "64",
        target_endian = "little"
    )))]
    fn add(&self, allocator: Allocator) {
        // SAFETY: This method is only called from `AllocatorGuard::drop`.
        // `AllocatorGuard`s are only created by `AllocatorPool::get`, so the `Allocator` must have
        // been created by this pool. Therefore, it is the correct type for the pool.
        unsafe {
            self.standard.add(allocator);
        }
    }
}

/// A guard object representing exclusive access to an [`Allocator`] from the pool.
///
/// On drop, the `Allocator` is reset and returned to the pool.
pub struct AllocatorGuard<'alloc_pool> {
    allocator: ManuallyDrop<Allocator>,
    #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
    is_fixed_size: bool,
    pool: &'alloc_pool AllocatorPool,
}

impl AllocatorGuard<'_> {
    /// Returns `true` if this allocator is a fixed-size allocator (suitable for raw transfer).
    #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
    pub fn is_fixed_size(&self) -> bool {
        self.is_fixed_size
    }
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

        #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
        self.pool.add(allocator, self.is_fixed_size);

        #[cfg(not(all(
            feature = "fixed_size",
            target_pointer_width = "64",
            target_endian = "little"
        )))]
        self.pool.add(allocator);
    }
}
