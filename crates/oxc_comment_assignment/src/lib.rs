//! Comment assignment for the Oxc AST.
//!
//! The pass currently assigns node IDs in preparation for attaching comments to nodes.

use oxc_ast::{AstKind, ast::Program};
use oxc_ast_visit::Visit;
use oxc_syntax::node::NodeId;

/// Assigns AST node IDs in preparation for comment assignment.
#[derive(Default)]
pub struct CommentAssignment {
    next_node_id: u32,
}

impl CommentAssignment {
    /// Create a new assignment pass.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assign fresh node IDs to every node in `program`.
    ///
    /// IDs are dense and follow AST visitor preorder, starting with [`NodeId::ROOT`]
    /// for the Program. The traversal runs even when the program contains no comments.
    /// Node IDs are updated through their interior-mutable cells.
    ///
    /// This renumbers existing nodes, invalidating any semantic data keyed by their
    /// previous node IDs. Such data must be rebuilt after running this pass.
    ///
    /// # Panics
    ///
    /// Panics if the number of nodes exceeds the range represented by [`NodeId`].
    pub fn assign(mut self, program: &Program<'_>) {
        self.next_node_id = 0;
        self.visit_program(program);
    }
}

impl<'a> Visit<'a> for CommentAssignment {
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let node_id = NodeId::new(self.next_node_id as usize);
        self.next_node_id += 1;
        kind.set_node_id(node_id);
    }
}
