//! Implementations of methods for [`AstNodes`].

use crate::ast_nodes::AstNodes;

impl<'a> AstNodes<'a> {
    /// Returns an iterator over all ancestor nodes in the AST, starting from self.
    ///
    /// The iteration includes the current node and proceeds upward through the tree,
    /// terminating after yielding the root `Program` node.
    ///
    /// # Example hierarchy
    /// ```text
    /// Program
    ///   └─ BlockStatement
    ///       └─ ExpressionStatement  <- self
    /// ```
    /// For `self` as ExpressionStatement, this yields: [ExpressionStatement, BlockStatement, Program]
    pub fn ancestors(&self) -> impl Iterator<Item = &AstNodes<'a>> {
        // Start with the current node and walk up the tree, including Program
        std::iter::successors(Some(self), |node| {
            // Continue iteration until we've yielded Program (root node)
            // After Program, parent() would still return Program, so stop there
            if matches!(node, AstNodes::Program(_)) { None } else { Some(node.parent()) }
        })
    }
}
