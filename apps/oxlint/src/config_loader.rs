use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use ignore::DirEntry;

use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{
    Config, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore, LintFilter, Oxlintrc,
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::{
    DEFAULT_JSONC_OXLINTRC_NAME, DEFAULT_OXLINTRC_NAME, DEFAULT_TS_OXLINTRC_NAME,
    config_discovery::{ConfigDiscovery, ConfigFileNames},
};

use crate::config_discovery::{ConfigConflict, DiscoveredConfigFile};

use crate::{VITE_CONFIG_NAME, vp_version};

const GIT_DIR: &str = ".git";
const NODE_MODULES_DIR: &str = "node_modules";

#[cfg(feature = "napi")]
use crate::js_config;
use crate::js_config::JsConfigResult;

const OXLINT_CONFIG_FILE_NAMES: ConfigFileNames = ConfigFileNames {
    json: DEFAULT_OXLINTRC_NAME,
    jsonc: DEFAULT_JSONC_OXLINTRC_NAME,
    js: DEFAULT_TS_OXLINTRC_NAME,
    vite: VITE_CONFIG_NAME,
};

/// Discover config files by walking UP from each file's directory to ancestors.
///
/// Used by CLI where we have specific files to lint and need to find configs
/// that apply to them.
///
/// Example: For files `/project/src/foo.js` and `/project/src/bar/baz.js`:
/// - Checks `/project/src/bar/`, `/project/src/`, `/project/`, `/`
/// - Returns paths to matching config files found
///
/// In Vite+ mode, only `vite.config.ts` is discovered.
pub fn discover_configs_in_ancestors<P: AsRef<Path>>(
    files: &[P],
    base_config_path: &Path,
) -> impl IntoIterator<Item = DiscoveredConfigFile> {
    let mut config_paths = FxHashSet::<DiscoveredConfigFile>::default();
    let mut visited_dirs = FxHashSet::default();

    for file in files {
        let path = file.as_ref();
        let mut base_config_found = false;
        // Start from the file's parent directory and walk up the tree
        let mut current = path.parent();
        while let Some(dir) = current {
            if base_config_found {
                // Stop if we've reached the base config file (e.g., root oxlintrc)
                // to avoid duplicate loading and filling nested config with configs outside from the root config.
                break;
            }
            // Stop if we've already checked this directory (and its ancestors)
            let inserted = visited_dirs.insert(dir.to_path_buf());
            if !inserted {
                break;
            }
            for config in find_configs_in_directory(dir) {
                if config.path() == base_config_path {
                    base_config_found = true;
                    break;
                }
                config_paths.insert(config);
            }
            current = dir.parent();
        }
    }

    config_paths
}

/// Discover config files by walking DOWN from a root directory.
/// Will skip the base config file (e.g., root oxlintrc) to avoid duplicate loading.
/// In Vite+ mode, only `vite.config.ts` is discovered.
///
/// Used by LSP where we have a workspace root and need to discover all configs
/// upfront for file watching and diagnostics.
pub fn discover_configs_in_tree(
    root: &Path,
    base_config_path: &Path,
) -> impl IntoIterator<Item = DiscoveredConfigFile> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false) // don't skip hidden files
        .parents(false) // disable gitignore from parent dirs
        .ignore(false) // disable .ignore files
        .git_global(false) // disable global gitignore
        .follow_links(true)
        .build_parallel();

    let (sender, receiver) = mpsc::channel::<Vec<DiscoveredConfigFile>>();
    let mut builder =
        ConfigWalkBuilder { sender, base_config_path: base_config_path.to_path_buf() };
    walker.visit(&mut builder);
    drop(builder);

    receiver.into_iter().flatten()
}

/// Check if a directory contains an oxlint config file.
fn find_configs_in_directory(dir: &Path) -> Vec<DiscoveredConfigFile> {
    ConfigDiscovery::new(OXLINT_CONFIG_FILE_NAMES, vp_version().is_some())
        .find_configs_in_directory(dir)
}

// Helper types for parallel directory walking
struct ConfigWalkBuilder {
    sender: mpsc::Sender<Vec<DiscoveredConfigFile>>,
    base_config_path: PathBuf,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for ConfigWalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(ConfigWalkCollector {
            configs: vec![],
            sender: self.sender.clone(),
            base_config_path: self.base_config_path.clone(),
        })
    }
}

struct ConfigWalkCollector {
    configs: Vec<DiscoveredConfigFile>,
    sender: mpsc::Sender<Vec<DiscoveredConfigFile>>,
    base_config_path: PathBuf,
}

impl Drop for ConfigWalkCollector {
    fn drop(&mut self) {
        let configs = std::mem::take(&mut self.configs);
        self.sender.send(configs).unwrap();
    }
}

