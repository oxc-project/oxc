use crate::Allocator;

/// Accessor for getting the underlying allocator.
pub trait AllocatorAccessor<'a> {
    /// Get the underlying allocator.
    fn allocator(self) -> &'a Allocator;
}

impl<'a> AllocatorAccessor<'a> for &'a Allocator {
    #[inline]
    fn allocator(self) -> &'a Allocator {
        self
    }
}
