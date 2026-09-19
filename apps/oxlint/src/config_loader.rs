use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use cow_utils::CowUtils;
use ignore::DirEntry;

use oxc_config::{
    ConfigConflict, ConfigDiscovery, ConfigFileNames, DiscoveredConfigFile, is_js_config_path,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{
    Config, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore, LintFilter, Oxlintrc,
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::utils::normalize_path;
use crate::{
    DEFAULT_JSONC_OXLINTRC_NAME, DEFAULT_MTS_OXLINTRC_NAME, DEFAULT_OXLINTRC_NAME,
    DEFAULT_TS_OXLINTRC_NAME,
};
use crate::{VITE_CONFIG_NAME, vp_version};

const GIT_DIR: &str = ".git";
const NODE_MODULES_DIR: &str = "node_modules";

#[cfg(feature = "napi")]
use crate::js_config;
#[cfg(feature = "napi")]
use crate::js_config::JsConfigResult;

#[cfg(not(feature = "napi"))]
pub struct JsConfigResult {
    pub config: Option<Oxlintrc>,
}

const OXLINT_CONFIG_FILE_NAMES: ConfigFileNames = ConfigFileNames {
    json: DEFAULT_OXLINTRC_NAME,
    jsonc: DEFAULT_JSONC_OXLINTRC_NAME,
    js: &[DEFAULT_TS_OXLINTRC_NAME, DEFAULT_MTS_OXLINTRC_NAME],
    vite: VITE_CONFIG_NAME,
};

fn config_discovery() -> ConfigDiscovery {
    ConfigDiscovery::new(OXLINT_CONFIG_FILE_NAMES, cfg!(feature = "napi") && vp_version().is_some())
}

pub fn config_file_names() -> Vec<&'static str> {
    config_discovery().config_file_names()
}

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
///
/// Conflicts (multiple configs in the same dir) are returned in the second
/// tuple element so callers can surface them as load errors alongside other
/// parse/build failures.
pub fn discover_configs_in_ancestors<P: AsRef<Path>>(
    files: &[P],
    base_config_path: &Path,
) -> (FxHashSet<DiscoveredConfigFile>, Vec<ConfigConflict>) {
    let discovery = config_discovery();
    let mut config_paths = FxHashSet::<DiscoveredConfigFile>::default();
    let mut visited_dirs = FxHashSet::default();
    let mut conflicts = Vec::new();

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
            match discovery.find_unique_config_by_readdir(dir, true) {
                Ok(Some(config)) => {
                    if config.path() == base_config_path {
                        base_config_found = true;
                    } else {
                        config_paths.insert(config);
                    }
                }
                Ok(None) => {}
                Err(conflict) => conflicts.push(conflict),
            }
            current = dir.parent();
        }
    }

    (config_paths, conflicts)
}

/// Discover config files by walking DOWN from a root directory.
/// Will skip the base config file (e.g., root oxlintrc) to avoid duplicate loading.
/// In Vite+ mode, only `vite.config.ts` is discovered.
///
/// Used by LSP where we have a workspace root and need to discover all configs
/// upfront for file watching and diagnostics.
///
/// `excluded_dirs` holds the `workingDirectories` of the workspace folder: a directory which is
/// handled by its own worker must not contribute a nested config to its workspace folder,
/// otherwise the same file would be linted with two different configurations.
pub fn discover_configs_in_tree(
    root: &Path,
    base_config_path: &Path,
    excluded_dirs: &[PathBuf],
) -> impl IntoIterator<Item = DiscoveredConfigFile> {
    let excluded_dirs = excluded_dirs.to_vec();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false) // don't skip hidden files
        .parents(false) // disable gitignore from parent dirs
        .ignore(false) // disable .ignore files
        .git_global(false) // disable global gitignore
        .follow_links(true)
        .filter_entry(move |entry| !excluded_dirs.iter().any(|dir| dir == entry.path()))
        .build_parallel();

    let (sender, receiver) = mpsc::channel::<Vec<DiscoveredConfigFile>>();
    let mut builder =
        ConfigWalkBuilder { sender, base_config_path: base_config_path.to_path_buf() };
    walker.visit(&mut builder);
    drop(builder);

    receiver.into_iter().flatten()
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
    config_discovery().discover_config_file(entry.path())
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

/// Non-fatal problems found while loading configuration files.
///
/// Unlike [`ConfigLoadError`], none of these stop the config from being used.
#[derive(Debug, Default)]
pub struct ConfigLoadWarnings {
    /// Report these unconditionally, e.g. a root-only option set in a nested config.
    pub always: Vec<OxcDiagnostic>,
    /// A config names type-aware rules without enabling `options.typeAware`, so they never run.
    /// Suppressed once type-aware linting is decided for the whole run, either way.
    pub type_aware_rules_never_run: Vec<OxcDiagnostic>,
    /// A config sets `options.typeCheck` without enabling `options.typeAware`, so `tsgolint` is
    /// never handed its files and nothing is type-checked. Suppressed once type-aware linting
    /// is decided for the whole run, and superseded by the fatal error when a flag requested
    /// type checking with nothing type-aware.
    pub type_check_without_type_aware: Vec<OxcDiagnostic>,
    /// A config does not set `options.typeCheck` although a config above it does, and the option
    /// is not inherited. Suppressed once `typeCheck` is decided for the whole run, on *or* off,
    /// because which config would have won no longer matters.
    pub type_check_not_inherited: Vec<OxcDiagnostic>,
}

