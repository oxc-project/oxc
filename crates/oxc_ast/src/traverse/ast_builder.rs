use oxc_allocator::Allocator;

use super::{cell::GCell, SharedBox};

/// AST builder for creating AST nodes for traversable AST.
// Further methods are added by `#[ast_node]` macro.
pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    #[inline]
    pub fn alloc<T>(&self, node: T) -> SharedBox<'a, T> {
        GCell::from_mut(self.allocator.alloc(node))
    }
}
