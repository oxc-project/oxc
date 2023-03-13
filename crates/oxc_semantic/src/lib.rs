#![feature(is_some_and)]
#![feature(let_chains)]

mod binder;
mod builder;
mod node;
mod scope;
mod symbol;

use std::rc::Rc;

pub use builder::SemanticBuilder;
pub use node::{AstNode, AstNodes, SemanticNode};
use oxc_ast::{SourceType, Trivias};
pub use scope::{Scope, ScopeFlags, ScopeTree};

pub struct Semantic<'a> {
    source_type: SourceType,

    nodes: AstNodes<'a>,

    scopes: ScopeTree,

    trivias: Rc<Trivias>,
}

impl<'a> Semantic<'a> {
    #[must_use]
    pub fn source_type(&self) -> &SourceType {
        &self.source_type
    }

    #[must_use]
    pub fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }

    #[must_use]
    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    #[must_use]
    pub fn trivias(&self) -> &Trivias {
        &self.trivias
    }
}
