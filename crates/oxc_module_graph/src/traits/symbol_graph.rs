use std::fmt::Debug;
use std::hash::Hash;

/// Cross-module symbol linking.
///
/// This trait abstracts over different symbol resolution strategies.
/// Rolldown can implement this on its own `SymbolRefDb`,
/// while the default implementation uses the built-in `SymbolRefDb`.
pub trait SymbolGraph {
    /// The symbol reference type (e.g., `SymbolRef` in oxc, or Rolldown's own `SymbolRef`).
    type SymbolRef: Copy + Eq + Hash + Debug;

    /// Follow link chains to find the canonical (final) symbol.
    fn canonical_ref_for(&self, symbol: Self::SymbolRef) -> Self::SymbolRef;

    /// Link `from` to resolve to `to`.
    ///
    /// After this call, `canonical_ref_for(from)` should eventually
    /// return `canonical_ref_for(to)`.
    fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef);

    /// Get the declared name of a symbol.
    fn symbol_name(&self, symbol: Self::SymbolRef) -> &str;
}
