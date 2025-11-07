use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use ignore::gitignore::Gitignore;
use log::{debug, warn};
use oxc_linter::{AllowWarnDeny, FixKind, LintIgnoreMatcher};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::{Diagnostic, Pattern, Uri};

use oxc_linter::{
    Config, ConfigStore, ConfigStoreBuilder, ExternalPluginStore, LintOptions, Oxlintrc,
};
use tower_lsp_server::UriExt;

use crate::{
    ConcurrentHashMap, LINT_CONFIG_FILE,
    linter::{
        config_walker::ConfigWalker,
        error_with_position::DiagnosticReport,
        isolated_lint_handler::{IsolatedLintHandler, IsolatedLintHandlerOptions},
        options::{LintOptions as LSPLintOptions, Run, UnusedDisableDirectives},
    },
    utils::normalize_path,
    worker::ToolRestartChanges,
};

pub struct ServerLinterBuilder {
    root_uri: Uri,
    options: LSPLintOptions,
}

impl ServerLinterBuilder {
    pub fn new(root_uri: Uri, options: serde_json::Value) -> Self {
        let options = match serde_json::from_value::<LSPLintOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };
        Self { root_uri, options }
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(&self) -> ServerLinter {
        let root_path = self.root_uri.to_file_path().unwrap();
        let mut nested_ignore_patterns = Vec::new();
        let (nested_configs, mut extended_paths) =
            Self::create_nested_configs(&root_path, &self.options, &mut nested_ignore_patterns);
        let config_path = self.options.config_path.as_ref().map_or(LINT_CONFIG_FILE, |v| v);
        let config = normalize_path(root_path.join(config_path));
        let oxlintrc = if config.try_exists().is_ok_and(|exists| exists) {
            if let Ok(oxlintrc) = Oxlintrc::from_file(&config) {
                oxlintrc
            } else {
                warn!("Failed to initialize oxlintrc config: {}", config.to_string_lossy());
                Oxlintrc::default()
            }
        } else {
            warn!(
                "Config file not found: {}, fallback to default config",
                config.to_string_lossy()
            );
            Oxlintrc::default()
        };

        let base_patterns = oxlintrc.ignore_patterns.clone();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let config_builder =
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc, None, &mut external_plugin_store)
                .unwrap_or_default();

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = self.options.use_nested_configs();
        let fix_kind = FixKind::from(self.options.fix_kind.clone());

        let use_cross_module = config_builder.plugins().has_import()
            || (use_nested_config
                && nested_configs.pin().values().any(|config| config.plugins().has_import()));

        extended_paths.extend(config_builder.extended_paths.clone());
        let base_config = config_builder.build(&external_plugin_store).unwrap_or_else(|err| {
            warn!("Failed to build config: {err}");
            ConfigStoreBuilder::empty().build(&external_plugin_store).unwrap()
        });

        let lint_options = LintOptions {
            fix: fix_kind,
            report_unused_directive: match self.options.unused_disable_directives {
                UnusedDisableDirectives::Allow => None, // or AllowWarnDeny::Allow, should be the same?
                UnusedDisableDirectives::Warn => Some(AllowWarnDeny::Warn),
                UnusedDisableDirectives::Deny => Some(AllowWarnDeny::Deny),
            },
            ..Default::default()
        };
        let config_store = ConfigStore::new(
            base_config,
            if use_nested_config {
                let nested_configs = nested_configs.pin();
                nested_configs
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<FxHashMap<_, _>>()
            } else {
                FxHashMap::default()
            },
            external_plugin_store,
        );

        let isolated_linter = IsolatedLintHandler::new(
            lint_options,
            config_store,
            &IsolatedLintHandlerOptions {
                use_cross_module,
                type_aware: self.options.type_aware,
                fix_kind: FixKind::from(self.options.fix_kind.clone()),
                root_path: root_path.to_path_buf(),
                tsconfig_path: self.options.ts_config_path.as_ref().map(|path| {
                    let path = Path::new(path).to_path_buf();
                    if path.is_relative() { root_path.join(path) } else { path }
                }),
            },
        );

