use std::ops::{Deref, DerefMut, Index, IndexMut};

use indextree::{Ancestors, Arena, Node, NodeId};

use super::{Scope, ScopeFlags, ScopeId};
use crate::node::AstNode;

#[derive(Debug)]
pub struct ScopeTree {
    scopes: Arena<Scope>,

    root_scope_id: ScopeId,
}

impl ScopeTree {
    #[must_use]
    pub fn new(root_strict_mode: bool) -> Self {
        let mut scopes = Arena::new();
        let root_scope = Scope::new(ScopeFlags::Top, root_strict_mode);
        let root_scope_id = scopes.new_node(root_scope).into();
        Self { scopes, root_scope_id }
    }

    #[must_use]
    pub const fn root_scope_id(&self) -> ScopeId {
        self.root_scope_id
    }

    #[must_use]
    pub fn ancestors(&self, scope_id: ScopeId) -> Ancestors<'_, Scope> {
        scope_id.ancestors(&self.scopes)
    }

    #[must_use]
    pub fn node_scope(&self, node: &AstNode) -> &Scope {
        self.scopes[node.get().scope_id().indextree_id()].get()
    }

    #[must_use]
    pub fn node_scope_ancestors(&self, node: &AstNode) -> Ancestors<'_, Scope> {
        self.ancestors(node.get().scope_id())
    }

    /// # Panics
    /// When parent scope cannot be found, but this will not happen because
    /// scopes are never removed.
    #[must_use]
    pub fn parent_node_id(&self, scope_id: ScopeId) -> NodeId {
        self.scopes[*scope_id].parent().unwrap()
    }

    #[must_use]
    pub fn parent_scope(&self, scope_id: ScopeId) -> &Scope {
        let parent_id = self.parent_node_id(scope_id);
        self.scopes[parent_id].get()
    }

    #[must_use]
    pub fn parent_scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope {
        let parent_id = self.parent_node_id(scope_id);
        self.scopes[parent_id].get_mut()
    }

    #[must_use]
    pub fn strict_mode(&self, node: &AstNode) -> bool {
        let scope = self.node_scope(node);
        node.get().strict_mode(scope)
    }
}

impl Index<NodeId> for ScopeTree {
    type Output = Node<Scope>;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.scopes[id]
    }
}

impl IndexMut<NodeId> for ScopeTree {
    fn index_mut(&mut self, id: NodeId) -> &mut Node<Scope> {
        &mut self.scopes[id]
    }
}

impl Index<ScopeId> for ScopeTree {
    type Output = Scope;

    fn index(&self, id: ScopeId) -> &Self::Output {
        self.scopes[id.indextree_id()].get()
    }
}

impl IndexMut<ScopeId> for ScopeTree {
    fn index_mut(&mut self, id: ScopeId) -> &mut Scope {
        self.scopes[id.indextree_id()].get_mut()
    }
}

impl Deref for ScopeTree {
    type Target = Arena<Scope>;

    fn deref(&self) -> &Self::Target {
        &self.scopes
    }
}

impl DerefMut for ScopeTree {
    fn deref_mut(&mut self) -> &mut Arena<Scope> {
        &mut self.scopes
    }
}
