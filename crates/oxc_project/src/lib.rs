//! Project-level coordination for cross-file type checking.
//!
//! Provides `Project` for managing global types, module resolution, and
//! cross-file export maps. Implements `CheckerHost` so per-file checkers
//! can resolve imports.
//!
//! # Architecture (v1 — single-threaded, lazy resolution)
//!
//! lib.d.ts is treated as file index 0 — checked via the same
//! `ensure_file_checked` path as user files, not via a separate bootstrap.
//! All root scope declarations from lib.d.ts become "exports" accessible
//! via `get_global_type`.
//!
//! Each file's AST and Semantic are kept alive in a `FileCell` (self-referential
//! struct via `self_cell!`). The `Allocator` owns the AST memory, and `Semantic`
//! borrows from it. This enables future cross-file AST access (export *,
//! re-exports, declaration merging) without re-parsing.
//!
//! User files are checked sequentially starting at index 1. When a checker
//! encounters an import, `resolve_import` looks up the export map. If the
//! dependency file hasn't been checked yet, it's checked on demand (lazy).
//! This eliminates the need for topological sorting and handles circular
//! imports naturally (cycle detection returns `any_type` at the re-entry point).
//!
//! # Future parallelism
//!
//! The lazy approach maps naturally to a dataflow task graph: each file's
//! check is a task, imports create dependency edges, rayon work-stealing
//! schedules execution. `RefCell` on the export map swaps to `DashMap`.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Instant;

use oxc_checker::{Checker, allocate_intrinsics, find_lib_source};
use oxc_checker_host::{CheckerHost, IntrinsicIds};
use oxc_diagnostics::OxcDiagnostic;
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::{CompactStr, SourceType};
use oxc_types::{TypeArena, TypeId};
use rustc_hash::{FxHashMap, FxHashSet};

use self_cell::self_cell;

// ---------------------------------------------------------------------------
// FileCell: self-referential struct keeping AST + Semantic alive
// ---------------------------------------------------------------------------

/// Owned data for a parsed file. The `Allocator` owns the AST memory;
/// `FileBorrowed` borrows from it.
struct FileOwned {
    allocator: oxc_allocator::Allocator,
    source: String,
}

/// Borrowed data that references the AST in `FileOwned::allocator`.
/// The `Program` is allocated into the bump arena (via `Allocator::alloc`)
/// so that `Semantic` can reference it with the same lifetime.
struct FileBorrowed<'a> {
    semantic: oxc_semantic::Semantic<'a>,
    program: &'a oxc_ast::ast::Program<'a>,
    module_record: oxc_syntax::module_record::ModuleRecord<'a>,
}

self_cell!(
    /// A parsed and bound file. Owns the allocator (AST memory) and
    /// provides access to the Semantic, Program, and ModuleRecord that
    /// borrow from it. Kept alive in `Project::file_cells` for cross-file
    /// resolution.
    struct FileCell {
        owner: FileOwned,
        #[covariant]
        dependent: FileBorrowed,
    }
);

// ---------------------------------------------------------------------------
// Project
// ---------------------------------------------------------------------------

/// Timing breakdown for benchmarking.
pub struct CheckTiming {
    pub parse_ms: f64,
    pub bind_ms: f64,
    pub check_ms: f64,
    pub total_ms: f64,
}

/// Result of checking a project.
pub struct CheckResult {
    pub diagnostics: Vec<(PathBuf, Vec<OxcDiagnostic>)>,
    pub timing: CheckTiming,
    pub files_checked: usize,
}

/// Index of lib.d.ts in the file arrays. Always slot 0.
const LIB_FILE_INDEX: usize = 0;

/// Project-level state for cross-file type checking.
pub struct Project {
    /// Shared intrinsic type IDs allocated once and reused by all per-file
    /// checkers. Ensures TypeId identity comparisons work across files.
    intrinsics: IntrinsicIds,

    /// Module resolver (None for single-file mode).
    resolver: Option<Resolver>,

    /// Ordered file paths in the project.
    /// Index 0 is always the lib.d.ts slot (synthetic path).
    /// User files start at index 1.
    file_paths: Vec<PathBuf>,

    /// Resolved path → index into file_paths.
    file_index: FxHashMap<PathBuf, usize>,

