use std::{
    alloc::{self, GlobalAlloc, Layout, System},
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::Mutex,
};

use crate::{
    Allocator,
    fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE, RAW_METADATA_SIZE},
};

const TWO_GIB: usize = 1 << 31;
const FOUR_GIB: usize = 1 << 32;

/// A thread-safe pool for reusing [`Allocator`] instances to reduce allocation overhead.
///
/// Internally uses a `Vec` protected by a `Mutex` to store available allocators.
#[derive(Default)]
pub struct AllocatorPool {
    allocators: Mutex<Vec<FixedSizeAllocator>>,
}

impl AllocatorPool {
    /// Creates a new [`AllocatorPool`] with capacity for the given number of `FixedSizeAllocator` instances.
    pub fn new(size: usize) -> AllocatorPool {
        // Each allocator consumes a large block of memory, so create them on demand instead of upfront
        let allocators = Vec::with_capacity(size);
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
        let allocator = allocator.unwrap_or_else(FixedSizeAllocator::new);

        AllocatorGuard { allocator: ManuallyDrop::new(allocator), pool: self }
    }

    /// Add a [`FixedSizeAllocator`] to the pool.
    ///
    /// The `Allocator` should be empty, ready to be re-used.
    ///
    /// # Panics
    ///
    /// Panics if the underlying mutex is poisoned.
    fn add(&self, allocator: FixedSizeAllocator) {
        let mut allocators = self.allocators.lock().unwrap();
        allocators.push(allocator);
    }
}

/// A guard object representing exclusive access to an [`Allocator`] from the pool.
///
/// On drop, the `Allocator` is reset and returned to the pool.
pub struct AllocatorGuard<'alloc_pool> {
    allocator: ManuallyDrop<FixedSizeAllocator>,
    pool: &'alloc_pool AllocatorPool,
}

impl Deref for AllocatorGuard<'_> {
    type Target = Allocator;

    fn deref(&self) -> &Self::Target {
        &self.allocator.allocator
    }
}

impl Drop for AllocatorGuard<'_> {
    /// Return [`Allocator`] back to the pool.
    fn drop(&mut self) {
        // SAFETY: After taking ownership of the `FixedSizeAllocator`, we do not touch the `ManuallyDrop` again
        let mut allocator = unsafe { ManuallyDrop::take(&mut self.allocator) };
        allocator.reset();
        self.pool.add(allocator);
    }
}

// What we ideally want is an allocation 2 GiB in size, aligned on 4 GiB.
// But system allocator on Mac OS refuses allocations with 4 GiB alignment.
// https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/unix.rs#L16-L27
// https://github.com/rust-lang/rust/issues/30170
//
// So we instead allocate 4 GiB with 2 GiB alignment, and then use either the 1st or 2nd half
// of the allocation, one of which is guaranteed to be on a 4 GiB boundary.
//
// TODO: We could use this workaround only on Mac OS, and just allocate what we actually want on Linux.
// Windows OS allocator also doesn't support high alignment allocations, so Rust contains a workaround
// which over-allocates (6 GiB in this case).
// https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/windows.rs#L120-L137
// Could just use that built-in workaround, rather than implementing our own, or allocate a 6 GiB chunk
// with alignment 16, to skip Rust's built-in workaround.
// Note: Rust's workaround will likely commit a whole page of memory, just to store the real pointer.
const ALLOC_SIZE: usize = BLOCK_SIZE + TWO_GIB;
const ALLOC_ALIGN: usize = TWO_GIB;

const ALLOC_LAYOUT: Layout = match Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

/// Structure which wraps an [`Allocator`] with fixed size of 2 GiB, and aligned on 4 GiB.
///
/// To achieve this, we manually allocate memory to back the `Allocator`'s single chunk.
/// We over-allocate 4 GiB, and then use a part of that allocation to back the `Allocator`.
/// Inner `Allocator` is wrapped in `ManuallyDrop` to prevent it freeing the memory itself,
/// and `FixedSizeAllocator` has a custom `Drop` impl which frees the whole of the original allocation.
///
/// We allocate via `System` allocator, bypassing any registered alternative global allocator
/// (e.g. Mimalloc in linter). Mimalloc complains that it cannot serve allocations with high alignment,
/// and presumably it's pointless to try to obtain such large allocations from a thread-local heap,
/// so better to go direct to the system allocator anyway.
pub struct FixedSizeAllocator {
    /// `Allocator` which utilizes part of the original allocation
    allocator: ManuallyDrop<Allocator>,
    /// Pointer to start of original allocation
    alloc_ptr: NonNull<u8>,
}

