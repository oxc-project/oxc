use std::ops::{Deref, DerefMut, Index, IndexMut};

use indextree::{Arena, NodeId};

use super::{AstNode, AstNodeId, SemanticNode};

/// Untyped AST nodes flattened into an indextree
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    /// The memory storage of the indextree is backed by a vector,
    /// which allows for efficient traversal.
    /// This also allows for parallel traversal by using `rayon`.
    nodes: Arena<SemanticNode<'a>>,
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
