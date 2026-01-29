use std::path::{Path, PathBuf};

use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{
    Config, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore, LintFilter, Oxlintrc,
};

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
