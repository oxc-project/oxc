mod editorconfig;
#[cfg(feature = "napi")]
mod js_config;
mod nested;
mod overrides;
mod scopes;

#[cfg(feature = "napi")]
pub use js_config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader};
pub use nested::NestedConfigCtx;
pub use scopes::ConfigScopes;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use editorconfig_parser::EditorConfig;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::Deserialize;
use serde_json::Value;
use tracing::instrument;

use oxc_config::{ConfigDiscovery, DiscoveredConfigFile, is_js_config_path, vp_version};

use self::{
    editorconfig::{apply_editorconfig, resolve_editorconfig_overrides, root_properties},
    overrides::OxfmtrcOverrides,
};
use super::{
    FormatStrategy,
    global_ignore::matches_with_ancestors,
    options::{ValidatedOptions, validate},
    oxfmtrc::{FormatConfig, Oxfmtrc},
    support::FileKind,
    utils,
};

pub fn config_discovery() -> ConfigDiscovery {
    if cfg!(feature = "napi") && vp_version().is_some() {
        ConfigDiscovery::vite_plus()
    } else {
        ConfigDiscovery::oxfmt()
    }
}

/// Everything a config file load needs besides the file itself,
/// shared by the root load ([`ConfigScopes::load`]) and nested probes ([`NestedConfigCtx`]).
///
/// Cloning is shallow.
#[derive(Clone)]
struct ConfigLoader {
    discovery: ConfigDiscovery,
    /// Parsed `.editorconfig`, shared by every resolver this loads,
    /// instead of re-reading and re-parsing the same file.
    editorconfig: Option<Arc<EditorConfig>>,
    #[cfg(feature = "napi")]
    js_loader: Option<JsConfigLoaderCb>,
}

impl ConfigLoader {
    fn new(
        editorconfig: Option<EditorConfig>,
        #[cfg(feature = "napi")] js_loader: Option<JsConfigLoaderCb>,
    ) -> Self {
        Self {
            discovery: config_discovery(),
            editorconfig: editorconfig.map(Arc::new),
            #[cfg(feature = "napi")]
            js_loader,
        }
    }

    /// Load the root config, handling both JSON/JSONC and JS/TS config files.
    ///
    /// When `explicit_config` is `Some`, it is treated as an explicitly specified config file.
    /// When `explicit_config` is `None`, auto-discovery searches upwards from `cwd`,
    /// and falls back to the default (empty) config.
    ///
    /// # Errors
    /// Returns error if config file loading or parsing fails.
    fn load_root(
        &self,
        cwd: &Path,
        explicit_config: Option<&Path>,
    ) -> Result<ConfigResolver, String> {
        // Explicit path: normalize and load directly
        if let Some(config_path) = explicit_config {
            let path = utils::normalize_relative_path(cwd, config_path);
            if !is_js_config_path(&path) {
                return ConfigResolver::from_json_config(Some(&path), self.editorconfig.clone());
            }
            let raw_config = self
                .load_js_config(&path)?
                // Explicit `--config`: missing `.fmt` is an error.
                .ok_or_else(|| {
                    format!("Expected a `fmt` field in the default export of {}", path.display())
                })?;
            return Ok(self.js_resolver(&path, raw_config));
        }

        // Auto-discovery: search upwards from cwd, load in one pass
        for dir in cwd.ancestors() {
            if let Some(resolver) = self.load_in_dir(dir)? {
                return Ok(resolver);
            }
        }

        // No config found, use defaults
        ConfigResolver::from_json_config(None, self.editorconfig.clone())
    }

    /// Load a config file located directly inside `dir` (no `build_and_validate`).
    ///
    /// NOTE: Returns `Ok(None)` when `dir` has no config file,
    /// or the file is a `vite.config.*` whose default export lacks a `.fmt` field.
    /// Callers decide how to handle it:
    /// - [`Self::load_root`] (ancestor walk): skip and continue upward
    /// - [`NestedConfigCtx`] (nested probe): no config in this dir
    fn load_in_dir(&self, dir: &Path) -> Result<Option<ConfigResolver>, String> {
        let Some(config_file) = self
            .discovery
            .find_unique_config_by_readdir(dir, false)
            .map_err(|e| Into::<oxc_diagnostics::OxcDiagnostic>::into(e).to_string())?
        else {
            return Ok(None);
        };

        let (path, raw_config) = match config_file {
            DiscoveredConfigFile::Json(path) | DiscoveredConfigFile::Jsonc(path) => {
                return ConfigResolver::from_json_config(Some(&path), self.editorconfig.clone())
                    .map(Some);
            }
            DiscoveredConfigFile::Js(path) => {
                // Non-Vite JS config: `loadJsConfig` never returns `null`; failures bubble up as `Err`.
                let raw_config = self
                    .load_js_config(&path)?
                    .expect("loadJsConfig never returns null for non-Vite JS config");
                (path, raw_config)
            }
            DiscoveredConfigFile::Vite(path) => {
                let Some(raw_config) = self.load_js_config(&path)? else {
                    return Ok(None);
                };
                (path, raw_config)
            }
        };
        Ok(Some(self.js_resolver(&path, raw_config)))
    }

