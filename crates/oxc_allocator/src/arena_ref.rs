use crate::Box;
use std::borrow::Cow;

/// Cheap reference-to-reference conversion for
/// [arena](crate::Allocator)-allocated types.
///
/// This trait is functionally equivalent to [`AsRef`] except
/// 1. No `&` syntax sugar
/// 2. Borrows are for the lifetime of the arena (`'alloc`)
pub trait AsArenaRef<'alloc, T: ?Sized> {
    /// Converts this type into a shared reference of the (usually inferred) input type.
    fn as_arena_ref(&'alloc self) -> &'alloc T;
}

impl<'alloc, T: ?Sized> AsArenaRef<'alloc, T> for Box<'alloc, T> {
    #[inline]
    fn as_arena_ref(&'alloc self) -> &'alloc T {
        self
    }
}

impl<'alloc, T: ?Sized + 'alloc> AsArenaRef<'alloc, T> for Cow<'alloc, T>
where
    T: ToOwned,
{
    #[inline]
    fn as_arena_ref(&'alloc self) -> &'alloc T {
        self
    }
}