/// What the CLI flags and the editor settings decided for the whole run.
///
/// A config problem which those decisions have already settled, or which a fatal error already
/// reports, is not reported a second time.
#[derive(Debug, Default, Clone, Copy)]
pub struct RunOverrides {
    /// `--type-aware`, `--type-check-only`, or the editor's `typeAware: true`: every file is
    /// linted with type-aware rules, regardless of what its config says.
    pub type_aware_forced: bool,
    /// The editor's `typeAware: false`: no file is, so nothing a config says about type-aware
    /// linting or type checking comes into play at all.
    pub type_aware_disabled: bool,
    /// `--type-check`, `--type-check-only`, or the editor's `typeCheck` setting either way:
    /// every file handed to `tsgolint` is type-checked, or none is.
    pub type_check_overridden: bool,
    /// The run is about to fail with "`--type-check` requires type-aware linting", which covers
    /// what [`ConfigLoadWarnings::type_check_without_type_aware`] reports.
    pub type_check_without_type_aware_is_fatal: bool,
}

impl ConfigLoadWarnings {
    /// The warnings to report, given what the run has already decided.
    pub fn to_report(&self, overrides: RunOverrides) -> impl Iterator<Item = &OxcDiagnostic> {
        fn unless(
            warnings: &[OxcDiagnostic],
            suppressed: bool,
        ) -> impl Iterator<Item = &OxcDiagnostic> {
            warnings.iter().take(if suppressed { 0 } else { warnings.len() })
        }

        let type_aware_settled = overrides.type_aware_forced || overrides.type_aware_disabled;
        self.always
            .iter()
            .chain(unless(&self.type_aware_rules_never_run, type_aware_settled))
            .chain(unless(
                &self.type_check_without_type_aware,
                type_aware_settled || overrides.type_check_without_type_aware_is_fatal,
            ))
            .chain(unless(
                &self.type_check_not_inherited,
                overrides.type_check_overridden || overrides.type_aware_disabled,
            ))
    }
}

/// High-level errors that can occur when loading CLI configurations.
///
/// This groups together failures related to the root configuration file
/// and to any nested configuration files discovered during loading.
#[derive(Debug)]
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
    /// Non-fatal problems found while loading nested configs, e.g. a root-only option set in a
    /// nested config. The option is ignored and linting continues.
    pub warnings: ConfigLoadWarnings,
}