    fn js_resolver(&self, path: &Path, raw_config: Value) -> ConfigResolver {
        ConfigResolver::new(
            raw_config,
            path.parent().map(Path::to_path_buf),
            self.editorconfig.clone(),
        )
    }
}

#[cfg(feature = "napi")]
impl ConfigLoader {
    /// Load a JS/TS config file via NAPI and return the raw JSON value.
    ///
    /// Returns `Ok(None)` when the JS side returns `null` (Vite+ `.fmt` missing).
    fn load_js_config(&self, path: &Path) -> Result<Option<Value>, String> {
        let js_config_loader = self
            .js_loader
            .as_ref()
            .expect("JS config loader must be set when `napi` feature is enabled");
        let value = js_config_loader(path.to_string_lossy().into_owned()).map_err(|err| {
            format!(
                "{}\n{err}\nEnsure the file has a valid default export of a JSON-serializable configuration object.",
                path.display()
            )
        })?;

        Ok(if value.is_null() { None } else { Some(value) })
    }
}

#[cfg(not(feature = "napi"))]
impl ConfigLoader {
    /// JS/TS config files need the Node.js CLI.
    #[expect(clippy::unused_self)]
    fn load_js_config(&self, path: &Path) -> Result<Option<Value>, String> {
        Err(format!(
            "JS/TS config file ({}) is not supported in pure Rust CLI.\nUse JSON/JSONC instead.",
            path.display()
        ))
    }
}

// ---

