use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::{fs, io};

use compact_str::CompactString;
use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_syntax::module_record as syntax;
use oxc_syntax::symbol::SymbolId;

use crate::types::{
    ImportEdge, ImportKind, ImportRecordIdx, IndirectExportEntry, LocalExport, ModuleIdx,
    NamedImport, ResolvedImportRecord, StarExportEntry, SymbolRef,
};

use super::module::Module;
use super::{DefaultModuleGraph, SymbolRefDb};

/// Errors from building the module graph.
#[derive(Debug)]
pub enum BuildError {
    /// Failed to read a file.
    Io(PathBuf, io::Error),
    /// Parse errors in a file (non-fatal; module is still added).
    ParseErrors(PathBuf, Vec<String>),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(path, err) => write!(f, "IO error reading {}: {err}", path.display()),
            Self::ParseErrors(path, errs) => {
                write!(f, "Parse errors in {}:", path.display())?;
                for e in errs {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
        }
    }
}

/// Result of building a module graph.
pub struct BuildResult {
    /// The constructed module graph.
    pub graph: DefaultModuleGraph,
    /// The symbol database.
    pub symbols: SymbolRefDb,
    /// Non-fatal errors encountered during building.
    pub errors: Vec<BuildError>,
}

/// Builds a `DefaultModuleGraph` from entry points.
///
/// Uses `oxc_parser` + `oxc_semantic` to parse files,
/// and `oxc_resolver` to resolve import specifiers via BFS.
#[derive(Debug)]
pub struct ModuleGraphBuilder {
    /// The module graph being built.
    pub graph: DefaultModuleGraph,
    /// The symbol database being built.
    pub symbols: SymbolRefDb,
    /// Maps absolute paths to module indices (deduplication).
    path_to_idx: FxHashMap<PathBuf, ModuleIdx>,
    /// Non-fatal errors.
    errors: Vec<BuildError>,
}

