use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::NonNull,
};

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::super::Arena;

// Linux's system allocator supports high alignment allocations directly, so we just request what we want.
// We assume that other non-MacOS, non-Windows platforms also support high alignment allocations.

/// Layout of backing allocations.
const ALLOC_LAYOUT: Layout = match Layout::from_size_align(BLOCK_SIZE, BLOCK_ALIGN) {
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
        // Allocate block of memory aligned on `BLOCK_ALIGN`.
        // SAFETY: `ALLOC_LAYOUT` does not have zero size.
        let alloc_ptr = unsafe { System.alloc(ALLOC_LAYOUT) };
        let alloc_ptr = NonNull::new(alloc_ptr)?;

        debug_assert!(alloc_ptr.addr().get().is_multiple_of(BLOCK_ALIGN));

        // SAFETY:
        // * Region starting at `alloc_ptr` with `BLOCK_SIZE` bytes is the allocation we just made.
        // * `alloc_ptr` has high alignment (`BLOCK_ALIGN`).
        // * `BLOCK_SIZE` is large and a multiple of 16.
        // * `alloc_ptr` has permission for writes.
        let arena = unsafe { Self::from_raw_parts(alloc_ptr, BLOCK_SIZE, alloc_ptr, ALLOC_LAYOUT) };

        Some(arena)
    }
}
