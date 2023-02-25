mod builder;
mod node;

pub use builder::SemanticBuilder;
pub use node::{AstNode, AstNodes};

pub struct Semantic<'a> {
    nodes: AstNodes<'a>,
}

impl<'a> Semantic<'a> {
    #[must_use]
    pub const fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }
}