impl ignore::ParallelVisitor for ConfigWalkCollector {
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                // Skip `.git` and `node_modules` directories entirely - they are not part of the
                // lintable project tree for config discovery.
                if entry.file_type().is_some_and(|ft| ft.is_dir())
                    && (entry.file_name() == OsStr::new(GIT_DIR)
                        || entry.file_name() == OsStr::new(NODE_MODULES_DIR))
                {
                    return ignore::WalkState::Skip;
                }
                if let Some(config) = to_discovered_config(&entry, &self.base_config_path) {
                    self.configs.push(config);
                }
                ignore::WalkState::Continue
            }
            Err(_) => ignore::WalkState::Skip,
        }
    }
}

fn to_discovered_config(entry: &DirEntry, base_config_path: &Path) -> Option<DiscoveredConfigFile> {
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        return None;
    }
    if entry.path() == base_config_path {
        // Skip the base config file (e.g., root oxlintrc) to avoid duplicate loading
        return None;
    }
    ConfigDiscovery::new(OXLINT_CONFIG_FILE_NAMES, vp_version().is_some())
        .discover_config_file(entry.path())
}

pub struct LoadedConfig {
    /// The directory this config applies to
    pub dir: PathBuf,
    /// The built configuration
    pub config: Config,
    /// Ignore patterns from this config
    pub ignore_patterns: Vec<String>,
    /// Paths from extends directives
    pub extended_paths: Vec<PathBuf>,
}

/// Errors that can occur when loading configs
#[derive(Debug)]
pub enum ConfigLoadError {
    /// Failed to parse the config file
    Parse {
        path: PathBuf,
        error: OxcDiagnostic,
    },
    /// Failed to build the ConfigStore
    Build {
        path: PathBuf,
        error: String,
    },

    JsConfigFileFoundButJsRuntimeNotAvailable,

    Diagnostic(OxcDiagnostic),
}

impl ConfigLoadError {
    /// Get the path of the config file that failed
    pub fn path(&self) -> Option<&Path> {
        match self {
            ConfigLoadError::Parse { path, .. } | ConfigLoadError::Build { path, .. } => Some(path),
            _ => None,
        }
    }
}

/// High-level errors that can occur when loading CLI configurations.
///
/// This groups together failures related to the root configuration file
/// and to any nested configuration files discovered during loading.
pub enum CliConfigLoadError {
    /// An error that occurred while loading or parsing the root configuration.
    RootConfig(OxcDiagnostic),
    /// One or more errors that occurred while loading nested configuration files.
    NestedConfigs(Vec<ConfigLoadError>),
}

/// Collection of the root configuration and all successfully loaded nested configs.
///
/// Returned by [`ConfigLoader::load_root_and_nested`].
pub struct LoadedConfigs {
    /// The root `oxlintrc` configuration used as the base for all linting.
    pub root: Oxlintrc,
    /// Mapping from directory paths to the effective [`Config`] for that directory.
    pub nested: FxHashMap<PathBuf, Config>,
    /// Ignore patterns from nested configs, paired with the directory they apply to.
    pub nested_ignore_patterns: Vec<(Vec<String>, PathBuf)>,
}

pub struct ConfigLoader<'a> {
    external_linter: Option<&'a ExternalLinter>,
    external_plugin_store: &'a mut ExternalPluginStore,
    filters: &'a [LintFilter],
    workspace_uri: Option<&'a str>,
    #[cfg(feature = "napi")]
    #[expect(clippy::struct_field_names)]
    js_config_loader: Option<&'a js_config::JsConfigLoaderCb>,
}

impl<'a> ConfigLoader<'a> {
    /// Create a new ConfigLoader
    ///
    /// # Arguments
    /// * `external_linter` - Optional external linter for plugin support
    /// * `external_plugin_store` - Store for external plugins
    /// * `filters` - Lint filters to apply to configs
    /// * `workspace_uri` - Workspace URI  - only `Some` in LSP, `None` in CLI
    pub fn new(
        external_linter: Option<&'a ExternalLinter>,
        external_plugin_store: &'a mut ExternalPluginStore,
        filters: &'a [LintFilter],
        workspace_uri: Option<&'a str>,
    ) -> Self {
        Self {
            external_linter,
            external_plugin_store,
            filters,
            workspace_uri,
            #[cfg(feature = "napi")]
            js_config_loader: None,
        }
    }

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_js_config_loader(
        mut self,
        js_config_loader: Option<&'a js_config::JsConfigLoaderCb>,
    ) -> Self {
        if let Some(js_loader) = js_config_loader {
            self.js_config_loader = Some(js_loader);
        }

        self
    }

    /// Load a single config from a file path
    fn load(path: &Path) -> Result<Oxlintrc, ConfigLoadError> {
        Oxlintrc::from_file(path)
            .map_err(|error| ConfigLoadError::Parse { path: path.to_path_buf(), error })
    }