impl FixedSizeAllocator {
    /// Create a new [`FixedSizeAllocator`].
    #[expect(clippy::items_after_statements)]
    pub fn new() -> Self {
        // Allocate block of memory.
        // SAFETY: `ALLOC_LAYOUT` does not have zero size.
        let alloc_ptr = unsafe { System.alloc(ALLOC_LAYOUT) };
        let Some(alloc_ptr) = NonNull::new(alloc_ptr) else {
            alloc::handle_alloc_error(ALLOC_LAYOUT);
        };

        // Get pointer to use for allocator chunk, aligned to 4 GiB.
        // `alloc_ptr` is aligned on 2 GiB, so `alloc_ptr % FOUR_GIB` is either 0 or `TWO_GIB`.
        //
        // * If allocation is already aligned on 4 GiB, `offset == 0`.
        //   Chunk occupies 1st half of the allocation.
        // * If allocation is not aligned on 4 GiB, `offset == TWO_GIB`.
        //   Adding `offset` to `alloc_ptr` brings it up to 4 GiB alignment.
        //   Chunk occupies 2nd half of the allocation.
        //
        // Either way, `chunk_ptr` is aligned on 4 GiB.
        let offset = alloc_ptr.as_ptr() as usize % FOUR_GIB;
        // SAFETY: We allocated 4 GiB of memory, so adding `offset` to `alloc_ptr` is in bounds
        let chunk_ptr = unsafe { alloc_ptr.add(offset) };

        debug_assert!(chunk_ptr.as_ptr() as usize % BLOCK_ALIGN == 0);

        const CHUNK_SIZE: usize = BLOCK_SIZE - RAW_METADATA_SIZE;
        const _: () = assert!(CHUNK_SIZE % Allocator::RAW_MIN_ALIGN == 0);

        // SAFETY: Memory region starting at `chunk_ptr` with `CHUNK_SIZE` bytes is within
        // the allocation we just made.
        // `chunk_ptr` has high alignment (4 GiB). `CHUNK_SIZE` is large and a multiple of 16.
        let allocator = unsafe { Allocator::from_raw_parts(chunk_ptr, CHUNK_SIZE) };

        // Store pointer to original allocation, so it can be used to deallocate in `drop`
        Self { allocator: ManuallyDrop::new(allocator), alloc_ptr }
    }

    /// Reset this [`FixedSizeAllocator`].
    fn reset(&mut self) {
        // Set cursor back to end
        self.allocator.reset();

        // Set data pointer back to start.
        // SAFETY: Fixed-size allocators have data pointer originally aligned on `BLOCK_ALIGN`,
        // and size less than `BLOCK_ALIGN`. So we can restore original data pointer by rounding down
        // to next multiple of `BLOCK_ALIGN`.
        unsafe {
            let data_ptr = self.allocator.data_ptr();
            let offset = data_ptr.as_ptr() as usize % BLOCK_ALIGN;
            let data_ptr = data_ptr.sub(offset);
            self.allocator.set_data_ptr(data_ptr);
        }
    }
}

impl Drop for FixedSizeAllocator {
    fn drop(&mut self) {
        // SAFETY: Originally allocated from `System` allocator at `alloc_ptr`, with layout `ALLOC_LAYOUT`
        unsafe { System.dealloc(self.alloc_ptr.as_ptr(), ALLOC_LAYOUT) }
    }
}

// SAFETY: `Allocator` is `Send`.
// Moving `alloc_ptr: NonNull<u8>` across threads along with the `Allocator` is safe.
unsafe impl Send for FixedSizeAllocator {}
