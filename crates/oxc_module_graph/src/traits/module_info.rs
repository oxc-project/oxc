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

    /// Iterate named exports as `(name, symbol_ref)` pairs.
    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef));

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
}
