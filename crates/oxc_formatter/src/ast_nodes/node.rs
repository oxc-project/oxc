use std::{fmt, mem::transmute_copy, ops::Deref};

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use super::AstNodes;

/// Stack-allocated wrapper around an AST node.
///
/// `'me` is the parent's stack frame lifetime - the lifetime of the borrow against `self`'s
/// stack frame in the calling context. As the formatter recurses top-down through the AST,
/// `'me` shrinks naturally: each level's wrapper borrows its parent (which lives in an outer
/// stack frame) for `'me`, and produces children that borrow the current wrapper for a
/// shorter `'this` lifetime.
///
/// `'a` is the AST/arena lifetime - the lifetime of the parsed AST data.
pub struct AstNode<'me, 'a, T> {
    /// The wrapped AST node.
    ///
    /// Public so call sites can match on enum-typed inners directly (e.g.
    /// `match key.inner { PropertyKey::StringLiteral(s) => ... }`) and use
    /// [`AstNode::with_inner`] to rewrap.
    pub inner: &'a T,
    pub parent: AstNodes<'me, 'a>,
    /// The start position of the following sibling node, or 0 if none.
    pub following_span_start: u32,
}

// Manually implement `Copy`/`Clone` so they don't require `T: Copy + Clone` bounds.
// `AstNode` only holds a `&'a T`, so it's `Copy` regardless of `T`.
impl<T> Copy for AstNode<'_, '_, T> {}
impl<T> Clone for AstNode<'_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

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

impl<'me, 'a, T> AstNode<'me, 'a, Option<T>> {
    /// Specialise to an `Option<AstNode<T>>` viewed as the same logical position.
    ///
    /// Takes `self` by value (since `AstNode` is `Copy`) and returns a sibling wrapper
    /// inheriting `self.parent` and `self.following_span_start`.
    pub fn as_ref(self) -> Option<AstNode<'me, 'a, T>> {
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

impl<'me, 'a, T> AstNode<'me, 'a, T> {
    /// Returns an iterator over all ancestor nodes in the AST, starting from self.
    ///
    /// The iteration includes the current node and proceeds upward through the tree,
    /// terminating after yielding the root `Program` node.
    ///
    /// This is a convenience method that delegates to `self.parent().ancestors()`.
    pub fn ancestors(&self) -> impl Iterator<Item = &AstNodes<'me, 'a>> {
        self.parent().ancestors()
    }

    /// Returns the parent node.
    #[inline]
    pub fn parent(&self) -> &AstNodes<'me, 'a> {
        &self.parent
    }

    /// Returns the grandparent node (parent's parent).
    pub fn grand_parent(&self) -> &AstNodes<'me, 'a> {
        self.parent().parent()
    }

    /// Construct a sibling `AstNode` for an inner value at the same logical position.
    ///
    /// Used to specialise an enum-typed `AstNode` (e.g. `AstNode<Expression>`) to its
    /// concrete variant (e.g. `AstNode<BooleanLiteral>`) on the stack, inheriting this
    /// node's `parent` and `following_span_start`.
    ///
    /// Replaces the previous arena-allocating `as_ast_nodes()` API with a stack-only
    /// equivalent: callers match on `self.inner` directly and use `with_inner` to
    /// rewrap the matched variant.
    #[inline]
    pub fn with_inner<U>(&self, inner: &'a U) -> AstNode<'me, 'a, U> {
        AstNode { inner, parent: self.parent, following_span_start: self.following_span_start }
    }
}

impl<'me, 'a, T> AstNode<'me, 'a, T> {
    pub fn new(inner: &'a T, parent: AstNodes<'me, 'a>) -> Self {
        AstNode { inner, parent, following_span_start: 0 }
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

impl<'me, 'a> AstNode<'me, 'a, ExpressionStatement<'a>> {
    /// Check if this ExpressionStatement is the body of an arrow function expression
    ///
    /// Example:
    /// `() => expression;`
    ///        ^^^^^^^^^^ This ExpressionStatement is the body of an arrow function
    ///
    /// `() => { return expression; }`
    ///         ^^^^^^^^^^^^^^^^^^^^ This ExpressionStatement is NOT the body of an arrow function
    pub fn is_arrow_function_body(&self) -> bool {
        matches!(self.parent().parent(), AstNodes::ArrowFunctionExpression(arrow) if arrow.expression)
    }
}

impl<'me, 'a> AstNode<'me, 'a, ImportExpression<'a>> {
    /// Converts the arguments of the ImportExpression into an `AstNode` representing a `Vec` of `Argument`.
    ///
    // TODO: Eliminate Allocator usage - see NORTH_STAR.md.
    // Currently synthesises a `Vec` in the arena to reuse `CallExpression`'s argument formatting
    // logic. A future refactor should switch to slice-based iteration so this works without an
    // allocator.
    #[inline]
    pub fn to_arguments<'this>(
        &'this self,
        allocator: &'a Allocator,
    ) -> AstNode<'this, 'a, Vec<'a, Argument<'a>>> {
        // Convert ImportExpression's source and options to Vec<'a, Argument<'me, 'a>>.
        let mut arguments = Vec::new_in(allocator);

        // SAFETY: Argument inherits all Expression variants through the inherit_variants! macro,
        // so Expression and Argument have identical memory layout for shared variants.
        // Both are discriminated unions where each Expression variant (e.g., Expression::Identifier)
        // has a corresponding Argument variant (e.g., Argument::Identifier) with the same discriminant
        // and the same inner type (Box<'a, T>). Transmuting Expression to Argument via transmute_copy
        // is safe because we're just copying the bits (discriminant + pointer).
        unsafe {
            arguments.push(transmute_copy(&self.inner.source));
            if let Some(ref options) = self.inner.options {
                arguments.push(transmute_copy(options));
            }
        }

        let arguments_ref = allocator.alloc(arguments);

        AstNode {
            inner: arguments_ref,
            parent: AstNodes::ImportExpression(self),
            following_span_start: self.following_span_start,
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, CallExpression<'a>> {
    /// Check if the passing span is the callee of this CallExpression
    pub fn is_callee_span(&self, span: Span) -> bool {
        self.inner.callee.span() == span
    }

    /// Check if the passing span is an argument of this CallExpression
    pub fn is_argument_span(&self, span: Span) -> bool {
        !self.is_callee_span(span)
    }
}

impl<'me, 'a> AstNode<'me, 'a, NewExpression<'a>> {
    /// Check if the passing span is the callee of this NewExpression
    pub fn is_callee_span(&self, span: Span) -> bool {
        self.inner.callee.span() == span
    }

    /// Check if the passing span is an argument of this NewExpression
    pub fn is_argument_span(&self, span: Span) -> bool {
        !self.is_callee_span(span)
    }
}
