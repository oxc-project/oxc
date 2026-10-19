//! Implementations of methods for [`AstNodes`].

use oxc_span::{GetSpan, Span};

use crate::{ast_nodes::AstNodes, formatter::JsFormatter};

impl<'a> AstNodes<'a> {
    /// Returns an iterator over all ancestor nodes in the AST, starting from self.
    ///
    /// The iteration includes the current node and proceeds upward through the tree,
    /// terminating after yielding the root node (`Program`).
    ///
    /// # Example hierarchy
    /// ```text
    /// Program
    ///   └─ BlockStatement
    ///       └─ ExpressionStatement  <- self
    /// ```
    /// For `self` as ExpressionStatement, this yields: [ExpressionStatement, BlockStatement, Program]
    pub fn ancestors<'f>(
        &self,
        f: &'f JsFormatter<'_, 'a>,
    ) -> impl Iterator<Item = AstNodes<'a>> + use<'f, 'a> {
        f.context().ancestors().ancestors_of(*self)
    }

    /// Returns the parent of this node, which is on the ancestor stack or a child of the node on top of it.
    ///
    /// # Panics
    ///
    /// Panics when called on a `Dummy` node.
    #[inline]
    pub fn parent(&self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        f.context().ancestors().parent_of(*self)
    }

    /// If the node is a ChainExpression, recursively skip to its parent until a non-ChainExpression node is found.
    /// This is useful for analyses that want to ignore the presence of ChainExpressions in the AST.
    pub fn without_chain_expression(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        let mut node = self;
        while let AstNodes::ChainExpression(_) = node {
            node = node.parent(f);
        }
        node
    }

    /// Whether `self` and `other` are the same node.
    #[inline]
    pub fn is(&self, other: &AstNodes<'_>) -> bool {
        self.address() == other.address()
            && std::mem::discriminant(self) == std::mem::discriminant(other)
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
