use crate::{ScopeTree, SymbolTable};

#[derive(Default)]
pub struct Scoping {
    pub(crate) symbols: SymbolTable,
    pub(crate) scopes: ScopeTree,
}

impl Scoping {
    pub fn new(symbols: SymbolTable, scopes: ScopeTree) -> Self {
        Self { symbols, scopes }
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    pub fn into_symbols_scopes(self) -> (SymbolTable, ScopeTree) {
        (self.symbols, self.scopes)
    }
}
