use std::ops::Deref;

use indextree::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeId(NodeId);

impl ScopeId {
    #[must_use]
    pub fn new(node_id: NodeId) -> Self {
        Self(node_id)
    }

    #[must_use]
    pub fn indextree_id(&self) -> NodeId {
        self.0
    }
}

impl From<NodeId> for ScopeId {
    fn from(node_id: NodeId) -> Self {
        Self(node_id)
    }
}

impl Deref for ScopeId {
    type Target = NodeId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
