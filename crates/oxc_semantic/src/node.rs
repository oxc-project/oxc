#![allow(non_upper_case_globals)] // for bitflags

use bitflags::bitflags;
use oxc_ast::AstKind;
use oxc_index::{define_index_type, IndexVec};

use crate::scope::{ScopeFlags, ScopeId};

define_index_type! {
    pub struct AstNodeId = usize;
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NodeFlags: u8 {
        const JSDoc    = 1 << 0; // If the Node has a JSDoc comment attached
        const Class    = 1 << 1; // If Node is inside a class
        const HasYield = 1 << 2; // If function has yield statement

    }
}

impl NodeFlags {
    pub fn has_jsdoc(&self) -> bool {
        self.contains(Self::JSDoc)
    }

    pub fn has_class(&self) -> bool {
        self.contains(Self::Class)
    }

    pub fn has_yield(&self) -> bool {
        self.contains(Self::HasYield)
    }
}

/// Semantic node contains all the semantic information about an ast node.
#[derive(Debug, Clone, Copy)]
pub struct AstNode<'a> {
    id: AstNodeId,
    /// A pointer to the ast node, which resides in the `bumpalo` memory arena.
    kind: AstKind<'a>,

    /// Associated Scope (initialized by binding)
    scope_id: ScopeId,

    flags: NodeFlags,
}

impl<'a> AstNode<'a> {
    pub fn new(kind: AstKind<'a>, scope_id: ScopeId, flags: NodeFlags) -> Self {
        Self { id: AstNodeId::new(0), kind, scope_id, flags }
    }

    pub fn id(&self) -> AstNodeId {
        self.id
    }

    pub fn kind(&self) -> AstKind<'a> {
        self.kind
    }

    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    pub fn flags(&self) -> NodeFlags {
        self.flags
    }

    pub fn flags_mut(&mut self) -> &mut NodeFlags {
        &mut self.flags
    }

    pub fn strict_mode(&self, flags: ScopeFlags) -> bool {
        // All parts of a ClassDeclaration or a ClassExpression are strict mode code.
        flags.is_strict_mode() || self.flags.has_class()
    }

    pub fn is_specific_id_reference(&self, name: &str) -> bool {
        self.kind().is_specific_id_reference(name)
    }
}

/// Untyped AST nodes flattened into an vec
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    nodes: IndexVec<AstNodeId, AstNode<'a>>,
    parent_ids: IndexVec<AstNodeId, Option<AstNodeId>>,
}

impl<'a> AstNodes<'a> {
    pub fn iter(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.nodes.iter()
    }

    /// Walk up the AST, iterating over each parent node.
    ///
    /// The first node produced by this iterator is the first parent of the node
    /// pointed to by `node_id`. The last node will usually be a `Program`.
    pub fn iter_parents(&self, node_id: AstNodeId) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        let curr = Some(self.get_node(node_id));
        AstNodeParentIter { curr, nodes: self }
    }

    pub fn kind(&self, ast_node_id: AstNodeId) -> AstKind<'a> {
        self.nodes[ast_node_id].kind
    }

    pub fn parent_id(&self, ast_node_id: AstNodeId) -> Option<AstNodeId> {
        self.parent_ids[ast_node_id]
    }

    pub fn parent_kind(&self, ast_node_id: AstNodeId) -> Option<AstKind<'a>> {
        self.parent_id(ast_node_id).map(|node_id| self.kind(node_id))
    }

    pub fn parent_node(&self, ast_node_id: AstNodeId) -> Option<&AstNode<'a>> {
        self.parent_id(ast_node_id).map(|node_id| self.get_node(node_id))
    }

    pub fn get_node(&self, ast_node_id: AstNodeId) -> &AstNode<'a> {
        &self.nodes[ast_node_id]
    }

    pub fn get_node_mut(&mut self, ast_node_id: AstNodeId) -> &mut AstNode<'a> {
        &mut self.nodes[ast_node_id]
    }

    /// Walk up the AST, iterating over each parent node.
    ///
    /// The first node produced by this iterator is the first parent of the node
    /// pointed to by `node_id`. The last node will usually be a `Program`.
    pub fn ancestors(&self, ast_node_id: AstNodeId) -> impl Iterator<Item = AstNodeId> + '_ {
        let parent_ids = &self.parent_ids;
        std::iter::successors(Some(ast_node_id), |node_id| parent_ids[*node_id])
    }

    pub fn add_node(&mut self, node: AstNode<'a>, parent_id: Option<AstNodeId>) -> AstNodeId {
        let mut node = node;
        let ast_node_id = self.parent_ids.push(parent_id);
        node.id = ast_node_id;
        self.nodes.push(node);
        ast_node_id
    }
}

#[derive(Debug)]
pub struct AstNodeParentIter<'s, 'a> {
    curr: Option<&'s AstNode<'a>>,
    nodes: &'s AstNodes<'a>,
}

impl<'s, 'a> Iterator for AstNodeParentIter<'s, 'a> {
    type Item = &'s AstNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.curr;
        self.curr = self.curr.and_then(|curr| self.nodes.parent_node(curr.id()));

        next
    }
}
