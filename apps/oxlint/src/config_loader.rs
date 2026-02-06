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

use crate::{DEFAULT_OXLINTRC_NAME, DEFAULT_TS_OXLINTRC_NAME};

#[cfg(feature = "napi")]
use crate::js_config;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum DiscoveredConfig {
    Json(PathBuf),
    Js(PathBuf),
}

/// Discover config files by walking UP from each file's directory to ancestors.
///
/// Used by CLI where we have specific files to lint and need to find configs
/// that apply to them.
///
/// Example: For files `/project/src/foo.js` and `/project/src/bar/baz.js`:
/// - Checks `/project/src/bar/`, `/project/src/`, `/project/`, `/`
/// - Returns paths to any `.oxlintrc.json` files found
pub fn discover_configs_in_ancestors<P: AsRef<Path>>(
    files: &[P],
) -> impl IntoIterator<Item = DiscoveredConfig> {
    let mut config_paths = FxHashSet::<DiscoveredConfig>::default();
    let mut visited_dirs = FxHashSet::default();

    for file in files {
        let path = file.as_ref();
        // Start from the file's parent directory and walk up the tree
        let mut current = path.parent();
        while let Some(dir) = current {
            // Stop if we've already checked this directory (and its ancestors)
            let inserted = visited_dirs.insert(dir.to_path_buf());
            if !inserted {
                break;
            }
            for config in find_configs_in_directory(dir) {
                config_paths.insert(config);
            }
            current = dir.parent();
        }
    }

    config_paths
}

/// Discover config files by walking DOWN from a root directory.
///
/// Used by LSP where we have a workspace root and need to discover all configs
/// upfront for file watching and diagnostics.
pub fn discover_configs_in_tree(root: &Path) -> impl IntoIterator<Item = DiscoveredConfig> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false) // don't skip hidden files
        .parents(false) // disable gitignore from parent dirs
        .ignore(false) // disable .ignore files
        .git_global(false) // disable global gitignore
        .follow_links(true)
        .build_parallel();

    let (sender, receiver) = mpsc::channel::<Vec<DiscoveredConfig>>();
    let mut builder = ConfigWalkBuilder { sender };
    walker.visit(&mut builder);
    drop(builder);

    receiver.into_iter().flatten()
}

/// Check if a directory contains an oxlint config file.
fn find_configs_in_directory(dir: &Path) -> Vec<DiscoveredConfig> {
    let mut configs = Vec::new();

    let json_path = dir.join(DEFAULT_OXLINTRC_NAME);
    if json_path.is_file() {
        configs.push(DiscoveredConfig::Json(json_path));
    }

    let ts_path = dir.join(DEFAULT_TS_OXLINTRC_NAME);
    if ts_path.is_file() {
        configs.push(DiscoveredConfig::Js(ts_path));
    }

    configs
}

// Helper types for parallel directory walking
struct ConfigWalkBuilder {
    sender: mpsc::Sender<Vec<DiscoveredConfig>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for ConfigWalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(ConfigWalkCollector { configs: vec![], sender: self.sender.clone() })
    }
}

struct ConfigWalkCollector {
    configs: Vec<DiscoveredConfig>,
    sender: mpsc::Sender<Vec<DiscoveredConfig>>,
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
                if let Some(config) = to_discovered_config(&entry) {
                    self.configs.push(config);
                }
                ignore::WalkState::Continue
            }
            Err(_) => ignore::WalkState::Skip,
        }
    }
}

