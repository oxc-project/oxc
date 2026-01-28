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
use rustc_hash::FxHashSet;

use crate::DEFAULT_OXLINTRC_NAME;

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
) -> impl IntoIterator<Item = PathBuf> {
    let mut config_paths = FxHashSet::<PathBuf>::default();
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
            if let Some(config_path) = find_config_in_directory(dir) {
                config_paths.insert(config_path);
            }
            current = dir.parent();
        }
    }

    config_paths.into_iter()
}

/// Discover config files by walking DOWN from a root directory.
///
/// Used by LSP where we have a workspace root and need to discover all configs
/// upfront for file watching and diagnostics.
pub fn discover_configs_in_tree(root: &Path) -> impl IntoIterator<Item = PathBuf> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false) // don't skip hidden files
        .parents(false) // disable gitignore from parent dirs
        .ignore(false) // disable .ignore files
        .git_global(false) // disable global gitignore
        .follow_links(true)
        .build_parallel();

    let (sender, receiver) = mpsc::channel::<Vec<Arc<OsStr>>>();
    let mut builder = ConfigWalkBuilder { sender };
    walker.visit(&mut builder);
    drop(builder);

    receiver.into_iter().flatten().map(|p| PathBuf::from(p.as_ref()))
}

/// Check if a directory contains an oxlint config file.
fn find_config_in_directory(dir: &Path) -> Option<PathBuf> {
    let config_path = dir.join(DEFAULT_OXLINTRC_NAME);
    if config_path.is_file() { Some(config_path) } else { None }
}

// Helper types for parallel directory walking
struct ConfigWalkBuilder {
    sender: mpsc::Sender<Vec<Arc<OsStr>>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for ConfigWalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(ConfigWalkCollector { paths: vec![], sender: self.sender.clone() })
    }
}

struct ConfigWalkCollector {
    paths: Vec<Arc<OsStr>>,
    sender: mpsc::Sender<Vec<Arc<OsStr>>>,
}

impl Drop for ConfigWalkCollector {
    fn drop(&mut self) {
        let paths = std::mem::take(&mut self.paths);
        self.sender.send(paths).unwrap();
    }
}

impl ignore::ParallelVisitor for ConfigWalkCollector {
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                if is_config_file(&entry) {
                    self.paths.push(entry.path().as_os_str().into());
                }
                ignore::WalkState::Continue
            }
            Err(_) => ignore::WalkState::Skip,
        }
    }
}

fn is_config_file(entry: &DirEntry) -> bool {
    let Some(file_type) = entry.file_type() else { return false };
    if file_type.is_dir() {
        return false;
    }
    let Some(file_name) = entry.path().file_name() else { return false };
    file_name == DEFAULT_OXLINTRC_NAME
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

pub struct ConfigLoader<'a> {
    external_linter: Option<&'a ExternalLinter>,
    external_plugin_store: &'a mut ExternalPluginStore,
    filters: &'a [LintFilter],
}

impl<'a> ConfigLoader<'a> {
    /// Create a new ConfigLoader
    ///
    /// # Arguments
    /// * `external_linter` - Optional external linter for plugin support
    /// * `external_plugin_store` - Store for external plugins
    /// * `filters` - Lint filters to apply to configs
    pub fn new(
        external_linter: Option<&'a ExternalLinter>,
        external_plugin_store: &'a mut ExternalPluginStore,
        filters: &'a [LintFilter],
    ) -> Self {
        Self { external_linter, external_plugin_store, filters }
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
    pub fn load_many(
        &mut self,
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>) {
        let mut configs = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            match self.load(path.as_ref()) {
                Ok(config) => configs.push(config),
                Err(e) => errors.push(e),
            }
        }

        (configs, errors)
    }
}
