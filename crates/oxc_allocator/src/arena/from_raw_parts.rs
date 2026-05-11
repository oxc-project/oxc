//! Methods only available when `from_raw_parts` feature is enabled.
//! These methods are only used by raw transfer.

use std::{alloc::Layout, cell::Cell, ptr::NonNull};

use super::{
    Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE, ChunkFooter, EMPTY_ARENA_DATA_PTR,
    utils::is_pointer_aligned_to,
};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Construct a static-sized [`Arena`] from a region within an existing memory allocation.
    ///
    /// `start_ptr` and `size` describe the region the `Arena` will use as its single chunk.
    ///
    /// `backing_alloc_ptr` and `layout` describe the underlying allocation that owns this region.
    /// They are stored in the `ChunkFooter` and used by the `Arena`'s [`Drop`] implementation to free the allocation.
    /// The chunk region (`start_ptr..start_ptr + size`) must lie entirely within the allocation described by
    /// `backing_alloc_ptr` and `layout`.
    ///
    /// The [`Arena`] which is returned takes ownership of the backing allocation, and it will be freed
    /// via the [`System`] allocator (using `backing_alloc_ptr` and `layout`) when the `Arena` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Arena` in `ManuallyDrop`.
    ///
    /// The [`Arena`] returned by this function cannot grow.
    ///
    /// # SAFETY
    ///
    /// * `start_ptr` must be aligned on [`CHUNK_ALIGN`].
    /// * `size` must be a multiple of [`CHUNK_ALIGN`].
    /// * `size` must be at least [`CHUNK_FOOTER_SIZE`].
    /// * The memory region starting at `start_ptr` and encompassing `size` bytes must be entirely within
    ///   the allocation described by `backing_alloc_ptr` and `layout`
    ///   (i.e. `start_ptr >= backing_alloc_ptr` and `start_ptr + size <= backing_alloc_ptr + layout.size()`).
    /// * The allocation described by `backing_alloc_ptr` and `layout` must have been allocated from
    ///   the [`System`] allocator with that same `layout` (or caller must wrap the `Arena` in `ManuallyDrop`
    ///   and ensure the backing memory is freed correctly themselves).
    /// * `start_ptr` and `backing_alloc_ptr` must have permission for writes.
    ///
    /// [`System`]: std::alloc::System
    pub unsafe fn from_raw_parts(
        start_ptr: NonNull<u8>,
        size: usize,
        backing_alloc_ptr: NonNull<u8>,
        layout: Layout,
    ) -> Self {
        // Debug assert that `start_ptr` and `size` fulfill size and alignment requirements
        debug_assert!(is_pointer_aligned_to(start_ptr, CHUNK_ALIGN));
        debug_assert!(size.is_multiple_of(CHUNK_ALIGN));
        debug_assert!(size >= CHUNK_FOOTER_SIZE);
        debug_assert!(start_ptr.addr().get().checked_add(size).is_some());

        // Debug assert that the chunk region lies within the backing allocation
        debug_assert!(start_ptr >= backing_alloc_ptr);
        debug_assert!(backing_alloc_ptr.addr().get().checked_add(layout.size()).is_some());
        debug_assert!(
            start_ptr.addr().get() + size <= backing_alloc_ptr.addr().get() + layout.size()
        );

        let size_without_footer = size - CHUNK_FOOTER_SIZE;

        // Construct `ChunkFooter` and write into end of chunk region.
        // SAFETY: Caller guarantees:
        // * `start_ptr` is the start of a region of `size` bytes within the backing allocation.
        // * `size` is `>= CHUNK_FOOTER_SIZE` - so `size - CHUNK_FOOTER_SIZE` cannot wrap around.
        let chunk_footer_ptr = unsafe { start_ptr.add(size_without_footer) }.cast::<ChunkFooter>();

        // Initial cursor sits at the footer, which is the end of the allocatable region.
        // The footer is aligned on `CHUNK_ALIGN`, which is `>= MIN_ALIGN`, so this is already aligned to `MIN_ALIGN`.
        //
        // Set `is_fixed_size` to `true` so `Drop` impl for `Arena` will free the backing memory via `System` allocator.
        // `is_fixed_size: true` also prevents the `Arena` from ever getting any further chunks
        // (enforced in allocation slow path `try_alloc_layout_slow_impl`).
        // `Arena::reset` only resets the cursor pointer, and doesn't deallocate the chunk,
        // so `is_fixed_size` will remain permanently `true` for this `Arena`, even across `reset` calls.
        let cursor_ptr = chunk_footer_ptr.cast::<u8>();
        let chunk_footer = ChunkFooter {
            backing_alloc_ptr,
            layout,
            previous_chunk_footer_ptr: Cell::new(None),
            cursor_ptr: Cell::new(cursor_ptr),
            is_fixed_size: true,
        };
        // SAFETY: If caller has upheld safety requirements, `chunk_footer_ptr` is `CHUNK_FOOTER_SIZE` bytes
        // from the end of the chunk region, and aligned on `CHUNK_ALIGN`.
        // Therefore `chunk_footer_ptr` is valid for writing a `ChunkFooter`.
        unsafe { chunk_footer_ptr.write(chunk_footer) };

        // Create `Arena`
        Self::new_impl(start_ptr, cursor_ptr, Some(chunk_footer_ptr))
    }

    /// Get the current cursor pointer for this [`Arena`]'s current chunk.
    ///
    /// If the `Arena` is empty (has no chunks), this returns a dangling pointer aligned to `CHUNK_ALIGN`.
    pub fn cursor_ptr(&self) -> NonNull<u8> {
        self.cursor_ptr.get()
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
    /// * `cursor_ptr` must point to within the `Arena`'s current chunk.
    /// * `cursor_ptr` must be equal to or after start pointer for this chunk.
    /// * `cursor_ptr` must be equal to or before the chunk's `ChunkFooter`.
    /// * `cursor_ptr` must be aligned to `MIN_ALIGN`.
    /// * `cursor_ptr` must have permission for writes.
    /// * No live references to data in the current chunk before `cursor_ptr` can exist.
    pub unsafe fn set_cursor_ptr(&self, cursor_ptr: NonNull<u8>) {
        debug_assert!(cursor_ptr >= self.start_ptr.get());
        debug_assert!(
            self.current_chunk_footer_ptr
                .get()
                .is_some_and(|footer_ptr| cursor_ptr <= footer_ptr.cast::<u8>())
        );
        debug_assert!(is_pointer_aligned_to(cursor_ptr, MIN_ALIGN));

        // SAFETY: Caller guarantees `Arena` has at least 1 allocated chunk, and `cursor_ptr` is valid
        #[expect(clippy::unnecessary_safety_comment)]
        self.cursor_ptr.set(cursor_ptr);
    }

    /// Get pointer to end of the data region of this [`Arena`]'s current chunk
    /// i.e to the start of the `ChunkFooter`.
    ///
    /// If `Arena` has not allocated any chunks, returns a dangling pointer aligned to `CHUNK_ALIGN`.
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        match self.current_chunk_footer_ptr.get() {
            Some(chunk_footer_ptr) => chunk_footer_ptr.cast::<u8>(),
            None => EMPTY_ARENA_DATA_PTR,
        }
    }

    /// Get pointer to end of this [`Arena`]'s current chunk (after the `ChunkFooter`).
    ///
    /// If `Arena` has not allocated any chunks, returns a dangling pointer aligned to `CHUNK_ALIGN`.
    pub fn end_ptr(&self) -> NonNull<u8> {
        match self.current_chunk_footer_ptr.get() {
            // SAFETY: `chunk_footer_ptr` always points to a valid `ChunkFooter`, so stepping past it
            // cannot be out of bounds of the chunk's allocation.
            Some(chunk_footer_ptr) => unsafe { chunk_footer_ptr.add(1).cast::<u8>() },
            None => EMPTY_ARENA_DATA_PTR,
        }
    }
}
