use std::{cell::Cell, hash::Hash};

use oxc_syntax::node::AstNodeId;

pub trait AstNode {
    fn ast_node_id(&self) -> Option<AstNodeId>;
}

/// Thin wrapper around `Cell<Option<AstNodeId>>`
/// It is used to have an empty hash trait implemented for it.
#[derive(Default, Debug, Clone)]
pub struct AstNodeIdContainer(Cell<Option<AstNodeId>>);

impl AstNodeIdContainer {
    pub fn get(&self) -> Option<AstNodeId> {
        self.0.get()
    }
}

impl Hash for AstNodeIdContainer {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}
