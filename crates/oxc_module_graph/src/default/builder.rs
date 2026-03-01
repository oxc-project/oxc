use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::{fs, io};

use compact_str::CompactString;
use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrowFunctionExpression, AwaitExpression, Function, Program};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::SourceType;
use oxc_syntax::module_record as syntax;
use oxc_syntax::scope::ScopeFlags;
use oxc_syntax::symbol::SymbolId;

use crate::graph::ModuleGraph;
use crate::module::{ExternalModule, NormalModule, SideEffects};
use crate::types::{
    ExportsKind, ImportKind, ImportRecordIdx, ImportRecordMeta, IndirectExportEntry, LocalExport,
    ModuleIdx, NamedImport, ResolvedImportRecord, StarExportEntry, SymbolRef, WrapKind,
};

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
    pub graph: ModuleGraph,
    /// Non-fatal errors encountered during building.
    pub errors: Vec<BuildError>,
}

/// Builds a `ModuleGraph` from entry points.
///
/// Uses `oxc_parser` + `oxc_semantic` to parse files,
/// and `oxc_resolver` to resolve import specifiers via BFS.
#[derive(Debug)]
pub struct ModuleGraphBuilder {
    /// The module graph being built.
    pub graph: ModuleGraph,
    /// Maps absolute paths to module indices (deduplication).
    path_to_idx: FxHashMap<PathBuf, ModuleIdx>,
    /// Maps external specifiers to module indices (deduplication).
    external_specifier_to_idx: FxHashMap<CompactString, ModuleIdx>,
    /// Non-fatal errors.
    errors: Vec<BuildError>,
}