        ServerLinter::new(
            self.options.run,
            Arc::new(Mutex::new(isolated_linter)),
            LintIgnoreMatcher::new(&base_patterns, &root_path, nested_ignore_patterns),
            Self::create_ignore_glob(&root_path),
            extended_paths,
        )
    }

    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    fn create_nested_configs(
        root_path: &Path,
        options: &LSPLintOptions,
        nested_ignore_patterns: &mut Vec<(Vec<String>, PathBuf)>,
    ) -> (ConcurrentHashMap<PathBuf, Config>, FxHashSet<PathBuf>) {
        let mut extended_paths = FxHashSet::default();
        // nested config is disabled, no need to search for configs
        if !options.use_nested_configs() {
            return (ConcurrentHashMap::default(), extended_paths);
        }

        let paths = ConfigWalker::new(root_path).paths();
        let nested_configs =
            ConcurrentHashMap::with_capacity_and_hasher(paths.capacity(), FxBuildHasher);

        for path in paths {
            let file_path = Path::new(&path);
            let Some(dir_path) = file_path.parent() else {
                continue;
            };

            let Ok(oxlintrc) = Oxlintrc::from_file(file_path) else {
                warn!("Skipping invalid config file: {}", file_path.display());
                continue;
            };
            // Collect ignore patterns and their root
            nested_ignore_patterns.push((oxlintrc.ignore_patterns.clone(), dir_path.to_path_buf()));
            let mut external_plugin_store = ExternalPluginStore::new(false);
            let Ok(config_store_builder) = ConfigStoreBuilder::from_oxlintrc(
                false,
                oxlintrc,
                None,
                &mut external_plugin_store,
            ) else {
                warn!("Skipping config (builder failed): {}", file_path.display());
                continue;
            };
            extended_paths.extend(config_store_builder.extended_paths.clone());
            let config = config_store_builder.build(&external_plugin_store).unwrap_or_else(|err| {
                warn!("Failed to build nested config for {}: {:?}", dir_path.display(), err);
                ConfigStoreBuilder::empty().build(&external_plugin_store).unwrap()
            });
            nested_configs.pin().insert(dir_path.to_path_buf(), config);
        }

        (nested_configs, extended_paths)
    }

    #[expect(clippy::filetype_is_file)]
    fn create_ignore_glob(root_path: &Path) -> Vec<Gitignore> {
        let walk = ignore::WalkBuilder::new(root_path)
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .build()
            .flatten();

        let mut gitignore_globs = vec![];
        for entry in walk {
            if !entry.file_type().is_some_and(|v| v.is_file()) {
                continue;
            }
            let ignore_file_path = entry.path();
            if !ignore_file_path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|v| [".eslintignore", ".gitignore"].contains(&v))
            {
                continue;
            }
            if let Some(ignore_file_dir) = ignore_file_path.parent() {
                let mut builder = ignore::gitignore::GitignoreBuilder::new(ignore_file_dir);
                builder.add(ignore_file_path);
                if let Ok(gitignore) = builder.build() {
                    gitignore_globs.push(gitignore);
                }
            }
        }

        gitignore_globs
    }
}

pub struct ServerLinter {
    run: Run,
    isolated_linter: Arc<Mutex<IsolatedLintHandler>>,
    ignore_matcher: LintIgnoreMatcher,
    gitignore_glob: Vec<Gitignore>,
    extended_paths: FxHashSet<PathBuf>,
    diagnostics: Arc<ConcurrentHashMap<String, Option<Vec<DiagnosticReport>>>>,
}

impl ServerLinter {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn new(
        run: Run,
        isolated_linter: Arc<Mutex<IsolatedLintHandler>>,
        ignore_matcher: LintIgnoreMatcher,
        gitignore_glob: Vec<Gitignore>,
        extended_paths: FxHashSet<PathBuf>,
    ) -> Self {
        Self {
            run,
            isolated_linter,
            ignore_matcher,
            gitignore_glob,
            extended_paths,
            diagnostics: Arc::new(ConcurrentHashMap::default()),
        }
    }

    pub fn remove_diagnostics(&self, uri: &Uri) {
        self.diagnostics.pin().remove(&uri.to_string());
    }

    pub fn get_cached_diagnostics(&self, uri: &Uri) -> Option<Vec<DiagnosticReport>> {
        if let Some(diagnostics) = self.diagnostics.pin().get(&uri.to_string()) {
            // when the uri is ignored, diagnostics is None.
            // We want to return Some(vec![]), so the Worker knows there are no diagnostics for this file.
            return Some(diagnostics.clone().unwrap_or_default());
        }
        None
    }

    pub fn get_cached_files_of_diagnostics(&self) -> Vec<Uri> {
        self.diagnostics.pin().keys().filter_map(|s| Uri::from_str(s).ok()).collect()
    }

    pub async fn revalidate_diagnostics(&self, uris: Vec<Uri>) -> Vec<(String, Vec<Diagnostic>)> {
        let mut diagnostics = Vec::with_capacity(uris.len());
        for uri in uris {
            if let Some(file_diagnostic) = self.run_single(&uri, None).await {
                diagnostics.push((
                    uri.to_string(),
                    file_diagnostic.into_iter().map(|d| d.diagnostic).collect(),
                ));
            }
        }
        diagnostics
    }

