use crate::types::{MatchImportKind, ModuleIdx, SymbolRef};

/// Context passed to [`ImportHooks::on_resolved`] with full resolution details.
pub struct ImportResolutionContext<'a> {
    /// Module that contains the import declaration.
    pub importer: ModuleIdx,
    /// Local symbol bound by the import (e.g., the `foo` in `import { foo }`).
    pub local_symbol: SymbolRef,
    /// The name being imported (e.g., "foo", "default", "*").
    pub imported_name: &'a str,
    /// Index into the importer's `import_records` for this import.
    pub record_idx: usize,
    /// The target module that the import resolves to.
    pub target_module: ModuleIdx,
    /// The resolution result.
    pub result: &'a MatchImportKind,
    /// Re-export chain followed during resolution (empty for direct imports).
    pub reexport_chain: &'a [SymbolRef],
}

/// Optional hooks for consumer-specific import matching behavior.
///
/// 3 of 5 old `ImportMatcher` callbacks are now built-in:
/// - `on_missing_module` → external modules are in graph → use `external.namespace_ref`
/// - `on_before_match` → `exports_kind.is_commonjs()` → `NormalAndNamespace` with `record.namespace_ref`
/// - `on_no_match` → `has_dynamic_exports` → `NormalAndNamespace` fallback
///
/// Only 2 optional hooks remain for consumer-specific logic.
pub trait ImportHooks {
    /// Called after every import resolution (successful or not).
    ///
    /// Use for: re-export chain tracking, namespace alias setup, CJS symbol tracking.
    fn on_resolved(&mut self, ctx: &ImportResolutionContext) {
        let _ = ctx;
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
