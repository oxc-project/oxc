mod editorconfig;
#[cfg(feature = "napi")]
mod js_config;
mod nested;
mod overrides;

pub use editorconfig::resolve_editorconfig_path;
#[cfg(feature = "napi")]
pub use js_config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader};
pub use nested::NestedConfigCtx;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use editorconfig_parser::EditorConfig;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde_json::Value;
use tracing::instrument;

use oxc_config::{ConfigDiscovery, ConfigFileNames, DiscoveredConfigFile, is_js_config_path};
#[cfg(feature = "napi")]
use oxc_formatter::FormatOptions;

use self::{
    editorconfig::{apply_editorconfig, has_editorconfig_overrides, load_editorconfig},
    overrides::OxfmtrcOverrides,
};
use super::{
    FormatStrategy,
    options::to_oxc_formatter,
    oxfmtrc::{FormatConfig, Oxfmtrc},
    support::FileKind,
    utils,
};

const OXFMT_CONFIG_FILE_NAMES: ConfigFileNames = ConfigFileNames {
    json: ".oxfmtrc.json",
    jsonc: ".oxfmtrc.jsonc",
    js: "oxfmt.config.ts",
    vite: "vite.config.ts",
};

pub fn config_discovery() -> ConfigDiscovery {
    ConfigDiscovery::new(
        OXFMT_CONFIG_FILE_NAMES,
        cfg!(feature = "napi") && utils::vp_version().is_some(),
    )
}

