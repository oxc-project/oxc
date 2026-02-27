use std::fmt::Debug;
use std::hash::Hash;

use crate::types::MatchImportKind;

/// Callback trait for consumer-specific import matching behavior.
///
/// The generic `match_imports` algorithm calls these methods when it encounters
/// cases that require consumer-specific knowledge (CJS modules, external
/// modules, dynamic exports).
///
/// Default implementations make the trait work as pure ESM matching out of the
/// box — consumers only override the callbacks they need.
pub trait ImportMatcher {
    /// Module index type — must match the `ModuleStore` being used.
    type ModuleIdx: Copy + Eq + Hash + Debug;
    /// Symbol reference type — must match the `SymbolGraph` being used.
    type SymbolRef: Copy + Eq + Hash + Debug;

    /// Called when the target module is not in the `ModuleStore` (external/missing).
    ///
    /// `importer_idx` and `record_idx` identify which import record triggered this
    /// lookup, allowing consumers to look up per-record data (e.g., namespace_ref).
    ///
    /// Return `Some(kind)` to resolve to a specific symbol, `None` to treat as `NoMatch`.
    fn on_missing_module(
        &mut self,
        importer_idx: Self::ModuleIdx,
        record_idx: usize,
        target_idx: Self::ModuleIdx,
        import_name: &str,
        is_namespace: bool,
    ) -> Option<MatchImportKind<Self::SymbolRef>> {
        let _ = (importer_idx, record_idx, target_idx, import_name, is_namespace);
        None
    }

    /// Called before looking up `resolved_exports`. Allows short-circuiting
    /// for CJS modules, dynamic modules, etc.
    ///
    /// `importer_idx` and `record_idx` identify which import record triggered this
    /// lookup, allowing consumers to look up per-record data (e.g., namespace_ref).
    ///
    /// Return `Some(kind)` to short-circuit, `None` to proceed with normal ESM lookup.
    fn on_before_match(
        &mut self,
        importer_idx: Self::ModuleIdx,
        record_idx: usize,
        target_idx: Self::ModuleIdx,
        import_name: &str,
        is_namespace: bool,
    ) -> Option<MatchImportKind<Self::SymbolRef>> {
        let _ = (importer_idx, record_idx, target_idx, import_name, is_namespace);
        None
    }

    /// Called when the import name is not found in `resolved_exports`.
    ///
    /// Allows dynamic fallback (e.g., `has_dynamic_exports` → namespace).
    /// Return `Some(kind)` for fallback, `None` for `NoMatch`.
    fn on_no_match(
        &mut self,
        target_idx: Self::ModuleIdx,
        import_name: &str,
    ) -> Option<MatchImportKind<Self::SymbolRef>> {
        let _ = (target_idx, import_name);
        None
    }

    /// Called when a match is found but the export has `came_from_cjs` set.
    ///
    /// Allows override (e.g., `DynamicFallbackWithCommonjsReference`).
    /// Return `Some(kind)` to override, `None` to use the normal match.
    fn on_cjs_match(
        &mut self,
        target_idx: Self::ModuleIdx,
        import_name: &str,
        matched_symbol: Self::SymbolRef,
    ) -> Option<MatchImportKind<Self::SymbolRef>> {
        let _ = (target_idx, import_name);
        // Default: treat CJS export as a normal match.
        Some(MatchImportKind::Normal { symbol_ref: matched_symbol })
    }

    /// Called after successful resolution with the full re-export chain.
    ///
    /// `importer_idx` is the module that contains the original import statement.
    ///
    /// Allows consumers to track re-export chains for tree-shaking or
    /// side-effect dependency recording.
    fn on_resolved(
        &mut self,
        importer_idx: Self::ModuleIdx,
        local_symbol: Self::SymbolRef,
        resolved: &MatchImportKind<Self::SymbolRef>,
        reexport_chain: &[Self::SymbolRef],
    ) {
        let _ = (importer_idx, local_symbol, resolved, reexport_chain);
    }
}

/// A no-op `ImportMatcher` for pure ESM matching with no consumer-specific behavior.
///
/// All callbacks use the default implementations (no short-circuiting,
/// no CJS interop, no dynamic fallback).
pub struct DefaultImportMatcher<Idx, Sym>(std::marker::PhantomData<(Idx, Sym)>);

impl<Idx, Sym> Default for DefaultImportMatcher<Idx, Sym> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<Idx: Copy + Eq + Hash + Debug, Sym: Copy + Eq + Hash + Debug> ImportMatcher
    for DefaultImportMatcher<Idx, Sym>
{
    type ModuleIdx = Idx;
    type SymbolRef = Sym;
}
