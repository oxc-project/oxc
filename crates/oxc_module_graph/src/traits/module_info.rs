use compact_str::CompactString;
use rustc_hash::FxHashMap;

use crate::ModuleIdx;
use crate::types::{
    IndirectExportEntry, LocalExport, NamedImport, ResolvedImportRecord, StarExportEntry, SymbolRef,
};

/// Read-only access to a module's import/export declarations.
///
/// This trait abstracts over different module representations.
/// Rolldown can implement this on `NormalModule`/`EcmaView`,
/// while the default implementation uses the built-in `Module` type.
pub trait ModuleInfo {
    /// This module's index in the graph.
    fn module_idx(&self) -> ModuleIdx;

    /// All named exports declared locally by this module.
    fn named_exports(&self) -> &FxHashMap<CompactString, LocalExport>;

    /// All named imports consumed by this module.
    fn named_imports(&self) -> &FxHashMap<SymbolRef, NamedImport>;

    /// Import records (after resolution, each contains target `ModuleIdx`).
    fn import_records(&self) -> &[ResolvedImportRecord];

    /// The `SymbolRef` for this module's default export.
    fn default_export_ref(&self) -> SymbolRef;

    /// The `SymbolRef` for this module's namespace object (`import * as ns`).
    fn namespace_object_ref(&self) -> SymbolRef;

    /// Star export entries (`export * from './foo'`).
    fn star_export_entries(&self) -> &[StarExportEntry];

    /// Indirect export entries (`export { x } from './foo'`).
    fn indirect_export_entries(&self) -> &[IndirectExportEntry];

    /// Whether this module has ESM syntax (`import`/`export`).
    fn has_module_syntax(&self) -> bool;
}