    pub fn load_js_configs(
        &self,
        paths: &[PathBuf],
    ) -> Result<Vec<JsConfigResult>, Vec<ConfigLoadError>> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }

        #[cfg(not(feature = "napi"))]
        {
            return Err(vec![ConfigLoadError::JsConfigFileFoundButJsRuntimeNotAvailable]);
        }

        #[cfg(feature = "napi")]
        let Some(js_config_loader) = self.js_config_loader else {
            return Err(vec![ConfigLoadError::JsConfigFileFoundButJsRuntimeNotAvailable]);
        };

        let paths_as_strings: Vec<String> =
            paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

        match js_config_loader(paths_as_strings) {
            Ok(results) => Ok(results),
            Err(diagnostics) => {
                Err(diagnostics.into_iter().map(ConfigLoadError::Diagnostic).collect())
            }
        }
    }

    /// Load multiple configs, returning successes and errors separately
    ///
    /// This allows callers to decide how to handle errors (fail fast vs continue)
    fn load_many(
        &mut self,
        paths: impl IntoIterator<Item = DiscoveredConfigFile>,
        root_config_dir: Option<&Path>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>) {
        let mut configs = Vec::new();
        let mut errors = Vec::new();

        let mut by_dir = FxHashMap::<PathBuf, Vec<DiscoveredConfigFile>>::default();

        for config in paths {
            let Some(dir) = config.path().parent().map(Path::to_path_buf) else {
                continue;
            };

            by_dir.entry(dir).or_default().push(config);
        }

        let mut js_configs = Vec::new();

        for (dir, config_files) in by_dir {
            if config_files.len() > 1 {
                errors.push(ConfigLoadError::Diagnostic(
                    ConfigConflict::new(dir.clone(), config_files).into(),
                ));
                continue;
            }

            match config_files.into_iter().next() {
                Some(DiscoveredConfigFile::Json(path) | DiscoveredConfigFile::Jsonc(path)) => {
                    match Self::load(path.as_path()) {
                        Ok(config) => configs.push(config),
                        Err(e) => errors.push(e),
                    }
                }
                Some(DiscoveredConfigFile::Js(path) | DiscoveredConfigFile::Vite(path)) => {
                    js_configs.push(path);
                }
                None => {
                    debug_assert!(
                        false,
                        "Expected at least one config file for directory {}",
                        dir.display()
                    );
                }
            }
        }

        match self.load_js_configs(&js_configs) {
            Ok(loaded_js_configs) => {
                configs.extend(loaded_js_configs.into_iter().filter_map(|c| c.config));
            }
            Err(mut js_errors) => {
                errors.append(&mut js_errors);
            }
        }

        let mut built_configs = Vec::new();

        for config in configs {
            let path = config.path.clone();
            let dir = path.parent().unwrap().to_path_buf();
            let ignore_patterns = config.ignore_patterns.clone();
            let is_root_config = root_config_dir
                .and_then(|root| path.parent().map(|parent| parent == root))
                .unwrap_or(false);

            if !is_root_config {
                let options = &config.options;
                if options.type_aware.is_some() {
                    errors
                        .push(ConfigLoadError::Diagnostic(nested_type_aware_not_supported(&path)));
                    continue;
                }
                if options.type_check.is_some() {
                    errors
                        .push(ConfigLoadError::Diagnostic(nested_type_check_not_supported(&path)));
                    continue;
                }
                if options.deny_warnings.is_some() {
                    errors.push(ConfigLoadError::Diagnostic(nested_deny_warnings_not_supported(
                        &path,
                    )));
                    continue;
                }
                if options.max_warnings.is_some() {
                    errors.push(ConfigLoadError::Diagnostic(nested_max_warnings_not_supported(
                        &path,
                    )));
                    continue;
                }
                if options.report_unused_disable_directives.is_some() {
                    errors.push(ConfigLoadError::Diagnostic(
                        nested_report_unused_disable_directives_not_supported(&path),
                    ));
                    continue;
                }
            }

            let builder = match ConfigStoreBuilder::from_oxlintrc(
                false,
                config,
                self.external_linter,
                self.external_plugin_store,
                self.workspace_uri,
            ) {
                Ok(builder) => builder,
                Err(e) => {
                    errors.push(ConfigLoadError::Build { path, error: e.to_string() });
                    continue;
                }
            };

            let extended_paths = builder.extended_paths.clone();

            match builder
                .with_filters(self.filters)
                .build(self.external_plugin_store)
                .map_err(|e| ConfigLoadError::Build { path: path.clone(), error: e.to_string() })
            {
                Ok(config) => built_configs.push(LoadedConfig {
                    dir,
                    config,
                    ignore_patterns,
                    extended_paths,
                }),
                Err(e) => errors.push(e),
            }
        }

        (built_configs, errors)
    }

    pub(crate) fn load_discovered_with_root_dir(
        &mut self,
        root_dir: &Path,
        configs: impl IntoIterator<Item = DiscoveredConfigFile>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>) {
        self.load_many(configs, Some(root_dir))
    }

    /// Try to load config from a specific directory.
    ///
    /// In Vite+ mode (`VP_VERSION` set): only checks for `vite.config.ts`.
    /// Otherwise: checks for `.oxlintrc.json`, `.oxlintrc.jsonc`, and `oxlint.config.ts`.
    ///
    /// Returns `Ok(Some(config))` if found, `Ok(None)` if not found, or `Err` on error.
    fn try_load_config_from_dir(&self, dir: &Path) -> Result<Option<Oxlintrc>, OxcDiagnostic> {
        let config_file = ConfigDiscovery::new(OXLINT_CONFIG_FILE_NAMES, vp_version().is_some())
            .find_unique_config_in_directory(dir)
            .map_err(Into::<oxc_diagnostics::OxcDiagnostic>::into)?;

        match config_file {
            Some(DiscoveredConfigFile::Json(path) | DiscoveredConfigFile::Jsonc(path)) => {
                Oxlintrc::from_file(&path).map(Some)
            }
            Some(DiscoveredConfigFile::Js(path)) => {
                let config = self.load_root_js_config(&path)?;
                debug_assert!(config.is_some(), "oxlint.config.ts should always return a config");
                Ok(config)
            }
            Some(DiscoveredConfigFile::Vite(path)) => self.load_root_js_config(&path),
            None => Ok(None),
        }
    }

    pub(crate) fn load_root_config(
        &self,
        cwd: &Path,
        config_path: Option<&PathBuf>,
    ) -> Result<Oxlintrc, OxcDiagnostic> {
        if let Some(config_path) = config_path {
            return self.load_explicit_config(cwd, config_path);
        }

        match self.try_load_config_from_dir(cwd)? {
            Some(config) => Ok(config),
            None => Ok(Oxlintrc::default()),
        }
    }

    /// Load root config by searching up parent directories.
    ///
    /// This is used by the LSP when a workspace folder is nested (e.g., `apps/app1`).
    /// It searches from the current directory up to parent directories to find a config file.
    ///
    /// # Arguments
    /// * `cwd` - Current working directory (workspace root for LSP)
    /// * `config_path` - Optional explicit path to the root config file
    ///
    /// # Returns
    /// The first config found when searching up the directory tree, or default if none found.
    pub(crate) fn load_root_config_with_ancestor_search(
        &self,
        cwd: &Path,
        config_path: Option<&PathBuf>,
    ) -> Result<Oxlintrc, OxcDiagnostic> {
        // If an explicit config path is provided, use it directly
        if let Some(config_path) = config_path {
            return self.load_explicit_config(cwd, config_path);
        }

        // Search up the directory tree for a config file
        let mut current = Some(cwd);
        while let Some(dir) = current {
            if let Some(config) = self.try_load_config_from_dir(dir)? {
                return Ok(config);
            }
            // Move to parent directory
            current = dir.parent();
        }

        // No config found in any ancestor directory
        Ok(Oxlintrc::default())
    }

    /// Load an explicitly specified config file (via `--config`).
    /// For JS/TS configs, `None` from JS side (e.g., vite.config.ts without `.lint`) is an error.
    fn load_explicit_config(
        &self,
        cwd: &Path,
        config_path: &Path,
    ) -> Result<Oxlintrc, OxcDiagnostic> {
        let full_path = cwd.join(config_path);
        if is_js_config_path(&full_path) {
            return self.load_root_js_config(&full_path)?.ok_or_else(|| {
                OxcDiagnostic::error(format!(
                    "Expected a `lint` field in the default export of {}",
                    full_path.display()
                ))
            });
        }
        Oxlintrc::from_file(&full_path)
    }

    /// Load a single JS/TS config file. Returns `Ok(None)` when JS side signals "skip"
    /// (e.g., vite.config.ts without `.lint` field).
    fn load_root_js_config(&self, path: &Path) -> Result<Option<Oxlintrc>, OxcDiagnostic> {
        match self.load_js_configs(&[path.to_path_buf()]) {
            Ok(mut results) => Ok(results.pop().and_then(|r| r.config)),
            Err(errors) => {
                if let Some(first) = errors.into_iter().next() {
                    match first {
                        ConfigLoadError::JsConfigFileFoundButJsRuntimeNotAvailable => {
                            Err(js_config_not_supported_diagnostic(path))
                        }
                        ConfigLoadError::Diagnostic(diag) => Err(diag),
                        // `load_js_configs` only returns the two variants above, but keep this
                        // resilient if that changes.
                        ConfigLoadError::Parse { error, .. } => Err(error),
                        ConfigLoadError::Build { error, .. } => Err(OxcDiagnostic::error(error)),
                    }
                } else {
                    Err(OxcDiagnostic::error("Failed to load JavaScript/TypeScript config."))
                }
            }
        }
    }

    /// Load the root configuration and optionally discover and load nested configs.
    ///
    /// This is the main entry point for CLI config loading. It first loads the root
    /// `oxlintrc` configuration, then optionally discovers and loads nested configs
    /// by walking up from each file path's directory.
    ///
    /// # Arguments
    /// * `cwd` - Current working directory for resolving relative paths
    /// * `config_path` - Optional explicit path to the root config file
    /// * `paths` - File paths to lint (used for discovering nested configs)
    /// * `search_for_nested_configs` - Whether to discover nested configs in ancestor directories
    ///
    /// # Errors
    /// Returns [`CliConfigLoadError::RootConfig`] if the root config fails to load,
    /// or [`CliConfigLoadError::NestedConfigs`] if any nested config fails to load.
    pub fn load_root_and_nested(
        &mut self,
        cwd: &Path,
        config_path: Option<&PathBuf>,
        paths: &[Arc<OsStr>],
        search_for_nested_configs: bool,
    ) -> Result<LoadedConfigs, CliConfigLoadError> {
        let oxlintrc = match self.load_root_config(cwd, config_path) {
            Ok(config) => config,
            Err(err) => return Err(CliConfigLoadError::RootConfig(err)),
        };

        if !search_for_nested_configs {
            return Ok(LoadedConfigs {
                root: oxlintrc,
                nested: FxHashMap::default(),
                nested_ignore_patterns: vec![],
            });
        }

        // Discover config files by walking up from each file's directory
        let config_paths: Vec<_> =
            paths.iter().map(|p| Path::new(p.as_ref()).to_path_buf()).collect();
        let discovered_configs = discover_configs_in_ancestors(&config_paths, &oxlintrc.path);

        let (configs, errors) = self.load_many(discovered_configs, Some(cwd));

        // Fail if any config failed (CLI requires all configs to be valid)
        if !errors.is_empty() {
            return Err(CliConfigLoadError::NestedConfigs(errors));
        }

        // Convert loaded configs to nested config format
        let mut nested_ignore_patterns = Vec::with_capacity(configs.len());
        let nested_configs = build_nested_configs(configs, &mut nested_ignore_patterns, None);

        Ok(LoadedConfigs { root: oxlintrc, nested: nested_configs, nested_ignore_patterns })
    }
}

