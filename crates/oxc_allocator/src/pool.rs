use std::{mem::ManuallyDrop, ops::Deref, sync::Mutex};

use crate::Allocator;

/// A thread-safe pool for reusing [`Allocator`] instances to reduce allocation overhead.
///
/// Internally uses a `Vec` protected by a `Mutex` to store available allocators.
#[derive(Default)]
pub struct AllocatorPool {
    allocators: Mutex<Vec<AllocatorWrapper>>,
}

impl AllocatorPool {
    /// Creates a new [`AllocatorPool`] pre-filled with the given number of default `AllocatorWrapper` instances.
    pub fn new(size: usize) -> AllocatorPool {
        let allocators = AllocatorWrapper::new_vec(size);
        AllocatorPool { allocators: Mutex::new(allocators) }
    }

    /// Retrieves an [`Allocator`] from the pool, or creates a new one if the pool is empty.
    ///
    /// Returns an [`AllocatorGuard`] that gives access to the allocator.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    pub fn get(&self) -> AllocatorGuard {
        let allocator = {
            let mut allocators = self.allocators.lock().unwrap();
            allocators.pop()
        };
        let allocator = allocator.unwrap_or_else(AllocatorWrapper::new);

        AllocatorGuard { allocator: ManuallyDrop::new(allocator), pool: self }
    }

    /// Add an [`AllocatorWrapper`] to the pool.
    ///
    /// The `Allocator` should be empty, ready to be re-used.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    fn add(&self, allocator: AllocatorWrapper) {
        let mut allocators = self.allocators.lock().unwrap();
        allocators.push(allocator);
    }
}

/// A guard object representing exclusive access to an [`Allocator`] from the pool.
///
/// On drop, the `Allocator` is reset and returned to the pool.
pub struct AllocatorGuard<'alloc_pool> {
    allocator: ManuallyDrop<AllocatorWrapper>,
    pool: &'alloc_pool AllocatorPool,
}

impl Deref for AllocatorGuard<'_> {
    type Target = Allocator;

    fn deref(&self) -> &Self::Target {
        self.allocator.get()
    }
}

impl Drop for AllocatorGuard<'_> {
    /// Return [`Allocator`] back to the pool.
    fn drop(&mut self) {
        // SAFETY: After taking ownership of the `AllocatorWrapper`, we do not touch the `ManuallyDrop` again
        let mut allocator = unsafe { ManuallyDrop::take(&mut self.allocator) };
        allocator.reset();
        self.pool.add(allocator);
    }
}

mod wrapper {
    use crate::Allocator;

    /// Structure which wraps an [`Allocator`].
    ///
    /// This implementation adds no value to `Allocator`, but we can add support for fixed-size allocators
    /// by providing a different implementation of `AllocatorWrapper` behind a feature flag.
    pub struct AllocatorWrapper(Allocator);

    impl AllocatorWrapper {
        /// Create a new [`AllocatorWrapper`].
        pub fn new() -> Self {
            Self(Allocator::default())
        }

        /// Get reference to underlying [`Allocator`].
        pub fn get(&self) -> &Allocator {
            &self.0
        }

        /// Reset the [`Allocator`] in this [`AllocatorWrapper`].
        pub fn reset(&mut self) {
            self.0.reset();
        }

        /// Create a `Vec` of [`AllocatorWrapper`]s.
        pub fn new_vec(size: usize) -> Vec<Self> {
            std::iter::repeat_with(Self::new).take(size).collect()
        }
    }
}

use wrapper::AllocatorWrapper;
