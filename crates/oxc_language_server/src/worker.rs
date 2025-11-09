use log::debug;
use serde_json::json;
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::{
    UriExt,
    jsonrpc::ErrorCode,
    lsp_types::{
        CodeActionKind, CodeActionOrCommand, Diagnostic, DidChangeWatchedFilesRegistrationOptions,
        FileEvent, FileSystemWatcher, GlobPattern, OneOf, Range, Registration, RelativePattern,
        TextEdit, Unregistration, Uri, WatchKind, WorkspaceEdit,
    },
};

use crate::{
    formatter::{ServerFormatter, ServerFormatterBuilder},
    linter::{ServerLinter, ServerLinterBuilder},
    tool::{Tool, ToolBuilder},
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
    // Initialized options from the client
    // If None, the worker has not been initialized yet
    options: Mutex<Option<serde_json::Value>>,
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
    ///
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn is_responsible_for_uri(&self, uri: &Uri) -> bool {
        if let Some(path) = uri.to_file_path() {
            return path.starts_with(self.root_uri.to_file_path().unwrap());
        }
        false
    }

    /// Start all programs (linter, formatter) for the worker.
    /// This should be called after the client has sent the workspace configuration.
    pub async fn start_worker(&self, options: serde_json::Value) {
        *self.options.lock().await = Some(options.clone());
        *self.server_linter.write().await =
            Some(ServerLinterBuilder::new(self.root_uri.clone(), options.clone()).build());

        *self.server_formatter.write().await =
            Some(ServerFormatterBuilder::new(self.root_uri.clone(), options).build());
    }

    /// Initialize file system watchers for the workspace.
    /// These watchers are used to watch for changes in the lint configuration files.
    /// The returned watchers will be registered to the client.
    pub async fn init_watchers(&self) -> Vec<Registration> {
        let mut registrations = Vec::new();

        // clone the options to avoid locking the mutex
        let options_json = { self.options.lock().await.clone().unwrap_or_default() };

        let lint_patterns = self
            .server_linter
            .read()
            .await
            .as_ref()
            .map(|linter| linter.get_watcher_patterns(options_json.clone()));
        let format_patterns = self
            .server_formatter
            .read()
            .await
            .as_ref()
            .map(|formatter| formatter.get_watcher_patterns(options_json));

        if let Some(lint_patterns) = lint_patterns
            && !lint_patterns.is_empty()
        {
            registrations.push(registration_tool_watcher_id(
                "linter",
                &self.root_uri,
                lint_patterns,
            ));
        }

        if let Some(format_patterns) = format_patterns
            && !format_patterns.is_empty()
        {
            registrations.push(registration_tool_watcher_id(
                "formatter",
                &self.root_uri,
                format_patterns,
            ));
        }

        registrations
    }

    /// Check if the worker needs to be initialized with options
    pub async fn needs_init_options(&self) -> bool {
        self.options.lock().await.is_none()
    }

    /// Remove all diagnostics for the given URI
    pub async fn remove_diagnostics(&self, uri: &Uri) {
        let server_linter_guard = self.server_linter.read().await;
        let Some(server_linter) = server_linter_guard.as_ref() else {
            return;
        };
        server_linter.remove_diagnostics(uri);
    }

    /// Run different tools to collect diagnostics.
    pub async fn run_diagnostic(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<Diagnostic>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return None;
        };

        server_linter.run_diagnostic(uri, content).await
    }

    /// Run different tools to collect diagnostics on change.
    pub async fn run_diagnostic_on_change(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<Diagnostic>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return None;
        };

        server_linter.run_diagnostic_on_change(uri, content).await
    }

    /// Run different tools to collect diagnostics on save.
    pub async fn run_diagnostic_on_save(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<Diagnostic>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return None;
        };

        server_linter.run_diagnostic_on_save(uri, content).await
    }

    /// Format a file with the current formatter
    /// - If no file is not formattable or ignored, [`None`] is returned
    /// - If the file is formattable, but no changes are made, an empty vector is returned
    pub async fn format_file(&self, uri: &Uri, content: Option<String>) -> Option<Vec<TextEdit>> {
        let Some(server_formatter) = &*self.server_formatter.read().await else {
            return None;
        };

        server_formatter.run_format(uri, content)
    }

    /// Shutdown the worker and return any necessary changes to be made after shutdown.
    /// This includes clearing diagnostics and unregistering file watchers.
    pub async fn shutdown(
        &self,
    ) -> (
        // The URIs that need to have their diagnostics removed after shutdown
        Vec<Uri>,
        // Watchers that need to be unregistered
        Vec<Unregistration>,
    ) {
        let mut uris_to_clear_diagnostics = Vec::new();

        if let Some(server_linter) = &*self.server_linter.read().await
            && let Some(uris) = server_linter.shutdown().uris_to_clear_diagnostics
        {
            uris_to_clear_diagnostics.extend(uris);
        }

        (
            uris_to_clear_diagnostics,
            vec![
                unregistration_tool_watcher_id("linter", &self.root_uri),
                unregistration_tool_watcher_id("formatter", &self.root_uri),
            ],
        )
    }

    /// Get code actions or commands for the given range
    /// It uses the [`ServerLinter`] cached diagnostics if available, otherwise it will lint the file
    /// If `is_source_fix_all_oxc` is true, it will return a single code action that applies all fixes
    pub async fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        range: &Range,
        only_code_action_kinds: Option<Vec<CodeActionKind>>,
    ) -> Vec<CodeActionOrCommand> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return vec![];
        };

        server_linter.get_code_actions_or_commands(uri, range, only_code_action_kinds).await
    }

    /// Handle file changes that are watched by the client
    /// At the moment, this only handles changes to lint configuration files
    /// When a change is detected, the linter is refreshed and all diagnostics are revalidated
    pub async fn did_change_watched_files(
        &self,
        file_event: &FileEvent,
    ) -> (
        // Diagnostic reports that need to be revalidated
        Option<Vec<(String, Vec<Diagnostic>)>>,
        // New watchers that need to be registered
        Vec<Registration>,
        // Watchers that need to be unregistered
        Vec<Unregistration>,
    ) {
        // Scope the first lock so it is dropped before the second lock
        let options = {
            let options_guard = self.options.lock().await;
            options_guard.clone().unwrap_or_default()
        };

        let mut registrations = vec![];
        let mut unregistrations = vec![];
        let mut diagnostics = None;

        let mut new_formatter = None;
        if let Some(formatter) = self.server_formatter.read().await.as_ref() {
            let format_change = formatter
                .handle_watched_file_change(&file_event.uri, &self.root_uri, options.clone())
                .await;

            new_formatter = format_change.tool;

            if let Some(patterns) = format_change.watch_patterns {
                unregistrations.push(unregistration_tool_watcher_id("formatter", &self.root_uri));
                if !patterns.is_empty() {
                    registrations.push(registration_tool_watcher_id(
                        "formatter",
                        &self.root_uri,
                        patterns,
                    ));
                }
            }
        }
        if let Some(new_formatter) = new_formatter {
            *self.server_formatter.write().await = Some(new_formatter);
        }

        let mut new_linter = None;
        if let Some(linter) = self.server_linter.read().await.as_ref() {
            let lint_change =
                linter.handle_watched_file_change(&file_event.uri, &self.root_uri, options).await;

            new_linter = lint_change.tool;
            diagnostics = lint_change.diagnostic_reports;

            if let Some(patterns) = lint_change.watch_patterns {
                unregistrations.push(unregistration_tool_watcher_id("linter", &self.root_uri));
                if !patterns.is_empty() {
                    registrations.push(registration_tool_watcher_id(
                        "linter",
                        &self.root_uri,
                        patterns,
                    ));
                }
            }
        }

        if let Some(new_linter) = new_linter {
            *self.server_linter.write().await = Some(new_linter);
        }

        (diagnostics, registrations, unregistrations)
    }

    /// Handle server configuration changes from the client
    ///
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub async fn did_change_configuration(
        &self,
        changed_options_json: serde_json::Value,
    ) -> (
        // Diagnostic reports that need to be revalidated
        Option<Vec<(String, Vec<Diagnostic>)>>,
        // New watchers that need to be registered
        Vec<Registration>,
        // Watchers that need to be unregistered
        Vec<Unregistration>,
    ) {
        // Scope the first lock so it is dropped before the second lock
        let old_options = {
            let options_guard = self.options.lock().await;
            options_guard.clone().unwrap_or_default()
        };
        debug!(
            "
        configuration changed:
        incoming: {changed_options_json:?}
        current: {old_options:?}
        "
        );

        {
            let mut options_guard = self.options.lock().await;
            *options_guard = Some(changed_options_json.clone());
        }

        let mut registrations = vec![];
        let mut unregistrations = vec![];
        let mut diagnostics = None;

        let mut new_formatter = None;
        if let Some(formatter) = self.server_formatter.read().await.as_ref() {
            let format_change = formatter
                .handle_configuration_change(
                    &self.root_uri,
                    &old_options,
                    changed_options_json.clone(),
                )
                .await;

            new_formatter = format_change.tool;

            if let Some(patterns) = format_change.watch_patterns {
                unregistrations.push(unregistration_tool_watcher_id("formatter", &self.root_uri));
                if !patterns.is_empty() {
                    registrations.push(registration_tool_watcher_id(
                        "formatter",
                        &self.root_uri,
                        patterns,
                    ));
                }
            }
        }
        if let Some(new_formatter) = new_formatter {
            *self.server_formatter.write().await = Some(new_formatter);
        }

        let mut new_linter = None;
        if let Some(linter) = self.server_linter.read().await.as_ref() {
            let lint_change = linter
                .handle_configuration_change(&self.root_uri, &old_options, changed_options_json)
                .await;

            new_linter = lint_change.tool;
            diagnostics = lint_change.diagnostic_reports;

            if let Some(patterns) = lint_change.watch_patterns {
                unregistrations.push(unregistration_tool_watcher_id("linter", &self.root_uri));
                if !patterns.is_empty() {
                    registrations.push(registration_tool_watcher_id(
                        "linter",
                        &self.root_uri,
                        patterns,
                    ));
                }
            }
        }

        if let Some(new_linter) = new_linter {
            *self.server_linter.write().await = Some(new_linter);
        }

        (diagnostics, registrations, unregistrations)
    }

    /// Execute a command for the workspace.
    /// Currently, only the `oxc.fixAll` command is supported.
    ///
    /// # Errors
    /// Returns `ErrorCode` when the command is found but could not be executed.
    pub async fn execute_command(
        &self,
        command: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return Ok(None);
        };

        if !server_linter.is_responsible_for_command(command) {
            return Ok(None);
        }

        server_linter.execute_command(command, arguments).await
    }
}

