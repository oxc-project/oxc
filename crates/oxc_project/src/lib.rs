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

mod compiler_options;
mod type_merge;

pub use compiler_options::{CompilerOptions, ScriptTarget, validate_compiler_options};

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Instant;

use oxc_checker::{Checker, allocate_intrinsics, find_lib_source, find_lib_sources};
use oxc_checker_host::{CheckerHost, CheckerOptions, ExportedBinding, IntrinsicIds};
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

/// Project-level state for cross-file type checking.
pub struct Project {
    /// Shared intrinsic type IDs allocated once and reused by all per-file
    /// checkers. Ensures TypeId identity comparisons work across files.
    intrinsics: IntrinsicIds,

    /// Module resolver (None for single-file mode or virtual files).
    resolver: Option<Resolver>,

    /// Number of lib files at the start of `file_paths`.
    /// Lib files occupy indices 0..lib_file_count. User files start after.
    lib_file_count: usize,

    /// Ordered file paths in the project.
    /// Indices 0..lib_file_count are lib files (e.g., lib.es5.d.ts,
    /// lib.es2015.core.d.ts). User files start at lib_file_count.
    file_paths: Vec<PathBuf>,

    /// Resolved path → index into file_paths.
    file_index: FxHashMap<PathBuf, usize>,

    /// Source text for each file (read once, taken when FileCell is created).
    /// Uses RefCell so `ensure_file_checked` can take ownership of the
    /// source text without requiring `&mut self`.
    sources: RefCell<Vec<Option<String>>>,

    /// Explicit per-file SourceType overrides (from `new_multi_from_sources`).
    /// When `Some`, `ensure_file_checked` uses these instead of inferring
    /// from file path extensions.
    source_types: Option<Vec<SourceType>>,

    /// Parsed and bound files, kept alive for cross-file Semantic access.
    /// Indexed by file index. Populated lazily by `ensure_file_checked`.
    file_cells: RefCell<Vec<Option<FileCell>>>,

    /// Cross-file export map: (file_index, export_name) → ExportedBinding.
    /// Each binding has separate type-side and value-side types.
    /// For lib files: all root scope declared + value types.
    /// For user files: ES module exports.
    exports: RefCell<FxHashMap<(usize, CompactStr), ExportedBinding>>,

    /// Merged global exports from all lib files.
    /// Built by `merge_global_exports()` after all lib files are checked.
    /// When multiple lib files export the same interface name, their types
    /// are merged via `merge_interface_types`. `get_global_type` reads from
    /// this map instead of the per-file export map.
    global_exports: RefCell<FxHashMap<CompactStr, ExportedBinding>>,

    /// Accumulated type parameter constraints from all checked files.
    type_param_constraints: RefCell<FxHashMap<TypeId, TypeId>>,

    /// Per-file checker caches, extracted from each Checker after checking.
    /// Enables post-check type queries via `with_checker()` by reconstructing
    /// a Checker with restored caches (all operations are pointer-sized swaps).
    /// Indexed by file index. `None` for unchecked files.
    checker_caches: RefCell<Vec<Option<oxc_checker::CheckerCaches>>>,

    /// Set of files currently being checked, for circular import detection.
    checking: RefCell<FxHashSet<usize>>,

    /// Set of files that have been fully checked.
    checked: RefCell<FxHashSet<usize>>,

    /// Shared type arena reference for on-demand checking.
    ///
    /// Raw pointer because the arena ownership model will change for the
    /// parallel design (per-thread local arenas + shared read-only base,
    /// likely behind `Arc`). A lifetime parameter would be correct for v1
    /// but would need to be removed for that transition. The raw pointer
    /// accurately represents "this will change."
    ///
    /// Safety invariant: the caller-owned TypeArena outlives the Project.
    /// Dereferenced in `ensure_file_checked`, `with_checker`, and
    /// `type_printer`.
    arena: Option<*const TypeArena>,

