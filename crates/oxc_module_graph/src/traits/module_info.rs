use std::fmt::Debug;
use std::hash::Hash;

/// Read-only access to a module's import/export declarations.
///
/// This trait abstracts over different module representations.
/// Rolldown can implement this on `NormalModule`/`EcmaView`,
/// while the default implementation uses the built-in `Module` type.
///
/// Uses callback-based iteration (`for_each_*`) instead of returning
/// concrete collection references, so consumers can use any internal
/// collection types (e.g., `FxHashMap` vs `FxIndexMap`).
pub trait ModuleInfo {
    /// The module index type (e.g., `ModuleIdx`).
    type ModuleIdx: Copy + Eq + Hash + Debug;
    /// The symbol reference type (e.g., `SymbolRef`).
    type SymbolRef: Copy + Eq + Hash + Debug;

    /// This module's index in the graph.
    fn module_idx(&self) -> Self::ModuleIdx;

    /// The `SymbolRef` for this module's default export.
    fn default_export_ref(&self) -> Self::SymbolRef;

    /// The `SymbolRef` for this module's namespace object (`import * as ns`).
    fn namespace_object_ref(&self) -> Self::SymbolRef;

    /// Whether this module has ESM syntax (`import`/`export`).
    fn has_module_syntax(&self) -> bool;

    /// Iterate named exports as `(name, symbol_ref, came_from_cjs)` triples.
    ///
    /// - `name`: the exported name
    /// - `symbol_ref`: the local symbol being exported
    /// - `came_from_cjs`: `true` if this export originated from CommonJS
    ///   (e.g., `exports.foo = 1`). Affects star re-export semantics:
    ///   CJS "default" exports are propagated through `export *`, and
    ///   ambiguity detection is suppressed for CJS-originated exports.
    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef, bool));

    /// Iterate named imports as `(local_symbol, imported_name, record_idx, is_namespace)`.
    ///
    /// - `local_symbol`: the local binding symbol
    /// - `imported_name`: the name in the source module ("foo", "default", "*")
    /// - `record_idx`: index into the module's import records
    /// - `is_namespace`: true if this is `import * as ns`
    #[expect(clippy::type_complexity)]
    fn for_each_named_import(&self, f: &mut dyn FnMut(Self::SymbolRef, &str, usize, bool));

    /// Number of import records.
    fn import_record_count(&self) -> usize;

    /// Get the resolved module for an import record by index.
    fn import_record_resolved_module(&self, idx: usize) -> Option<Self::ModuleIdx>;

    /// Iterate star export target module indices.
    fn for_each_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx));

    /// Iterate indirect exports as `(exported_name, imported_name, resolved_module)`.
    fn for_each_indirect_export(&self, f: &mut dyn FnMut(&str, &str, Self::ModuleIdx));

    /// Whether this module uses CommonJS (`module.exports` / `exports.x`).
    ///
    /// Used by `compute_has_dynamic_exports` to determine if star re-exports
    /// from this module should be treated as dynamic (CJS exports are not
    /// statically analyzable).
    fn is_commonjs(&self) -> bool;

    /// Whether this module contains top-level `await`.
    ///
    /// Used by `compute_tla` to propagate TLA status through static import chains.
    fn has_top_level_await(&self) -> bool;

    /// Iterate import records as `(record_idx, resolved_module, import_kind)`.
    ///
    /// Gives algorithms access to edge kind information for each import.
    /// Used by execution-order sort and TLA propagation.
    fn for_each_import_record(
        &self,
        f: &mut dyn FnMut(usize, Option<Self::ModuleIdx>, crate::types::ImportKind),
    );

    /// The module's own side-effects state (before propagation).
    ///
    /// Returns:
    /// - `Some(true)` — has side effects (user-defined or analyzed)
    /// - `Some(false)` — no side effects (analyzed), check dependencies
    /// - `None` — no-treeshake or similar, always keep
    fn side_effects(&self) -> Option<bool>;

    /// Iterate star export targets from ESM `export * from '...'` only.
    ///
    /// Excludes CJS reexport patterns (e.g., `module.exports = require('x')`).
    /// Used by side-effects propagation, which should only follow ESM star exports.
    ///
    /// Default: delegates to `for_each_star_export`.
    fn for_each_esm_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx)) {
        self.for_each_star_export(f);
    }

    /// If the given symbol is a named import in this module, return its import info.
    ///
    /// Returns `Some((imported_name, record_idx, is_namespace))` if the symbol
    /// is a named import, `None` otherwise.
    ///
    /// This is needed by the re-export-chain-following algorithm: when a
    /// resolved export symbol turns out to be an import in its owning module,
    /// the algorithm recurses into that module's import target.
    fn symbol_import_info(&self, symbol: Self::SymbolRef) -> Option<(&str, usize, bool)>;
}
