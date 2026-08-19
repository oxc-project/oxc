use std::cell::Cell;

use oxc_allocator::{Allocator, GetAllocator};
use oxc_ast::builder::{AstBuild, GetAstBuilder};
use oxc_syntax::node::NodeId;

/// AST builder used by the parser.
///
/// IDs are allocated as nodes are constructed. The counter is intentionally not part of parser
/// checkpoints: nodes discarded by lookahead or reparsing leave gaps, and IDs are never reused.
pub(crate) struct ParserAstBuilder<'a> {
    allocator: &'a Allocator,
    next_node_id: Cell<u32>,
}

impl<'a> ParserAstBuilder<'a> {
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        // `0` is reserved for `Program` (`NodeId::ROOT`) and doubles as `NodeId::DUMMY`.
        Self { allocator, next_node_id: Cell::new(1) }
    }

    #[cfg(test)]
    #[inline]
    pub fn next_node_id(&self) -> u32 {
        self.next_node_id.get()
    }
}

impl<'a> AstBuild<'a> for ParserAstBuilder<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        let raw = self.next_node_id.get();
        self.next_node_id.set(raw.checked_add(1).expect("AST node ID overflow"));
        NodeId::new(raw as usize)
    }
}

impl<'a> GetAstBuilder<'a> for ParserAstBuilder<'a> {
    type Builder = Self;

    #[inline]
    fn builder(&self) -> &Self {
        self
    }
}

impl<'a> GetAllocator<'a> for ParserAstBuilder<'a> {
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_monotonic_non_root_ids() {
        let allocator = Allocator::default();
        let builder = ParserAstBuilder::new(&allocator);
        assert_eq!(builder.node_id(), NodeId::new(1));
        assert_eq!(builder.node_id(), NodeId::new(2));
        assert_eq!(builder.next_node_id(), 3);
    }
}