impl ModuleGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DefaultModuleGraph::new(),
            symbols: SymbolRefDb::new(),
            path_to_idx: FxHashMap::default(),
            errors: Vec::new(),
        }
    }

    /// Build the module graph starting from the given entry point paths.
    ///
    /// Performs BFS: parse each file, resolve its import specifiers,
    /// and enqueue newly discovered modules.
    pub fn build(mut self, entries: &[PathBuf]) -> BuildResult {
        let resolver = Resolver::new(ResolveOptions {
            extensions: vec![
                ".js".into(),
                ".mjs".into(),
                ".cjs".into(),
                ".ts".into(),
                ".tsx".into(),
                ".jsx".into(),
                ".mts".into(),
                ".cts".into(),
                ".d.ts".into(),
                ".d.mts".into(),
                ".d.cts".into(),
                ".json".into(),
            ],
            ..ResolveOptions::default()
        });

        let mut queue: VecDeque<PathBuf> = VecDeque::new();

        // Enqueue entry points.
        for entry in entries {
            let abs = if entry.is_absolute() {
                entry.clone()
            } else {
                std::env::current_dir().unwrap_or_default().join(entry)
            };
            let canonical = fs::canonicalize(&abs).unwrap_or(abs);
            if !self.path_to_idx.contains_key(&canonical) {
                let idx = self.graph.next_idx();
                self.path_to_idx.insert(canonical.clone(), idx);
                queue.push_back(canonical);
            }
        }

        // BFS
        while let Some(path) = queue.pop_front() {
            let idx = self.path_to_idx[&path];
            self.process_module(&resolver, &path, idx, &mut queue);
        }

        BuildResult { graph: self.graph, symbols: self.symbols, errors: self.errors }
    }

    fn process_module(
        &mut self,
        resolver: &Resolver,
        path: &Path,
        idx: ModuleIdx,
        queue: &mut VecDeque<PathBuf>,
    ) {
        // Read file
        let source_text = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                self.errors.push(BuildError::Io(path.to_path_buf(), e));
                self.add_empty_module(idx, path);
                return;
            }
        };

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap_or(SourceType::mjs());
        let ret = Parser::new(&allocator, &source_text, source_type).parse();

        if ret.panicked {
            let errs: Vec<String> = ret.errors.iter().map(|e| format!("{e:?}")).collect();
            self.errors.push(BuildError::ParseErrors(path.to_path_buf(), errs));
        }

        // Run semantic analysis to get symbol information
        let sem_ret = SemanticBuilder::new().build(&ret.program);
        let scoping = sem_ret.semantic.into_scoping();

        // Ensure symbol DB has space
        let next_module_count = idx.index() + 1;
        self.symbols.ensure_modules(next_module_count);

        // Copy all symbols from scoping into our SymbolRefDb
        let symbol_count = scoping.symbols_len();
        #[expect(clippy::cast_possible_truncation)]
        for sym_id_raw in 0..symbol_count {
            let sym_id = SymbolId::from_raw_unchecked(sym_id_raw as u32);
            let name = scoping.symbol_name(sym_id).to_string();
            self.symbols.add_symbol(idx, name);
        }

        let module_record = &ret.module_record;

        // Build named exports
        let named_exports = build_named_exports(idx, module_record);

        // Build named imports
        let named_imports = build_named_imports(idx, module_record);

        // Resolve imports and build import records + dependency edges
        let dir = path.parent().unwrap_or(Path::new("."));
        let (import_records, dependencies) =
            self.resolve_imports(resolver, dir, module_record, queue);

        // Star export entries
        let star_export_entries = build_star_exports(module_record, &self.path_to_idx);

        // Indirect export entries
        let indirect_export_entries = build_indirect_exports(module_record, &self.path_to_idx);

        // Default export ref and namespace object ref
        // We use synthetic symbols if needed
        let default_export_ref = self.get_or_create_symbol(idx, "__default__");
        let namespace_object_ref = self.get_or_create_symbol(idx, "__namespace__");

        let module = Module {
            idx,
            path: path.to_path_buf(),
            has_module_syntax: module_record.has_module_syntax,
            is_commonjs: false,
            named_exports,
            named_imports,
            import_records,
            default_export_ref,
            namespace_object_ref,
            star_export_entries,
            indirect_export_entries,
            dependencies,
        };

        self.graph.add_module(module);
    }

    fn resolve_imports(
        &mut self,
        resolver: &Resolver,
        dir: &Path,
        module_record: &syntax::ModuleRecord,
        queue: &mut VecDeque<PathBuf>,
    ) -> (Vec<ResolvedImportRecord>, Vec<ImportEdge>) {
        let mut import_records = Vec::new();
        let mut dependencies = Vec::new();

        for (specifier, _) in &module_record.requested_modules {
            let specifier_str = specifier.as_str();
            let resolved = resolver.resolve(dir, specifier_str).ok();

            let target_idx = resolved.as_ref().map(|res| {
                let resolved_path = res.path().to_path_buf();
                if let Some(&existing_idx) = self.path_to_idx.get(&resolved_path) {
                    existing_idx
                } else {
                    // Allocate the next available index.
                    // path_to_idx.len() is the count of all known modules.
                    let new_idx = ModuleIdx::from_usize(self.path_to_idx.len());
                    self.path_to_idx.insert(resolved_path.clone(), new_idx);
                    queue.push_back(resolved_path);
                    new_idx
                }
            });

            import_records.push(ResolvedImportRecord {
                specifier: CompactString::from(specifier_str),
                resolved_module: target_idx,
                kind: ImportKind::Static,
            });

            if let Some(target) = target_idx {
                dependencies.push(ImportEdge {
                    specifier: CompactString::from(specifier_str),
                    target,
                    is_type: false,
                });
            }
        }

        (import_records, dependencies)
    }

    fn add_empty_module(&mut self, idx: ModuleIdx, path: &Path) {
        self.symbols.ensure_modules(idx.index() + 1);
        let default_ref = self.get_or_create_symbol(idx, "__default__");
        let ns_ref = self.get_or_create_symbol(idx, "__namespace__");

        let module = Module {
            idx,
            path: path.to_path_buf(),
            has_module_syntax: false,
            is_commonjs: false,
            named_exports: FxHashMap::default(),
            named_imports: FxHashMap::default(),
            import_records: Vec::new(),
            default_export_ref: default_ref,
            namespace_object_ref: ns_ref,
            star_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            dependencies: Vec::new(),
        };
        self.graph.add_module(module);
    }

    fn get_or_create_symbol(&mut self, module: ModuleIdx, name: &str) -> SymbolRef {
        self.symbols.add_symbol(module, name.to_string())
    }
}

