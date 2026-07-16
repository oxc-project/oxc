//! AST Node ID and flags.

use bitflags::bitflags;

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds, Dummy};
use oxc_ast_macros::ast;
use oxc_index::define_nonmax_u32_index_type;

use crate::semantic_id::SemanticId;

define_nonmax_u32_index_type! {
    /// AST Node ID
    #[ast]
    #[clone_in(semantic_id)]
    #[content_eq(skip)]
    #[estree(skip)]
    pub struct NodeId;
}

impl NodeId {
    /// Mock node id.
    ///
    /// This is used for synthetically-created AST nodes, among other things.
    pub const DUMMY: Self = NodeId::new(0);

    /// Node id of the Program node.
    pub const ROOT: Self = NodeId::new(0);
}

impl Default for NodeId {
    #[inline]
    fn default() -> Self {
        Self::DUMMY
    }
}

impl<'a> Dummy<'a> for NodeId {
    #[inline]
    fn dummy(_: &'a Allocator) -> Self {
        Self::DUMMY
    }
}

impl<'alloc> CloneIn<'alloc> for NodeId {
    type Cloned = Self;

    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this method only delegates
    fn clone_in_impl(&self, with_semantic_ids: CloneInSemanticIds, _: &'alloc Allocator) -> Self {
        self.clone_id(with_semantic_ids)
    }
}

impl SemanticId for NodeId {}

bitflags! {
    /// Contains additional information about an AST node.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NodeFlags: u8 {
        /// Set if the Node has a JSDoc comment attached
        const JSDoc     = 1 << 0;
        /// Set functions containing yield statements
        const HasYield  = 1 << 2;
    }
}

impl NodeFlags {
    /// Returns `true` if this node has a JSDoc comment attached to it.
    #[inline]
    pub fn has_jsdoc(self) -> bool {
        self.contains(Self::JSDoc)
    }

    /// Returns `true` if this function has a yield statement.
    #[inline]
    pub fn has_yield(self) -> bool {
        self.contains(Self::HasYield)
    }
}
