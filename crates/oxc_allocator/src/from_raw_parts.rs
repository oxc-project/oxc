//! Define additional methods, used only by raw transfer:
//!
//! * [`Allocator::from_raw_parts`]
//! * [`Allocator::cursor_ptr`]
//! * [`Allocator::set_cursor_ptr`]
//! * [`Allocator::data_end_ptr`]
//! * [`Allocator::end_ptr`]

use std::ptr::NonNull;

use crate::{
    Allocator,
    arena::{Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE},
};

impl Allocator {
    /// Minimum size for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_SIZE: usize = CHUNK_FOOTER_SIZE;

    /// Minimum alignment for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_ALIGN: usize = CHUNK_ALIGN;

    /// Construct a static-sized [`Allocator`] from an existing memory allocation.
    ///
    /// The [`Allocator`] which is returned takes ownership of the memory allocation,
    /// and the allocation will be freed if the `Allocator` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Allocator` in `ManuallyDrop`.
    ///
    /// The [`Allocator`] returned by this function cannot grow.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be aligned on [`RAW_MIN_ALIGN`].
    /// * `size` must be a multiple of [`RAW_MIN_ALIGN`].
    /// * `size` must be at least [`RAW_MIN_SIZE`].
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within a single allocation.
    /// * The memory region starting at `ptr` and encompassing `size` bytes must have been allocated from system
    ///   allocator with alignment of [`RAW_MIN_ALIGN`] (or caller must wrap the `Allocator` in `ManuallyDrop`
    ///   and ensure the backing memory is freed correctly themselves).
    /// * `ptr` must have permission for writes.
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    /// [`RAW_MIN_SIZE`]: Self::RAW_MIN_SIZE
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        // SAFETY: Safety requirements of `Arena::from_raw_parts` are the same as for this method
        let arena = unsafe { Arena::from_raw_parts(ptr, size) };
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
    /// * `ptr` must be equal to or after data pointer for this chunk.
    /// * `ptr` must be equal to or before the chunk's `ChunkFooter`.
    /// * No live references to data in the current chunk before `ptr` can exist.
    pub unsafe fn set_cursor_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees `Allocator` has at least 1 allocated chunk, and `ptr` is valid.
        // The `Arena` contained in `Allocator` has `MIN_ALIGN = 1`, so no alignment requirement for `ptr`.
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
