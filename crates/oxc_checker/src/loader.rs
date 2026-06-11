//! Parallel, wave-based program loading (pass A).
//!
//! Each wave parses a batch of files in parallel; per file it forces the
//! `IsolatedDeclarations` transform, lowers the surface, and resolves module
//! specifiers. Files newly reachable through resolution (e.g.
//! `node_modules/**/*.d.ts`) join the next wave. No file ever needs another
//! file to produce its surface, so waves only grow the file set — there is no
//! dependency ordering.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::surface::{FileSurface, build_surface};

/// Result of resolving one module specifier, path-based (pre-link).
#[derive(Debug)]
pub enum RawResolution {
    /// Resolved to a TypeScript file that joins the program.
    File(PathBuf),
    /// Resolved to a non-TypeScript file (plain `.js`, `.json`, ...). Imports
    /// from it are typed `any`.
    External,
    /// Resolution failed → TS2307 at use sites.
    NotFound,
}

/// File classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// Implementation file: surfaced via the forced ID transform and checked.
    Ts,
    /// Declaration file: it *is* a surface; never checked.
    Dts,
}

/// Pass-A output for one file.
#[derive(Debug)]
pub struct LoadedFile {
    /// Absolute path.
    pub path: PathBuf,
    /// Source text (kept: pass B re-parses, diagnostics render against it).
    pub source_text: String,
    /// Classification.
    pub kind: FileKind,
    /// The lowered surface.
    pub surface: FileSurface,
    /// Specifier → resolution for every module reference in the file.
    pub resolutions: FxHashMap<Box<str>, RawResolution>,
    /// Parse + forced-isolated-declarations diagnostics.
    pub diagnostics: Vec<OxcDiagnostic>,
    /// Parsing failed: the file is not checked and its exports are opaque.
    pub parse_failed: bool,
}

/// Load the transitive program from `roots`, in parallel waves.
pub fn load(roots: Vec<PathBuf>, resolver: &oxc_resolver::Resolver) -> Vec<LoadedFile> {
    let mut seen: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue: Vec<PathBuf> = roots;
    let mut loaded: Vec<LoadedFile> = Vec::new();

    while !queue.is_empty() {
        let batch: Vec<PathBuf> =
            std::mem::take(&mut queue).into_iter().filter(|p| seen.insert(p.clone())).collect();
        if batch.is_empty() {
            break;
        }
        let results: Vec<LoadedFile> =
            batch.par_iter().map(|path| load_file(path, resolver)).collect();
        for file in &results {
            for resolution in file.resolutions.values() {
                if let RawResolution::File(path) = resolution
                    && !seen.contains(path)
                {
                    queue.push(path.clone());
                }
            }
        }
        loaded.extend(results);
    }
    loaded
}

fn file_kind(path: &Path) -> FileKind {
    let s = path.to_string_lossy();
    if s.ends_with(".d.ts") || s.ends_with(".d.mts") || s.ends_with(".d.cts") {
        FileKind::Dts
    } else {
        FileKind::Ts
    }
}

/// Whether a resolved path is a TypeScript file that should join the program.
fn joins_program(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.ends_with(".ts") || s.ends_with(".tsx") || s.ends_with(".mts") || s.ends_with(".cts")
}

fn load_file(path: &Path, resolver: &oxc_resolver::Resolver) -> LoadedFile {
    let kind = file_kind(path);
    let source_text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            return LoadedFile {
                path: path.to_path_buf(),
                source_text: String::new(),
                kind,
                surface: FileSurface { opaque_exports: true, ..FileSurface::default() },
                resolutions: FxHashMap::default(),
                diagnostics: vec![OxcDiagnostic::error(format!(
                    "Cannot read file '{}': {err}.",
                    path.display()
                ))],
                parse_failed: true,
            };
        }
    };

    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source_text, source_type).parse();

    let mut diagnostics: Vec<OxcDiagnostic> = parsed.errors;
    let parse_failed = parsed.panicked || !diagnostics.is_empty();

    // Surface: a .d.ts file is already a surface; a .ts file goes through the
    // *forced* IsolatedDeclarations transform, whose violations are hard
    // errors here.
    let surface = if parse_failed {
        FileSurface { opaque_exports: true, ..FileSurface::default() }
    } else if kind == FileKind::Dts {
        build_surface(&parsed.program)
    } else {
        let id_ret = IsolatedDeclarations::new(
            &allocator,
            IsolatedDeclarationsOptions { strip_internal: false },
        )
        .build(&parsed.program);
        diagnostics.extend(id_ret.errors);
        let mut surface = build_surface(&id_ret.program);
        // The declaration output drops unreferenced local declarations; the
        // checker still needs local enums as symbols.
        crate::surface::augment_surface_with_local_enums(&mut surface, &parsed.program);
        surface
    };

    // Resolve every module reference in the *original* program (the surface
    // only retains type-relevant imports; checking needs them all).
    let mut specifiers: FxHashSet<&str> = FxHashSet::default();
    if !parse_failed {
        for stmt in &parsed.program.body {
            use oxc_ast::ast::Statement;
            match stmt {
                Statement::ImportDeclaration(import) => {
                    specifiers.insert(import.source.value.as_str());
                }
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(source) = &export.source {
                        specifiers.insert(source.value.as_str());
                    }
                }
                Statement::ExportAllDeclaration(export) => {
                    specifiers.insert(export.source.value.as_str());
                }
                _ => {}
            }
        }
    }
    // The surface can also reference specifiers (always a subset in practice;
    // kept for safety).
    for import in &surface.imports {
        specifiers.insert(&import.specifier);
    }

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let resolutions = specifiers
        .into_iter()
        .map(|specifier| {
            let resolution = match resolver.resolve(dir, specifier) {
                Ok(resolution) => {
                    let full_path = resolution.full_path();
                    if joins_program(&full_path) {
                        RawResolution::File(full_path)
                    } else {
                        RawResolution::External
                    }
                }
                // Node builtins are typed by @types/node in real tsc; v0
                // treats them as untyped externals rather than erroring.
                Err(oxc_resolver::ResolveError::Builtin { .. }) => RawResolution::External,
                Err(_) => RawResolution::NotFound,
            };
            (Box::from(specifier), resolution)
        })
        .collect();

    LoadedFile {
        path: path.to_path_buf(),
        source_text,
        kind,
        surface,
        resolutions,
        diagnostics,
        parse_failed,
    }
}
