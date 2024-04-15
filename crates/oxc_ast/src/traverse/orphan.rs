use std::ops::Deref;

/// Wrapper for AST nodes which have been disconnected from the AST.
///
/// This type is central to preventing a node from being attached to the AST in multiple places.
///
/// `Orphan` cannot be `Copy` or `Clone`, or it would allow creating a duplicate ref to the
/// contained node. The original `Orphan` could be attached to the AST, and then the copy
/// could also be attached to the AST elsewhere.
#[repr(transparent)]
pub struct Orphan<T>(T);

impl<T> Orphan<T> {
    /// Wrap node to indicate it's disconnected from AST.
    /// SAFETY: Caller must ensure that `node` is not attached to the AST.
    #[inline]
    pub unsafe fn new(node: T) -> Self {
        Self(node)
    }

    /// Unwrap node from `Orphan<T>`.
    /// This should only be done before inserting it into the AST.
    /// Not unsafe as there is nothing bad you can do with an un-orphaned AST node.
    /// No APIs are provided to attach nodes to the AST, unless they're wrapped in `Orphan<T>`.
    #[inline]
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Orphan<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}