impl ModuleGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: ModuleGraph::new(),
            path_to_idx: FxHashMap::default(),
            external_specifier_to_idx: FxHashMap::default(),
            errors: Vec::new(),
        }
    }

    /// Build the module graph starting from the given entry point paths.
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
        let mut entry_indices = Vec::new();

        // Enqueue entry points.
        for entry in entries {
            let abs = if entry.is_absolute() {
                entry.clone()
            } else {
                std::env::current_dir().unwrap_or_default().join(entry)
            };
            let canonical = fs::canonicalize(&abs).unwrap_or(abs);
            if !self.path_to_idx.contains_key(&canonical) {
                let idx = self.graph.alloc_module_idx();
                self.path_to_idx.insert(canonical.clone(), idx);
                entry_indices.push(idx);
                queue.push_back(canonical);
            }
        }

        self.graph.set_entries(entry_indices);

        // BFS
        while let Some(path) = queue.pop_front() {
            let idx = self.path_to_idx[&path];
            self.process_module(&resolver, &path, idx, &mut queue);
        }

        BuildResult { graph: self.graph, errors: self.errors }
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

        // Copy all symbols from scoping into our SymbolRefDb without remapping
        // their semantic SymbolIds.
        let symbol_count = scoping.symbols_len();
        self.graph.ensure_module_symbol_capacity(idx, symbol_count);
        #[expect(clippy::cast_possible_truncation)]
        for sym_id_raw in 0..symbol_count {
            let sym_id = SymbolId::from_raw_unchecked(sym_id_raw as u32);
            let name = scoping.symbol_name(sym_id).to_string();
            self.graph.set_symbol_name(idx, sym_id, name);
            self.graph.init_symbol_self_link(idx, sym_id);
        }

        let module_record = &ret.module_record;

        // Build named exports (Fix 1: use real SymbolIds from scoping)
        let named_exports = build_named_exports(idx, module_record, &scoping);

        // Build specifier→record_idx mapping for named imports.
        let specifier_to_record_idx: FxHashMap<&str, usize> = module_record
            .requested_modules
            .keys()
            .enumerate()
            .map(|(i, k)| (k.as_str(), i))
            .collect();

        // Build named imports (Fix 1: use real SymbolIds from scoping)
        let named_imports =
            build_named_imports(idx, module_record, &scoping, &specifier_to_record_idx);

        // Resolve imports and build import records (Fix 4: creates ExternalModules for bare specifiers)
        let dir = path.parent().unwrap_or(Path::new("."));
        let import_records = self.resolve_imports(resolver, dir, module_record, queue);

        // Star export entries (Fix 2: resolve targets via resolver)
        let star_export_entries =
            build_star_exports(module_record, resolver, dir, &self.path_to_idx);

        // Indirect export entries (Fix 2: resolve targets via resolver)
        let indirect_export_entries =
            build_indirect_exports(module_record, resolver, dir, &self.path_to_idx);

        // Default export ref and namespace object ref
        let default_export_ref = self.get_or_create_symbol(idx, "__default__");
        let namespace_object_ref = self.get_or_create_symbol(idx, "__namespace__");

        // Fix 3: detect top-level await from AST
        let tla = has_top_level_await(&ret.program);

        let module = NormalModule {
            idx,
            path: path.to_path_buf(),
            has_module_syntax: module_record.has_module_syntax,
            exports_kind: ExportsKind::None,
            has_top_level_await: tla,
            side_effects: SideEffects::True,
            named_exports,
            named_imports,
            import_records,
            default_export_ref,
            namespace_object_ref,
            star_export_entries,
            indirect_export_entries,
            has_lazy_export: false,
            execution_order_sensitive: false,
            // Link-time results — initialized to defaults.
            wrap_kind: WrapKind::None,
            original_wrap_kind: WrapKind::None,
            wrapper_ref: None,
            required_by_other_module: false,
            resolved_exports: FxHashMap::default(),
            has_dynamic_exports: false,
            is_tla_or_contains_tla: false,
            propagated_side_effects: false,
            exec_order: u32::MAX,
        };

        self.graph.add_normal_module(module);
    }

    /// Resolve static ESM imports from `requested_modules`.
    ///
    /// When resolution fails for a bare specifier (not starting with `.` or `/`),
    /// an `ExternalModule` is created so the graph can represent unresolvable
    /// dependencies like `"react"` or `"lodash"`.
    fn resolve_imports(
        &mut self,
        resolver: &Resolver,
        dir: &Path,
        module_record: &syntax::ModuleRecord,
        queue: &mut VecDeque<PathBuf>,
    ) -> Vec<ResolvedImportRecord> {
        let mut import_records = Vec::new();

        for (specifier, _) in &module_record.requested_modules {
            let specifier_str = specifier.as_str();

            let target_idx = match resolver.resolve(dir, specifier_str) {
                Ok(res) => {
                    let resolved_path = res.path().to_path_buf();
                    if let Some(&existing_idx) = self.path_to_idx.get(&resolved_path) {
                        Some(existing_idx)
                    } else {
                        let new_idx = self.graph.alloc_module_idx();
                        self.path_to_idx.insert(resolved_path.clone(), new_idx);
                        queue.push_back(resolved_path);
                        Some(new_idx)
                    }
                }
                Err(_) => {
                    // Fix 4: Create ExternalModule for bare specifiers
                    if !specifier_str.starts_with('.')
                        && !specifier_str.starts_with('/')
                        && !specifier_str.starts_with('#')
                    {
                        let compact_spec = CompactString::from(specifier_str);
                        if let Some(&existing_idx) =
                            self.external_specifier_to_idx.get(&compact_spec)
                        {
                            Some(existing_idx)
                        } else {
                            let ext_idx = self.graph.alloc_module_idx();
                            let ns_ref =
                                self.graph.add_symbol(ext_idx, format!("{specifier_str}_ns"));
                            self.graph.add_external_module(ExternalModule {
                                idx: ext_idx,
                                specifier: compact_spec.clone(),
                                side_effects: SideEffects::True,
                                namespace_ref: ns_ref,
                                exec_order: u32::MAX,
                            });
                            self.external_specifier_to_idx.insert(compact_spec, ext_idx);
                            Some(ext_idx)
                        }
                    } else {
                        None // Truly unresolvable relative import
                    }
                }
            };

            // Dummy namespace_ref — the builder doesn't do linking.
            // Consumers that need real namespace_refs populate them during link.
            let namespace_ref = SymbolRef::new(
                target_idx.unwrap_or(ModuleIdx::from_usize(0)),
                SymbolId::from_raw_unchecked(0),
            );

            import_records.push(ResolvedImportRecord {
                specifier: CompactString::from(specifier_str),
                resolved_module: target_idx,
                kind: ImportKind::Static,
                namespace_ref,
                meta: ImportRecordMeta::empty(),
            });
        }

        import_records
    }

    fn add_empty_module(&mut self, idx: ModuleIdx, path: &Path) {
        let default_ref = self.get_or_create_symbol(idx, "__default__");
        let ns_ref = self.get_or_create_symbol(idx, "__namespace__");

        let module = NormalModule {
            idx,
            path: path.to_path_buf(),
            has_module_syntax: false,
            exports_kind: ExportsKind::None,
            has_top_level_await: false,
            side_effects: SideEffects::True,
            has_lazy_export: false,
            execution_order_sensitive: false,
            named_exports: FxHashMap::default(),
            named_imports: FxHashMap::default(),
            import_records: Vec::new(),
            default_export_ref: default_ref,
            namespace_object_ref: ns_ref,
            star_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            wrap_kind: WrapKind::None,
            original_wrap_kind: WrapKind::None,
            wrapper_ref: None,
            required_by_other_module: false,
            resolved_exports: FxHashMap::default(),
            has_dynamic_exports: false,
            is_tla_or_contains_tla: false,
            propagated_side_effects: false,
            exec_order: u32::MAX,
        };
        self.graph.add_normal_module(module);
    }

    fn get_or_create_symbol(&mut self, module: ModuleIdx, name: &str) -> SymbolRef {
        self.graph.alloc_synthetic_symbol(module, name.to_string())
    }
}

