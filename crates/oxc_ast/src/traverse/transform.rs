#![allow(dead_code)] // TODO: Remove this attr once used in a transform

use oxc_allocator::Allocator;

use super::{Ancestor, AstBuilder, SharedBox};

pub fn transform() {
    // TODO
}

/// Traverse context.
///
/// Passed to all AST visitor functions.
///
/// Provides ability to:
/// * Query parent/ancestor of current node.
/// * Create AST nodes via `ctx.ast`.
/// * Allocate into arena via `ctx.alloc()`.
pub struct TraverseCtx<'a> {
    stack: Vec<Ancestor<'a>>,
    pub ast: AstBuilder<'a>,
}

impl<'a> TraverseCtx<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { stack: Vec::new(), ast: AstBuilder::new(allocator) }
    }

    #[allow(dead_code)] // TODO: Remove this attr once method is used in a transform
    #[inline]
    pub fn alloc<T>(&self, node: T) -> SharedBox<'a, T> {
        self.ast.alloc(node)
    }
}

impl<'a> TraverseCtx<'a> {
    #[inline]
    pub fn parent(&self) -> Ancestor<'a> {
        *self.stack.last().unwrap()
    }

    #[inline]
    pub fn ancestor(&self, levels: usize) -> Option<Ancestor<'a>> {
        self.stack.get(self.stack.len() - levels).copied()
    }

    #[inline]
    pub fn push_stack(&mut self, parent: Ancestor<'a>) {
        self.stack.push(parent);
    }

    #[inline]
    pub fn pop_stack(&mut self) {
        self.stack.pop();
    }

    #[inline]
    pub fn replace_stack(&mut self, parent: Ancestor<'a>) {
        let index = self.stack.len() - 1;
        self.stack[index] = parent;
    }
}