    /// Source text for each file (read once, taken when FileCell is created).
    /// Index 0 is lib.d.ts source (or None if not found).
    /// Uses RefCell so `ensure_file_checked` can take ownership of the
    /// source text without requiring `&mut self`.
    sources: RefCell<Vec<Option<String>>>,

    /// Parsed and bound files, kept alive for cross-file Semantic access.
    /// Indexed by file index. Populated lazily by `ensure_file_checked`.
    file_cells: RefCell<Vec<Option<FileCell>>>,

    /// Cross-file export map: (file_index, export_name) → TypeId.
    /// For lib.d.ts (index 0): all root scope declared types.
    /// For user files: ES module exports.
    /// Uses RefCell for interior mutability — checkers hold &self (via
    /// CheckerHost) while exports are being populated. Swappable to
    /// DashMap for the parallel version.
    export_types: RefCell<FxHashMap<(usize, CompactStr), TypeId>>,

    /// Accumulated type parameter constraints from all checked files.
    /// Keyed by TypeParameter TypeId → constraint TypeId. Since TypeIds
    /// are globally unique (shared arena), there are no collisions.
    /// Per-file checkers query this via `get_type_param_constraint` when
    /// they encounter a type parameter from another file.
    type_param_constraints: RefCell<FxHashMap<TypeId, TypeId>>,

    /// Set of files currently being checked, for circular import detection.
    checking: RefCell<FxHashSet<usize>>,

    /// Set of files that have been fully checked.
    checked: RefCell<FxHashSet<usize>>,

    /// Shared type arena reference for on-demand checking.
    /// Set during check_all, None otherwise.
    arena: Option<*const TypeArena>,

    /// Accumulated timing.
    parse_ms: RefCell<f64>,
    bind_ms: RefCell<f64>,
    check_ms: RefCell<f64>,

    /// Accumulated diagnostics from on-demand checking.
    all_diagnostics: RefCell<Vec<(PathBuf, Vec<OxcDiagnostic>)>>,
}

// SAFETY: Project is only used single-threaded in v1. The raw pointer
// to TypeArena is valid for the duration of check_all. For the parallel
// version, this will be replaced with proper lifetime-safe references.
unsafe impl Send for Project {}
unsafe impl Sync for Project {}

impl Project {
    /// Create a project with lib.d.ts as file 0.
    ///
    /// lib.d.ts is checked eagerly so the Project is immediately usable
    /// as a `CheckerHost` (global types available, intrinsics allocated).
    /// The arena pointer is retained for lazy `resolve_import` calls.
    pub fn new(arena: &TypeArena) -> Self {
        let intrinsics = allocate_intrinsics(arena);
        let lib_source = find_lib_source();

        let project = Self {
            intrinsics,
            resolver: None,
            // lib.d.ts is file 0
            file_paths: vec![PathBuf::from("<lib.es5.d.ts>")],
            file_index: FxHashMap::default(),
            sources: RefCell::new(vec![lib_source]),
            file_cells: RefCell::new(vec![None]),
            export_types: RefCell::new(FxHashMap::default()),
            type_param_constraints: RefCell::new(FxHashMap::default()),
            checking: RefCell::new(FxHashSet::default()),
            checked: RefCell::new(FxHashSet::default()),
            arena: Some(arena as *const TypeArena),
            parse_ms: RefCell::new(0.0),
            bind_ms: RefCell::new(0.0),
            check_ms: RefCell::new(0.0),
            all_diagnostics: RefCell::new(Vec::new()),
        };
        project.ensure_file_checked(LIB_FILE_INDEX);
        project
    }

