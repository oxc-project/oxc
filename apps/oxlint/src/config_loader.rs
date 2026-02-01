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

use crate::DEFAULT_OXLINTRC_NAME;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum DiscoveredConfig {
    Json(PathBuf),
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
            if let Some(config) = find_config_in_directory(dir) {
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
fn find_config_in_directory(dir: &Path) -> Option<DiscoveredConfig> {
    let config_path = dir.join(DEFAULT_OXLINTRC_NAME);
    if config_path.is_file() { Some(DiscoveredConfig::Json(config_path)) } else { None }
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
    Parse { path: PathBuf, error: OxcDiagnostic },
    /// Failed to build the ConfigStore
    Build { path: PathBuf, error: String },
}

impl ConfigLoadError {
    /// Get the path of the config file that failed
    pub fn path(&self) -> &Path {
        match self {
            ConfigLoadError::Parse { path, .. } | ConfigLoadError::Build { path, .. } => path,
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
        Self { external_linter, external_plugin_store, filters, workspace_uri }
    }

    /// Load a single config from a file path
    fn load(&mut self, path: &Path) -> Result<LoadedConfig, ConfigLoadError> {
        let oxlintrc = Oxlintrc::from_file(path)
            .map_err(|error| ConfigLoadError::Parse { path: path.to_path_buf(), error })?;

        let dir = oxlintrc.path.parent().unwrap().to_path_buf();
        let ignore_patterns = oxlintrc.ignore_patterns.clone();

        let builder = ConfigStoreBuilder::from_oxlintrc(
            false,
            oxlintrc,
            self.external_linter,
            self.external_plugin_store,
            self.workspace_uri,
        )
        .map_err(|e| ConfigLoadError::Build { path: path.to_path_buf(), error: e.to_string() })?;

        let extended_paths = builder.extended_paths.clone();

        let config =
            builder.with_filters(self.filters).build(self.external_plugin_store).map_err(|e| {
                ConfigLoadError::Build { path: path.to_path_buf(), error: e.to_string() }
            })?;

        Ok(LoadedConfig { dir, config, ignore_patterns, extended_paths })
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

        for path in paths {
            match path {
                DiscoveredConfig::Json(path) => match self.load(&path) {
                    Ok(config) => configs.push(config),
                    Err(e) => errors.push(e),
                },
            }
        }

        (configs, errors)
    }

    pub(crate) fn load_discovered(
        &mut self,
        configs: impl IntoIterator<Item = DiscoveredConfig>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>) {
        self.load_many(configs)
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
        let oxlintrc = match find_oxlint_config(cwd, config_path) {
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

fn find_oxlint_config(cwd: &Path, config: Option<&PathBuf>) -> Result<Oxlintrc, OxcDiagnostic> {
    let path: &Path = config.map_or(DEFAULT_OXLINTRC_NAME.as_ref(), PathBuf::as_ref);
    let full_path = cwd.join(path);

    if config.is_some() || full_path.exists() {
        return Oxlintrc::from_file(&full_path);
    }
    Ok(Oxlintrc::default())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::find_oxlint_config;

    #[test]
    fn test_config_path_with_parent_references() {
        let cwd = std::env::current_dir().unwrap();

        // Test case 1: Invalid path that should fail
        let invalid_config = PathBuf::from("child/../../fixtures/linter/eslintrc.json");
        let result = find_oxlint_config(&cwd, Some(&invalid_config));
        assert!(result.is_err(), "Expected config lookup to fail with invalid path");

        // Test case 2: Valid path that should pass
        let valid_config = PathBuf::from("fixtures/linter/eslintrc.json");
        let result = find_oxlint_config(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with valid path");

        // Test case 3: Valid path using parent directory (..) syntax that should pass
        let valid_parent_config = PathBuf::from("fixtures/linter/../linter/eslintrc.json");
        let result = find_oxlint_config(&cwd, Some(&valid_parent_config));
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
}
