use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use crate::Allocator;

/// A thread-safe pool for reusing `Allocator` instances to reduce allocation overhead.
///
/// Internally uses a `Vec` protected by a `Mutex` to store available allocators.
#[derive(Default)]
pub struct AllocatorPool {
    allocators: Mutex<Vec<Allocator>>,
}

impl AllocatorPool {
    /// Creates a new `AllocatorPool` pre-filled with the given number of default `Allocator` instances.
    pub fn new(size: usize) -> AllocatorPool {
        let allocators = std::iter::repeat_with(Allocator::default).take(size).collect();
        AllocatorPool { allocators: Mutex::new(allocators) }
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
            allocators.pop()
        };
        let allocator = allocator.unwrap_or_default();

        AllocatorGuard { allocator: ManuallyDrop::new(allocator), pool: self }
    }

    /// Add an `Allocator` to the pool.
    ///
    /// The `Allocator` should be empty, ready to be re-used.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    fn add(&self, allocator: Allocator) {
        let mut allocators = self.allocators.lock().unwrap();
        allocators.push(allocator);
    }
}

/// A guard object representing exclusive access to an `Allocator` from the pool.
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

impl DerefMut for AllocatorGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.allocator
    }
}

impl Drop for AllocatorGuard<'_> {
    /// Return `Allocator` back to the pool.
    fn drop(&mut self) {
        // SAFETY: After taking ownership of the `Allocator`, we do not touch the `ManuallyDrop` again
        let mut allocator = unsafe { ManuallyDrop::take(&mut self.allocator) };
        allocator.reset();
        self.pool.add(allocator);
    }
}
