//! [`Arena::new_fixed_size`]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::NonNull,
};

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::{Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE};

const TWO_GIB: usize = 1 << 31;
const FOUR_GIB: usize = 1 << 32;

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

/// Layout of backing allocations for fixed-size allocators.
const ALLOC_LAYOUT: Layout = match Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

const _: () = {
    // `ChunkFooter` lives in the last `CHUNK_FOOTER_SIZE` bytes of the block and must be aligned on `CHUNK_ALIGN` (16).
    // `BLOCK_SIZE` and `CHUNK_FOOTER_SIZE` are both multiples of `CHUNK_ALIGN`.
    assert!(BLOCK_SIZE > 0);
    assert!(BLOCK_SIZE.is_multiple_of(CHUNK_ALIGN));
    assert!(BLOCK_SIZE >= CHUNK_FOOTER_SIZE);
    assert!(CHUNK_FOOTER_SIZE.is_multiple_of(CHUNK_ALIGN));
};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Construct a static-sized [`Arena`] backed by an allocation made via the [`System`] allocator.
    ///
    /// The returned [`Arena`] uses a single chunk of `BLOCK_SIZE` bytes, aligned on `BLOCK_ALIGN`.
    /// It cannot grow.
    ///
    /// Allocation is made via [`System`] allocator, bypassing any registered alternative global allocator
    /// (e.g. Mimalloc in linter). Mimalloc complains that it cannot serve allocations with high alignment,
    /// and presumably it's pointless to try to obtain such large allocations from a thread-local heap,
    /// so better to go direct to the system allocator anyway.
    ///
    /// Returns `None` if the allocation fails.
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
        let offset = alloc_ptr.as_ptr() as usize % FOUR_GIB;
        // SAFETY: We allocated 4 GiB of memory, so adding `offset` to `alloc_ptr` is in bounds
        let chunk_ptr = unsafe { alloc_ptr.add(offset) };

        debug_assert!(chunk_ptr.addr().get().is_multiple_of(BLOCK_ALIGN));

        // The `Arena`'s chunk fills the whole `BLOCK_SIZE` block.
        // `ChunkFooter` sits in the last `CHUNK_FOOTER_SIZE` bytes of the block.
        //
        // SAFETY:
        // * Region starting at `chunk_ptr` with `BLOCK_SIZE` bytes is within the allocation we just made.
        // * `chunk_ptr` has high alignment (4 GiB).
        // * `BLOCK_SIZE` is large and a multiple of 16.
        // * `chunk_ptr` has permission for writes.
        let arena = unsafe { Self::from_raw_parts(chunk_ptr, BLOCK_SIZE, alloc_ptr, ALLOC_LAYOUT) };
        Some(arena)
    }
}
