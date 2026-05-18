use std::path::{Path, PathBuf};

use oxc_config::GlobSet;

use crate::core::oxfmtrc::{FormatConfig, OxfmtOverrideConfig};

/// Resolved overrides for file-specific matching.
/// Similar to `EditorConfig`, this also handles `FormatConfig` override resolution.
#[derive(Debug)]
pub struct OxfmtrcOverrides {
    base_dir: Option<PathBuf>,
    entries: Vec<OverrideEntry>,
}

impl OxfmtrcOverrides {
    pub fn new(overrides: Vec<OxfmtOverrideConfig>, base_dir: Option<PathBuf>) -> Self {
        Self {
            base_dir,
            entries: overrides
                .into_iter()
                .map(|o| OverrideEntry {
                    files: o.files,
                    exclude_files: o.exclude_files,
                    options: o.options,
                })
                .collect(),
        }
    }

    /// Check if any overrides exist that match the given path.
    pub fn has_match(&self, path: &Path) -> bool {
        let relative = self.relative_path(path);
        self.entries.iter().any(|e| Self::is_entry_match(e, &relative))
    }

    /// Get all matching override options for a given path.
    pub fn get_matching(&self, path: &Path) -> impl Iterator<Item = &FormatConfig> + '_ {
        let relative = self.relative_path(path);
        self.entries.iter().filter(move |e| Self::is_entry_match(e, &relative)).map(|e| &e.options)
    }

    /// NOTE: On Windows, `to_string_lossy()` produces `\`-separated paths.
    /// This is OK since `fast_glob::glob_match()` supports both `/` and `\` via `std::path::is_separator`.
    fn relative_path(&self, path: &Path) -> String {
        self.base_dir
            .as_ref()
            .and_then(|dir| path.strip_prefix(dir).ok())
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned()
    }

    fn is_entry_match(entry: &OverrideEntry, relative: &str) -> bool {
        entry.files.is_match(relative) && !entry.exclude_files.is_match(relative)
    }
}

// ---

/// A single override entry with normalized glob patterns.
/// NOTE: Written path patterns are glob patterns; use `/` as the path separator on all platforms.
#[derive(Debug)]
struct OverrideEntry {
    files: GlobSet,
    exclude_files: GlobSet,
    options: FormatConfig,
}