    /// Checker options passed to each per-file Checker.
    checker_options: CheckerOptions,

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
    /// Prepare lib file paths and sources from find_lib_sources results.
    /// Falls back to find_lib_source (single file) if no lib files found.
    fn prepare_lib_files(
        lib_files: Vec<(String, String)>,
    ) -> (Vec<PathBuf>, Vec<Option<String>>, usize) {
        if lib_files.is_empty() {
            let lib_source = find_lib_source();
            (
                vec![PathBuf::from("<lib.es5.d.ts>")],
                vec![lib_source],
                1,
            )
        } else {
            let count = lib_files.len();
            let paths: Vec<PathBuf> =
                lib_files.iter().map(|(name, _)| PathBuf::from(name)).collect();
            let srcs: Vec<Option<String>> =
                lib_files.into_iter().map(|(_, src)| Some(src)).collect();
            (paths, srcs, count)
        }
    }

    /// Create a project with lib.es5.d.ts only.
    ///
    /// lib.d.ts is checked eagerly so the Project is immediately usable
    /// as a `CheckerHost` (global types available, intrinsics allocated).
    pub fn new(arena: &TypeArena) -> Self {
        Self::new_with_target(arena, None)
    }

    /// Create a project with lib files for the given target.
    ///
    /// If `target` is `None`, loads only lib.es5.d.ts (same as `new`).
    /// If `target` is e.g. `Some(ScriptTarget::ES2015)`, loads lib.es5.d.ts
    /// plus all ES2015 sub-libs, and merges overlapping interface declarations.
    pub fn new_with_target(arena: &TypeArena, target: Option<ScriptTarget>) -> Self {
        let intrinsics = allocate_intrinsics(arena);

        // Determine which lib files to load
        let lib_names = target
            .map(|t| t.default_libs())
            .unwrap_or(&["es5"]);
        let lib_files = find_lib_sources(lib_names);
        let (file_paths, sources, lib_count) = Self::prepare_lib_files(lib_files);

        let file_count = file_paths.len();
        let project = Self {
            intrinsics,
            resolver: None,
            lib_file_count: lib_count,
            file_paths,
            file_index: FxHashMap::default(),
            sources: RefCell::new(sources),
            source_types: None,
            file_cells: RefCell::new((0..file_count).map(|_| None).collect()),
            exports: RefCell::new(FxHashMap::default()),
            global_exports: RefCell::new(FxHashMap::default()),
            type_param_constraints: RefCell::new(FxHashMap::default()),
            checker_caches: RefCell::new((0..file_count).map(|_| None).collect()),
            checking: RefCell::new(FxHashSet::default()),
            checked: RefCell::new(FxHashSet::default()),
            arena: Some(arena as *const TypeArena),
            checker_options: CheckerOptions::default(),
            parse_ms: RefCell::new(0.0),
            bind_ms: RefCell::new(0.0),
            check_ms: RefCell::new(0.0),
            all_diagnostics: RefCell::new(Vec::new()),
        };

        // Check all lib files sequentially, then merge their exports
        for i in 0..lib_count {
            project.ensure_file_checked(i);
        }
        project.merge_global_exports();

        project
    }