impl Default for ModuleGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect top-level `await` by visiting the AST.
///
/// By not recursing into `Function` or `ArrowFunctionExpression`, we only find
/// `await` at the module's top level (including top-level class static blocks,
/// which is correct — those are evaluated at module evaluation time).
fn has_top_level_await(program: &Program) -> bool {
    struct TlaDetector {
        found: bool,
    }

    impl<'a> Visit<'a> for TlaDetector {
        fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
            // Don't recurse — await inside functions is not top-level.
        }

        fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {
            // Don't recurse — await inside arrows is not top-level.
        }

        fn visit_await_expression(&mut self, _it: &AwaitExpression<'a>) {
            self.found = true;
        }
    }

    let mut detector = TlaDetector { found: false };
    detector.visit_program(program);
    detector.found
}

/// Build named exports from the module record.
///
/// Uses the `Scoping` from semantic analysis to look up real `SymbolId`s
/// for each export's local binding.
fn build_named_exports(
    idx: ModuleIdx,
    record: &syntax::ModuleRecord,
    scoping: &Scoping,
) -> FxHashMap<CompactString, LocalExport> {
    let mut exports = FxHashMap::default();
    let root_scope = scoping.root_scope_id();

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

        // Look up the real SymbolId from semantic analysis.
        let symbol = if let Some(sym_id) = scoping.find_binding(root_scope, local_name.into()) {
            SymbolRef::new(idx, sym_id)
        } else {
            // Fallback for edge cases (e.g., `export default <expr>` with no binding).
            #[expect(clippy::cast_possible_truncation)]
            SymbolRef::new(idx, SymbolId::from_raw_unchecked(exports.len() as u32))
        };

        exports.insert(
            export_name.clone(),
            LocalExport { exported_name: export_name, local_symbol: symbol },
        );
    }

    exports
}

/// Build named imports from the module record.
///
/// Uses the `Scoping` from semantic analysis to look up real `SymbolId`s
/// for each import's local binding.
fn build_named_imports(
    idx: ModuleIdx,
    record: &syntax::ModuleRecord,
    scoping: &Scoping,
    specifier_to_record_idx: &FxHashMap<&str, usize>,
) -> FxHashMap<SymbolRef, NamedImport> {
    let mut imports = FxHashMap::default();
    let root_scope = scoping.root_scope_id();

    for entry in &record.import_entries {
        let imported_name = match &entry.import_name {
            syntax::ImportImportName::Name(ns) => CompactString::from(ns.name.as_str()),
            syntax::ImportImportName::NamespaceObject => CompactString::new("*"),
            syntax::ImportImportName::Default(_) => CompactString::new("default"),
        };

        let local_name = entry.local_name.name.as_str();

        // Look up the real SymbolId from semantic analysis.
        let local_symbol = if let Some(sym_id) = scoping.find_binding(root_scope, local_name.into())
        {
            SymbolRef::new(idx, sym_id)
        } else {
            // Fallback: use import count as synthetic ID.
            #[expect(clippy::cast_possible_truncation)]
            SymbolRef::new(idx, SymbolId::from_raw_unchecked(imports.len() as u32))
        };

        let record_idx =
            specifier_to_record_idx.get(entry.module_request.name.as_str()).copied().unwrap_or(0);

        imports.insert(
            local_symbol,
            NamedImport {
                imported_name,
                local_symbol,
                record_idx: ImportRecordIdx::from_usize(record_idx),
                is_type: entry.is_type,
            },
        );
    }

    imports
}

/// Build star export entries, resolving module targets via the resolver.
fn build_star_exports(
    record: &syntax::ModuleRecord,
    resolver: &Resolver,
    dir: &Path,
    path_to_idx: &FxHashMap<PathBuf, ModuleIdx>,
) -> Vec<StarExportEntry> {
    record
        .star_export_entries
        .iter()
        .filter_map(|entry| {
            let module_request = entry.module_request.as_ref()?;
            let specifier = module_request.name.as_str();

            // Resolve the module specifier to find the target module index.
            let resolved_module = resolver
                .resolve(dir, specifier)
                .ok()
                .and_then(|res| path_to_idx.get(&res.path().to_path_buf()).copied());

            Some(StarExportEntry {
                module_request: CompactString::from(specifier),
                resolved_module,
                span: entry.span,
            })
        })
        .collect()
}

/// Build indirect export entries, resolving module targets via the resolver.
fn build_indirect_exports(
    record: &syntax::ModuleRecord,
    resolver: &Resolver,
    dir: &Path,
    path_to_idx: &FxHashMap<PathBuf, ModuleIdx>,
) -> Vec<IndirectExportEntry> {
    record
        .indirect_export_entries
        .iter()
        .filter_map(|entry| {
            let module_request = entry.module_request.as_ref()?;
            let specifier = module_request.name.as_str();

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

            // Resolve the module specifier to find the target module index.
            let resolved_module = resolver
                .resolve(dir, specifier)
                .ok()
                .and_then(|res| path_to_idx.get(&res.path().to_path_buf()).copied());

            Some(IndirectExportEntry {
                exported_name,
                imported_name,
                module_request: CompactString::from(specifier),
                resolved_module,
                span: entry.span,
            })
        })
        .collect()
}
