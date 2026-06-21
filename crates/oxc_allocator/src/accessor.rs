use crate::Allocator;

/// Accessor for getting the underlying allocator.
pub trait GetAllocator<'a> {
    /// Get the underlying allocator.
    fn allocator(self) -> &'a Allocator;
}

impl<'a> GetAllocator<'a> for &'a Allocator {
    #[inline]
    fn allocator(self) -> &'a Allocator {
        self
    }
}