    /// Create a project for multi-file checking.
    ///
    /// lib.d.ts is prepended as file 0. User files start at index 1.
    pub fn new_multi(arena: &TypeArena, user_file_paths: Vec<PathBuf>) -> Self {
        let intrinsics = allocate_intrinsics(arena);
        let lib_source = find_lib_source();

        // Prepend lib.d.ts as file 0, user files start at index 1
        let mut file_paths = Vec::with_capacity(1 + user_file_paths.len());
        file_paths.push(PathBuf::from("<lib.es5.d.ts>"));
        file_paths.extend(user_file_paths);

        // File index maps canonical paths to indices (for module resolution).
        // lib.d.ts uses a synthetic path and isn't resolved via imports.
        let file_index: FxHashMap<PathBuf, usize> = file_paths
            .iter()
            .enumerate()
            .skip(1) // skip lib.d.ts synthetic path
            .map(|(i, p)| (p.clone(), i))
            .collect();

        // Read all sources upfront. Index 0 is lib.d.ts.
        let mut sources = Vec::with_capacity(file_paths.len());
        sources.push(lib_source);
        for path in file_paths.iter().skip(1) {
            sources.push(std::fs::read_to_string(path).ok());
        }

        let file_count = file_paths.len();

        let resolver = Resolver::new(ResolveOptions {
            extensions: vec![
                ".ts".to_string(),
                ".tsx".to_string(),
                ".d.ts".to_string(),
                ".js".to_string(),
                ".jsx".to_string(),
            ],
            ..ResolveOptions::default()
        });

        let project = Self {
            intrinsics,
            resolver: Some(resolver),
            file_paths,
            file_index,
            sources: RefCell::new(sources),
            file_cells: RefCell::new((0..file_count).map(|_| None).collect()),
            export_types: RefCell::new(FxHashMap::default()),
            type_param_constraints: RefCell::new(FxHashMap::default()),
            checking: RefCell::new(FxHashSet::default()),
            checked: RefCell::new(FxHashSet::default()),
            arena: Some(arena as *const TypeArena),
            parse_ms: RefCell::new(0.0),
            bind_ms: RefCell::new(0.0),
            check_ms: RefCell::new(0.0),
            all_diagnostics: RefCell::new(Vec::new()),
        };
        project.ensure_file_checked(LIB_FILE_INDEX);
        project
    }

    /// Check all files in the project.
    ///
    /// lib.d.ts (index 0) was already checked during construction.
    /// User files are checked sequentially. Import dependencies are
    /// resolved lazily — when a checker needs a type from another file,
    /// that file is checked on demand.
    pub fn check_all(&mut self) -> CheckResult {
        let total_start = Instant::now();

        // Check all files. Index 0 (lib.d.ts) is already checked.
        // Files may trigger on-demand checking of dependencies via
        // resolve_import → ensure_file_checked.
        for i in 0..self.file_paths.len() {
            self.ensure_file_checked(i);
        }
        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
        // Don't count lib.d.ts in the user-facing file count
        let files_checked = self.checked.borrow().len().saturating_sub(1);

        CheckResult {
            diagnostics: self.all_diagnostics.take(),
            timing: CheckTiming {
                parse_ms: *self.parse_ms.borrow(),
                bind_ms: *self.bind_ms.borrow(),
                check_ms: *self.check_ms.borrow(),
                total_ms,
            },
            files_checked,
        }
    }

