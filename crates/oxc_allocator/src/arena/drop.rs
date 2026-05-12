//! `Drop` implementation for `Arena`, and `reset` method.

use std::{
    alloc::{self, GlobalAlloc, System},
    ptr::NonNull,
};

use super::{Arena, ChunkFooter, utils::is_pointer_aligned_to};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Reset this arena.
    ///
    /// Performs mass deallocation on everything allocated in this arena by resetting the pointer into
    /// the underlying chunk of memory to the start of the chunk. Does not run any `Drop` implementations
    /// on deallocated objects. See [the top-level documentation] for details.
    ///
    /// If this arena has allocated multiple chunks to allocate into, then the excess chunks are returned to
    /// the global allocator.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let mut arena = Arena::new();
    ///
    /// // Allocate a bunch of things
    /// {
    ///     for i in 0..100 {
    ///         arena.alloc(i);
    ///     }
    /// }
    ///
    /// // Reset the arena
    /// arena.reset();
    ///
    /// // Allocate some new things in the space
    /// // previously occupied by the original things
    /// for j in 200..400 {
    ///     arena.alloc(j);
    /// }
    /// ```
    ///
    /// [the top-level documentation]: Arena
    pub fn reset(&mut self) {
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.reset();

        // Takes `&mut self` so `self` must be unique, and there can't be any borrows active
        // that would get invalidated by resetting
        unsafe {
            let Some(current_footer_ptr) = self.current_chunk_footer_ptr.get() else {
                return;
            };

            // Deallocate all chunks except the current one
            let prev_footer_ptr =
                current_footer_ptr.as_ref().previous_chunk_footer_ptr.replace(None);
            dealloc_chunk_list(prev_footer_ptr);

            // Reset the bump cursor to the end of the chunk.
            // We don't need to reset `cursor_ptr` in `ChunkFooter`, as it'll be set if the chunk is retired later on.
            // `iter_allocated_chunks_raw` ignores `cursor_ptr` of the current chunk.
            debug_assert!(
                is_pointer_aligned_to(current_footer_ptr, MIN_ALIGN),
                "bump pointer {current_footer_ptr:#p} should be aligned to the minimum alignment of {MIN_ALIGN:#x}"
            );
            self.cursor_ptr.set(current_footer_ptr.cast::<u8>());

            debug_assert!(
                current_footer_ptr.as_ref().previous_chunk_footer_ptr.get().is_none(),
                "We should only have a single chunk"
            );
            debug_assert_eq!(
                self.cursor_ptr.get(),
                current_footer_ptr.cast::<u8>(),
                "Our chunk's bump cursor should be reset to the start of its allocation"
            );
        }
    }
}

/// `Drop` implementation for `Arena`.
impl<const MIN_ALIGN: usize> Drop for Arena<MIN_ALIGN> {
    fn drop(&mut self) {
        unsafe {
            dealloc_chunk_list(self.current_chunk_footer_ptr.get());
        }
    }
}

/// Deallocate all chunks in linked list, starting with the chunk whose footer is pointed to by `footer_ptr`.
///
/// # SAFETY
/// `footer_ptr` must point to a valid `ChunkFooter`.
#[inline]
unsafe fn dealloc_chunk_list(footer_ptr: Option<NonNull<ChunkFooter>>) {
    let mut next_footer_ptr = footer_ptr;

    while let Some(footer_ptr) = next_footer_ptr {
        // Create `&ChunkFooter` reference within a block, to ensure the reference is not live
        // when we deallocate the chunk's memory (which includes the `ChunkFooter`)
        {
            // SAFETY: `footer_ptr` always points to a valid `ChunkFooter`
            next_footer_ptr = unsafe { footer_ptr.as_ref() }.previous_chunk_footer_ptr.get();
        }

        // SAFETY: `footer_ptr` always points to a valid `ChunkFooter`
        unsafe { dealloc_arena_chunk(footer_ptr) };
    }
}

/// Deallocate the chunk whose footer is pointed to by `footer_ptr`.
///
/// # SAFETY
/// `footer_ptr` must point to a valid `ChunkFooter`.
pub unsafe fn dealloc_arena_chunk(footer_ptr: NonNull<ChunkFooter>) {
    // Create `&ChunkFooter` reference within a block, to ensure the reference is not live
    // when we deallocate the chunk's memory (which includes the `ChunkFooter`)
    let (backing_alloc_ptr, layout, is_fixed_size) = {
        // SAFETY: Caller guarantees that `footer_ptr` points to a valid `ChunkFooter`
        let footer = unsafe { footer_ptr.as_ref() };
        (footer.backing_alloc_ptr.as_ptr(), footer.layout, footer.is_fixed_size)
    };

    if is_fixed_size {
        // SAFETY: Each `ChunkFooter`'s `backing_alloc_ptr` and `layout` describe its backing allocation.
        // `is_fixed_size` is `true`, so backing allocation was made via `System` allocator.
        unsafe { System.dealloc(backing_alloc_ptr, layout) };
    } else {
        // SAFETY: Each `ChunkFooter`'s `backing_alloc_ptr` and `layout` describe its backing allocation.
        // `is_fixed_size` is `false`, so backing allocation was made via global allocator.
        unsafe { alloc::dealloc(backing_alloc_ptr, layout) };
    }
}
