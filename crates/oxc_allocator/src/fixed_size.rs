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
    /// Returns `None` if the allocation fails.
    ///
    /// [`System`]: std::alloc::System
    pub fn new_fixed_size() -> Option<Self> {
        let arena = Arena::new_fixed_size()?;
        Some(Self::from_arena(arena))
    }
}
