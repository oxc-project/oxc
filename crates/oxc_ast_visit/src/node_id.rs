use oxc_ast::{AstKind, ast::Program};
use oxc_syntax::node::NodeId;

use crate::Visit;

/// Assigns dense [`NodeId`]s to every node in an AST.
///
/// This is the lightweight alternative to semantic analysis for consumers that
/// need stable node identities but do not need scopes, symbols, or references.
/// IDs use the same depth-first traversal order as [`oxc_semantic::SemanticBuilder`],
/// starting with [`NodeId::ROOT`] for the [`Program`].
///
/// Running semantic analysis later will assign the same IDs as long as the AST
/// has not been structurally changed in between the two passes.
pub struct AstNodeIdAssigner {
    node_count: u32,
}

impl AstNodeIdAssigner {
    /// Assign node IDs to `program` and return the number of assigned IDs.
    pub fn assign(program: &Program<'_>) -> u32 {
        let mut assigner = Self { node_count: 0 };
        assigner.visit_program(program);
        assigner.node_count
    }
}

impl<'a> Visit<'a> for AstNodeIdAssigner {
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let node_id = NodeId::new(self.node_count as usize);
        kind.set_node_id(node_id);
        self.node_count += 1;
    }
}