pub fn materialize_default_plugins(config: &mut Oxlintrc) {
    config.plugins.get_or_insert_with(Default::default);
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
        {
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
    }

    /// Load multiple configs, returning successes and errors separately
    ///
    /// This allows callers to decide how to handle errors (fail fast vs continue)
    fn load_many(
        &mut self,
        paths: impl IntoIterator<Item = DiscoveredConfigFile>,
        root_config_dir: Option<&Path>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>, ConfigLoadWarnings) {
        let mut configs = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = ConfigLoadWarnings::default();

        // Group by parent dir to catch multiple configs in the same directory.
        // NOTE: CLI path (`discover_configs_in_ancestors`) already enforces uniqueness,
        // so this only fires for the LSP path (`discover_configs_in_tree`)
        // where the walker streams entries without grouping.
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

        for mut config in configs {
            let path = config.path.clone();
            let dir = path.parent().unwrap().to_path_buf();
            let ignore_patterns = config.ignore_patterns.clone();
            let is_root_config = root_config_dir
                .and_then(|root| path.parent().map(|parent| parent == root))
                .unwrap_or(false);
            let display_path = config_display_path(&path, root_config_dir);

            // `denyWarnings` and `maxWarnings` control the process exit code, which is a property
            // of the whole run and cannot be scoped to a directory. Every other option in
            // `options` is resolved per file from the config which governs it, so nested configs
            // may set them freely.
            //
            // A nested config which sets one of the two remaining root-only options used to be
            // dropped entirely, which silently removed every diagnostic for that package. Warn
            // and ignore just the offending option instead.
            if !is_root_config {
                if config.options.deny_warnings.take().is_some() {
                    warnings.always.push(nested_deny_warnings_not_supported(&display_path));
                }
                if config.options.max_warnings.take().is_some() {
                    warnings.always.push(nested_max_warnings_not_supported(&display_path));
                }
            }

            // Captured before the config is consumed, to warn below if these rules can never run.
            let explicit_type_aware_rules = if is_root_config {
                // Only nested configs are checked: this is where dropping the implicit inheritance
                // from the root config changes behaviour.
                Vec::new()
            } else {
                config.rules.explicitly_enabled_type_aware_rules()
            };

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
                Ok(config) => {
                    // `typeAware` is resolved from the config which governs the file, and is not
                    // inherited from the root config, so a nested config which names type-aware
                    // rules without enabling the option silently runs none of them.
                    if !explicit_type_aware_rules.is_empty() && config.type_aware() != Some(true) {
                        warnings.type_aware_rules_never_run.push(
                            nested_type_aware_rules_without_type_aware(
                                &display_path,
                                &explicit_type_aware_rules,
                            ),
                        );
                    }
                    // `typeCheck` is scoped the same way, and `tsgolint` only ever sees the files
                    // linted with type-aware rules, so type-checking a config's files without
                    // enabling `typeAware` for them reports nothing at all. The root config is
                    // built by the caller, so `root_config_warnings` covers that one.
                    if !is_root_config {
                        push_type_check_without_type_aware(
                            &config,
                            &display_path,
                            false,
                            &mut warnings,
                        );
                    }
                    built_configs.push(LoadedConfig {
                        dir,
                        config,
                        ignore_patterns,
                        extended_paths,
                    });
                }
                Err(e) => errors.push(e),
            }
        }

        (built_configs, errors, warnings)
    }

    pub(crate) fn load_discovered_with_root_dir(
        &mut self,
        root_dir: &Path,
        configs: impl IntoIterator<Item = DiscoveredConfigFile>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadError>, ConfigLoadWarnings) {
        self.load_many(configs, Some(root_dir))
    }

    /// Try to load config from a specific directory.
    ///
    /// In Vite+ mode (`VP_VERSION` set): only checks for `vite.config.ts`.
    /// Otherwise: checks for `.oxlintrc.json`, `.oxlintrc.jsonc`, `oxlint.config.ts`,
    /// and `oxlint.config.mts`.
    ///
    /// Returns `Ok(Some(config))` if found, `Ok(None)` if not found, or `Err` on error.
    fn try_load_config_from_dir(
        &self,
        discovery: &ConfigDiscovery,
        dir: &Path,
    ) -> Result<Option<Oxlintrc>, OxcDiagnostic> {
        let config_file =
            discovery.find_unique_config_by_readdir(dir, true).map_err(OxcDiagnostic::from)?;

        match config_file {
            Some(DiscoveredConfigFile::Json(path) | DiscoveredConfigFile::Jsonc(path)) => {
                Oxlintrc::from_file(&path).map(Some)
            }
            Some(DiscoveredConfigFile::Js(path)) => {
                let config = self.load_root_js_config(&path)?;
                debug_assert!(
                    config.is_some(),
                    "oxlint JS/TS config should always return a config"
                );
                Ok(config)
            }
            Some(DiscoveredConfigFile::Vite(path)) => self.load_root_js_config(&path),
            None => Ok(None),
        }
    }

    /// Completes a config load, once the caller has built the root config.
    ///
    /// The root config is the one config `ConfigLoader` does not build itself: the caller does,
    /// after resolving `extends`. The problems which need it, or which need the whole set of
    /// configs at once, are found here and added to the ones found while loading. Both the CLI
    /// and the language server call this, so an editor reports the same problems the CLI does,
    /// once each.
    ///
    /// Paths are printed relative to the root config's own directory, which is not necessarily
    /// the working directory: a config found by walking up the tree still names its packages the
    /// way it sees them. `display_root` is what to measure them against when there is no root
    /// config file at all. It is the directory the search started from (the working directory
    /// for the CLI, the workspace root for the language server), which is what every warning
    /// raised while loading is already relative to.
    ///
    /// `type_aware_forced` says whether `--type-aware`, `--type-check-only` or the editor's
    /// `typeAware` setting makes *every* file type-aware, which decides how many configs the
    /// `typeCheck` of a config above them was going to reach.
    #[must_use]
    pub fn finish_load(
        mut warnings: ConfigLoadWarnings,
        root: &Config,
        nested_configs: &FxHashMap<PathBuf, Config>,
        display_root: &Path,
        type_aware_forced: bool,
    ) -> ConfigLoadWarnings {
        let root_dir = root.path().and_then(Path::parent).unwrap_or(display_root);
        if let Some(root_path) = root.path() {
            push_type_check_without_type_aware(
                root,
                &config_display_path(root_path, Some(root_dir)),
                true,
                &mut warnings,
            );
        }
        push_configs_missing_type_check(
            root,
            nested_configs,
            root_dir,
            type_aware_forced,
            &mut warnings,
        );
        warnings
    }

    /// Load root config by searching up parent directories.
    ///
    /// It searches from the current directory up to parent directories to find a config file.
    ///
    /// # Arguments
    /// * `cwd` - Current working directory (workspace root for LSP)
    /// * `config_path` - Optional explicit path to the root config file
    ///
    /// # Returns
    /// The first config found when searching up the directory tree, or default if none found.
    pub(crate) fn load_root_config(
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
            if let Some(config) = self.try_load_config_from_dir(&config_discovery(), dir)? {
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
        // Normalize away `.`/`..` components:
        // this path (config's parent directory) becomes the root for `ignorePatterns` matching,
        // which is compared against the (normalized) lint target paths as a literal prefix.
        // If a root containing `..`, it never matches.
        let full_path = normalize_path(cwd.join(config_path));
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
                warnings: ConfigLoadWarnings::default(),
            });
        }

        // Discover config files by walking up from each file's directory
        let config_paths: Vec<_> =
            paths.iter().map(|p| Path::new(p.as_ref()).to_path_buf()).collect();
        let (discovered_configs, conflicts) =
            discover_configs_in_ancestors(&config_paths, &oxlintrc.path);

        let (configs, mut errors, warnings) = self.load_many(discovered_configs, Some(cwd));

        // Propagate upstream conflicts as load errors alongside parse/build failures.
        for conflict in conflicts {
            errors.push(ConfigLoadError::Diagnostic(conflict.into()));
        }

        // Fail if any config failed (CLI requires all configs to be valid)
        if !errors.is_empty() {
            return Err(CliConfigLoadError::NestedConfigs(errors));
        }

        // Convert loaded configs to nested config format
        let mut nested_ignore_patterns = Vec::with_capacity(configs.len());
        let nested_configs = build_nested_configs(configs, &mut nested_ignore_patterns, None);

        Ok(LoadedConfigs {
            root: oxlintrc,
            nested: nested_configs,
            nested_ignore_patterns,
            warnings,
        })
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
        path.to_string_lossy().cow_replace('\\', "/")
    ))
    .with_help("Run oxlint via the npm package, or use JSON config files (.oxlintrc.json or .oxlintrc.jsonc).")
}