/// Build a map of directory paths to their effective configurations.
///
/// Processes a list of loaded configs and organizes them into a hashmap keyed by
/// directory path. Also collects ignore patterns and optionally tracks extended paths.
///
/// # Arguments
/// * `configs` - Successfully loaded configurations to process
/// * `nested_ignore_patterns` - Output: populated with (ignore_patterns, directory) tuples
/// * `extended_paths` - Optional set to collect paths from `extends` directives.
///   Pass `Some` when tracking extended configs for file watching (LSP), `None` otherwise (CLI).
pub fn build_nested_configs(
    configs: Vec<LoadedConfig>,
    nested_ignore_patterns: &mut Vec<(Vec<String>, PathBuf)>,
    mut extended_paths: Option<&mut FxHashSet<PathBuf>>,
) -> FxHashMap<PathBuf, Config> {
    let mut nested_configs =
        FxHashMap::<PathBuf, Config>::with_capacity_and_hasher(configs.len(), FxBuildHasher);

    for loaded in configs {
        nested_ignore_patterns.push((loaded.ignore_patterns, loaded.dir.clone()));
        if let Some(extended_paths) = extended_paths.as_deref_mut() {
            extended_paths.extend(loaded.extended_paths);
        }
        nested_configs.insert(loaded.dir, loaded.config);
    }

    nested_configs
}

