//! Methods only available when `from_raw_parts` feature is enabled.
//! These methods are only used by raw transfer.

use std::{alloc::Layout, cell::Cell, ptr::NonNull};

use super::{Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE, ChunkFooter, EMPTY_CHUNK};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Construct a static-sized [`Arena`] from an existing memory allocation.
    ///
    /// The [`Arena`] which is returned takes ownership of the memory allocation,
    /// and the allocation will be freed if the `Arena` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Arena` in `ManuallyDrop`.
    ///
    /// The [`Arena`] returned by this function cannot grow.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be aligned on [`CHUNK_ALIGN`].
    /// * `size` must be a multiple of [`CHUNK_ALIGN`].
    /// * `size` must be at least [`CHUNK_FOOTER_SIZE`].
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within a single allocation.
    /// * The memory region starting at `ptr` and encompassing `size` bytes must have been allocated
    ///   from system allocator with alignment of [`CHUNK_ALIGN`] (or caller must wrap the `Arena` in `ManuallyDrop`
    ///   and ensure the backing memory is freed correctly themselves).
    /// * `ptr` must have permission for writes.
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        // Debug assert that `ptr` and `size` fulfill size and alignment requirements
        debug_assert!((ptr.as_ptr() as usize).is_multiple_of(CHUNK_ALIGN));
        debug_assert!(size.is_multiple_of(CHUNK_ALIGN));
        debug_assert!(size >= CHUNK_FOOTER_SIZE);

        let size_without_footer = size - CHUNK_FOOTER_SIZE;

        // Construct `ChunkFooter` and write into end of allocation.
        // SAFETY: Caller guarantees:
        // * `ptr` is the start of an allocation of `size` bytes.
        // * `size` is `>= CHUNK_FOOTER_SIZE` - so `size - CHUNK_FOOTER_SIZE` cannot wrap around.
        let chunk_footer_ptr = unsafe { ptr.add(size_without_footer) }.cast::<ChunkFooter>();
        // SAFETY: Caller guarantees `size` is a multiple of `CHUNK_ALIGN`.
        // Caller guarantees region from `ptr` to `ptr + size` forms a single allocation, so it must be a valid layout.
        let layout = unsafe { Layout::from_size_align_unchecked(size, CHUNK_ALIGN) };
        let chunk_footer = ChunkFooter {
            start_ptr: ptr,
            layout,
            previous_chunk_footer_ptr: Cell::new(EMPTY_CHUNK.get()),
            cursor_ptr: Cell::new(chunk_footer_ptr.cast::<u8>()),
        };

        // SAFETY: If caller has upheld safety requirements, `chunk_footer_ptr` is `CHUNK_FOOTER_SIZE` bytes
        // from the end of the allocation, and aligned on `CHUNK_ALIGN`.
        // Therefore `chunk_footer_ptr` is valid for writing a `ChunkFooter`.
        unsafe { chunk_footer_ptr.write(chunk_footer) };

        // Create `Arena` and set `can_grow` to `false`. This means that the memory chunk we've just created
        // will remain its only chunk. Therefore it can never be deallocated, until the `Arena` is dropped.
        // `Arena::reset` would only reset the "cursor" pointer, not deallocate the memory.
        let mut arena = Self::new_impl(chunk_footer_ptr);
        arena.can_grow = false;
        arena
    }

    /// Set cursor pointer for this [`Arena`]'s current chunk.
    ///
    /// This is dangerous, and this method should not ordinarily be used.
    /// It is only here for manually resetting the `Arena`.
    ///
    /// # SAFETY
    ///
    /// * `Arena` must have at least 1 allocated chunk.
    ///   It is UB to call this method on an `Arena` which has not allocated i.e. fresh from `Arena::new`.
    /// * `ptr` must point to within the `Arena`'s current chunk.
    /// * `ptr` must be equal to or after data pointer for this chunk.
    /// * `ptr` must be equal to or before the chunk's `ChunkFooter`.
    /// * `ptr` must be aligned to `MIN_ALIGN`.
    /// * No live references to data in the current chunk before `ptr` can exist.
    pub unsafe fn set_cursor_ptr(&self, ptr: NonNull<u8>) {
        debug_assert!(ptr.as_ptr() >= self.start_ptr.get().as_ptr());
        debug_assert!(ptr.as_ptr() <= self.current_chunk_footer.get().as_ptr().cast::<u8>());
        debug_assert!(ptr.addr().get().is_multiple_of(MIN_ALIGN));

        // SAFETY: Caller guarantees `Arena` has at least 1 allocated chunk, and `ptr` is valid
        #[expect(clippy::unnecessary_safety_comment)]
        self.cursor_ptr.set(ptr);
    }

    /// Get pointer to end of the data region of this [`Arena`]'s current chunk
    /// i.e to the start of the `ChunkFooter`.
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        self.current_chunk_footer.get().cast::<u8>()
    }

    /// Get pointer to end of this [`Arena`]'s current chunk (after the `ChunkFooter`).
    pub fn end_ptr(&self) -> NonNull<u8> {
        let chunk_footer_ptr = self.current_chunk_footer.get();

        // SAFETY: `chunk_footer_ptr` always points to a valid `ChunkFooter`, so stepping past it cannot be
        // out of bounds of the chunk's allocation. If `Arena` has not allocated, `chunk_footer_ptr`
        // returns a pointer to the static empty chunk, it's still valid.
        unsafe { chunk_footer_ptr.add(1).cast::<u8>() }
    }
}
