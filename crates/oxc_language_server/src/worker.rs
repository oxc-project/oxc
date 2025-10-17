use std::vec;

use log::debug;
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::{
    UriExt,
    lsp_types::{
        CodeActionOrCommand, Diagnostic, FileEvent, FileSystemWatcher, GlobPattern, OneOf, Range,
        RelativePattern, TextEdit, Uri, WatchKind,
    },
};

use crate::{
    ConcurrentHashMap, FORMAT_CONFIG_FILE,
    code_actions::{apply_all_fix_code_action, apply_fix_code_actions, fix_all_text_edit},
    formatter::{options::FormatOptions, server_formatter::ServerFormatter},
    linter::{
        error_with_position::DiagnosticReport,
        options::LintOptions,
        server_linter::{ServerLinter, ServerLinterRun},
    },
    options::Options,
    utils::normalize_path,
};

/// A worker that manages the individual tools for a specific workspace
/// and reports back the results to the [`Backend`](crate::backend::Backend).
///
/// Each worker is responsible for a specific root URI and configures the tools `cwd` to that root URI.
/// The [`Backend`](crate::backend::Backend) is responsible to target the correct worker for a given file URI.
pub struct WorkspaceWorker {
    root_uri: Uri,
    server_linter: RwLock<Option<ServerLinter>>,
    server_formatter: RwLock<Option<ServerFormatter>>,
    options: Mutex<Option<Options>>,
}

impl WorkspaceWorker {
    /// Create a new workspace worker.
    /// This will not start any programs, use [`start_worker`](Self::start_worker) for that.
    /// Depending on the client, we need to request the workspace configuration in `initialized.
    pub fn new(root_uri: Uri) -> Self {
        Self {
            root_uri,
            server_linter: RwLock::new(None),
            server_formatter: RwLock::new(None),
            options: Mutex::new(None),
        }
    }

    /// Get the root URI of the worker
    pub fn get_root_uri(&self) -> &Uri {
        &self.root_uri
    }

    /// Check if the worker is responsible for the given URI
    /// A worker is responsible for a URI if the URI is a file URI and is located within the root URI of the worker
    /// e.g. root URI: file:///path/to/root
    ///      responsible for: file:///path/to/root/file.js
    ///      not responsible for: file:///path/to/other/file.js
    pub fn is_responsible_for_uri(&self, uri: &Uri) -> bool {
        if let Some(path) = uri.to_file_path() {
            return path.starts_with(self.root_uri.to_file_path().unwrap());
        }
        false
    }

    /// Start all programs (linter, formatter) for the worker.
    /// This should be called after the client has sent the workspace configuration.
    pub async fn start_worker(&self, options: &Options) {
        *self.options.lock().await = Some(options.clone());

        *self.server_linter.write().await = Some(ServerLinter::new(&self.root_uri, &options.lint));
        if options.format.experimental {
            debug!("experimental formatter enabled");
            *self.server_formatter.write().await =
                Some(ServerFormatter::new(&self.root_uri, &options.format));
        }
    }