/// Formats a config file path for a diagnostic, relative to the directory the config search
/// started from and with `/` separators.
///
/// An absolute path would make the rendered message wrap at a different column on every machine,
/// which snapshot tests cannot rely on.
fn config_display_path(path: &Path, root: Option<&Path>) -> String {
    let relative = root.and_then(|root| path.strip_prefix(root).ok()).unwrap_or(path);
    let lossy = relative.to_string_lossy();
    lossy.cow_replace('\\', "/").into_owned()
}

fn nested_deny_warnings_not_supported(path: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `options.denyWarnings` option is only supported in the root config. It was found in {path} and is ignored."
    ))
    .with_help("Move `options.denyWarnings` to the root configuration file.")
}

fn nested_type_aware_rules_without_type_aware(path: &str, rules: &[String]) -> OxcDiagnostic {
    let (plural, subject) =
        if rules.len() == 1 { ("", "it never runs") } else { ("s", "they never run") };
    OxcDiagnostic::warn(format!(
        "{path} enables the type-aware rule{plural} {}, but does not set `options.typeAware`, so {subject}.",
        rules.join(", "),
    ))
    .with_help(
        "Set `options.typeAware` to `true` in this config, or `extends` a config which does. It is not inherited from the root config.",
    )
}

/// Warn if `config` type-checks its files without linting them with type-aware rules.
///
/// `typeCheck` and `typeAware` are both resolved from the config which governs a file, and
/// `tsgolint` is only handed the files linted with type-aware rules, so `typeCheck` alone reports
/// nothing. Suppressed when type-aware linting is forced on from the CLI or the editor, which
/// enables it for every file.
fn push_type_check_without_type_aware(
    config: &Config,
    display_path: &str,
    is_root_config: bool,
    warnings: &mut ConfigLoadWarnings,
) {
    if config.type_check() != Some(true) || config.type_aware() == Some(true) {
        return;
    }
    // Advising `extends` of a config which sets the option, or noting that it is not inherited
    // from the root config, only makes sense when there is a config above this one.
    let help = if is_root_config {
        "Set `options.typeAware` to `true` in this config."
    } else {
        "Set `options.typeAware` to `true` in this config, or `extends` a config which does. It is not inherited from the root config."
    };
    warnings.type_check_without_type_aware.push(
        OxcDiagnostic::warn(format!(
            "{display_path} sets `options.typeCheck`, but does not set `options.typeAware`, so no TypeScript diagnostics are reported for its files."
        ))
        .with_help(help),
    );
}

/// The nearest config above `dir` which sets `options.typeCheck`.
///
/// Configs which leave the option unset are skipped, because they are the ones this resolves
/// the value for. The root config governs everything the nested configs do not, so it comes
/// last.
fn nearest_type_check_ancestor<'c>(
    dir: &Path,
    nested_configs: &'c FxHashMap<PathBuf, Config>,
    root: &'c Config,
) -> Option<&'c Config> {
    for ancestor in dir.ancestors().skip(1) {
        if let Some(config) = nested_configs.get(ancestor)
            && config.type_check().is_some()
        {
            return Some(config);
        }
    }
    root.type_check().is_some().then_some(root)
}

