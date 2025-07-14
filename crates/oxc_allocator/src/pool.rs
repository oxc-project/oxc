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

#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
mod wrapper {
    use crate::Allocator;

    /// Structure which wraps an [`Allocator`].
    ///
    /// Default implementation which is just a wrapper around an [`Allocator`].
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

#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
mod wrapper {
    use crate::{
        Allocator,
        fixed_size::{CHUNK_ALIGN, FixedSizeAllocator},
    };

    /// Structure which wraps an [`Allocator`] with fixed size of 2 GiB, and aligned on 4 GiB.
    ///
    /// See [`FixedSizeAllocator`] for more details.
    pub struct AllocatorWrapper(FixedSizeAllocator);

    impl AllocatorWrapper {
        /// Create a new [`AllocatorWrapper`].
        pub fn new() -> Self {
            Self(FixedSizeAllocator::new())
        }

        /// Get reference to underlying [`Allocator`].
        pub fn get(&self) -> &Allocator {
            &self.0
        }

        /// Reset the [`Allocator`] in this [`AllocatorWrapper`].
        pub fn reset(&mut self) {
            // Set cursor back to end
            self.0.reset();

            // Set data pointer back to start.
            // SAFETY: Fixed-size allocators have data pointer originally aligned on `CHUNK_ALIGN`,
            // and size less than `CHUNK_ALIGN`. So we can restore original data pointer by rounding down
            // to next multiple of `CHUNK_ALIGN`.
            unsafe {
                let data_ptr = self.0.data_ptr();
                let offset = data_ptr.as_ptr() as usize % CHUNK_ALIGN;
                let data_ptr = data_ptr.sub(offset);
                self.0.set_data_ptr(data_ptr);
            }
        }

        /// Create a `Vec` of [`AllocatorWrapper`]s.
        pub fn new_vec(size: usize) -> Vec<Self> {
            // Each allocator consumes a large block of memory, so create them on demand instead of upfront
            Vec::with_capacity(size)
        }
    }
}

use wrapper::AllocatorWrapper;
