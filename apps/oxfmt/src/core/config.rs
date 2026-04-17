use std::path::{Path, PathBuf};

use editorconfig_parser::{
    EditorConfig, EditorConfigProperties, EditorConfigProperty, EndOfLine, IndentStyle,
    MaxLineLength, QuoteType,
};
use fast_glob::glob_match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde_json::Value;
use tracing::instrument;

use oxc_formatter::FormatOptions;
use oxc_toml::Options as TomlFormatterOptions;

#[cfg(feature = "napi")]
use super::js_config::JsConfigLoaderCb;
use super::{
    FormatFileStrategy,
    oxfmtrc::{
        EndOfLineConfig, FormatConfig, OxfmtOptions, OxfmtOverrideConfig, Oxfmtrc,
        finalize_external_options, sync_external_options, to_oxfmt_options,
    },
    utils,
};

/// JSON/JSONC config file names, in order of preference.
const JSON_CONFIG_FILES: &[&str] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];
/// JS/TS config file extensions.
const JS_CONFIG_EXTENSIONS: &[&str] = &["ts", "mts", "cts", "js", "mjs", "cjs"];
/// Oxfmt JS/TS config file name.
/// Only `.ts` extension is supported, matching oxlint's behavior.
#[cfg(feature = "napi")]
const OXFMT_JS_CONFIG_NAME: &str = "oxfmt.config.ts";
/// Vite+ config file name that may contain Oxfmt config under a `.fmt` field.
/// Only `.ts` extension is supported, matching oxlint's behavior.
#[cfg(feature = "napi")]
const VITE_PLUS_CONFIG_NAME: &str = "vite.config.ts";

fn is_js_config_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|ext| JS_CONFIG_EXTENSIONS.contains(&ext))
}

#[cfg(feature = "napi")]
fn is_vite_plus_config(path: &Path) -> bool {
    path.file_name().and_then(|f| f.to_str()).is_some_and(|name| name == VITE_PLUS_CONFIG_NAME)
}

/// Whether Vite+ mode is active (i.e., `VP_VERSION` env var is set).
#[cfg(feature = "napi")]
fn is_vite_plus_mode() -> bool {
    std::env::var_os("VP_VERSION").is_some()
}

/// Returns an iterator of all supported config file names, in priority order.
///
/// When `VP_VERSION` env var is set, only `vite.config.ts` is recognized.
/// When it is not set, `vite.config.ts` is excluded from the candidates.
pub fn all_config_file_names() -> impl Iterator<Item = &'static str> {
    #[cfg(feature = "napi")]
    {
        if is_vite_plus_mode() {
            return vec![VITE_PLUS_CONFIG_NAME].into_iter();
        }
        JSON_CONFIG_FILES
            .iter()
            .copied()
            .chain([OXFMT_JS_CONFIG_NAME])
            .collect::<Vec<_>>()
            .into_iter()
    }
    #[cfg(not(feature = "napi"))]
    JSON_CONFIG_FILES.iter().copied()
}

pub fn resolve_editorconfig_path(cwd: &Path) -> Option<PathBuf> {
    // Search the nearest `.editorconfig` from cwd upwards
    cwd.ancestors().map(|dir| dir.join(".editorconfig")).find(|p| p.exists())
}

/// Resolve format options directly from a raw JSON config value.
///
/// This is the simplified path for the NAPI `format()` API,
/// which doesn't need `.oxfmtrc` overrides, `.editorconfig`, or ignore patterns.
///
/// If `cwd` is provided, relative Tailwind paths are resolved against it.
#[cfg(feature = "napi")]
pub fn resolve_options_from_value(
    raw_config: Value,
    strategy: &FormatFileStrategy,
    cwd: Option<&Path>,
) -> Result<ResolvedOptions, String> {
    let mut format_config: FormatConfig =
        serde_json::from_value(raw_config).map_err(|err| err.to_string())?;
    if let Some(cwd) = cwd {
        format_config.resolve_tailwind_paths(cwd);
    }

    let mut external_options =
        serde_json::to_value(&format_config).expect("FormatConfig serialization should not fail");
    let oxfmt_options = to_oxfmt_options(format_config)?;

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
        /// For embedded language (xxx-in-js) formatting
        external_options: Value,
        /// Optional filepath override for external callbacks (e.g., Tailwind sorter).
        /// When set, this path is used instead of `FormatFileStrategy::path`
        /// as the `options.filepath` passed to external callbacks.
        /// Needed for js-in-xxx where the strategy path is a dummy,
        /// but callbacks need the parent file path to resolve their config.
        filepath_override: Option<PathBuf>,
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
                filepath_override: None,
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
    /// Ignore glob built from this config's `ignorePatterns`.
    ignore_glob: Option<Gitignore>,
}

