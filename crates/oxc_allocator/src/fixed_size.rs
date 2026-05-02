//! [`Allocator::new_fixed_size`]

use crate::{Allocator, arena::Arena};

impl Allocator {
    /// Construct a static-sized [`Allocator`] backed by an allocation made via the [`System`] allocator.
    ///
    /// The returned [`Allocator`] uses a single chunk of `BLOCK_SIZE` bytes, aligned on `BLOCK_ALIGN`.
    /// It cannot grow.
    ///
    /// Allocation is made via [`System`] allocator, bypassing any registered alternative global allocator
    /// (e.g. Mimalloc in linter). Mimalloc complains that it cannot serve allocations with high alignment,
    /// and presumably it's pointless to try to obtain such large allocations from a thread-local heap,
    /// so better to go direct to the system allocator anyway.
    ///
    /// The backing allocation is freed via the global allocator (NOT [`System`]) when the [`Allocator`]
    /// is dropped. Because the allocation here is made via [`System`], the returned [`Allocator`]
    /// MUST NOT be dropped, or [`dealloc`] will try to free the allocation via the wrong allocator.
    ///
    /// Returns `None` if the allocation fails.
    ///
    /// # SAFETY
    ///
    /// The returned [`Allocator`] must not be dropped.
    /// Caller must wrap it in [`ManuallyDrop`] and free the backing allocation manually via [`System::dealloc`],
    /// using the [`Layout`] stored in the inner `Arena`'s `ChunkFooter`.
    ///
    /// [`System`]: std::alloc::System
    /// [`dealloc`]: std::alloc::dealloc
    /// [`System::dealloc`]: std::alloc::GlobalAlloc::dealloc
    /// [`Layout`]: std::alloc::Layout
    /// [`ManuallyDrop`]: std::mem::ManuallyDrop
    pub unsafe fn new_fixed_size() -> Option<Self> {
        // SAFETY: Safety requirements of `Arena::new_fixed_size` are the same as for this method.
        let arena = unsafe { Arena::new_fixed_size() }?;
        Some(Self::from_arena(arena))
    }
}
