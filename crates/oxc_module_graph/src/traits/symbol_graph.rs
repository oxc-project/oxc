use crate::types::SymbolRef;

/// Cross-module symbol linking.
///
/// This trait abstracts over different symbol resolution strategies.
/// Rolldown can implement this on `SymbolRefDb`,
/// while the default implementation uses the built-in `SymbolRefDb`.
pub trait SymbolGraph {
    /// Follow link chains to find the canonical (final) symbol.
    fn canonical_ref_for(&self, symbol: SymbolRef) -> SymbolRef;

    /// Link `from` to resolve to `to`.
    ///
    /// After this call, `canonical_ref_for(from)` should eventually
    /// return `canonical_ref_for(to)`.
    fn link(&mut self, from: SymbolRef, to: SymbolRef);

    /// Get the declared name of a symbol.
    fn symbol_name(&self, symbol: SymbolRef) -> &str;
}
