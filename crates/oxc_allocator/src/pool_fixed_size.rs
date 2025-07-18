use std::{
    alloc::{self, GlobalAlloc, Layout, System},
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use oxc_ast_macros::ast;

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
    /// Allocators in the pool
    allocators: Mutex<Vec<FixedSizeAllocator>>,
    /// ID to assign to next `Allocator` that's created
    next_id: AtomicU32,
}

impl AllocatorPool {
    /// Creates a new [`AllocatorPool`] with capacity for the given number of `FixedSizeAllocator` instances.
    pub fn new(size: usize) -> AllocatorPool {
        // Each allocator consumes a large block of memory, so create them on demand instead of upfront
        let allocators = Vec::with_capacity(size);
        AllocatorPool { allocators: Mutex::new(allocators), next_id: AtomicU32::new(0) }
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

        let allocator = allocator.unwrap_or_else(|| {
            // Each allocator needs to have a unique ID, but the order those IDs are assigned in
            // doesn't matter, so `Ordering::Relaxed` is fine
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            // Protect against IDs wrapping around.
            // TODO: Does this work? Do we need it anyway?
            assert!(id < u32::MAX, "Created too many allocators");
            FixedSizeAllocator::new(id)
        });

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

/// Metadata about a [`FixedSizeAllocator`].
///
/// Is stored in the memory backing the [`FixedSizeAllocator`], after `RawTransferMetadata`,
/// which is after the section of the allocation which [`Allocator`] uses for its chunk.
#[ast]
pub struct FixedSizeAllocatorMetadata {
    /// ID of this allocator
    pub id: u32,
    /// Pointer to start of original allocation backing the `FixedSizeAllocator`
    pub alloc_ptr: NonNull<u8>,
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

/// Structure which wraps an [`Allocator`] with fixed size of 2 GiB - 16, and aligned on 4 GiB.
///
/// # Allocation strategy
///
/// To achieve this, we manually allocate memory to back the `Allocator`'s single chunk,
/// and to store other metadata.
///
/// We over-allocate 4 GiB, and then use only half of that allocation - either the 1st half,
/// or the 2nd half, depending on the alignment of the allocation received from `alloc.alloc()`.
/// One of those halves will be aligned on 4 GiB, and that's the one we use.
///
/// Inner `Allocator` is wrapped in `ManuallyDrop` to prevent it freeing the memory itself,
/// and `FixedSizeAllocator` has a custom `Drop` impl which frees the whole of the original allocation.
///
/// We allocate via `System` allocator, bypassing any registered alternative global allocator
/// (e.g. Mimalloc in linter). Mimalloc complains that it cannot serve allocations with high alignment,
/// and presumably it's pointless to try to obtain such large allocations from a thread-local heap,
/// so better to go direct to the system allocator anyway.
///
/// # Regions of the allocated memory
///
/// 2 GiB of the allocated memory is not used at all (see above).
///
/// The remaining 2 GiB - 16 bytes, which *is* used, is split up as follows:
///
/// ```txt
///                                                         WHOLE BLOCK - aligned on 4 GiB
/// <-----------------------------------------------------> Allocated block (`BLOCK_SIZE` bytes)
///
///                                                         ALLOCATOR
/// <----------------------------------------->             `Allocator` chunk (`CHUNK_SIZE` bytes)
///                                      <---->             Bumpalo's `ChunkFooter` (aligned on 16)
/// <----------------------------------->                   `Allocator` chunk data storage (for AST)
///
///                                                         METADATA
///                                            <---->       `RawTransferMetadata`
///                                                  <----> `FixedSizeAllocatorMetadata`
///
///                                                         BUFFER SENT TO JS
/// <----------------------------------------------->       Buffer sent to JS (`BUFFER_SIZE` bytes)
/// ```
///
/// Note that the buffer sent to JS includes both the `Allocator` chunk, and `RawTransferMetadata`,
/// but does NOT include `FixedSizeAllocatorMetadata`.
///
/// The end of the region used for `Allocator` chunk must be aligned on `Allocator::RAW_MIN_ALIGN` (16),
/// due to the requirements of Bumpalo. We manage that by:
/// * `BLOCK_SIZE` is a multiple of 16.
/// * `RawTransferMetadata` is 16 bytes.
/// * Size of `FixedSizeAllocatorMetadata` is rounded up to a multiple of 16.
pub struct FixedSizeAllocator {
    /// `Allocator` which utilizes part of the original allocation
    allocator: ManuallyDrop<Allocator>,
}

impl FixedSizeAllocator {
    /// Create a new [`FixedSizeAllocator`].
    #[expect(clippy::items_after_statements)]
    pub fn new(id: u32) -> Self {
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

        const FIXED_METADATA_SIZE_ROUNDED: usize =
            size_of::<FixedSizeAllocatorMetadata>().next_multiple_of(Allocator::RAW_MIN_ALIGN);
        const FIXED_METADATA_OFFSET: usize = BLOCK_SIZE - FIXED_METADATA_SIZE_ROUNDED;
        const _: () =
            assert!(FIXED_METADATA_OFFSET % align_of::<FixedSizeAllocatorMetadata>() == 0);

        const CHUNK_SIZE: usize = FIXED_METADATA_OFFSET - RAW_METADATA_SIZE;
        const _: () = assert!(CHUNK_SIZE % Allocator::RAW_MIN_ALIGN == 0);

        // SAFETY: Memory region starting at `chunk_ptr` with `CHUNK_SIZE` bytes is within
        // the allocation we just made.
        // `chunk_ptr` has high alignment (4 GiB). `CHUNK_SIZE` is large and a multiple of 16.
        let allocator = unsafe { Allocator::from_raw_parts(chunk_ptr, CHUNK_SIZE) };

        // Write `FixedSizeAllocatorMetadata` to after space reserved for `RawTransferMetadata`,
        // which is after the end of the allocator chunk
        let metadata = FixedSizeAllocatorMetadata { alloc_ptr, id };
        // SAFETY: `FIXED_METADATA_OFFSET` is `FIXED_METADATA_SIZE_ROUNDED` bytes before end of
        // the allocation, so there's space for `FixedSizeAllocatorMetadata`.
        // It's sufficiently aligned for `FixedSizeAllocatorMetadata`.
        unsafe {
            let metadata_ptr =
                chunk_ptr.add(FIXED_METADATA_OFFSET).cast::<FixedSizeAllocatorMetadata>();
            metadata_ptr.write(metadata);
        }

        Self { allocator: ManuallyDrop::new(allocator) }
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
        // Get pointer to start of original allocation from `FixedSizeAllocatorMetadata`
        let alloc_ptr = {
            // SAFETY: This `Allocator` was created by this `FixedSizeAllocator`.
            // `&FixedSizeAllocatorMetadata` ref only lives until end of this block.
            let metadata = unsafe { self.allocator.fixed_size_metadata() };
            metadata.alloc_ptr
        };

        // SAFETY: Originally allocated from `System` allocator at `alloc_ptr`, with layout `ALLOC_LAYOUT`
        unsafe { System.dealloc(alloc_ptr.as_ptr(), ALLOC_LAYOUT) }
    }
}

// SAFETY: `Allocator` is `Send`.
// Moving `alloc_ptr: NonNull<u8>` across threads along with the `Allocator` is safe.
unsafe impl Send for FixedSizeAllocator {}

impl Allocator {
    /// Get reference to the [`FixedSizeAllocatorMetadata`] for this [`Allocator`].
    ///
    /// # SAFETY
    /// * This `Allocator` must have been created by a `FixedSizeAllocator`.
    /// * Reference returned by this method must not be allowed to live beyond the life of
    ///   the `FixedSizeAllocator`.
    unsafe fn fixed_size_metadata(&self) -> &FixedSizeAllocatorMetadata {
        // SAFETY: Caller guarantees this `Allocator` was created by a `FixedSizeAllocator`.
        //
        // `FixedSizeAllocator::new` writes `FixedSizeAllocatorMetadata` after the end of
        // the chunk owned by the `Allocator`, and `RawTransferMetadata` (see above).
        // `end_ptr` is end of the allocator chunk (after the chunk header).
        // So `end_ptr + RAW_METADATA_SIZE` points to a valid, initialized `FixedSizeAllocatorMetadata`.
        //
        // We never create `&mut` references to `FixedSizeAllocatorMetadata`,
        // and it's not part of the buffer sent to JS, so no danger of aliasing violations.
        unsafe {
            let metadata_ptr =
                self.end_ptr().add(RAW_METADATA_SIZE).cast::<FixedSizeAllocatorMetadata>();
            metadata_ptr.as_ref()
        }
    }
}
