//! Define additional methods, used only by raw transfer:
//!
//! * [`Allocator::from_raw_parts`]
//! * [`Allocator::alloc_bytes_start`]
//! * [`Allocator::data_ptr`]
//! * [`Allocator::set_data_ptr`]
//! * [`Allocator::set_cursor_ptr`]
//! * [`Allocator::data_end_ptr`]
//! * [`Allocator::end_ptr`]

use std::ptr::NonNull;

use crate::{
    Allocator,
    bump::{ALIGN, Bump, ChunkFooter, FOOTER_SIZE},
};

impl Allocator {
    /// Minimum size for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_SIZE: usize = FOOTER_SIZE;

    /// Minimum alignment for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_ALIGN: usize = ALIGN;

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
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within
    ///   a single allocation.
    ///
    /// # Panics
    ///
    /// Panics on a big endian system.
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    /// [`RAW_MIN_SIZE`]: Self::RAW_MIN_SIZE
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        // Only support little-endian systems.
        #[expect(clippy::manual_assert)]
        if cfg!(target_endian = "big") {
            panic!("`Allocator::from_raw_parts` is not supported on big-endian systems.");
        }

        // Debug assert that `ptr` and `size` fulfill size and alignment requirements
        debug_assert!((ptr.as_ptr() as usize).is_multiple_of(ALIGN));
        debug_assert!(size.is_multiple_of(ALIGN));
        debug_assert!(size >= FOOTER_SIZE);

        // Create bump allocator from raw parts
        // SAFETY: Caller guarantees the memory region is valid
        let bump = unsafe { Bump::from_raw_parts(ptr, size) };

