use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock, RwLock},
};

use editorconfig_parser::EditorConfig;
use rustc_hash::FxHashMap;

use oxc_config::ConfigDiscovery;
use oxc_diagnostics::OxcDiagnostic;

#[cfg(feature = "napi")]
use super::js_config::JsConfigLoaderCb;
use super::{
    ConfigResolver, build_resolver_from_discovered, config_discovery,
    editorconfig::load_editorconfig,
};

/// Result of loading a direct config in a single directory.
type ConfigLoadResult = Result<Option<Arc<ConfigResolver>>, String>;

/// Walk-wide shared cache for direct-config loads.
///
/// Each entry's `OnceLock` ensures the underlying load runs at most once per
/// directory across all visitors and across phases.
type ConfigLoadCache = Arc<Mutex<FxHashMap<PathBuf, Arc<OnceLock<ConfigLoadResult>>>>>;

/// Walk-wide shared map of "directory has a direct config" entries.
///
/// Lock discipline: never hold this lock across a `ConfigLoadCache` load.
/// Acquire the read/write lock, do the lookup or insert, release immediately.
type ScopeByDir = Arc<RwLock<FxHashMap<PathBuf, Arc<ConfigResolver>>>>;

/// Walk-wide shared cache for the parsed `.editorconfig`.
///
/// Loaded on first access (via `OnceLock`) and cloned per nested-config load
/// instead of re-reading and re-parsing the same file for every probed dir.
/// `Err` is cached too, so a malformed `.editorconfig` is not retried.
type EditorconfigCache = Arc<OnceLock<Result<Option<EditorConfig>, String>>>;

/// Shared on-demand nested-config detection infrastructure.
///
/// State is centralized to share caches and signals across all visitors and all phases:
/// - Phase 2 (direct file targets)
/// - Phase 3 (parallel walk, visitors)
/// - and the stdin path
///
/// Cloning is shallow (each field is already `Arc` / `Copy`).
#[derive(Clone)]
pub struct NestedConfigCtx {
    discovery: ConfigDiscovery,
    editorconfig_path: Option<Arc<Path>>,
    editorconfig_cache: EditorconfigCache,
    #[cfg(feature = "napi")]
    js_config_loader: Option<JsConfigLoaderCb>,
    scope_by_dir: ScopeByDir,
    config_load_cache: ConfigLoadCache,
}

impl NestedConfigCtx {
    pub fn new(
        editorconfig_path: Option<Arc<Path>>,
        #[cfg(feature = "napi")] js_config_loader: Option<JsConfigLoaderCb>,
    ) -> Self {
        Self {
            discovery: config_discovery(),
            editorconfig_path,
            editorconfig_cache: Arc::new(OnceLock::new()),
            #[cfg(feature = "napi")]
            js_config_loader,
            scope_by_dir: Arc::new(RwLock::new(FxHashMap::default())),
            config_load_cache: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// Get the parsed `.editorconfig`, loading once and reusing the cached
    /// `EditorConfig` (or cached `Err`) for every subsequent caller.
    fn cached_editorconfig(&self) -> Result<Option<EditorConfig>, String> {
        self.editorconfig_cache
            .get_or_init(|| load_editorconfig(self.editorconfig_path.as_deref()))
            .clone()
    }

    /// Returns `true` if `path`'s file name matches a supported config file.
    pub fn is_config_file(&self, path: &Path) -> bool {
        self.discovery.discover_config_file(path).is_some()
    }

    /// Look up a registered scope for `dir` without probing.
    pub fn lookup_scope(&self, dir: &Path) -> Option<Arc<ConfigResolver>> {
        self.scope_by_dir.read().expect("scope_by_dir rwlock poisoned").get(dir).cloned()
    }

    /// Whether any nested config has been registered walk-wide.
    pub fn config_found(&self) -> bool {
        !self.scope_by_dir.read().expect("scope_by_dir rwlock poisoned").is_empty()
    }

    /// Read `scope_by_dir` for `dir`; on miss, probe via the load cache and register the result.
    ///
    /// Returns:
    /// - `Ok(Some(_))` — `dir` has a direct config (registered)
    /// - `Ok(None)` — no direct config in `dir`
    /// - `Err(_)` — load / parse / validate failure
    ///
    /// `OnceLock::get_or_init` blocks concurrent callers for the same `dir` until the first init completes.
    /// `Ok(Some(_))` / `Ok(None)` / `Err(_)` are all cached,
    /// so broken configs are not retried and "no config in this dir" lookups stay O(1).
    pub fn probe_dir(&self, dir: &Path) -> Result<Option<Arc<ConfigResolver>>, String> {
        if let Some(hit) = self.lookup_scope(dir) {
            return Ok(Some(hit));
        }

        // Acquire (or insert) the cell, then drop the outer mutex immediately.
        let cell = {
            let mut guard =
                self.config_load_cache.lock().expect("config_load_cache mutex poisoned");
            let entry = guard.entry(dir.to_path_buf()).or_insert_with(|| Arc::new(OnceLock::new()));
            Arc::clone(entry)
        };
        let load_result = cell
            .get_or_init(|| {
                let editorconfig = self.cached_editorconfig()?;
                self.load_direct_in_dir(dir, editorconfig)
            })
            .clone();

        match load_result? {
            Some(loaded) => {
                let mut guard = self.scope_by_dir.write().expect("scope_by_dir rwlock poisoned");
                guard.entry(dir.to_path_buf()).or_insert_with(|| Arc::clone(&loaded));
                Ok(Some(loaded))
            }
            None => Ok(None),
        }
    }

    /// Load and validate a config file located directly inside `dir`.
    fn load_direct_in_dir(
        &self,
        dir: &Path,
        editorconfig: Option<EditorConfig>,
    ) -> ConfigLoadResult {
        let Some(config_file) = self
            .discovery
            .find_unique_config_by_readdir(dir, false)
            .map_err(|e| Into::<OxcDiagnostic>::into(e).to_string())?
        else {
            return Ok(None);
        };

        let load_err = |err: String| format!("Failed to load config in {}: {err}", dir.display());

        let Some(mut resolver) = build_resolver_from_discovered(
            config_file,
            editorconfig,
            #[cfg(feature = "napi")]
            self.js_config_loader.as_ref(),
        )
        .map_err(load_err)?
        else {
            return Ok(None);
        };

        resolver.build_and_validate().map_err(load_err)?;
        Ok(Some(Arc::new(resolver)))
    }
}
