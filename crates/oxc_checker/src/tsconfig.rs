//! Minimal `tsconfig.json` loading.
//!
//! Module-resolution semantics (`paths`, `baseUrl`, project references,
//! `extends`) are handled by `oxc_resolver`, which reads the tsconfig itself.
//! This module only extracts what the checker needs directly: the file set
//! (`files` / `include` / `exclude`) and strictness flags.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Parsed subset of `tsconfig.json`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TsConfig {
    /// `compilerOptions`.
    pub compiler_options: CompilerOptions,
    /// Explicit file list, relative to the tsconfig directory.
    pub files: Option<Vec<String>>,
    /// Include patterns. v0 supports directory prefixes; glob syntax is
    /// reduced to its leading literal path segments.
    pub include: Option<Vec<String>>,
    /// Exclude patterns, treated the same way as `include`.
    pub exclude: Option<Vec<String>>,
}

/// Parsed subset of `compilerOptions`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompilerOptions {
    /// `strict`.
    pub strict: Option<bool>,
    /// `strictNullChecks` (defaults to `strict`).
    pub strict_null_checks: Option<bool>,
    /// `isolatedDeclarations`. The checker forces this mode regardless; the
    /// flag is accepted for compatibility.
    pub isolated_declarations: Option<bool>,
}

impl TsConfig {
    /// Load and parse a `tsconfig.json` (JSONC comments are stripped).
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<TsConfig, String> {
        let mut text = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        json_strip_comments::strip(&mut text)
            .map_err(|e| format!("Failed to strip comments from {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse {}: {e}", path.display()))
    }

    /// Effective `strictNullChecks`.
    pub fn strict_null_checks(&self) -> bool {
        self.compiler_options.strict_null_checks.or(self.compiler_options.strict).unwrap_or(false)
    }

    /// Compute the root file set for the program.
    ///
    /// - `files` entries are taken verbatim.
    /// - `include` entries contribute their leading literal path segments as
    ///   walk roots (globs are not fully evaluated in v0).
    /// - Without either, the tsconfig directory is walked.
    ///
    /// `node_modules` and dot-directories are always skipped during walks;
    /// `node_modules` files still enter the program via import resolution.
    pub fn root_files(&self, dir: &Path) -> Vec<PathBuf> {
        if let Some(files) = &self.files {
            return files.iter().map(|f| dir.join(f)).collect();
        }
        let roots: Vec<PathBuf> = match &self.include {
            Some(include) => {
                include.iter().map(|pattern| dir.join(literal_prefix(pattern))).collect()
            }
            None => vec![dir.to_path_buf()],
        };
        let excludes: Vec<PathBuf> = self
            .exclude
            .iter()
            .flatten()
            .map(|pattern| dir.join(literal_prefix(pattern)))
            .collect();

        let mut out = Vec::new();
        for root in roots {
            if root.is_file() {
                if is_checkable_ext(&root) {
                    out.push(root);
                }
                continue;
            }
            let walker = walkdir::WalkDir::new(&root).follow_links(false).into_iter();
            for entry in walker.filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !(name == "node_modules" || name.starts_with('.'))
            }) {
                let Ok(entry) = entry else { continue };
                if entry.file_type().is_dir() {
                    continue;
                }
                let path = entry.path();
                if !is_checkable_ext(path) {
                    continue;
                }
                if excludes.iter().any(|ex| path.starts_with(ex)) {
                    continue;
                }
                out.push(path.to_path_buf());
            }
        }
        out.sort();
        out.dedup();
        out
    }
}

/// Whether a path has an extension the checker accepts as a root file.
pub fn is_checkable_ext(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.ends_with(".ts") || s.ends_with(".tsx") || s.ends_with(".mts") || s.ends_with(".cts")
}

/// Leading literal path segments of an include/exclude pattern
/// (`src/**/*.ts` → `src`).
fn literal_prefix(pattern: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for seg in pattern.split('/') {
        if seg.contains('*') || seg.contains('?') || seg.contains('[') {
            break;
        }
        out.push(seg);
    }
    out
}
