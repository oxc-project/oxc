use std::path::PathBuf;

use compact_str::CompactString;
use rustc_hash::FxHashMap;

use crate::traits::ModuleInfo;
use crate::types::{
    ImportEdge, IndirectExportEntry, LocalExport, ModuleIdx, NamedImport, ResolvedImportRecord,
    StarExportEntry, SymbolRef,
};

/// Default module implementation — stores all import/export data.
#[derive(Debug)]
pub struct Module {
    /// This module's index in the graph.
    pub idx: ModuleIdx,
    /// Absolute file path.
    pub path: PathBuf,
    /// Whether this module has ESM syntax.
    pub has_module_syntax: bool,

    /// Named exports: export_name → LocalExport.
    pub named_exports: FxHashMap<CompactString, LocalExport>,
    /// Named imports: local_symbol → NamedImport.
    pub named_imports: FxHashMap<SymbolRef, NamedImport>,
    /// Import records (resolved).
    pub import_records: Vec<ResolvedImportRecord>,

    /// SymbolRef for default export.
    pub default_export_ref: SymbolRef,
    /// SymbolRef for namespace object.
    pub namespace_object_ref: SymbolRef,

    /// Star exports: `export * from '...'`
    pub star_export_entries: Vec<StarExportEntry>,
    /// Indirect exports: `export { x } from '...'`
    pub indirect_export_entries: Vec<IndirectExportEntry>,

    /// Dependency edges to other modules.
    pub dependencies: Vec<ImportEdge>,
}

impl ModuleInfo for Module {
    fn module_idx(&self) -> ModuleIdx {
        self.idx
    }

    fn named_exports(&self) -> &FxHashMap<CompactString, LocalExport> {
        &self.named_exports
    }

    fn named_imports(&self) -> &FxHashMap<SymbolRef, NamedImport> {
        &self.named_imports
    }

    fn import_records(&self) -> &[ResolvedImportRecord] {
        &self.import_records
    }

    fn default_export_ref(&self) -> SymbolRef {
        self.default_export_ref
    }

    fn namespace_object_ref(&self) -> SymbolRef {
        self.namespace_object_ref
    }

    fn star_export_entries(&self) -> &[StarExportEntry] {
        &self.star_export_entries
    }

    fn indirect_export_entries(&self) -> &[IndirectExportEntry] {
        &self.indirect_export_entries
    }

    fn has_module_syntax(&self) -> bool {
        self.has_module_syntax
    }
}
