use core::fmt;
use std::ops::Deref;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_span::{GetSpan, Span};

use super::AstNodes;

pub struct AstNode<'a, T> {
    pub(super) inner: &'a T,
    pub parent: &'a AstNodes<'a>,
    pub(super) allocator: &'a Allocator,
    pub(super) following_span: Option<Span>,
}

impl<T: fmt::Debug> fmt::Debug for AstNode<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AstNode")
            .field("inner", &self.inner)
            .field("parent", &self.parent.debug_name())
            .field("following_span", &self.following_span)
            .finish()
    }
}

impl<'a, T> Deref for AstNode<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        self.inner
    }
}

impl<'a, T> AsRef<T> for AstNode<'a, T> {
    fn as_ref(&self) -> &'a T {
        self.inner
    }
}

impl<'a, T> AstNode<'a, Option<T>> {
    pub fn as_ref(&self) -> Option<&'a AstNode<'a, T>> {
        self.allocator
            .alloc(self.inner.as_ref().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_span: self.following_span,
            }))
            .as_ref()
    }
}

impl<T: GetSpan> GetSpan for AstNode<'_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<T: GetSpan> GetSpan for &AstNode<'_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<T> AstNode<'_, T> {
    /// Returns an iterator over all ancestor nodes in the AST, starting from self.
    ///
    /// The iteration includes the current node and proceeds upward through the tree,
    /// terminating after yielding the root `Program` node.
    ///
    /// This is a convenience method that delegates to `self.parent.ancestors()`.
    ///
    /// # Example
    /// ```text
    /// Program
    ///   └─ BlockStatement
    ///       └─ ExpressionStatement  <- self
    /// ```
    /// For `self` as ExpressionStatement, this yields: [ExpressionStatement, BlockStatement, Program]
    ///
    /// # Usage
    /// ```ignore
    /// // Find the first ancestor that matches a condition
    /// let parent = self.ancestors()
    ///     .find(|p| matches!(p, AstNodes::ForStatement(_)))
    ///     .unwrap();
    ///
    /// // Get the nth ancestor
    /// let great_grandparent = self.ancestors().nth(3);
    ///
    /// // Check if any ancestor is a specific type
    /// let in_arrow_fn = self.ancestors()
    ///     .any(|p| matches!(p, AstNodes::ArrowFunctionExpression(_)));
    /// ```
    pub fn ancestors(&self) -> impl Iterator<Item = &AstNodes<'_>> {
        self.parent.ancestors()
    }

    /// Returns the grandparent node (parent's parent).
    ///
    /// This is a convenience method equivalent to `self.parent.parent()`.
    pub fn grand_parent(&self) -> &AstNodes<'_> {
        self.parent.parent()
    }
}

impl<'a> AstNode<'a, Program<'a>> {
    pub fn new(inner: &'a Program<'a>, parent: &'a AstNodes<'a>, allocator: &'a Allocator) -> Self {
        AstNode { inner, parent, allocator, following_span: None }
    }
}
