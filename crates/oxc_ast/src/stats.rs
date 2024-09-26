use std::cell::Cell;

/// Statistics about data held in the AST.
///
/// Comprises number of AST nodes, scopes, symbols, and references.
///
/// These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
/// `ScopeTree`, and `SymbolTable` to store info for all these items.
#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub nodes: Cell<u32>,
    pub scopes: Cell<u32>,
    pub symbols: Cell<u32>,
    pub references: Cell<u32>,
}

impl Stats {
    /// Rewind the stats to the given state.
    pub fn rewind(&self, to: &Self) {
        self.nodes.set(to.nodes.get());
        self.scopes.set(to.scopes.get());
        self.symbols.set(to.symbols.get());
        self.references.set(to.references.get());
    }
}