    /// Initialize file system watchers for the workspace.
    /// These watchers are used to watch for changes in the lint configuration files.
    /// The returned watchers will be registered to the client.
    pub async fn init_watchers(&self) -> Vec<FileSystemWatcher> {
        let mut watchers = Vec::new();

        // clone the options to avoid locking the mutex
        let options = self.options.lock().await;
        let default_options = Options::default();
        let options = options.as_ref().unwrap_or(&default_options);
        let use_nested_configs = options.lint.use_nested_configs();

        // append the base watcher
        watchers.push(FileSystemWatcher {
            glob_pattern: GlobPattern::Relative(RelativePattern {
                base_uri: OneOf::Right(self.root_uri.clone()),
                pattern: options
                    .lint
                    .config_path
                    .as_ref()
                    .unwrap_or(&"**/.oxlintrc.json".to_owned())
                    .to_owned(),
            }),
            kind: Some(WatchKind::all()), // created, deleted, changed
        });

        if options.format.experimental {
            watchers.push(FileSystemWatcher {
                glob_pattern: GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(self.root_uri.clone()),
                    pattern: options
                        .format
                        .config_path
                        .as_ref()
                        .map_or(FORMAT_CONFIG_FILE, |v| v)
                        .to_owned(),
                }),
                kind: Some(WatchKind::all()), // created, deleted, changed
            });
        }

        let Some(root_path) = &self.root_uri.to_file_path() else {
            return watchers;
        };

        // Add watchers for all extended config paths of the current linter
        let Some(extended_paths) =
            self.server_linter.read().await.as_ref().map(|linter| linter.extended_paths.clone())
        else {
            return watchers;
        };

        for path in &extended_paths {
            // ignore .oxlintrc.json files when using nested configs
            if path.ends_with(".oxlintrc.json") && use_nested_configs {
                continue;
            }

            let pattern = path.strip_prefix(root_path).unwrap_or(path);

            watchers.push(FileSystemWatcher {
                glob_pattern: GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(self.root_uri.clone()),
                    pattern: normalize_path(pattern).to_string_lossy().to_string(),
                }),
                kind: Some(WatchKind::all()), // created, deleted, changed
            });
        }

        watchers
    }

    /// Check if the worker needs to be initialized with options
    pub async fn needs_init_options(&self) -> bool {
        self.options.lock().await.is_none()
    }

    pub async fn has_active_formatter(&self) -> bool {
        self.server_formatter.read().await.is_some()
    }

    /// Remove all diagnostics for the given URI
    pub async fn remove_diagnostics(&self, uri: &Uri) {
        let server_linter_guard = self.server_linter.read().await;
        let Some(server_linter) = server_linter_guard.as_ref() else {
            return;
        };
        server_linter.remove_diagnostics(uri);
    }

    /// Refresh the server linter with the current options
    /// This will recreate the linter and re-read the config files.
    /// Call this when the options have changed and the linter needs to be updated.
    async fn refresh_server_linter(&self, lint_options: &LintOptions) {
        let server_linter = ServerLinter::new(&self.root_uri, lint_options);

        *self.server_linter.write().await = Some(server_linter);
    }

    /// Restart the server formatter with the current options
    /// This will recreate the formatter and re-read the config files.
    /// Call this when the options have changed and the formatter needs to be updated.
    async fn refresh_server_formatter(&self, format_options: &FormatOptions) {
        let server_formatter = ServerFormatter::new(&self.root_uri, format_options);

        *self.server_formatter.write().await = Some(server_formatter);
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable, [`None`] is returned
    /// - If the file is lintable, but no diagnostics are found, an empty vector is returned
    pub async fn lint_file(
        &self,
        uri: &Uri,
        content: Option<String>,
        run_type: ServerLinterRun,
    ) -> Option<Vec<DiagnosticReport>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return None;
        };

        server_linter.run_single(uri, content, run_type).await
    }

    /// Format a file with the current formatter
    /// - If no formatter is active, [`None`] is returned
    /// - If the formatter is active, but no changes are made, an empty vector is returned
    pub async fn format_file(&self, uri: &Uri, content: Option<String>) -> Option<Vec<TextEdit>> {
        let Some(server_formatter) = &*self.server_formatter.read().await else {
            return None;
        };

        server_formatter.run_single(uri, content)
    }

    /// Revalidate diagnostics for the given URIs
    /// This will re-lint all opened files and return the new diagnostics
    async fn revalidate_diagnostics(
        &self,
        uris: Vec<Uri>,
    ) -> ConcurrentHashMap<String, Vec<DiagnosticReport>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return ConcurrentHashMap::default();
        };

        server_linter.revalidate_diagnostics(uris).await
    }

    /// Get all clear diagnostics for the current workspace
    /// This should be called when:
    /// - The linter is disabled (not currently implemented)
    /// - The workspace is closed
    /// - The server is shut down
    ///
    /// This will return a list of URIs that had diagnostics before, each with an empty diagnostics list
    pub async fn get_clear_diagnostics(&self) -> Vec<(String, Vec<Diagnostic>)> {
        self.server_linter
            .read()
            .await
            .as_ref()
            .map(|server_linter| {
                server_linter
                    .get_cached_files_of_diagnostics()
                    .iter()
                    .map(|uri| (uri.to_string(), vec![]))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// Get code actions or commands for the given range
    /// It uses the [`ServerLinter`] cached diagnostics if available, otherwise it will lint the file
    /// If `is_source_fix_all_oxc` is true, it will return a single code action that applies all fixes
    pub async fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        range: &Range,
        is_source_fix_all_oxc: bool,
    ) -> Vec<CodeActionOrCommand> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return vec![];
        };

        let value = if let Some(cached_diagnostics) = server_linter.get_cached_diagnostics(uri) {
            cached_diagnostics
        } else {
            let diagnostics = server_linter.run_single(uri, None, ServerLinterRun::Always).await;
            diagnostics.unwrap_or_default()
        };

        if value.is_empty() {
            return vec![];
        }

        let reports = value
            .iter()
            .filter(|r| r.diagnostic.range == *range || range_overlaps(*range, r.diagnostic.range));

        if is_source_fix_all_oxc {
            return apply_all_fix_code_action(reports, uri).map_or(vec![], |code_actions| {
                vec![CodeActionOrCommand::CodeAction(code_actions)]
            });
        }

        let mut code_actions_vec: Vec<CodeActionOrCommand> = vec![];

        for report in reports {
            if let Some(fix_actions) = apply_fix_code_actions(report, uri) {
                code_actions_vec
                    .extend(fix_actions.into_iter().map(CodeActionOrCommand::CodeAction));
            }
        }

        code_actions_vec
    }

    /// This function is used for executing the `oxc.fixAll` command
    pub async fn get_diagnostic_text_edits(&self, uri: &Uri) -> Vec<TextEdit> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return vec![];
        };
        let value = if let Some(cached_diagnostics) = server_linter.get_cached_diagnostics(uri) {
            cached_diagnostics
        } else {
            let diagnostics = server_linter.run_single(uri, None, ServerLinterRun::Always).await;
            diagnostics.unwrap_or_default()
        };

        if value.is_empty() {
            return vec![];
        }

        fix_all_text_edit(value.iter())
    }

    /// Handle file changes that are watched by the client
    /// At the moment, this only handles changes to lint configuration files
    /// When a change is detected, the linter is refreshed and all diagnostics are revalidated
    pub async fn did_change_watched_files(
        &self,
        _file_event: &FileEvent,
    ) -> Option<ConcurrentHashMap<String, Vec<DiagnosticReport>>> {
        // TODO: the tools should implement a helper function to detect if the changed file is relevant
        let files = {
            let server_linter_guard = self.server_linter.read().await;
            let server_linter = server_linter_guard.as_ref()?;
            server_linter.get_cached_files_of_diagnostics()
        };
        let options = self.options.lock().await;
        let format_options = options.as_ref().map(|o| o.format.clone()).unwrap_or_default();
        let lint_options = options.as_ref().map(|o| o.lint.clone()).unwrap_or_default();

        if format_options.experimental {
            tokio::join!(
                self.refresh_server_formatter(&format_options),
                self.refresh_server_linter(&lint_options)
            );
        } else {
            self.refresh_server_linter(&lint_options).await;
        }

        Some(self.revalidate_diagnostics(files).await)
    }

    /// Handle server configuration changes from the client
    pub async fn did_change_configuration(
        &self,
        changed_options: &Options,
    ) -> (
        // Diagnostic reports that need to be revalidated
        Option<ConcurrentHashMap<String, Vec<DiagnosticReport>>>,
        // File system watcher for lint/fmt config changes
        // - `None` if no watcher changes are needed
        // - empty vector if all watchers should be removed
        Option<Vec<FileSystemWatcher>>,
        // Is true, when the formatter was added to the workspace worker
        bool,
    ) {
        // Scope the first lock so it is dropped before the second lock
        let current_option = {
            let options_guard = self.options.lock().await;
            options_guard.clone()
        }
        .unwrap_or_default();

        debug!(
            "
        configuration changed:
        incoming: {changed_options:?}
        current: {current_option:?}
        "
        );

        {
            let mut options_guard = self.options.lock().await;
            *options_guard = Some(changed_options.clone());
        }

        let mut formatting = false;
        let mut removed_formatter = false;

        // create all watchers again, because maybe one tool configuration is changed
        // and we unregister the workspace watcher and register a new one.
        // Without adding the old watchers back, the client would not watch them anymore.
        //
        // TODO: create own watcher for each tool with its own id,
        // so we can unregister only the watcher that changed.
        let mut watchers = Vec::new();

        if current_option.format == changed_options.format {
            if changed_options.format.experimental {
                watchers.push(FileSystemWatcher {
                    glob_pattern: GlobPattern::Relative(RelativePattern {
                        base_uri: OneOf::Right(self.root_uri.clone()),
                        pattern: changed_options
                            .format
                            .config_path
                            .as_ref()
                            .map_or(FORMAT_CONFIG_FILE, |v| v)
                            .to_owned(),
                    }),
                    kind: Some(WatchKind::all()), // created, deleted, changed
                });
            }
        } else if changed_options.format.experimental {
            debug!("experimental formatter enabled/restarted");
            self.refresh_server_formatter(&changed_options.format).await;

            formatting = true;

            watchers.push(FileSystemWatcher {
                glob_pattern: GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(self.root_uri.clone()),
                    pattern: changed_options
                        .format
                        .config_path
                        .as_ref()
                        .map_or(FORMAT_CONFIG_FILE, |v| v)
                        .to_owned(),
                }),
                kind: Some(WatchKind::all()), // created, deleted, changed
            });
        } else {
            debug!("experimental formatter disabled");
            *self.server_formatter.write().await = None;
            removed_formatter = true;
        }

        if ServerLinter::needs_restart(&current_option.lint, &changed_options.lint) {
            let files = {
                let server_linter_guard = self.server_linter.read().await;
                let server_linter = server_linter_guard.as_ref();
                if let Some(server_linter) = server_linter {
                    server_linter.get_cached_files_of_diagnostics()
                } else {
                    vec![]
                }
            };
            self.refresh_server_linter(&changed_options.lint).await;

            watchers.push(FileSystemWatcher {
                glob_pattern: GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(self.root_uri.clone()),
                    pattern: changed_options
                        .lint
                        .config_path
                        .as_ref()
                        .unwrap_or(&"**/.oxlintrc.json".to_string())
                        .to_owned(),
                }),
                kind: Some(WatchKind::all()), // created, deleted, changed
            });

            return (Some(self.revalidate_diagnostics(files).await), Some(watchers), formatting);
        } else if !watchers.is_empty() || removed_formatter {
            // when we added/removed the formatter watcher, we also need to add the linter watcher again
            watchers.push(FileSystemWatcher {
                glob_pattern: GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(self.root_uri.clone()),
                    pattern: changed_options
                        .lint
                        .config_path
                        .as_ref()
                        .unwrap_or(&"**/.oxlintrc.json".to_string())
                        .to_owned(),
                }),
                kind: Some(WatchKind::all()), // created, deleted, changed
            });
        }

        let watchers = if watchers.is_empty() { None } else { Some(watchers) };

        (None, watchers, formatting)
    }
}

