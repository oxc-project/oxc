//! AST Node ID and flags.

use bitflags::bitflags;

use oxc_allocator::{Allocator, CloneIn, Dummy};
use oxc_ast_macros::ast;
use oxc_index::define_nonmax_u32_index_type;

define_nonmax_u32_index_type! {
    /// AST Node ID
    #[ast]
    #[clone_in(default)]
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

    fn clone_in(&self, _: &'alloc Allocator) -> Self {
        // `clone_in` should never reach this, because `CloneIn` skips `node_id` field
        unreachable!();
    }

    #[inline]
    fn clone_in_with_semantic_ids(&self, _: &'alloc Allocator) -> Self {
        *self
    }
}

bitflags! {
    /// Contains additional information about an AST node.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NodeFlags: u8 {
        /// Set if the Node has a JSDoc comment attached
        const JSDoc     = 1 << 0;
        /// Set on Nodes inside classes
        const Class     = 1 << 1;
        /// Set functions containing yield statements
        const HasYield  = 1 << 2;
        /// Set for `export { specifier }`
        const ExportSpecifier  = 1 << 3;
    }
}

impl NodeFlags {
    /// Returns `true` if this node has a JSDoc comment attached to it.
    #[inline]
    pub fn has_jsdoc(self) -> bool {
        self.contains(Self::JSDoc)
    }

    /// Returns `true` if this node is inside a class.
    #[inline]
    pub fn has_class(self) -> bool {
        self.contains(Self::Class)
    }

    /// Returns `true` if this function has a yield statement.
    #[inline]
    pub fn has_yield(self) -> bool {
        self.contains(Self::HasYield)
    }

    /// Returns `true` if this function has an export specifier.
    #[inline]
    pub fn has_export_specifier(self) -> bool {
        self.contains(Self::ExportSpecifier)
    }
}
