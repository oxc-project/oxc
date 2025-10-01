use std::iter::FusedIterator;

use oxc_ast::{AstKind, ast::Program};
use oxc_index::{IndexSlice, IndexVec};
use oxc_syntax::{
    node::{NodeFlags, NodeId},
    scope::ScopeId,
};

#[cfg(feature = "linter")]
use oxc_ast::AstType;

#[cfg(feature = "cfg")]
use oxc_cfg::BlockNodeId;

use super::AstNode;

#[cfg(feature = "linter")]
use crate::ast_types_bitset::AstTypesBitset;

/// Untyped AST nodes flattened into an vec
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    nodes: IndexVec<NodeId, AstNode<'a>>,
    /// `node` -> `parent`
    parent_ids: IndexVec<NodeId, NodeId>,
    /// `node` -> `flags`
    flags: IndexVec<NodeId, NodeFlags>,
    /// `node` -> `cfg_id` (control flow graph node)
    #[cfg(feature = "cfg")]
    cfg_ids: IndexVec<NodeId, BlockNodeId>,
    /// Stores a set of bits of a fixed size, where each bit represents a single [`AstKind`]. If the bit is set (1),
    /// then the AST contains at least one node of that kind. If the bit is not set (0), then the AST does not contain
    /// any nodes of that kind.
    #[cfg(feature = "linter")]
    node_kinds_set: AstTypesBitset,
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
        self.nodes[node_id].kind()
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

    /// Get flags for a node.
    #[inline]
    pub fn flags(&self, node_id: NodeId) -> NodeFlags {
        self.flags[node_id]
    }

    /// Get a mutable reference to a node's flags.
    #[inline]
    pub fn flags_mut(&mut self, node_id: NodeId) -> &mut NodeFlags {
        &mut self.flags[node_id]
    }

    /// ID of the control flow graph node this node is in.
    ///
    /// See [oxc_cfg::ControlFlowGraph] for more information.
    #[inline]
    #[cfg(feature = "cfg")]
    pub fn cfg_id(&self, node_id: NodeId) -> BlockNodeId {
        self.cfg_ids[node_id]
    }

    /// Get the [`Program`] that's also the root of the AST.
    #[inline]
    pub fn program(&self) -> &'a Program<'a> {
        if let Some(node) = self.nodes.first()
            && let AstKind::Program(program) = node.kind()
        {
            return program;
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
        #[cfg(feature = "cfg")] cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        let node_id = self.parent_ids.push(parent_node_id);
        let node = AstNode::new(kind, scope_id, node_id);
        self.nodes.push(node);
        self.flags.push(flags);
        #[cfg(feature = "cfg")]
        self.cfg_ids.push(cfg_id);
        #[cfg(feature = "linter")]
        self.node_kinds_set.set(kind.ty());
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
        #[cfg(feature = "cfg")] cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        assert!(self.parent_ids.is_empty(), "Program node must be the first node in the AST.");
        debug_assert!(
            matches!(kind, AstKind::Program(_)),
            "Program node must be of kind `AstKind::Program`"
        );
        self.parent_ids.push(NodeId::ROOT);
        self.nodes.push(AstNode::new(kind, scope_id, NodeId::ROOT));
        self.flags.push(flags);
        #[cfg(feature = "cfg")]
        self.cfg_ids.push(cfg_id);
        #[cfg(feature = "linter")]
        self.node_kinds_set.set(AstType::Program);
        NodeId::ROOT
    }

    /// Reserve space for at least `additional` more nodes.
    pub fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
        self.parent_ids.reserve(additional);
        self.flags.reserve(additional);
        #[cfg(feature = "cfg")]
        self.cfg_ids.reserve(additional);
    }

    /// Checks if the AST contains any nodes of the given types.
    ///
    /// ## Example
    /// ```
    /// # fn get_nodes<'a>() -> AstNodes<'a> { AstNodes::default() }
    ///
    /// use oxc_ast::AstType;
    /// use oxc_semantic::{AstNodes, AstTypesBitset};
    ///
    /// let for_stmt = AstTypesBitset::from_types(&[AstType::ForStatement]);
    /// let import_export_decl = AstTypesBitset::from_types(&[
    ///   AstType::ImportDeclaration,
    ///   AstType::ExportNamedDeclaration,
    /// ]);
    ///
    /// let nodes: AstNodes = get_nodes();
    /// // `true` if there is a `for` loop anywhere in the AST
    /// nodes.contains_any(&for_stmt);
    /// // `true` if there is at least one import OR one export in the AST
    /// nodes.contains_any(&import_export_decl);
    /// ```
    #[cfg(feature = "linter")]
    pub fn contains_any(&self, bitset: &AstTypesBitset) -> bool {
        self.node_kinds_set.intersects(bitset)
    }

    /// Checks if the AST contains all of the given types.
    ///
    /// ## Example
    /// ```
    /// # fn get_nodes<'a>() -> AstNodes<'a> { AstNodes::default() }
    ///
    /// use oxc_ast::AstType;
    /// use oxc_semantic::{AstNodes, AstTypesBitset};
    ///
    /// let for_stmt = AstTypesBitset::from_types(&[AstType::ForStatement]);
    /// let import_export_decl = AstTypesBitset::from_types(&[
    ///   AstType::ImportDeclaration,
    ///   AstType::ExportNamedDeclaration,
    /// ]);
    ///
    /// let nodes: AstNodes = get_nodes();
    /// // `true` if there is a `for` loop anywhere in the AST
    /// nodes.contains_all(&for_stmt);
    /// // `true` if there is at least one import AND one export in the AST
    /// nodes.contains_all(&import_export_decl);
    /// ```
    #[cfg(feature = "linter")]
    pub fn contains_all(&self, bitset: &AstTypesBitset) -> bool {
        self.node_kinds_set.contains(bitset)
    }

    /// Checks if the AST contains a node of the given type.
    ///
    /// ## Example
    /// ```
    /// # fn get_nodes<'a>() -> AstNodes<'a> { AstNodes::default() }
    ///
    /// use oxc_ast::AstType;
    /// use oxc_semantic::{AstNodes, AstTypesBitset};
    ///
    /// let nodes: AstNodes = get_nodes();
    /// // `true` if there is a `for` loop anywhere in the AST
    /// nodes.contains(AstType::ForStatement);
    /// // `true` if there is an `ImportDeclaration` anywhere in the AST
    /// nodes.contains(AstType::ImportDeclaration);
    /// ```
    #[cfg(feature = "linter")]
    pub fn contains(&self, ty: AstType) -> bool {
        self.node_kinds_set.has(ty)
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