/// Build a `ConfigResolver` from a single discovered config file (no ancestor walk,
/// no `build_and_validate`).
///
/// NOTE: Returns `Ok(None)` when the discovered file is a `vite.config.ts` whose
/// default export lacks a `.fmt` field.
/// Callers decide how to handle it:
/// - [`ConfigResolver::from_config`] (explicit `--config`): treat as an error
/// - [`ConfigResolver::discover_config`] (ancestor walk): skip and continue upward
/// - [`NestedConfigCtx::load_direct_in_dir`] (nested probe): no config in this dir
pub fn build_resolver_from_discovered(
    config_file: DiscoveredConfigFile,
    editorconfig: Option<EditorConfig>,
    #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
) -> Result<Option<ConfigResolver>, String> {
    match config_file {
        DiscoveredConfigFile::Json(path) | DiscoveredConfigFile::Jsonc(path) => {
            ConfigResolver::from_json_config(Some(&path), editorconfig).map(Some)
        }
        #[cfg(not(feature = "napi"))]
        DiscoveredConfigFile::Js(path) | DiscoveredConfigFile::Vite(path) => Err(format!(
            "JS/TS config file ({}) is not supported in pure Rust CLI.\nUse JSON/JSONC instead.",
            path.display()
        )),
        #[cfg(feature = "napi")]
        DiscoveredConfigFile::Js(path) => {
            // Non-Vite JS config: `loadJsConfig` never returns `null`; failures bubble up as `Err`.
            let raw_config = load_js_config(
                js_config_loader
                    .expect("JS config loader must be set when `napi` feature is enabled"),
                &path,
            )?
            .expect("loadJsConfig never returns null for non-Vite JS config");
            Ok(Some(ConfigResolver::new(
                raw_config,
                path.parent().map(Path::to_path_buf),
                editorconfig,
            )))
        }
        #[cfg(feature = "napi")]
        DiscoveredConfigFile::Vite(path) => {
            let Some(raw_config) = load_js_config(
                js_config_loader
                    .expect("JS config loader must be set when `napi` feature is enabled"),
                &path,
            )?
            else {
                return Ok(None);
            };
            Ok(Some(ConfigResolver::new(
                raw_config,
                path.parent().map(Path::to_path_buf),
                editorconfig,
            )))
        }
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

/// Resolve options for a pre-classified file and build a [`ResolveOutcome`].
///
/// This is the simplified path for the NAPI `format()` API,
/// which doesn't need `.oxfmtrc` overrides, `.editorconfig`, or ignore patterns.
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
    // Validate eagerly: `from_format_config` skips validation for `ExternalFormatter*` kinds,
    // so range-out values (e.g., `printWidth: 1000`) would otherwise silently reach Prettier.
    let _ = to_oxc_formatter(&format_config)?;
    if let Some(plugin) = kind.requires_plugin(&format_config) {
        return Ok(ResolveOutcome::MissingPlugin(plugin));
    }
    FormatStrategy::from_format_config(format_config, kind).map(ResolveOutcome::Format)
}

/// Resolved options ready for the embedded callback to drive `oxc_formatter`.
#[cfg(feature = "napi")]
#[derive(Debug)]
pub struct EmbeddedCallbackResolved {
    pub format_options: Box<FormatOptions>,
    /// Retained so nested embedded callbacks can derive Prettier options on demand.
    /// (e.g., CSS-in-JS inside the embedded JS)
    pub config: Box<FormatConfig>,
    pub parent_filepath: PathBuf,
}

/// Resolve options for an embedded JS/TS fragment.
///
/// Called from [`crate::api::text_to_doc_api`] when Prettier invokes the
/// `prettier-plugin-oxfmt` callback with the typed config + parent filepath
/// recovered from `_oxfmtPluginOptionsJson`.
///
/// Returns the materialized pieces directly rather than a [`FormatStrategy`]
/// because the callback drives `oxc_formatter` itself, not via `SourceFormatter::format()`.
///
/// Tailwind paths in `config` are already absolute (resolved by the host before serialization),
/// so no `cwd` is threaded through here.
#[cfg(feature = "napi")]
pub fn resolve_for_embedded_js(
    config: FormatConfig,
    parent_filepath: PathBuf,
) -> Result<EmbeddedCallbackResolved, String> {
    let format_options = Box::new(to_oxc_formatter(&config)?);
    Ok(EmbeddedCallbackResolved { format_options, config: Box::new(config), parent_filepath })
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
    /// Retained because the slow path must re-deserialize [`FormatConfig`] from it.
    /// (see [`Self::resolve_options`]).
    /// Cloning a typed `base_config` is not enough, since `apply_editorconfig` only fills `is_none()` fields,
    /// so per-file `[src/*.ts]` sections couldn't override values that the `[*]` section already baked in.
    raw_config: Value,
    /// Directory containing the config file (for relative path resolution in overrides).
    config_dir: Option<PathBuf>,
    /// Cached typed `FormatConfig` after `.oxfmtrc` base + `.editorconfig` `[*]` section is folded in.
    /// Used as the fast-path snapshot when no per-file overrides apply.
    base_config: Option<FormatConfig>,
    /// Resolved overrides from `.oxfmtrc` for file-specific matching.
    oxfmtrc_overrides: Option<OxfmtrcOverrides>,
    /// Ignore glob built from this config's `ignorePatterns`.
    ignore_glob: Option<Gitignore>,
    /// Parsed `.editorconfig`, if any.
    editorconfig: Option<EditorConfig>,
}

impl ConfigResolver {
    /// Shared internal constructor used by both:
    /// - `from_json_config()` (JSON/JSONC)
    /// - and `from_config()` (JS/TS config evaluated externally)
    fn new(
        raw_config: Value,
        config_dir: Option<PathBuf>,
        editorconfig: Option<EditorConfig>,
    ) -> Self {
        Self {
            raw_config,
            config_dir,
            base_config: None,
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
        // Always load the nearest `.editorconfig` if exists
        let editorconfig = load_editorconfig(editorconfig_path)?;

        // Explicit path: normalize and load directly
        if let Some(config_path) = oxfmtrc_path {
            let path = utils::normalize_relative_path(cwd, config_path);

            if is_js_config_path(&path) {
                #[cfg(not(feature = "napi"))]
                {
                    return Err(format!(
                        "JS/TS config file ({}) is not supported in pure Rust CLI.\nUse JSON/JSONC instead.",
                        path.display()
                    ));
                }
                #[cfg(feature = "napi")]
                {
                    let raw_config = load_js_config(
                        js_config_loader
                            .expect("JS config loader must be set when `napi` feature is enabled"),
                        &path,
                    )?
                    // Explicit `--config`: missing `.fmt` is an error.
                    .ok_or_else(|| {
                        format!(
                            "Expected a `fmt` field in the default export of {}",
                            path.display()
                        )
                    })?;

                    return Ok(Self::new(
                        raw_config,
                        path.parent().map(Path::to_path_buf),
                        editorconfig,
                    ));
                }
            }

            return Self::from_json_config(Some(&path), editorconfig);
        }

        // Auto-discovery: search upwards from cwd, load in one pass
        Self::discover_config(
            cwd,
            editorconfig,
            #[cfg(feature = "napi")]
            js_config_loader,
        )
    }

    /// Auto-discover and load config by searching upwards from `cwd`.
    fn discover_config(
        cwd: &Path,
        editorconfig: Option<EditorConfig>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Result<Self, String> {
        let discovery = config_discovery();
        for dir in cwd.ancestors() {
            let Some(config_file) = discovery
                .find_unique_config_by_readdir(dir, false)
                .map_err(|e| Into::<oxc_diagnostics::OxcDiagnostic>::into(e).to_string())?
            else {
                continue;
            };

            // `Ok(None)` (Vite+ `.fmt` missing) → keep searching upwards.
            if let Some(resolver) = build_resolver_from_discovered(
                config_file,
                editorconfig.clone(),
                #[cfg(feature = "napi")]
                js_config_loader,
            )? {
                return Ok(resolver);
            }
        }

        // No config found, use defaults
        Self::from_json_config(None, editorconfig)
    }

    /// Create a resolver by loading JSON/JSONC config from a file path.
    ///
    /// Also used as the default (empty config) fallback when no config file is found.
    #[instrument(level = "debug", name = "oxfmt::config::from_json_config", skip_all)]
    pub(crate) fn from_json_config(
        oxfmtrc_path: Option<&Path>,
        editorconfig: Option<EditorConfig>,
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
    /// - `self.base_config` is set to the validated `FormatConfig` snapshot
    ///   (with `.editorconfig` `[*]` already folded in)
    /// - `self.oxfmtrc_overrides` is set if `overrides` exists
    /// - `self.ignore_glob` is built from `ignorePatterns`
    ///
    /// Validation runs eagerly via `to_oxc_formatter(&base_config)` so invalid
    /// values (e.g., `printWidth: 99999`) are surfaced at config load time
    /// regardless of which file kind is later processed.
    ///
    /// # Errors
    /// Returns error if config deserialization or validation fails.
    #[instrument(level = "debug", name = "oxfmt::config::build_and_validate", skip_all)]
    pub fn build_and_validate(&mut self) -> Result<(), String> {
        let oxfmtrc: Oxfmtrc =
            serde_json::from_value(self.raw_config.clone()).map_err(|err| err.to_string())?;

        // Resolve `overrides` from `Oxfmtrc` for later per-file matching
        let base_dir = self.config_dir.clone();
        self.oxfmtrc_overrides =
            oxfmtrc.overrides.map(|overrides| OxfmtrcOverrides::new(overrides, base_dir));

        let mut format_config = oxfmtrc.format_config;

        // Apply `.editorconfig` root section now. Per-file `[src/*.ts]` sections
        // are deferred to the slow path during `resolve_options()`.
        if let Some(editorconfig) = &self.editorconfig
            && let Some(props) =
                editorconfig.sections().iter().find(|s| s.name == "*").map(|s| &s.properties)
        {
            apply_editorconfig(&mut format_config, props);
        }

        if let Some(config_dir) = &self.config_dir {
            format_config.resolve_tailwind_paths(config_dir);
        }

        // Eagerly validate; see method doc for the rationale.
        let _ = to_oxc_formatter(&format_config)?;

        // Save cached snapshot for fast path: no per-file overrides
        self.base_config = Some(format_config);

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
        let format_config = self.resolve_options(kind.path())?;
        #[cfg(feature = "napi")]
        if let Some(plugin) = kind.requires_plugin(&format_config) {
            return Ok(ResolveOutcome::MissingPlugin(plugin));
        }
        FormatStrategy::from_format_config(format_config, kind).map(ResolveOutcome::Format)
    }

    /// Resolve `FormatConfig` for a specific file path.
    ///
    /// Priority (later wins):
    /// - `.editorconfig` (fallback for unset fields)
    /// - `.oxfmtrc` base
    /// - `.oxfmtrc` overrides matching the file path
    ///
    /// Fast path: Skips validation within this method because `base_config` is pre-validated in [`Self::build_and_validate`].
    /// Slow path: Always validates the merged config here.
    /// - For `OxcFormatter` / `OxfmtToml` kinds, [`FormatStrategy::from_format_config`] also re-validates via typed conversion (redundant but harmless).
    /// - For `ExternalFormatter*` kinds, this is the only safety net before values reach Prettier.
    ///
    /// # Errors
    /// Returns `Err` when overrides introduce invalid values, including:
    /// - range-out values (e.g., `printWidth: 1000`)
    /// - broken compound-option combinations (e.g., `sortImports.groups` + `partitionByNewline`)
    fn resolve_options(&self, path: &Path) -> Result<FormatConfig, String> {
        let has_editorconfig_overrides =
            self.editorconfig.as_ref().is_some_and(|ec| has_editorconfig_overrides(ec, path));
        let has_oxfmtrc_overrides =
            self.oxfmtrc_overrides.as_ref().is_some_and(|o| o.has_match(path));

        // Fast path: no per-file overrides → reuse the cached (already-validated) snapshot.
        // `.editorconfig` `[*]` is already folded in during `build_and_validate()`.
        if !has_editorconfig_overrides && !has_oxfmtrc_overrides {
            return Ok(self
                .base_config
                .clone()
                .expect("`build_and_validate()` must be called first"));
        }

        // Slow path: must rebuild from `raw_config`, NOT from `base_config`.
        // See `raw_config` field doc for why cloning the typed snapshot is insufficient.
        let mut format_config: FormatConfig = serde_json::from_value(self.raw_config.clone())
            .expect("`build_and_validate()` should catch this before");

        // Apply oxfmtrc overrides first (explicit settings)
        if let Some(overrides) = &self.oxfmtrc_overrides {
            for options in overrides.get_matching(path) {
                format_config.merge(options);
            }
        }
        // Apply `.editorconfig` as fallback (fills in unset fields only).
        // `EditorConfig::resolve` returns `[*]` + `[src/*.ts]` merged, with per-file
        // values winning, so per-file editorconfig fallback works even after overrides.
        if let Some(ec) = &self.editorconfig {
            let props = ec.resolve(path);
            apply_editorconfig(&mut format_config, &props);
        }

        if let Some(config_dir) = &self.config_dir {
            format_config.resolve_tailwind_paths(config_dir);
        }

        // Validate the merged config; see method doc for what kinds of errors are caught
        // and why this is the only safety net for `ExternalFormatter*` kinds.
        let _ = to_oxc_formatter(&format_config)?;

        Ok(format_config)
    }
}

/// Resolve the nearest config scope for a file, or fall back to the root resolver.
///
/// `ctx` is `None` when the caller wants to bypass nested-config detection.
/// In that case the root resolver is returned unconditionally.
///
/// When `ctx` is `Some`, the ancestor chain of `file` is walked,
/// short-circuiting on `root_config_resolver.config_dir()` to avoid re-loading the root via `ctx`.
/// (which would create a duplicate `Arc` and, with `napi`, re-invoke the JS config loader)
pub fn resolve_file_scope_config(
    file: &Path,
    root_config_resolver: &Arc<ConfigResolver>,
    ctx: Option<&NestedConfigCtx>,
) -> Result<Arc<ConfigResolver>, String> {
    let Some(ctx) = ctx else {
        return Ok(Arc::clone(root_config_resolver));
    };
    let Some(parent) = file.parent() else {
        return Ok(Arc::clone(root_config_resolver));
    };

    let root_config_dir = root_config_resolver.config_dir();
    for dir in parent.ancestors() {
        if Some(dir) == root_config_dir {
            return Ok(Arc::clone(root_config_resolver));
        }
        if let Some(r) = ctx.probe_dir(dir)? {
            return Ok(r);
        }
    }

    Ok(Arc::clone(root_config_resolver))
}

/// Load a JS/TS config file via NAPI and return the raw JSON value.
///
/// Returns `Ok(None)` when the JS side returns `null` (Vite+ `.fmt` missing).
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
    /// even for `ExternalFormatter*` kinds (which don't re-validate inside
    /// `from_format_config`). Without slow-path validation in `resolve_options`,
    /// `printWidth: 1000` (above LineWidth::MAX = 320) would silently leak into
    /// the Prettier options.
    #[test]
    #[cfg(feature = "napi")]
    fn override_only_invalid_value_is_rejected_for_external_formatter() {
        let resolver = resolver_from_json(serde_json::json!({
            "printWidth": 80,
            "overrides": [
                { "files": ["*.json"], "options": { "printWidth": 1000 } }
            ]
        }));

        // Slow path triggers because the override matches.
        let kind = FileKind::ExternalFormatter {
            path: Arc::from(PathBuf::from("data.json").as_path()),
            parser_name: "json",
            supports_tailwind: false,
            supports_oxfmt: false,
            supports_svelte: false,
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

    /// Smoke test: when no overrides match, `resolve()` returns successfully.
    ///
    /// `resolve_options` itself skips re-validation on the fast path
    /// (just clones the pre-validated `base_config`), but
    /// `FormatStrategy::from_format_config` still calls `to_oxc_formatter` for
    /// `OxcFormatter`/`OxfmtToml`, so this test cannot directly assert "no re-validation"
    /// — only that the overall call succeeds.
    #[test]
    fn fast_path_resolve_succeeds() {
        let resolver = resolver_from_json(serde_json::json!({ "printWidth": 80 }));

        let kind = FileKind::OxfmtToml { path: Arc::from(PathBuf::from("Cargo.toml").as_path()) };
        assert!(resolver.resolve(kind).is_ok());
    }

    /// `resolve_for_api` must validate even for `ExternalFormatter*` kinds.
    /// Without the eager `to_oxc_formatter` call, `printWidth: 1000` would
    /// silently flow through to Prettier via the NAPI `format()` API.
    #[test]
    #[cfg(feature = "napi")]
    fn resolve_for_api_rejects_invalid_value_for_external_formatter() {
        let kind = FileKind::ExternalFormatter {
            path: Arc::from(PathBuf::from("style.css").as_path()),
            parser_name: "css",
            supports_tailwind: true,
            supports_oxfmt: false,
            supports_svelte: false,
        };
        let err = resolve_for_api(serde_json::json!({ "printWidth": 1000 }), kind, Path::new("."))
            .unwrap_err();
        assert!(err.contains("printWidth"), "expected printWidth validation error, got: {err}");
    }
}
