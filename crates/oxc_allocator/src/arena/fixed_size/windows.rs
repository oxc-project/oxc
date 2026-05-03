use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::NonNull,
};

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::super::Arena;

// Windows system allocator doesn't support high alignment allocations directly.
//
// Rust's `std` implements a workaround which over-allocates `size + align` bytes, and stores the real pointer
// in a hidden slot before the returned allocation, likely committing an extra page just for the 8-byte pointer.
// <https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/windows.rs#L120-L137>
//
// That's unnecessary for our use case, because we already store the real allocation pointer in the `ChunkFooter`.
// So we side-step `std`'s workaround by requesting a low-alignment allocation that's large enough to find
// a `BLOCK_ALIGN`-aligned chunk of `BLOCK_SIZE` bytes inside it, and we align the chunk pointer to
// `BLOCK_ALIGN` ourselves.

const ALLOC_SIZE: usize = BLOCK_SIZE + BLOCK_ALIGN;
const ALLOC_ALIGN: usize = 16;

// For rounding up to `BLOCK_ALIGN` to make sense, `BLOCK_ALIGN` must be a multiple of `ALLOC_ALIGN`
const _: () = assert!(BLOCK_ALIGN.is_multiple_of(ALLOC_ALIGN));

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
    ///
    /// [`System`]: std::alloc::System
    pub fn new_fixed_size() -> Option<Self> {
        // Allocate block of memory with low alignment, large enough to find a `BLOCK_ALIGN`-aligned chunk inside.
        // SAFETY: `ALLOC_LAYOUT` does not have zero size.
        let alloc_ptr = unsafe { System.alloc(ALLOC_LAYOUT) };
        let alloc_ptr = NonNull::new(alloc_ptr)?;

        // Get pointer to use for allocator chunk, by rounding `alloc_ptr` up to the next `BLOCK_ALIGN` boundary.
        //
        // `offset` is the number of bytes between `alloc_ptr` and that next-aligned address:
        // `(BLOCK_ALIGN - (alloc_addr % BLOCK_ALIGN)) % BLOCK_ALIGN`
        //
        // The final `% BLOCK_ALIGN` is needed because if `alloc_addr` is already aligned,
        // `BLOCK_ALIGN - (alloc_addr % BLOCK_ALIGN)` = `BLOCK_ALIGN - 0` = `BLOCK_ALIGN`. But we want `0`.
        // (actually it doesn't matter, `BLOCK_ALIGN` would also be valid, but 0 is neater.)
        //
        // The expression below collapses both `%`s into a single masked unsigned negation.
        // `wrapping_neg()` computes `-alloc_addr (mod 2^N)` (where `N = usize::BITS`),
        // so taking it `mod BLOCK_ALIGN` gives `-alloc_addr (mod BLOCK_ALIGN)` which is exactly the formula above.
        // (Compiler reduces `% BLOCK_ALIGN` to `& (BLOCK_ALIGN - 1)` because `BLOCK_ALIGN` is a power-of-2 const.)
        //
        // `offset` is in `0..BLOCK_ALIGN`, so `chunk_ptr + BLOCK_SIZE` is within the allocation
        // (we allocated `BLOCK_SIZE + BLOCK_ALIGN` bytes).
        let alloc_addr = alloc_ptr.addr().get();
        let offset = alloc_addr.wrapping_neg() % BLOCK_ALIGN;
        // SAFETY: `offset < BLOCK_ALIGN` and we allocated `BLOCK_SIZE + BLOCK_ALIGN` bytes,
        // so `alloc_ptr + offset` is in bounds
        let chunk_ptr = unsafe { alloc_ptr.add(offset) };

        debug_assert!(chunk_ptr.addr().get().is_multiple_of(BLOCK_ALIGN));

        // SAFETY:
        // * Region starting at `chunk_ptr` with `BLOCK_SIZE` bytes is within the allocation we just made.
        // * `chunk_ptr` has high alignment (`BLOCK_ALIGN`).
        // * `BLOCK_SIZE` is large and a multiple of 16.
        // * `chunk_ptr` and `alloc_ptr` have permission for writes.
        let arena = unsafe { Self::from_raw_parts(chunk_ptr, BLOCK_SIZE, alloc_ptr, ALLOC_LAYOUT) };

        Some(arena)
    }
}
