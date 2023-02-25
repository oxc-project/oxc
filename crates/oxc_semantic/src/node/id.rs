use std::ops::Deref;

use indextree::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstNodeId(NodeId);

impl AstNodeId {
    #[must_use]
    pub const fn new(node_id: NodeId) -> Self {
        Self(node_id)
    }

    #[must_use]
    pub const fn indextree_id(&self) -> NodeId {
        self.0
    }
}

impl From<NodeId> for AstNodeId {
    fn from(node_id: NodeId) -> Self {
        Self(node_id)
    }
}

impl From<AstNodeId> for NodeId {
    fn from(ast_node_id: AstNodeId) -> Self {
        ast_node_id.indextree_id()
    }
}

impl Deref for AstNodeId {
    type Target = NodeId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