        Self::from_bump(bump)
    }

    /// Allocate space for `bytes` bytes at start of [`Allocator`]'s current chunk.
    ///
    /// Returns a pointer to the start of an uninitialized section of `bytes` bytes.
    ///
    /// Note: [`alloc_layout`] allocates at *end* of the current chunk (backward allocation),
    /// hence the need for this method, to allocate at *start* of current chunk.
    ///
    /// This method is dangerous, and should not ordinarily be used.
    ///
    /// This method moves the pointer to start of the current chunk forwards, so it no longer correctly
    /// describes the start of the allocation obtained from system allocator.
    ///
    /// The `Allocator` **must not be allowed to be dropped** or it would be UB.
    /// Only use this method if you prevent that possibility. e.g.:
    ///
    /// 1. Set the data pointer back to its correct value before it is dropped, using [`set_data_ptr`].
    /// 2. Wrap the `Allocator` in `ManuallyDrop`, and deallocate its memory manually with the correct pointer.
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
    /// [`set_data_ptr`]: Self::set_data_ptr
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    pub unsafe fn alloc_bytes_start(&self, bytes: usize) -> NonNull<u8> {
        // Round up number of bytes to reserve to multiple of `ALIGN`,
        // so data pointer remains aligned on `ALIGN`.
        //
        // `saturating_add` is required to prevent overflow in case `bytes > usize::MAX - ALIGN`.
        // In that case, `alloc_bytes` will be rounded down instead of up here, but that's OK because
        // no allocation can be larger than `isize::MAX` bytes, and therefore it's guaranteed that
        // `free_capacity < isize::MAX`. So `free_capacity > alloc_bytes` check below will always fail
        // for such a large value of `alloc_bytes`, regardless of rounding.
        //
        // It's preferable to use branchless `saturating_add` instead of `assert!(size <= isize::MAX as usize)`,
        // to avoid a branch here.
        let alloc_bytes = bytes.saturating_add(ALIGN - 1) & !(ALIGN - 1);

        let data_ptr = self.data_ptr();
        let cursor_ptr = self.cursor_ptr();
        // SAFETY: Cursor pointer is always `>=` data pointer.
        // Both pointers are within same allocation, and derived from the same original pointer.
        let free_capacity = unsafe { cursor_ptr.offset_from_unsigned(data_ptr) };

        // Check sufficient capacity to write `alloc_bytes` bytes, without overwriting data already
        // stored in allocator.
        // Could use `>=` here and it would be sufficient capacity, but use `>` instead so this assertion
        // fails if current chunk is the empty chunk and `bytes` is 0.
        assert!(free_capacity > alloc_bytes);

        // Calculate new data pointer.
        // SAFETY: We checked above that distance between data pointer and cursor is `>= alloc_bytes`,
        // so moving data pointer forwards by `alloc_bytes` cannot place it after cursor pointer.
        let new_data_ptr = unsafe { data_ptr.add(alloc_bytes) };

        // Set new data pointer.
        // SAFETY: `Allocator` must have at least 1 allocated chunk or check for sufficient capacity
        // above would have failed.
        // `new_data_ptr` cannot be after the cursor pointer, or free capacity check would have failed.
        // Data pointer is always aligned on `ALIGN`, and we rounded `alloc_bytes` up to a multiple
        // of `ALIGN`, so that remains the case.
        // It is caller's responsibility to ensure that the `Allocator` is not dropped after this call.
        unsafe { self.set_data_ptr(new_data_ptr) };

        // Return original data pointer
        data_ptr
    }

    /// Get data pointer for this [`Allocator`]'s current chunk.
    pub fn data_ptr(&self) -> NonNull<u8> {
        // SAFETY: We access the chunk footer safely
        unsafe { self.bump().data_ptr() }
    }

    /// Set data pointer for this [`Allocator`]'s current chunk.
    ///
    /// This is dangerous, and this method should not ordinarily be used.
    /// It is only here for manually writing data to start of the allocator chunk,
    /// and then adjusting the start pointer to after it.
    ///
    /// If calling this method with any pointer which is not the original data pointer for this
    /// `Allocator` chunk, the `Allocator` must NOT be allowed to be dropped after this call,
    /// because data pointer no longer correctly describes the start of the allocation obtained
    /// from system allocator. If the `Allocator` were dropped, it'd be UB.
    ///
    /// Only use this method if you prevent that possibility. e.g.:
    ///
    /// 1. Set the data pointer back to its correct value before it is dropped, using [`set_data_ptr`].
    /// 2. Wrap the `Allocator` in `ManuallyDrop`, and deallocate its memory manually with the correct pointer.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    ///   It is UB to call this method on an `Allocator` which has not allocated
    ///   i.e. fresh from `Allocator::new`.
    /// * `ptr` must point to within the `Allocator`'s current chunk.
    /// * `ptr` must be equal to or before cursor pointer for this chunk.
    /// * `ptr` must be aligned on [`RAW_MIN_ALIGN`].
    /// * `Allocator` must not be dropped if `ptr` is not the original data pointer for this chunk.
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    /// [`set_data_ptr`]: Self::set_data_ptr
    pub unsafe fn set_data_ptr(&self, ptr: NonNull<u8>) {
        debug_assert!((ptr.as_ptr() as usize).is_multiple_of(ALIGN));
        // SAFETY: Caller guarantees validity
        unsafe { self.bump().set_data_ptr(ptr) };
    }

    /// Get cursor pointer for this [`Allocator`]'s current chunk.
    fn cursor_ptr(&self) -> NonNull<u8> {
        // SAFETY: We access the bump pointer safely
        unsafe { self.bump().cursor_ptr() }
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
    pub unsafe fn set_cursor_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees validity
        unsafe { self.bump().set_cursor_ptr(ptr) };
    }

    /// Get pointer to end of the data region of this [`Allocator`]'s current chunk
    /// i.e to the start of the `ChunkFooter`.
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        self.chunk_footer_ptr().cast::<u8>()
    }

    /// Get pointer to end of this [`Allocator`]'s current chunk (after the `ChunkFooter`).
    pub fn end_ptr(&self) -> NonNull<u8> {
        // SAFETY: `chunk_footer_ptr` returns pointer to a valid `ChunkFooter`,
        // so stepping past it cannot be out of bounds of the chunk's allocation.
        // If `Allocator` has not allocated, so `chunk_footer_ptr` returns a pointer to the static
        // empty chunk, it's still valid.
        unsafe { self.chunk_footer_ptr().add(1).cast::<u8>() }
    }

    /// Get pointer to current chunk's [`ChunkFooter`].
    fn chunk_footer_ptr(&self) -> NonNull<ChunkFooter> {
        // SAFETY: We access the chunk pointer safely
        unsafe { self.bump().chunk_footer_ptr() }
    }
}