    /// Ensure a file has been checked, checking it on demand if needed.
    /// Handles circular imports via cycle detection (returns without
    /// checking if the file is already being checked).
    fn ensure_file_checked(&self, file_idx: usize) {
        // Already fully checked
        if self.checked.borrow().contains(&file_idx) {
            return;
        }

        // Currently being checked — circular import, skip
        if !self.checking.borrow_mut().insert(file_idx) {
            return;
        }

        // Take the source text — FileCell takes ownership, no duplication.
        let Some(source) = self.sources.borrow_mut()[file_idx].take() else {
            self.checking.borrow_mut().remove(&file_idx);
            self.checked.borrow_mut().insert(file_idx);
            return;
        };

        // SAFETY: arena pointer is valid for the duration of check_all
        let arena = unsafe { &*self.arena.unwrap() };

        let is_lib = file_idx == LIB_FILE_INDEX;
        let source_type = if is_lib {
            SourceType::d_ts()
        } else {
            SourceType::from_path(&self.file_paths[file_idx]).unwrap_or_default()
        };
        let path_str = self.file_paths[file_idx].to_string_lossy().to_string();

        // Parse + bind inside FileCell builder. The Allocator owns the AST
        // memory; Semantic, Program, and ModuleRecord borrow from it.
        // Timing is recorded via Cell so the builder closure can write to it.
        let parse_elapsed = std::cell::Cell::new(0.0f64);
        let bind_elapsed = std::cell::Cell::new(0.0f64);
        let file_cell = FileCell::new(
            FileOwned {
                allocator: oxc_allocator::Allocator::default(),
                source,
            },
            |owned| {
                let parse_start = Instant::now();
                let parsed = oxc_parser::Parser::new(
                    &owned.allocator,
                    &owned.source,
                    source_type,
                ).parse();
                parse_elapsed.set(parse_start.elapsed().as_secs_f64() * 1000.0);

                // Destructure to move program and module_record out
                let oxc_parser::ParserReturn { program, module_record, .. } = parsed;
                // Park program in the bump allocator so Semantic can
                // reference it with the allocator's lifetime.
                let program = owned.allocator.alloc(program);

                let bind_start = Instant::now();
                let semantic = oxc_semantic::SemanticBuilder::new()
                    .build(program)
                    .semantic;
                bind_elapsed.set(bind_start.elapsed().as_secs_f64() * 1000.0);

                FileBorrowed { semantic, program, module_record }
            },
        );
        *self.parse_ms.borrow_mut() += parse_elapsed.get();
        *self.bind_ms.borrow_mut() += bind_elapsed.get();

        // Check phase — borrows Semantic from FileCell. The Checker is
        // temporary; the FileCell (and its Semantic) outlive it.
        let check_start = Instant::now();
        let exports;
        let file_constraints;
        let diagnostics;
        {
            let borrowed = file_cell.borrow_dependent();
            let mut checker = Checker::new_with_host(
                &borrowed.semantic,
                arena,
                self,
                path_str,
                file_idx as u16,
            );
            checker.check_program(&borrowed.program);

            // Extract exports while checker is alive.
            exports = if !borrowed.module_record.has_module_syntax {
                // Ambient file (lib.d.ts or .d.ts without module syntax):
                // export all root scope declared types.
                Self::extract_ambient_declarations(&mut checker)
            } else {
                // ES module: export only explicitly exported bindings.
                Self::extract_module_exports(&mut checker, &borrowed.module_record)
            };

            // For lib.d.ts: eagerly resolve all type param constraints
            // so they're available to per-file checkers after the checker
            // is dropped. Regular files resolve constraints lazily.
            if is_lib {
                checker.eagerly_resolve_type_param_constraints();
            }

            // Extract caches from checker before it's dropped.
            file_constraints = checker.take_type_param_constraints();
            diagnostics = checker.take_diagnostics();
        } // checker dropped; FileCell borrow released
        *self.check_ms.borrow_mut() += check_start.elapsed().as_secs_f64() * 1000.0;

        // Store FileCell — Semantic stays alive for future cross-file access.
        self.file_cells.borrow_mut()[file_idx] = Some(file_cell);

        // Record exports
        {
            let mut export_map = self.export_types.borrow_mut();
            for (name, type_id) in exports {
                export_map.insert((file_idx, name), type_id);
            }
        }

        // Merge constraints
        if !file_constraints.is_empty() {
            self.type_param_constraints.borrow_mut().extend(file_constraints);
        }

        if !diagnostics.is_empty() {
            self.all_diagnostics.borrow_mut().push(
                (self.file_paths[file_idx].clone(), diagnostics)
            );
        }

        self.checking.borrow_mut().remove(&file_idx);
        self.checked.borrow_mut().insert(file_idx);
    }

    /// Extract all root scope declared types from an ambient file (lib.d.ts
    /// or .d.ts without module syntax). All declarations are globally visible.
    fn extract_ambient_declarations(
        checker: &mut Checker<'_>,
    ) -> Vec<(CompactStr, TypeId)> {
        let root_scope = checker.semantic().scoping().root_scope_id();
        let symbols: Vec<oxc_syntax::symbol::SymbolId> = checker
            .semantic()
            .scoping()
            .iter_bindings_in(root_scope)
            .collect();

        let mut exports = Vec::new();
        for symbol_id in symbols {
            let name = checker.semantic().scoping().symbol_name(symbol_id).to_string();
            let type_id = checker.get_declared_type_of_symbol(symbol_id);
            if type_id != checker.any_type {
                exports.push((CompactStr::new(&name), type_id));
            }
        }
        exports
    }