    fn is_ignored(&self, uri: &Uri) -> bool {
        let Some(uri_path) = uri.to_file_path() else {
            return true;
        };

        if self.ignore_matcher.should_ignore(&uri_path) {
            debug!("ignored: {uri:?}");
            return true;
        }

        for gitignore in &self.gitignore_glob {
            if !uri_path.starts_with(gitignore.path()) {
                continue;
            }
            if gitignore.matched_path_or_any_parents(&uri_path, uri_path.is_dir()).is_ignore() {
                debug!("ignored: {uri:?}");
                return true;
            }
        }
        false
    }

    /// Lint a single file, return `None` if the file is ignored.
    pub async fn run_single(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if self.is_ignored(uri) {
            return None;
        }

        let diagnostics = {
            let mut isolated_linter = self.isolated_linter.lock().await;
            isolated_linter.run_single(uri, content.clone())
        };

        self.diagnostics.pin().insert(uri.to_string(), diagnostics.clone());

        diagnostics
    }

    /// Lint a single file, return `None` if the file is ignored.
    /// Only runs if the `run` option is set to `OnType`.
    pub async fn run_single_on_change(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if self.run != Run::OnType {
            return None;
        }
        self.run_single(uri, content).await
    }

    /// Lint a single file, return `None` if the file is ignored.
    /// Only runs if the `run` option is set to `OnSave`.
    pub async fn run_single_on_save(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if self.run != Run::OnSave {
            return None;
        }
        self.run_single(uri, content).await
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub async fn handle_configuration_change(
        &self,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges<ServerLinter> {
        let old_option = match serde_json::from_value::<LSPLintOptions>(old_options_json.clone()) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };

        let new_options = match serde_json::from_value::<LSPLintOptions>(new_options_json.clone()) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };

        if !Self::needs_restart(&old_option, &new_options) {
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: None,
                watch_patterns: None,
            };
        }

        // get the cached files before refreshing the linter, and revalidate them after
        let cached_files = self.get_cached_files_of_diagnostics();
        let new_linter =
            ServerLinterBuilder::new(root_uri.clone(), new_options_json.clone()).build();
        let diagnostics = Some(new_linter.revalidate_diagnostics(cached_files).await);

        let patterns = {
            if old_option.config_path == new_options.config_path
                && old_option.use_nested_configs() == new_options.use_nested_configs()
            {
                None
            } else {
                Some(new_linter.get_watch_patterns(
                    new_options_json,
                    root_uri.to_file_path().as_ref().unwrap(),
                ))
            }
        };

