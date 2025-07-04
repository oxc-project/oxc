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

#[cfg(not(feature = "fixed_size"))]
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

#[cfg(feature = "fixed_size")]
mod wrapper {
    use std::{
        alloc::{self, Layout},
        mem::ManuallyDrop,
        ptr::NonNull,
    };

    use crate::Allocator;

    // Only 64-bit little-endian platforms are supported at present
    const IS_SUPPORTED_PLATFORM: bool =
        cfg!(all(target_pointer_width = "64", target_endian = "little"));

    const TWO_GIB: usize = 1 << 31;
    // `1 << 32`.
    // We use `IS_SUPPORTED_PLATFORM as usize * 32` to avoid compilation failure on 32-bit platforms.
    const FOUR_GIB: usize = 1 << (IS_SUPPORTED_PLATFORM as usize * 32);

    // What we ideally want is an allocation 2 GiB in size, aligned on 4 GiB.
    // But system allocator on Mac OS refuses allocations with 4 GiB alignment.
    // https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/unix.rs#L16-L27
    // https://github.com/rust-lang/rust/issues/30170
    //
    // So we instead allocate 4 GiB with 2 GiB alignment, and then use either the 1st or 2nd half
    // of the allocation, one of which is guaranteed to be on a 4 GiB boundary.
    //
    // TODO: We could use this workaround only on Mac OS, and just allocate what we actually want on
    // Linux and Windows. Windows OS allocator also doesn't support high alignment allocations,
    // so Rust contains a workaround which over-allocates (6 GiB in this case).
    // https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/windows.rs#L120-L137
    // Could just use that built-in workaround, rather than implementing our own.
    const ALLOC_SIZE: usize = FOUR_GIB;
    const ALLOC_ALIGN: usize = TWO_GIB;
    const CHUNK_SIZE: usize = TWO_GIB;
    const CHUNK_ALIGN: usize = FOUR_GIB;

    const ALLOC_LAYOUT: Layout = match Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN) {
        Ok(layout) => layout,
        Err(_) => unreachable!(),
    };

    /// Structure which wraps an [`Allocator`] with fixed size of 2 GiB, and aligned on 4 GiB.
    ///
    /// To achieve this, we manually allocate memory to back the `Allocator`'s single chunk.
    /// We over-allocate 4 GiB, and then use a part of that allocation to back the `Allocator`.
    /// Inner `Allocator` is wrapped in `ManuallyDrop` to prevent it freeing the memory itself,
    /// and `AllocatorWrapper` has a custom `Drop` impl which frees the whole of the original allocation.
    pub struct AllocatorWrapper {
        /// Pointer to start of original allocation
        alloc_ptr: NonNull<u8>,
        /// `Allocator` which utilizes part of the original allocation
        allocator: ManuallyDrop<Allocator>,
    }

    impl AllocatorWrapper {
        /// Create a new [`AllocatorWrapper`].
        pub fn new() -> Self {
            assert!(
                IS_SUPPORTED_PLATFORM,
                "Fixed size allocators are only supported on 64-bit little-endian platforms"
            );

            // Allocate block of memory.
            // SAFETY: Layout does not have zero size.
            let alloc_ptr = unsafe { alloc::alloc(ALLOC_LAYOUT) };
            let Some(alloc_ptr) = NonNull::new(alloc_ptr) else {
                alloc::handle_alloc_error(ALLOC_LAYOUT);
            };

            // Get pointer to use for allocator chunk, aligned to 4 GiB.
            // SAFETY: `offset` is either 0 or `TWO_GIB`.
            // We allocated 4 GiB of memory, so adding `offset` to `alloc_ptr` is in bounds.
            let chunk_ptr = unsafe {
                let offset = alloc_ptr.as_ptr() as usize % ALLOC_SIZE;
                alloc_ptr.add(offset)
            };

            debug_assert!(chunk_ptr.as_ptr() as usize % CHUNK_ALIGN == 0);

            // SAFETY: Memory region starting at `chunk_ptr` with `CHUNK_SIZE` bytes is within
            // the allocation we just made.
            // `chunk_ptr` has high alignment (4 GiB). `size` is large and a high power of 2 (2 GiB).
            let allocator = unsafe { Allocator::from_raw_parts(chunk_ptr, CHUNK_SIZE) };

            // Store pointer to original allocation, so it can be used to deallocate in `drop`
            Self { alloc_ptr, allocator: ManuallyDrop::new(allocator) }
        }

        /// Get reference to underlying [`Allocator`].
        pub fn get(&self) -> &Allocator {
            &self.allocator
        }

        /// Reset the [`Allocator`] in this [`AllocatorWrapper`].
        pub fn reset(&mut self) {
            self.allocator.reset();
        }

        /// Create a `Vec` of [`AllocatorWrapper`]s.
        pub fn new_vec(size: usize) -> Vec<Self> {
            // Each allocator consumes a large block of memory, so create them on demand instead of upfront
            Vec::with_capacity(size)
        }
    }

    impl Drop for AllocatorWrapper {
        fn drop(&mut self) {
            // SAFETY: Originally allocated at `alloc_ptr`, with layout `ALLOC_LAYOUT`
            unsafe { alloc::dealloc(self.alloc_ptr.as_ptr(), ALLOC_LAYOUT) }
        }
    }

    // SAFETY: `Allocator` is `Send`.
    // Moving `alloc_ptr: NonNull<u8>` across threads along with the `Allocator` is safe.
    unsafe impl Send for AllocatorWrapper {}
}

use wrapper::AllocatorWrapper;