    /// Extract ES module exports from a checked file.
    /// Eagerly resolves types for exported symbols that weren't accessed during checking.
    fn extract_module_exports(
        checker: &mut Checker<'_>,
        module_record: &oxc_syntax::module_record::ModuleRecord<'_>,
    ) -> Vec<(CompactStr, TypeId)> {
        use oxc_syntax::module_record::{ExportExportName, ExportLocalName};

        let root_scope = checker.semantic().scoping().root_scope_id();
        let mut exports = Vec::new();

        // Collect into owned Strings to avoid borrowing checker during the loop
        let binding_map: FxHashMap<String, oxc_syntax::symbol::SymbolId> = checker
            .semantic()
            .scoping()
            .iter_bindings_in(root_scope)
            .map(|sym_id| {
                (checker.semantic().scoping().symbol_name(sym_id).to_string(), sym_id)
            })
            .collect();

        for entry in &module_record.local_export_entries {
            let export_name = match &entry.export_name {
                ExportExportName::Name(n) => n.name.as_str(),
                ExportExportName::Default(_) => "default",
                ExportExportName::Null => continue,
            };

            let local_name = match &entry.local_name {
                ExportLocalName::Name(n) | ExportLocalName::Default(n) => n.name.as_str(),
                ExportLocalName::Null => continue,
            };

            let Some(&symbol_id) = binding_map.get(&local_name.to_string()) else {
                continue;
            };

            // Eagerly resolve the type if not already cached (it may not
            // have been referenced during check_program).
            let type_id = checker.get_cached_symbol_type(symbol_id)
                .or_else(|| checker.get_cached_declared_type(symbol_id))
                .unwrap_or_else(|| {
                    // Try both value-side and type-side resolution
                    let t = checker.get_type_of_symbol(symbol_id);
                    if t != checker.any_type { return t; }
                    checker.get_declared_type_of_symbol(symbol_id)
                });
            let type_id = Some(type_id).filter(|&t| t != checker.any_type);

            if let Some(type_id) = type_id {
                exports.push((CompactStr::new(export_name), type_id));
            }
        }

        exports
    }

    /// Resolve a module specifier from a given file to a file index.
    #[allow(dead_code)]
    fn resolve_module_to_index(&self, _from_idx: usize, specifier: &str) -> Option<usize> {
        let resolver = self.resolver.as_ref()?;
        let from_dir = self.file_paths[_from_idx].parent()?;
        let resolution = resolver.resolve(from_dir, specifier).ok()?;
        let resolved_path = resolution.path().canonicalize().ok()?;
        self.file_index.get(&resolved_path).copied()
    }
}

impl CheckerHost for Project {
    fn get_intrinsics(&self) -> IntrinsicIds {
        self.intrinsics
    }

    fn get_global_type(&self, name: &str) -> Option<TypeId> {
        // Ensure lib.d.ts has been checked
        self.ensure_file_checked(LIB_FILE_INDEX);
        self.export_types.borrow().get(&(LIB_FILE_INDEX, CompactStr::new(name))).copied()
    }

    fn resolve_import(
        &self,
        from_file: &str,
        module_specifier: &str,
        export_name: &str,
    ) -> Option<TypeId> {
        let from_path = Path::new(from_file);
        let from_dir = from_path.parent()?;
        let resolver = self.resolver.as_ref()?;
        let resolution = resolver.resolve(from_dir, module_specifier).ok()?;
        let resolved_path = resolution.path().canonicalize().ok()?;
        let &file_idx = self.file_index.get(&resolved_path)?;

        // Lazy: ensure the dependency file is checked before looking up its exports
        self.ensure_file_checked(file_idx);

        self.export_types.borrow().get(&(file_idx, CompactStr::new(export_name))).copied()
    }

    fn get_type_param_constraint(&self, type_id: TypeId) -> Option<TypeId> {
        self.type_param_constraints.borrow().get(&type_id).copied()
    }

    fn get_symbol_name(&self, file_idx: u16, symbol_id: oxc_syntax::symbol::SymbolId) -> Option<CompactStr> {
        let cells = self.file_cells.borrow();
        let cell = cells.get(file_idx as usize)?.as_ref()?;
        let name = cell.borrow_dependent().semantic.scoping().symbol_name(symbol_id);
        Some(CompactStr::new(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_loads_global_types() {
        let arena = TypeArena::with_capacity(64);
        let project = Project::new(&arena);
        let result = project.get_global_type("Array");
        assert!(result.is_some(), "Array should be found in lib.d.ts");
    }

    #[test]
    fn project_implements_checker_host() {
        let arena = TypeArena::with_capacity(64);
        let project = Project::new(&arena);
        let host: &dyn CheckerHost = &project;
        assert!(host.resolve_import("test.ts", "./foo", "x").is_none());
    }
}
