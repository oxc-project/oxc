mod builder;
mod node;

use std::rc::Rc;

pub use builder::SemanticBuilder;
pub use node::{AstNode, AstNodes};
use oxc_ast::Trivias;

pub struct Semantic<'a> {
    nodes: AstNodes<'a>,

    trivias: Rc<Trivias>,
}

impl<'a> Semantic<'a> {
    #[must_use]
    pub const fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }

    #[must_use]
    pub fn trivias(&self) -> &Trivias {
        &self.trivias
    }
}