impl Default for ModuleGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Build named exports from the module record.
fn build_named_exports(
    idx: ModuleIdx,
    record: &syntax::ModuleRecord,
) -> FxHashMap<CompactString, LocalExport> {
    let mut exports = FxHashMap::default();

    for entry in &record.local_export_entries {
        let export_name = match &entry.export_name {
            syntax::ExportExportName::Name(ns) => CompactString::from(ns.name.as_str()),
            syntax::ExportExportName::Default(_) => CompactString::new("default"),
            syntax::ExportExportName::Null => continue,
        };

        let local_name = match &entry.local_name {
            syntax::ExportLocalName::Name(ns) | syntax::ExportLocalName::Default(ns) => {
                ns.name.as_str()
            }
            syntax::ExportLocalName::Null => continue,
        };

        // Use a synthetic symbol based on the name.
        // In a real implementation, we'd map to the semantic SymbolId.
        #[expect(clippy::cast_possible_truncation)]
        let symbol = SymbolRef::new(idx, SymbolId::from_raw_unchecked(exports.len() as u32));
        _ = local_name;

        exports.insert(
            export_name.clone(),
            LocalExport { exported_name: export_name, local_symbol: symbol },
        );
    }

    exports
}

/// Build named imports from the module record.
fn build_named_imports(
    idx: ModuleIdx,
    record: &syntax::ModuleRecord,
) -> FxHashMap<SymbolRef, NamedImport> {
    let mut imports = FxHashMap::default();

    for (i, entry) in record.import_entries.iter().enumerate() {
        let imported_name = match &entry.import_name {
            syntax::ImportImportName::Name(ns) => CompactString::from(ns.name.as_str()),
            syntax::ImportImportName::NamespaceObject => CompactString::new("*"),
            syntax::ImportImportName::Default(_) => CompactString::new("default"),
        };

        #[expect(clippy::cast_possible_truncation)]
        let local_symbol = SymbolRef::new(idx, SymbolId::from_raw_unchecked(i as u32));

        imports.insert(
            local_symbol,
            NamedImport {
                imported_name,
                local_symbol,
                record_idx: ImportRecordIdx::from_usize(0), // simplified
                is_type: entry.is_type,
            },
        );
    }

    imports
}

/// Build star export entries.
fn build_star_exports(
    record: &syntax::ModuleRecord,
    path_to_idx: &FxHashMap<PathBuf, ModuleIdx>,
) -> Vec<StarExportEntry> {
    let _ = path_to_idx;
    record
        .star_export_entries
        .iter()
        .filter_map(|entry| {
            let module_request = entry.module_request.as_ref()?;
            Some(StarExportEntry {
                module_request: CompactString::from(module_request.name.as_str()),
                resolved_module: None, // Will be resolved in binding phase
                span: entry.span,
            })
        })
        .collect()
}

/// Build indirect export entries.
fn build_indirect_exports(
    record: &syntax::ModuleRecord,
    path_to_idx: &FxHashMap<PathBuf, ModuleIdx>,
) -> Vec<IndirectExportEntry> {
    let _ = path_to_idx;
    record
        .indirect_export_entries
        .iter()
        .filter_map(|entry| {
            let module_request = entry.module_request.as_ref()?;
            let exported_name = match &entry.export_name {
                syntax::ExportExportName::Name(ns) => CompactString::from(ns.name.as_str()),
                syntax::ExportExportName::Default(_) => CompactString::new("default"),
                syntax::ExportExportName::Null => return None,
            };
            let imported_name = match &entry.import_name {
                syntax::ExportImportName::Name(ns) => CompactString::from(ns.name.as_str()),
                syntax::ExportImportName::All | syntax::ExportImportName::AllButDefault => {
                    CompactString::new("*")
                }
                syntax::ExportImportName::Null => return None,
            };

            Some(IndirectExportEntry {
                exported_name,
                imported_name,
                module_request: CompactString::from(module_request.name.as_str()),
                resolved_module: None,
                span: entry.span,
            })
        })
        .collect()
}
