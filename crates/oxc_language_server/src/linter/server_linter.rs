use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use globset::Glob;
use ignore::gitignore::Gitignore;
use log::{debug, warn};
use rustc_hash::{FxBuildHasher, FxHashMap};
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::Uri;

use oxc_linter::{
    AllowWarnDeny, Config, ConfigStore, ConfigStoreBuilder, ExternalPluginStore, LintOptions,
    Oxlintrc,
};
use tower_lsp_server::UriExt;

use crate::linter::{
    error_with_position::DiagnosticReport,
    isolated_lint_handler::{IsolatedLintHandler, IsolatedLintHandlerOptions},
};
use crate::options::UnusedDisableDirectives;
use crate::{ConcurrentHashMap, OXC_CONFIG_FILE, Options};

use super::config_walker::ConfigWalker;

pub struct ServerLinter {
    isolated_linter: Arc<Mutex<IsolatedLintHandler>>,
    gitignore_glob: Vec<Gitignore>,
    pub extended_paths: Vec<PathBuf>,
}

impl ServerLinter {
    pub fn new(root_uri: &Uri, options: &Options) -> Self {
        let root_path = root_uri.to_file_path().unwrap();
        let (nested_configs, mut extended_paths) = Self::create_nested_configs(&root_path, options);
        let config_path = options.config_path.as_ref().map_or(OXC_CONFIG_FILE, |v| v);
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

        // clone because we are returning it for ignore builder
        let config_builder = ConfigStoreBuilder::from_oxlintrc(
            false,
            oxlintrc.clone(),
            None,
            &mut ExternalPluginStore::default(),
        )
        .unwrap_or_default();

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = options.use_nested_configs();

        let use_cross_module = config_builder.plugins().has_import()
            || (use_nested_config
                && nested_configs.pin().values().any(|config| config.plugins().has_import()));

        extended_paths.extend(config_builder.extended_paths.clone());
        let external_plugin_store = ExternalPluginStore::default();
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
            ExternalPluginStore::default(),
        );

        let isolated_linter = IsolatedLintHandler::new(
            lint_options,
            config_store,
            &IsolatedLintHandlerOptions {
                use_cross_module,
                root_path: root_path.to_path_buf(),
                tsconfig_path: options
                    .ts_config_path
                    .as_ref()
                    .map(|path| Path::new(path).to_path_buf()),
            },
        );