/// Warn about the type-aware configs which an ancestor's `options.typeCheck` does *not* reach.
///
/// `typeCheck` is resolved from the config which governs each file and is never inherited, so
/// turning it on in a config leaves every type-aware package below it untouched. That is easy
/// to miss when a config predates per-file type checking, so it is reported once per affected
/// package. A package which sets the option itself, either way, is left alone.
fn push_configs_missing_type_check(
    root: &Config,
    nested_configs: &FxHashMap<PathBuf, Config>,
    root_dir: &Path,
    type_aware_forced: bool,
    warnings: &mut ConfigLoadWarnings,
) {
    let display = |config: &Config| {
        config.path().map_or_else(String::new, |path| config_display_path(path, Some(root_dir)))
    };
    let mut silent: Vec<(String, String)> = nested_configs
        .iter()
        .filter(|(_, config)| {
            // Forcing type-aware linting on hands `tsgolint` every file, so a config which never
            // enabled it also becomes one of the configs an ancestor's `typeCheck` misses.
            (type_aware_forced || config.type_aware() == Some(true))
                && config.type_check().is_none()
        })
        .filter_map(|(dir, config)| {
            let ancestor = nearest_type_check_ancestor(dir, nested_configs, root)?;
            (ancestor.type_check() == Some(true)).then(|| (display(config), display(ancestor)))
        })
        .collect();
    // `nested_configs` is a hash map, so sort to keep the diagnostics in a stable order.
    silent.sort_unstable();
    for (display_path, ancestor_path) in silent {
        warnings.type_check_not_inherited.push(
            OxcDiagnostic::warn(format!(
                "{display_path} does not set `options.typeCheck`, so its files are not type-checked although {ancestor_path} enables it."
            ))
            .with_help(
                "Set `options.typeCheck` to `true` in this config, or share it through `extends`. It is not inherited from the config above it.",
            ),
        );
    }
}

