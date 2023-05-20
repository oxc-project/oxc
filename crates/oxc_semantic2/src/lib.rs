mod builder;
mod mangler;
pub mod reference;
pub mod scope;
pub mod symbol;

pub use crate::builder::SemanticBuilder;
use crate::{mangler::Mangler, scope::ScopeTree, symbol::SymbolTable};

pub struct Semantic {
    pub scope_tree: ScopeTree,
    pub symbol_table: SymbolTable,
}

impl Semantic {
    pub fn mangle(&mut self) {
        Mangler::mangle(self);
    }
}