fn range_overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && a.end >= b.start
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_get_root_uri() {
        let worker = WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap());

        assert_eq!(worker.get_root_uri(), &Uri::from_str("file:///root/").unwrap());
    }

    #[test]
    fn test_is_responsible() {
        let worker = WorkspaceWorker::new(Uri::from_str("file:///path/to/root").unwrap());

        assert!(
            worker.is_responsible_for_uri(&Uri::from_str("file:///path/to/root/file.js").unwrap())
        );
        assert!(worker.is_responsible_for_uri(
            &Uri::from_str("file:///path/to/root/folder/file.js").unwrap()
        ));
        assert!(
            !worker
                .is_responsible_for_uri(&Uri::from_str("file:///path/to/other/file.js").unwrap())
        );
    }
}

#[cfg(test)]
mod test_watchers {
    use tower_lsp_server::{
        UriExt,
        lsp_types::{FileSystemWatcher, Uri},
    };

    use crate::{options::Options, worker::WorkspaceWorker};

    struct Tester {
        pub worker: WorkspaceWorker,
    }

    impl Tester {
        pub fn new(relative_root_dir: &'static str, options: &Options) -> Self {
            let absolute_path =
                std::env::current_dir().expect("could not get current dir").join(relative_root_dir);
            let uri =
                Uri::from_file_path(absolute_path).expect("could not convert current dir to uri");

            let worker = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { Self::create_workspace_worker(uri, options).await });

