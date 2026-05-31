//! Dynamic AST node storage used by [`SemanticBuilder`] during traversal.
//!
//! Two backends are available:
//!
//! 1. [`NodeStorage::Full`] — every node is recorded in [`AstNodes`], giving
//!    random access by [`NodeId`] after the build finishes. Required by
//!    consumers that walk the whole tree (linter, formatter, mangler).
//! 2. [`NodeStorage::Ancestors`] — only the *live ancestor chain*
//!    (`root..=current`) is retained, as a [`Stack`] pushed on `enter_node` and
//!    popped on `leave_node`. This serves every *upward* query the binder, the
//!    class-table builder, and the syntax checker need (parents and ancestors),
//!    while avoiding the per-node allocations of full storage. Used by pipelines
//!    that discard the AST nodes and keep only [`Scoping`] (transform, minify,
//!    define/inject).
//!
//! Both backends allocate [`NodeId`]s from the same monotonic counter, so the
//! ids stored in [`Scoping`] are identical regardless of the backend.
//!
//! [`SemanticBuilder`]: crate::SemanticBuilder
//! [`Scoping`]: crate::Scoping

use itertools::Either;

use oxc_ast::AstKind;
use oxc_data_structures::stack::Stack;
use oxc_syntax::{
    node::{NodeFlags, NodeId},
    scope::ScopeId,
};

#[cfg(feature = "cfg")]
use oxc_cfg::BlockNodeId;

use super::AstNode;
use crate::node::AstNodes;

/// Initial capacity for the ancestor stack, sized to cover the maximum AST nesting
/// depth of typical code so it does not reallocate during a build.
const ANCESTOR_STACK_CAPACITY: usize = 128;

/// A single entry in the ancestor [`Stack`] of [`NodeStorage::Ancestors`].
pub struct StackEntry<'a> {
    id: NodeId,
    node: AstNode<'a>,
    flags: NodeFlags,
}

/// Dynamic AST node storage. See the [module docs](self).
pub enum NodeStorage<'a> {
    /// Full random-access storage, retained after the build.
    Full(AstNodes<'a>),
    /// Only the live ancestor chain (`root..=current`) is retained during the build.
    Ancestors {
        /// The live ancestor chain: `stack[0]` is the root, `stack.last()` the current node.
        /// Pushed on enter and popped on leave, so it only ever holds the current nesting depth.
        stack: Stack<StackEntry<'a>>,
        /// Total number of nodes created so far. Doubles as the allocator for the next
        /// [`NodeId`] — needed because the stack pops, so its length is the current nesting
        /// depth, not the total node count.
        len: u32,
    },
}

impl Default for NodeStorage<'_> {
    fn default() -> Self {
        Self::ancestor_stack()
    }
}

impl<'a> NodeStorage<'a> {
    /// Create full random-access storage.
    pub fn full() -> Self {
        NodeStorage::Full(AstNodes::default())
    }

    /// Create lightweight ancestor-stack storage.
    ///
    /// No storage is allocated yet (the [`Stack`] starts dangling), so creating one and then
    /// discarding it — when full storage is requested instead — is free. Call
    /// [`reserve`](NodeStorage::reserve) before traversal to pre-size it.
    pub fn ancestor_stack() -> Self {
        NodeStorage::Ancestors { stack: Stack::new(), len: 0 }
    }

