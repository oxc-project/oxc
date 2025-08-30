use std::iter::FusedIterator;

use oxc_allocator::{Address, GetAddress};
use oxc_ast::{AstKind, ast::Program};
use oxc_cfg::BlockNodeId;
use oxc_index::{IndexSlice, IndexVec};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    node::{NodeFlags, NodeId},
    scope::ScopeId,
};

/// Semantic node contains all the semantic information about an ast node.
#[derive(Debug, Clone, Copy)]
pub struct AstNode<'a> {
    id: NodeId,
    /// A pointer to the ast node, which resides in the `bumpalo` memory arena.
    kind: AstKind<'a>,

    /// Associated Scope (initialized by binding)
    scope_id: ScopeId,

    /// Associated `BasicBlockId` in CFG (initialized by control_flow)
    cfg_id: BlockNodeId,

    flags: NodeFlags,
}

impl<'a> AstNode<'a> {
    #[inline]
    pub(crate) fn new(
        kind: AstKind<'a>,
        scope_id: ScopeId,
        cfg_id: BlockNodeId,
        flags: NodeFlags,
        id: NodeId,
    ) -> Self {
        Self { id, kind, scope_id, cfg_id, flags }
    }

    /// This node's unique identifier.
    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// ID of the control flow graph node this node is in.
    ///
    /// See [oxc_cfg::ControlFlowGraph] for more information.
    #[inline]
    pub fn cfg_id(&self) -> BlockNodeId {
        self.cfg_id
    }

    /// Access the underlying struct from [`oxc_ast`].
    #[inline]
    pub fn kind(&self) -> AstKind<'a> {
        self.kind
    }

    /// The scope in which this node was declared.
    ///
    /// It is important to note that this is _not_ the scope created _by_ the
    /// node. For example, given a function declaration, this is the scope where
    /// the function is declared, not the scope created by its body.
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    /// Flags providing additional information about the node.
    #[inline]
    pub fn flags(&self) -> NodeFlags {
        self.flags
    }

    /// Get a mutable reference to this node's flags.
    #[inline]
    pub fn flags_mut(&mut self) -> &mut NodeFlags {
        &mut self.flags
    }
}

impl GetSpan for AstNode<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.kind.span()
    }
}

impl GetAddress for AstNode<'_> {
    #[inline]
    fn address(&self) -> Address {
        self.kind.address()
    }
}

/// Untyped AST nodes flattened into an vec
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    nodes: IndexVec<NodeId, AstNode<'a>>,
    /// `node` -> `parent`
    parent_ids: IndexVec<NodeId, NodeId>,
}

impl<'a> AstNodes<'a> {
    /// Iterate over all [`AstNode`]s in this AST.
    pub fn iter(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.nodes.iter()
    }

    /// Returns the number of node in this AST.
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if there are no nodes in this AST.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Walk up the AST, iterating over each parent [`NodeId`].
    ///
    /// The first node produced by this iterator is the parent of `node_id`.
    /// The last node will always be [`AstKind::Program`].
    #[inline]
    pub fn ancestor_ids(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> + Clone + '_ {
        AstNodeIdAncestorsIter::new(node_id, self)
    }

    /// Walk up the AST, iterating over each parent [`AstKind`].
    ///
    /// The first node produced by this iterator is the parent of `node_id`.
    /// The last node will always be [`AstKind::Program`].
    #[inline]
    pub fn ancestor_kinds(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = AstKind<'a>> + Clone + '_ {
        self.ancestor_ids(node_id).map(|id| self.kind(id))
    }

    /// Walk up the AST, iterating over each parent [`AstNode`].
    ///
    /// The first node produced by this iterator is the parent of `node_id`.
    /// The last node will always be [`AstKind::Program`].
    #[inline]
    pub fn ancestors(&self, node_id: NodeId) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        self.ancestor_ids(node_id).map(|id| self.get_node(id))
    }

