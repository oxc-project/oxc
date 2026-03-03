use oxc_syntax::symbol::SymbolId;

use super::ModuleIdx;

/// A cross-module symbol reference: a (module, symbol) pair.
///
/// This uniquely identifies a symbol across the entire module graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolRef {
    /// The module that owns this symbol.
    pub owner: ModuleIdx,
    /// The symbol within the owning module.
    pub symbol: SymbolId,
}

impl SymbolRef {
    pub fn new(owner: ModuleIdx, symbol: SymbolId) -> Self {
        Self { owner, symbol }
    }
}