fn js_config_not_supported_diagnostic(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "JavaScript/TypeScript config file ({}) found but JS runtime not available.",
        path.display()
    ))
    .with_help("Run oxlint via the npm package, or use JSON config files (.oxlintrc.json or .oxlintrc.jsonc).")
}

fn is_js_config_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("js" | "mjs" | "cjs" | "ts" | "cts" | "mts")
    )
}

fn nested_type_aware_not_supported(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "The `options.typeAware` option is only supported in the root config, but it was found in {}.",
        path.display()
    ))
    .with_help("Move `options.typeAware` to the root configuration file.")
}

fn nested_type_check_not_supported(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "The `options.typeCheck` option is only supported in the root config, but it was found in {}.",
        path.display()
    ))
    .with_help("Move `options.typeCheck` to the root configuration file.")
}

fn nested_deny_warnings_not_supported(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "The `options.denyWarnings` option is only supported in the root config, but it was found in {}.",
        path.display()
    ))
    .with_help("Move `options.denyWarnings` to the root configuration file.")
}

fn nested_max_warnings_not_supported(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "The `options.maxWarnings` option is only supported in the root config, but it was found in {}.",
        path.display()
    ))
    .with_help("Move `options.maxWarnings` to the root configuration file.")
}

fn nested_report_unused_disable_directives_not_supported(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "The `options.reportUnusedDisableDirectives` option is only supported in the root config, but it was found in {}.",
        path.display()
    ))
    .with_help("Move `options.reportUnusedDisableDirectives` to the root configuration file.")
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use oxc_linter::{ConfigStoreBuilder, ExternalPluginStore};

    use super::{ConfigLoadError, ConfigLoader, is_js_config_path};
    use crate::config_discovery::DiscoveredConfigFile;
    #[cfg(feature = "napi")]
    use crate::js_config::{JsConfigLoaderCb, JsConfigResult};

    #[cfg(feature = "napi")]
    fn make_js_loader<F>(f: F) -> JsConfigLoaderCb
    where
        F: Fn(Vec<String>) -> Result<Vec<JsConfigResult>, Vec<oxc_diagnostics::OxcDiagnostic>>
            + Send
            + Sync
            + 'static,
    {
        Box::new(f)
    }

    #[cfg(feature = "napi")]
    fn make_js_config(
        path: PathBuf,
        type_aware: Option<bool>,
        type_check: Option<bool>,
    ) -> JsConfigResult {
        let mut config: oxc_linter::Oxlintrc = serde_json::from_value(serde_json::json!({
            "options": { "typeAware": type_aware, "typeCheck": type_check }
        }))
        .unwrap();
        config.path = path.clone();
        if let Some(config_dir) = path.parent() {
            config.set_config_dir(config_dir);
        }
        JsConfigResult { path, config: Some(config) }
    }

    #[cfg(feature = "napi")]
    fn make_js_config_with_rules(path: PathBuf, rules: &serde_json::Value) -> JsConfigResult {
        let mut config: oxc_linter::Oxlintrc = serde_json::from_value(serde_json::json!({
            "rules": rules
        }))
        .unwrap();
        config.path = path.clone();
        if let Some(config_dir) = path.parent() {
            config.set_config_dir(config_dir);
        }
        JsConfigResult { path, config: Some(config) }
    }

    #[test]
    fn test_config_path_with_parent_references() {
        let cwd = std::env::current_dir().unwrap();
        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        // Test case 1: Invalid path that should fail
        let invalid_config = PathBuf::from("child/../../fixtures/cli/linter/eslintrc.json");
        let result = loader.load_root_config(&cwd, Some(&invalid_config));
        assert!(result.is_err(), "Expected config lookup to fail with invalid path");

        // Test case 2: Valid path that should pass
        let valid_config = PathBuf::from("fixtures/cli/linter/eslintrc.json");
        let result = loader.load_root_config(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with valid path");

        // Test case 3: Valid path using parent directory (..) syntax that should pass
        let valid_parent_config = PathBuf::from("fixtures/cli/linter/../linter/eslintrc.json");
        let result = loader.load_root_config(&cwd, Some(&valid_parent_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with parent directory syntax");

        // Verify the resolved path is correct
        if let Ok(config) = result {
            assert_eq!(
                config.path.file_name().unwrap().to_str().unwrap(),
                "eslintrc.json",
                "Config file name should be preserved after path resolution"
            );
        }
    }

    #[test]
    fn test_load_root_config_with_ancestor_search() {
        let cwd = std::env::current_dir().unwrap();
        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        // Test case 1: Search from nested directory should find config in parent
        // Uses fixture: ancestor_search/apps/app1 -> should find ancestor_search/.oxlintrc.json
        let nested_dir = cwd.join("apps/oxlint/fixtures/cli/ancestor_search/apps/app1");
        if nested_dir.exists() {
            let result = loader.load_root_config_with_ancestor_search(&nested_dir, None);
            assert!(result.is_ok(), "Expected ancestor search to find config or return default");

            // Verify the config was actually found (not just default)
            if let Ok(config) = result {
                // The fixture has a .oxlintrc.json with no-console rule
                assert!(
                    config.path.ends_with(".oxlintrc.json") || config.path.as_os_str().is_empty(),
                    "Expected to find .oxlintrc.json or default config"
                );
            }
        }

        // Test case 2: Explicit config path should still work
        // Uses dedicated fixture with .oxlintrc.json
        let valid_config =
            PathBuf::from("fixtures/cli/ancestor_search_explicit_config/.oxlintrc.json");
        let result = loader.load_root_config_with_ancestor_search(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with explicit path");

        // Test case 3: When no config exists in any ancestor, should return default
        let temp_dir = std::env::temp_dir().join("oxc_test_no_config");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temporary test directory");
        let result = loader.load_root_config_with_ancestor_search(&temp_dir, None);
        assert!(result.is_ok(), "Expected default config when no config found");
        std::fs::remove_dir_all(&temp_dir).expect("Failed to cleanup temporary test directory");
    }

    #[test]
    fn test_is_js_config_path() {
        assert!(is_js_config_path(Path::new("my-config.js")));
        assert!(is_js_config_path(Path::new("my-config.cjs")));
        assert!(is_js_config_path(Path::new("my-config.mjs")));
        assert!(is_js_config_path(Path::new("my-config.ts")));
        assert!(is_js_config_path(Path::new("my-config.cts")));
        assert!(is_js_config_path(Path::new("my-config.mts")));
        assert!(!is_js_config_path(Path::new("oxlint.config.json")));
    }

    #[test]
    fn test_nested_json_config_rejects_type_aware() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/.oxlintrc.json");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, r#"{ "options": { "typeAware": true } }"#).unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (_configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ConfigLoadError::Diagnostic(_)));
    }

    #[test]
    fn test_nested_json_config_rejects_deny_warnings() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/.oxlintrc.json");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, r#"{ "options": { "denyWarnings": true } }"#).unwrap();
        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (_configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ConfigLoadError::Diagnostic(_)));
    }

    #[test]
    fn test_nested_json_config_rejects_report_unused_disable_directives() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/.oxlintrc.json");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(
            &nested_path,
            r#"{ "options": { "reportUnusedDisableDirectives": "warn" } }"#,
        )
        .unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (_configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ConfigLoadError::Diagnostic(_)));
    }

    #[test]
    fn test_nested_json_config_allows_type_aware_from_extends() {
        let root_dir = tempfile::tempdir().unwrap();
        let base_path = root_dir.path().join("base/.oxlintrc.json");
        let nested_path = root_dir.path().join("nested/.oxlintrc.json");
        std::fs::create_dir_all(base_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&base_path, r#"{ "options": { "typeAware": true } }"#).unwrap();
        std::fs::write(&nested_path, r#"{ "extends": ["../base/.oxlintrc.json"] }"#).unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        assert!(errors.is_empty());
        assert_eq!(configs.len(), 1);
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_root_oxlint_config_ts_allows_type_aware() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.path().join("oxlint.config.ts");
        std::fs::write(&root_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| make_js_config(PathBuf::from(path), Some(true), None))
                .collect())
        });
        let loader = loader.with_js_config_loader(Some(&js_loader));

        let config = loader
            .load_root_config(root_dir.path(), Some(&PathBuf::from("oxlint.config.ts")))
            .unwrap();

        assert_eq!(config.options.type_aware, Some(true));
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_root_oxlint_config_ts_allows_type_check() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.path().join("oxlint.config.ts");
        std::fs::write(&root_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| make_js_config(PathBuf::from(path), None, Some(true)))
                .collect())
        });
        let loader = loader.with_js_config_loader(Some(&js_loader));

        let config = loader
            .load_root_config(root_dir.path(), Some(&PathBuf::from("oxlint.config.ts")))
            .unwrap();

        assert_eq!(config.options.type_check, Some(true));
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_root_oxlint_config_ts_rejects_missing_builtin_rule() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.path().join("oxlint.config.ts");
        std::fs::write(&root_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let js_loader = make_js_loader({
            move |paths| {
                assert_eq!(paths, vec![root_path.to_string_lossy().to_string()]);
                Ok(vec![make_js_config_with_rules(
                    root_path.clone(),
                    &serde_json::json!({ "no-console-typo": "error" }),
                )])
            }
        });

        let oxlintrc = {
            let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
            let loader = loader.with_js_config_loader(Some(&js_loader));
            loader
                .load_root_config(root_dir.path(), Some(&PathBuf::from("oxlint.config.ts")))
                .unwrap()
        };

        let err = ConfigStoreBuilder::from_oxlintrc(
            false,
            oxlintrc,
            None,
            &mut external_plugin_store,
            None,
        )
        .unwrap_err();

        assert_eq!(err.to_string(), "Rule 'no-console-typo' not found in plugin 'eslint'");
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_rejects_type_aware() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/oxlint.config.ts");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| make_js_config(PathBuf::from(path), Some(false), None))
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (_configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ConfigLoadError::Diagnostic(_)));
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_rejects_type_check() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/oxlint.config.ts");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| make_js_config(PathBuf::from(path), None, Some(false)))
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (_configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ConfigLoadError::Diagnostic(_)));
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_rejects_deny_warnings() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/oxlint.config.ts");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| {
                    let path = PathBuf::from(path);
                    let mut config = make_js_config(path.clone(), None, None).config.unwrap();
                    config.options.deny_warnings = Some(true);
                    JsConfigResult { path, config: Some(config) }
                })
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (_configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ConfigLoadError::Diagnostic(_)));
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_allows_type_aware_from_extends() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/oxlint.config.ts");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| {
                    let path = PathBuf::from(path);
                    let mut config = make_js_config(path.clone(), None, None).config.unwrap();
                    config.extends_configs = vec![
                        serde_json::from_value(
                            serde_json::json!({ "options": { "typeAware": true } }),
                        )
                        .unwrap(),
                    ];
                    JsConfigResult { path, config: Some(config) }
                })
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(errors.is_empty());
        assert_eq!(configs.len(), 1);
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_allows_type_check_from_extends() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/oxlint.config.ts");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| {
                    let path = PathBuf::from(path);
                    let mut config = make_js_config(path.clone(), None, None).config.unwrap();
                    config.extends_configs = vec![
                        serde_json::from_value(
                            serde_json::json!({ "options": { "typeCheck": true } }),
                        )
                        .unwrap(),
                    ];
                    JsConfigResult { path, config: Some(config) }
                })
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(errors.is_empty());
        assert_eq!(configs.len(), 1);
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_vite_config_loads() {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/vite.config.ts");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let expected_path = nested_path.clone();
        let js_loader = make_js_loader(move |paths| {
            assert_eq!(paths, vec![expected_path.to_string_lossy().to_string()]);
            Ok(paths
                .into_iter()
                .map(|path| make_js_config(PathBuf::from(path), None, None))
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (configs, errors) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Vite(nested_path)],
        );
        assert!(errors.is_empty());
        assert_eq!(configs.len(), 1);
    }

    #[test]
    fn test_jsonc_config_discovery() {
        let root_dir = tempfile::tempdir().unwrap();
        // Create only a .oxlintrc.jsonc file
        std::fs::write(root_dir.path().join(".oxlintrc.jsonc"), r#"{ /* comment */ "rules": {} }"#)
            .unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let result = loader.load_root_config(root_dir.path(), None);
        assert!(result.is_ok(), "Expected .oxlintrc.jsonc to be discovered and loaded");
        let config = result.unwrap();
        assert!(
            config.path.to_string_lossy().ends_with(".oxlintrc.jsonc"),
            "Expected config path to end with .oxlintrc.jsonc, got: {}",
            config.path.display()
        );
    }

    #[test]
    fn test_json_and_jsonc_conflict() {
        let root_dir = tempfile::tempdir().unwrap();
        // Create both .oxlintrc.json and .oxlintrc.jsonc
        std::fs::write(root_dir.path().join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();
        std::fs::write(root_dir.path().join(".oxlintrc.jsonc"), r#"{ /* comment */ "rules": {} }"#)
            .unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let result = loader.load_root_config(root_dir.path(), None);
        assert!(
            result.is_err(),
            "Expected an error when both .oxlintrc.json and .oxlintrc.jsonc exist"
        );
    }

    #[test]
    fn test_json_and_ts_conflict() {
        let root_dir = tempfile::tempdir().unwrap();
        std::fs::write(root_dir.path().join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();
        std::fs::write(root_dir.path().join("oxlint.config.ts"), "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let result = loader.load_root_config(root_dir.path(), None);
        assert!(result.is_err(), "Expected an error when both JSON and TS configs exist");
    }

    #[test]
    fn test_jsonc_and_ts_conflict() {
        let root_dir = tempfile::tempdir().unwrap();
        std::fs::write(root_dir.path().join(".oxlintrc.jsonc"), r#"{ /* comment */ "rules": {} }"#)
            .unwrap();
        std::fs::write(root_dir.path().join("oxlint.config.ts"), "export default {};").unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        let result = loader.load_root_config(root_dir.path(), None);
        assert!(result.is_err(), "Expected an error when both JSONC and TS configs exist");
    }

    #[test]
    fn test_discover_configs_skips_node_modules() {
        use super::discover_configs_in_tree;

        let root_dir = tempfile::tempdir().unwrap();
        // Create a valid root config
        let base_config = root_dir.path().join(".oxlintrc.json");
        std::fs::write(&base_config, r#"{ "rules": {} }"#).unwrap();

        // Create a nested node_modules directory with a config file inside
        let node_modules = root_dir.path().join("node_modules").join("some-pkg");
        std::fs::create_dir_all(&node_modules).unwrap();
        std::fs::write(node_modules.join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

        // Create a legitimate nested config (not in node_modules)
        let nested_dir = root_dir.path().join("packages").join("foo");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(nested_dir.join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

        let discovered: Vec<_> =
            discover_configs_in_tree(root_dir.path(), &base_config).into_iter().collect();

        // Should find the nested config but NOT the one inside node_modules
        assert_eq!(discovered.len(), 1, "Expected only 1 config (not the node_modules one)");
        let path = match &discovered[0] {
            DiscoveredConfigFile::Json(p) => p.clone(),
            _ => panic!("Expected Json config"),
        };
        assert!(
            path.starts_with(nested_dir),
            "Expected config in packages/foo, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_discover_configs_skips_git_dir() {
        use super::discover_configs_in_tree;

        let root_dir = tempfile::tempdir().unwrap();
        let base_config = root_dir.path().join(".oxlintrc.json");
        std::fs::write(&base_config, r#"{ "rules": {} }"#).unwrap();

        let git_dir = root_dir.path().join(".git").join("hooks");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

        let nested_dir = root_dir.path().join("packages").join("foo");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(nested_dir.join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

        let discovered: Vec<_> =
            discover_configs_in_tree(root_dir.path(), &base_config).into_iter().collect();

        assert_eq!(discovered.len(), 1, "Expected only 1 config (not the .git one)");
        let path = match &discovered[0] {
            DiscoveredConfigFile::Json(p) => p.clone(),
            _ => panic!("Expected Json config"),
        };
        assert!(
            path.starts_with(nested_dir),
            "Expected config in packages/foo, got: {}",
            path.display()
        );
    }
}
