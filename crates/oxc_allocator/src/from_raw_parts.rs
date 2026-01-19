//! Define additional methods, used only by raw transfer:
//!
//! * [`Allocator::from_raw_parts`]
//! * [`Allocator::alloc_bytes_start`]
//! * [`Allocator::data_end_ptr`]
//! * [`Allocator::end_ptr`]

// All methods just delegate to methods of `Arena`
#![expect(clippy::inline_always)]

use std::{alloc::Layout, ptr::NonNull};

use crate::{
    Allocator,
    arena::{AllocatorArena as Arena, CHUNK_ALIGN, FOOTER_SIZE},
};

const _: () = assert!(FOOTER_SIZE >= CHUNK_ALIGN);

impl Allocator {
    /// Minimum size for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_SIZE: usize = FOOTER_SIZE;

    /// Minimum alignment for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_ALIGN: usize = CHUNK_ALIGN;

    /// Construct a static-sized [`Allocator`] from an existing memory allocation.
    ///
    /// The [`Allocator`] which is returned takes ownership of the memory allocation,
    /// and the allocation will be freed if the `Allocator` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Allocator` in `ManuallyDrop`.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be aligned on [`RAW_MIN_ALIGN`].
    /// * `layout.size()` must be a multiple of [`RAW_MIN_ALIGN`].
    /// * `layout.size()` must be at least [`RAW_MIN_SIZE`].
    /// * `layout.align()` must be at least [`RAW_MIN_ALIGN`].
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within
    ///   a single allocation.
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    /// [`RAW_MIN_SIZE`]: Self::RAW_MIN_SIZE
    #[inline(always)]
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, layout: Layout) -> Self {
        // SAFETY: Safety requirements of this method match `Arena::from_raw_parts`'s requirements
        let arena = unsafe { Arena::from_raw_parts(ptr, layout) };
        Self::from_arena(arena)
    }

    /// Allocate space for `bytes` bytes at start of [`Allocator`]'s current chunk.
    ///
    /// Returns a pointer to the start of an uninitialized section of `bytes` bytes.
    ///
    /// Note: [`alloc_layout`] allocates at *end* of the current chunk, because `Arena` bumps downwards,
    /// hence the need for this method, to allocate at *start* of current chunk.
    ///
    /// This method is dangerous, and should not ordinarily be used.
    ///
    /// This method moves the pointer to start of the current chunk forwards, so it no longer correctly
    /// describes the start of the allocation obtained from system allocator.
    ///
    /// The `Allocator` **must not be allowed to be dropped** or it would be UB.
    /// Only use this method if you prevent that possibililty. e.g.:
    ///
    /// 1. Set the start pointer back to its correct value before it is dropped,
    ///    using `Arena::set_start_ptr`.
    /// 2. Wrap the `Allocator` in `ManuallyDrop`, and deallocate it manually with the correct pointer.
    ///
    /// # Panics
    ///
    /// Panics if insufficient capacity for `bytes`
    /// (after rounding up to nearest multiple of [`RAW_MIN_ALIGN`]).
    ///
    /// # SAFETY
    ///
    /// `Allocator` must not be dropped after calling this method (see above).
    ///
    /// [`alloc_layout`]: Self::alloc_layout
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    #[inline(always)]
    pub unsafe fn alloc_bytes_start(&self, bytes: usize) -> NonNull<u8> {
        // SAFETY: Safety constraints of `Arena::alloc_bytes_start` are same as this method's
        unsafe { self.arena().alloc_bytes_start(bytes) }
    }

    /// Get pointer to end of the data region of this [`Allocator`]'s current chunk
    /// (i.e. to the start of the `ChunkFooter`).
    #[inline(always)]
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        self.arena().data_end_ptr()
    }

    /// Get pointer to end of this [`Allocator`]'s current chunk (after the `ChunkFooter`).
    #[inline(always)]
    pub fn end_ptr(&self) -> NonNull<u8> {
        self.arena().end_ptr()
    }

    /// Get data pointer for this [`Allocator`]'s current chunk.
    ///
    /// This is an alias for `arena().start_ptr()` for backward compatibility.
    #[inline(always)]
    pub fn data_ptr(&self) -> NonNull<u8> {
        self.arena().start_ptr()
    }

    /// Set data pointer for this [`Allocator`]'s current chunk.
    ///
    /// This is an alias for `arena().set_start_ptr()` for backward compatibility.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    /// * `ptr` must point to within the `Allocator`'s current chunk.
    /// * `ptr` must be aligned on [`RAW_MIN_ALIGN`].
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    #[inline(always)]
    pub unsafe fn set_data_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees safety requirements
        unsafe { self.arena().set_start_ptr(ptr) }
    }
}
