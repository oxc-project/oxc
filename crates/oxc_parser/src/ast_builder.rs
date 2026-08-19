use std::cell::Cell;

use oxc_allocator::{Allocator, GetAllocator};
use oxc_ast::{
    AstType,
    builder::{AstBuild, GetAstBuilder},
};
use oxc_span::Span;
use oxc_syntax::node::NodeId;

use crate::comment_attacher::ParserCommentAttacher;

/// AST builder used by the parser.
///
/// AST constructors produce [`NodeId::DUMMY`]. A real ID is assigned only when a committed node
/// becomes a comment owner. This keeps parser-assigned IDs dense without making speculative or
/// temporary AST nodes part of the ID space.
pub(crate) struct ParserAstBuilder<'a> {
    allocator: &'a Allocator,
    next_node_id: Cell<u32>,
    comment_attacher: &'a ParserCommentAttacher<'a>,
}

impl<'a> ParserAstBuilder<'a> {
    #[inline]
    pub fn new(allocator: &'a Allocator, comment_attacher: &'a ParserCommentAttacher<'a>) -> Self {
        // `0` is reserved for `Program` (`NodeId::ROOT`) and doubles as `NodeId::DUMMY`.
        Self { allocator, next_node_id: Cell::new(1), comment_attacher }
    }

    #[inline]
    pub fn next_node_id(&self) -> u32 {
        self.next_node_id.get()
    }

    #[inline]
    pub fn rewind(&self, next_node_id: u32) {
        debug_assert!(next_node_id <= self.next_node_id.get());
        self.next_node_id.set(next_node_id);
        self.comment_attacher.rewind_nodes(NodeId::new(next_node_id as usize));
    }

    /// Return `node_id` if it is already real, otherwise allocate a new parser-owned ID.
    #[inline]
    pub fn ensure_node_id(&self, node_id: NodeId) -> NodeId {
        if node_id != NodeId::DUMMY {
            return node_id;
        }
        let raw = self.next_node_id.get();
        self.next_node_id.set(raw.checked_add(1).expect("AST node ID overflow"));
        NodeId::new(raw as usize)
    }
}

impl<'a> AstBuild<'a> for ParserAstBuilder<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        NodeId::DUMMY
    }

    fn finish_node(&self, node_id: &Cell<NodeId>, span: Span, ty: AstType) {
        if !self.comment_attacher.node_needs_id(span, ty) {
            return;
        }
        let id = self.ensure_node_id(node_id.get());
        node_id.set(id);
        self.comment_attacher.finish_node(id, span, ty);
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
    fn allocates_ids_only_for_committed_comment_owners() {
        let allocator = Allocator::default();
        let comment_attacher = ParserCommentAttacher::new(&allocator);
        let builder = ParserAstBuilder::new(&allocator, &comment_attacher);
        assert_eq!(builder.node_id(), NodeId::DUMMY);
        assert_eq!(builder.next_node_id(), 1);
        assert_eq!(builder.ensure_node_id(NodeId::DUMMY), NodeId::new(1));
        assert_eq!(builder.ensure_node_id(NodeId::new(1)), NodeId::new(1));
        assert_eq!(builder.ensure_node_id(NodeId::DUMMY), NodeId::new(2));
        assert_eq!(builder.next_node_id(), 3);
    }
}
