use std::path::{Path, PathBuf};

use serde_json::Value;

use oxc_formatter::{
    FormatOptions,
    oxfmtrc::{OxfmtOptions, Oxfmtrc},
};

use super::{FormatFileStrategy, utils};

/// Resolve config file path from cwd and optional explicit path.
pub fn resolve_config_path(cwd: &Path, config_path: Option<&Path>) -> Option<PathBuf> {
    // If `--config` is explicitly specified, use that path
    if let Some(config_path) = config_path {
        return Some(if config_path.is_absolute() {
            config_path.to_path_buf()
        } else {
            cwd.join(config_path)
        });
    }

    // If `--config` is not specified, search the nearest config file from cwd upwards
    // Support both `.json` and `.jsonc`, but prefer `.json` if both exist
    cwd.ancestors().find_map(|dir| {
        for filename in [".oxfmtrc.json", ".oxfmtrc.jsonc"] {
            let config_path = dir.join(filename);
            if config_path.exists() {
                return Some(config_path);
            }
        }
        None
    })
}

/// Resolved options for each file type.
/// Each variant contains only the options needed for that formatter.
pub enum ResolvedOptions {
    /// For JS/TS files formatted by oxc_formatter.
    OxcFormatter {
        format_options: FormatOptions,
        /// For embedded language formatting (e.g., CSS in template literals)
        external_options: Value,
    },
    /// For non-JS files formatted by external formatter (Prettier).
    #[cfg(feature = "napi")]
    ExternalFormatter { external_options: Value },
    /// For `package.json` files: optionally sorted then formatted.
    #[cfg(feature = "napi")]
    ExternalFormatterPackageJson { external_options: Value, sort_package_json: bool },
}

/// Configuration resolver that derives all config values from a single `serde_json::Value`.
///
/// Priority order: `Oxfmtrc::default()` → (TODO: editorconfig) → user's oxfmtrc
pub struct ConfigResolver {
    /// User's raw config as JSON value.
    /// It contains every possible field, even those not recognized by `Oxfmtrc`.
    /// e.g. `printWidth`: recognized by both `Oxfmtrc` and Prettier
    /// e.g. `vueIndentScriptAndStyle`: not recognized by `Oxfmtrc`, but used by Prettier
    /// e.g. `svelteSortAttributes`: not recognized by Prettier by default
    raw_config: Value,
    /// Cached parsed options after validation.
    /// Used to avoid re-parsing during per-file resolution, if `.editorconfig` is not used.
    /// NOTE: Currently, only `.editorconfig` provides per-file overrides, `.oxfmtrc` does not.
    cached_options: Option<(FormatOptions, OxfmtOptions, Value)>,
    // TODO: Add editorconfig support
}

impl ConfigResolver {
    /// Create a new resolver from a raw JSON config value.
    #[cfg(feature = "napi")]
    pub fn from_value(raw_config: Value) -> Self {
        Self { raw_config, cached_options: None }
    }

    /// Create a resolver by loading config from a file path.
    ///
    /// # Errors
    /// Returns error if:
    /// - Config file is specified but not found or invalid
    /// - Config file parsing fails
    pub fn from_config_path(config_path: Option<&Path>) -> Result<Self, String> {
        // Read and parse config file, or use empty JSON if not found
        let json_string = match config_path {
            Some(path) => {
                let mut json_string = utils::read_to_string(path)
                    // Do not include OS error, it differs between platforms
                    .map_err(|_| {
                        format!("Failed to read config {}: File not found", path.display())
                    })?;
                // Strip comments (JSONC support)
                json_strip_comments::strip(&mut json_string).map_err(|err| {
                    format!("Failed to strip comments from {}: {err}", path.display())
                })?;
                json_string
            }
            None => "{}".to_string(),
        };

        // Parse as raw JSON value
        let raw_config: Value = serde_json::from_str(&json_string)
            .map_err(|err| format!("Failed to parse config: {err}"))?;

        Ok(Self { raw_config, cached_options: None })
    }

