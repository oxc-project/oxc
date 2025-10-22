use log::debug;
use serde_json::json;
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::{
    UriExt,
    lsp_types::{
        CodeActionOrCommand, Diagnostic, DidChangeWatchedFilesRegistrationOptions, FileEvent,
        FileSystemWatcher, GlobPattern, OneOf, Range, Registration, RelativePattern, TextEdit,
        Unregistration, Uri, WatchKind,
    },
};

use crate::{
    code_actions::{apply_all_fix_code_action, apply_fix_code_actions, fix_all_text_edit},
    formatter::{options::FormatOptions, server_formatter::ServerFormatter},
    linter::{
        error_with_position::DiagnosticReport,
        options::LintOptions,
        server_linter::{ServerLinter, ServerLinterRun},
    },
    options::Options,
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
    pub async fn init_watchers(&self) -> Vec<Registration> {
        let mut registrations = Vec::new();

        let Some(root_path) = &self.root_uri.to_file_path() else {
            return registrations;
        };

        // clone the options to avoid locking the mutex
        let options = self.options.lock().await;
        let lint_options = options.as_ref().map(|o| o.lint.clone()).unwrap_or_default();
        let format_options = options.as_ref().map(|o| o.format.clone()).unwrap_or_default();
        let lint_patterns = self
            .server_linter
            .read()
            .await
            .as_ref()
            .map(|linter| linter.get_watch_patterns(&lint_options, root_path));
        let format_patterns = self
            .server_formatter
            .read()
            .await
            .as_ref()
            .map(|formatter| formatter.get_watcher_patterns(&format_options));

        if let Some(lint_patterns) = lint_patterns {
            registrations.push(Registration {
                id: format!("watcher-linter-{}", self.root_uri.as_str()),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
                    watchers: lint_patterns
                        .into_iter()
                        .map(|pattern| FileSystemWatcher {
                            glob_pattern: GlobPattern::Relative(RelativePattern {
                                base_uri: OneOf::Right(self.root_uri.clone()),
                                pattern,
                            }),
                            kind: Some(WatchKind::all()), // created, deleted, changed
                        })
                        .collect::<Vec<_>>(),
                })),
            });
        }

        if format_options.experimental
            && let Some(format_patterns) = format_patterns
        {
            registrations.push(Registration {
                id: format!("watcher-formatter-{}", self.root_uri.as_str()),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
                    watchers: format_patterns
                        .into_iter()
                        .map(|pattern| FileSystemWatcher {
                            glob_pattern: GlobPattern::Relative(RelativePattern {
                                base_uri: OneOf::Right(self.root_uri.clone()),
                                pattern,
                            }),
                            kind: Some(WatchKind::all()), // created, deleted, changed
                        })
                        .collect::<Vec<_>>(),
                })),
            });
        }

        registrations
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
    async fn revalidate_diagnostics(&self, uris: Vec<Uri>) -> Vec<(String, Vec<Diagnostic>)> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return Vec::new();
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
    ) -> Option<Vec<(String, Vec<Diagnostic>)>> {
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
        Option<Vec<(String, Vec<Diagnostic>)>>,
        // New watchers that need to be registered
        Vec<Registration>,
        // Watchers that need to be unregistered
        Vec<Unregistration>,
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

        let mut registrations = vec![];
        let mut unregistrations = vec![];
        let mut diagnostics = None;

        if current_option.format != changed_options.format {
            if changed_options.format.experimental {
                self.refresh_server_formatter(&changed_options.format).await;
                formatting = true;

                // Extract pattern data without holding the lock
                let patterns = {
                    let formatter_guard = self.server_formatter.read().await;
                    formatter_guard.as_ref().and_then(|formatter| {
                        formatter.get_changed_watch_patterns(
                            &current_option.format,
                            &changed_options.format,
                        )
                    })
                };

                if let Some(patterns) = patterns {
                    if current_option.format.experimental {
                        // unregister the old watcher
                        unregistrations.push(Unregistration {
                            id: format!("watcher-formatter-{}", self.root_uri.as_str()),
                            method: "workspace/didChangeWatchedFiles".to_string(),
                        });
                    }

                    registrations.push(Registration {
                        id: format!("watcher-formatter-{}", self.root_uri.as_str()),
                        method: "workspace/didChangeWatchedFiles".to_string(),
                        register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
                            watchers: patterns
                                .into_iter()
                                .map(|pattern| FileSystemWatcher {
                                    glob_pattern: GlobPattern::Relative(RelativePattern {
                                        base_uri: OneOf::Right(self.root_uri.clone()),
                                        pattern,
                                    }),
                                    kind: Some(WatchKind::all()), // created, deleted, changed
                                })
                                .collect::<Vec<_>>(),
                        })),
                    });
                }
            } else {
                *self.server_formatter.write().await = None;

                unregistrations.push(Unregistration {
                    id: format!("watcher-formatter-{}", self.root_uri.as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                });
            }
        }

        if ServerLinter::needs_restart(&current_option.lint, &changed_options.lint) {
            // get the cached files before refreshing the linter
            let linter_files = {
                let linter_guard = self.server_linter.read().await;
                linter_guard
                    .as_ref()
                    .map(|linter: &ServerLinter| linter.get_cached_files_of_diagnostics())
            };

            self.refresh_server_linter(&changed_options.lint).await;

            // Get the Watch patterns (including the files from oxlint `extends`)
            let patterns = {
                let linter_guard = self.server_linter.read().await;
                linter_guard.as_ref().and_then(|linter: &ServerLinter| {
                    linter.get_changed_watch_patterns(
                        &current_option.lint,
                        &changed_options.lint,
                        self.root_uri.to_file_path().as_ref().unwrap(),
                    )
                })
            };

            // revalidate diagnostics for previously cached files
            if let Some(linter_files) = linter_files {
                diagnostics = Some(self.revalidate_diagnostics(linter_files).await);
            }

            if let Some(patterns) = patterns {
                unregistrations.push(Unregistration {
                    id: format!("watcher-linter-{}", self.root_uri.as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                });

                registrations.push(Registration {
                    id: format!("watcher-linter-{}", self.root_uri.as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                    register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
                        watchers: patterns
                            .into_iter()
                            .map(|pattern| FileSystemWatcher {
                                glob_pattern: GlobPattern::Relative(RelativePattern {
                                    base_uri: OneOf::Right(self.root_uri.clone()),
                                    pattern,
                                }),
                                kind: Some(WatchKind::all()), // created, deleted, changed
                            })
                            .collect::<Vec<_>>(),
                    })),
                });
            }
        }

        (diagnostics, registrations, unregistrations, formatting)
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
    use serde_json::json;
    use tower_lsp_server::{
        UriExt,
        lsp_types::{
            DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, OneOf,
            Registration, RelativePattern, Unregistration, Uri, WatchKind,
        },
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

        fn init_watchers(&self) -> Vec<Registration> {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { self.worker.init_watchers().await })
        }

        fn did_change_configuration(
            &self,
            options: &Options,
        ) -> (Vec<Registration>, Vec<Unregistration>) {
            let (_, registration, unregistration, _) = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { self.worker.did_change_configuration(options).await });

            (registration, unregistration)
        }

        pub fn assert_eq_registration(
            &self,
            registration: &Registration,
            tool: &str,
            patterns: &[&str],
        ) {
            assert_eq!(
                *registration,
                Registration {
                    id: format!("watcher-{}-{}", tool, self.worker.get_root_uri().as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                    register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
                        watchers: patterns
                            .iter()
                            .map(|pattern| {
                                FileSystemWatcher {
                                    glob_pattern: GlobPattern::Relative(RelativePattern {
                                        base_uri: OneOf::Right(self.worker.get_root_uri().clone()),
                                        pattern: (*pattern).to_string(),
                                    }),
                                    kind: Some(WatchKind::all()), // created, deleted, changed
                                }
                            })
                            .collect(),
                    })),
                }
            );
        }
    }

    mod init_watchers {
        use crate::{
            formatter::options::FormatOptions, linter::options::LintOptions, options::Options,
            worker::test_watchers::Tester,
        };

        #[test]
        fn test_default_options() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let registrations = tester.init_watchers();

            assert_eq!(registrations.len(), 1);
            tester.assert_eq_registration(&registrations[0], "linter", &["**/.oxlintrc.json"]);
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
            let registrations = tester.init_watchers();

            assert_eq!(registrations.len(), 1);
            tester.assert_eq_registration(&registrations[0], "linter", &["configs/lint.json"]);
        }

        #[test]
        fn test_linter_extends_configs() {
            let tester = Tester::new("fixtures/watcher/linter_extends", &Options::default());
            let registrations = tester.init_watchers();

            // The `.oxlintrc.json` extends `./lint.json -> 2 watchers
            assert_eq!(registrations.len(), 1);
            tester.assert_eq_registration(
                &registrations[0],
                "linter",
                &["**/.oxlintrc.json", "lint.json"],
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
            let registrations = tester.init_watchers();

            assert_eq!(registrations.len(), 1);
            tester.assert_eq_registration(
                &registrations[0],
                "linter",
                &[".oxlintrc.json", "lint.json"],
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
            tester.assert_eq_registration(&watchers[0], "linter", &["**/.oxlintrc.json"]);
            tester.assert_eq_registration(
                &watchers[1],
                "formatter",
                &[".oxfmtrc.json", ".oxfmtrc.jsonc"],
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
            tester.assert_eq_registration(&watchers[0], "linter", &["**/.oxlintrc.json"]);
            tester.assert_eq_registration(&watchers[1], "formatter", &["configs/formatter.json"]);
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
            tester.assert_eq_registration(&watchers[0], "linter", &["configs/lint.json"]);
            tester.assert_eq_registration(&watchers[1], "formatter", &["configs/formatter.json"]);
        }
    }

    mod did_change_configuration {
        use tower_lsp_server::lsp_types::Unregistration;

        use crate::{
            formatter::options::FormatOptions,
            linter::options::{LintOptions, Run},
            options::Options,
            worker::test_watchers::Tester,
        };

        #[test]
        fn test_no_change() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let (registration, unregistrations) =
                tester.did_change_configuration(&Options::default());
            assert!(registration.is_empty());
            assert!(unregistrations.is_empty());
        }

        #[test]
        fn test_lint_config_path_change() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                lint: LintOptions {
                    config_path: Some("configs/lint.json".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            });

            assert_eq!(unregistrations.len(), 1);
            assert_eq!(registration.len(), 1);

            assert_eq!(
                unregistrations[0],
                Unregistration {
                    id: format!("watcher-linter-{}", tester.worker.get_root_uri().as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                }
            );
            tester.assert_eq_registration(&registration[0], "linter", &["configs/lint.json"]);
        }

        #[test]
        fn test_lint_other_option_change() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                // run is the only option that does not require a restart
                lint: LintOptions { run: Run::OnSave, ..Default::default() },
                ..Default::default()
            });
            assert!(unregistrations.is_empty());
            assert!(registration.is_empty());
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
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                format: FormatOptions { experimental: true, ..Default::default() },
                ..Default::default()
            });

            assert!(registration.is_empty());
            assert!(unregistrations.is_empty());
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
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                lint: LintOptions {
                    config_path: Some("configs/lint.json".to_string()),
                    ..Default::default()
                },
                format: FormatOptions { experimental: true, ..Default::default() },
            });

            assert_eq!(unregistrations.len(), 1);
            assert_eq!(
                unregistrations[0],
                Unregistration {
                    id: format!("watcher-linter-{}", tester.worker.get_root_uri().as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                }
            );
            tester.assert_eq_registration(&registration[0], "linter", &["configs/lint.json"]);
        }

        #[test]
        fn test_formatter_experimental_enabled() {
            let tester = Tester::new("fixtures/watcher/default", &Options::default());
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                format: FormatOptions { experimental: true, ..Default::default() },
                ..Default::default()
            });

            assert_eq!(unregistrations.len(), 0);
            assert_eq!(registration.len(), 1);
            tester.assert_eq_registration(
                &registration[0],
                "formatter",
                &[".oxfmtrc.json", ".oxfmtrc.jsonc"],
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
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                format: FormatOptions {
                    experimental: true,
                    config_path: Some("configs/formatter.json".to_string()),
                },
                ..Default::default()
            });

            assert_eq!(unregistrations.len(), 1);
            assert_eq!(registration.len(), 1);
            assert_eq!(
                unregistrations[0],
                Unregistration {
                    id: format!("watcher-formatter-{}", tester.worker.get_root_uri().as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                }
            );

            tester.assert_eq_registration(
                &registration[0],
                "formatter",
                &["configs/formatter.json"],
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
            let (registration, unregistrations) = tester.did_change_configuration(&Options {
                format: FormatOptions { experimental: false, ..Default::default() },
                ..Default::default()
            });

            assert_eq!(unregistrations.len(), 1);
            assert_eq!(registration.len(), 0);
            assert_eq!(
                unregistrations[0],
                Unregistration {
                    id: format!("watcher-formatter-{}", tester.worker.get_root_uri().as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                }
            );
        }
    }
}
