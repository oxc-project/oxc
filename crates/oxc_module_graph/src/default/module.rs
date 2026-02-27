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

    /// Named exports: export_name -> LocalExport.
    pub named_exports: FxHashMap<CompactString, LocalExport>,
    /// Named imports: local_symbol -> NamedImport.
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
    type ModuleIdx = ModuleIdx;
    type SymbolRef = SymbolRef;

    fn module_idx(&self) -> ModuleIdx {
        self.idx
    }

    fn default_export_ref(&self) -> SymbolRef {
        self.default_export_ref
    }

    fn namespace_object_ref(&self) -> SymbolRef {
        self.namespace_object_ref
    }

    fn has_module_syntax(&self) -> bool {
        self.has_module_syntax
    }

    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, SymbolRef)) {
        for (name, local_export) in &self.named_exports {
            f(name.as_str(), local_export.local_symbol);
        }
    }

    fn for_each_named_import(&self, f: &mut dyn FnMut(SymbolRef, &str, usize, bool)) {
        for named_import in self.named_imports.values() {
            let is_ns = named_import.imported_name.as_str() == "*";
            f(
                named_import.local_symbol,
                named_import.imported_name.as_str(),
                named_import.record_idx.index(),
                is_ns,
            );
        }
    }

    fn import_record_count(&self) -> usize {
        self.import_records.len()
    }

    fn import_record_resolved_module(&self, idx: usize) -> Option<ModuleIdx> {
        self.import_records.get(idx)?.resolved_module
    }

    fn for_each_star_export(&self, f: &mut dyn FnMut(ModuleIdx)) {
        for entry in &self.star_export_entries {
            if let Some(target) = entry.resolved_module {
                f(target);
            }
        }
    }

    fn for_each_indirect_export(&self, f: &mut dyn FnMut(&str, &str, ModuleIdx)) {
        for entry in &self.indirect_export_entries {
            if let Some(target) = entry.resolved_module {
                f(entry.exported_name.as_str(), entry.imported_name.as_str(), target);
            }
        }
    }
}
