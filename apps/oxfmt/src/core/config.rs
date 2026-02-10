use std::path::{Path, PathBuf};

use editorconfig_parser::{
    EditorConfig, EditorConfigProperties, EditorConfigProperty, EndOfLine, IndentStyle,
    MaxLineLength,
};
use fast_glob::glob_match;
use serde_json::Value;
use tracing::instrument;

use oxc_formatter::FormatOptions;
use oxc_toml::Options as TomlFormatterOptions;

use super::{
    FormatFileStrategy,
    oxfmtrc::{
        EndOfLineConfig, FormatConfig, OxfmtOptions, OxfmtOverrideConfig, Oxfmtrc,
        finalize_external_options, sync_external_options,
    },
    utils,
};

/// Resolve config file path from cwd and optional explicit path.
pub fn resolve_oxfmtrc_path(cwd: &Path, config_path: Option<&Path>) -> Option<PathBuf> {
    // If `--config` is explicitly specified, use that path
    if let Some(config_path) = config_path {
        return Some(utils::normalize_relative_path(cwd, config_path));
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

pub fn resolve_editorconfig_path(cwd: &Path) -> Option<PathBuf> {
    // Search the nearest `.editorconfig` from cwd upwards
    cwd.ancestors().map(|dir| dir.join(".editorconfig")).find(|p| p.exists())
}

/// Resolve format options directly from a raw JSON config value.
///
/// This is the simplified path for the NAPI `format()` API,
/// which doesn't need `.oxfmtrc` overrides, `.editorconfig`, or ignore patterns.
#[cfg(feature = "napi")]
pub fn resolve_options_from_value(
    cwd: &Path,
    raw_config: Value,
    strategy: &FormatFileStrategy,
) -> Result<ResolvedOptions, String> {
    let mut format_config: FormatConfig = serde_json::from_value(raw_config)
        .map_err(|err| format!("Failed to deserialize FormatConfig: {err}"))?;
    format_config.resolve_tailwind_paths(cwd);

    let mut external_options =
        serde_json::to_value(&format_config).expect("FormatConfig serialization should not fail");
    let oxfmt_options = format_config
        .into_oxfmt_options()
        .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

    sync_external_options(&oxfmt_options.format_options, &mut external_options);

    Ok(ResolvedOptions::from_oxfmt_options(oxfmt_options, external_options, strategy))
}

// ---

/// Resolved options for each file type.
/// Each variant contains only the options needed for that formatter.
#[derive(Debug)]
pub enum ResolvedOptions {
    /// For JS/TS files formatted by oxc_formatter.
    OxcFormatter {
        format_options: Box<FormatOptions>,
        /// For embedded language formatting (e.g., CSS in template literals)
        external_options: Value,
        insert_final_newline: bool,
    },
    /// For TOML files.
    OxfmtToml { toml_options: TomlFormatterOptions, insert_final_newline: bool },
    /// For non-JS files formatted by external formatter (Prettier).
    #[cfg(feature = "napi")]
    ExternalFormatter { external_options: Value, insert_final_newline: bool },
    /// For `package.json` files: optionally sorted then formatted.
    #[cfg(feature = "napi")]
    ExternalFormatterPackageJson {
        external_options: Value,
        sort_package_json: Option<sort_package_json::SortOptions>,
        insert_final_newline: bool,
    },
}

impl ResolvedOptions {
    /// Build `ResolvedOptions` from `OxfmtOptions`, `external_options`, and `FormatFileStrategy`.
    ///
    /// This also applies plugin-specific options (Tailwind, oxfmt plugin flags) based on strategy.
    fn from_oxfmt_options(
        oxfmt_options: OxfmtOptions,
        mut external_options: Value,
        strategy: &FormatFileStrategy,
    ) -> Self {
        // Apply plugin-specific options based on strategy
        finalize_external_options(&mut external_options, strategy);

        #[cfg(feature = "napi")]
        let OxfmtOptions { format_options, toml_options, sort_package_json, insert_final_newline } =
            oxfmt_options;
        #[cfg(not(feature = "napi"))]
        let OxfmtOptions { format_options, toml_options, insert_final_newline, .. } = oxfmt_options;

        match strategy {
            FormatFileStrategy::OxcFormatter { .. } => ResolvedOptions::OxcFormatter {
                format_options: Box::new(format_options),
                external_options,
                insert_final_newline,
            },
            FormatFileStrategy::OxfmtToml { .. } => {
                ResolvedOptions::OxfmtToml { toml_options, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { .. } => {
                ResolvedOptions::ExternalFormatter { external_options, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatterPackageJson { .. } => {
                ResolvedOptions::ExternalFormatterPackageJson {
                    external_options,
                    sort_package_json,
                    insert_final_newline,
                }
            }
            #[cfg(not(feature = "napi"))]
            _ => {
                unreachable!("If `napi` feature is disabled, this should not be passed here")
            }
        }
    }
}

// ---

/// Configuration resolver to handle `.oxfmtrc` and `.editorconfig` files.
///
/// Priority order: `Oxfmtrc::default()` → user's `.oxfmtrc` base → `.oxfmtrc` overrides
/// `.editorconfig` is applied as fallback for unset fields only.
#[derive(Debug)]
pub struct ConfigResolver {
    /// User's raw config as JSON value.
    /// It contains every possible field, even those not recognized by `Oxfmtrc`.
    /// e.g. `printWidth`: recognized by both `Oxfmtrc` and Prettier
    /// e.g. `vueIndentScriptAndStyle`: not recognized by `Oxfmtrc`, but used by Prettier
    /// e.g. `svelteSortAttributes`: not recognized by Prettier and require plugins
    raw_config: Value,
    /// Directory containing the config file (for relative path resolution in overrides).
    config_dir: Option<PathBuf>,
    /// Cached parsed options after validation.
    /// Used to avoid re-parsing during per-file resolution, if no per-file overrides exist.
    cached_options: Option<(OxfmtOptions, Value)>,
    /// Resolved overrides from `.oxfmtrc` for file-specific matching.
    oxfmtrc_overrides: Option<OxfmtrcOverrides>,
    /// Parsed `.editorconfig`, if any.
    editorconfig: Option<EditorConfig>,
}

impl ConfigResolver {
    /// Create a resolver by loading config from a file path.
    ///
    /// # Errors
    /// Returns error if:
    /// - Config file is specified but not found or invalid
    /// - Config file parsing fails
    #[instrument(level = "debug", name = "oxfmt::config::from_config_paths", skip_all)]
    pub fn from_config_paths(
        cwd: &Path,
        oxfmtrc_path: Option<&Path>,
        editorconfig_path: Option<&Path>,
    ) -> Result<Self, String> {
        // Read and parse config file, or use empty JSON if not found
        let json_string = match oxfmtrc_path {
            Some(path) => {
                let mut json_string = utils::read_to_string(path)
                    // Do not include OS error, it differs between platforms
                    .map_err(|_| format!("Failed to read {}: File not found", path.display()))?;
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
        // Store the config directory for override path resolution
        let config_dir = oxfmtrc_path.and_then(|p| p.parent().map(Path::to_path_buf));

        let editorconfig = match editorconfig_path {
            Some(path) => {
                let str = utils::read_to_string(path)
                    .map_err(|_| format!("Failed to read {}: File not found", path.display()))?;

                // Use the directory containing `.editorconfig` as the base, not the CLI's cwd.
                // This ensures patterns like `[src/*.ts]` are resolved relative to where `.editorconfig` is located.
                Some(EditorConfig::parse(&str).with_cwd(path.parent().unwrap_or(cwd)))
            }
            None => None,
        };

        Ok(Self {
            raw_config,
            config_dir,
            cached_options: None,
            oxfmtrc_overrides: None,
            editorconfig,
        })
    }

    /// Validate config and return ignore patterns (= non-formatting option) for file walking.
    ///
    /// Validated options are cached for fast path resolution.
    ///
    /// # Errors
    /// Returns error if config deserialization fails.
    #[instrument(level = "debug", name = "oxfmt::config::build_and_validate", skip_all)]
    pub fn build_and_validate(&mut self) -> Result<Vec<String>, String> {
        let oxfmtrc: Oxfmtrc = serde_json::from_value(self.raw_config.clone())
            .map_err(|err| format!("Failed to deserialize Oxfmtrc: {err}"))?;

        // Resolve `overrides` from `Oxfmtrc` for later per-file matching
        let base_dir = self.config_dir.clone();
        self.oxfmtrc_overrides =
            oxfmtrc.overrides.map(|overrides| OxfmtrcOverrides::new(overrides, base_dir));

        let mut format_config = oxfmtrc.format_config;

        // If `.editorconfig` is used, apply its root section first
        // If there are per-file overrides, they will be applied during `resolve()`
        if let Some(editorconfig) = &self.editorconfig
            && let Some(props) =
                editorconfig.sections().iter().find(|s| s.name == "*").map(|s| &s.properties)
        {
            apply_editorconfig(&mut format_config, props);
        }

        // Resolve relative tailwind paths before serialization
        if let Some(config_dir) = &self.config_dir {
            format_config.resolve_tailwind_paths(config_dir);
        }

        // NOTE: Revisit this when adding Prettier plugin support.
        // We use `format_config` directly instead of merging with `raw_config`.
        // To preserve plugin-specific options,
        // we would need to merge `raw_config` first, then apply `format_config` values on top.
        // Or we could keep track of plugin options separately in `FormatConfig` itself,
        // like how Tailwindcss options are handled currently.
        let mut external_options = serde_json::to_value(&format_config)
            .expect("FormatConfig serialization should not fail");

        // Convert `FormatConfig` to `OxfmtOptions`, applying defaults where needed
        let oxfmt_options = format_config
            .into_oxfmt_options()
            .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

        // Apply common Prettier mappings for caching.
        // Plugin options will be added later in `resolve()` via `finalize_external_options()`.
        // If we finalize here, every per-file options contain plugin options even if not needed.
        sync_external_options(&oxfmt_options.format_options, &mut external_options);

        // Save cache for fast path: no per-file overrides
        self.cached_options = Some((oxfmt_options, external_options));

        let ignore_patterns = oxfmtrc.ignore_patterns.unwrap_or_default();
        Ok(ignore_patterns)
    }

    /// Resolve format options for a specific file.
    #[instrument(level = "debug", name = "oxfmt::config::resolve", skip_all, fields(path = %strategy.path().display()))]
    pub fn resolve(&self, strategy: &FormatFileStrategy) -> ResolvedOptions {
        let (oxfmt_options, external_options) = self.resolve_options(strategy.path());
        ResolvedOptions::from_oxfmt_options(oxfmt_options, external_options, strategy)
    }

    /// Resolve options for a specific file path.
    /// Priority: oxfmtrc base → oxfmtrc overrides → editorconfig (fallback for unset fields) -> defaults
    ///
    /// Returns cached options (with `strategy: None` applied) for later plugin option addition.
    fn resolve_options(&self, path: &Path) -> (OxfmtOptions, Value) {
        let has_editorconfig_overrides =
            self.editorconfig.as_ref().is_some_and(|ec| has_editorconfig_overrides(ec, path));
        let has_oxfmtrc_overrides =
            self.oxfmtrc_overrides.as_ref().is_some_and(|o| o.has_match(path));

        // Fast path: no per-file overrides
        // `.editorconfig` root section is already applied during `build_and_validate()`
        if !has_editorconfig_overrides && !has_oxfmtrc_overrides {
            return self
                .cached_options
                .clone()
                .expect("`build_and_validate()` must be called first");
        }

        // Slow path: reconstruct `FormatConfig` to apply overrides
        // Overrides are merged at `FormatConfig` level, not `OxfmtOptions` level
        let mut format_config: FormatConfig = serde_json::from_value(self.raw_config.clone())
            .expect("`build_and_validate()` should catch this before");

        // Apply oxfmtrc overrides first (explicit settings)
        if let Some(overrides) = &self.oxfmtrc_overrides {
            for options in overrides.get_matching(path) {
                format_config.merge(options);
            }
        }
        // Apply `.editorconfig` as fallback (fills in unset fields only)
        if let Some(ec) = &self.editorconfig {
            let props = ec.resolve(path);
            apply_editorconfig(&mut format_config, &props);
        }

        // Resolve relative tailwind paths before serialization
        if let Some(config_dir) = &self.config_dir {
            format_config.resolve_tailwind_paths(config_dir);
        }

        // NOTE: See `build_and_validate()` for details about `external_options` handling
        let mut external_options = serde_json::to_value(&format_config)
            .expect("FormatConfig serialization should not fail");
        let oxfmt_options = format_config
            .into_oxfmt_options()
            .expect("If this fails, there is an issue with override values");

        sync_external_options(&oxfmt_options.format_options, &mut external_options);

        (oxfmt_options, external_options)
    }
}

// ---

/// Resolved overrides from `.oxfmtrc` for file-specific matching.
/// Similar to `EditorConfig`, this handles `FormatConfig` override resolution.
#[derive(Debug)]
struct OxfmtrcOverrides {
    base_dir: Option<PathBuf>,
    entries: Vec<OxfmtrcOverrideEntry>,
}

impl OxfmtrcOverrides {
    fn new(overrides: Vec<OxfmtOverrideConfig>, base_dir: Option<PathBuf>) -> Self {
        // Normalize glob patterns by adding `**/` prefix to patterns without `/`.
        // This matches ESLint/Prettier behavior.
        let normalize_patterns = |patterns: Vec<String>| {
            patterns
                .into_iter()
                // This may be problematic if user writes glob patterns with `\` as separator on Windows.
                // But fine for now since:
                // - `fast_glob::glob_match()` supports both `/` and `\`
                // - Glob patterns are usually written with `/` even on Windows
                .map(|pat| if pat.contains('/') { pat } else { format!("**/{pat}") })
                .collect()
        };

        Self {
            base_dir,
            entries: overrides
                .into_iter()
                .map(|o| OxfmtrcOverrideEntry {
                    files: normalize_patterns(o.files),
                    exclude_files: o.exclude_files.map(normalize_patterns).unwrap_or_default(),
                    options: o.options,
                })
                .collect(),
        }
    }

    /// Check if any overrides exist that match the given path.
    fn has_match(&self, path: &Path) -> bool {
        let relative = self.relative_path(path);
        self.entries.iter().any(|e| Self::is_entry_match(e, &relative))
    }

    /// Get all matching override options for a given path.
    fn get_matching(&self, path: &Path) -> impl Iterator<Item = &FormatConfig> + '_ {
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

    fn is_entry_match(entry: &OxfmtrcOverrideEntry, relative: &str) -> bool {
        entry.files.iter().any(|glob| glob_match(glob, relative))
            && !entry.exclude_files.iter().any(|glob| glob_match(glob, relative))
    }
}

/// A single override entry with normalized glob patterns.
/// NOTE: Written path patterns are glob patterns; use `/` as the path separator on all platforms.
#[derive(Debug)]
struct OxfmtrcOverrideEntry {
    files: Vec<String>,
    exclude_files: Vec<String>,
    options: FormatConfig,
}

// ---

/// Check if `.editorconfig` has per-file overrides for this path.
///
/// Returns `true` if the resolved properties differ from the root `[*]` section.
///
/// Currently, only the following properties are considered for overrides:
/// - max_line_length
/// - end_of_line
/// - indent_style
/// - indent_size
/// - insert_final_newline
fn has_editorconfig_overrides(editorconfig: &EditorConfig, path: &Path) -> bool {
    let sections = editorconfig.sections();

    // No sections, or only root `[*]` section → no overrides
    if sections.is_empty() || matches!(sections, [s] if s.name == "*") {
        return false;
    }

    let resolved = editorconfig.resolve(path);

    // Get the root `[*]` section properties
    let root_props = sections.iter().find(|s| s.name == "*").map(|s| &s.properties);

    // Compare only the properties we care about
    match root_props {
        Some(root) => {
            resolved.max_line_length != root.max_line_length
                || resolved.end_of_line != root.end_of_line
                || resolved.indent_style != root.indent_style
                || resolved.indent_size != root.indent_size
                || resolved.insert_final_newline != root.insert_final_newline
        }
        // No `[*]` section means any resolved property is an override
        None => {
            resolved.max_line_length != EditorConfigProperty::Unset
                || resolved.end_of_line != EditorConfigProperty::Unset
                || resolved.indent_style != EditorConfigProperty::Unset
                || resolved.indent_size != EditorConfigProperty::Unset
                || resolved.insert_final_newline != EditorConfigProperty::Unset
        }
    }
}

/// Apply `.editorconfig` properties to `FormatConfig`.
///
/// Only applies values that are not already set in the user's config.
/// NOTE: Only properties checked by [`has_editorconfig_overrides`] are applied here.
fn apply_editorconfig(config: &mut FormatConfig, props: &EditorConfigProperties) {
    #[expect(clippy::cast_possible_truncation)]
    if config.print_width.is_none()
        && let EditorConfigProperty::Value(MaxLineLength::Number(v)) = props.max_line_length
    {
        config.print_width = Some(v as u16);
    }

    if config.end_of_line.is_none()
        && let EditorConfigProperty::Value(eol) = props.end_of_line
    {
        config.end_of_line = Some(match eol {
            EndOfLine::Lf => EndOfLineConfig::Lf,
            EndOfLine::Cr => EndOfLineConfig::Cr,
            EndOfLine::Crlf => EndOfLineConfig::Crlf,
        });
    }

    if config.use_tabs.is_none()
        && let EditorConfigProperty::Value(style) = props.indent_style
    {
        config.use_tabs = Some(match style {
            IndentStyle::Tab => true,
            IndentStyle::Space => false,
        });
    }

    #[expect(clippy::cast_possible_truncation)]
    if config.tab_width.is_none()
        && let EditorConfigProperty::Value(size) = props.indent_size
    {
        config.tab_width = Some(size as u8);
    }

    if config.insert_final_newline.is_none()
        && let EditorConfigProperty::Value(v) = props.insert_final_newline
    {
        config.insert_final_newline = Some(v);
    }
}
