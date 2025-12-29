use log::debug;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::{
    jsonrpc::ErrorCode,
    ls_types::{
        CodeActionKind, CodeActionOrCommand, Diagnostic, DidChangeWatchedFilesRegistrationOptions,
        FileEvent, FileSystemWatcher, GlobPattern, OneOf, Range, Registration, RelativePattern,
        TextEdit, Unregistration, Uri, WatchKind, WorkspaceEdit,
    },
};

use crate::{
    ToolRestartChanges,
    file_system::LSPFileSystem,
    tool::{DiagnosticResult, Tool, ToolBuilder},
};

/// A worker that manages the individual tools for a specific workspace
/// and reports back the results to the [`Backend`](crate::backend::Backend).
///
/// Each worker is responsible for a specific root URI and configures the tools `cwd` to that root URI.
/// The [`Backend`](crate::backend::Backend) is responsible to target the correct worker for a given file URI.
pub struct WorkspaceWorker {
    root_uri: Uri,
    tools: RwLock<Vec<Box<dyn Tool>>>,
    builders: Arc<[Box<dyn ToolBuilder>]>,
    // Initialized options from the client
    // If None, the worker has not been initialized yet
    pub(crate) options: Mutex<Option<serde_json::Value>>,

    // Whether the client is in diagnostic pull mode
    diagnostic_pull_mode: bool,
    // Keep track of published diagnostics to clear them on shutdown (only in push mode)
    published_diagnostics: Mutex<FxHashSet<Uri>>,
}

