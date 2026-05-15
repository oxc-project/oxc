use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::NonNull,
};

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::super::{Arena, ChunkFooter};

// System allocator on Mac OS refuses allocations with 4 GiB alignment, so we over-allocate
// `BLOCK_SIZE + TWO_GIB` (4 GiB - 16) bytes with 2 GiB alignment, and then use either the 1st or 2nd half
// of the allocation, one of which is guaranteed to be on a 4 GiB boundary.
// <https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/unix.rs#L16-L27>
// <https://github.com/rust-lang/rust/issues/30170>
//
// On Linux MUSL, allocation requests with 4 GiB alignment succeed, but then produce a segfault when the allocation
// is freed. So we use the same trick as on Mac OS - over-allocate with 2 GiB alignment.
// <https://github.com/oxc-project/oxc/issues/22339>
// <https://www.openwall.com/lists/musl/2026/05/12/>
//
// On Linux GLIBC, allocation requests with 4 GiB alignment work correctly, so we could request exactly what we need.
// But it makes little difference in practice - because all allocations are aligned on 4 GiB, the effective limit
// on number of fixed-size arenas is the same either way.
// For simplicity, we just use the same allocation strategy on all Linux variants.

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

    /// Attempt to grow the [`Arena`]'s current chunk in place to accommodate an allocation of `Layout`.
    ///
    /// If the chunk can be grown in place to accommodate the request:
    /// * Returns `Some(new_ptr)`, where `new_ptr` is the pointer to write the layout at.
    /// * Updates `start_ptr`.
    /// * Does NOT update `cursor_ptr` - that is left to the caller.
    ///
    /// If the chunk could not be grown in place to accommodate the request, returns `None`.
    ///
    /// On Linux and Mac OS, fixed size chunks cannot currently be grown in place, so always returns `None`.
    ///
    /// # SAFETY
    ///
    /// * `Arena` must be fixed-size (created via `Arena::new_fixed_size`).
    /// * Arena must not be able to accommodate an allocation of `layout` within current chunk, prior to growing it.
    #[expect(unused_variables)]
    #[cfg_attr(not(debug_assertions), expect(clippy::unused_self))]
    #[inline(always)] // Because it's a no-op
    pub(in super::super) unsafe fn grow_fixed_size_chunk(
        &self,
        layout: Layout,
    ) -> Option<NonNull<u8>> {
        #[cfg(debug_assertions)]
        {
            let footer_ptr = self.current_chunk_footer_ptr.get().expect("Arena has no chunks");
            // SAFETY: `footer_ptr` always points to a valid `ChunkFooter`
            let footer = unsafe { footer_ptr.as_ref() };
            assert!(
                footer.is_fixed_size,
                "Only fixed-size allocators should be passed to `Arena::grow_fixed_size_chunk`"
            );
        }

        None
    }
}

/// Deallocate the chunk whose footer is pointed to by `footer_ptr`, when the chunk is fixed size
/// (created via `Arena::from_raw_parts` or `Arena::new_fixed_size`).
///
/// `dealloc_chunk` in `drop` module delegates to this function when chunk's `is_fixed_size` flag is set.
/// `free_fixed_size_allocator` in `pool/fixed_size.rs` also uses this function for deallocation.
///
/// # SAFETY
///
/// * `footer_ptr` must point to a valid `ChunkFooter`.
/// * `ChunkFooter` must be for a fixed size chunk (created via `Arena::from_raw_parts` or `Arena::new_fixed_size`).
pub unsafe fn dealloc_fixed_size_arena_chunk(footer_ptr: NonNull<ChunkFooter>) {
    // Create `&ChunkFooter` reference within a block, to ensure the reference is not live
    // when we deallocate the chunk's memory (which includes the `ChunkFooter`)
    let (backing_alloc_ptr, layout, is_fixed_size) = {
        // SAFETY: Caller guarantees that `footer_ptr` points to a valid `ChunkFooter`
        let footer = unsafe { footer_ptr.as_ref() };
        (footer.backing_alloc_ptr, footer.layout, footer.is_fixed_size)
    };

    debug_assert!(
        is_fixed_size,
        "Only fixed-size allocators should be passed to `dealloc_fixed_size_arena_chunk` to deallocate"
    );

    // SAFETY: Each `ChunkFooter`'s `backing_alloc_ptr` and `layout` describe its backing allocation.
    // Caller guarantees `is_fixed_size` is `true`, so backing allocation was made via `System` allocator.
    unsafe { System.dealloc(backing_alloc_ptr.as_ptr(), layout) };
}
