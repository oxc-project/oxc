use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use bitflags::bitflags;
use cow_utils::CowUtils;
use ignore::DirEntry;

use oxc_config::{
    ConfigConflict, ConfigDiscovery, DiscoveredConfigFile, is_js_config_path, vp_version,
};
use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_linter::{
    Config, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore, LintFilter, Oxlintrc,
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::utils::normalize_path;

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

pub fn config_discovery() -> ConfigDiscovery {
    if cfg!(feature = "napi") && vp_version().is_some() {
        ConfigDiscovery::vite_plus()
    } else {
        ConfigDiscovery::oxlint()
    }
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
/// In Vite+ mode, only `vite.config.*` is discovered.
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
            match discovery.find_unique_config_by_readdir(dir) {
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

/// A problem found while loading configuration files.
#[derive(Debug)]
pub enum ConfigLoadDiagnostic {
    /// A config file could not be loaded and is skipped. The CLI stops, the language server
    /// lints without it.
    Error(ConfigLoadError),
    /// A config file was loaded, and this is only reported.
    Warning(ConfigLoadWarning),
}

impl From<ConfigLoadError> for ConfigLoadDiagnostic {
    fn from(error: ConfigLoadError) -> Self {
        Self::Error(error)
    }
}

impl From<ConfigLoadWarning> for ConfigLoadDiagnostic {
    fn from(warning: ConfigLoadWarning) -> Self {
        Self::Warning(warning)
    }
}

impl ConfigLoadDiagnostic {
    /// A warning, not reported when the run sets any of the `unless` overrides. Every warning is
    /// built here.
    fn warning_unless(diagnostic: OxcDiagnostic, unless: RunOverrides) -> Self {
        ConfigLoadWarning::new(diagnostic, unless).into()
    }

    /// Whether to report this, given what the run decided. Errors are always reported.
    pub fn is_reported(&self, overrides: RunOverrides) -> bool {
        match self {
            Self::Error(_) => true,
            Self::Warning(warning) => warning.is_reported(overrides),
        }
    }

    /// Splits `diagnostics` into the errors and everything else, keeping their order.
    pub fn partition(diagnostics: Vec<Self>) -> (Vec<ConfigLoadError>, Vec<ConfigLoadWarning>) {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        for diagnostic in diagnostics {
            match diagnostic {
                Self::Error(error) => errors.push(error),
                Self::Warning(warning) => warnings.push(warning),
            }
        }
        (errors, warnings)
    }
}

/// A problem in a config file which did not stop it from being used, e.g. an option which is
/// ignored. Its severity is the one of `diagnostic`, a warning or an advice.
#[derive(Debug)]
pub struct ConfigLoadWarning {
    pub diagnostic: OxcDiagnostic,
    /// Not reported when the run sets any of these overrides, which settle what the warning is
    /// about. Empty for a warning which is always reported.
    pub unless: RunOverrides,
}

impl ConfigLoadWarning {
    /// A warning, not reported when the run sets any of the `unless` overrides. Every warning is
    /// built here.
    fn new(diagnostic: OxcDiagnostic, unless: RunOverrides) -> Self {
        debug_assert!(diagnostic.severity != Severity::Error, "a warning cannot be an error");
        Self { diagnostic, unless }
    }

    /// Whether to report this, given what the run decided.
    pub fn is_reported(&self, overrides: RunOverrides) -> bool {
        !overrides.intersects(self.unless)
    }
}

bitflags! {
    /// What the CLI flags, the editor settings or the kind of run decided for every file, whatever
    /// the configs say. Built by [`RunOverrides::for_cli`] and
    /// [`RunOverrides::for_language_server`].
    ///
    /// Also the mask of [`ConfigLoadWarning::unless`]. Which config warning is reported in which
    /// run, where (a) is a nested `denyWarnings` / `maxWarnings`, (b) type-aware rules in a config
    /// which leaves `typeAware` unset, (c) `typeCheck` without `typeAware`, and (d) a type-aware
    /// config which does not inherit the `typeCheck` of the config above it. A run which stops on
    /// a fatal error reports none of them.
    ///
    /// | Run                     | (a) | (b) | (c) | (d) |
    /// | ----------------------- | --- | --- | --- | --- |
    /// | CLI                     | yes | yes | yes | yes |
    /// | CLI `--type-aware`      | yes | no  | no  | yes |
    /// | CLI `--type-check`      | yes | yes | yes | no  |
    /// | CLI `--type-check-only` | yes | no  | no  | no  |
    /// | LSP                     | no  | yes | yes | yes |
    /// | LSP `typeAware: true`   | no  | no  | no  | yes |
    /// | LSP `typeAware: false`  | no  | no  | no  | no  |
    /// | LSP `typeCheck` set     | no  | yes | yes | no  |
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct RunOverrides: u8 {
        /// Whether files are type-checked does not depend on `options.typeCheck`: `--type-check`
        /// and `--type-check-only` type-check every file given to `tsgolint`, and the editor's
        /// `typeCheck` setting decides it either way.
        const TYPE_CHECK_OVERRIDDEN = 1 << 0;
        /// The exit code does not depend on `options.denyWarnings` and `options.maxWarnings`: the
        /// language server has none, and reads neither option.
        const EXIT_CODE_OVERRIDDEN = 1 << 1;
        /// `--type-aware`, `--type-check-only`, or the editor's `typeAware: true`: every file is
        /// linted with type-aware rules.
        const TYPE_AWARE_FORCED = 1 << 2;
        /// The editor's `typeAware: false`: no file is linted with type-aware rules.
        const TYPE_AWARE_DISABLED = 1 << 3;
    }
}

impl RunOverrides {
    /// What a CLI run decided. `type_aware_flag` is `--type-aware` or `--type-check-only`,
    /// `type_check_flag` is `--type-check` or `--type-check-only`.
    pub fn for_cli(type_aware_flag: bool, type_check_flag: bool) -> Self {
        let mut overrides = Self::empty();
        overrides.set(Self::TYPE_AWARE_FORCED, type_aware_flag);
        overrides.set(Self::TYPE_CHECK_OVERRIDDEN, type_check_flag);
        overrides
    }

    /// What a language server run decided, from the editor's `typeAware` and `typeCheck`
    /// settings.
    pub fn for_language_server(type_aware: Option<bool>, type_check: Option<bool>) -> Self {
        let mut overrides = Self::EXIT_CODE_OVERRIDDEN;
        overrides.set(Self::TYPE_AWARE_FORCED, type_aware == Some(true));
        overrides.set(Self::TYPE_AWARE_DISABLED, type_aware == Some(false));
        overrides.set(Self::TYPE_CHECK_OVERRIDDEN, type_check.is_some());
        overrides
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
    /// Problems found while loading nested configs which did not stop the load, e.g. a root-only
    /// option set in a nested config.
    pub warnings: Vec<ConfigLoadWarning>,
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
    ///
    /// Config paths in warnings are relative to `display_root`, or absolute without one.
    fn load_many(
        &mut self,
        paths: impl IntoIterator<Item = DiscoveredConfigFile>,
        root_config_dir: Option<&Path>,
        display_root: Option<&Path>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadDiagnostic>) {
        let mut configs = Vec::new();
        let mut diagnostics = Vec::new();

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

        // Sorted, so the configs, and the problems found in them, come in the same order on every
        // machine: the hash of a path depends on where the project is checked out. The files are
        // read in directory order, JavaScript configs in one batch after the JSON ones, and the
        // configs are then built in directory order whatever their kind.
        let mut by_dir: Vec<_> = by_dir.into_iter().collect();
        by_dir.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        let mut js_configs = Vec::new();

        for (dir, config_files) in by_dir {
            if config_files.len() > 1 {
                diagnostics.push(
                    ConfigLoadError::Diagnostic(
                        ConfigConflict::new(dir.clone(), config_files).into(),
                    )
                    .into(),
                );
                continue;
            }

            match config_files.into_iter().next() {
                Some(DiscoveredConfigFile::Json(path) | DiscoveredConfigFile::Jsonc(path)) => {
                    match Self::load(path.as_path()) {
                        Ok(config) => configs.push(config),
                        Err(e) => diagnostics.push(e.into()),
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
                // Both kinds are already in directory order, the order `by_dir` gives, so this
                // stable sort only merges two runs. By directory rather than by file path, which
                // can disagree: `a/-x/.oxlintrc.json` comes before `a/.oxlintrc.json`.
                configs.sort_by(|a, b| a.path.parent().cmp(&b.path.parent()));
            }
            Err(js_errors) => {
                diagnostics.extend(js_errors.into_iter().map(ConfigLoadDiagnostic::from));
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

            // `denyWarnings` and `maxWarnings` control the process exit code, which is a property
            // of the whole run and cannot be scoped to a directory. Every other option in
            // `options` is resolved per file from the config which governs it, so nested configs
            // may set them freely.
            //
            // A nested config which sets one of the two remaining root-only options used to be
            // dropped entirely, which silently removed every diagnostic for that package. Warn
            // and ignore just the offending option instead. A value inherited through `extends`
            // is ignored without a warning, which would otherwise fire for every package of a
            // shared base.
            if !is_root_config {
                // Only computed for a config which is warned about.
                let display_path = || config_display_path(&path, display_root);
                if config.options.deny_warnings.take().is_some() {
                    diagnostics.push(ConfigLoadDiagnostic::warning_unless(
                        nested_deny_warnings_not_supported(&display_path()),
                        RunOverrides::EXIT_CODE_OVERRIDDEN,
                    ));
                }
                if config.options.max_warnings.take().is_some() {
                    diagnostics.push(ConfigLoadDiagnostic::warning_unless(
                        nested_max_warnings_not_supported(&display_path()),
                        RunOverrides::EXIT_CODE_OVERRIDDEN,
                    ));
                }
            }

            // Captured before the config is consumed, to warn below if these rules can never run.
            // Only nested configs are checked: this is where dropping the implicit inheritance from
            // the root config changes behaviour. Skipped when the config sets `typeAware` itself,
            // either way: its own value wins over any config it extends, and `false` is a
            // deliberate opt-out.
            let explicit_type_aware_rules = if is_root_config || config.options.type_aware.is_some()
            {
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
                    diagnostics.push(ConfigLoadError::Build { path, error: e.to_string() }.into());
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
                    // rules without setting the option, itself or through `extends`, silently
                    // runs none of them. An explicit `false` is a deliberate opt-out.
                    if !explicit_type_aware_rules.is_empty() && config.type_aware().is_none() {
                        // Not reported when the run turns type-aware linting on or off for every
                        // file, whatever the configs say.
                        diagnostics.push(ConfigLoadDiagnostic::warning_unless(
                            nested_type_aware_rules_without_type_aware(
                                &config_display_path(&path, display_root),
                                &explicit_type_aware_rules,
                            ),
                            RunOverrides::TYPE_AWARE_FORCED | RunOverrides::TYPE_AWARE_DISABLED,
                        ));
                    }
                    // `typeCheck` is scoped the same way, and `tsgolint` only ever sees the files
                    // linted with type-aware rules. The root config is built by the caller, so
                    // `finish_load` covers that one.
                    if !is_root_config
                        && let Some(warning) = type_check_without_type_aware(&config, false, || {
                            config_display_path(&path, display_root)
                        })
                    {
                        diagnostics.push(warning.into());
                    }
                    built_configs.push(LoadedConfig {
                        dir,
                        config,
                        ignore_patterns,
                        extended_paths,
                    });
                }
                Err(e) => diagnostics.push(e.into()),
            }
        }

        (built_configs, diagnostics)
    }

    pub(crate) fn load_discovered_with_root_dir(
        &mut self,
        root_dir: &Path,
        configs: impl IntoIterator<Item = DiscoveredConfigFile>,
    ) -> (Vec<LoadedConfig>, Vec<ConfigLoadDiagnostic>) {
        // A language server may serve several workspace folders, so their warnings name config
        // files by their absolute path.
        self.load_many(configs, Some(root_dir), None)
    }

    /// The warnings which need the root config, or the whole set of configs at once.
    ///
    /// The root config is the one config `ConfigLoader` does not build itself: the caller does,
    /// after resolving `extends`. Both the CLI and the language server call this once they have
    /// built it, and report these warnings along with the ones found while loading, so an editor
    /// reports the same problems the CLI does, once each. Config paths are relative to
    /// `display_root`, or absolute without one, like the warnings found while loading.
    ///
    /// `type_aware_forced` says whether `--type-aware`, `--type-check-only` or the editor's
    /// `typeAware` setting makes *every* file type-aware, which decides how many configs the
    /// `typeCheck` of a config above them was going to reach.
    #[must_use]
    pub fn finish_load(
        root: &Config,
        nested_configs: &FxHashMap<PathBuf, Config>,
        display_root: Option<&Path>,
        type_aware_forced: bool,
    ) -> Vec<ConfigLoadWarning> {
        let mut warnings = Vec::new();
        if let Some(root_path) = root.path()
            && let Some(warning) = type_check_without_type_aware(root, true, || {
                config_display_path(root_path, display_root)
            })
        {
            warnings.push(warning);
        }
        warnings.extend(configs_missing_type_check(
            root,
            nested_configs,
            display_root,
            type_aware_forced,
        ));
        warnings
    }

    /// Try to load config from a specific directory.
    ///
    /// Checks the names from [`config_discovery`].
    ///
    /// Returns `Ok(Some(config))` if found, `Ok(None)` if not found, or `Err` on error.
    fn try_load_config_from_dir(
        &self,
        discovery: &ConfigDiscovery,
        dir: &Path,
    ) -> Result<Option<Oxlintrc>, OxcDiagnostic> {
        let config_file =
            discovery.find_unique_config_by_readdir(dir).map_err(OxcDiagnostic::from)?;

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
                warnings: Vec::new(),
            });
        }

        // Discover config files by walking up from each file's directory
        let config_paths: Vec<_> =
            paths.iter().map(|p| Path::new(p.as_ref()).to_path_buf()).collect();
        let (discovered_configs, conflicts) =
            discover_configs_in_ancestors(&config_paths, &oxlintrc.path);

        let (configs, mut diagnostics) = self.load_many(discovered_configs, Some(cwd), Some(cwd));

        // Propagate upstream conflicts as load errors alongside parse/build failures.
        for conflict in conflicts {
            diagnostics.push(ConfigLoadError::Diagnostic(conflict.into()).into());
        }

        // Fail if any config failed (CLI requires all configs to be valid)
        let (errors, warnings) = ConfigLoadDiagnostic::partition(diagnostics);
        if !errors.is_empty() {
            // The run stops here, so the warnings are dropped.
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

/// Formats a config file path for a diagnostic, relative to `root` if given, with `/` separators.
///
/// The CLI passes the directory the config search started from: an absolute path would make the
/// rendered message wrap at a different column on every machine, which snapshot tests cannot rely
/// on.
pub fn config_display_path(path: &Path, root: Option<&Path>) -> String {
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

/// Warn if `config` type-checks its files without linting them with type-aware rules.
///
/// `typeCheck` and `typeAware` are both resolved from the config which governs a file, and
/// `tsgolint` is only handed the files linted with type-aware rules, so `typeCheck` alone reports
/// nothing. Not reported when the run turns type-aware linting on or off for every file.
fn type_check_without_type_aware(
    config: &Config,
    is_root_config: bool,
    display_path: impl FnOnce() -> String,
) -> Option<ConfigLoadWarning> {
    if config.type_check() != Some(true) || config.type_aware() == Some(true) {
        return None;
    }
    // Advising `extends` of a config which sets the option, or noting that it is not inherited
    // from the root config, only makes sense when there is a config above this one.
    let help = if is_root_config {
        "Set `options.typeAware` to `true` in this config."
    } else {
        "Set `options.typeAware` to `true` in this config, or `extends` a config which does. It is not inherited from the root config."
    };
    let display_path = display_path();
    Some(ConfigLoadWarning::new(
        OxcDiagnostic::warn(format!(
            "{display_path} sets `options.typeCheck`, but does not set `options.typeAware`, so no TypeScript diagnostics are reported for its files."
        ))
        .with_help(help),
        RunOverrides::TYPE_AWARE_FORCED | RunOverrides::TYPE_AWARE_DISABLED,
    ))
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
/// package. A package which sets the option itself, either way, is left alone. Not reported
/// when the run decides type checking for every file, or turns type-aware linting off.
fn configs_missing_type_check(
    root: &Config,
    nested_configs: &FxHashMap<PathBuf, Config>,
    display_root: Option<&Path>,
    type_aware_forced: bool,
) -> Vec<ConfigLoadWarning> {
    // Only the root config can lack a file.
    let display = |config: &Config| {
        config.path().map_or_else(
            || "the root config".to_string(),
            |path| config_display_path(path, display_root),
        )
    };
    let mut missing: Vec<(String, String)> = nested_configs
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
    // `nested_configs` is a hash map, so sort to keep the warnings in a stable order.
    missing.sort_unstable();
    missing
        .into_iter()
        .map(|(display_path, ancestor_path)| {
            ConfigLoadWarning::new(
                OxcDiagnostic::warn(format!(
                    "{display_path} does not set `options.typeCheck`, so its files are not type-checked although {ancestor_path} enables it."
                ))
                .with_help(
                    "Set `options.typeCheck` to `true` in this config, or share it through `extends`. It is not inherited from the config above it.",
                ),
                RunOverrides::TYPE_CHECK_OVERRIDDEN | RunOverrides::TYPE_AWARE_DISABLED,
            )
        })
        .collect()
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

fn nested_max_warnings_not_supported(path: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `options.maxWarnings` option is only supported in the root config. It was found in {path} and is ignored."
    ))
    .with_help("Move `options.maxWarnings` to the root configuration file.")
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use oxc_linter::{ConfigStoreBuilder, ExternalPluginStore, Oxlintrc};
    use rustc_hash::FxHashMap;

    use super::{
        ConfigLoadDiagnostic, ConfigLoadError, ConfigLoader, RunOverrides, build_nested_configs,
    };
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

    /// Problems come in the order of the config paths, whatever order the configs are found in.
    #[test]
    fn test_nested_configs_are_loaded_in_path_order() {
        let root_dir = tempfile::tempdir().unwrap();
        let names = ["b", "c", "a", "e", "d"];
        let paths: Vec<PathBuf> =
            names.iter().map(|name| root_dir.path().join(name).join(".oxlintrc.json")).collect();
        for path in &paths {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "{ invalid").unwrap();
        }

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (_configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            paths.into_iter().map(DiscoveredConfigFile::Json),
        );
        let error_paths: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| match diagnostic {
                ConfigLoadDiagnostic::Error(ConfigLoadError::Parse { path, .. }) => {
                    path.strip_prefix(root_dir.path()).unwrap()
                }
                diagnostic => panic!("unexpected diagnostic: {diagnostic:?}"),
            })
            .collect();
        let expected: Vec<PathBuf> = ["a", "b", "c", "d", "e"]
            .iter()
            .map(|name| PathBuf::from(name).join(".oxlintrc.json"))
            .collect();
        assert_eq!(error_paths, expected);
    }

    /// JavaScript configs are read in a batch after the JSON ones, but built, and so reported, in
    /// directory order with them.
    #[cfg(feature = "napi")]
    #[test]
    fn test_nested_js_and_json_configs_are_built_in_path_order() {
        let root_dir = tempfile::tempdir().unwrap();
        let invalid_rules = serde_json::json!({ "no-such-rule": "error" });
        let mut discovered = Vec::new();
        // `a/-x` sorts before `a` by file path, since `-` comes before `.`, but after it by
        // directory, the order the files were read in.
        for (name, is_js) in [("c", true), ("b", false), ("a/-x", false), ("a", true)] {
            let dir = root_dir.path().join(name);
            std::fs::create_dir_all(&dir).unwrap();
            if is_js {
                let path = dir.join("oxlint.config.ts");
                std::fs::write(&path, "export default {};").unwrap();
                discovered.push(DiscoveredConfigFile::Js(path));
            } else {
                let path = dir.join(".oxlintrc.json");
                std::fs::write(&path, r#"{ "rules": { "no-such-rule": "error" } }"#).unwrap();
                discovered.push(DiscoveredConfigFile::Json(path));
            }
        }

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let js_loader = make_js_loader(move |paths| {
            Ok(paths
                .into_iter()
                .map(|path| make_js_config_with_rules(PathBuf::from(path), &invalid_rules))
                .collect())
        });
        loader = loader.with_js_config_loader(Some(&js_loader));

        let (_configs, diagnostics) =
            loader.load_discovered_with_root_dir(root_dir.path(), discovered);
        let error_paths: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| match diagnostic {
                ConfigLoadDiagnostic::Error(ConfigLoadError::Build { path, .. }) => {
                    path.strip_prefix(root_dir.path()).unwrap()
                }
                diagnostic => panic!("unexpected diagnostic: {diagnostic:?}"),
            })
            .collect();
        let expected: Vec<PathBuf> =
            ["a/oxlint.config.ts", "a/-x/.oxlintrc.json", "b/.oxlintrc.json", "c/oxlint.config.ts"]
                .iter()
                .map(PathBuf::from)
                .collect();
        assert_eq!(error_paths, expected);
    }

    /// Loads a single nested config from a temporary directory and returns
    /// `(number of configs, diagnostics)`.
    fn load_nested_json(contents: &str) -> (usize, Vec<ConfigLoadDiagnostic>) {
        let root_dir = tempfile::tempdir().unwrap();
        let nested_path = root_dir.path().join("nested/.oxlintrc.json");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, contents).unwrap();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut loader = ConfigLoader::new(None, &mut external_plugin_store, &[], None);
        let (configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        (configs.len(), diagnostics)
    }

    /// The messages of `diagnostics`, asserting that they are all warnings.
    fn warning_messages(diagnostics: &[ConfigLoadDiagnostic]) -> Vec<&str> {
        diagnostics
            .iter()
            .map(|diagnostic| match diagnostic {
                ConfigLoadDiagnostic::Warning(warning) => warning.diagnostic.message.as_ref(),
                ConfigLoadDiagnostic::Error(error) => panic!("unexpected error: {error:?}"),
            })
            .collect()
    }

    #[test]
    fn test_nested_json_config_allows_type_aware() {
        let (configs, diagnostics) = load_nested_json(r#"{ "options": { "typeAware": true } }"#);
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty());
    }

    /// Loads a tree of configs: `root`, if given, plus one config per `(directory, contents)`
    /// pair, and finishes the load the way both front ends do. `type_aware_forced` stands in for
    /// `--type-aware` and the editor's `typeAware: true`. Returns the number of nested configs and
    /// every diagnostic, those found while loading followed by those of `finish_load`, with
    /// config paths relative to the tree.
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
    ) -> (usize, Vec<ConfigLoadDiagnostic>) {
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
        let (configs, mut diagnostics) =
            loader.load_many(discovered, Some(root_dir.path()), Some(root_dir.path()));

        let mut nested_ignore_patterns = Vec::new();
        // The root config is discovered too, but it is the caller which builds and passes it, so
        // drop it from the nested map the way `load_root_and_nested` does.
        let nested_configs: FxHashMap<_, _> =
            build_nested_configs(configs, &mut nested_ignore_patterns, None)
                .into_iter()
                .filter(|(dir, _)| dir != root_dir.path())
                .collect();

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

        diagnostics.extend(
            ConfigLoader::finish_load(
                &root_config,
                &nested_configs,
                Some(root_dir.path()),
                type_aware_forced,
            )
            .into_iter()
            .map(ConfigLoadDiagnostic::from),
        );
        (nested_configs.len(), diagnostics)
    }

    /// The help of the only warning in `diagnostics`.
    fn only_warning_help(diagnostics: &[ConfigLoadDiagnostic]) -> String {
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        let ConfigLoadDiagnostic::Warning(warning) = &diagnostics[0] else {
            panic!("expected a warning: {diagnostics:?}");
        };
        warning.diagnostic.help.as_deref().unwrap().to_string()
    }

    /// A nested config may set `typeCheck`: it is resolved per file from the config which governs
    /// it, like `typeAware`.
    #[test]
    fn test_nested_json_config_allows_type_check() {
        let (configs, diagnostics) =
            load_nested_json(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#);
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// `typeCheck` without `typeAware` type-checks nothing, because `tsgolint` is only handed
    /// the files linted with type-aware rules.
    #[test]
    fn test_nested_json_config_warns_about_type_check_without_type_aware() {
        let (configs, diagnostics) = load_nested_json(r#"{ "options": { "typeCheck": true } }"#);
        assert_eq!(configs, 1);
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("sets `options.typeCheck`"), "{}", messages[0]);
        // A nested config can pick the option up from a shared one, which the root cannot.
        assert!(only_warning_help(&diagnostics).contains("extends"));
    }

    /// `typeAware: false` leaves the config without type-aware rules, exactly as leaving the
    /// option unset does.
    #[test]
    fn test_nested_json_config_warns_about_type_check_with_type_aware_false() {
        let (configs, diagnostics) =
            load_nested_json(r#"{ "options": { "typeAware": false, "typeCheck": true } }"#);
        assert_eq!(configs, 1);
        assert_eq!(warning_messages(&diagnostics).len(), 1);
    }

    /// The root config is built by the caller, so `load_many` must leave it to `finish_load`
    /// rather than warning about it a second time. `load_tree` hands the root config to the
    /// discovery path as well, which is what would double it up.
    #[test]
    fn test_root_type_check_without_type_aware_warns_once() {
        let (_, diagnostics) = load_tree(
            Some(r#"{ "options": { "typeCheck": true } }"#),
            &[("packages/app", r#"{ "rules": { "no-debugger": "error" } }"#)],
            false,
        );
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(
            messages[0].starts_with(".oxlintrc.json sets `options.typeCheck`"),
            "{}",
            messages[0]
        );
        // The root has no config above it, so `extends` is not the advice to give.
        let help = only_warning_help(&diagnostics);
        assert!(!help.contains("extends"), "root help should not mention `extends`: {help}");
    }

    /// `typeCheck` is not inherited, so turning it on in a config leaves every type-aware package
    /// below it silently un-type-checked. Reported once per package, against the config which
    /// enables it, so a deep tree says which one to look at.
    #[test]
    fn test_warns_about_type_aware_configs_without_type_check() {
        let (nested, diagnostics) = load_tree(
            Some(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
            &[
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
                ("packages/lib", r#"{ "options": { "typeAware": true } }"#),
            ],
            false,
        );
        assert_eq!(nested, 2);
        assert_eq!(
            warning_messages(&diagnostics),
            [
                "packages/app/.oxlintrc.json does not set `options.typeCheck`, so its files are not type-checked although .oxlintrc.json enables it.",
                "packages/lib/.oxlintrc.json does not set `options.typeCheck`, so its files are not type-checked although .oxlintrc.json enables it.",
            ]
        );
    }

    /// `--type-aware` and the editor's `typeAware: true` hand `tsgolint` every file, so a package
    /// which never enabled type-aware linting also becomes one an ancestor's `typeCheck` misses.
    #[test]
    fn test_warns_about_configs_made_type_aware_by_the_flag() {
        let nested = &[("packages/app", r#"{ "rules": { "no-debugger": "error" } }"#)][..];
        let root = Some(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#);

        // The package's own config sets neither option, so nothing is missing from it.
        let (_, diagnostics) = load_tree(root, nested, false);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");

        // The flag then makes its files type-aware.
        let (_, diagnostics) = load_tree(root, nested, true);
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("packages/app/.oxlintrc.json does not set"));
    }

    /// The comparison is against the nearest config above which sets the option, not against the
    /// root: a monorepo usually turns the option on in an intermediate package config.
    #[test]
    fn test_warns_against_the_nearest_ancestor_which_sets_type_check() {
        let (_, diagnostics) = load_tree(
            Some(r#"{ "options": { "typeAware": true } }"#),
            &[
                ("packages", r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
            ],
            false,
        );
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("packages/app/.oxlintrc.json does not set"));
        assert!(messages[0].contains("although packages/.oxlintrc.json enables it"));
    }

    /// With no root config file, an intermediate config which enables the option is still the
    /// one compared against.
    #[test]
    fn test_warns_without_a_root_config() {
        let (_, diagnostics) = load_tree(
            None,
            &[
                ("packages", r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
            ],
            false,
        );
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("packages/app/.oxlintrc.json does not set"));
        assert!(messages[0].contains("although packages/.oxlintrc.json enables it"));
    }

    /// No ancestor config sets the option, so there is nothing to inherit.
    #[test]
    fn test_does_not_warn_when_no_ancestor_sets_type_check() {
        let (_, diagnostics) = load_tree(
            Some(r#"{ "options": { "typeAware": true } }"#),
            &[("packages/app", r#"{ "options": { "typeAware": true } }"#)],
            false,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// The nearest ancestor which sets the option sets it to `false`, so the package loses
    /// nothing.
    #[test]
    fn test_does_not_warn_when_the_nearest_ancestor_disables_type_check() {
        let (_, diagnostics) = load_tree(
            Some(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
            &[
                ("packages", r#"{ "options": { "typeAware": true, "typeCheck": false } }"#),
                ("packages/app", r#"{ "options": { "typeAware": true } }"#),
            ],
            false,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// A package which sets the option itself, either way, is left alone.
    #[test]
    fn test_does_not_warn_when_the_config_sets_type_check_itself() {
        for nested in [
            r#"{ "options": { "typeAware": true, "typeCheck": true } }"#,
            r#"{ "options": { "typeAware": true, "typeCheck": false } }"#,
        ] {
            let (_, diagnostics) = load_tree(
                Some(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
                &[("packages/app", nested)],
                false,
            );
            assert!(diagnostics.is_empty(), "unexpected warning for {nested}: {diagnostics:?}");
        }
    }

    /// A package which is not type-aware is never handed to `tsgolint`, so `typeCheck` was never
    /// going to reach it and there is nothing to migrate.
    #[test]
    fn test_does_not_warn_about_a_config_which_is_not_type_aware() {
        let (_, diagnostics) = load_tree(
            Some(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
            &[("packages/app", r#"{ "rules": { "no-debugger": "error" } }"#)],
            false,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// Which warning is reported in which kind of run, as documented on [`RunOverrides`]. A new
    /// warning goes in this table, with the runs where it is true and can be acted on.
    #[test]
    fn test_warnings_by_run() {
        let runs = [
            ("CLI", RunOverrides::for_cli(false, false)),
            ("CLI --type-aware", RunOverrides::for_cli(true, false)),
            ("CLI --type-check", RunOverrides::for_cli(false, true)),
            ("CLI --type-check-only", RunOverrides::for_cli(true, true)),
            ("language server", RunOverrides::for_language_server(None, None)),
            (
                "language server, typeAware: true",
                RunOverrides::for_language_server(Some(true), None),
            ),
            (
                "language server, typeAware: false",
                RunOverrides::for_language_server(Some(false), None),
            ),
            (
                "language server, typeCheck set",
                RunOverrides::for_language_server(None, Some(false)),
            ),
        ];
        // (warning, root config, nested config, reported in each run)
        let warnings = [
            (
                "(a) denyWarnings",
                None,
                r#"{ "options": { "denyWarnings": true } }"#,
                [true, true, true, true, false, false, false, false],
            ),
            (
                "(a) maxWarnings",
                None,
                r#"{ "options": { "maxWarnings": 1 } }"#,
                [true, true, true, true, false, false, false, false],
            ),
            (
                "(b) type-aware rules, typeAware unset",
                None,
                r#"{ "rules": { "typescript/no-floating-promises": "error" } }"#,
                [true, false, true, false, true, false, false, true],
            ),
            (
                "(c) typeCheck without typeAware",
                None,
                r#"{ "options": { "typeCheck": true } }"#,
                [true, false, true, false, true, false, false, true],
            ),
            (
                "(d) typeCheck not inherited",
                Some(r#"{ "options": { "typeAware": true, "typeCheck": true } }"#),
                r#"{ "options": { "typeAware": true } }"#,
                [true, true, false, false, true, true, false, false],
            ),
        ];
        for (warning, root, nested, expected) in warnings {
            let (_, diagnostics) = load_tree(root, &[("nested", nested)], false);
            assert_eq!(warning_messages(&diagnostics).len(), 1, "{warning}");
            for ((run, overrides), expected) in runs.iter().zip(expected) {
                assert_eq!(diagnostics[0].is_reported(*overrides), expected, "{warning} in {run}");
            }
        }
    }

    #[test]
    fn test_nested_json_config_does_not_warn_about_type_check_false() {
        let (configs, diagnostics) = load_nested_json(r#"{ "options": { "typeCheck": false } }"#);
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_nested_json_config_warns_about_deny_warnings() {
        let (configs, diagnostics) = load_nested_json(r#"{ "options": { "denyWarnings": true } }"#);
        // The option is ignored, but the rest of the config still applies.
        assert_eq!(configs, 1);
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("options.denyWarnings"));
    }

    /// Naming type-aware rules without enabling `options.typeAware` is reported, unless the run
    /// turns type-aware linting on (the rules run after all) or off (the editor's choice) for
    /// every file.
    #[test]
    fn test_nested_json_config_warns_about_type_aware_rules_without_type_aware() {
        let (configs, diagnostics) =
            load_nested_json(r#"{ "rules": { "typescript/no-floating-promises": "error" } }"#);
        assert_eq!(configs, 1);
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("typescript/no-floating-promises"));
        assert!(messages[0].contains("does not set `options.typeAware`"));
        assert!(diagnostics[0].is_reported(RunOverrides::default()));
        assert!(!diagnostics[0].is_reported(RunOverrides::TYPE_AWARE_FORCED));
        assert!(!diagnostics[0].is_reported(RunOverrides::TYPE_AWARE_DISABLED));
    }

    /// An explicit `typeAware: false`, in the config or in one it extends, is a deliberate opt-out.
    #[test]
    fn test_nested_json_config_does_not_warn_about_type_aware_rules_with_type_aware_false() {
        let (configs, diagnostics) = load_nested_json(
            r#"{ "options": { "typeAware": false }, "rules": { "typescript/no-floating-promises": "error" } }"#,
        );
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty());

        // `load_nested_json_extending` asserts that nothing is reported.
        assert_eq!(
            load_nested_json_extending(
                r#"{ "options": { "typeAware": false } }"#,
                r#"{ "extends": ["../base/.oxlintrc.json"], "rules": { "typescript/no-floating-promises": "error" } }"#,
            ),
            Some(false)
        );
    }

    #[test]
    fn test_nested_json_config_does_not_warn_about_type_aware_rules_with_type_aware() {
        let (configs, diagnostics) = load_nested_json(
            r#"{ "options": { "typeAware": true }, "rules": { "typescript/no-floating-promises": "error" } }"#,
        );
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_nested_json_config_warns_about_max_warnings() {
        let (configs, diagnostics) = load_nested_json(r#"{ "options": { "maxWarnings": 10 } }"#);
        assert_eq!(configs, 1);
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("options.maxWarnings"));
    }

    #[test]
    fn test_nested_json_config_allows_report_unused_disable_directives() {
        let (configs, diagnostics) =
            load_nested_json(r#"{ "options": { "reportUnusedDisableDirectives": "warn" } }"#);
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_nested_json_config_allows_respect_eslint_disable_directives() {
        let (configs, diagnostics) =
            load_nested_json(r#"{ "options": { "respectEslintDisableDirectives": false } }"#);
        assert_eq!(configs, 1);
        assert!(diagnostics.is_empty());
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
        let (configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Json(nested_path)],
        );
        assert!(diagnostics.is_empty());
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

        let (configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(diagnostics.is_empty());
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

        let (configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert!(diagnostics.is_empty());
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

        let (configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert_eq!(configs.len(), 1);
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("options.denyWarnings"));
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

        let (configs, diagnostics) = loader.load_discovered_with_root_dir(
            root_dir.path(),
            [DiscoveredConfigFile::Js(nested_path)],
        );
        assert_eq!(configs.len(), 1);
        // A value set through `extends` counts as the config's own, so it is warned about like
        // one: the extended config does not set `typeAware` either.
        let messages = warning_messages(&diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("sets `options.typeCheck`"), "{}", messages[0]);
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
