use std::{
    alloc::{self, GlobalAlloc, Layout, System},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
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
// TODO: We could use this workaround only on Mac OS, and just allocate what we actually want on Linux.
// Windows OS allocator also doesn't support high alignment allocations, so Rust contains a workaround
// which over-allocates (6 GiB in this case).
// https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/windows.rs#L120-L137
// Could just use that built-in workaround, rather than implementing our own, or allocate a 6 GiB chunk
// with alignment 16, to skip Rust's built-in workaround.
// Note: Rust's workaround will likely commit a whole page of memory, just to store the real pointer.
const ALLOC_SIZE: usize = FOUR_GIB;
const ALLOC_ALIGN: usize = TWO_GIB;

const ALLOC_LAYOUT: Layout = match Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

const CHUNK_SIZE: usize = TWO_GIB;
pub const CHUNK_ALIGN: usize = FOUR_GIB;

const CHUNK_LAYOUT: Layout = match Layout::from_size_align(CHUNK_SIZE, CHUNK_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

/// Structure which wraps an [`Allocator`] with fixed size of 2 GiB, and aligned on 4 GiB.
///
/// To achieve this, we manually allocate memory to back the `Allocator`'s single chunk.
/// We over-allocate 4 GiB, and then use a part of that allocation to back the `Allocator`.
/// Inner `Allocator` is wrapped in `ManuallyDrop` to prevent it freeing the memory itself,
/// and `AllocatorWrapper` has a custom `Drop` impl which frees the whole of the original allocation.
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
    ///
    /// # Panics
    /// Panics if not a 64-bit little-endian platform.
    pub fn new() -> Self {
        assert!(
            IS_SUPPORTED_PLATFORM,
            "Fixed size allocators are only supported on 64-bit little-endian platforms"
        );

        // Allocate block of memory.
        // SAFETY: Layout does not have zero size.
        let alloc_ptr = unsafe { System.alloc(ALLOC_LAYOUT) };
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
        // `chunk_ptr` has high alignment (4 GiB).
        // `CHUNK_LAYOUT`'s `size` is large and a high power of 2 (2 GiB).
        // `CHUNK_LAYOUT`'s `align` is large (4 GiB).
        let allocator = unsafe { Allocator::from_raw_parts(chunk_ptr, CHUNK_LAYOUT) };

        // Store pointer to original allocation, so it can be used to deallocate in `drop`
        Self { allocator: ManuallyDrop::new(allocator), alloc_ptr }
    }
}

impl Drop for FixedSizeAllocator {
    fn drop(&mut self) {
        // SAFETY: Originally allocated from `System` allocator at `alloc_ptr`, with layout `ALLOC_LAYOUT`
        unsafe { System.dealloc(self.alloc_ptr.as_ptr(), ALLOC_LAYOUT) }
    }
}

impl Deref for FixedSizeAllocator {
    type Target = Allocator;

    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

impl DerefMut for FixedSizeAllocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.allocator
    }
}

// SAFETY: `Allocator` is `Send`.
// Moving `alloc_ptr: NonNull<u8>` across threads along with the `Allocator` is safe.
unsafe impl Send for FixedSizeAllocator {}
