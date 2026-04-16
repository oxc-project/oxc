//! `Drop` implementation for `Arena`, and `reset` method.

use std::{alloc, ptr::NonNull};

use super::{Arena, ChunkFooter, EMPTY_CHUNK, utils::is_pointer_aligned_to};

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
            if self.current_chunk_footer.get().as_ref().is_empty() {
                return;
            }

            let cur_chunk = self.current_chunk_footer.get();

            // Deallocate all chunks except the current one
            let prev_chunk =
                cur_chunk.as_ref().previous_chunk_footer_ptr.replace(EMPTY_CHUNK.get());
            dealloc_chunk_list(prev_chunk);

            // Reset the bump cursor to the end of the chunk.
            // We don't need to reset `cursor_ptr` in `ChunkFooter`, as it'll be set if the chunk is retired later on.
            // `iter_allocated_chunks_raw` ignores `cursor_ptr` of the current chunk.
            debug_assert!(
                is_pointer_aligned_to(cur_chunk.as_ptr(), MIN_ALIGN),
                "bump pointer {cur_chunk:#p} should be aligned to the minimum alignment of {MIN_ALIGN:#x}"
            );
            self.cursor_ptr.set(cur_chunk.cast::<u8>());

            let current_chunk_footer = self.current_chunk_footer.get().as_ref();
            debug_assert!(
                current_chunk_footer.previous_chunk_footer_ptr.get().as_ref().is_empty(),
                "We should only have a single chunk"
            );
            debug_assert_eq!(
                self.cursor_ptr.get(),
                self.current_chunk_footer.get().cast::<u8>(),
                "Our chunk's bump cursor should be reset to the start of its allocation"
            );
        }
    }
}

/// `Drop` implementation for `Arena`.
impl<const MIN_ALIGN: usize> Drop for Arena<MIN_ALIGN> {
    fn drop(&mut self) {
        unsafe {
            dealloc_chunk_list(self.current_chunk_footer.get());
        }
    }
}

#[inline]
unsafe fn dealloc_chunk_list(mut footer: NonNull<ChunkFooter>) {
    unsafe {
        while !footer.as_ref().is_empty() {
            let f = footer;
            footer = f.as_ref().previous_chunk_footer_ptr.get();
            alloc::dealloc(f.as_ref().start_ptr.as_ptr(), f.as_ref().layout);
        }
    }
}
