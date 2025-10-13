use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use ignore::gitignore::Gitignore;
use log::{debug, warn};
use oxc_linter::{AllowWarnDeny, LintIgnoreMatcher};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::Uri;

use oxc_linter::{
    Config, ConfigStore, ConfigStoreBuilder, ExternalPluginStore, LintOptions, Oxlintrc,
};
use tower_lsp_server::UriExt;

use crate::linter::options::UnusedDisableDirectives;
use crate::linter::{
    error_with_position::DiagnosticReport,
    isolated_lint_handler::{IsolatedLintHandler, IsolatedLintHandlerOptions},
    options::{LintOptions as LSPLintOptions, Run},
};
use crate::utils::normalize_path;
use crate::{ConcurrentHashMap, LINT_CONFIG_FILE};

use super::config_walker::ConfigWalker;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerLinterRun {
    OnType,
    OnSave,
    Always,
}

pub struct ServerLinter {
    isolated_linter: Arc<Mutex<IsolatedLintHandler>>,
    ignore_matcher: LintIgnoreMatcher,
    gitignore_glob: Vec<Gitignore>,
    lint_on_run: Run,
    diagnostics: ServerLinterDiagnostics,
    pub extended_paths: FxHashSet<PathBuf>,
}

#[derive(Debug, Default)]
struct ServerLinterDiagnostics {
    isolated_linter: Arc<ConcurrentHashMap<String, Option<Vec<DiagnosticReport>>>>,
}

impl ServerLinterDiagnostics {
    pub fn get_diagnostics(&self, path: &str) -> Option<Vec<DiagnosticReport>> {
        let mut reports = Vec::new();
        let mut found = false;
        if let Some(entry) = self.isolated_linter.pin().get(path) {
            found = true;
            if let Some(diagnostics) = entry {
                reports.extend(diagnostics.clone());
            }
        }
        if found { Some(reports) } else { None }
    }

    pub fn remove_diagnostics(&self, path: &str) {
        self.isolated_linter.pin().remove(path);
    }

    pub fn get_cached_files_of_diagnostics(&self) -> Vec<String> {
        self.isolated_linter.pin().keys().cloned().collect::<Vec<_>>()
    }
}