    /// Validate config and return ignore patterns for file walking.
    ///
    /// # Errors
    /// Returns error if config deserialization fails.
    pub fn build_and_validate(&mut self) -> Result<Vec<String>, String> {
        let oxfmtrc: Oxfmtrc = serde_json::from_value(self.raw_config.clone())
            .map_err(|err| format!("Failed to deserialize Oxfmtrc: {err}"))?;

        // TODO: Apply editorconfig settings
        // if let Some(editorconfig) = &self.editorconfig {
        //   // Priority: oxfmtrc default < editorconfig < user's oxfmtrc
        //   if oxfmtrc.print_width.is_none() && let Some(max_line_length) = editorconfig.get_max_line_length() {
        //     oxfmtrc.print_width = Some(max_line_length);
        //   }
        //   // ... others
        // }

        let (format_options, oxfmt_options) = oxfmtrc
            .into_options()
            .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

        // Apply our defaults for Prettier options too
        // e.g. set `printWidth: 100` if not specified (= Prettier default: 80)
        let mut external_options = self.raw_config.clone();
        Oxfmtrc::populate_prettier_config(&format_options, &mut external_options);

        let ignore_patterns = oxfmt_options.ignore_patterns.clone();

        // NOTE: Save cache for fast path: no per-file overrides
        self.cached_options = Some((format_options, oxfmt_options, external_options));

        Ok(ignore_patterns)
    }

    /// Resolve format options for a specific file.
    pub fn resolve(&self, strategy: &FormatFileStrategy) -> ResolvedOptions {
        // TODO: Check if editorconfig has any overrides for this file
        let has_editorconfig_and_overrides = false;

        #[cfg_attr(not(feature = "napi"), allow(unused_variables))]
        let (format_options, oxfmt_options, external_options) = if has_editorconfig_and_overrides {
            self.resolve_with_overrides(strategy)
        } else {
            // Resolve format options for a specific file.
            // Either:
            // - `.editorconfig` is NOT used
            // - or used but per-file overrides do NOT exist for this file
            self.cached_options
                .clone()
                .expect("`build_and_validate()` must be called before `resolve()`")
        };

        match strategy {
            FormatFileStrategy::OxcFormatter { .. } => {
                ResolvedOptions::OxcFormatter { format_options, external_options }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { .. } => {
                ResolvedOptions::ExternalFormatter { external_options }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatterPackageJson { .. } => {
                ResolvedOptions::ExternalFormatterPackageJson {
                    external_options,
                    sort_package_json: oxfmt_options.sort_package_json,
                }
            }
            #[cfg(not(feature = "napi"))]
            _ => {
                unreachable!("If `napi` feature is disabled, this should not be passed here")
            }
        }
    }

    /// Resolve format options for a specific file.
    /// Since `.editorconfig` may contain per-file patterns, options are resolved per-file.
    fn resolve_with_overrides(
        &self,
        _strategy: &FormatFileStrategy,
    ) -> (FormatOptions, OxfmtOptions, Value) {
        let oxfmtrc: Oxfmtrc = serde_json::from_value(self.raw_config.clone())
            .expect("`build_and_validate()` should catch this before `resolve()`");

        // TODO: Apply base + per-file editorconfig settings
        // if let Some(editorconfig) = &self.editorconfig.resolve(strategy.path()) {
        //   // Priority: oxfmtrc default < editorconfig < user's oxfmtrc
        //   if oxfmtrc.print_width.is_none() && let Some(max_line_length) = editorconfig.get_max_line_length() {
        //     oxfmtrc.print_width = Some(max_line_length);
        //   }
        //   // ... others
        // }

        let (format_options, oxfmt_options) = oxfmtrc
            .into_options()
            .expect("If this fails, there is an issue with editorconfig insertion above");

        // Apply our defaults for Prettier options too
        // e.g. set `printWidth: 100` if not specified (= Prettier default: 80)
        let mut external_options = self.raw_config.clone();
        Oxfmtrc::populate_prettier_config(&format_options, &mut external_options);

        (format_options, oxfmt_options, external_options)
    }
}
