use crate::{ScopeTree, SymbolTable};

pub struct Scoping {
    symbols: SymbolTable,
    scopes: ScopeTree,
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
