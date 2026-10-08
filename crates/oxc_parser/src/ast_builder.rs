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
    // NonMaxU32 stores the complement of its index. Keep the counter in that
    // form so constructing each NodeId can reuse its bits.
    next_node_id_complement: Cell<u32>,
}

impl<'a> ParserAstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator, next_node_id_complement: Cell::new(!1) }
    }
}

impl<'a> AstBuild<'a> for ParserAstBuilder<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        let complement = self.next_node_id_complement.get();
        // Check the reserved maximum before decrementing, so the counter
        // cannot wrap and reuse an ID.
        let node_id = NodeId::new((!complement) as usize);
        self.next_node_id_complement.set(complement - 1);
        node_id
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
    #[should_panic(expected = "index_vec index overflow")]
    fn node_ids_reject_the_reserved_maximum() {
        let allocator = Allocator::default();
        let builder = ParserAstBuilder::new(&allocator);
        builder.next_node_id_complement.set(1);
        assert_eq!(builder.node_id().index(), NodeId::MAX_INDEX);
        builder.node_id();
    }
}
