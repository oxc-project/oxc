use oxc_allocator::{Address, GetAddress};
use oxc_ast::AstKind;
use oxc_data_structures::stack::Stack;
use oxc_syntax::node::NodeId;

/// Stack of ancestors of the node currently being visited.
///
/// This is one of the three independent pieces of state the builder maintains
/// while traversing (the other two being the standalone `NodeId` counter and
/// the optional full [`AstNodes`] store). It does **not** depend on the full
/// node store, so it is available even when that store is not retained.
///
/// It is the *only* ancestry information the syntax checker reads, and exposes
/// an **upward-walk-only** API: you can look at the current node and walk up
/// towards the root. There is no downward/sideways traversal.
///
/// The top of the stack is the node currently being visited. Each frame stores
/// the node's [`AstKind`] (which also carries its [`NodeId`]).
///
/// [`AstNodes`]: super::AstNodes
pub struct AncestryStack<'a> {
    /// Path from the root (`Program`) down to the node currently being visited.
    /// The last element is the current node.
    stack: Stack<AstKind<'a>>,
}

/// Iterator walking up the ancestry stack, yielding [`AstKind`]s.
type AncestorKinds<'s, 'a> = std::iter::Copied<std::iter::Rev<std::slice::Iter<'s, AstKind<'a>>>>;

impl Default for AncestryStack<'_> {
    fn default() -> Self {
        // Reserve enough frames for typical AST nesting depth so the stack does not reallocate
        // while traversing. `Stack`'s own default capacity is only 4 for a 16-byte `AstKind`.
        Self { stack: Stack::with_capacity(64) }
    }
}

impl<'a> AncestryStack<'a> {
    /// Push the node currently being entered onto the stack.
    #[inline]
    pub(crate) fn push(&mut self, kind: AstKind<'a>) {
        self.stack.push(kind);
    }

    /// Pop the current node when leaving it.
    #[inline]
    pub(crate) fn pop(&mut self) {
        self.stack.pop();
    }

    /// [`NodeId`] of the node currently being visited, or [`NodeId::ROOT`] when
    /// the stack is empty.
    ///
    /// Used by the builder to restore `current_node_id` on `leave_node` without
    /// consulting the full node store.
    #[inline]
    pub(crate) fn current_node_id(&self) -> NodeId {
        self.stack.last().map_or(NodeId::ROOT, AstKind::node_id)
    }

    /// [`NodeId`] of the parent of the node currently being visited, or
    /// [`NodeId::ROOT`] when the current node is the root (`Program`).
    #[inline]
    pub(crate) fn parent_node_id(&self) -> NodeId {
        let stack = self.stack.as_slice();
        // `Program` is its own parent.
        let parent_index = stack.len().saturating_sub(2);
        stack.get(parent_index).map_or(NodeId::ROOT, AstKind::node_id)
    }

    /// [`AstKind`] of the node currently being visited (top of the stack).
    ///
    /// # Panics
    /// Panics if the stack is empty (i.e. not inside a node visit).
    #[inline]
    pub fn current_kind(&self) -> AstKind<'a> {
        *self.stack.last().expect("ancestry stack is empty")
    }

    /// [`Address`] of the node currently being visited.
    #[inline]
    pub fn current_address(&self) -> Address {
        self.current_kind().address()
    }

    /// [`AstKind`] of the parent of the node currently being visited.
    ///
    /// Returns the current node's own kind when the current node is the root
    /// (`Program`), as `Program` is its own parent.
    #[inline]
    pub fn parent_kind(&self) -> AstKind<'a> {
        let stack = self.stack.as_slice();
        // `Program` is its own parent.
        let parent_index = stack.len().saturating_sub(2);
        stack[parent_index]
    }

    /// Walk up the stack, yielding each ancestor's [`AstKind`].
    ///
    /// The first kind produced is the parent of the current node; the last is
    /// `Program`.
    #[inline]
    pub fn ancestor_kinds(&self) -> AncestorKinds<'_, 'a> {
        // Skip the current node (last element), walk towards the root.
        let stack = self.stack.as_slice();
        let upto = stack.len().saturating_sub(1);
        stack[..upto].iter().rev().copied()
    }

    /// Walk up the stack starting *at* a chosen node (inclusive), yielding each
    /// [`AstKind`].
    ///
    /// `from` selects the start node:
    /// - `None` starts at the current node (so the first kind is the current
    ///   node, then its ancestors).
    /// - `Some(node_id)` starts at `node_id`, which must be on the current path
    ///   (an ancestor of, or equal to, the node being visited). If not found,
    ///   the returned iterator is empty.
    #[inline]
    pub fn ancestor_kinds_from(&self, from: Option<NodeId>) -> AncestorKinds<'_, 'a> {
        let stack = self.stack.as_slice();
        let end = match from {
            None => stack.len(),
            Some(node_id) => {
                stack.iter().rposition(|kind| kind.node_id() == node_id).map_or(0, |i| i + 1)
            }
        };
        stack[..end].iter().rev().copied()
    }

    /// Find the node with the given [`NodeId`] by walking up the current path,
    /// returning its [`AstKind`].
    ///
    /// Used by checks that resolve `scope -> node` via
    /// `Scoping::get_node_id(scope)` (and similar `class -> node` lookups). The
    /// target is always an ancestor of the node being visited.
    ///
    /// # Panics
    /// Panics if no ancestor has the given `node_id`.
    #[inline]
    pub fn find_kind_by_node_id(&self, node_id: NodeId) -> AstKind<'a> {
        self.stack
            .as_slice()
            .iter()
            .rev()
            .find(|kind| kind.node_id() == node_id)
            .copied()
            .expect("node id not found on the ancestry stack")
    }
}