/// Outcome of resolving a [`FileKind`] against a [`FormatConfig`],
/// constructed by [`ConfigResolver::resolve`] / [`resolve_for_api`].
#[derive(Debug)]
pub enum ResolveOutcome {
    /// Ready to format with this strategy.
    Format(FormatStrategy),
    /// The file's parser requires a plugin that the resolved config did NOT enable.
    /// The payload carries the missing config key (e.g. `"svelte"`)
    /// so callers can construct a friendly error or log message.
    #[cfg_attr(not(feature = "napi"), expect(dead_code))]
    MissingPlugin(&'static str),
}

/// Apply the missing-plugin gate, then build the [`ResolveOutcome`].
/// The gate's single home: [`ConfigResolver::resolve`] and [`resolve_for_api`] both end here,
/// so a plugin-gating change can never leave one path behind.
fn into_outcome(
    config: Arc<FormatConfig>,
    validated: Arc<ValidatedOptions>,
    kind: FileKind,
) -> ResolveOutcome {
    #[cfg(feature = "napi")]
    if let Some(plugin) = kind.requires_plugin(&config) {
        return ResolveOutcome::MissingPlugin(plugin);
    }
    ResolveOutcome::Format(FormatStrategy { kind, config, validated })
}

/// Resolve options for a pre-classified file and build a [`ResolveOutcome`].
///
/// This is the simplified path for the NAPI `format()` API.
/// It resolves the caller-supplied [`FormatConfig`] directly instead of
/// discovering or loading project configuration.
///
/// Relative Tailwind paths are resolved against provided `cwd`.
///
/// Returns `Err` only when the merged config fails validation.
#[cfg(feature = "napi")]
pub fn resolve_for_api(
    raw_config: Value,
    kind: FileKind,
    cwd: &Path,
) -> Result<ResolveOutcome, String> {
    let mut format_config: FormatConfig =
        serde_json::from_value(raw_config).map_err(|err| err.to_string())?;
    format_config.resolve_tailwind_paths(cwd);
    // Validate eagerly, as the single gate for every option (core + js/sortImports):
    // downstream mapping consumes the derived artifacts and cannot re-fail,
    // and `Prettier` kinds have no later chance before values reach Prettier.
    let validated = validate(&format_config)?;
    Ok(into_outcome(Arc::new(format_config), Arc::new(validated), kind))
}

// ---

/// Configuration resolver to handle `.oxfmtrc` and `.editorconfig` files.
///
/// Priority (later wins):
/// - `.editorconfig` (fallback for unset fields)
/// - `.oxfmtrc` base
/// - `.oxfmtrc` overrides matching the file path.
#[derive(Debug)]
pub struct ConfigResolver {
    /// User's raw config as JSON value.
    ///
    /// Retained because the slow path must re-deserialize [`FormatConfig`] from it. (see [`Self::resolve_options`]).
    /// Rebuilding from the typed `base` snapshot is not enough, since `apply_editorconfig` only fills `is_none()` fields,
    /// so per-file `[src/*.ts]` sections couldn't override values that the `[*]` section already baked in.
    raw_config: Value,
    /// Directory containing the config file (for relative path resolution in overrides).
    config_dir: Option<PathBuf>,
    /// Fast-path snapshot for files without per-file overrides:
    /// the typed `FormatConfig` (`.oxfmtrc` base + `.editorconfig` `[*]` folded in)
    /// together with its validation-gate artifacts,
    /// so the pair can never go stale against each other and the fast path re-derives nothing.
    /// `Arc` so the fast path hands out shares instead of deep-cloning per file.
    base: Option<(Arc<FormatConfig>, Arc<ValidatedOptions>)>,
    /// Resolved overrides from `.oxfmtrc` for file-specific matching.
    oxfmtrc_overrides: Option<OxfmtrcOverrides>,
    /// Ignore glob built from this config's `ignorePatterns`.
    ignore_glob: Option<Gitignore>,
    /// Parsed `.editorconfig`, if any.
    editorconfig: Option<Arc<EditorConfig>>,
}

impl ConfigResolver {
    /// Shared internal constructor used by both:
    /// - `from_json_config()` (JSON/JSONC)
    /// - and [`ConfigLoader`] (JS/TS config evaluated externally)
    fn new(
        raw_config: Value,
        config_dir: Option<PathBuf>,
        editorconfig: Option<Arc<EditorConfig>>,
    ) -> Self {
        Self {
            raw_config,
            config_dir,
            base: None,
            oxfmtrc_overrides: None,
            ignore_glob: None,
            editorconfig,
        }
    }

    /// Returns the directory containing the config file, if any was loaded.
    pub fn config_dir(&self) -> Option<&Path> {
        self.config_dir.as_deref()
    }

    /// Returns `true` if the given path should be ignored by this config's `ignorePatterns`.
    pub fn is_path_ignored(&self, path: &Path, is_dir: bool) -> bool {
        self.ignore_glob.as_ref().is_some_and(|glob| matches_with_ancestors(glob, path, is_dir))
    }

    /// Create a resolver by loading JSON/JSONC config from a file path.
    ///
    /// Also used as the default (empty config) fallback when no config file is found.
    #[instrument(level = "debug", name = "oxfmt::config::from_json_config", skip_all)]
    fn from_json_config(
        oxfmtrc_path: Option<&Path>,
        editorconfig: Option<Arc<EditorConfig>>,
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

        Ok(Self::new(raw_config, config_dir, editorconfig))
    }

    /// Validate config and build the ignore glob from `ignorePatterns` for file walking.
    ///
    /// Side effects:
    /// - `self.base` is set to the validated `FormatConfig` snapshot
    ///   (with `.editorconfig` `[*]` already folded in) paired with its gate artifacts
    /// - `self.oxfmtrc_overrides` is set if `overrides` exists
    /// - `self.ignore_glob` is built from `ignorePatterns`
    ///
    /// Validation runs eagerly via `validate()`,
    /// so invalid values are surfaced at config load time, rather than format time.
    ///
    /// # Errors
    /// Returns error if config deserialization or validation fails.
    #[instrument(level = "debug", name = "oxfmt::config::build_and_validate", skip_all)]
    pub fn build_and_validate(&mut self) -> Result<(), String> {
        let oxfmtrc = Oxfmtrc::deserialize(&self.raw_config).map_err(|err| err.to_string())?;

        // Resolve `overrides` from `Oxfmtrc` for later per-file matching
        let base_dir = self.config_dir.clone();
        self.oxfmtrc_overrides =
            oxfmtrc.overrides.map(|overrides| OxfmtrcOverrides::new(overrides, base_dir));

        let mut format_config = oxfmtrc.format_config;

        // Apply `.editorconfig` root section now.
        // Per-file sections are deferred to the slow path during `resolve_options()`.
        if let Some(editorconfig) = &self.editorconfig
            && let Some(props) = root_properties(editorconfig)
        {
            apply_editorconfig(&mut format_config, props);
        }

        if let Some(config_dir) = &self.config_dir {
            format_config.resolve_tailwind_paths(config_dir);
        }

        // Eagerly validate; see method doc for the rationale.
        // The snapshot and its gate artifacts are cached as one pair for the fast path.
        let validated = validate(&format_config)?;
        self.base = Some((Arc::new(format_config), Arc::new(validated)));

        // Build ignore glob from `ignorePatterns` config field
        let ignore_patterns = oxfmtrc.ignore_patterns.unwrap_or_default();
        self.ignore_glob = build_ignore_glob(self.config_dir.as_deref(), &ignore_patterns)?;

        Ok(())
    }

