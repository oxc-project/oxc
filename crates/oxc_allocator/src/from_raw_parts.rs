//! Define additional [`Allocator::from_raw_parts`] method, used only by raw transfer.

use std::{alloc::Layout, ptr::NonNull};

use crate::{
    Allocator,
    arena::{ArenaDefault as Arena, CHUNK_ALIGN, FOOTER_SIZE},
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
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, layout: Layout) -> Self {
        // SAFETY: Safety requirements of this method match `Arena::from_raw_parts`'s requirements
        let arena = unsafe { Arena::from_raw_parts(ptr, layout) };
        Self::from_arena(arena)
    }
}

/// Returns `true` if `n` is a multiple of `divisor`.
const fn is_multiple_of(n: usize, divisor: usize) -> bool {
    n % divisor == 0
}