/// Upward-walking ancestry view used by the syntax checker and binder.
///
/// Dispatches between two backends so the checker/binder code is written once:
///
/// - [`Ancestry::Nodes`] — when the full [`AstNodes`] store is built (linter,
///   mangler), ancestry is read from the store via `current_node_id`. No
///   separate stack is maintained, so this path has no per-node overhead.
/// - [`Ancestry::Stack`] — when the store is *not* built (compiler pipeline),
///   ancestry is read from the [`AncestryStack`] maintained during traversal.
///
/// Either way the surface is upward-walk-only and never exposes random access.
///
/// [`AstNodes`]: super::AstNodes
#[derive(Clone, Copy)]
pub enum Ancestry<'a, 'n> {
    /// Read ancestry from the full node store.
    Nodes { nodes: &'n super::AstNodes<'a>, current_node_id: NodeId },
    /// Read ancestry from the standalone stack.
    Stack(&'n AncestryStack<'a>),
}

impl<'a, 'n> Ancestry<'a, 'n> {
    /// [`AstKind`] of the node currently being visited.
    #[inline]
    pub fn current_kind(self) -> AstKind<'a> {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => nodes.kind(current_node_id),
            Ancestry::Stack(stack) => stack.current_kind(),
        }
    }

    /// [`Address`] of the node currently being visited.
    #[inline]
    pub fn current_address(self) -> Address {
        self.current_kind().address()
    }

    /// [`AstKind`] of the parent of the node currently being visited.
    #[inline]
    pub fn parent_kind(self) -> AstKind<'a> {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => nodes.parent_kind(current_node_id),
            Ancestry::Stack(stack) => stack.parent_kind(),
        }
    }

    /// [`NodeId`] of the parent of the node currently being visited.
    #[inline]
    pub fn parent_node_id(self) -> NodeId {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => nodes.parent_id(current_node_id),
            Ancestry::Stack(stack) => stack.parent_node_id(),
        }
    }

    /// Walk up, yielding each ancestor's [`AstKind`] (parent first, `Program`
    /// last).
    #[inline]
    pub fn ancestor_kinds(self) -> impl Iterator<Item = AstKind<'a>> + Clone + 'n {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => {
                itertools::Either::Left(nodes.ancestor_kinds(current_node_id))
            }
            Ancestry::Stack(stack) => itertools::Either::Right(stack.ancestor_kinds()),
        }
    }

    /// Walk up starting *at* a chosen node (inclusive). `None` starts at the
    /// current node; `Some(node_id)` starts at `node_id` (which must be on the
    /// current path).
    #[inline]
    pub fn ancestor_kinds_from(
        self,
        from: Option<NodeId>,
    ) -> impl Iterator<Item = AstKind<'a>> + Clone + 'n {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => {
                let start = from.unwrap_or(current_node_id);
                itertools::Either::Left(
                    std::iter::once(nodes.kind(start)).chain(nodes.ancestor_kinds(start)),
                )
            }
            Ancestry::Stack(stack) => itertools::Either::Right(stack.ancestor_kinds_from(from)),
        }
    }

    /// Find the node with the given [`NodeId`] and return its [`AstKind`]. The
    /// target is always an ancestor of the node being visited.
    #[inline]
    pub fn find_kind_by_node_id(self, node_id: NodeId) -> AstKind<'a> {
        match self {
            Ancestry::Nodes { nodes, .. } => nodes.kind(node_id),
            Ancestry::Stack(stack) => stack.find_kind_by_node_id(node_id),
        }
    }
}
