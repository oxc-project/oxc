//! # oxc_checker
//!
//! Experimental **eager** TypeScript type checker built on **isolated
//! declarations**. See `DESIGN.md` for the architecture.
//!
//! Pipeline: load all files in parallel waves (parse + forced
//! `IsolatedDeclarations` surface extraction + module resolution) → link the
//! surfaces into a frozen, `Send + Sync` [`link::ProgramEnv`] → check every
//! file in parallel against it.

mod check;
mod diagnostics;
mod ir;
mod link;
mod loader;
mod lower;
mod surface;
mod tsconfig;

use std::path::{Path, PathBuf};

use oxc_diagnostics::OxcDiagnostic;

pub use crate::{
    ir::{FileId, SymbolId, TypeId},
    link::ProgramEnv,
    tsconfig::TsConfig,
};

/// Diagnostics for one file.
#[derive(Debug)]
pub struct FileResult {
    /// Absolute path.
    pub path: PathBuf,
    /// Source text (for rendering diagnostics).
    pub source_text: String,
    /// All diagnostics, ordered by source position.
    pub diagnostics: Vec<OxcDiagnostic>,
}

/// Result of checking a project.
#[derive(Debug)]
pub struct CheckResult {
    /// Per-file results, in deterministic (discovery) order.
    pub files: Vec<FileResult>,
}

impl CheckResult {
    /// Total number of diagnostics.
    pub fn error_count(&self) -> usize {
        self.files.iter().map(|f| f.diagnostics.len()).sum()
    }
}

/// Check a project rooted at a directory or described by a `tsconfig.json`.
///
/// Isolated declarations are forced: violations are reported as errors
/// alongside type errors.
///
/// # Errors
/// When the tsconfig cannot be loaded or no TypeScript files are found.
pub fn check_project(path: &Path) -> Result<CheckResult, String> {
    let path = std::path::absolute(path).map_err(|e| e.to_string());
    let path = path?;
    let (dir, tsconfig_path) = if path.is_dir() {
        let tsconfig = path.join("tsconfig.json");
        (path, tsconfig.is_file().then_some(tsconfig))
    } else {
        let dir = path.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        (dir, Some(path))
    };

    let config = match &tsconfig_path {
        Some(p) => TsConfig::load(p)?,
        None => TsConfig::default(),
    };

    let roots = config.root_files(&dir);
    if roots.is_empty() {
        return Err(format!("No TypeScript files found under {}", dir.display()));
    }

    let resolver = make_resolver(tsconfig_path);
    let loaded = loader::load(roots, &resolver);
    let env = link::link(loaded, config.strict_null_checks());
    let mut per_file = check::check_program(&env);

    let files = env
        .files
        .into_iter()
        .zip(per_file.iter_mut())
        .map(|(file, checked)| {
            let mut diagnostics = file.diagnostics;
            diagnostics.append(checked);
            diagnostics
                .sort_by_key(|d| d.labels.first().map_or(0, oxc_diagnostics::LabeledSpan::offset));
            FileResult { path: file.path, source_text: file.source_text, diagnostics }
        })
        .collect();

    Ok(CheckResult { files })
}

fn make_resolver(tsconfig: Option<PathBuf>) -> oxc_resolver::Resolver {
    use oxc_resolver::{
        ResolveOptions, Resolver, TsconfigDiscovery, TsconfigOptions, TsconfigReferences,
    };
    Resolver::new(ResolveOptions {
        extensions: [
            ".ts", ".tsx", ".d.ts", ".mts", ".cts", ".d.mts", ".d.cts", ".js", ".mjs", ".cjs",
            ".json",
        ]
        .map(String::from)
        .into(),
        extension_alias: vec![
            (".js".into(), vec![".ts".into(), ".tsx".into(), ".d.ts".into(), ".js".into()]),
            (".mjs".into(), vec![".mts".into(), ".d.mts".into(), ".mjs".into()]),
            (".cjs".into(), vec![".cts".into(), ".d.cts".into(), ".cjs".into()]),
            (".ts".into(), vec![".ts".into(), ".d.ts".into(), ".tsx".into()]),
            (".mts".into(), vec![".mts".into(), ".d.mts".into()]),
            (".cts".into(), vec![".cts".into(), ".d.cts".into()]),
        ],
        condition_names: vec!["types".into(), "import".into(), "require".into(), "node".into()],
        main_fields: vec!["types".into(), "module".into(), "main".into()],
        // `node:*` and bare builtins resolve as builtin errors, which the
        // loader treats as external (`any`) modules.
        builtin_modules: true,
        tsconfig: tsconfig.map(|config_file| {
            TsconfigDiscovery::Manual(TsconfigOptions {
                config_file,
                references: TsconfigReferences::Auto,
            })
        }),
        ..ResolveOptions::default()
    })
}