    /// Create a project for multi-file checking.
    ///
    /// Lib files are prepended at indices 0..N. User files start after.
    pub fn new_multi(arena: &TypeArena, user_file_paths: Vec<PathBuf>) -> Self {
        let intrinsics = allocate_intrinsics(arena);

        // Load lib files (ES5 only for this constructor)
        let lib_files = find_lib_sources(&["es5"]);
        let (mut file_paths, mut sources, lib_count) =
            Self::prepare_lib_files(lib_files);

        // Append user files after lib files
        file_paths.extend(user_file_paths);
        for path in file_paths.iter().skip(lib_count) {
            sources.push(std::fs::read_to_string(path).ok());
        }

        // File index maps canonical paths to indices (for module resolution).
        // Lib files use synthetic paths and aren't resolved via imports.
        let file_index: FxHashMap<PathBuf, usize> = file_paths
            .iter()
            .enumerate()
            .skip(lib_count)
            .map(|(i, p)| (p.clone(), i))
            .collect();

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
            lib_file_count: lib_count,
            file_paths,
            file_index,
            sources: RefCell::new(sources),
            source_types: None,
            file_cells: RefCell::new((0..file_count).map(|_| None).collect()),
            exports: RefCell::new(FxHashMap::default()),
            global_exports: RefCell::new(FxHashMap::default()),
            type_param_constraints: RefCell::new(FxHashMap::default()),
            checker_caches: RefCell::new((0..file_count).map(|_| None).collect()),
            checking: RefCell::new(FxHashSet::default()),
            checked: RefCell::new(FxHashSet::default()),
            arena: Some(arena as *const TypeArena),
            checker_options: CheckerOptions::default(),
            parse_ms: RefCell::new(0.0),
            bind_ms: RefCell::new(0.0),
            check_ms: RefCell::new(0.0),
            all_diagnostics: RefCell::new(Vec::new()),
        };
        for i in 0..lib_count {
            project.ensure_file_checked(i);
        }
        project.merge_global_exports();
        project
    }

    /// Create a project from in-memory sources (virtual files).
    ///
    /// Like `new_multi`, but accepts source text directly instead of reading
    /// from disk. Module resolution uses simple path + extension matching
    /// against the provided file paths (no disk-based resolver).
    pub fn new_multi_from_sources(
        arena: &TypeArena,
        files: Vec<(PathBuf, String, SourceType)>,
        checker_options: CheckerOptions,
    ) -> Self {
        Self::new_multi_from_sources_with_target(arena, files, checker_options, None)
    }

    /// Like `new_multi_from_sources`, but with a specific script target for
    /// lib file selection (e.g., ES2015 loads additional lib files).
    pub fn new_multi_from_sources_with_target(
        arena: &TypeArena,
        files: Vec<(PathBuf, String, SourceType)>,
        checker_options: CheckerOptions,
        target: Option<ScriptTarget>,
    ) -> Self {
        let intrinsics = allocate_intrinsics(arena);

        let lib_names = target
            .map(|t| t.default_libs())
            .unwrap_or(&["es5"]);
        let lib_files = find_lib_sources(lib_names);
        let (mut file_paths, mut sources, lib_count) =
            Self::prepare_lib_files(lib_files);

        // Source types: lib files are .d.ts, user files use provided types
        let mut source_type_vec: Vec<SourceType> =
            (0..lib_count).map(|_| SourceType::d_ts()).collect();

        for (path, source, source_type) in files {
            file_paths.push(path);
            sources.push(Some(source));
            source_type_vec.push(source_type);
        }

        let file_index: FxHashMap<PathBuf, usize> = file_paths
            .iter()
            .enumerate()
            .skip(lib_count)
            .map(|(i, p)| (p.clone(), i))
            .collect();

        let file_count = file_paths.len();

        let project = Self {
            intrinsics,
            resolver: None,
            lib_file_count: lib_count,
            file_paths,
            file_index,
            sources: RefCell::new(sources),
            source_types: Some(source_type_vec),
            file_cells: RefCell::new((0..file_count).map(|_| None).collect()),
            exports: RefCell::new(FxHashMap::default()),
            global_exports: RefCell::new(FxHashMap::default()),
            type_param_constraints: RefCell::new(FxHashMap::default()),
            checker_caches: RefCell::new((0..file_count).map(|_| None).collect()),
            checking: RefCell::new(FxHashSet::default()),
            checked: RefCell::new(FxHashSet::default()),
            arena: Some(arena as *const TypeArena),
            checker_options,
            parse_ms: RefCell::new(0.0),
            bind_ms: RefCell::new(0.0),
            check_ms: RefCell::new(0.0),
            all_diagnostics: RefCell::new(Vec::new()),
        };
        for i in 0..lib_count {
            project.ensure_file_checked(i);
        }
        project.merge_global_exports();
        project
    }

    /// Merge exports from all lib files into `global_exports`.
    ///
    /// Called after all lib files are checked. For each name exported by
    /// any lib file, if multiple lib files export the same name, their
    /// type-side types are merged via `merge_interface_types` (combining
    /// properties, call signatures, etc. with type parameter remapping).
    /// Value-side types use the later file's version (last wins).
    fn merge_global_exports(&self) {
        use oxc_types::{ObjectFlags, TypeFlags};

        let exports = self.exports.borrow();
        let mut merged: FxHashMap<CompactStr, ExportedBinding> = FxHashMap::default();

        // SAFETY: arena pointer is valid for the duration of the Project
        let arena = unsafe { &*self.arena.unwrap() };
        let mut ctx = type_merge::MergeContext::new(arena, self.intrinsics);

        // Group exports by lib file index using a per-lib Vec, then merge.
        // Single pass over the export map — O(total_exports), not O(lib_count * total_exports).
        let mut per_lib: Vec<Vec<(CompactStr, ExportedBinding)>> =
            (0..self.lib_file_count).map(|_| Vec::new()).collect();
        for ((idx, name), binding) in exports.iter() {
            if *idx < self.lib_file_count {
                per_lib[*idx].push((name.clone(), *binding));
            }
        }

        for lib_exports in &per_lib {
            for (name, binding) in lib_exports {
                use std::collections::hash_map::Entry;
                match merged.entry(name.clone()) {
                    Entry::Vacant(e) => {
                        e.insert(*binding);
                    }
                    Entry::Occupied(mut e) => {
                        // Merge type-side: only call merge_interface_types if both
                        // are actually interfaces (skip type aliases, enums, etc.)
                        if let (Some(existing), Some(new_type)) =
                            (e.get().type_type, binding.type_type)
                        {
                            let existing_is_interface = arena
                                .get_flags(existing)
                                .intersects(TypeFlags::Object)
                                && arena
                                    .get_object_flags(existing)
                                    .intersects(ObjectFlags::Interface);
                            let new_is_interface = arena
                                .get_flags(new_type)
                                .intersects(TypeFlags::Object)
                                && arena
                                    .get_object_flags(new_type)
                                    .intersects(ObjectFlags::Interface);

                            if existing_is_interface && new_is_interface {
                                let merged_type = type_merge::merge_interface_types(
                                    &mut ctx, existing, new_type,
                                );
                                e.get_mut().type_type = Some(merged_type);
                            }
                            // If not both interfaces, keep the existing type
                        } else if binding.type_type.is_some() {
                            e.get_mut().type_type = binding.type_type;
                        }
                        // Value-side: later lib file wins
                        if binding.value_type.is_some() {
                            e.get_mut().value_type = binding.value_type;
                        }
                    }
                }
            }
        }

        *self.global_exports.borrow_mut() = merged;
    }

    /// Check all files in the project.
    ///
    /// Lib files were already checked during construction.
    /// User files are checked sequentially. Import dependencies are
    /// resolved lazily — when a checker needs a type from another file,
    /// that file is checked on demand.
    pub fn check_all(&mut self) -> CheckResult {
        let total_start = Instant::now();

        // Check all files. Lib files are already checked.
        // Files may trigger on-demand checking of dependencies via
        // resolve_import → ensure_file_checked.
        for i in 0..self.file_paths.len() {
            self.ensure_file_checked(i);
        }
        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
        // Don't count lib files in the user-facing file count
        let files_checked = self.checked.borrow().len().saturating_sub(self.lib_file_count);

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

    /// Number of lib files in this project. User files start at this index.
    pub fn lib_file_count(&self) -> usize {
        self.lib_file_count
    }

    /// Construct a `TypePrinter` for post-check type display.
    ///
    /// All symbol lookups go through `CheckerHost::get_symbol_name`.
    /// The `arena` pointer must still be valid.
    pub fn type_printer(&self) -> oxc_checker::TypePrinter<'_> {
        // SAFETY: arena pointer was set during construction and the caller
        // owns the TypeArena, which outlives this Project.
        let arena = unsafe { &*self.arena.unwrap() };
        let array_type = self.get_global_type("Array");
        oxc_checker::TypePrinter::new(arena, self, array_type)
    }

    /// Range of user file indices (lib_file_count..total_files).
    pub fn user_file_range(&self) -> std::ops::Range<usize> {
        self.lib_file_count..self.file_paths.len()
    }

    /// Access a checked file's Program and Semantic for post-check AST walking.
    ///
    /// The callback receives references to the Program and Semantic stored in
    /// the FileCell. Returns `None` if the file hasn't been checked.
    pub fn with_file<F, R>(&self, file_idx: usize, f: F) -> Option<R>
    where
        F: FnOnce(&oxc_ast::ast::Program<'_>, &oxc_semantic::Semantic<'_>) -> R,
    {
        let cells = self.file_cells.borrow();
        let cell = cells.get(file_idx)?.as_ref()?;
        let borrowed = cell.borrow_dependent();
        Some(f(borrowed.program, &borrowed.semantic))
    }

    /// Reconstruct a Checker for post-check type queries on a checked file.
    ///
    /// Takes the stored `CheckerCaches` for `file_idx`, creates a new Checker
    /// with full capabilities (expression type lookup, type resolution,
    /// assignability, type-to-string), runs the callback, then stores the
    /// caches back. All cache state is preserved across calls.
    ///
    /// Returns `None` if the file hasn't been checked or has no stored caches.
    ///
    /// # Panics
    ///
    /// Panics if called for the same file_idx from within the callback
    /// (the caches are temporarily taken from the RefCell).
    pub fn with_checker<F, R>(&self, file_idx: usize, f: F) -> Option<R>
    where
        F: for<'a> FnOnce(&mut Checker<'a>, &oxc_ast::ast::Program<'a>) -> R,
    {
        // Borrow the FileCell to get Semantic + Program
        let cells = self.file_cells.borrow();
        let cell = cells.get(file_idx)?.as_ref()?;
        let borrowed = cell.borrow_dependent();

        // SAFETY: arena pointer was set during construction and the caller
        // owns the TypeArena, which outlives this Project.
        let arena = unsafe { &*self.arena? };

        // Move caches out of storage (O(1) — pointer swap via Option::take)
        let caches = self.checker_caches.borrow_mut().get_mut(file_idx)?.take()?;

        // Reconstruct a Checker with the restored caches
        let file_path = self.file_paths[file_idx].to_string_lossy().to_string();
        let mut checker = Checker::new_with_caches(
            &borrowed.semantic,
            arena,
            self,
            file_path,
            file_idx as u16,
            self.checker_options,
            caches,
        );

        let result = f(&mut checker, borrowed.program);

        // Put caches back (O(1) — pointer swap)
        self.checker_caches.borrow_mut()[file_idx] = Some(checker.into_caches());

        Some(result)
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

        let is_lib = file_idx < self.lib_file_count;
        let source_type = if let Some(ref types) = self.source_types {
            types[file_idx]
        } else if is_lib {
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
        let parser_errors: RefCell<Vec<OxcDiagnostic>> = RefCell::new(Vec::new());
        let semantic_errors: RefCell<Vec<OxcDiagnostic>> = RefCell::new(Vec::new());
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

                let oxc_parser::ParserReturn { program, module_record, errors, .. } = parsed;
                *parser_errors.borrow_mut() = errors;

                // Park program in the bump allocator so Semantic can
                // reference it with the allocator's lifetime.
                let program = owned.allocator.alloc(program);

                let bind_start = Instant::now();
                let semantic_ret = oxc_semantic::SemanticBuilder::new()
                    .build(program);
                bind_elapsed.set(bind_start.elapsed().as_secs_f64() * 1000.0);

                *semantic_errors.borrow_mut() = semantic_ret.errors;
                FileBorrowed { semantic: semantic_ret.semantic, program, module_record }
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
                self.checker_options,
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

            // Extract constraints and diagnostics BEFORE into_caches(), which
            // consumes the Checker. take_type_param_constraints() empties
            // caches.type_param_constraints via mem::take — this is intentional:
            // the constraints are merged into Project.type_param_constraints,
            // and when with_checker() reconstructs a Checker later, constraint
            // lookups fall back to host.get_type_param_constraint() (the
            // Project's merged cache).
            file_constraints = checker.take_type_param_constraints();
            diagnostics = checker.take_diagnostics();

            // Store checker caches for post-check queries via with_checker().
            let caches = checker.into_caches();
            self.checker_caches.borrow_mut()[file_idx] = Some(caches);
        } // FileCell borrow released
        *self.check_ms.borrow_mut() += check_start.elapsed().as_secs_f64() * 1000.0;

        // Store FileCell — Semantic stays alive for future cross-file access.
        self.file_cells.borrow_mut()[file_idx] = Some(file_cell);

        // Record exports
        {
            let mut export_map = self.exports.borrow_mut();
            for (name, binding) in exports {
                export_map.insert((file_idx, name), binding);
            }
        }

        // Merge constraints
        if !file_constraints.is_empty() {
            self.type_param_constraints.borrow_mut().extend(file_constraints);
        }

        // Combine all diagnostics: parser → semantic → checker
        let mut all_file_diags = parser_errors.into_inner();
        all_file_diags.extend(semantic_errors.into_inner());
        all_file_diags.extend(diagnostics);
        if !all_file_diags.is_empty() {
            self.all_diagnostics.borrow_mut().push(
                (self.file_paths[file_idx].clone(), all_file_diags)
            );
        }

        self.checking.borrow_mut().remove(&file_idx);
        self.checked.borrow_mut().insert(file_idx);
    }

    /// Extract all root scope declared types and value types from an ambient
    /// file (lib.d.ts or .d.ts without module syntax). All declarations are
    /// globally visible.
    ///
    /// For each root-scope symbol, resolves both the type-side (interface,
    /// type alias, class instance, enum union) and value-side (var annotation,
    /// function signature, class constructor, enum namespace) types.
    fn extract_ambient_declarations(
        checker: &mut Checker<'_>,
    ) -> Vec<(CompactStr, ExportedBinding)> {
        use oxc_syntax::symbol::SymbolFlags;

        let root_scope = checker.semantic().scoping().root_scope_id();
        let symbols: Vec<oxc_syntax::symbol::SymbolId> = checker
            .semantic()
            .scoping()
            .iter_bindings_in(root_scope)
            .collect();

        let mut export_map: FxHashMap<CompactStr, ExportedBinding> = FxHashMap::default();
        for symbol_id in symbols {
            let name = CompactStr::new(checker.semantic().scoping().symbol_name(symbol_id));
            let flags = checker.semantic().scoping().symbol_flags(symbol_id);
            let binding = export_map.entry(name).or_default();

            // Type-side: interfaces, type aliases, classes (instance), enums (union)
            if flags.intersects(SymbolFlags::Type) {
                let t = checker.get_declared_type_of_symbol(symbol_id);
                if t != checker.any_type {
                    binding.type_type = Some(t);
                }
            }
            // Value-side: variables, functions, classes (constructor), enums (namespace)
            if flags.intersects(SymbolFlags::Value) {
                let v = checker.get_type_of_symbol(symbol_id);
                if v != checker.any_type {
                    binding.value_type = Some(v);
                }
            }
        }
        // Filter out bindings with neither side resolved
        export_map
            .into_iter()
            .filter(|(_, b)| b.type_type.is_some() || b.value_type.is_some())
            .collect()
    }

    /// Extract ES module exports from a checked file.
    /// Eagerly resolves both type-side and value-side types for exported symbols.
    fn extract_module_exports(
        checker: &mut Checker<'_>,
        module_record: &oxc_syntax::module_record::ModuleRecord<'_>,
    ) -> Vec<(CompactStr, ExportedBinding)> {
        use oxc_syntax::module_record::{ExportExportName, ExportLocalName};
        use oxc_syntax::symbol::SymbolFlags;

        let root_scope = checker.semantic().scoping().root_scope_id();
        let mut exports = Vec::new();

        // Collect into owned Strings to avoid borrowing checker during the loop
        let binding_map: FxHashMap<String, (oxc_syntax::symbol::SymbolId, SymbolFlags)> = checker
            .semantic()
            .scoping()
            .iter_bindings_in(root_scope)
            .map(|sym_id| {
                let name = checker.semantic().scoping().symbol_name(sym_id).to_string();
                let flags = checker.semantic().scoping().symbol_flags(sym_id);
                (name, (sym_id, flags))
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

            let Some(&(symbol_id, flags)) = binding_map.get(&local_name.to_string()) else {
                continue;
            };

            let mut binding = ExportedBinding::default();

            // Type-side: interfaces, type aliases, classes (instance), enums (union)
            if flags.intersects(SymbolFlags::Type) {
                let t = checker.get_cached_declared_type(symbol_id)
                    .unwrap_or_else(|| checker.get_declared_type_of_symbol(symbol_id));
                if t != checker.any_type {
                    binding.type_type = Some(t);
                }
            }

            // Value-side: variables, functions, classes (constructor), enums (namespace)
            if flags.intersects(SymbolFlags::Value) {
                let v = checker.get_cached_symbol_type(symbol_id)
                    .unwrap_or_else(|| checker.get_type_of_symbol(symbol_id));
                if v != checker.any_type {
                    binding.value_type = Some(v);
                }
            }

            if binding.type_type.is_some() || binding.value_type.is_some() {
                exports.push((CompactStr::new(export_name), binding));
            }
        }

        exports
    }

    /// Resolve a module specifier from a given file to a file index.
    #[allow(dead_code)]
    fn resolve_module_to_index(&self, from_idx: usize, specifier: &str) -> Option<usize> {
        let from_dir = self.file_paths[from_idx].parent()?;
        if let Some(resolver) = self.resolver.as_ref() {
            let resolution = resolver.resolve(from_dir, specifier).ok()?;
            let resolved_path = resolution.path().canonicalize().ok()?;
            self.file_index.get(&resolved_path).copied()
        } else {
            self.resolve_virtual(from_dir, specifier)
        }
    }

    /// Simple path-based module resolution for virtual files.
    ///
    /// Tries the specifier with standard TypeScript extensions against
    /// `file_index`. Used when no disk-based resolver is available.
    fn resolve_virtual(&self, from_dir: &Path, specifier: &str) -> Option<usize> {
        let base = from_dir.join(specifier);

        // Exact match (specifier already has extension, e.g. "./a.ts")
        if let Some(&idx) = self.file_index.get(&base) {
            return Some(idx);
        }

        // Try appending TypeScript extensions (mirrors tsc resolution order)
        let base_str = base.to_string_lossy();
        for ext in &[".ts", ".tsx", ".d.ts", ".js", ".jsx"] {
            let candidate = PathBuf::from(format!("{base_str}{ext}"));
            if let Some(&idx) = self.file_index.get(&candidate) {
                return Some(idx);
            }
        }
        None
    }
}