    /// Access the underlying struct from [`oxc_ast`].
    #[inline]
    pub fn kind(&self, node_id: NodeId) -> AstKind<'a> {
        self.nodes[node_id].kind
    }

    /// Get id of this node's parent.
    #[inline]
    pub fn parent_id(&self, node_id: NodeId) -> NodeId {
        self.parent_ids[node_id]
    }

    /// Get the kind of the parent node.
    pub fn parent_kind(&self, node_id: NodeId) -> AstKind<'a> {
        self.kind(self.parent_id(node_id))
    }

    /// Get a reference to a node's parent.
    pub fn parent_node(&self, node_id: NodeId) -> &AstNode<'a> {
        self.get_node(self.parent_id(node_id))
    }

    #[inline]
    pub fn get_node(&self, node_id: NodeId) -> &AstNode<'a> {
        &self.nodes[node_id]
    }

    #[inline]
    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut AstNode<'a> {
        &mut self.nodes[node_id]
    }

    /// Get the [`Program`] that's also the root of the AST.
    #[inline]
    pub fn program(&self) -> &'a Program<'a> {
        if let Some(node) = self.nodes.first() {
            if let AstKind::Program(program) = node.kind {
                return program;
            }
        }

        unreachable!();
    }

    /// Create and add an [`AstNode`] to the [`AstNodes`] tree and get its [`NodeId`].
    /// Node must not be [`Program`]; if it is, use [`add_program_node`] instead.
    ///
    /// [`Program`]: oxc_ast::ast::Program
    /// [`add_program_node`]: AstNodes::add_program_node
    #[inline]
    pub fn add_node(
        &mut self,
        kind: AstKind<'a>,
        scope_id: ScopeId,
        parent_node_id: NodeId,
        cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        let node_id = self.parent_ids.push(parent_node_id);
        let node = AstNode::new(kind, scope_id, cfg_id, flags, node_id);
        self.nodes.push(node);
        node_id
    }

    /// Create and add an [`AstNode`] to the [`AstNodes`] tree and get its [`NodeId`].
    ///
    /// # Panics
    ///
    /// Panics if this is not the first node being added to the AST.
    pub fn add_program_node(
        &mut self,
        kind: AstKind<'a>,
        scope_id: ScopeId,
        cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        assert!(self.parent_ids.is_empty(), "Program node must be the first node in the AST.");
        debug_assert!(
            matches!(kind, AstKind::Program(_)),
            "Program node must be of kind `AstKind::Program`"
        );
        self.parent_ids.push(NodeId::ROOT);
        self.nodes.push(AstNode::new(kind, scope_id, cfg_id, flags, NodeId::ROOT));
        NodeId::ROOT
    }

    /// Reserve space for at least `additional` more nodes.
    pub fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
        self.parent_ids.reserve(additional);
    }
}

impl<'a, 'n> IntoIterator for &'n AstNodes<'a> {
    type IntoIter = std::slice::Iter<'n, AstNode<'a>>;
    type Item = &'n AstNode<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

/// Iterator over ancestors of an AST node, starting with the node itself.
///
/// Yields `NodeId` of each AST node. The last node yielded is `Program`.
#[derive(Debug, Clone)]
pub struct AstNodeIdAncestorsIter<'n> {
    current_node_id: NodeId,
    parent_ids: &'n IndexSlice<NodeId, [NodeId]>,
}

impl<'n> AstNodeIdAncestorsIter<'n> {
    fn new(node_id: NodeId, nodes: &'n AstNodes<'_>) -> Self {
        Self { current_node_id: node_id, parent_ids: nodes.parent_ids.as_slice() }
    }
}

impl Iterator for AstNodeIdAncestorsIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_node_id == NodeId::ROOT {
            // `Program`'s parent is itself, so next node is `None` if this node is `Program`
            return None;
        }

        self.current_node_id = self.parent_ids[self.current_node_id];
        Some(self.current_node_id)
    }
}

impl FusedIterator for AstNodeIdAncestorsIter<'_> {}