impl ServerLinter {
    pub fn new(root_uri: &Uri, options: &LSPLintOptions) -> Self {
        let root_path = root_uri.to_file_path().unwrap();
        let mut nested_ignore_patterns = Vec::new();
        let (nested_configs, mut extended_paths) =
            Self::create_nested_configs(&root_path, options, &mut nested_ignore_patterns);
        let config_path = options.config_path.as_ref().map_or(LINT_CONFIG_FILE, |v| v);
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
        let use_nested_config = options.use_nested_configs();

        let use_cross_module = config_builder.plugins().has_import()
            || (use_nested_config
                && nested_configs.pin().values().any(|config| config.plugins().has_import()));

        extended_paths.extend(config_builder.extended_paths.clone());
        let base_config = config_builder.build(&external_plugin_store).unwrap_or_else(|err| {
            warn!("Failed to build config: {err}");
            ConfigStoreBuilder::empty().build(&external_plugin_store).unwrap()
        });

        let lint_options = LintOptions {
            fix: options.fix_kind(),
            report_unused_directive: match options.unused_disable_directives {
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
                type_aware: options.type_aware,
                root_path: root_path.to_path_buf(),
                tsconfig_path: options.ts_config_path.as_ref().map(|path| {
                    let path = Path::new(path).to_path_buf();
                    if path.is_relative() { root_path.join(path) } else { path }
                }),
            },
        );

        Self {
            isolated_linter: Arc::new(Mutex::new(isolated_linter)),
            ignore_matcher: LintIgnoreMatcher::new(
                &base_patterns,
                &root_path,
                nested_ignore_patterns,
            ),
            gitignore_glob: Self::create_ignore_glob(&root_path),
            extended_paths,
            lint_on_run: options.run,
            diagnostics: ServerLinterDiagnostics::default(),
        }
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

    pub fn remove_diagnostics(&self, uri: &Uri) {
        self.diagnostics.remove_diagnostics(&uri.to_string());
    }

    pub fn get_cached_diagnostics(&self, uri: &Uri) -> Option<Vec<DiagnosticReport>> {
        self.diagnostics.get_diagnostics(&uri.to_string())
    }

    pub fn get_cached_files_of_diagnostics(&self) -> Vec<Uri> {
        self.diagnostics
            .get_cached_files_of_diagnostics()
            .into_iter()
            .filter_map(|s| Uri::from_str(&s).ok())
            .collect()
    }

    pub async fn revalidate_diagnostics(
        &self,
        uris: Vec<Uri>,
    ) -> ConcurrentHashMap<String, Vec<DiagnosticReport>> {
        let map = ConcurrentHashMap::default();
        for uri in uris {
            if let Some(diagnostics) = self.run_single(&uri, None, ServerLinterRun::Always).await {
                map.pin().insert(uri.to_string(), diagnostics);
            }
        }
        map
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

    pub async fn run_single(
        &self,
        uri: &Uri,
        content: Option<String>,
        run_type: ServerLinterRun,
    ) -> Option<Vec<DiagnosticReport>> {
        let (oxlint, tsgolint) = match (run_type, self.lint_on_run) {
            // run everything on save, or when it is forced
            (ServerLinterRun::Always, _) | (ServerLinterRun::OnSave, Run::OnSave) => (true, true),
            // run only oxlint on type
            // tsgolint does not support memory source_text
            (ServerLinterRun::OnType, Run::OnType) => (true, false),
            // it does not match, run nothing
            (ServerLinterRun::OnType, Run::OnSave) => (false, false),
            // In onType mode, only TypeScript type checking runs on save
            // If type_aware is disabled (tsgo_linter is None), skip everything to preserve diagnostics
            (ServerLinterRun::OnSave, Run::OnType) => (false, true),
        };

        // return `None` when both tools do not want to be used
        if !oxlint && !tsgolint {
            return None;
        }

        if self.is_ignored(uri) {
            return None;
        }

        if oxlint {
            let diagnostics = {
                let mut isolated_linter = self.isolated_linter.lock().await;
                isolated_linter.run_single(uri, content.clone())
            };
            self.diagnostics.isolated_linter.pin().insert(uri.to_string(), diagnostics);
        }

        self.diagnostics.get_diagnostics(&uri.to_string())
    }

    pub fn needs_restart(old_options: &LSPLintOptions, new_options: &LSPLintOptions) -> bool {
        old_options.config_path != new_options.config_path
            || old_options.ts_config_path != new_options.ts_config_path
            || old_options.use_nested_configs() != new_options.use_nested_configs()
            || old_options.fix_kind() != new_options.fix_kind()
            || old_options.unused_disable_directives != new_options.unused_disable_directives
            // TODO: only the TsgoLinter needs to be dropped or created
            || old_options.type_aware != new_options.type_aware
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use crate::{
        ConcurrentHashMap,
        linter::{
            error_with_position::DiagnosticReport,
            options::{LintOptions, Run, UnusedDisableDirectives},
            server_linter::{ServerLinter, ServerLinterDiagnostics},
        },
        tester::{Tester, get_file_path},
    };
    use rustc_hash::FxHashMap;

    #[test]
    fn test_create_nested_configs_with_disabled_nested_configs() {
        let mut flags = FxHashMap::default();
        flags.insert("disable_nested_configs".to_string(), "true".to_string());

        let mut nested_ignore_patterns = Vec::new();
        let (configs, _) = ServerLinter::create_nested_configs(
            Path::new("/root/"),
            &LintOptions { flags, ..LintOptions::default() },
            &mut nested_ignore_patterns,
        );

        assert!(configs.is_empty());
    }

    #[test]
    fn test_create_nested_configs() {
        let mut nested_ignore_patterns = Vec::new();
        let (configs, _) = ServerLinter::create_nested_configs(
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
    fn test_get_diagnostics_found_and_none_entries() {
        let key = "file:///test.js".to_string();

        // Case 1: Entry present, Some diagnostics
        let diag = DiagnosticReport::default();
        let diag_map = ConcurrentHashMap::default();
        diag_map.pin().insert(key.clone(), Some(vec![diag]));

        let server_diag =
            super::ServerLinterDiagnostics { isolated_linter: std::sync::Arc::new(diag_map) };
        let result = server_diag.get_diagnostics(&key);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);

        // Case 2: Entry present, but value is None
        let diag_map_none = ConcurrentHashMap::default();
        diag_map_none.pin().insert(key.clone(), None);

        let server_diag_none =
            ServerLinterDiagnostics { isolated_linter: std::sync::Arc::new(diag_map_none) };
        let result_none = server_diag_none.get_diagnostics(&key);
        assert!(result_none.is_some());
        assert_eq!(result_none.unwrap().len(), 0);

        // Case 3: No entry at all
        let server_diag_empty = ServerLinterDiagnostics::default();
        let result_empty = server_diag_empty.get_diagnostics(&key);
        assert!(result_empty.is_none());
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_lint_on_run_on_type_on_type() {
        Tester::new(
            "fixtures/linter/lint_on_run/on_type",
            Some(LintOptions { type_aware: true, run: Run::OnType, ..Default::default() }),
        )
        .test_and_snapshot_single_file_with_run_type("on-type.ts", Run::OnType);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_lint_on_run_on_type_on_save() {
        Tester::new(
            "fixtures/linter/lint_on_run/on_save",
            Some(LintOptions { type_aware: true, run: Run::OnType, ..Default::default() }),
        )
        .test_and_snapshot_single_file_with_run_type("on-save.ts", Run::OnSave);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_lint_on_run_on_save_on_type() {
        Tester::new(
            "fixtures/linter/lint_on_run/on_save",
            Some(LintOptions { type_aware: true, run: Run::OnSave, ..Default::default() }),
        )
        .test_and_snapshot_single_file_with_run_type("on-type.ts", Run::OnType);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_lint_on_run_on_save_on_save() {
        Tester::new(
            "fixtures/linter/lint_on_run/on_type",
            Some(LintOptions { type_aware: true, run: Run::OnSave, ..Default::default() }),
        )
        .test_and_snapshot_single_file_with_run_type("on-save.ts", Run::OnSave);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_lint_on_run_on_type_on_save_without_type_aware() {
        Tester::new(
            "fixtures/linter/lint_on_run/on_type",
            Some(LintOptions { type_aware: false, run: Run::OnType, ..Default::default() }),
        )
        .test_and_snapshot_single_file_with_run_type("on-save-no-type-aware.ts", Run::OnSave);
    }

    #[test]
    fn test_no_errors() {
        Tester::new("fixtures/linter/no_errors", None)
            .test_and_snapshot_single_file("hello_world.js");
    }

    #[test]
    fn test_no_console() {
        Tester::new("fixtures/linter/deny_no_console", None)
            .test_and_snapshot_single_file("hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        Tester::new("fixtures/linter/issue_9958", None).test_and_snapshot_single_file("issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        Tester::new("fixtures/linter/regexp_feature", None)
            .test_and_snapshot_single_file("index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new("fixtures/linter/astro", None).test_and_snapshot_single_file("debugger.astro");
        Tester::new("fixtures/linter/vue", None).test_and_snapshot_single_file("debugger.vue");
        Tester::new("fixtures/linter/svelte", None)
            .test_and_snapshot_single_file("debugger.svelte");
        // ToDo: fix Tester to work only with Uris and do not access the file system
        // Tester::new("fixtures/linter/nextjs").test_and_snapshot_single_file("%5B%5B..rest%5D%5D/debugger.ts");
    }

    #[test]
    fn test_invalid_syntax_file() {
        Tester::new("fixtures/linter/invalid_syntax", None)
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_debugger() {
        Tester::new("fixtures/linter/cross_module", None)
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        Tester::new("fixtures/linter/cross_module", None).test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        Tester::new("fixtures/linter/cross_module_nested_config", None)
            .test_and_snapshot_multiple_file(&["dep-a.ts", "folder/folder-dep-a.ts"]);
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        Tester::new("fixtures/linter/cross_module_extended_config", None)
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_multiple_suggestions() {
        Tester::new(
            "fixtures/linter/multiple_suggestions",
            Some(LintOptions {
                flags: FxHashMap::from_iter([(
                    "fix_kind".to_string(),
                    "safe_fix_or_suggestion".to_string(),
                )]),
                ..Default::default()
            }),
        )
        .test_and_snapshot_single_file("forward_ref.ts");
    }

    #[test]
    fn test_report_unused_directives() {
        Tester::new(
            "fixtures/linter/unused_disabled_directives",
            Some(LintOptions {
                unused_disable_directives: UnusedDisableDirectives::Deny,
                ..Default::default()
            }),
        )
        .test_and_snapshot_single_file("test.js");
    }

    #[test]
    fn test_root_ignore_patterns() {
        let tester = Tester::new("fixtures/linter/ignore_patterns", None);
        tester.test_and_snapshot_multiple_file(&[
            "ignored-file.ts",
            "another_config/not-ignored-file.ts",
        ]);
    }

    #[test]
    fn test_ts_alias() {
        Tester::new(
            "fixtures/linter/ts_path_alias",
            Some(LintOptions {
                ts_config_path: Some("./deep/tsconfig.json".to_string()),
                ..Default::default()
            }),
        )
        .test_and_snapshot_single_file("deep/src/dep-a.ts");
    }

    #[test]
    #[cfg(not(target_endian = "big"))] // TODO: tsgolint doesn't support big endian?
    fn test_tsgo_lint() {
        let tester = Tester::new(
            "fixtures/linter/tsgolint",
            Some(LintOptions { type_aware: true, run: Run::OnSave, ..Default::default() }),
        );
        tester.test_and_snapshot_single_file("no-floating-promises/index.ts");
    }

    #[test]
    fn test_ignore_js_plugins() {
        let tester = Tester::new(
            "fixtures/linter/js_plugins",
            Some(LintOptions { run: Run::OnSave, ..Default::default() }),
        );
        tester.test_and_snapshot_single_file("index.js");
    }
}
