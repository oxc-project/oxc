use oxc_allocator::{Allocator, CloneIn};

use crate::ast::Modifier;

impl<'alloc> CloneIn<'alloc> for Modifier {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        *self
    }
}
