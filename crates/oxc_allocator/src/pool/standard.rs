use std::{iter, sync::Mutex};

use crate::Allocator;

/// A thread-safe pool for reusing [`Allocator`] instances, that uses standard allocators.
///
/// Unlike `FixedSizeAllocatorPool`, the `Allocator`s used in this pool are suitable for general use,
/// but not for raw transfer.
pub struct StandardAllocatorPool {
    allocators: Mutex<Vec<Allocator>>,
}

impl StandardAllocatorPool {
    /// Create a new [`StandardAllocatorPool`] for use across the specified number of threads.
    #[cfg_attr(all(feature = "fixed_size", not(feature = "disable_fixed_size")), expect(dead_code))]
    pub fn new(thread_count: usize) -> StandardAllocatorPool {
        let allocators = iter::repeat_with(Allocator::new).take(thread_count).collect();
        StandardAllocatorPool { allocators: Mutex::new(allocators) }
    }

    /// Retrieve an [`Allocator`] from the pool, or create a new one if the pool is empty.
    ///
    /// # Panics
    /// Panics if the underlying mutex is poisoned.
    pub fn get(&self) -> Allocator {
        let allocator = {
            let mut allocators = self.allocators.lock().unwrap();
            allocators.pop()
        };
        allocator.unwrap_or_else(Allocator::new)
    }

    /// Add an [`Allocator`] to the pool.
    ///
    /// The `Allocator` is reset by this method, so it's ready to be re-used.
    ///
    /// # SAFETY
    /// The `Allocator` must have been created by a `StandardAllocatorPool` (not `FixedSizeAllocatorPool`).
    ///
    /// # Panics
    /// Panics if the underlying mutex is poisoned.
    pub(super) unsafe fn add(&self, mut allocator: Allocator) {
        allocator.reset();
        let mut allocators = self.allocators.lock().unwrap();
        allocators.push(allocator);
    }
}

// Dummy implementations of interfaces from `fixed_size`, just to stop clippy complaining.
// Seems to be necessary due to feature unification.
#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
#[allow(
    dead_code,
    missing_docs,
    clippy::missing_safety_doc,
    clippy::unused_self,
    clippy::allow_attributes
)]
mod dummies {
    use std::{ptr::NonNull, sync::atomic::AtomicBool};

    use crate::Allocator;

    #[doc(hidden)]
    pub struct FixedSizeAllocatorMetadata {
        pub id: u32,
        pub(crate) alloc_ptr: NonNull<u8>,
        pub is_double_owned: AtomicBool,
    }

    #[doc(hidden)]
    pub unsafe fn free_fixed_size_allocator(_metadata_ptr: NonNull<FixedSizeAllocatorMetadata>) {
        unreachable!();
    }

    #[doc(hidden)]
    impl Allocator {
        pub unsafe fn fixed_size_metadata_ptr(&self) -> NonNull<FixedSizeAllocatorMetadata> {
            unreachable!();
        }
    }
}
#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
pub use dummies::*;
