use std::mem;

use crate::{Allocator, Vec};

/// A trait to create a dummy AST node, and replace an existing node with a dummy.
pub trait TakeIn<'a>: Sized {
    /// Create a dummy node.
    fn dummy_in(allocator: &'a Allocator) -> Self;

    /// Replace node with a dummy.
    #[must_use]
    fn take_in(&mut self, allocator: &'a Allocator) -> Self {
        let dummy = Self::dummy_in(allocator);
        mem::replace(self, dummy)
    }
}

impl<'a, T> TakeIn<'a> for Vec<'a, T> {
    /// Create a dummy [`Vec`].
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Vec::new_in(allocator)
    }
}