    /// Resolve options for a pre-classified file and build a [`ResolveOutcome`].
    ///
    /// Returns `Err` only when the merged config (after override application) fails validation.
    #[instrument(level = "debug", name = "oxfmt::config::resolve", skip_all, fields(path = %kind.path().display()))]
    pub fn resolve(&self, kind: FileKind) -> Result<ResolveOutcome, String> {
        let (format_config, validated) = self.resolve_options(kind.path())?;
        Ok(into_outcome(format_config, validated, kind))
    }

    /// Resolve `FormatConfig` for a specific file path.
    ///
    /// Priority (later wins):
    /// - `.editorconfig` (fallback for unset fields)
    /// - `.oxfmtrc` base
    /// - `.oxfmtrc` overrides matching the file path
    ///
    /// Fast path: reuses the snapshot + gate artifacts cached by [`Self::build_and_validate`].
    /// Slow path: always validates the merged config here
    ///   the single gate for every kind (the format step's option mapping is infallible;
    ///   for `Prettier` kinds this is also the only safety net before values reach Prettier).
    ///
    /// # Errors
    /// Returns `Err` when overrides introduce invalid values, including:
    /// - range-out values (e.g., `printWidth: 1000`)
    /// - broken compound-option combinations (e.g., `sortImports.groups` + `partitionByNewline`)
    fn resolve_options(
        &self,
        path: &Path,
    ) -> Result<(Arc<FormatConfig>, Arc<ValidatedOptions>), String> {
        let oxfmtrc_overrides =
            self.oxfmtrc_overrides.as_ref().map_or_else(Vec::new, |o| o.matching(path));
        // `.editorconfig` `[*]` is already folded in during `build_and_validate()`,
        // so only a per-file section that changes the result counts as an override.
        let editorconfig_overrides =
            self.editorconfig.as_ref().and_then(|ec| resolve_editorconfig_overrides(ec, path));

        // Fast path: no per-file overrides → share the cached (already-validated) snapshot.
        if oxfmtrc_overrides.is_empty() && editorconfig_overrides.is_none() {
            let (config, validated) =
                self.base.as_ref().expect("`build_and_validate()` must be called first");
            return Ok((Arc::clone(config), Arc::clone(validated)));
        }

        // Slow path: must rebuild from `raw_config`, NOT from the cached `base` snapshot.
        // See `raw_config` field doc for why the typed snapshot is insufficient here.
        // Deserializing from `&raw_config` avoids deep-cloning the JSON tree per file.
        let mut format_config = FormatConfig::deserialize(&self.raw_config)
            .expect("`build_and_validate()` should catch this before");

        // Apply oxfmtrc overrides first (explicit settings)
        for options in oxfmtrc_overrides {
            format_config.merge(options);
        }
        // Apply `.editorconfig` as fallback (fills in unset fields only).
        // The per-file resolution is `[*]` + `[src/*.ts]` merged with per-file values winning,
        // so per-file editorconfig fallback works even after overrides.
        // `None` means `[*]` alone is authoritative for the applied properties: no second resolve.
        if let Some(ec) = &self.editorconfig
            && let Some(props) = editorconfig_overrides.as_ref().or_else(|| root_properties(ec))
        {
            apply_editorconfig(&mut format_config, props);
        }

        if let Some(config_dir) = &self.config_dir {
            format_config.resolve_tailwind_paths(config_dir);
        }

        // Validate the merged config;
        // see method doc for what kinds of errors are caught and why this is the single gate.
        let validated = validate(&format_config)?;

        Ok((Arc::new(format_config), Arc::new(validated)))
    }
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
        oxc_config::validate_ignore_pattern(pattern)?;