impl ConfigResolver {
    /// Shared internal constructor used by both `from_json_config()` (JSON/JSONC)
    /// and `from_config()` (JS/TS config evaluated externally).
    fn new(
        raw_config: Value,
        config_dir: Option<PathBuf>,
        editorconfig: Option<EditorConfig>,
    ) -> Self {
        Self {
            raw_config,
            config_dir,
            cached_options: None,
            oxfmtrc_overrides: None,
            editorconfig,
            ignore_glob: None,
        }
    }

    /// Returns the directory containing the config file, if any was loaded.
    pub fn config_dir(&self) -> Option<&Path> {
        self.config_dir.as_deref()
    }

    /// Returns `true` if this config has any `ignorePatterns`.
    pub fn has_ignore_patterns(&self) -> bool {
        self.ignore_glob.is_some()
    }

    /// Returns `true` if the given path should be ignored by this config's `ignorePatterns`.
    pub fn is_path_ignored(&self, path: &Path, is_dir: bool) -> bool {
        self.ignore_glob.as_ref().is_some_and(|glob| {
            // `matched_path_or_any_parents()` panics if path is not under the glob's root.
            path.starts_with(glob.path())
                && glob.matched_path_or_any_parents(path, is_dir).is_ignore()
        })
    }

    /// Create a resolver, handling both JSON/JSONC and JS/TS config files.
    ///
    /// When `oxfmtrc_path` is `Some`, it is treated as an explicitly specified config file.
    /// When `oxfmtrc_path` is `None`, auto-discovery searches upwards from `cwd`.
    ///
    /// If the resolved config path is a JS/TS file:
    /// - With `napi` feature: evaluates it via the provided `js_config_loader` callback.
    /// - Without `napi` feature: returns an error (requires the Node.js CLI).
    ///
    /// # Errors
    /// Returns error if config file loading or parsing fails.
    pub fn from_config(
        cwd: &Path,
        oxfmtrc_path: Option<&Path>,
        editorconfig_path: Option<&Path>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Result<Self, String> {
        // Explicit path: normalize and load directly
        if let Some(config_path) = oxfmtrc_path {
            let path = utils::normalize_relative_path(cwd, config_path);
            return Self::load_config_at(
                cwd,
                &path,
                editorconfig_path,
                #[cfg(feature = "napi")]
                js_config_loader,
            );
        }

        // Auto-discovery: search upwards from cwd, load in one pass
        Self::discover_config(
            cwd,
            editorconfig_path,
            #[cfg(feature = "napi")]
            js_config_loader,
        )
    }

    /// Load a config file at a known path.
    /// Handles both JSON/JSONC and JS/TS config files.
    fn load_config_at(
        cwd: &Path,
        path: &Path,
        editorconfig_path: Option<&Path>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Result<Self, String> {
        #[cfg(not(feature = "napi"))]
        if is_js_config_file(path) {
            return Err(
                "JS/TS config files are not supported in pure Rust CLI.\nUse JSON/JSONC instead."
                    .to_string(),
            );
        }

        #[cfg(feature = "napi")]
        if is_js_config_file(path) {
            // Load successful and `.fmt` field found -> Use it as config
            // Load failed (e.g. syntax error, missing dependencies) -> Propagate error
            let raw_config = load_js_config(
                js_config_loader
                    .expect("JS config loader must be set when `napi` feature is enabled"),
                path,
            )?
            // Load successful but no `.fmt` field -> Error (explicitly specified config must have it)
            .ok_or_else(|| {
                format!("Expected a `fmt` field in the default export of {}", path.display())
            })?;

            let editorconfig = load_editorconfig(cwd, editorconfig_path)?;
            return Ok(Self::new(raw_config, path.parent().map(Path::to_path_buf), editorconfig));
        }

        Self::from_json_config(cwd, Some(path), editorconfig_path)
    }

    /// Auto-discover and load config by searching upwards from `cwd`.
    ///
    /// Tries each candidate file in priority order. If a `vite.config.ts` is found
    /// but lacks a `.fmt` field, it is skipped and the search continues.
    fn discover_config(
        cwd: &Path,
        editorconfig_path: Option<&Path>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Result<Self, String> {
        let candidates: Vec<&str> = all_config_file_names().collect();
        for dir in cwd.ancestors() {
            for filename in &candidates {
                let path = dir.join(filename);
                if !path.exists() {
                    continue;
                }

                // For `vite.config.ts`
                #[cfg(feature = "napi")]
                if is_vite_plus_config(&path) {
                    // Load successful and `.fmt` field found -> Use it as config
                    // Load failed (e.g. syntax error, missing dependencies) -> Propagate error
                    if let Some(raw_config) = load_js_config(
                        js_config_loader
                            .expect("JS config loader must be set when `napi` feature is enabled"),
                        &path,
                    )? {
                        let editorconfig = load_editorconfig(cwd, editorconfig_path)?;
                        let config_dir = path.parent().map(Path::to_path_buf);
                        return Ok(Self::new(raw_config, config_dir, editorconfig));
                    }
                    // Load successful but no `.fmt` field found -> Skip this file and continue searching.
                    continue;
                }

                // Use Oxfmt config if found, even if a `vite.config.ts` with missing `.fmt` is present.
                return Self::load_config_at(
                    cwd,
                    &path,
                    editorconfig_path,
                    #[cfg(feature = "napi")]
                    js_config_loader,
                );
            }
        }

        // No config found — use defaults
        Self::from_json_config(cwd, None, editorconfig_path)
    }

    /// Create a resolver by loading JSON/JSONC config from a file path.
    ///
    /// Also used as the default (empty config) fallback when no config file is found.
    #[instrument(level = "debug", name = "oxfmt::config::from_json_config", skip_all)]
    pub(crate) fn from_json_config(
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
        let raw_config: Value =
            serde_json::from_str(&json_string).map_err(|err| err.to_string())?;
        // Store the config directory for override path resolution
        let config_dir = oxfmtrc_path.and_then(|p| p.parent().map(Path::to_path_buf));
        let editorconfig = load_editorconfig(cwd, editorconfig_path)?;

        Ok(Self::new(raw_config, config_dir, editorconfig))
    }

    /// Validate config and return ignore patterns (= non-formatting option) for file walking.
    ///
    /// Validated options are cached for fast path resolution.
    ///
    /// # Errors
    /// Returns error if config deserialization fails.
    #[instrument(level = "debug", name = "oxfmt::config::build_and_validate", skip_all)]
    pub fn build_and_validate(&mut self) -> Result<(), String> {
        let oxfmtrc: Oxfmtrc =
            serde_json::from_value(self.raw_config.clone()).map_err(|err| err.to_string())?;

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
        let oxfmt_options = to_oxfmt_options(format_config)?;

        // Apply common Prettier mappings for caching.
        // Plugin options will be added later in `resolve()` via `finalize_external_options()`.
        // If we finalize here, every per-file options contain plugin options even if not needed.
        sync_external_options(&oxfmt_options.format_options, &mut external_options);

        // Save cache for fast path: no per-file overrides
        self.cached_options = Some((oxfmt_options, external_options));

        // Build ignore glob from `ignorePatterns` config field
        let ignore_patterns = oxfmtrc.ignore_patterns.unwrap_or_default();
        self.ignore_glob = build_ignore_glob(self.config_dir.as_deref(), &ignore_patterns)?;

        Ok(())
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
        let oxfmt_options = to_oxfmt_options(format_config)
            .expect("If this fails, there is an issue with override values");

        sync_external_options(&oxfmt_options.format_options, &mut external_options);

        (oxfmt_options, external_options)
    }
}

/// Load a JS/TS config file via NAPI and return the raw JSON value.
///
/// Returns `Ok(None)` when the JS side returns `null` for `vite.config.ts` without `.fmt` field,
/// signaling that this config should be skipped during auto-discovery.
#[cfg(feature = "napi")]
fn load_js_config(
    js_config_loader: &JsConfigLoaderCb,
    path: &Path,
) -> Result<Option<Value>, String> {
    let value = js_config_loader(path.to_string_lossy().into_owned()).map_err(|err| {
        format!(
            "{}\n{err}\nEnsure the file has a valid default export of a JSON-serializable configuration object.",
            path.display()
        )
    })?;

    Ok(if value.is_null() { None } else { Some(value) })
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

/// Load `.editorconfig` from a path if provided.
fn load_editorconfig(
    cwd: &Path,
    editorconfig_path: Option<&Path>,
) -> Result<Option<EditorConfig>, String> {
    match editorconfig_path {
        Some(path) => {
            let str = utils::read_to_string(path)
                .map_err(|_| format!("Failed to read {}: File not found", path.display()))?;

            // Use the directory containing `.editorconfig` as the base, not the CLI's cwd.
            // This ensures patterns like `[src/*.ts]` are resolved relative to where `.editorconfig` is located.
            Ok(Some(EditorConfig::parse(&str).with_cwd(path.parent().unwrap_or(cwd))))
        }
        None => Ok(None),
    }
}

/// Check if `.editorconfig` has per-file overrides for this path.
///
/// Returns `true` if the resolved properties differ from the root `[*]` section.
///
/// Currently, only the following properties are considered for overrides:
/// - max_line_length
/// - end_of_line
/// - indent_style
/// - indent_size
/// - tab_width
/// - insert_final_newline
/// - quote_type
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
                || resolved.tab_width != root.tab_width
                || resolved.insert_final_newline != root.insert_final_newline
                || resolved.quote_type != root.quote_type
        }
        // No `[*]` section means any resolved property is an override
        None => {
            resolved.max_line_length != EditorConfigProperty::Unset
                || resolved.end_of_line != EditorConfigProperty::Unset
                || resolved.indent_style != EditorConfigProperty::Unset
                || resolved.indent_size != EditorConfigProperty::Unset
                || resolved.tab_width != EditorConfigProperty::Unset
                || resolved.insert_final_newline != EditorConfigProperty::Unset
                || resolved.quote_type != EditorConfigProperty::Unset
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

    if config.tab_width.is_none() {
        // Match Prettier's behavior: Only use `indent_size` when `useTabs: false`.
        // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/config/editorconfig/editorconfig-to-prettier.js#L25-L30
        #[expect(clippy::cast_possible_truncation)]
        if config.use_tabs == Some(false)
            && let EditorConfigProperty::Value(size) = props.indent_size
        {
            config.tab_width = Some(size as u8);
        } else if let EditorConfigProperty::Value(size) = props.tab_width {
            config.tab_width = Some(size as u8);
        }
    }

    if config.insert_final_newline.is_none()
        && let EditorConfigProperty::Value(v) = props.insert_final_newline
    {
        config.insert_final_newline = Some(v);
    }

    if config.single_quote.is_none() {
        match props.quote_type {
            EditorConfigProperty::Value(QuoteType::Single) => {
                config.single_quote = Some(true);
            }
            EditorConfigProperty::Value(QuoteType::Double) => {
                config.single_quote = Some(false);
            }
            _ => {}
        }
    }
}

// ---

/// Check if a directory contains any recognized config file.
pub fn has_config_in_directory(dir: &Path) -> bool {
    all_config_file_names().any(|name| dir.join(name).exists())
}

/// Build an ignore glob from config `ignorePatterns`.
/// Patterns are resolved relative to the config file's directory.
fn build_ignore_glob(
    config_dir: Option<&Path>,
    ignore_patterns: &[String],
) -> Result<Option<Gitignore>, String> {
    if ignore_patterns.is_empty() {
        return Ok(None);
    }
    let Some(config_dir) = config_dir else {
        return Ok(None);
    };

    let mut builder = GitignoreBuilder::new(config_dir);
    for pattern in ignore_patterns {
        if builder.add_line(None, pattern).is_err() {
            return Err(format!("Failed to add ignore pattern `{pattern}` from `ignorePatterns`"));
        }
    }
    let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
    Ok(Some(gitignore))
}