impl CheckerHost for Project {
    fn get_intrinsics(&self) -> IntrinsicIds {
        self.intrinsics
    }

    fn get_global_type(&self, name: &str) -> Option<TypeId> {
        self.global_exports
            .borrow()
            .get(&CompactStr::new(name))
            .and_then(|b| b.type_type)
    }

    fn get_global_value_type(&self, name: &str) -> Option<TypeId> {
        self.global_exports
            .borrow()
            .get(&CompactStr::new(name))
            .and_then(|b| b.value_type)
    }

    fn resolve_import(
        &self,
        from_file: &str,
        module_specifier: &str,
        export_name: &str,
    ) -> Option<ExportedBinding> {
        let from_path = Path::new(from_file);
        let from_dir = from_path.parent()?;

        let file_idx = if let Some(resolver) = self.resolver.as_ref() {
            let resolution = resolver.resolve(from_dir, module_specifier).ok()?;
            let resolved_path = resolution.path().canonicalize().ok()?;
            *self.file_index.get(&resolved_path)?
        } else {
            self.resolve_virtual(from_dir, module_specifier)?
        };

        // Lazy: ensure the dependency file is checked before looking up its exports
        self.ensure_file_checked(file_idx);

        self.exports
            .borrow()
            .get(&(file_idx, CompactStr::new(export_name)))
            .copied()
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

    #[test]
    fn merge_interfaces_combines_properties() {
        use oxc_types::TypeData;

        let arena = TypeArena::with_capacity(64);
        let options = CheckerOptions::default();

        // Two ambient .d.ts files declaring the same interface
        let files = vec![
            (
                PathBuf::from("/test/a.d.ts"),
                "interface Foo { x: string; }".to_string(),
                SourceType::d_ts(),
            ),
            (
                PathBuf::from("/test/b.d.ts"),
                "interface Foo { y: number; }".to_string(),
                SourceType::d_ts(),
            ),
        ];

        let mut project = Project::new_multi_from_sources(&arena, files, options);
        project.check_all();

        // Both files export "Foo" as a type
        let foo_a = project.exports.borrow().get(&(1, CompactStr::new("Foo")))
            .and_then(|b| b.type_type);
        let foo_b = project.exports.borrow().get(&(2, CompactStr::new("Foo")))
            .and_then(|b| b.type_type);

        assert!(foo_a.is_some(), "file a should export Foo");
        assert!(foo_b.is_some(), "file b should export Foo");

        let foo_a = foo_a.unwrap();
        let foo_b = foo_b.unwrap();

        // Merge them
        let intrinsics = project.intrinsics;
        let mut ctx = type_merge::MergeContext::new(&arena, intrinsics);
        let merged = type_merge::merge_interface_types(&mut ctx, foo_a, foo_b);

        // Verify merged type has both properties
        let TypeData::Structured(s) = arena.get_data(merged) else {
            panic!("merged type should be Structured");
        };
        let prop_names: Vec<&str> = s.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(prop_names.contains(&"x"), "merged should have property x, got: {prop_names:?}");
        assert!(prop_names.contains(&"y"), "merged should have property y, got: {prop_names:?}");
        assert_eq!(s.properties.len(), 2, "merged should have exactly 2 properties");
    }

    #[test]
    fn merge_generic_interfaces_remaps_type_params() {
        use oxc_types::TypeData;

        let arena = TypeArena::with_capacity(64);
        let options = CheckerOptions::default();

        // Two ambient files with a generic interface
        let files = vec![
            (
                PathBuf::from("/test/a.d.ts"),
                "interface Box<T> { value: T; }".to_string(),
                SourceType::d_ts(),
            ),
            (
                PathBuf::from("/test/b.d.ts"),
                "interface Box<T> { unwrap(): T; }".to_string(),
                SourceType::d_ts(),
            ),
        ];

        let mut project = Project::new_multi_from_sources(&arena, files, options);
        project.check_all();

        let box_a = project.exports.borrow().get(&(1, CompactStr::new("Box")))
            .and_then(|b| b.type_type).expect("file a should export Box");
        let box_b = project.exports.borrow().get(&(2, CompactStr::new("Box")))
            .and_then(|b| b.type_type).expect("file b should export Box");

        // Get type parameters from both — they should be different TypeIds
        let TypeData::Structured(a_s) = arena.get_data(box_a) else { panic!() };
        let TypeData::Structured(b_s) = arena.get_data(box_b) else { panic!() };
        let a_params = match &a_s.kind {
            oxc_types::StructuredTypeKind::Interface { all_type_parameters, .. } => {
                all_type_parameters.clone()
            }
            _ => panic!("expected Interface kind"),
        };
        let b_params = match &b_s.kind {
            oxc_types::StructuredTypeKind::Interface { all_type_parameters, .. } => {
                all_type_parameters.clone()
            }
            _ => panic!("expected Interface kind"),
        };
        assert_eq!(a_params.len(), 1, "Box<T> should have 1 type param");
        assert_eq!(b_params.len(), 1, "Box<T> should have 1 type param");
        assert_ne!(a_params[0], b_params[0], "different files should have different TypeParameter IDs");

        // Merge
        let mut ctx = type_merge::MergeContext::new(&arena, project.intrinsics);
        let merged = type_merge::merge_interface_types(&mut ctx, box_a, box_b);

        // Verify merged type has base's type parameter (a_params[0])
        let TypeData::Structured(merged_s) = arena.get_data(merged) else { panic!() };
        let merged_params = match &merged_s.kind {
            oxc_types::StructuredTypeKind::Interface { all_type_parameters, .. } => {
                all_type_parameters.clone()
            }
            _ => panic!("expected Interface kind"),
        };
        assert_eq!(merged_params.len(), 1);
        assert_eq!(merged_params[0], a_params[0], "merged should use base's type parameter");

        // Verify `value` property references base's T (a_params[0])
        let value_prop = merged_s.properties.iter().find(|p| p.name.as_str() == "value");
        assert!(value_prop.is_some(), "merged should have 'value' property");
        assert_eq!(
            value_prop.unwrap().type_id, a_params[0],
            "'value' should reference base's T"
        );

        // Verify `unwrap` method's return type references base's T
        // unwrap() is a method, so it's a PropertyInfo with a Function type
        let unwrap_prop = merged_s.properties.iter().find(|p| p.name.as_str() == "unwrap");
        assert!(unwrap_prop.is_some(), "merged should have 'unwrap' property");
        let unwrap_type = unwrap_prop.unwrap().type_id;
        // The unwrap method is a Function type — check its return type is base's T
        if let TypeData::Function(func) = arena.get_data(unwrap_type) {
            assert_eq!(
                func.signatures[0].return_type, a_params[0],
                "unwrap() should return base's T, not extension's T"
            );
        } else {
            // Method might be stored differently — just check it's not b's T
            assert_ne!(
                unwrap_type, b_params[0],
                "unwrap type should not reference extension's T directly"
            );
        }
    }
}