            Self { worker }
        }

        async fn create_workspace_worker(absolute_path: Uri, options: &Options) -> WorkspaceWorker {
            let worker = WorkspaceWorker::new(absolute_path);
            worker.start_worker(options).await;

            worker
        }

        fn init_watchers(&self) -> Vec<FileSystemWatcher> {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { self.worker.init_watchers().await })
        }

        fn did_change_configuration(&self, options: &Options) -> Option<Vec<FileSystemWatcher>> {
            let (_, watchers, _) = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { self.worker.did_change_configuration(options).await });
            watchers
        }
    }

    mod init_watchers {
        use tower_lsp_server::lsp_types::{GlobPattern, OneOf, RelativePattern};

        use crate::{
            formatter::options::FormatOptions, linter::options::LintOptions, options::Options,
            worker::test_watchers::Tester,
        };

        #[test]
        fn test_default_options() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 1);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
        }

        #[test]
        fn test_custom_config_path() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    lint: LintOptions {
                        config_path: Some("configs/lint.json".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 1);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/lint.json".to_string(),
                })
            );
        }

        #[test]
        fn test_linter_extends_configs() {
            let tester = Tester::new("fixtures/watcher/linter_extends", &Options::default());
            let watchers = tester.init_watchers();

            // The `.oxlintrc.json` extends `./lint.json -> 2 watchers
            assert_eq!(watchers.len(), 2);

            // nested configs pattern
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );

            // extends of root config
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "lint.json".to_string(),
                })
            );
        }

        #[test]
        fn test_linter_extends_custom_config_path() {
            let tester = Tester::new(
                "fixtures/watcher/linter_extends",
                &Options {
                    lint: LintOptions {
                        config_path: Some(".oxlintrc.json".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: ".oxlintrc.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "lint.json".to_string(),
                })
            );
        }

        #[test]
        fn test_formatter_experimental_enabled() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                },
            );
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: ".oxfmtrc.json".to_string(),
                })
            );
        }

        #[test]
        fn test_formatter_custom_config_path() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    format: FormatOptions {
                        experimental: true,
                        config_path: Some("configs/formatter.json".to_string()),
                    },
                    ..Default::default()
                },
            );
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/formatter.json".to_string(),
                })
            );
        }

        #[test]
        fn test_linter_and_formatter_custom_config_path() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    lint: LintOptions {
                        config_path: Some("configs/lint.json".to_string()),
                        ..Default::default()
                    },
                    format: FormatOptions {
                        experimental: true,
                        config_path: Some("configs/formatter.json".to_string()),
                    },
                },
            );
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/lint.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/formatter.json".to_string(),
                })
            );
        }
    }

    mod did_change_configuration {
        use tower_lsp_server::lsp_types::{GlobPattern, OneOf, RelativePattern};

        use crate::{
            formatter::options::FormatOptions,
            linter::options::{LintOptions, Run},
            options::Options,
            worker::test_watchers::Tester,
        };

        #[test]
        fn test_no_change() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let watchers = tester.did_change_configuration(&Options::default());
            assert!(watchers.is_none());
        }

        #[test]
        fn test_lint_config_path_change() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let watchers = tester
                .did_change_configuration(&Options {
                    lint: LintOptions {
                        config_path: Some("configs/lint.json".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .unwrap();

            assert_eq!(watchers.len(), 1);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/lint.json".to_string(),
                })
            );
        }

        #[test]
        fn test_lint_other_option_change() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let watchers = tester.did_change_configuration(&Options {
                // run is the only option that does not require a restart
                lint: LintOptions { run: Run::OnSave, ..Default::default() },
                ..Default::default()
            });
            assert!(watchers.is_none());
        }

        #[test]
        fn test_no_changes_with_formatter() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                },
            );
            let watchers = tester
                .did_change_configuration(&Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                })
                .unwrap();

            // TODO: should be none
            assert!(watchers.len() == 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: ".oxfmtrc.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
        }

        #[test]
        fn test_lint_config_path_change_with_formatter() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                },
            );
            let watchers = tester
                .did_change_configuration(&Options {
                    lint: LintOptions {
                        config_path: Some("configs/lint.json".to_string()),
                        ..Default::default()
                    },
                    format: FormatOptions { experimental: true, ..Default::default() },
                })
                .unwrap();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: ".oxfmtrc.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/lint.json".to_string(),
                })
            );
        }

        #[test]
        fn test_formatter_experimental_enabled() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let watchers = tester
                .did_change_configuration(&Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                })
                .unwrap();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: ".oxfmtrc.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
        }

        #[test]
        fn test_formatter_custom_config_path() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                },
            );
            let watchers = tester
                .did_change_configuration(&Options {
                    format: FormatOptions {
                        experimental: true,
                        config_path: Some("configs/formatter.json".to_string()),
                    },
                    ..Default::default()
                })
                .unwrap();

            assert_eq!(watchers.len(), 2);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "configs/formatter.json".to_string(),
                })
            );
            assert_eq!(
                watchers[1].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
        }

        #[test]
        fn test_formatter_disabling() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                &Options {
                    format: FormatOptions { experimental: true, ..Default::default() },
                    ..Default::default()
                },
            );
            let watchers = tester
                .did_change_configuration(&Options {
                    format: FormatOptions { experimental: false, ..Default::default() },
                    ..Default::default()
                })
                .unwrap();
            assert!(watchers.len() == 1);
            assert_eq!(
                watchers[0].glob_pattern,
                GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(tester.worker.get_root_uri().clone()),
                    pattern: "**/.oxlintrc.json".to_string(),
                })
            );
        }
    }
}
