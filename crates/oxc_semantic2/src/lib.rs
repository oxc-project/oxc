mod builder;
pub mod scope;
pub mod symbol;

pub use crate::builder::SemanticBuilder;
use crate::{scope::ScopeTree, symbol::SymbolTable};

pub struct Semantic {
    pub scope_tree: ScopeTree,
    pub symbol_table: SymbolTable,
}