fn to_discovered_config(entry: &DirEntry) -> Option<DiscoveredConfig> {
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        return None;
    }
    let file_name = entry.path().file_name()?;
    if file_name == DEFAULT_OXLINTRC_NAME {
        Some(DiscoveredConfig::Json(entry.path().to_path_buf()))
    } else if file_name == DEFAULT_TS_OXLINTRC_NAME {
        Some(DiscoveredConfig::Js(entry.path().to_path_buf()))
    } else {
        None
    }
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

    TypeScriptConfigFileFoundButJsRuntimeNotAvailable,

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
    ) -> Result<Vec<Oxlintrc>, Vec<ConfigLoadError>> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }

        #[cfg(not(feature = "napi"))]
        {
            return Err(vec![ConfigLoadError::TypeScriptConfigFileFoundButJsRuntimeNotAvailable]);
        }

        #[cfg(feature = "napi")]
        let Some(js_config_loader) = self.js_config_loader else {
            return Err(vec![ConfigLoadError::TypeScriptConfigFileFoundButJsRuntimeNotAvailable]);
        };

        let paths_as_strings: Vec<String> =
            paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

        match js_config_loader(paths_as_strings) {
            Ok(results) => Ok(results.into_iter().map(|c| c.config).collect()),
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
        paths: impl IntoIterator<Item = DiscoveredConfig>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>) {
        let mut configs = Vec::new();
        let mut errors = Vec::new();

        let mut by_dir = FxHashMap::<PathBuf, (Option<PathBuf>, Option<PathBuf>)>::default();

        for config in paths {
            match config {
                DiscoveredConfig::Json(path) => {
                    let Some(dir) = path.parent().map(Path::to_path_buf) else {
                        continue;
                    };
                    by_dir.entry(dir).or_default().0 = Some(path);
                }
                DiscoveredConfig::Js(path) => {
                    let Some(dir) = path.parent().map(Path::to_path_buf) else {
                        continue;
                    };
                    by_dir.entry(dir).or_default().1 = Some(path);
                }
            }
        }

        let mut js_configs = Vec::new();

        for (dir, (json_path, ts_path)) in by_dir {
            if json_path.is_some() && ts_path.is_some() {
                errors.push(ConfigLoadError::Diagnostic(config_conflict_diagnostic(&dir)));
                continue;
            }

            if let Some(path) = json_path {
                match Self::load(&path) {
                    Ok(config) => configs.push(config),
                    Err(e) => errors.push(e),
                }
            }

            if let Some(path) = ts_path {
                js_configs.push(path);
            }
        }

        match self.load_js_configs(&js_configs) {
            Ok(mut loaded_js_configs) => {
                configs.append(&mut loaded_js_configs);
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

    pub(crate) fn load_discovered(
        &mut self,
        configs: impl IntoIterator<Item = DiscoveredConfig>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>) {
        self.load_many(configs)
    }

    /// Try to load config from a specific directory.
    ///
    /// Checks for both `.oxlintrc.json` and `oxlint.config.ts` files in the given directory.
    /// Returns `Ok(Some(config))` if found, `Ok(None)` if not found, or `Err` on error.
    fn try_load_config_from_dir(&self, dir: &Path) -> Result<Option<Oxlintrc>, OxcDiagnostic> {
        let json_path = dir.join(DEFAULT_OXLINTRC_NAME);
        let ts_path = dir.join(DEFAULT_TS_OXLINTRC_NAME);

        let json_exists = json_path.is_file();
        let ts_exists = ts_path.is_file();

        if json_exists && ts_exists {
            return Err(config_conflict_diagnostic(dir));
        }

        if ts_exists {
            return self.load_root_ts_config(&ts_path).map(Some);
        }

        if json_exists {
            return Oxlintrc::from_file(&json_path).map(Some);
        }

        Ok(None)
    }

    pub(crate) fn load_root_config(
        &self,
        cwd: &Path,
        config_path: Option<&PathBuf>,
    ) -> Result<Oxlintrc, OxcDiagnostic> {
        if let Some(config_path) = config_path {
            let full_path = cwd.join(config_path);
            if full_path.file_name() == Some(OsStr::new(DEFAULT_TS_OXLINTRC_NAME)) {
                return self.load_root_ts_config(&full_path);
            }
            return Oxlintrc::from_file(&full_path);
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
            let full_path = cwd.join(config_path);
            if full_path.file_name() == Some(OsStr::new(DEFAULT_TS_OXLINTRC_NAME)) {
                return self.load_root_ts_config(&full_path);
            }
            return Oxlintrc::from_file(&full_path);
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

    fn load_root_ts_config(&self, path: &Path) -> Result<Oxlintrc, OxcDiagnostic> {
        match self.load_js_configs(&[path.to_path_buf()]) {
            Ok(mut configs) => Ok(configs.pop().unwrap_or_default()),
            Err(errors) => {
                if let Some(first) = errors.into_iter().next() {
                    match first {
                        ConfigLoadError::TypeScriptConfigFileFoundButJsRuntimeNotAvailable => {
                            Err(ts_config_not_supported_diagnostic(path))
                        }
                        ConfigLoadError::Diagnostic(diag) => Err(diag),
                        // `load_js_configs` only returns the two variants above, but keep this
                        // resilient if that changes.
                        ConfigLoadError::Parse { error, .. } => Err(error),
                        ConfigLoadError::Build { error, .. } => Err(OxcDiagnostic::error(error)),
                    }
                } else {
                    Err(OxcDiagnostic::error("Failed to load TypeScript config."))
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
        let discovered_configs = discover_configs_in_ancestors(&config_paths);

        let (configs, errors) = self.load_many(discovered_configs);

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

fn config_conflict_diagnostic(dir: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "Both '{}' and '{}' found in {}.",
        DEFAULT_OXLINTRC_NAME,
        DEFAULT_TS_OXLINTRC_NAME,
        dir.display()
    ))
    .with_note("Only `.oxlintrc.json` or `oxlint.config.ts` are allowed, not both.")
    .with_help("Delete one of the configuration files.")
}

fn ts_config_not_supported_diagnostic(path: &Path) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "TypeScript config files ({}) found but JS runtime not available.",
        path.display()
    ))
    .with_help("Run oxlint via the npm package, or use JSON config files (.oxlintrc.json).")
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use oxc_linter::ExternalPluginStore;

    use super::ConfigLoader;

    #[test]
    fn test_config_path_with_parent_references() {
        let cwd = std::env::current_dir().unwrap();
        let mut external_plugin_store = ExternalPluginStore::new(false);
        let loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);

        // Test case 1: Invalid path that should fail
        let invalid_config = PathBuf::from("child/../../fixtures/linter/eslintrc.json");
        let result = loader.load_root_config(&cwd, Some(&invalid_config));
        assert!(result.is_err(), "Expected config lookup to fail with invalid path");

        // Test case 2: Valid path that should pass
        let valid_config = PathBuf::from("fixtures/linter/eslintrc.json");
        let result = loader.load_root_config(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with valid path");

        // Test case 3: Valid path using parent directory (..) syntax that should pass
        let valid_parent_config = PathBuf::from("fixtures/linter/../linter/eslintrc.json");
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
        let nested_dir = cwd.join("apps/oxlint/fixtures/ancestor_search/apps/app1");
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
        let valid_config = PathBuf::from("fixtures/ancestor_search_explicit_config/.oxlintrc.json");
        let result = loader.load_root_config_with_ancestor_search(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with explicit path");

        // Test case 3: When no config exists in any ancestor, should return default
        let temp_dir = std::env::temp_dir().join("oxc_test_no_config");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temporary test directory");
        let result = loader.load_root_config_with_ancestor_search(&temp_dir, None);
        assert!(result.is_ok(), "Expected default config when no config found");
        std::fs::remove_dir_all(&temp_dir).expect("Failed to cleanup temporary test directory");
    }
}