        Self {
            isolated_linter: Arc::new(Mutex::new(isolated_linter)),
            gitignore_glob: Self::create_ignore_glob(&root_path, &oxlintrc),
            extended_paths,
        }
    }

    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    fn create_nested_configs(
        root_path: &Path,
        options: &Options,
    ) -> (ConcurrentHashMap<PathBuf, Config>, Vec<PathBuf>) {
        let mut extended_paths = Vec::new();
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
            let Ok(config_store_builder) = ConfigStoreBuilder::from_oxlintrc(
                false,
                oxlintrc,
                None,
                &mut ExternalPluginStore::default(),
            ) else {
                warn!("Skipping config (builder failed): {}", file_path.display());
                continue;
            };
            extended_paths.extend(config_store_builder.extended_paths.clone());
            let external_plugin_store = ExternalPluginStore::default();
            let config = config_store_builder.build(&external_plugin_store).unwrap_or_else(|err| {
                warn!("Failed to build nested config for {}: {:?}", dir_path.display(), err);
                ConfigStoreBuilder::empty().build(&external_plugin_store).unwrap()
            });
            nested_configs.pin().insert(dir_path.to_path_buf(), config);
        }

        (nested_configs, extended_paths)
    }

    fn create_ignore_glob(root_path: &Path, oxlintrc: &Oxlintrc) -> Vec<Gitignore> {
        let mut builder = globset::GlobSetBuilder::new();
        // Collecting all ignore files
        builder.add(Glob::new("**/.eslintignore").unwrap());
        builder.add(Glob::new("**/.gitignore").unwrap());

        let ignore_file_glob_set = builder.build().unwrap();

        let walk = ignore::WalkBuilder::new(root_path)
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .build()
            .flatten();

        let mut gitignore_globs = vec![];
        for entry in walk {
            let ignore_file_path = entry.path();
            if !ignore_file_glob_set.is_match(ignore_file_path) {
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

        if oxlintrc.ignore_patterns.is_empty() {
            return gitignore_globs;
        }

        let Some(oxlintrc_dir) = oxlintrc.path.parent() else {
            warn!("Oxlintrc path has no parent, skipping inline ignore patterns");
            return gitignore_globs;
        };

        let mut builder = ignore::gitignore::GitignoreBuilder::new(oxlintrc_dir);
        for entry in &oxlintrc.ignore_patterns {
            builder.add_line(None, entry).expect("Failed to add ignore line");
        }
        gitignore_globs.push(builder.build().unwrap());
        gitignore_globs
    }

    fn is_ignored(&self, uri: &Uri) -> bool {
        for gitignore in &self.gitignore_glob {
            if let Some(uri_path) = uri.to_file_path() {
                if !uri_path.starts_with(gitignore.path()) {
                    continue;
                }
                if gitignore.matched_path_or_any_parents(&uri_path, uri_path.is_dir()).is_ignore() {
                    debug!("ignored: {uri:?}");
                    return true;
                }
            }
        }
        false
    }

    pub async fn run_single(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if self.is_ignored(uri) {
            return None;
        }

        self.isolated_linter.lock().await.run_single(uri, content)
    }
}

/// Normalize a path by removing `.` and resolving `..` components,
/// without touching the filesystem.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.as_ref().components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {
                // Skip current directory component
            }
            Component::Normal(c) => {
                result.push(c);
            }
            Component::RootDir | Component::Prefix(_) => {
                result.push(component.as_os_str());
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use crate::{
        Options,
        linter::server_linter::{ServerLinter, normalize_path},
        tester::{Tester, get_file_path},
    };
    use rustc_hash::FxHashMap;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/root/directory/./.oxlintrc.json")),
            Path::new("/root/directory/.oxlintrc.json")
        );
    }

    #[test]
    fn test_create_nested_configs_with_disabled_nested_configs() {
        let mut flags = FxHashMap::default();
        flags.insert("disable_nested_configs".to_string(), "true".to_string());

        let (configs, _) = ServerLinter::create_nested_configs(
            Path::new("/root/"),
            &Options { flags, ..Options::default() },
        );

        assert!(configs.is_empty());
    }

    #[test]
    fn test_create_nested_configs() {
        let (configs, _) = ServerLinter::create_nested_configs(
            &get_file_path("fixtures/linter/init_nested_configs"),
            &Options::default(),
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
            .test_and_snapshot_single_file("dep-a.ts");
        Tester::new("fixtures/linter/cross_module_nested_config", None)
            .test_and_snapshot_single_file("folder/folder-dep-a.ts");
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
            Some(Options {
                flags: FxHashMap::from_iter([(
                    "fix_kind".to_string(),
                    "safe_fix_or_suggestion".to_string(),
                )]),
                ..Options::default()
            }),
        )
        .test_and_snapshot_single_file("forward_ref.ts");
    }

    #[test]
    fn test_report_unused_directives() {
        use crate::options::UnusedDisableDirectives;
        Tester::new(
            "fixtures/linter/unused_disabled_directives",
            Some(Options {
                unused_disable_directives: UnusedDisableDirectives::Deny,
                ..Default::default()
            }),
        )
        .test_and_snapshot_single_file("test.js");
    }

    #[test]
    fn test_root_ignore_patterns() {
        Tester::new("fixtures/linter/root_ignore_patterns", None)
            .test_and_snapshot_single_file("ignored-file.ts");
    }

    #[test]
    fn test_ts_alias() {
        Tester::new(
            "fixtures/linter/ts_path_alias",
            Some(Options {
                ts_config_path: Some("./deep/tsconfig.json".to_string()),
                ..Default::default()
            }),
        )
        .test_and_snapshot_single_file("deep/src/dep-a.ts");
    }
}