        if builder.add_line(None, pattern).is_err() {
            return Err(format!("Failed to add ignore pattern `{pattern}` from `ignorePatterns`"));
        }
    }
    let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
    Ok(Some(gitignore))
}

// ---

#[cfg(test)]
mod tests_slow_path_validation {
    use std::{path::PathBuf, sync::Arc};

    use super::*;

    fn resolver_from_json(raw: serde_json::Value) -> ConfigResolver {
        let mut resolver = ConfigResolver::new(raw, None, None);
        resolver.build_and_validate().expect("base config must be valid for these tests");
        resolver
    }

    /// PR #21919 follow-up: invalid override values must be caught at resolve time
    /// (the format step is infallible, so `resolve_options`'s slow-path validation is the only gate).
    /// Without it, `printWidth: 1000` (above LineWidth::MAX = 320) would silently leak into the Prettier options.
    #[test]
    #[cfg(feature = "napi")]
    fn override_only_invalid_value_is_rejected_for_prettier() {
        let resolver = resolver_from_json(serde_json::json!({
            "printWidth": 80,
            "overrides": [
                { "files": ["*.json"], "options": { "printWidth": 1000 } }
            ]
        }));

        // Slow path triggers because the override matches.
        let kind = FileKind::Prettier {
            path: Arc::from(PathBuf::from("data.json").as_path()),
            parser_name: "json",
        };
        let err = resolver.resolve(kind).unwrap_err();
        assert!(err.contains("printWidth"), "expected printWidth validation error, got: {err}");
    }

    #[test]
    fn override_only_invalid_value_is_rejected_for_oxc_formatter() {
        let resolver = resolver_from_json(serde_json::json!({
            "tabWidth": 2,
            "overrides": [
                { "files": ["*.ts"], "options": { "tabWidth": 250 } }
            ]
        }));

        let kind = FileKind::OxcFormatter {
            path: Arc::from(PathBuf::from("src/test.ts").as_path()),
            source_type: oxc_span::SourceType::ts(),
        };
        let err = resolver.resolve(kind).unwrap_err();
        assert!(err.contains("tabWidth"), "expected tabWidth validation error, got: {err}");
    }

    /// Smoke test: when no overrides match, `resolve()` returns successfully from the fast path
    /// (shared pre-validated snapshot + gate artifacts, no re-validation anywhere downstream).
    #[test]
    fn fast_path_resolve_succeeds() {
        let resolver = resolver_from_json(serde_json::json!({ "printWidth": 80 }));
        let kind = FileKind::OxfmtToml { path: Arc::from(PathBuf::from("Cargo.toml").as_path()) };
        assert!(resolver.resolve(kind).is_ok());
    }

    /// `resolve_for_api` must validate even for `Prettier` kinds.
    /// Without the eager `validate()` call,
    /// `printWidth: 1000` would silently flow through to Prettier via the NAPI `format()` API.
    #[test]
    #[cfg(feature = "napi")]
    fn resolve_for_api_rejects_invalid_value_for_prettier() {
        let kind = FileKind::Prettier {
            path: Arc::from(PathBuf::from("page.vue").as_path()),
            parser_name: "vue",
        };
        let err = resolve_for_api(serde_json::json!({ "printWidth": 1000 }), kind, Path::new("."))
            .unwrap_err();
        assert!(err.contains("printWidth"), "expected printWidth validation error, got: {err}");
    }
}

#[cfg(test)]
mod tests_ignore_patterns_validation {
    use std::path::Path;

    use super::build_ignore_glob;

    fn build(pattern: &str) -> Result<(), String> {
        build_ignore_glob(Some(Path::new("/repo/config")), &[pattern.to_string()]).map(|_| ())
    }

    // Pattern-level cases are covered by `oxc_config::validate_ignore_pattern` tests;
    // these only check that `build_ignore_glob` rejects a config containing one.
    #[test]
    fn rejects_parent_directory_components() {
        let error = build("../src/skip.js").unwrap_err();
        assert_eq!(
            error,
            "Invalid pattern `../src/skip.js` in `ignorePatterns`: `..` is not supported, patterns are resolved within the config file's directory"
        );
    }

    #[test]
    fn accepts_patterns_without_parent_directory_components() {
        assert!(build("src/skip.js").is_ok());
    }
}
