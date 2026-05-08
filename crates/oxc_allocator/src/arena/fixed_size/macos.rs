use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::NonNull,
};

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::super::Arena;

// System allocator on Mac OS refuses allocations with 4 GiB alignment, so we over-allocate
// `BLOCK_SIZE + TWO_GIB` (4 GiB - 16) bytes with 2 GiB alignment, and then use either the 1st or 2nd half
// of the allocation, one of which is guaranteed to be on a 4 GiB boundary.
// <https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/unix.rs#L16-L27>
// <https://github.com/rust-lang/rust/issues/30170>

const TWO_GIB: usize = 1 << 31;
const FOUR_GIB: usize = 1 << 32;

const ALLOC_SIZE: usize = BLOCK_SIZE + TWO_GIB;
const ALLOC_ALIGN: usize = TWO_GIB;

const _: () = {
    // The over-alloc trick relies on the chunk fitting in either half of the allocation
    assert!(BLOCK_SIZE <= TWO_GIB);
    assert!(BLOCK_ALIGN == FOUR_GIB);
};

/// Layout of backing allocations.
const ALLOC_LAYOUT: Layout = match Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

const _: () = assert!(ALLOC_LAYOUT.size() > 0);

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Construct a static-sized [`Arena`] backed by an allocation made via the [`System`] allocator.
    ///
    /// The returned [`Arena`] uses a single chunk of `BLOCK_SIZE` bytes, aligned on `BLOCK_ALIGN`.
    /// It cannot grow.
    ///
    /// Returns `None` if the allocation fails.
    ///
    /// See module-level docs for the rationale and platform-specific allocation strategy.
    pub fn new_fixed_size() -> Option<Self> {
        // Allocate block of memory.
        // SAFETY: `ALLOC_LAYOUT` does not have zero size.
        let alloc_ptr = unsafe { System.alloc(ALLOC_LAYOUT) };
        let alloc_ptr = NonNull::new(alloc_ptr)?;

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
        let offset = alloc_ptr.addr().get() % FOUR_GIB;
        // SAFETY: We allocated 4 GiB of memory, so adding `offset` to `alloc_ptr` is in bounds
        let chunk_ptr = unsafe { alloc_ptr.add(offset) };

        debug_assert!(chunk_ptr.addr().get().is_multiple_of(BLOCK_ALIGN));

        // SAFETY:
        // * Region starting at `chunk_ptr` with `BLOCK_SIZE` bytes is within the allocation we just made.
        // * `chunk_ptr` has high alignment (4 GiB).
        // * `BLOCK_SIZE` is large and a multiple of 16.
        // * `chunk_ptr` and `alloc_ptr` have permission for writes.
        let arena = unsafe { Self::from_raw_parts(chunk_ptr, BLOCK_SIZE, alloc_ptr, ALLOC_LAYOUT) };

        Some(arena)
    }
}
