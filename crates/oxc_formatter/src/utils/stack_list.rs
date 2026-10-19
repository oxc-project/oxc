//! A list whose items live on the call stack of a recursive walk.

use std::iter;

/// An item of a recursive walk down the AST, linked to the item of the caller's frame.
///
/// An [`AstNode`](crate::ast_nodes::AstNode) refers to its parent on the stack,
/// so a walk that needs every node of a chain at once (e.g. a member chain)
/// keeps each one in its own frame and links them, instead of collecting them in a loop.
pub struct StackList<'s, T> {
    pub item: T,
    pub outer: Option<&'s StackList<'s, T>>,
}

impl<'s, T> StackList<'s, T> {
    pub fn new(item: T, outer: Option<&'s StackList<'s, T>>) -> Self {
        Self { item, outer }
    }

    /// Returns the items from the innermost (this one) to the outermost.
    pub fn iter(&'s self) -> impl Iterator<Item = &'s T> {
        iter::successors(Some(self), |list| list.outer).map(|list| &list.item)
    }
}