impl WorkspaceWorker {
    /// Create a new workspace worker.
    /// This will not start any programs, use [`start_worker`](Self::start_worker) for that.
    /// Depending on the client, we need to request the workspace configuration in `initialized` request.
    pub fn new(
        root_uri: Uri,
        builders: Arc<[Box<dyn ToolBuilder>]>,
        diagnostic_pull_mode: bool,
    ) -> Self {
        Self {
            root_uri,
            tools: RwLock::new(vec![]),
            builders,
            options: Mutex::new(None),
            diagnostic_pull_mode,
            published_diagnostics: Mutex::new(FxHashSet::default()),
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
        *self.tools.write().await = self
            .builders
            .iter()
            .map(|builder| builder.build_boxed(&self.root_uri, options.clone()))
            .collect();

        *self.options.lock().await = Some(options);
    }

    /// Initialize file system watchers for the workspace.
    /// These watchers are used to watch for changes in the lint configuration files.
    /// The returned watchers will be registered to the client.
    pub async fn init_watchers(&self) -> Vec<Registration> {
        // clone the options to avoid locking the mutex
        let options_json = { self.options.lock().await.clone().unwrap_or_default() };

        self.tools
            .read()
            .await
            .iter()
            .filter_map(|tool| {
                let patterns = tool.get_watcher_patterns(options_json.clone());
                if patterns.is_empty() {
                    None
                } else {
                    Some(registration_tool_watcher_id(tool.name(), &self.root_uri, patterns))
                }
            })
            .collect()
    }

    /// Check if the worker needs to be initialized with options
    pub async fn needs_init_options(&self) -> bool {
        self.options.lock().await.is_none()
    }

    /// Remove all internal cache for the given URI, if any.
    pub async fn remove_uri_cache(&self, uri: &Uri) {
        self.tools.read().await.iter().for_each(|tool| {
            tool.remove_uri_cache(uri);
        });
    }

    /// Common aggregator for tool-provided diagnostics.
    async fn collect_diagnostics_with<F>(
        &self,
        uri: &Uri,
        content: Option<&str>,
        run: F,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String>
    where
        F: Fn(&Box<dyn Tool>, &Uri, Option<&str>) -> DiagnosticResult,
    {
        let mut aggregated: FxHashMap<Uri, Vec<Diagnostic>> = FxHashMap::default();

        for tool in self.tools.read().await.iter() {
            let tool_diagnostics = run(tool, uri, content);

            match tool_diagnostics {
                Ok(diags) => {
                    for (entry_uri, mut diags) in diags {
                        aggregated.entry(entry_uri).or_default().append(&mut diags);
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        // In push mode, keep track of published diagnostics to clear them on shutdown
        if !self.diagnostic_pull_mode {
            let new_published_uris: FxHashSet<Uri> = aggregated.keys().cloned().collect();
            self.published_diagnostics.lock().await.extend(new_published_uris);
        }

        // In push mode, keep track of published diagnostics to clear them on shutdown
        if !self.diagnostic_pull_mode {
            let new_published_uris: FxHashSet<Uri> = aggregated.keys().cloned().collect();
            self.published_diagnostics.lock().await.extend(new_published_uris);
        }

        let mut result = Vec::with_capacity(aggregated.len());
        for (uri, diags) in aggregated {
            result.push((uri, diags));
        }
        Ok(result)
    }

    /// Run different tools to collect diagnostics.
    pub async fn run_diagnostic(
        &self,
        uri: &Uri,
        content: Option<&str>,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String> {
        self.collect_diagnostics_with(uri, content, |tool, uri, content| {
            tool.run_diagnostic(uri, content)
        })
        .await
    }

    /// Run different tools to collect diagnostics on change.
    pub async fn run_diagnostic_on_change(
        &self,
        uri: &Uri,
        content: Option<&str>,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String> {
        self.collect_diagnostics_with(uri, content, |tool, uri, content| {
            tool.run_diagnostic_on_change(uri, content)
        })
        .await
    }

    /// Run different tools to collect diagnostics on save.
    pub async fn run_diagnostic_on_save(
        &self,
        uri: &Uri,
        content: Option<&str>,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String> {
        self.collect_diagnostics_with(uri, content, |tool, uri, content| {
            tool.run_diagnostic_on_save(uri, content)
        })
        .await
    }

    /// Format a file with the current formatter
    /// - If no file is not formattable or ignored, [`None`] is returned
    /// - If the file is formattable, but no changes are made, an empty vector is returned
    pub async fn format_file(&self, uri: &Uri, content: Option<&str>) -> Option<Vec<TextEdit>> {
        for tool in self.tools.read().await.iter() {
            if let Some(edits) = tool.run_format(uri, content) {
                return Some(edits);
            }
        }
        None
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
        let uris_to_clear_diagnostics =
            self.published_diagnostics.lock().await.drain().collect::<Vec<Uri>>();
        let mut watchers_to_unregister = Vec::new();
        for tool in self.tools.read().await.iter() {
            tool.shutdown();
            watchers_to_unregister
                .push(unregistration_tool_watcher_id(tool.name(), &self.root_uri));
        }

        (uris_to_clear_diagnostics, watchers_to_unregister)
    }

    /// Get code actions or commands for the given range.
    /// It calls all tools and collects their code actions or commands.
    /// If `only_code_action_kinds` is provided, only code actions of the specified kinds are returned.
    pub async fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        range: &Range,
        only_code_action_kinds: Option<Vec<CodeActionKind>>,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();
        for tool in self.tools.read().await.iter() {
            actions.extend(tool.get_code_actions_or_commands(
                uri,
                range,
                only_code_action_kinds.as_ref(),
            ));
        }
        actions
    }

    /// Handle file changes that are watched by the client
    /// At the moment, this only handles changes to lint configuration files
    /// When a change is detected, the linter is refreshed and all diagnostics are revalidated
    pub async fn did_change_watched_files(
        &self,
        file_event: &FileEvent,
        needs_diagnostic_refresh: &mut bool,
        file_system: Option<&LSPFileSystem>,
    ) -> (
        // Diagnostic reports that need to be revalidated
        Option<Vec<(Uri, Vec<Diagnostic>)>>,
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

        self.handle_tool_changes(file_system, needs_diagnostic_refresh, |tool, builder| {
            tool.handle_watched_file_change(
                builder,
                &file_event.uri,
                &self.root_uri,
                options.clone(),
            )
        })
        .await
    }

    /// Handle server configuration changes from the client
    ///
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub async fn did_change_configuration(
        &self,
        changed_options_json: serde_json::Value,
        needs_diagnostic_refresh: &mut bool,
        file_system: Option<&LSPFileSystem>,
    ) -> (
        // Diagnostic reports that need to be revalidated
        Option<Vec<(Uri, Vec<Diagnostic>)>>,
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

        let result = self
            .handle_tool_changes(file_system, needs_diagnostic_refresh, |tool, builder| {
                tool.handle_configuration_change(
                    builder,
                    &self.root_uri,
                    &old_options,
                    changed_options_json.clone(),
                )
            })
            .await;

        {
            let mut options_guard = self.options.lock().await;
            *options_guard = Some(changed_options_json);
        }

        result
    }

    /// Common implementation for handling tool changes that may result in
    /// diagnostics updates, watcher registrations/unregistrations, and tool replacement
    async fn handle_tool_changes<F>(
        &self,
        file_system: Option<&LSPFileSystem>,
        needs_diagnostic_refresh: &mut bool,
        change_handler: F,
    ) -> (Option<Vec<(Uri, Vec<Diagnostic>)>>, Vec<Registration>, Vec<Unregistration>)
    where
        F: Fn(&mut Box<dyn Tool>, &dyn ToolBuilder) -> ToolRestartChanges,
    {
        let mut registrations = vec![];
        let mut unregistrations = vec![];
        let mut diagnostics: Option<Vec<(Uri, Vec<Diagnostic>)>> = None;

        let mut tools = self.tools.write().await;
        debug_assert_eq!(
            tools.len(),
            self.builders.len(),
            "tools and builders must have the same length"
        );
        for (tool, builder) in tools.iter_mut().zip(self.builders.iter()) {
            let builder: &dyn ToolBuilder = builder.as_ref();
            let change = change_handler(tool, builder);

            if let Some(patterns) = change.watch_patterns {
                unregistrations.push(unregistration_tool_watcher_id(tool.name(), &self.root_uri));
                if !patterns.is_empty() {
                    registrations.push(registration_tool_watcher_id(
                        tool.name(),
                        &self.root_uri,
                        patterns,
                    ));
                }
            }
            if let Some(replaced_tool) = change.tool {
                *tool = replaced_tool;
                *needs_diagnostic_refresh = true;

                let Some(file_system) = file_system else {
                    continue;
                };

                for uri in file_system.keys() {
                    let Ok(mut reports) =
                        tool.run_diagnostic(&uri, file_system.get(&uri).as_deref())
                    else {
                        // If diagnostics could not be run, skip this URI, but continue with others
                        // TODO: Should we aggregate errors instead? One by one, or all together?
                        continue;
                    };
                    if !reports.is_empty() {
                        if let Some(existing_diagnostics) = &mut diagnostics {
                            existing_diagnostics.append(&mut reports);
                        } else {
                            diagnostics = Some(reports);
                        }
                    }
                }
            }
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
        for tool in self.tools.read().await.iter() {
            if tool.is_responsible_for_command(command) {
                return tool.execute_command(command, arguments);
            }
        }
        Ok(None)
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

    use std::sync::Arc;
    use tower_lsp_server::ls_types::{CodeActionOrCommand, FileChangeType, FileEvent, Range, Uri};

    use crate::{
        ToolBuilder,
        file_system::LSPFileSystem,
        tests::{FAKE_COMMAND, FakeToolBuilder},
        worker::WorkspaceWorker,
    };

    fn create_builders() -> Arc<[Box<dyn ToolBuilder>]> {
        Arc::new([Box::new(FakeToolBuilder) as Box<dyn ToolBuilder>])
    }

    #[test]
    fn test_get_root_uri() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), Arc::new([]), false);

        assert_eq!(worker.get_root_uri(), &Uri::from_str("file:///root/").unwrap());
    }

    #[test]
    fn test_is_responsible() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///path/to/root").unwrap(),
            Arc::new([]),
            false,
        );

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

    #[tokio::test]
    async fn test_needs_init_options() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), Arc::new([]), false);
        assert!(worker.needs_init_options().await);
        worker.start_worker(serde_json::Value::Null).await;
        assert!(!worker.needs_init_options().await);
    }

    #[tokio::test]
    async fn test_init_watchers() {
        // with one watcher
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        worker.start_worker(serde_json::Value::Null).await;
        let registrations = worker.init_watchers().await;
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].id, "watcher-FakeTool-file:///root/");

        // with no watchers
        let worker_no_watchers =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        worker_no_watchers.start_worker(serde_json::json!({"some_option": true})).await;
        let registrations_no_watchers = worker_no_watchers.init_watchers().await;
        assert_eq!(registrations_no_watchers.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_command() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        worker.start_worker(serde_json::Value::Null).await;

        // Test command not found
        let result = worker.execute_command("unknown.command", vec![]).await;
        assert!(result.is_ok());
        assert!(result.ok().unwrap().is_none());

        // Test command found but no arguments
        let result = worker.execute_command(FAKE_COMMAND, vec![]).await;
        assert!(result.is_ok());
        assert!(result.ok().unwrap().is_none());

        // Test command found with arguments
        let result = worker.execute_command(FAKE_COMMAND, vec![serde_json::Value::Null]).await;
        assert!(result.is_ok());
        assert!(result.ok().unwrap().is_some());
    }

    #[tokio::test]
    async fn test_watched_files_change_notification() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        worker.start_worker(serde_json::Value::Null).await;

        let fs = LSPFileSystem::default();
        fs.set(
            Uri::from_str("file:///root/diagnostics.config").unwrap(),
            "hello world".to_string(),
        );
        let mut needs_diagnostic_refresh = false;

        let (diagnostics, registrations, unregistrations) = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/unknown.file").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                Some(&fs),
            )
            .await;

        // Since FakeToolBuilder does not know about "unknown.file", no diagnostics or registrations are expected
        assert!(diagnostics.is_none());
        assert_eq!(registrations.len(), 0); // No new registrations expected
        assert_eq!(unregistrations.len(), 0); // No unregistrations expected
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let (diagnostics, registrations, unregistrations) = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/watcher.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                Some(&fs),
            )
            .await;

