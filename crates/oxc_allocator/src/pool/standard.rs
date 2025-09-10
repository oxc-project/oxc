use std::{iter, mem::ManuallyDrop, sync::Mutex};

use crate::Allocator;

use super::AllocatorGuard;

/// A thread-safe pool for reusing [`Allocator`] instances to reduce allocation overhead.
///
/// Internally uses a `Vec` protected by a `Mutex` to store available allocators.
pub struct AllocatorPool {
    allocators: Mutex<Vec<Allocator>>,
}

impl AllocatorPool {
    /// Creates a new [`AllocatorPool`] for use across the specified number of threads.
    pub fn new(thread_count: usize) -> AllocatorPool {
        let allocators = iter::repeat_with(Allocator::new).take(thread_count).collect();
        AllocatorPool { allocators: Mutex::new(allocators) }
    }

    /// Retrieves an [`Allocator`] from the pool, or creates a new one if the pool is empty.
    ///
    /// Returns an [`AllocatorGuard`] that gives access to the allocator.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    pub fn get(&self) -> AllocatorGuard<'_> {
        let allocator = {
            let mut allocators = self.allocators.lock().unwrap();
            allocators.pop()
        };
        let allocator = allocator.unwrap_or_else(Allocator::new);

        AllocatorGuard { allocator: ManuallyDrop::new(allocator), pool: self }
    }

    /// Add an [`Allocator`] to the pool.
    ///
    /// The `Allocator` is reset by this method, so it's ready to be re-used.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    pub(super) fn add(&self, mut allocator: Allocator) {
        allocator.reset();
        let mut allocators = self.allocators.lock().unwrap();
        allocators.push(allocator);
    }
}

// Dummy implementations of interfaces from `fixed_size`, just to stop clippy complaining.
// Seems to be necessary due to feature unification.
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
pub use dummies::*;
