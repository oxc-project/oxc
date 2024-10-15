use oxc_ast::AstKind;
use oxc_cfg::BlockNodeId;
use oxc_index::IndexVec;
use oxc_span::GetSpan;
pub use oxc_syntax::node::{NodeFlags, NodeId};

use crate::scope::ScopeId;

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
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_semantic::AstNode;
    ///
    /// fn get_function_name<'a>(node: AstNode<'a>) -> Option<&'a str> {
    ///     match node.kind() {
    ///        AstKind::Function(func) => Some(func.name()),
    ///        _ => None,
    ///     }
    /// }
    /// ```
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
    fn span(&self) -> oxc_span::Span {
        self.kind.span()
    }
}

/// Untyped AST nodes flattened into an vec
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    /// The root node should always point to a `Program`, which is the real
    /// root of the tree. It isn't possible to statically check for this, so
    /// users should beware.
    root: Option<NodeId>,
    nodes: IndexVec<NodeId, AstNode<'a>>,
    /// `node` -> `parent`
    parent_ids: IndexVec<NodeId, Option<NodeId>>,
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

    /// Walk up the AST, iterating over each parent node.
    ///
    /// The first node produced by this iterator is the first parent of the node
    /// pointed to by `node_id`. The last node will usually be a `Program`.
    #[inline]
    pub fn iter_parents(&self, node_id: NodeId) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        AstNodeParentIter { current_node_id: Some(node_id), nodes: self }
    }

    /// Access the underlying struct from [`oxc_ast`].
    ///
    /// ## Example
    ///
    /// ```
    /// use oxc_semantic::AstNodes;
    /// use oxc_ast::AstKind;
    ///
    /// let ast: AstNodes<'_> = get_ast_in_some_way();
    /// assert!(matches!(
    ///     ast.kind(ast.root().unwrap()),
    ///     AstKind::Program(_)
    /// ));
    /// ```
    #[inline]
    pub fn kind(&self, node_id: NodeId) -> AstKind<'a> {
        self.nodes[node_id].kind
    }

    /// Get id of this node's parent.
    #[inline]
    pub fn parent_id(&self, node_id: NodeId) -> Option<NodeId> {
        self.parent_ids[node_id]
    }

    /// Get the kind of the parent node.
    pub fn parent_kind(&self, node_id: NodeId) -> Option<AstKind<'a>> {
        self.parent_id(node_id).map(|node_id| self.kind(node_id))
    }

    /// Get a reference to a node's parent.
    pub fn parent_node(&self, node_id: NodeId) -> Option<&AstNode<'a>> {
        self.parent_id(node_id).map(|node_id| self.get_node(node_id))
    }

    #[inline]
    pub fn get_node(&self, node_id: NodeId) -> &AstNode<'a> {
        &self.nodes[node_id]
    }

    #[inline]
    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut AstNode<'a> {
        &mut self.nodes[node_id]
    }

    /// Get the root [`NodeId`]. This always points to a [`Program`] node.
    ///
    /// Returns [`None`] if root node isn't set. This will never happen if you
    /// are obtaining an [`AstNodes`] that has already been constructed.
    ///
    /// [`Program`]: oxc_ast::ast::Program
    #[inline]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Get the root node as immutable reference, It is always guaranteed to be a [`Program`].
    ///
    /// Returns [`None`] if root node isn't set. This will never happen if you
    /// are obtaining an [`AstNodes`] that has already been constructed.
    ///
    /// [`Program`]: oxc_ast::ast::Program
    #[inline]
    pub fn root_node(&self) -> Option<&AstNode<'a>> {
        self.root().map(|id| self.get_node(id))
    }

    /// Get the root node as mutable reference, It is always guaranteed to be a [`Program`].
    ///
    /// Returns [`None`] if root node isn't set. This will never happen if you
    /// are obtaining an [`AstNodes`] that has already been constructed.
    ///
    /// [`Program`]: oxc_ast::ast::Program
    #[inline]
    pub fn root_node_mut(&mut self) -> Option<&mut AstNode<'a>> {
        self.root().map(|id| self.get_node_mut(id))
    }

    /// Walk up the AST, iterating over each parent node.
    ///
    /// The first node produced by this iterator is the first parent of the node
    /// pointed to by `node_id`. The last node will always be a [`Program`].
    ///
    /// [`Program`]: oxc_ast::ast::Program
    pub fn ancestors(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let parent_ids = &self.parent_ids;
        std::iter::successors(Some(node_id), |&node_id| parent_ids[node_id])
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
        let node_id = self.parent_ids.push(Some(parent_node_id));
        let node = AstNode::new(kind, scope_id, cfg_id, flags, node_id);
        self.nodes.push(node);
        node_id
    }

    /// Create and add an [`AstNode`] to the [`AstNodes`] tree and get its [`NodeId`].
    pub fn add_program_node(
        &mut self,
        kind: AstKind<'a>,
        scope_id: ScopeId,
        cfg_id: BlockNodeId,
        flags: NodeFlags,
    ) -> NodeId {
        let node_id = self.parent_ids.push(None);
        self.root = Some(node_id);
        let node = AstNode::new(kind, scope_id, cfg_id, flags, node_id);
        self.nodes.push(node);
        node_id
    }

    /// Reserve space for at least `additional` more nodes.
    pub fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
        self.parent_ids.reserve(additional);
    }
}

impl<'a, 'n> IntoIterator for &'n AstNodes<'a> {
    type Item = &'n AstNode<'a>;
    type IntoIter = std::slice::Iter<'n, AstNode<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

#[derive(Debug, Clone)]
pub struct AstNodeParentIter<'s, 'a> {
    current_node_id: Option<NodeId>,
    nodes: &'s AstNodes<'a>,
}

impl<'s, 'a> Iterator for AstNodeParentIter<'s, 'a> {
    type Item = &'s AstNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node_id) = self.current_node_id {
            self.current_node_id = self.nodes.parent_ids[node_id];
            Some(self.nodes.get_node(node_id))
        } else {
            None
        }
    }
}
