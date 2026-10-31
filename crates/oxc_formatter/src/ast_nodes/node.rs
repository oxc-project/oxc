use std::{fmt, mem::transmute_copy, ops::Deref};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use super::{AstNodeKind, AstNodes};

/// An AST node with its parent, created on the stack while the formatter descends the AST.
///
/// `'parent` is the lifetime of the parent node it refers to, `'a` is the lifetime of the AST.
/// An accessor for a child borrows `self`, so the parent must outlive the child.
///
/// # Links in the arena
///
/// A loop that walks down a chain (`a.b.c`, `a && b && c`) cannot keep each node on the stack,
/// as the next node refers to it as its parent.
/// Such a link is moved into the arena with `in_arena`, only when the walk continues past it:
/// - [`AstNodeKind::in_arena`] when the next step matches on its kind
/// - [`AstNode::in_arena`] when a match has already selected the variant,
///   as the specific node (e.g. `AstNode<CallExpression>`) is smaller than the kind
pub struct AstNode<'parent, 'a, T> {
    pub(super) inner: &'a T,
    pub(super) parent: AstNodes<'parent, 'a>,
    /// The start position of the following sibling node, or 0 if none.
    pub(super) following_span_start: u32,
}

// Not derived, to not require `T: Copy`
impl<T> Clone for AstNode<'_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AstNode<'_, '_, T> {}

impl<T: fmt::Debug> fmt::Debug for AstNode<'_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AstNode")
            .field("inner", &self.inner)
            .field("parent", &self.parent().debug_name())
            .field("following_span_start", &self.following_span_start)
            .finish()
    }
}

impl<'a, T> Deref for AstNode<'_, 'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        self.inner
    }
}

impl<'a, T> AsRef<T> for AstNode<'_, 'a, T> {
    fn as_ref(&self) -> &'a T {
        self.inner
    }
}

impl<'parent, 'a, T> AstNode<'parent, 'a, Option<T>> {
    pub fn as_ref(&self) -> Option<AstNode<'parent, 'a, T>> {
        self.inner.as_ref().map(|inner| AstNode {
            inner,
            parent: self.parent,
            following_span_start: self.following_span_start,
        })
    }
}

impl<T: GetSpan> GetSpan for AstNode<'_, '_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<T: GetSpan> GetSpan for &AstNode<'_, '_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<'parent, 'a, T> AstNode<'parent, 'a, T> {
    /// Returns the AST node, for the lifetime of the AST rather than the borrow of `self` (unlike [`Deref`] and [`AsRef`]).
    #[inline]
    pub fn inner(&self) -> &'a T {
        self.inner
    }

    /// Returns an iterator over all ancestor nodes in the AST, starting from self.
    ///
    /// The iteration includes the current node and proceeds upward through the tree,
    /// terminating after yielding the root `Program` node.
    ///
    /// This is a convenience method that delegates to `self.parent().ancestors()`.
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
    pub fn ancestors(&self) -> impl Iterator<Item = &AstNodes<'parent, 'a>> {
        self.parent().ancestors()
    }

    /// Returns the parent node.
    #[inline]
    pub fn parent(&self) -> &AstNodes<'parent, 'a> {
        &self.parent
    }

    /// The start of the following sibling node (0 if none), the bound of this node's trailing comments.
    #[inline]
    pub fn following_span_start(&self) -> u32 {
        self.following_span_start
    }

    /// Returns the grandparent node (parent's parent).
    ///
    /// This is a convenience method equivalent to `self.parent().parent()`.
    pub fn grand_parent(&self) -> &AstNodes<'parent, 'a> {
        self.parent().parent()
    }

    pub fn new(inner: &'a T, parent: AstNodes<'parent, 'a>) -> Self {
        AstNode { inner, parent, following_span_start: 0 }
    }

    /// Moves this node into the arena, see [links in the arena](AstNode#links-in-the-arena).
    #[inline]
    pub fn in_arena(self, allocator: &'parent Allocator) -> &'parent Self {
        allocator.alloc(self)
    }
}

impl<'parent> AstNodeKind<'parent, '_> {
    /// Moves this node into the arena, see [links in the arena](AstNode#links-in-the-arena).
    #[inline]
    pub fn in_arena(self, allocator: &'parent Allocator) -> &'parent Self {
        allocator.alloc(self)
    }
}

impl<T: GetSpan> AstNode<'_, '_, T> {
    /// Check if this node is the callee of a CallExpression or NewExpression
    pub fn is_call_like_callee(&self) -> bool {
        let callee = match self.parent() {
            AstNodes::CallExpression(call) => &call.callee,
            AstNodes::NewExpression(new) => &new.callee,
            _ => return false,
        };

        callee.span() == self.span()
    }

    /// Check if this node is the callee of a NewExpression
    pub fn is_new_callee(&self) -> bool {
        matches!(self.parent(), AstNodes::NewExpression(new) if new.callee.span() == self.span())
    }
}

impl<'parent, 'a> AstNode<'parent, 'a, ArrowFunctionBody<'a>> {
    /// Returns the concise expression body as an [`AstNode`].
    pub fn as_expression(&self) -> Option<AstNode<'parent, 'a, Expression<'a>>> {
        self.inner.as_expression().map(|inner| AstNode {
            inner,
            parent: self.parent,
            following_span_start: self.following_span_start,
        })
    }
}

impl<'a> AstNode<'_, 'a, ImportExpression<'a>> {
    /// Converts the arguments of the ImportExpression into an `AstNode` representing a `Vec` of `Argument`.
    #[inline]
    pub fn arguments(
        &self,
        allocator: &'a Allocator,
    ) -> AstNode<'_, 'a, ArenaVec<'a, Argument<'a>>> {
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

        AstNode {
            inner: allocator.alloc(arguments),
            parent: AstNodes::ImportExpression(self),
            following_span_start: self.following_span_start,
        }
    }
}

impl AstNode<'_, '_, CallExpression<'_>> {
    /// Check if the passing span is the callee of this CallExpression
    pub fn is_callee_span(&self, span: Span) -> bool {
        self.inner.callee.span() == span
    }

    /// Check if the passing span is an argument of this CallExpression
    pub fn is_argument_span(&self, span: Span) -> bool {
        !self.is_callee_span(span)
    }
}

impl AstNode<'_, '_, NewExpression<'_>> {
    /// Check if the passing span is the callee of this NewExpression
    pub fn is_callee_span(&self, span: Span) -> bool {
        self.inner.callee.span() == span
    }

    /// Check if the passing span is an argument of this NewExpression
    pub fn is_argument_span(&self, span: Span) -> bool {
        !self.is_callee_span(span)
    }
}
