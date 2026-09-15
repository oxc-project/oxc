use std::path::{Path, PathBuf};

use oxc_config::GlobSet;

use crate::core::{
    oxfmtrc::{FormatConfig, OxfmtOverrideConfig},
    support::Language,
};

/// Resolved overrides for file-specific matching.
/// Similar to `EditorConfig`, this also handles `FormatConfig` override resolution.
#[derive(Debug)]
pub struct OxfmtrcOverrides {
    base_dir: Option<PathBuf>,
    entries: Vec<OverrideEntry>,
}

impl OxfmtrcOverrides {
    pub fn new(overrides: Vec<OxfmtOverrideConfig>, base_dir: Option<PathBuf>) -> Self {
        let empty_options = serde_json::to_value(FormatConfig::default())
            .expect("`FormatConfig` serialization is infallible");
        Self {
            base_dir,
            entries: overrides
                .into_iter()
                .map(|o| OverrideEntry {
                    files: o.files,
                    exclude_files: o.exclude_files,
                    language: o.language,
                    has_options: serde_json::to_value(&o.options)
                        .is_ok_and(|options| options != empty_options),
                    options: o.options,
                })
                .collect(),
        }
    }

    /// Match `path` against every override once, in config order.
    pub fn matching(&self, path: &Path) -> MatchedOverrides<'_> {
        let relative = super::relative_to_config_dir(self.base_dir.as_deref(), path);

        let mut matched = MatchedOverrides::default();
        for entry in &self.entries {
            if !entry.files.is_match(&relative) || entry.exclude_files.is_match(&relative) {
                continue;
            }
            if entry.has_options {
                matched.options.push(&entry.options);
            }
            if entry.language.is_some() {
                matched.language = entry.language;
            }
        }
        matched
    }
}

/// The overrides matching one file.
#[derive(Debug, Default)]
pub struct MatchedOverrides<'a> {
    /// Options of matching entries that set any, in config order.
    /// Empty when no matching entry changes options, so callers can gate the fast path on `is_empty()`;
    /// a `language`-only entry does not force the slow path.
    pub options: Vec<&'a FormatConfig>,
    /// Language of the last matching entry that sets one.
    pub language: Option<Language>,
}

// ---

/// A single override entry with normalized glob patterns.
/// NOTE: Written path patterns are glob patterns; use `/` as the path separator on all platforms.
#[derive(Debug)]
struct OverrideEntry {
    files: GlobSet,
    exclude_files: GlobSet,
    language: Option<Language>,
    options: FormatConfig,
    // Precomputed so the per-file match never serializes `options`
    has_options: bool,
}
