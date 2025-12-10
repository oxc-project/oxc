//! Implementations of methods for [`AstNodes`].

use oxc_span::{GetSpan, Span};

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

    /// If the node is a ChainExpression, recursively skip to its parent until a non-ChainExpression node is found.
    /// This is useful for analyses that want to ignore the presence of ChainExpressions in the AST.
    pub fn without_chain_expression(&self) -> &AstNodes<'a> {
        match self {
            AstNodes::ChainExpression(chain_expression) => {
                chain_expression.parent.without_chain_expression()
            }
            _ => self,
        }
    }

    /// Check if the passing span is the callee of a CallExpression or NewExpression
    pub fn is_call_like_callee_span(&self, span: Span) -> bool {
        match self {
            AstNodes::CallExpression(expr) => expr.callee.span() == span,
            AstNodes::NewExpression(expr) => expr.callee.span() == span,
            _ => false,
        }
    }
}
