use std::ops::{Deref, DerefMut, Index, IndexMut};

use indextree::{Ancestors, Arena, NodeId};
use oxc_ast::AstKind;

use super::{AstNode, AstNodeId, SemanticNode};

/// Untyped AST nodes flattened into an indextree
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    /// The memory storage of the indextree is backed by a vector,
    /// which allows for efficient traversal.
    /// This also allows for parallel traversal by using `rayon`.
    nodes: Arena<SemanticNode<'a>>,
}

impl<'a> AstNodes<'a> {
    /// # Panics
    #[must_use]
    pub fn ancestors(&self, node: &AstNode<'a>) -> Ancestors<'_, SemanticNode<'a>> {
        let node_id = self.nodes.get_node_id(node).unwrap();
        node_id.ancestors(&self.nodes)
    }
}

impl<'a> Index<NodeId> for AstNodes<'a> {
    type Output = AstNode<'a>;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.nodes[id]
    }
}

impl<'a> IndexMut<NodeId> for AstNodes<'a> {
    fn index_mut(&mut self, id: NodeId) -> &mut AstNode<'a> {
        &mut self.nodes[id]
    }
}

impl<'a> Index<AstNodeId> for AstNodes<'a> {
    type Output = SemanticNode<'a>;

    fn index(&self, id: AstNodeId) -> &Self::Output {
        self.nodes[id.indextree_id()].get()
    }
}

impl<'a> IndexMut<AstNodeId> for AstNodes<'a> {
    fn index_mut(&mut self, id: AstNodeId) -> &mut SemanticNode<'a> {
        self.nodes[id.indextree_id()].get_mut()
    }
}

impl<'a> Deref for AstNodes<'a> {
    type Target = Arena<SemanticNode<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl<'a> DerefMut for AstNodes<'a> {
    fn deref_mut(&mut self) -> &mut Arena<SemanticNode<'a>> {
        &mut self.nodes
    }
}

impl<'a> AstNodes<'a> {
    #[must_use]
    pub fn kind<T: Into<NodeId>>(&self, id: T) -> AstKind<'a> {
        self.nodes[id.into()].get().kind
    }

    #[must_use]
    pub fn parent_kind(&self, node: &AstNode<'a>) -> AstKind<'a> {
        node.parent().map_or(AstKind::Root, |node_id| self.kind(node_id))
    }
}
