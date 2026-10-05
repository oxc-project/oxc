use std::cell::Cell;

use oxc_allocator::{Allocator, GetAllocator};
use oxc_ast::builder::{AstBuild, GetAstBuilder};
use oxc_syntax::node::NodeId;

/// Assign IDs as nodes are constructed, reserving zero for Program.
///
/// IDs follow construction order, rather than visitor order, and may have gaps
/// from discarded nodes. The counter is deliberately not rewound during
/// speculation or top-level await reparsing: nodes constructed after an earlier
/// checkpoint can still survive in the returned AST.
pub struct ParserAstBuilder<'a> {
    allocator: &'a Allocator,
    next_node_id: Cell<usize>,
}

impl<'a> ParserAstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator, next_node_id: Cell::new(1) }
    }
}

impl<'a> AstBuild<'a> for ParserAstBuilder<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        let index = self.next_node_id.get();
        assert!(index < NodeId::MAX_INDEX, "Too many AST nodes");
        self.next_node_id.set(index + 1);
        NodeId::new(index)
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