fn nested_max_warnings_not_supported(path: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `options.maxWarnings` option is only supported in the root config. It was found in {path} and is ignored."
    ))
    .with_help("Move `options.maxWarnings` to the root configuration file.")
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::ConfigLoadWarnings;

    use oxc_linter::{ConfigStoreBuilder, ExternalPluginStore, Oxlintrc};

    use super::{ConfigLoadError, ConfigLoader, RunOverrides, build_nested_configs};
    #[cfg(feature = "napi")]
    use crate::js_config::{JsConfigLoaderCb, JsConfigResult};
    use oxc_config::DiscoveredConfigFile;

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

        // Verify the resolved path is normalized, without `.`/`..` components
        if let Ok(config) = result {
            assert_eq!(config.path, cwd.join("fixtures/cli/linter/eslintrc.json"));
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
            let result = loader.load_root_config(&nested_dir, None);
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
        let result = loader.load_root_config(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with explicit path");

        // Test case 3: When no config exists in any ancestor, should return default
        let temp_dir = std::env::temp_dir().join("oxc_test_no_config");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temporary test directory");
        let result = loader.load_root_config(&temp_dir, None);
        assert!(result.is_ok(), "Expected default config when no config found");
        std::fs::remove_dir_all(&temp_dir).expect("Failed to cleanup temporary test directory");
    }

    /// Everything a config load produces for a temporary tree: the configs which survived, the
    /// errors, and the warnings the CLI and the language server would report.
    struct LoadedTree {
        nested_count: usize,
        errors: Vec<ConfigLoadError>,
        warnings: ConfigLoadWarnings,
    }

    fn type_aware_forced() -> RunOverrides {
        RunOverrides { type_aware_forced: true, ..RunOverrides::default() }
    }

    fn type_aware_disabled() -> RunOverrides {
        RunOverrides { type_aware_disabled: true, ..RunOverrides::default() }
    }

    fn type_check_overridden() -> RunOverrides {
        RunOverrides { type_check_overridden: true, ..RunOverrides::default() }
    }

    fn report_count(warnings: &ConfigLoadWarnings, overrides: RunOverrides) -> usize {
        warnings.to_report(overrides).count()
    }

    impl LoadedTree {
        /// The warnings, asserting the tree loaded cleanly first.
        fn unwrap_warnings(self) -> ConfigLoadWarnings {
            assert!(self.errors.is_empty());
            self.warnings
        }
    }

    /// Loads a tree of configs: `root`, if given, plus one config per `(directory, contents)`
    /// pair, and finishes the load the way both front ends do. `type_aware_forced` stands in for
    /// `--type-aware` and the editor's `typeAware: true`.
    ///
    /// A root config is handed to the discovery path as well as built separately, so the
    /// `!is_root_config` guard in `load_many` is exercised and a root problem can only be
    /// reported once. Without one, the root is built from `Oxlintrc::default()` rather than
    /// searched for, so the test never walks out of its temporary directory and picks up a
    /// config which happens to exist on the machine running it.
    fn load_tree(
        root: Option<&str>,
        nested: &[(&str, &str)],
        type_aware_forced: bool,
    ) -> LoadedTree {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.path().join(".oxlintrc.json");
        let mut discovered = Vec::new();
        if let Some(root) = root {
            std::fs::write(&root_path, root).unwrap();
            discovered.push(DiscoveredConfigFile::Json(root_path.clone()));
        }
        for (dir, contents) in nested {
            let nested_path = root_dir.path().join(dir).join(".oxlintrc.json");
            std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
            std::fs::write(&nested_path, contents).unwrap();
            discovered.push(DiscoveredConfigFile::Json(nested_path));
        }

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (configs, errors, warnings) =
            loader.load_discovered_with_root_dir(root_dir.path(), discovered);

        let mut nested_ignore_patterns = Vec::new();
        // The root config is discovered too, but it is the caller which builds and passes it, so
        // drop it from the nested map the way `load_root_and_nested` does.
        let nested_configs: rustc_hash::FxHashMap<_, _> =
            build_nested_configs(configs, &mut nested_ignore_patterns, None)
                .into_iter()
                .filter(|(dir, _)| dir != root_dir.path())
                .collect();
        let nested_count = nested_configs.len();

        let root_oxlintrc = match root {
            Some(_) => ConfigLoader::new(None, &mut external_plugin_store, &[], None)
                .load_root_config(root_dir.path(), Some(&root_path))
                .unwrap(),
            // Not `load_root_config` with no path: that walks up out of the temporary directory.
            None => Oxlintrc::default(),
        };
        let root_config = ConfigStoreBuilder::from_oxlintrc(
            false,
            root_oxlintrc,
            None,
            &mut external_plugin_store,
            None,
        )
        .unwrap()
        .build(&mut external_plugin_store)
        .unwrap();

        let warnings = ConfigLoader::finish_load(
            warnings,
            &root_config,
            &nested_configs,
            root_dir.path(),
            type_aware_forced,
        );
        LoadedTree { nested_count, errors, warnings }
    }

    /// A single nested config, with no root config above it.
    fn load_nested_json(contents: &str) -> (usize, Vec<ConfigLoadError>, ConfigLoadWarnings) {
        let tree = load_tree(None, &[("nested", contents)], false);
        (tree.nested_count, tree.errors, tree.warnings)
    }

    /// The warnings for a tree with a root config, which every caller here expects to load.
    fn tree_warnings(root: &str, nested: &[(&str, &str)]) -> ConfigLoadWarnings {
        load_tree(Some(root), nested, false).unwrap_warnings()
    }

    /// `typeCheck` is not inherited, so turning it on in a config leaves every type-aware package
    /// below it silently un-type-checked. Reported once per package.
    #[test]
    fn test_warns_about_type_aware_configs_without_type_check() {
        let warnings = tree_warnings(
            r#"{ "options": { "typeAware": true, "typeCheck": true } }"#,
            &[("packages/app", r#"{ "options": { "typeAware": true } }"#)],
        );
        assert_eq!(warnings.type_check_not_inherited.len(), 1);
        let warning = &warnings.type_check_not_inherited[0];
        assert!(warning.message.contains("packages/app/.oxlintrc.json"));
        assert!(warning.message.contains("options.typeCheck"));
        // Named after the config which enables it, so a deep tree says which one to look at.
        assert!(warning.message.contains(".oxlintrc.json enables it"));
        // This one is not about type-aware linting being off, so it stays reported when
        // type-aware linting is forced on; forcing it on can only add packages to the list.
        assert_eq!(report_count(&warnings, type_aware_forced()), 1);
        // A `typeCheck` override decides every file itself, so which config would have won no
        // longer matters.
        assert_eq!(report_count(&warnings, type_check_overridden()), 0);
        // With type-aware linting switched off in the editor, nothing is type-checked
        // regardless of what the configs say.
        assert_eq!(report_count(&warnings, type_aware_disabled()), 0);
    }

    /// `--type-aware` and the editor's `typeAware: true` hand `tsgolint` every file, so a package
    /// which never enabled type-aware linting also becomes one an ancestor's `typeCheck` misses.
    #[test]
    fn test_warns_about_configs_made_type_aware_by_the_flag() {
        let nested = &[("packages/app", r#"{ "rules": { "no-debugger": "error" } }"#)][..];
        let root = r#"{ "options": { "typeAware": true, "typeCheck": true } }"#;

        // The package's own config sets neither option, so nothing is missing from it.
        let tree = load_tree(Some(root), nested, false);
        assert!(tree.unwrap_warnings().type_check_not_inherited.is_empty());

        // The flag then makes its files type-aware.
        let tree = load_tree(Some(root), nested, true);
        let warnings = tree.unwrap_warnings();
        assert_eq!(warnings.type_check_not_inherited.len(), 1);
        assert!(
            warnings.type_check_not_inherited[0].message.contains("packages/app/.oxlintrc.json")
        );
    }

    /// The comparison is against the nearest config above which sets the option, not against the
    /// root: a monorepo usually turns the option on in an intermediate package config.
    #[test]
    fn test_warns_against_the_nearest_ancestor_which_sets_type_check() {
        let warnings = tree_warnings(
            r#"{ "options": { "typeAware": true } }"#,
            &[
                ("packages", r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
            ],
        );
        assert_eq!(warnings.type_check_not_inherited.len(), 1);
        let warning = &warnings.type_check_not_inherited[0];
        assert!(warning.message.contains("packages/app/.oxlintrc.json does not set"));
        assert!(warning.message.contains("packages/.oxlintrc.json enables it"));
    }

    /// With no root config at all, the warning still names the configs relative to the directory
    /// the search started from, like every warning raised while loading does.
    #[test]
    fn test_warns_with_paths_relative_to_the_search_root_without_a_root_config() {
        let warnings = load_tree(
            None,
            &[
                ("packages", r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
            ],
            false,
        )
        .unwrap_warnings();

        assert_eq!(warnings.type_check_not_inherited.len(), 1);
        let message = &warnings.type_check_not_inherited[0].message;
        // Relative to the search root, not the temporary directory's absolute path.
        assert!(message.starts_with("packages/app/.oxlintrc.json does not set"), "{message}");
        assert!(message.contains("although packages/.oxlintrc.json enables it"), "{message}");
    }

    /// No ancestor config sets the option, so there is nothing to inherit.
    #[test]
    fn test_does_not_warn_when_no_ancestor_sets_type_check() {
        let warnings = tree_warnings(
            r#"{ "options": { "typeAware": true } }"#,
            &[("packages/app", r#"{ "options": { "typeAware": true } }"#)],
        );
        assert!(warnings.type_check_not_inherited.is_empty());
    }

    /// The nearest ancestor which sets the option sets it to `false`, so the package loses
    /// nothing.
    #[test]
    fn test_does_not_warn_when_the_nearest_ancestor_disables_type_check() {
        let warnings = tree_warnings(
            r#"{ "options": { "typeAware": true, "typeCheck": true } }"#,
            &[
                ("packages", r#"{ "options": { "typeAware": true, "typeCheck": false } }"#),
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
            ],
        );
        assert!(warnings.type_check_not_inherited.is_empty());
    }

    /// A package which sets the option itself, either way, is left alone.
    #[test]
    fn test_does_not_warn_when_the_config_sets_type_check_itself() {
        for nested in [
            r#"{ "options": { "typeAware": true, "typeCheck": true } }"#,
            r#"{ "options": { "typeAware": true, "typeCheck": false } }"#,
        ] {
            let warnings = tree_warnings(
                r#"{ "options": { "typeAware": true, "typeCheck": true } }"#,
                &[("packages/app", nested)],
            );
            assert!(
                warnings.type_check_not_inherited.is_empty(),
                "unexpected warning for {nested}"
            );
        }
    }

    /// A package which is not type-aware is never handed to `tsgolint`, so `typeCheck` was never
    /// going to reach it and there is nothing to migrate.
    #[test]
    fn test_does_not_warn_about_a_config_which_is_not_type_aware() {
        let warnings = tree_warnings(
            r#"{ "options": { "typeAware": true, "typeCheck": true } }"#,
            &[("packages/app", r#"{ "rules": { "no-debugger": "error" } }"#)],
        );
        assert!(warnings.type_check_not_inherited.is_empty());
    }

    /// The root config is built by the caller, so `load_many` must leave it to
    /// `root_config_warnings` rather than warning about it a second time. `tree_warnings` hands
    /// the root config to the discovery path as well, which is what would double it up.
    #[test]
    fn test_root_type_check_without_type_aware_warns_once() {
        let warnings = tree_warnings(
            r#"{ "options": { "typeCheck": true } }"#,
            &[("packages/app", r#"{ "rules": { "no-debugger": "error" } }"#)],
        );
        assert_eq!(warnings.type_check_without_type_aware.len(), 1);
        assert!(warnings.type_check_without_type_aware[0].message.contains(".oxlintrc.json"));
        // The root has no config above it, so `extends` is not the advice to give.
        let help = warnings.type_check_without_type_aware[0].help.as_ref().unwrap();
        assert!(!help.contains("extends"), "root help should not mention `extends`: {help}");
    }

    #[test]
    fn test_nested_json_config_allows_type_aware() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "typeAware": true } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
    }

    #[test]
    fn test_nested_json_config_allows_type_check() {
        // `typeCheck` is resolved per file from the config which governs it, like `typeAware`, so
        // a nested config setting it no longer emits a warning.
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
    }

    /// `typeCheck` without `typeAware` type-checks nothing, because `tsgolint` is only handed
    /// the files linted with type-aware rules.
    #[test]
    fn test_nested_json_config_warns_about_type_check_without_type_aware() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "typeCheck": true } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert_eq!(warnings.type_check_without_type_aware.len(), 1);
        assert!(warnings.type_check_without_type_aware[0].message.contains("options.typeCheck"));
        // A nested config can pick the option up from a shared one, which the root cannot.
        let help = warnings.type_check_without_type_aware[0].help.as_ref().unwrap();
        assert!(help.contains("extends"));
        assert_eq!(warnings.to_report(RunOverrides::default()).count(), 1);
        // `--type-aware` enables type-aware linting for every file, so the option does apply.
        assert_eq!(report_count(&warnings, type_aware_forced()), 0);
        // The editor's `typeAware: false` means no file is linted with type-aware rules.
        assert_eq!(report_count(&warnings, type_aware_disabled()), 0);
        // The fatal "`--type-check` requires type-aware linting" already covers this case.
        assert_eq!(
            report_count(
                &warnings,
                RunOverrides {
                    type_check_without_type_aware_is_fatal: true,
                    ..RunOverrides::default()
                }
            ),
            0
        );
    }

    /// `typeAware: false` leaves the config without type-aware rules, exactly as leaving the
    /// option unset does.
    #[test]
    fn test_nested_json_config_warns_about_type_check_with_type_aware_false() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "typeAware": false, "typeCheck": true } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert_eq!(warnings.type_check_without_type_aware.len(), 1);
    }

    #[test]
    fn test_nested_json_config_allows_type_check_false() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "typeCheck": false } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
    }

    #[test]
    fn test_nested_json_config_warns_about_deny_warnings() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "denyWarnings": true } }"#);
        // The option is ignored, but the rest of the config still applies.
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert_eq!(warnings.always.len(), 1);
        assert!(warnings.always[0].message.contains("options.denyWarnings"));
    }

    #[test]
    fn test_nested_json_config_warns_about_max_warnings() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "maxWarnings": 10 } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert_eq!(warnings.always.len(), 1);
        assert!(warnings.always[0].message.contains("options.maxWarnings"));
    }

    #[test]
    fn test_nested_json_config_allows_report_unused_disable_directives() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "reportUnusedDisableDirectives": "warn" } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
    }

    #[test]
    fn test_nested_json_config_allows_respect_eslint_disable_directives() {
        let (configs, errors, warnings) =
            load_nested_json(r#"{ "options": { "respectEslintDisableDirectives": false } }"#);
        assert_eq!(configs, 1);
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
    }

    /// `options` is not inherited from the root config, so `extends` is how a package picks up a
    /// shared `typeAware`. Returns the `typeAware` of the built nested config.
    fn load_nested_json_extending(base: &str, nested: &str) -> Option<bool> {
        let root_dir = tempfile::tempdir().unwrap();
        let base_path = root_dir.path().join("base/.oxlintrc.json");
        let nested_path = root_dir.path().join("nested/.oxlintrc.json");
        std::fs::create_dir_all(base_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&base_path, base).unwrap();
        std::fs::write(&nested_path, nested).unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (configs, errors, warnings) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert_eq!(configs.len(), 1);
        configs[0].config.type_aware()
    }

    #[test]
    fn test_nested_json_config_allows_type_aware_from_extends() {
        assert_eq!(
            load_nested_json_extending(
                r#"{ "options": { "typeAware": true } }"#,
                r#"{ "extends": ["../base/.oxlintrc.json"] }"#,
            ),
            Some(true)
        );
    }

    #[test]
    fn test_nested_json_config_can_turn_off_extended_type_aware() {
        // The child wins over the config it extends, so a package can opt out of a shared setting.
        assert_eq!(
            load_nested_json_extending(
                r#"{ "options": { "typeAware": true } }"#,
                r#"{ "extends": ["../base/.oxlintrc.json"], "options": { "typeAware": false } }"#,
            ),
            Some(false)
        );
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
    fn test_nested_oxlint_config_ts_allows_type_aware() {
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

        let (configs, errors, warnings) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
        assert_eq!(configs.len(), 1);
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_allows_type_check() {
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

        let (configs, errors, warnings) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(errors.is_empty());
        assert!(warnings.always.is_empty());
        assert!(warnings.type_aware_rules_never_run.is_empty());
        assert!(warnings.type_check_without_type_aware.is_empty());
        assert_eq!(configs.len(), 1);
    }

    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_oxlint_config_ts_warns_about_deny_warnings() {
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

        let (configs, errors, warnings) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(errors.is_empty());
        assert_eq!(configs.len(), 1);
        assert_eq!(warnings.always.len(), 1);
        assert!(warnings.always[0].message.contains("options.denyWarnings"));
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

        let (configs, errors, _warnings) = loader.load_discovered_with_root_dir(
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

        let (configs, errors, _warnings) = loader.load_discovered_with_root_dir(
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

        let (configs, errors, _warnings) = loader.load_discovered_with_root_dir(
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
            discover_configs_in_tree(root_dir.path(), &base_config, &[]).into_iter().collect();

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
            discover_configs_in_tree(root_dir.path(), &base_config, &[]).into_iter().collect();

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
