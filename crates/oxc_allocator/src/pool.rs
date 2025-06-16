use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use crate::Allocator;

/// A thread-safe pool for reusing `Allocator` instances to reduce allocation overhead.
///
/// Internally uses a `Vec` protected by a `Mutex` to store available allocators.
#[derive(Default)]
pub struct AllocatorPool {
    allocators: Arc<Mutex<Vec<Allocator>>>,
}

impl AllocatorPool {
    /// Creates a new `AllocatorPool` pre-filled with the given number of default `Allocator` instances.
    pub fn new(size: usize) -> AllocatorPool {
        let allocators = std::iter::repeat_with(Allocator::default).take(size).collect();
        AllocatorPool { allocators: Arc::new(Mutex::new(allocators)) }
    }

    /// Retrieves an `Allocator` from the pool, or creates a new one if the pool is empty.
    ///
    /// Returns an `AllocatorPoolGuard` that gives mutable access to the allocator.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    pub fn get(&self) -> AllocatorGuard {
        let allocator = {
            let mut allocators = self.allocators.lock().unwrap();
            allocators.pop().unwrap_or_default()
        };

        AllocatorGuard {
            allocator: ManuallyDrop::new(allocator),
            pool: Arc::clone(&self.allocators),
        }
    }
}

/// A guard object representing exclusive access to an `Allocator` from the pool.
///
/// On drop, the `Allocator` is reset and returned to the pool.
pub struct AllocatorGuard {
    allocator: ManuallyDrop<Allocator>,
    pool: Arc<Mutex<Vec<Allocator>>>,
}

impl Deref for AllocatorGuard {
    type Target = Allocator;

    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

impl DerefMut for AllocatorGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.allocator
    }
}

impl Drop for AllocatorGuard {
    fn drop(&mut self) {
        // SAFETY: we're taking ownership and promise not to drop `allocator` again.
        let mut allocator = unsafe { ManuallyDrop::take(&mut self.allocator) };
        allocator.reset();
        let mut allocators = self.pool.lock().unwrap();
        allocators.push(allocator);
    }
}
