use std::{iter, sync::Mutex};

use oxc_data_structures::stack::Stack;

use crate::Allocator;

/// A thread-safe pool for reusing [`Allocator`] instances, that uses standard allocators.
///
/// Unlike `FixedSizeAllocatorPool`, the `Allocator`s used in this pool are suitable for general use,
/// but not for raw transfer.
pub struct StandardAllocatorPool {
    /// Allocators currently in the pool.
    /// We use a `Stack` because it's faster than `Vec` for `push` and `pop`,
    /// and those are the operations we do while `Mutex` lock is held.
    /// The shorter the time lock is held, the less contention there is.
    allocators: Mutex<Stack<Allocator>>,
}

impl StandardAllocatorPool {
    /// Create a new [`StandardAllocatorPool`] for use across the specified number of threads.
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