        ToolRestartChanges {
            tool: Some(new_linter),
            diagnostic_reports: diagnostics,
            watch_patterns: patterns,
        }
    }

    fn needs_restart(old_options: &LSPLintOptions, new_options: &LSPLintOptions) -> bool {
        old_options.config_path != new_options.config_path
            || old_options.ts_config_path != new_options.ts_config_path
            || old_options.use_nested_configs() != new_options.use_nested_configs()
            || old_options.fix_kind != new_options.fix_kind
            || old_options.unused_disable_directives != new_options.unused_disable_directives
            // TODO: only the TsgoLinter needs to be dropped or created
            || old_options.type_aware != new_options.type_aware
    }

    pub fn get_watch_patterns(&self, options: serde_json::Value, root_path: &Path) -> Vec<Pattern> {
        let options = match serde_json::from_value::<LSPLintOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };
        let mut watchers = vec![
            options.config_path.as_ref().unwrap_or(&"**/.oxlintrc.json".to_string()).to_owned(),
        ];

        for path in &self.extended_paths {
            // ignore .oxlintrc.json files when using nested configs
            if path.ends_with(".oxlintrc.json") && options.use_nested_configs() {
                continue;
            }

            let pattern = path.strip_prefix(root_path).unwrap_or(path);

            watchers.push(normalize_path(pattern).to_string_lossy().to_string());
        }
        watchers
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use crate::{
        linter::{options::LintOptions, server_linter::ServerLinterBuilder},
        tester::{Tester, get_file_path},
    };

    #[test]
    fn test_create_nested_configs_with_disabled_nested_configs() {
        let mut nested_ignore_patterns = Vec::new();
        let (configs, _) = ServerLinterBuilder::create_nested_configs(
            Path::new("/root/"),
            &LintOptions { disable_nested_config: true, ..LintOptions::default() },
            &mut nested_ignore_patterns,
        );

        assert!(configs.is_empty());
    }

    #[test]
    fn test_create_nested_configs() {
        let mut nested_ignore_patterns = Vec::new();
        let (configs, _) = ServerLinterBuilder::create_nested_configs(
            &get_file_path("fixtures/linter/init_nested_configs"),
            &LintOptions::default(),
            &mut nested_ignore_patterns,
        );
        let configs = configs.pin();
        let mut configs_dirs = configs.keys().collect::<Vec<&PathBuf>>();
        // sorting the key because for consistent tests results
        configs_dirs.sort();

        assert!(configs_dirs.len() == 3);
        assert!(configs_dirs[2].ends_with("deep2"));
        assert!(configs_dirs[1].ends_with("deep1"));
        assert!(configs_dirs[0].ends_with("init_nested_configs"));
    }

    #[test]
    fn test_no_errors() {
        Tester::new("fixtures/linter/no_errors", json!({}))
            .test_and_snapshot_single_file("hello_world.js");
    }

    #[test]
    fn test_no_console() {
        Tester::new("fixtures/linter/deny_no_console", json!({}))
            .test_and_snapshot_single_file("hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        Tester::new("fixtures/linter/issue_9958", json!({}))
            .test_and_snapshot_single_file("issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        Tester::new("fixtures/linter/regexp_feature", json!({}))
            .test_and_snapshot_single_file("index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new("fixtures/linter/astro", json!({}))
            .test_and_snapshot_single_file("debugger.astro");
        Tester::new("fixtures/linter/vue", json!({})).test_and_snapshot_single_file("debugger.vue");
        Tester::new("fixtures/linter/svelte", json!({}))
            .test_and_snapshot_single_file("debugger.svelte");
        // ToDo: fix Tester to work only with Uris and do not access the file system
        // Tester::new("fixtures/linter/nextjs").test_and_snapshot_single_file("%5B%5B..rest%5D%5D/debugger.ts");
    }

    #[test]
    fn test_invalid_syntax_file() {
        Tester::new("fixtures/linter/invalid_syntax", json!({}))
            .test_and_snapshot_multiple_file(&["debugger.ts", "invalid.vue"]);
    }

    #[test]
    fn test_cross_module_debugger() {
        Tester::new("fixtures/linter/cross_module", json!({}))
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        Tester::new("fixtures/linter/cross_module", json!({}))
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        Tester::new("fixtures/linter/cross_module_nested_config", json!({}))
            .test_and_snapshot_multiple_file(&["dep-a.ts", "folder/folder-dep-a.ts"]);
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        Tester::new("fixtures/linter/cross_module_extended_config", json!({}))
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_multiple_suggestions() {
        Tester::new(
            "fixtures/linter/multiple_suggestions",
            json!({
                "fixKind": "safe_fix_or_suggestion"
            }),
        )
        .test_and_snapshot_single_file("forward_ref.ts");
    }

    #[test]
    fn test_report_unused_directives() {
        Tester::new(
            "fixtures/linter/unused_disabled_directives",
            json!({
                "unusedDisableDirectives": "deny"
            }),
        )
        .test_and_snapshot_single_file("test.js");
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_report_tsgolint_unused_directives() {
        Tester::new(
            "fixtures/linter/tsgolint/unused_disabled_directives",
            json!({
                "unusedDisableDirectives": "deny",
                "typeAware": true
            }),
        )
        .test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_root_ignore_patterns() {
        let tester = Tester::new("fixtures/linter/ignore_patterns", json!({}));
        tester.test_and_snapshot_multiple_file(&[
            "ignored-file.ts",
            "another_config/not-ignored-file.ts",
        ]);
    }

    #[test]
    fn test_ts_alias() {
        Tester::new(
            "fixtures/linter/ts_path_alias",
            json!({
                "tsConfigPath": "./deep/tsconfig.json"
            }),
        )
        .test_and_snapshot_single_file("deep/src/dep-a.ts");
    }

    #[test]
    #[cfg(not(target_endian = "big"))] // TODO: tsgolint doesn't support big endian?
    fn test_tsgo_lint() {
        let tester = Tester::new(
            "fixtures/linter/tsgolint",
            json!({
                "typeAware": true,
                "fixKind": "all"
            }),
        );
        tester.test_and_snapshot_single_file("no-floating-promises/index.ts");
    }

    #[test]
    fn test_ignore_js_plugins() {
        let tester = Tester::new("fixtures/linter/js_plugins", json!({}));
        tester.test_and_snapshot_single_file("index.js");
    }

    // https://github.com/oxc-project/oxc/issues/14565
    #[test]
    fn test_issue_14565() {
        let tester = Tester::new(
            "fixtures/linter/issue_14565",
            json!({
                "run": "onSave"
            }),
        );
        tester.test_and_snapshot_single_file("foo-bar.astro");
    }
}