    /// Consume the storage, returning the recorded [`AstNodes`].
    ///
    /// In ancestor-stack mode the chain is empty by the end of traversal, so this returns an
    /// empty [`AstNodes`].
    pub fn into_ast_nodes(self) -> AstNodes<'a> {
        match self {
            NodeStorage::Full(nodes) => nodes,
            NodeStorage::Ancestors { .. } => AstNodes::default(),
        }
    }

    pub fn add_node(
        &mut self,
        kind: AstKind<'a>,
        scope_id: ScopeId,
        parent_node_id: NodeId,
        #[cfg(feature = "cfg")] cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        match self {
            NodeStorage::Full(nodes) => nodes.add_node(
                kind,
                scope_id,
                parent_node_id,
                #[cfg(feature = "cfg")]
                cfg_id,
                flags,
            ),
            NodeStorage::Ancestors { stack, len } => {
                let node_id = NodeId::new(*len as usize);
                *len += 1;
                kind.set_node_id(node_id);
                stack.push(StackEntry { id: node_id, node: AstNode::new(kind, scope_id), flags });
                node_id
            }
        }
    }

    pub fn add_program_node(
        &mut self,
        kind: AstKind<'a>,
        scope_id: ScopeId,
        #[cfg(feature = "cfg")] cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        match self {
            NodeStorage::Full(nodes) => nodes.add_program_node(
                kind,
                scope_id,
                #[cfg(feature = "cfg")]
                cfg_id,
                flags,
            ),
            NodeStorage::Ancestors { stack, len } => {
                debug_assert!(stack.is_empty(), "Program node must be the first node in the AST.");
                let node_id = NodeId::new(*len as usize);
                *len += 1;
                debug_assert_eq!(node_id, NodeId::ROOT);
                kind.set_node_id(node_id);
                stack.push(StackEntry { id: node_id, node: AstNode::new(kind, scope_id), flags });
                node_id
            }
        }
    }

    /// Pop the current node and return its parent's id.
    #[inline]
    pub fn pop_node(&mut self, current_node_id: NodeId) -> NodeId {
        match self {
            NodeStorage::Full(nodes) => nodes.parent_id(current_node_id),
            NodeStorage::Ancestors { stack, .. } => {
                stack.pop();
                stack.last().map_or(NodeId::ROOT, |entry| entry.id)
            }
        }
    }

    #[inline]
    pub fn get_node(&self, id: NodeId) -> &AstNode<'a> {
        match self {
            NodeStorage::Full(nodes) => nodes.get_node(id),
            NodeStorage::Ancestors { stack, .. } => &stack[position(stack, id)].node,
        }
    }

    #[inline]
    pub fn kind(&self, id: NodeId) -> AstKind<'a> {
        self.get_node(id).kind()
    }

    #[inline]
    pub fn parent_id(&self, id: NodeId) -> NodeId {
        match self {
            NodeStorage::Full(nodes) => nodes.parent_id(id),
            NodeStorage::Ancestors { stack, .. } => {
                let pos = position(stack, id);
                if pos == 0 { NodeId::ROOT } else { stack[pos - 1].id }
            }
        }
    }

    #[inline]
    pub fn parent_kind(&self, id: NodeId) -> AstKind<'a> {
        self.kind(self.parent_id(id))
    }

    #[inline]
    pub fn parent_node(&self, id: NodeId) -> &AstNode<'a> {
        self.get_node(self.parent_id(id))
    }

    #[inline]
    pub fn flags_mut(&mut self, id: NodeId) -> &mut NodeFlags {
        match self {
            NodeStorage::Full(nodes) => nodes.flags_mut(id),
            NodeStorage::Ancestors { stack, .. } => {
                let pos = position(stack, id);
                &mut stack[pos].flags
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            NodeStorage::Full(nodes) => nodes.len(),
            NodeStorage::Ancestors { len, .. } => *len as usize,
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        match self {
            // Full storage grows with the total node count.
            NodeStorage::Full(nodes) => nodes.reserve(additional),
            // The ancestor stack only ever holds the live `root..=current` chain, so it
            // pre-sizes to a typical maximum nesting depth rather than the node count.
            NodeStorage::Ancestors { stack, len } => {
                debug_assert!(*len == 0, "`reserve` must be called before traversal");
                *stack = Stack::with_capacity(ANCESTOR_STACK_CAPACITY);
            }
        }
    }

    /// Walk up the AST, iterating over each parent [`NodeId`].
    ///
    /// The first id produced is the parent of `id`; the last is always the root [`Program`].
    ///
    /// [`Program`]: oxc_ast::ast::Program
    #[inline]
    pub fn ancestor_ids(&self, id: NodeId) -> impl Iterator<Item = NodeId> + Clone + '_ {
        match self {
            NodeStorage::Full(nodes) => Either::Left(nodes.ancestor_ids(id)),
            NodeStorage::Ancestors { stack, .. } => Either::Right(StackAncestorIdsIter {
                stack: stack.as_slice(),
                index: position(stack, id),
            }),
        }
    }

    /// Walk up the AST, iterating over each parent [`AstKind`].
    #[inline]
    pub fn ancestor_kinds(&self, id: NodeId) -> impl Iterator<Item = AstKind<'a>> + Clone + '_ {
        self.ancestor_ids(id).map(move |id| self.kind(id))
    }

    /// Walk up the AST, iterating over each parent [`AstNode`].
    #[inline]
    pub fn ancestors(&self, id: NodeId) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        self.ancestor_ids(id).map(move |id| self.get_node(id))
    }

    /// Walk up the AST, iterating over each parent [`NodeId`] and [`AstNode`].
    #[inline]
    pub fn ancestors_enumerated(
        &self,
        id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &AstNode<'a>)> + Clone + '_ {
        self.ancestor_ids(id).map(move |id| (id, self.get_node(id)))
    }
}

/// Find the stack position of a live ancestor `id` in [`NodeStorage::Ancestors`].
///
/// The current node (top of stack) is the overwhelmingly common case and is checked first.
/// Other ancestors (e.g. a scope's or class's node) require a scan, but the stack depth
/// equals the AST nesting depth, which is small.
#[inline]
fn position<'a>(stack: &Stack<StackEntry<'a>>, id: NodeId) -> usize {
    if let Some(last) = stack.last()
        && last.id == id
    {
        return stack.len() - 1;
    }
    stack
        .iter()
        .rposition(|entry| entry.id == id)
        .expect("`NodeId` is not a live ancestor (not available in parent-pointer storage mode)")
}

/// Iterator over the ids of a node's ancestors in [`NodeStorage::Ancestors`].
///
/// Yields the parent first and the root ([`Program`]) last, matching the order used by full
/// [`AstNodes`] storage.
///
/// [`Program`]: oxc_ast::ast::Program
#[derive(Clone)]
pub struct StackAncestorIdsIter<'n, 'a> {
    stack: &'n [StackEntry<'a>],
    /// Position of the node whose ancestors are being yielded. Walks downward.
    index: usize,
}

impl Iterator for StackAncestorIdsIter<'_, '_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            // Root has no parent.
            return None;
        }
        self.index -= 1;
        Some(self.stack[self.index].id)
    }
}