/// Create an unregistration for a file system watcher for the given tool
fn unregistration_tool_watcher_id(tool: &str, root_uri: &Uri) -> Unregistration {
    Unregistration {
        id: format!("watcher-{tool}-{}", root_uri.as_str()),
        method: "workspace/didChangeWatchedFiles".to_string(),
    }
}

/// Create a registration for a file system watcher for the given tool and patterns
fn registration_tool_watcher_id(tool: &str, root_uri: &Uri, patterns: Vec<String>) -> Registration {
    Registration {
        id: format!("watcher-{tool}-{}", root_uri.as_str()),
        method: "workspace/didChangeWatchedFiles".to_string(),
        register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
            watchers: patterns
                .into_iter()
                .map(|pattern| FileSystemWatcher {
                    glob_pattern: GlobPattern::Relative(RelativePattern {
                        base_uri: OneOf::Right(root_uri.clone()),
                        pattern,
                    }),
                    kind: Some(WatchKind::all()), // created, deleted, changed
                })
                .collect::<Vec<_>>(),
        })),
    }
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

    use crate::worker::WorkspaceWorker;

    struct Tester {
        pub worker: WorkspaceWorker,
    }

    impl Tester {
        pub fn new(relative_root_dir: &'static str, options: serde_json::Value) -> Self {
            let absolute_path =
                std::env::current_dir().expect("could not get current dir").join(relative_root_dir);
            let uri =
                Uri::from_file_path(absolute_path).expect("could not convert current dir to uri");

            let worker = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { Self::create_workspace_worker(uri, options).await });

            Self { worker }
        }

        async fn create_workspace_worker(
            absolute_path: Uri,
            options: serde_json::Value,
        ) -> WorkspaceWorker {
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
            options: serde_json::Value,
        ) -> (Vec<Registration>, Vec<Unregistration>) {
            let (_, registration, unregistration) = tokio::runtime::Runtime::new()
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
        use serde_json::json;

        use crate::worker::test_watchers::Tester;

        #[test]
        fn test_default_options() {
            let tester = Tester::new("fixtures/watcher/default", json!({}));
            let registrations = tester.init_watchers();

            assert_eq!(registrations.len(), 1);
            tester.assert_eq_registration(&registrations[0], "linter", &["**/.oxlintrc.json"]);
        }

        #[test]
        fn test_custom_config_path() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                json!({
                    "configPath": "configs/lint.json"
                }),
            );
            let registrations = tester.init_watchers();

            assert_eq!(registrations.len(), 1);
            tester.assert_eq_registration(&registrations[0], "linter", &["configs/lint.json"]);
        }

        #[test]
        fn test_linter_extends_configs() {
            let tester = Tester::new("fixtures/watcher/linter_extends", json!({}));
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
                json!({
                    "configPath": ".oxlintrc.json"
                }),
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
                json!({
                    "fmt.experimental": true
                }),
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
                json!({
                    "fmt.experimental": true,
                    "fmt.configPath": "configs/formatter.json"
                }),
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
                json!({
                    "configPath": "configs/lint.json",
                    "fmt.experimental": true,
                    "fmt.configPath": "configs/formatter.json"
                }),
            );
            let watchers = tester.init_watchers();

            assert_eq!(watchers.len(), 2);
            tester.assert_eq_registration(&watchers[0], "linter", &["configs/lint.json"]);
            tester.assert_eq_registration(&watchers[1], "formatter", &["configs/formatter.json"]);
        }
    }

    mod did_change_configuration {
        use serde_json::json;
        use tower_lsp_server::lsp_types::Unregistration;

        use crate::worker::test_watchers::Tester;

        #[test]
        fn test_no_change() {
            let tester = Tester::new("fixtures/watcher/default", json!({}));
            let (registration, unregistrations) = tester.did_change_configuration(json!({}));
            assert!(registration.is_empty());
            assert!(unregistrations.is_empty());
        }

        #[test]
        fn test_lint_config_path_change() {
            let tester = Tester::new("fixtures/watcher/default", json!({}));
            let (registration, unregistrations) = tester.did_change_configuration(json!( {
                "configPath": "configs/lint.json"
            }));

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
            let tester = Tester::new("fixtures/watcher/default", json!({}));
            let (registration, unregistrations) = tester.did_change_configuration(json!({
                // run is the only option that does not require a restart
                "run": "onSave"
            }));
            assert!(unregistrations.is_empty());
            assert!(registration.is_empty());
        }

        #[test]
        fn test_no_changes_with_formatter() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                json!({
                    "fmt.experimental": true,
                }),
            );
            let (registration, unregistrations) = tester.did_change_configuration(json!({
                "fmt.experimental": true
            }));

            assert!(registration.is_empty());
            assert!(unregistrations.is_empty());
        }

        #[test]
        fn test_lint_config_path_change_with_formatter() {
            let tester = Tester::new(
                "fixtures/watcher/default",
                json!({
                  "fmt.experimental": true
                }),
            );
            let (registration, unregistrations) = tester.did_change_configuration(json!( {
                "configPath": "configs/lint.json",
                "fmt.experimental": true
            }));

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
            let tester = Tester::new("fixtures/watcher/default", json!({}));
            let (registration, unregistrations) = tester.did_change_configuration(json!({
                "fmt.experimental": true
            }));

            // The `WorkspaceWorker` does not know if the formatter was previously enabled or not,
            // so it will always unregister the old watcher.
            assert_eq!(unregistrations.len(), 1);
            assert_eq!(
                unregistrations[0],
                Unregistration {
                    id: format!("watcher-formatter-{}", tester.worker.get_root_uri().as_str()),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                }
            );

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
                json!({
                    "fmt.experimental": true
                }),
            );
            let (registration, unregistrations) = tester.did_change_configuration(json!({
                "fmt.experimental": true,
                "fmt.configPath": "configs/formatter.json"
            }));

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
                json!({
                    "fmt.experimental": true
                }),
            );
            let (registration, unregistrations) = tester.did_change_configuration(json!({
                "fmt.experimental": false
            }));

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
