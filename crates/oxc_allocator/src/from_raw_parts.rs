//! Define additional methods, used only by raw transfer:
//!
//! * [`Allocator::from_raw_parts`]
//! * [`Allocator::cursor_ptr`]
//! * [`Allocator::set_cursor_ptr`]
//! * [`Allocator::data_end_ptr`]
//! * [`Allocator::end_ptr`]

use std::{alloc::Layout, ptr::NonNull};

use crate::{
    Allocator,
    arena::{Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE},
};

impl Allocator {
    /// Minimum size for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_SIZE: usize = CHUNK_FOOTER_SIZE;

    /// Minimum alignment for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_ALIGN: usize = CHUNK_ALIGN;

    /// Construct a static-sized [`Allocator`] from a region within an existing memory allocation.
    ///
    /// `start_ptr` and `size` describe the region the `Allocator` will use as its single chunk.
    ///
    /// `backing_alloc_ptr` and `layout` describe the underlying allocation that owns this region.
    /// They are stored in the `ChunkFooter` and used by the `Allocator`'s [`Drop`] implementation to free the allocation.
    /// The chunk region (`start_ptr..start_ptr + size`) must lie entirely within the allocation described by
    /// `backing_alloc_ptr` and `layout`.
    ///
    /// The [`Allocator`] which is returned takes ownership of the backing allocation, and it will be freed
    /// via the global allocator (using `backing_alloc_ptr` and `layout`) when the `Allocator` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Allocator` in `ManuallyDrop`.
    ///
    /// The [`Allocator`] returned by this function cannot grow.
    ///
    /// # SAFETY
    ///
    /// * `start_ptr` must be aligned on [`RAW_MIN_ALIGN`].
    /// * `size` must be a multiple of [`RAW_MIN_ALIGN`].
    /// * `size` must be at least [`RAW_MIN_SIZE`].
    /// * The memory region starting at `start_ptr` and encompassing `size` bytes must be entirely within
    ///   the allocation described by `backing_alloc_ptr` and `layout`
    ///   (i.e. `start_ptr >= backing_alloc_ptr` and `start_ptr + size <= backing_alloc_ptr + layout.size()`).
    /// * The allocation described by `backing_alloc_ptr` and `layout` must have been allocated from
    ///   the global allocator with that same `layout` (or caller must wrap the `Allocator` in `ManuallyDrop`
    ///   and ensure the backing memory is freed correctly themselves).
    /// * `start_ptr` and `backing_alloc_ptr` must have permission for writes.
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    /// [`RAW_MIN_SIZE`]: Self::RAW_MIN_SIZE
    pub unsafe fn from_raw_parts(
        start_ptr: NonNull<u8>,
        size: usize,
        backing_alloc_ptr: NonNull<u8>,
        layout: Layout,
    ) -> Self {
        // SAFETY: Safety requirements of `Arena::from_raw_parts` are the same as for this method
        let arena = unsafe { Arena::from_raw_parts(start_ptr, size, backing_alloc_ptr, layout) };
        Self::from_arena(arena)
    }

    /// Get the current cursor pointer for this [`Allocator`]'s current chunk.
    ///
    /// If the `Allocator` is empty (has no chunks), this returns a dangling pointer.
    pub fn cursor_ptr(&self) -> NonNull<u8> {
        self.arena().cursor_ptr()
    }

    /// Set cursor pointer for this [`Allocator`]'s current chunk.
    ///
    /// This is dangerous, and this method should not ordinarily be used.
    /// It is only here for manually resetting the allocator.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    ///   It is UB to call this method on an `Allocator` which has not allocated
    ///   i.e. fresh from `Allocator::new`.
    /// * `ptr` must point to within the `Allocator`'s current chunk.
    /// * `ptr` must be equal to or after start pointer for this chunk.
    /// * `ptr` must be equal to or before the chunk's `ChunkFooter`.
    /// * `ptr` must be aligned to the inner `Arena`'s `MIN_ALIGN`.
    /// * `ptr` must have permission for writes.
    /// * No live references to data in the current chunk before `ptr` can exist.
    pub unsafe fn set_cursor_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees `Allocator` has at least 1 allocated chunk, and `ptr` is valid
        unsafe { self.arena().set_cursor_ptr(ptr) };
    }

    /// Get pointer to end of the data region of this [`Allocator`]'s current chunk
    /// i.e to the start of the `ChunkFooter`.
    ///
    /// If the `Allocator` is empty (has no chunks), this returns a dangling pointer.
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        self.arena().data_end_ptr()
    }

    /// Get pointer to end of this [`Allocator`]'s current chunk (after the `ChunkFooter`).
    ///
    /// If the `Allocator` is empty (has no chunks), this returns a dangling pointer.
    pub fn end_ptr(&self) -> NonNull<u8> {
        self.arena().end_ptr()
    }
}
