use std::path::PathBuf;

use compact_str::CompactString;
use rustc_hash::FxHashMap;

use crate::types::{
    ImportKind, IndirectExportEntry, LocalExport, ModuleIdx, NamedImport, ResolvedExport,
    ResolvedImportRecord, StarExportEntry, SymbolRef,
};

/// Side-effects status for a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffects {
    /// Has side effects (default).
    True,
    /// Declared side-effect-free (package.json or plugin).
    False,
    /// Cannot tree-shake (noTreeshake annotation).
    NoTreeshake,
}

impl SideEffects {
    /// Convert to the `Option<bool>` representation used by algorithms.
    ///
    /// - `True` → `Some(true)`
    /// - `False` → `Some(false)`
    /// - `NoTreeshake` → `None`
    pub fn to_option(self) -> Option<bool> {
        match self {
            Self::True => Some(true),
            Self::False => Some(false),
            Self::NoTreeshake => None,
        }
    }

    /// Create from the `Option<bool>` representation.
    pub fn from_option(opt: Option<bool>) -> Self {
        match opt {
            Some(true) => Self::True,
            Some(false) => Self::False,
            None => Self::NoTreeshake,
        }
    }
}

/// A normal (parseable) module in the module graph.
#[derive(Debug)]
pub struct NormalModule {
    // --- Identity ---
    /// This module's index in the graph.
    pub idx: ModuleIdx,
    /// Absolute file path.
    pub path: PathBuf,

    // --- Parse-time data ---
    /// Whether this module has ESM syntax (`import`/`export`).
    pub has_module_syntax: bool,
    /// Whether this module uses CommonJS (`module.exports` / `exports.x`).
    pub is_commonjs: bool,
    /// Whether this module contains top-level `await`.
    pub has_top_level_await: bool,
    /// The module's own side-effects state (before propagation).
    pub side_effects: SideEffects,

    // --- Import/export declarations ---
    /// Named exports: export_name → LocalExport.
    pub named_exports: FxHashMap<CompactString, LocalExport>,
    /// Named imports: local_symbol → NamedImport.
    pub named_imports: FxHashMap<SymbolRef, NamedImport>,
    /// Import records (resolved).
    pub import_records: Vec<ResolvedImportRecord>,
    /// Star exports: `export * from '...'`
    pub star_export_entries: Vec<StarExportEntry>,
    /// Indirect exports: `export { x } from '...'`
    pub indirect_export_entries: Vec<IndirectExportEntry>,

    // --- Module-level symbols ---
    /// SymbolRef for default export.
    pub default_export_ref: SymbolRef,
    /// SymbolRef for namespace object (`import * as ns`).
    pub namespace_object_ref: SymbolRef,

    // --- Link-time results (populated by graph.link() or individual algorithms) ---
    /// Resolved exports: export_name → ResolvedExport.
    pub resolved_exports: FxHashMap<CompactString, ResolvedExport>,
    /// Whether this module has dynamic exports (CJS or transitive `export *` from CJS/external).
    pub has_dynamic_exports: bool,
    /// Whether this module is affected by top-level `await` (directly or transitively).
    pub is_tla_or_contains_tla: bool,
    /// Whether this module has propagated side effects.
    pub propagated_side_effects: bool,
    /// Execution order index (set during link).
    pub exec_order: u32,
}

impl NormalModule {
    /// Get the resolved target module for an import record by index.
    pub fn import_record_resolved_module(&self, idx: usize) -> Option<ModuleIdx> {
        self.import_records.get(idx)?.resolved_module
    }

    /// If the given symbol is a named import in this module, return its import info.
    ///
    /// Returns `Some((imported_name, record_idx, is_namespace))`.
    pub fn symbol_import_info(&self, symbol: SymbolRef) -> Option<(&str, usize, bool)> {
        let import = self.named_imports.get(&symbol)?;
        let is_ns = import.imported_name.as_str() == "*";
        Some((import.imported_name.as_str(), import.record_idx.index(), is_ns))
    }

    /// Iterate over static import dependencies only.
    pub fn static_dependencies(&self) -> impl Iterator<Item = ModuleIdx> + '_ {
        self.import_records.iter().filter_map(|rec| {
            if rec.kind == ImportKind::Static { rec.resolved_module } else { None }
        })
    }

    /// Iterate over star export target module indices.
    pub fn star_export_modules(&self) -> impl Iterator<Item = ModuleIdx> + '_ {
        self.star_export_entries.iter().filter_map(|e| e.resolved_module)
    }
}

/// An external (unresolvable) module — first-class node in the graph.
#[derive(Debug)]
pub struct ExternalModule {
    /// This module's index in the graph.
    pub idx: ModuleIdx,
    /// The specifier string (e.g., "react", "lodash").
    pub specifier: CompactString,
    /// Side-effects status.
    pub side_effects: SideEffects,
    /// SymbolRef for this module's namespace object.
    pub namespace_ref: SymbolRef,

    // --- Link-time results ---
    /// Execution order index (set during link).
    pub exec_order: u32,
}

/// A module in the graph — either normal (parseable) or external.
#[derive(Debug)]
pub enum Module {
    Normal(NormalModule),
    External(ExternalModule),
}

impl Module {
    /// Get the module index.
    pub fn idx(&self) -> ModuleIdx {
        match self {
            Self::Normal(m) => m.idx,
            Self::External(m) => m.idx,
        }
    }

    /// Get the execution order.
    pub fn exec_order(&self) -> u32 {
        match self {
            Self::Normal(m) => m.exec_order,
            Self::External(m) => m.exec_order,
        }
    }

    /// Get the side effects status.
    pub fn side_effects(&self) -> SideEffects {
        match self {
            Self::Normal(m) => m.side_effects,
            Self::External(m) => m.side_effects,
        }
    }

    /// Get as a normal module reference, if it is one.
    pub fn as_normal(&self) -> Option<&NormalModule> {
        match self {
            Self::Normal(m) => Some(m),
            Self::External(_) => None,
        }
    }

    /// Get as a mutable normal module reference, if it is one.
    pub fn as_normal_mut(&mut self) -> Option<&mut NormalModule> {
        match self {
            Self::Normal(m) => Some(m),
            Self::External(_) => None,
        }
    }

    /// Get as an external module reference, if it is one.
    pub fn as_external(&self) -> Option<&ExternalModule> {
        match self {
            Self::External(m) => Some(m),
            Self::Normal(_) => None,
        }
    }

    /// Get as a mutable external module reference, if it is one.
    pub fn as_external_mut(&mut self) -> Option<&mut ExternalModule> {
        match self {
            Self::External(m) => Some(m),
            Self::Normal(_) => None,
        }
    }

    /// Get the namespace object ref (for both normal and external modules).
    pub fn namespace_object_ref(&self) -> SymbolRef {
        match self {
            Self::Normal(m) => m.namespace_object_ref,
            Self::External(m) => m.namespace_ref,
        }
    }
}
