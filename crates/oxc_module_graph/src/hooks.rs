use crate::types::{MatchImportKind, ModuleIdx, SymbolRef};

/// Optional hooks for consumer-specific import matching behavior.
///
/// 3 of 5 old `ImportMatcher` callbacks are now built-in:
/// - `on_missing_module` → external modules are in graph → use `external.namespace_ref`
/// - `on_before_match` → `is_commonjs` → `NormalAndNamespace` with `record.namespace_ref`
/// - `on_no_match` → `has_dynamic_exports` → `NormalAndNamespace` fallback
///
/// Only 2 optional hooks remain for consumer-specific logic.
pub trait ImportHooks {
    /// Called after every import resolution (successful or not).
    ///
    /// Use for: re-export chain tracking, namespace alias setup, CJS symbol tracking.
    fn on_resolved(
        &mut self,
        importer: ModuleIdx,
        local_symbol: SymbolRef,
        result: &MatchImportKind,
        reexport_chain: &[SymbolRef],
    ) {
        let _ = (importer, local_symbol, result, reexport_chain);
    }

    /// Called when no match found and no built-in fallback applies.
    ///
    /// Return `Some` to override, `None` for standard `NoMatch` error.
    fn on_final_no_match(
        &mut self,
        target: ModuleIdx,
        import_name: &str,
    ) -> Option<MatchImportKind> {
        let _ = (target, import_name);
        None
    }
}

/// Optional hooks for consumer-specific side-effects checks.
///
/// Built-in: `has_dynamic_exports` check (already computed by graph).
/// This hook adds extra checks beyond the built-in ones.
pub trait SideEffectsHooks {
    /// Extra side-effects check beyond the built-in `has_dynamic_exports`.
    ///
    /// Primary use: Rolldown's WrapKind (wrapped modules always have side effects).
    fn star_export_has_extra_side_effects(&self, importer: ModuleIdx, importee: ModuleIdx) -> bool {
        let _ = (importer, importee);
        false
    }
}

/// Configuration for the link pipeline.
#[derive(Default)]
pub struct LinkConfig<'a> {
    /// If true, dynamic imports are followed for execution order.
    pub include_dynamic_imports: bool,
    /// If true, built-in CJS namespace fallback is enabled in import matching.
    pub cjs_interop: bool,
    /// Optional import hooks for consumer-specific behavior.
    pub import_hooks: Option<&'a mut dyn ImportHooks>,
    /// Optional side-effects hooks for consumer-specific behavior.
    pub side_effects_hooks: Option<&'a dyn SideEffectsHooks>,
}
