//! The ancestor stack: the nodes whose `fmt` is running, from the root to the innermost.
//!
//! A node handle ([`AstNode`](super::AstNode)) is only a reference,
//! its ancestors are the stack's entries below it (or, for a child about to be formatted, the whole stack).
//! The generated `fmt` of every node with a kind pushes it while it runs,
//! and a node printed outside its parent's `fmt` (a flattened chain, a deferred list) needs its ancestors pushed by the printer.
//! Debug builds assert that every pushed node and every answered query is a child of the node below it.

use oxc_data_structures::stack::NonEmptyStack;

use super::AstNodes;

/// The start of the node following a child, see [`AstNodes::following_of_child`].
#[derive(Debug, Clone, Copy)]
pub enum Following {
    Start(u32),
    /// The parent's own following node.
    Inherit,
}

impl Following {
    /// `self`, or `inherited` for [`Following::Inherit`].
    #[inline]
    pub fn or_inherited(self, inherited: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Inherit => inherited(),
            start @ Self::Start(_) => start,
        }
    }
}

/// The ancestor stack, see the module docs.
pub struct Ancestors<'a> {
    /// The bottom entry is `Dummy`, the parent of the root.
    stack: NonEmptyStack<AstNodes<'a>>,
}

impl Default for Ancestors<'_> {
    fn default() -> Self {
        Self { stack: NonEmptyStack::new(AstNodes::Dummy()) }
    }
}

impl<'a> Ancestors<'a> {
    #[inline]
    pub fn push(&mut self, node: AstNodes<'a>) {
        debug_assert!(
            is_child_of(node, *self.stack.last()),
            "{} at {:?} is not a child of {}",
            node.debug_name(),
            node.span(),
            self.stack.last().debug_name(),
        );
        self.stack.push(node);
    }

    /// Pushes a node that was on top before, see [`JsFormatterExt::outside_current_node`](crate::formatter::JsFormatterExt::outside_current_node).
    #[inline]
    pub fn push_unchecked(&mut self, node: AstNodes<'a>) {
        self.stack.push(node);
    }

    #[inline]
    pub fn pop(&mut self) -> AstNodes<'a> {
        self.stack.pop()
    }

    /// The innermost node being formatted.
    #[inline]
    pub fn current(&self) -> AstNodes<'a> {
        *self.stack.last()
    }

    /// The parent of the current node.
    #[inline]
    pub fn parent_of_current(&self) -> AstNodes<'a> {
        let stack = self.stack.as_slice();
        stack[stack.len() - 2]
    }

    /// [`Self::following_span_start`] of the current node.
    pub fn following_span_start_of_current(&self) -> u32 {
        let current = self.current();
        self.resolve_following(self.stack.len() - 1, current.address(), current.span().start)
    }

    /// The index of `node` on the stack, or the stack's length for a child of the current node.
    fn position(&self, node: AstNodes<'a>) -> usize {
        let stack = self.stack.as_slice();
        let last = stack.len() - 1;
        if stack[last].is(&node) {
            return last;
        }
        let span = node.span();
        for index in (1..last).rev() {
            // Every entry above `node` is its descendant, ending within it
            // (not starting: decorators can precede `export`),
            // so one ending after it means `node` is not below
            if stack[index + 1].span().end > span.end {
                break;
            }
            if stack[index].is(&node) {
                return index;
            }
        }
        debug_assert!(
            is_child_of(node, stack[last]),
            "{} at {:?} is neither on the ancestor stack nor a child of {} at {:?}",
            node.debug_name(),
            span,
            stack[last].debug_name(),
            stack[last].span(),
        );
        stack.len()
    }

    /// The parent of `node`, which is on the stack or a child of the current node.
    #[inline]
    pub fn parent_of(&self, node: AstNodes<'a>) -> AstNodes<'a> {
        let stack = self.stack.as_slice();
        if stack[stack.len() - 1].is(&node) {
            return stack[stack.len() - 2];
        }
        stack[self.position(node) - 1]
    }

    /// `node` and its ancestors, ending at the root.
    pub fn ancestors_of(
        &self,
        node: AstNodes<'a>,
    ) -> impl Iterator<Item = AstNodes<'a>> + use<'_, 'a> {
        let position = self.position(node);
        let ancestors = &self.stack.as_slice()[1..position];
        std::iter::once(node).chain(ancestors.iter().rev().copied())
    }

    /// The start of the node following `node` (0 if none), from its parent's fields:
    /// the next one holding a node, or else the parent's own following node, up to the root.
    pub fn following_span_start(&self, node: AstNodes<'a>) -> u32 {
        let position = self.position(node);
        self.resolve_following(position, node.address(), node.span().start)
    }

    /// [`Self::following_span_start`] of a child of the current node which has no kind, so no [`AstNodes`].
    pub fn following_span_start_of_child(&self, child: usize, child_start: u32) -> u32 {
        self.resolve_following(self.stack.len(), child, child_start)
    }

    fn resolve_following(
        &self,
        mut position: usize,
        mut child: usize,
        mut child_start: u32,
    ) -> u32 {
        let stack = self.stack.as_slice();
        loop {
            let parent = stack[position - 1];
            match parent.following_of_child(child, child_start) {
                Some(Following::Start(start)) => return start,
                Some(Following::Inherit) => {
                    position -= 1;
                    child = parent.address();
                    child_start = parent.span().start;
                }
                None => {
                    debug_assert!(false, "the ancestor stack is not a path");
                    return 0;
                }
            }
        }
    }
}

/// Whether `node` is a child of `parent`, as the fields of `parent` (`Dummy` is the parent of any root).
fn is_child_of(node: AstNodes<'_>, parent: AstNodes<'_>) -> bool {
    matches!(parent, AstNodes::Dummy())
        || parent.following_of_child(node.address(), node.span().start).is_some()
}
