use std::{cell::Cell, hash::Hash};

use oxc_syntax::node::AstNodeId;

pub trait AstNode {
    /// Get the inner value of `self.ast_node_id`.
    fn ast_node_id(&self) -> Option<AstNodeId>;

    /// Sets the inner `self.ast_node_id` value.
    fn set_ast_node_id(&self, id: Option<AstNodeId>);

    /// Swaps the `self.ast_node_id` value and returns the old one.
    fn swap_ast_node_id(&self, id: Option<AstNodeId>) -> Option<AstNodeId>;
}

/// Thin wrapper around `Cell<Option<AstNodeId>>`
/// It is used to have an empty hash trait implemented for it.
#[derive(Default, Debug, Clone)]
pub struct AstNodeIdContainer(Cell<Option<AstNodeId>>);

impl AstNodeIdContainer {
    pub(crate) fn get(&self) -> Option<AstNodeId> {
        self.0.get()
    }

    pub(crate) fn set(&self, id: Option<AstNodeId>) {
        self.0.replace(id);
    }

    pub(crate) fn swap(&self, id: Option<AstNodeId>) -> Option<AstNodeId> {
        self.0.replace(id)
    }
}

impl Hash for AstNodeIdContainer {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}
