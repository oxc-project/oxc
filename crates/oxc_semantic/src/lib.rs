#![feature(is_some_and)]
#![feature(let_chains)]

mod binder;
mod builder;
mod module_record;
mod node;
mod scope;
mod symbol;

use std::rc::Rc;

pub use builder::SemanticBuilder;
pub use node::{AstNode, AstNodes, SemanticNode};
use oxc_ast::{module_record::ModuleRecord, SourceType, Trivias};
pub use scope::{Scope, ScopeFlags, ScopeTree};
pub use symbol::{Reference, Symbol, SymbolFlags, SymbolTable};

pub struct Semantic<'a> {
    source_type: SourceType,

    nodes: AstNodes<'a>,

    scopes: ScopeTree,

    symbols: SymbolTable,

    trivias: Rc<Trivias>,

    module_record: ModuleRecord,
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

    #[must_use]
    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
    }

    #[must_use]
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }
}
