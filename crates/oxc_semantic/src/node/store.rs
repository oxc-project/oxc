use oxc_syntax::node::{NodeFlags, NodeId};

use super::{Ancestry, AncestryStack, AstNodes};

/// The node-related state the builder keeps while traversing.
///
/// Bundles the three independent pieces of traversal state — the standalone
/// `NodeId` counter, the cursor/flags of the node currently being visited, and
/// the actual node storage — so the [`SemanticBuilder`] doesn't have to track
/// them as separate fields.
///
/// [`SemanticBuilder`]: crate::SemanticBuilder
pub struct AstNodeStore<'a> {
    /// Standalone `NodeId` counter. The source of truth for node ids, valid even
    /// when the full node store is not built.
    node_count: u32,
    /// `NodeId` of the node currently being visited.
    pub current_node_id: NodeId,
    /// Flags for the node currently being visited.
    pub current_node_flags: NodeFlags,
    /// The actual node storage. See [`AstNodeStoreKind`].
    pub kind: AstNodeStoreKind<'a>,
}

/// The node storage, kept as an enum so the two modes are **mutually exclusive
/// by construction** — the full [`AstNodes`] store and the ancestry stack can
/// never both be maintained at the same time:
///
/// - [`AstNodeStoreKind::Full`] — linter / mangler. The full node store is built
///   and ancestry is read from it.
/// - [`AstNodeStoreKind::Ancestry`] — compiler pipeline. Only the lightweight
///   ancestry stack is maintained; the full store is never built.
pub enum AstNodeStoreKind<'a> {
    /// Full AST node store (linter / mangler).
    Full(AstNodes<'a>),
    /// Ancestry stack only (compiler pipeline).
    Ancestry(AncestryStack<'a>),
}

impl Default for AstNodeStore<'_> {
    fn default() -> Self {
        Self {
            node_count: 0,
            current_node_id: NodeId::new(0),
            current_node_flags: NodeFlags::empty(),
            // Default to the compiler pipeline (ancestry stack, no full store).
            kind: AstNodeStoreKind::Ancestry(AncestryStack::default()),
        }
    }
}

impl<'a> AstNodeStore<'a> {
    /// Switch between the two mutually-exclusive modes. Called at construction
    /// before traversal starts.
    pub fn set_build_nodes(&mut self, yes: bool) {
        self.kind = if yes {
            AstNodeStoreKind::Full(AstNodes::default())
        } else {
            AstNodeStoreKind::Ancestry(AncestryStack::default())
        };
    }

    /// Number of nodes allocated so far. Source of truth, valid in both modes.
    #[inline]
    pub fn node_count(&self) -> u32 {
        self.node_count
    }

    /// Allocate the next [`NodeId`] from the standalone counter.
    #[inline]
    pub fn alloc_node_id(&mut self) -> NodeId {
        let node_id = NodeId::new(self.node_count as usize);
        self.node_count += 1;
        node_id
    }

    /// Reserve capacity in the full node store (no-op in ancestry mode).
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if let AstNodeStoreKind::Full(nodes) = &mut self.kind {
            nodes.reserve(additional);
        }
    }

    /// Upward-walking view of the current node's ancestors. Reads from the full
    /// node store when it is built, otherwise from the standalone ancestry stack.
    #[inline]
    pub fn ancestry(&self) -> Ancestry<'a, '_> {
        match &self.kind {
            AstNodeStoreKind::Full(nodes) => {
                Ancestry::Nodes { nodes, current_node_id: self.current_node_id }
            }
            AstNodeStoreKind::Ancestry(stack) => Ancestry::Stack(stack),
        }
    }

    /// Consume into the full [`AstNodes`] store, or an empty store in ancestry
    /// mode.
    #[inline]
    pub fn into_nodes(self) -> AstNodes<'a> {
        match self.kind {
            AstNodeStoreKind::Full(nodes) => nodes,
            AstNodeStoreKind::Ancestry(_) => AstNodes::default(),
        }
    }
}