        // Since FakeToolBuilder knows about "watcher.config", registrations are expected
        assert!(diagnostics.is_none());
        assert_eq!(unregistrations.len(), 1); // One unregistration expected
        assert_eq!(unregistrations[0].id, "watcher-FakeTool-file:///root/");
        assert_eq!(registrations.len(), 1); // One new registration expected
        assert_eq!(registrations[0].id, "watcher-FakeTool-file:///root/");
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let (diagnostics, registrations, unregistrations) = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/tool.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                Some(&fs),
            )
            .await;

        // Because we passed a file system that knows about "diagnostics.config", diagnostics are expected
        assert!(diagnostics.is_some());
        assert_eq!(diagnostics.unwrap().len(), 1); // One diagnostic report expected
        assert_eq!(registrations.len(), 0); // No new registrations expected
        assert_eq!(unregistrations.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // Need to refresh diagnostics

        needs_diagnostic_refresh = false;
        let (diagnostics, registrations, unregistrations) = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/tool.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                None,
            )
            .await;

        // No file system passed, so no diagnostics expected
        assert!(diagnostics.is_none());
        assert_eq!(registrations.len(), 0); // No new registrations expected
        assert_eq!(unregistrations.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // Need to refresh diagnostics
    }

    #[tokio::test]
    async fn test_did_change_configuration() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        worker.start_worker(serde_json::json!({"some_option": true})).await;

        let fs = LSPFileSystem::default();
        fs.set(
            Uri::from_str("file:///root/diagnostics.config").unwrap(),
            "hello world".to_string(),
        );
        let mut needs_diagnostic_refresh = false;

        let (diagnostics, registrations, unregistrations) = worker
            .did_change_configuration(
                serde_json::json!({"some_option": false}),
                &mut needs_diagnostic_refresh,
                Some(&fs),
            )
            .await;

        // Since FakeToolBuilder does not change anything based on configuration, no diagnostics or registrations are expected
        assert!(diagnostics.is_none());
        assert_eq!(registrations.len(), 0); // No new registrations expected
        assert_eq!(unregistrations.len(), 0); // No unregistrations expected
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let (diagnostics, registrations, unregistrations) = worker
            .did_change_configuration(
                serde_json::json!(2),
                &mut needs_diagnostic_refresh,
                Some(&fs),
            )
            .await;

        // Since FakeToolBuilder changes watcher patterns based on configuration, registrations are expected
        assert!(diagnostics.is_none());
        assert_eq!(unregistrations.len(), 1); // One unregistration expected
        assert_eq!(unregistrations[0].id, "watcher-FakeTool-file:///root/");
        assert_eq!(registrations.len(), 1); // One new registration expected
        assert_eq!(registrations[0].id, "watcher-FakeTool-file:///root/");
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let (diagnostics, registrations, unregistrations) = worker
            .did_change_configuration(
                serde_json::json!(3),
                &mut needs_diagnostic_refresh,
                Some(&fs),
            )
            .await;

        // Since FakeToolBuilder changes diagnostics based on configuration, diagnostics are expected
        assert!(diagnostics.is_some());
        assert_eq!(diagnostics.unwrap().len(), 1); // One diagnostic report expected
        assert_eq!(registrations.len(), 0); // No new registrations expected
        assert_eq!(unregistrations.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // Need to refresh diagnostics
    }

    #[tokio::test]
    async fn test_code_action_collection() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        worker.start_worker(serde_json::Value::Null).await;

        let actions = worker
            .get_code_actions_or_commands(
                &Uri::from_str("file:///root/file.js").unwrap(),
                &Range::default(),
                None,
            )
            .await;

        assert_eq!(actions.len(), 0);

        let actions = worker
            .get_code_actions_or_commands(
                &Uri::from_str("file:///root/code_action.config").unwrap(),
                &Range::default(),
                None,
            )
            .await;

        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.title, "Code Action title");
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[tokio::test]
    async fn test_run_diagnostic() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        let uri = Uri::from_str("file:///root/diagnostics.config").unwrap();

        worker.start_worker(serde_json::Value::Null).await;

        let diagnostics_no_content = worker.run_diagnostic(&uri, None).await.unwrap();

        assert_eq!(diagnostics_no_content.len(), 1);
        assert_eq!(diagnostics_no_content[0].0, uri);
        assert_eq!(diagnostics_no_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_no_content[0].1[0].message,
            "Fake diagnostic for content: <no content>"
        );

        let diagnostics_with_content =
            worker.run_diagnostic(&uri, Some("helloworld")).await.unwrap();

        assert_eq!(diagnostics_with_content.len(), 1);
        assert_eq!(diagnostics_with_content[0].0, uri);
        assert_eq!(diagnostics_with_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_with_content[0].1[0].message,
            "Fake diagnostic for content: helloworld"
        );

        let no_diagnostics = worker
            .run_diagnostic(&Uri::from_str("file:///root/unknown.file").unwrap(), None)
            .await
            .unwrap();

        assert!(no_diagnostics.is_empty());

        let error = worker
            .run_diagnostic(&Uri::from_str("file:///root/error.config").unwrap(), None)
            .await
            .unwrap_err();

        assert_eq!(error, "Fake diagnostic error");
    }

    #[tokio::test]
    async fn test_run_diagnostic_on_change() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        let uri = Uri::from_str("file:///root/diagnostics.config").unwrap();

        worker.start_worker(serde_json::Value::Null).await;

        let diagnostics_no_content = worker.run_diagnostic_on_change(&uri, None).await.unwrap();

        assert_eq!(diagnostics_no_content.len(), 1);
        assert_eq!(diagnostics_no_content[0].0, uri);
        assert_eq!(diagnostics_no_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_no_content[0].1[0].message,
            "Fake diagnostic for content: <no content>"
        );

        let diagnostics_with_content =
            worker.run_diagnostic_on_change(&uri, Some("helloworld")).await.unwrap();

        assert_eq!(diagnostics_with_content.len(), 1);
        assert_eq!(diagnostics_with_content[0].0, uri);
        assert_eq!(diagnostics_with_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_with_content[0].1[0].message,
            "Fake diagnostic for content: helloworld"
        );

        let no_diagnostics = worker
            .run_diagnostic_on_change(&Uri::from_str("file:///root/unknown.file").unwrap(), None)
            .await
            .unwrap();

        assert!(no_diagnostics.is_empty());

        let error = worker
            .run_diagnostic_on_change(&Uri::from_str("file:///root/error.config").unwrap(), None)
            .await
            .unwrap_err();

        assert_eq!(error, "Fake diagnostic error");
    }

    #[tokio::test]
    async fn test_run_diagnostic_on_save() {
        let worker =
            WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap(), create_builders(), false);
        let uri = Uri::from_str("file:///root/diagnostics.config").unwrap();
        worker.start_worker(serde_json::Value::Null).await;

        let diagnostics_no_content = worker.run_diagnostic_on_save(&uri, None).await.unwrap();

        assert_eq!(diagnostics_no_content.len(), 1);
        assert_eq!(diagnostics_no_content[0].0, uri);
        assert_eq!(diagnostics_no_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_no_content[0].1[0].message,
            "Fake diagnostic for content: <no content>"
        );

        let diagnostics_with_content =
            worker.run_diagnostic_on_save(&uri, Some("helloworld")).await.unwrap();

        assert_eq!(diagnostics_with_content.len(), 1);
        assert_eq!(diagnostics_with_content[0].0, uri);
        assert_eq!(diagnostics_with_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_with_content[0].1[0].message,
            "Fake diagnostic for content: helloworld"
        );

        let no_diagnostics = worker
            .run_diagnostic_on_save(&Uri::from_str("file:///root/unknown.file").unwrap(), None)
            .await
            .unwrap();

        assert!(no_diagnostics.is_empty());

        let error = worker
            .run_diagnostic_on_save(&Uri::from_str("file:///root/error.config").unwrap(), None)
            .await
            .unwrap_err();

        assert_eq!(error, "Fake diagnostic error");
    }
}
