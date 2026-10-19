use std::{fmt, mem::transmute_copy, ops::Deref, ptr};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use crate::formatter::JsFormatter;

use super::AstNodes;

/// A handle to an AST node: a `Copy` reference, never allocated.
///
/// The node's ancestors are not in the handle but on the formatter's ancestor stack
/// (see [`Ancestors`](super::Ancestors)), so the queries about them take the formatter.
pub struct AstNode<'a, T> {
    pub(super) inner: &'a T,
}

impl<T> Clone for AstNode<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AstNode<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for AstNode<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AstNode").field("inner", &self.inner).finish()
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
    pub fn as_option(self) -> Option<AstNode<'a, T>> {
        self.inner.as_ref().map(AstNode::new)
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

/// The address of an AST node, its identity.
#[inline]
pub fn node_address<T>(node: &T) -> usize {
    ptr::from_ref(node).addr()
}

/// Converts a handle of a node with a kind (or of an enum of them) to the typed view the ancestor stack holds.
pub trait AsAstNodes<'a> {
    fn as_ast_nodes(&self) -> AstNodes<'a>;
}

impl<'a, T> AstNode<'a, T> {
    #[inline]
    pub fn new(inner: &'a T) -> Self {
        AstNode { inner }
    }

    /// The node, for the arena lifetime (the [`AsRef`] one is for the borrow of the handle).
    #[inline]
    pub fn as_ref(self) -> &'a T {
        self.inner
    }

    /// The address of the node, see [`node_address`].
    #[inline]
    pub fn address(self) -> usize {
        node_address(self.inner)
    }
}

impl<'a, T> AstNode<'a, T>
where
    Self: AsAstNodes<'a>,
{
    /// See [`AsAstNodes`], callable without the trait in scope.
    #[inline]
    pub fn as_ast_nodes(self) -> AstNodes<'a> {
        AsAstNodes::as_ast_nodes(&self)
    }

    /// Returns an iterator over the ancestor nodes in the AST, starting from the parent.
    ///
    /// The iteration proceeds upward through the tree,
    /// terminating after yielding the root node (`Program`).
    ///
    /// # Example
    /// ```text
    /// Program
    ///   └─ BlockStatement
    ///       └─ ExpressionStatement  <- self
    /// ```
    /// For `self` as ExpressionStatement, this yields: [BlockStatement, Program]
    ///
    /// # Usage
    /// ```ignore
    /// // Find the first ancestor that matches a condition
    /// let parent = self.ancestors(f)
    ///     .find(|p| matches!(p, AstNodes::ForStatement(_)))
    ///     .unwrap();
    ///
    /// // Check if any ancestor is a specific type
    /// let in_arrow_fn = self.ancestors(f)
    ///     .any(|p| matches!(p, AstNodes::ArrowFunctionExpression(_)));
    /// ```
    pub fn ancestors<'f>(
        self,
        f: &'f JsFormatter<'_, 'a>,
    ) -> impl Iterator<Item = AstNodes<'a>> + use<'f, 'a, T> {
        f.context().ancestors().ancestors_of(self.as_ast_nodes()).skip(1)
    }

    /// Returns the parent node.
    #[inline]
    pub fn parent(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        f.context().ancestors().parent_of(self.as_ast_nodes())
    }

    /// Returns the grandparent node (parent's parent).
    pub fn grand_parent(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        self.parent(f).parent(f)
    }

    /// The start of the following sibling node (0 if none), the bound of this node's trailing comments.
    /// Computed from the parent, see [`Ancestors::following_span_start`](super::Ancestors::following_span_start).
    #[inline]
    pub fn following_span_start(self, f: &JsFormatter<'_, 'a>) -> u32 {
        f.context().ancestors().following_span_start(self.as_ast_nodes())
    }

    /// Check if this node is the callee of a CallExpression or NewExpression
    pub fn is_call_like_callee(self, f: &JsFormatter<'_, 'a>) -> bool
    where
        T: GetSpan,
    {
        self.parent(f).is_call_like_callee_span(self.span())
    }

    /// Check if this node is the callee of a NewExpression
    pub fn is_new_callee(self, f: &JsFormatter<'_, 'a>) -> bool
    where
        T: GetSpan,
    {
        matches!(self.parent(f), AstNodes::NewExpression(new) if new.callee.span() == self.span())
    }
}

impl<'a> AstNode<'a, ArrowFunctionBody<'a>> {
    /// Returns the concise expression body as an [`AstNode`].
    pub fn as_expression(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.as_expression().map(AstNode::new)
    }
}

impl<'a> AstNode<'a, ImportExpression<'a>> {
    /// Converts the arguments of the ImportExpression into an `AstNode` representing a `Vec` of `Argument`.
    /// Its items are this node's children: they are `source` and `options` themselves.
    #[inline]
    pub fn to_arguments(self, allocator: &'a Allocator) -> AstNode<'a, ArenaVec<'a, Argument<'a>>> {
        // Convert ImportExpression's source and options to ArenaVec<'a, Argument<'a>>.
        // This allows us to reuse CallExpression's argument formatting logic when printing
        // import expressions, since import(source, options) has the same structure as
        // a function call with arguments.
        let mut arguments = ArenaVec::new_in(&allocator);

        // SAFETY: Argument inherits all Expression variants through the inherit_variants! macro,
        // so Expression and Argument have identical memory layout for shared variants.
        // Both are discriminated unions where each Expression variant (e.g., Expression::Identifier)
        // has a corresponding Argument variant (e.g., Argument::Identifier) with the same discriminant
        // and the same inner type (ArenaBox<'a, T>). Transmuting Expression to Argument via `transmute_copy`
        // is safe because we're just copying the bits (discriminant + pointer).
        //
        // TODO(unsound): The logic in comment above is flawed - this is unsound.
        // `Expression` and `Argument`'s variants both contain `Box`es, which own the node the `Box`es contain.
        // `transmute_copy` here duplicates the `Box`, which means you have 2 `Box`es owning the same node.
        // This can lead to a violation of Rust's aliasing rules. Fix this.
        unsafe {
            arguments.push(transmute_copy(&self.inner.source));
            if let Some(ref options) = self.inner.options {
                arguments.push(transmute_copy(options));
            }
        }

        AstNode::new(allocator.alloc(arguments))
    }
}

impl AstNode<'_, CallExpression<'_>> {
    /// Check if the passing span is the callee of this CallExpression
    pub fn is_callee_span(self, span: Span) -> bool {
        self.inner.callee.span() == span
    }

    /// Check if the passing span is an argument of this CallExpression
    pub fn is_argument_span(self, span: Span) -> bool {
        !self.is_callee_span(span)
    }
}

impl AstNode<'_, NewExpression<'_>> {
    /// Check if the passing span is the callee of this NewExpression
    pub fn is_callee_span(self, span: Span) -> bool {
        self.inner.callee.span() == span
    }

    /// Check if the passing span is an argument of this NewExpression
    pub fn is_argument_span(self, span: Span) -> bool {
        !self.is_callee_span(span)
    }
}
