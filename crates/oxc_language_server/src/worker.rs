use std::{path::Path, sync::Arc};

use rustc_hash::FxHashSet;
use serde_json::json;
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::{
    jsonrpc::ErrorCode,
    ls_types::{
        CodeActionOrCommand, Diagnostic, DidChangeWatchedFilesRegistrationOptions, FileEvent,
        FileSystemWatcher, GlobPattern, OneOf, Pattern, Registration, RelativePattern, TextEdit,
        Unregistration, Uri, WatchKind, WorkspaceEdit,
    },
};
use tracing::debug;

use crate::{
    BuildContext, CodeActionParams, TextDocument, ToolRestartChanges,
    capabilities::DiagnosticMode,
    file_system::{LSPFileSystem, ResolvedPath},
    tool::{ClientMessage, DiagnosticResult, Tool, ToolBuilder},
    utils::matches_watcher_patterns,
    working_directories::is_below_ignored_directory,
};

pub struct WorkerToolChangeResult {
    /// Diagnostic reports that need to be revalidated
    pub diagnostics: Option<Vec<(Uri, Vec<Diagnostic>)>>,
    /// New watchers that need to be registered
    pub new_watchers: Vec<Registration>,
    /// Watchers that need to be unregistered
    pub removed_watchers: Vec<Unregistration>,
    /// Optional messages to be sent to the client, e.g., for misconfiguration
    pub client_messages: Vec<ClientMessage>,
    /// Whether the tool was replaced. A caller which also wants to rebuild the tool, because the
    /// build context changed, can skip it when the tool restarted anyway.
    pub tool_restarted: bool,
}

/// A worker that manages the individual tool for a specific workspace
/// and returns back the results of the running tool.
///
/// Each worker is responsible for a specific root URI and configures the tool's `cwd` to that root URI.
/// The [`WorkerManager`](crate::worker_manager::WorkerManager) is responsible to target the correct worker for a given file URI.
pub struct WorkspaceWorker {
    root_uri: Uri,
    // The normalized path of `root_uri`, resolved once: the routing compares it for every
    // `textDocument/*` notification.
    root_path: Option<ResolvedPath>,
    // The workspace folder this worker belongs to, when it was created for a `workingDirectories`
    // entry. `None` for a worker created from a client workspace folder.
    parent_uri: Option<Uri>,
    // the normalized path of `parent_uri`, resolved once
    parent_path: Option<ResolvedPath>,
    tool: RwLock<Option<Box<dyn Tool>>>,
    builder: Arc<dyn ToolBuilder>,
    // Initialized options from the client
    // If None, the worker has not been initialized yet
    pub(crate) options: Mutex<Option<serde_json::Value>>,
    // The resolved `workingDirectories` of this worker. They are owned by their own sub worker,
    // so the tool of this worker must not eagerly load anything below them.
    working_directories: RwLock<WorkingDirectoriesState>,

    // Whether the client is in diagnostic pull mode / push mode, or not supporting diagnostics at all
    diagnostic_mode: DiagnosticMode,
    // Keep track of published diagnostics to clear them on shutdown (only in push mode)
    published_diagnostics: Mutex<FxHashSet<Uri>>,
}

#[derive(Default)]
struct WorkingDirectoriesState {
    roots: Vec<Uri>,
    // the same roots, resolved once: `watches_uri` compares them for every watched file event
    resolved_roots: Vec<ResolvedPath>,
    // `workingDirectories: [{ "mode": "auto" }]` detects project roots from `package.json` files,
    // so they have to be watched in addition to the tool configuration files.
    auto: bool,
}

impl WorkspaceWorker {
    /// Create a new workspace worker.
    /// This will not start any programs, use [`start_worker`](Self::start_worker) for that.
    /// Depending on the client, we need to request the workspace configuration in `initialized` request.
    pub fn new(
        root_uri: Uri,
        builder: Arc<dyn ToolBuilder>,
        diagnostic_mode: DiagnosticMode,
    ) -> Self {
        Self {
            root_path: ResolvedPath::try_from(&root_uri).ok(),
            root_uri,
            parent_uri: None,
            parent_path: None,
            tool: RwLock::new(None),
            builder,
            options: Mutex::new(None),
            working_directories: RwLock::new(WorkingDirectoriesState::default()),
            diagnostic_mode,
            published_diagnostics: Mutex::new(FxHashSet::default()),
        }
    }

    /// Create a worker for a `workingDirectories` entry of the workspace folder `parent_uri`.
    ///
    /// A sub worker is an ordinary worker: it uses its own root as the tool working directory and
    /// resolves its configuration from there. It deliberately does **not** inherit the resolved
    /// configuration of its workspace folder, so a working directory behaves exactly like opening
    /// that directory as a workspace folder.
    pub fn new_sub_worker(
        root_uri: Uri,
        parent_uri: Uri,
        builder: Arc<dyn ToolBuilder>,
        diagnostic_mode: DiagnosticMode,
    ) -> Self {
        let mut worker = Self::new(root_uri, builder, diagnostic_mode);
        worker.parent_path = ResolvedPath::try_from(&parent_uri).ok();
        worker.parent_uri = Some(parent_uri);
        worker
    }

    /// Get the root URI of the worker
    pub fn get_root_uri(&self) -> &Uri {
        &self.root_uri
    }

    /// The normalized path of the root, `None` for a non `file://` root.
    pub(crate) fn get_root_path(&self) -> Option<&Path> {
        self.root_path.as_ref().map(ResolvedPath::as_path)
    }

    /// Get the workspace folder URI of a sub worker, `None` for a workspace folder worker.
    pub fn get_parent_uri(&self) -> Option<&Uri> {
        self.parent_uri.as_ref()
    }

    /// Whether this worker was created for a `workingDirectories` entry.
    pub fn is_sub_worker(&self) -> bool {
        self.parent_uri.is_some()
    }

    /// Whether the root of this worker is `path`, comparing the normalized paths resolved once.
    pub fn has_root_path(&self, path: Option<&Path>) -> bool {
        match (self.get_root_path(), path) {
            (Some(root), Some(path)) => root == path,
            // without a path there is nothing to compare
            (None, None) => true,
            _ => false,
        }
    }

    /// Whether the workspace folder of this sub worker is rooted at `path`.
    pub fn has_parent_path(&self, path: Option<&Path>) -> bool {
        match (self.parent_path.as_ref(), path) {
            (Some(parent), Some(path)) => parent.as_path() == path,
            _ => false,
        }
    }

    /// Get the current options of the worker, [`serde_json::Value::Null`] when not started yet.
    pub async fn get_options(&self) -> serde_json::Value {
        self.options.lock().await.clone().unwrap_or_default()
    }

    /// Set the resolved `workingDirectories` of this worker.
    /// Must be called before [`start_worker`](Self::start_worker) so the tool can exclude them.
    ///
    /// Returns `true` when the `auto` flag flipped, which changes the watcher patterns of this
    /// worker independently of what the tool decides.
    pub async fn set_working_directories(&self, roots: Vec<Uri>, auto: bool) -> bool {
        let resolved_roots =
            roots.iter().filter_map(|root| ResolvedPath::try_from(root).ok()).collect();

        let mut state = self.working_directories.write().await;
        let auto_changed = state.auto != auto;
        *state = WorkingDirectoriesState { roots, resolved_roots, auto };
        auto_changed
    }

    /// Get the resolved `workingDirectories` of this worker.
    pub async fn get_working_directories(&self) -> Vec<Uri> {
        self.working_directories.read().await.roots.clone()
    }

    /// Additional watcher patterns which are not provided by the tool.
    async fn extra_watcher_patterns(&self) -> Vec<String> {
        if self.working_directories.read().await.auto {
            // creating or deleting a `package.json` can add or remove a project root
            vec!["**/package.json".to_string()]
        } else {
            vec![]
        }
    }

    /// Start all programs (linter, formatter) for the worker.
    /// This should be called after the client has sent the workspace configuration.
    ///
    /// Returns messages to be sent to the client.
    pub async fn start_worker(&self, options: serde_json::Value) -> Vec<ClientMessage> {
        let excluded_roots = self.get_working_directories().await;
        let result = self.builder.build_with_context(
            &self.root_uri,
            options.clone(),
            BuildContext { excluded_roots: &excluded_roots, parent_root: self.parent_uri.as_ref() },
        );
        *self.tool.write().await = Some(result.tool);

        *self.options.lock().await = Some(options);

        result.client_messages
    }

    /// Initialize file system watchers for the workspace.
    /// These watchers are used to watch for changes in the lint configuration files.
    /// The returned watchers will be registered to the client.
    pub async fn init_watchers(&self) -> Option<Registration> {
        let patterns = self.watcher_patterns().await;
        if patterns.is_empty() {
            None
        } else {
            Some(registration_watcher_id(&self.root_uri, patterns))
        }
    }

    /// The watcher patterns this worker registers: the ones of its tool plus the ones it adds for
    /// itself.
    pub async fn watcher_patterns(&self) -> Vec<Pattern> {
        let mut patterns = self.tool_watcher_patterns().await;
        patterns.extend(self.extra_watcher_patterns().await);
        patterns
    }

    /// The watcher patterns of the tool alone, without the ones this worker adds for itself.
    async fn tool_watcher_patterns(&self) -> Vec<Pattern> {
        let options = self.get_options().await;
        self.tool
            .read()
            .await
            .as_ref()
            .map(|tool| tool.get_watcher_patterns(options))
            .unwrap_or_default()
    }

    /// Rebuild the tool of this worker with its current options.
    ///
    /// Needed when the [`BuildContext`] changed without the options changing, which happens when
    /// the `workingDirectories` of this worker are added or removed: the tool has to be rebuilt
    /// with the new set of excluded roots, otherwise it keeps the configs and ignore files of a
    /// directory which is now owned by another worker (or misses the ones it just took back).
    /// Returns the messages for the client and, when the watcher patterns of the rebuilt tool
    /// differ, the registrations which have to replace the current ones. Losing or gaining a
    /// config which is extended from outside the root changes those patterns.
    pub async fn rebuild_tool(
        &self,
    ) -> (Vec<ClientMessage>, Vec<Unregistration>, Vec<Registration>) {
        let extra_patterns = self.extra_watcher_patterns().await;

        // Lock order is the tool first, then the options and the working directories, the same way
        // `handle_tool_changes` does it. Both are read *inside* the critical section: a
        // configuration change or a reconciliation which lands in between is either fully before
        // or fully after this rebuild, so the tool is never built from options or excluded roots
        // which were already replaced while this rebuild waited for the lock.
        let (client_messages, patterns_before, patterns_after) = {
            let mut tool_guard = self.tool.write().await;
            let options = self.options.lock().await.clone().unwrap_or_default();
            let excluded_roots = self.working_directories.read().await.roots.clone();

            let mut patterns_before = tool_guard
                .as_ref()
                .map(|tool| tool.get_watcher_patterns(options.clone()))
                .unwrap_or_default();
            patterns_before.extend(extra_patterns.clone());

            self.builder.shutdown(&self.root_uri);
            let result = self.builder.build_with_context(
                &self.root_uri,
                options.clone(),
                BuildContext {
                    excluded_roots: &excluded_roots,
                    parent_root: self.parent_uri.as_ref(),
                },
            );

            let mut patterns_after = result.tool.get_watcher_patterns(options);
            patterns_after.extend(extra_patterns);
            *tool_guard = Some(result.tool);

            (result.client_messages, patterns_before, patterns_after)
        };

        if patterns_before == patterns_after {
            return (client_messages, vec![], vec![]);
        }

        let unregistrations = vec![unregistration_watcher_id(&self.root_uri)];
        let registrations = if patterns_after.is_empty() {
            vec![]
        } else {
            vec![registration_watcher_id(&self.root_uri, patterns_after)]
        };

        (client_messages, unregistrations, registrations)
    }

    /// Whether this worker has to see a watched file change.
    ///
    /// True when the file is inside its root, when it lives in a directory above its root (a
    /// config there can govern this worker), or when one of its absolute watcher patterns matches
    /// it, which is how a tool registers the config files it extends from outside its root.
    ///
    /// A file below one of the `workingDirectories` of this worker belongs to that worker, and a
    /// file below `node_modules` or `.git` is noise unless the tool explicitly watches it, for
    /// example a config extended from a dependency.
    pub async fn watches_uri(&self, uri: &Uri) -> bool {
        let (Ok(file), Some(root_path)) = (ResolvedPath::try_from(uri), self.get_root_path())
        else {
            // without a path there is nothing to compare, be conservative
            return true;
        };
        let file_path = file.as_path();

        // Below `node_modules` or `.git` only an absolute pattern counts: a relative glob like
        // `**/.oxlintrc.json` matches every dependency, an absolute one is a config this tool
        // explicitly extends from a dependency. A file below one of the working directories of
        // this worker belongs to that worker, for the same reason.
        if is_below_ignored_directory(root_path, file_path)
            || self.owns_excluded_path(file_path).await
        {
            return self.matches_absolute_pattern(file_path).await;
        }

        if file_path.starts_with(root_path) {
            return true;
        }
        if file_path.parent().is_some_and(|parent| root_path.starts_with(parent)) {
            return true;
        }

        // only now are the patterns of the tool needed
        self.matches_absolute_pattern(file_path).await
    }

    /// Whether an absolute watcher pattern of the tool matches the path, which is how a config
    /// extended from outside the root is watched.
    async fn matches_absolute_pattern(&self, file_path: &Path) -> bool {
        matches_watcher_patterns(&self.tool_watcher_patterns().await, file_path, file_path, true)
    }

    /// Whether the path is below one of the `workingDirectories` of this worker, which means it is
    /// owned by that worker and not by this one.
    async fn owns_excluded_path(&self, file_path: &Path) -> bool {
        self.working_directories
            .read()
            .await
            .resolved_roots
            .iter()
            .any(|excluded| file_path.starts_with(excluded.as_path()))
    }

    /// Whether the event matches a pattern this worker registered for itself, and not for its tool.
    ///
    /// The `auto` detection watches `**/package.json`, which is not a tool configuration file: the
    /// tool must not be restarted for it, in this worker or in any of its sub workers.
    pub async fn matches_detection_pattern(&self, uri: &Uri) -> bool {
        let extra_patterns = self.extra_watcher_patterns().await;
        if extra_patterns.is_empty() {
            return false;
        }

        let (Ok(file), Some(root_path)) = (ResolvedPath::try_from(uri), self.get_root_path())
        else {
            return false;
        };

        matches_watcher_patterns(&extra_patterns, file.as_path(), root_path, false)
    }

    /// Whether the tool of this worker watches the given URI.
    pub async fn matches_tool_pattern(&self, uri: &Uri) -> bool {
        let (Ok(file), Some(root_path)) = (ResolvedPath::try_from(uri), self.get_root_path())
        else {
            return false;
        };

        matches_watcher_patterns(
            &self.tool_watcher_patterns().await,
            file.as_path(),
            root_path,
            false,
        )
    }

    /// Re-register the watchers of this worker when its patterns differ from `patterns_before`.
    ///
    /// A tool which restarted on its own reports no watcher change, but its patterns can depend on
    /// the excluded roots it was rebuilt with: a config extended from outside the root is gained
    /// or lost with the directory holding it.
    pub async fn refresh_watchers_if_changed(
        &self,
        patterns_before: &[Pattern],
    ) -> (Vec<Unregistration>, Vec<Registration>) {
        if self.watcher_patterns().await == patterns_before {
            return (vec![], vec![]);
        }

        self.refresh_watchers().await
    }

    /// Force the file system watchers of this worker to be re-registered.
    ///
    /// Needed when the watcher patterns of the *worker* changed (the `workingDirectories` `auto`
    /// flag) while the tool itself did not request a watcher update.
    pub async fn refresh_watchers(&self) -> (Vec<Unregistration>, Vec<Registration>) {
        let unregistrations = vec![unregistration_watcher_id(&self.root_uri)];
        let registrations = self.init_watchers().await.into_iter().collect();
        (unregistrations, registrations)
    }

    /// Check if the worker needs to be initialized with options
    pub async fn needs_init_options(&self) -> bool {
        self.options.lock().await.is_none()
    }

    /// Remove all internal cache for the given URI, if any.
    pub async fn remove_uri_cache(&self, uri: &Uri) {
        if let Some(tool) = self.tool.read().await.as_ref() {
            tool.remove_uri_cache(uri);
        }
    }

    /// Common aggregator for tool-provided diagnostics.
    async fn collect_diagnostics_with<F>(
        &self,
        document: TextDocument<'_>,
        run: F,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String>
    where
        F: Fn(&Box<dyn Tool>, TextDocument) -> DiagnosticResult,
    {
        let tool_diagnostics = {
            let tool_guard = self.tool.read().await;
            let Some(tool) = tool_guard.as_ref() else {
                return Ok(Vec::new());
            };

            run(tool, document)
        };

        let diagnostics = match tool_diagnostics {
            Ok(diags) => diags,
            Err(err) => {
                return Err(err);
            }
        };

        // In push mode, keep track of published diagnostics to clear them on shutdown
        if self.diagnostic_mode == DiagnosticMode::Push {
            self.published_diagnostics
                .lock()
                .await
                .extend(diagnostics.iter().map(|(uri, _)| uri.clone()));
        }

        Ok(diagnostics)
    }

    /// Run different tools to collect diagnostics.
    ///
    /// # Errors
    /// When calling `Tool::run_diagnostic` results into an error.
    pub async fn run_diagnostic(
        &self,
        document: TextDocument<'_>,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String> {
        self.collect_diagnostics_with(document, |tool, document| tool.run_diagnostic(document))
            .await
    }

    /// Run different tools to collect diagnostics on change.
    ///
    /// # Errors
    /// When calling `Tool::run_diagnostic_on_change` results into an error.
    pub async fn run_diagnostic_on_change(
        &self,
        document: TextDocument<'_>,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String> {
        self.collect_diagnostics_with(document, |tool, document| {
            tool.run_diagnostic_on_change(document)
        })
        .await
    }

    /// Run different tools to collect diagnostics on save.
    ///
    /// # Errors
    /// When calling `Tool::run_diagnostic_on_save` results into an error.
    pub async fn run_diagnostic_on_save(
        &self,
        document: TextDocument<'_>,
    ) -> Result<Vec<(Uri, Vec<Diagnostic>)>, String> {
        self.collect_diagnostics_with(document, |tool, document| {
            tool.run_diagnostic_on_save(document)
        })
        .await
    }

    /// Format a file with the current formatter
    /// - If the file is not formattable or is ignored, an empty vector is returned
    /// - If the file is formattable, but no changes are made, an empty vector is returned
    /// - If a tool error occurs, an Err is returned
    ///
    /// # Errors
    /// When calling `Tool::run_format` results into an error.
    pub async fn format_file(&self, document: TextDocument<'_>) -> Result<Vec<TextEdit>, String> {
        let tool_guard = self.tool.read().await;
        let Some(tool) = tool_guard.as_ref() else {
            return Ok(Vec::new());
        };

        tool.run_format(document)
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

        self.builder.shutdown(&self.root_uri);
        watchers_to_unregister.push(unregistration_watcher_id(&self.root_uri));

        (uris_to_clear_diagnostics, watchers_to_unregister)
    }

    /// Get code actions or commands for the given request.
    /// It calls all tools and collects their code actions or commands.
    pub async fn get_code_actions_or_commands(
        &self,
        params: CodeActionParams,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();
        if let Some(tool) = self.tool.read().await.as_ref() {
            actions.extend(tool.get_code_actions_or_commands(params));
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
        owned_uris: &[Uri],
    ) -> WorkerToolChangeResult {
        // Scope the first lock so it is dropped before the second lock
        let options = {
            let options_guard = self.options.lock().await;
            options_guard.clone().unwrap_or_default()
        };

        self.handle_tool_changes(
            file_system,
            owned_uris,
            needs_diagnostic_refresh,
            move |tool, builder, context| {
                tool.handle_watched_file_change(
                    builder,
                    &file_event.uri,
                    &self.root_uri,
                    context,
                    options,
                )
            },
        )
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
        owned_uris: &[Uri],
    ) -> WorkerToolChangeResult {
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
            .handle_tool_changes(
                file_system,
                owned_uris,
                needs_diagnostic_refresh,
                |tool, builder, context| {
                    tool.handle_configuration_change(
                        builder,
                        &self.root_uri,
                        context,
                        &old_options,
                        changed_options_json.clone(),
                    )
                },
            )
            .await;

        {
            let mut options_guard = self.options.lock().await;
            *options_guard = Some(changed_options_json);
        }

        result
    }

    /// Common implementation for handling tool changes that may result in
    /// diagnostics updates, watcher registrations/unregistrations, and tool replacement
    ///
    /// `owned_uris` holds the open documents this worker is responsible for, as decided by the
    /// [`WorkerManager`](crate::worker_manager::WorkerManager) routing. A file below one of the
    /// `workingDirectories` of this worker (or below a nested workspace folder) belongs to that
    /// other worker and must not be linted twice, with two different configurations.
    async fn handle_tool_changes<F>(
        &self,
        file_system: Option<&LSPFileSystem>,
        owned_uris: &[Uri],
        needs_diagnostic_refresh: &mut bool,
        change_handler: F,
    ) -> WorkerToolChangeResult
    where
        F: FnOnce(&mut Box<dyn Tool>, &dyn ToolBuilder, BuildContext<'_>) -> ToolRestartChanges,
    {
        let mut registrations = vec![];
        let mut unregistrations = vec![];
        let mut diagnostics: Option<Vec<(Uri, Vec<Diagnostic>)>> = None;

        let extra_patterns = self.extra_watcher_patterns().await;

        let mut tools = self.tool.write().await;
        // The working directories are read *inside* the critical section, like the options are in
        // `rebuild_tool`: a reconciliation which lands while this handler waits for the tool lock
        // must not be undone by a tool built from the excluded roots it started from.
        let excluded_roots = self.working_directories.read().await.roots.clone();
        let context =
            BuildContext { excluded_roots: &excluded_roots, parent_root: self.parent_uri.as_ref() };
        let Some(tool) = tools.as_mut() else {
            // No tool to update, return early
            return WorkerToolChangeResult {
                diagnostics: None,
                new_watchers: registrations,
                removed_watchers: unregistrations,
                client_messages: Vec::new(), // TODO: Should we return a message to the client if the tool is not initialized?
                tool_restarted: false,
            };
        };
        let change = change_handler(tool, self.builder.as_ref(), context);

        if let Some(mut patterns) = change.watch_patterns {
            unregistrations.push(unregistration_watcher_id(&self.root_uri));
            patterns.extend(extra_patterns);
            if !patterns.is_empty() {
                registrations.push(registration_watcher_id(&self.root_uri, patterns));
            }
        }
        let tool_restarted = change.tool.is_some();
        if let Some(replaced_tool) = change.tool {
            *tool = replaced_tool;
            *needs_diagnostic_refresh = true;

            let Some(file_system) = file_system else {
                return WorkerToolChangeResult {
                    diagnostics: None,
                    new_watchers: registrations,
                    removed_watchers: unregistrations,
                    client_messages: change.client_messages,
                    tool_restarted,
                };
            };

            for uri in owned_uris {
                let document = file_system.get_document(uri);
                let Ok(mut reports) = tool.run_diagnostic(document) else {
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

        WorkerToolChangeResult {
            diagnostics,
            new_watchers: registrations,
            removed_watchers: unregistrations,
            client_messages: change.client_messages,
            tool_restarted,
        }
    }

    /// Execute a command for the workspace.
    ///
    /// # Errors
    /// Returns `ErrorCode` when the command is not found or could not be executed.
    pub async fn execute_command(
        &self,
        command: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        let tool_guard = self.tool.read().await;
        let Some(tool) = tool_guard.as_ref() else {
            return Ok(None);
        };
        tool.execute_command(command, arguments)
    }
}

/// Create an unregistration for a file system watcher
fn unregistration_watcher_id(root_uri: &Uri) -> Unregistration {
    Unregistration {
        id: format!("watcher-{}", root_uri.as_str()),
        method: "workspace/didChangeWatchedFiles".to_string(),
    }
}

/// Create a registration for a file system watcher for the given patterns
fn registration_watcher_id(root_uri: &Uri, patterns: Vec<String>) -> Registration {
    Registration {
        id: format!("watcher-{}", root_uri.as_str()),
        method: "workspace/didChangeWatchedFiles".to_string(),
        register_options: Some(json!(DidChangeWatchedFilesRegistrationOptions {
            watchers: patterns
                .into_iter()
                .map(|pattern| {
                    let glob_pattern = if Path::new(&pattern).is_absolute() {
                        GlobPattern::String(pattern)
                    } else {
                        GlobPattern::Relative(RelativePattern {
                            base_uri: OneOf::Right(root_uri.clone()),
                            pattern,
                        })
                    };
                    FileSystemWatcher {
                        glob_pattern,
                        kind: Some(WatchKind::all()), // created, deleted, changed
                    }
                })
                .collect::<Vec<_>>(),
        })),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use std::sync::Arc;
    use tower_lsp_server::{
        jsonrpc::ErrorCode,
        ls_types::{
            CodeActionContext, CodeActionOrCommand, FileChangeType, FileEvent, MessageType, Range,
            Uri,
        },
    };

    #[cfg(unix)]
    use tower_lsp_server::ls_types::{DidChangeWatchedFilesRegistrationOptions, GlobPattern};

    use crate::{
        ClientMessage, CodeActionParams, LanguageId, TextDocument, ToolBuilder,
        capabilities::DiagnosticMode,
        file_system::LSPFileSystem,
        tests::{FAKE_COMMAND, FakeToolBuilder, FakeToolDelays},
        worker::WorkspaceWorker,
    };

    fn create_builder() -> Arc<dyn ToolBuilder> {
        Arc::new(FakeToolBuilder::default()) as Arc<dyn ToolBuilder>
    }

    #[test]
    fn test_get_root_uri() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );

        assert_eq!(worker.get_root_uri(), &Uri::from_str("file:///root/").unwrap());
    }

    #[tokio::test]
    async fn test_needs_init_options() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        assert!(worker.needs_init_options().await);
        worker.start_worker(serde_json::Value::Null).await;
        assert!(!worker.needs_init_options().await);
    }

    #[tokio::test]
    async fn test_init_watchers() {
        // with one watcher
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::Value::Null).await;
        let registration = worker.init_watchers().await;
        assert!(registration.is_some());
        let registration = registration.unwrap();
        assert_eq!(registration.id, "watcher-file:///root/");

        // with no watchers
        let worker_no_watchers = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker_no_watchers.start_worker(serde_json::json!({"some_option": true})).await;
        let registration_no_watchers = worker_no_watchers.init_watchers().await;
        assert!(registration_no_watchers.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_registration_watcher_absolute_pattern() {
        let root_uri = Uri::from_str("file:///root/").unwrap();
        let registration =
            super::registration_watcher_id(&root_uri, vec!["/etc/**/*.json".to_string()]);

        let register_options = registration.register_options.unwrap();
        let options: DidChangeWatchedFilesRegistrationOptions =
            serde_json::from_value(register_options).unwrap();

        assert_eq!(options.watchers.len(), 1);
        match &options.watchers[0].glob_pattern {
            GlobPattern::String(pattern) => assert_eq!(pattern, "/etc/**/*.json"),
            GlobPattern::Relative(_) => {
                panic!("Expected absolute glob to be encoded as GlobPattern::String")
            }
        }
    }

    /// A rebuild holds the tool lock while it builds and reads the options inside it, so a
    /// configuration change which lands in between is not overwritten with the options the
    /// rebuild started from.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rebuild_does_not_overwrite_a_concurrent_configuration_change() {
        let worker = Arc::new(WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            Arc::new(FakeToolBuilder::default().with_delays(FakeToolDelays::with_build(50))),
            DiagnosticMode::None,
        ));
        worker.start_worker(serde_json::json!({ "version": 1 })).await;

        let rebuilding = Arc::clone(&worker);
        let rebuild = tokio::spawn(async move { rebuilding.rebuild_tool().await });

        // let the rebuild enter its critical section before the configuration changes
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let mut needs_diagnostic_refresh = false;
        worker
            .did_change_configuration(
                serde_json::json!({ "version": 2 }),
                &mut needs_diagnostic_refresh,
                None,
                &[],
            )
            .await;

        rebuild.await.unwrap();

        assert_eq!(worker.get_options().await, serde_json::json!({ "version": 2 }));
    }

    /// A rebuild reads the working directories inside the tool critical section, the same way it
    /// reads the options: a reconciliation which lands while this rebuild waits for the lock must
    /// not be undone by a tool built from the excluded roots it started from.
    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_rebuild_does_not_use_stale_working_directories() {
        let contexts = Arc::new(std::sync::Mutex::new(vec![]));
        let worker = Arc::new(WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            Arc::new(
                FakeToolBuilder::default()
                    .with_delays(FakeToolDelays::with_build(100))
                    .with_build_context_tracking(Arc::clone(&contexts)),
            ),
            DiagnosticMode::None,
        ));
        worker
            .set_working_directories(
                vec![Uri::from_str("file:///root/packages/a/").unwrap()],
                false,
            )
            .await;
        worker.start_worker(serde_json::json!({ "version": 1 })).await;

        // the first rebuild holds the tool lock while it builds
        let first = Arc::clone(&worker);
        let first = tokio::spawn(async move { first.rebuild_tool().await });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // the second one queues behind it, before the working directories change
        let second = Arc::clone(&worker);
        let second = tokio::spawn(async move { second.rebuild_tool().await });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // a reconciliation takes the working directory back while the second rebuild waits
        worker.set_working_directories(vec![], false).await;

        first.await.unwrap();
        second.await.unwrap();

        let contexts = contexts.lock().unwrap().clone();
        // the initial build, the first rebuild, then the one which waited: it has to see the
        // roots as they are now, not the ones it would have read before the lock
        assert_eq!(contexts, vec![1, 1, 0]);
    }

    #[tokio::test]
    async fn test_execute_command() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::Value::Null).await;

        // Test command not found
        let result = worker.execute_command("unknown.command", vec![]).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ErrorCode::InvalidParams);

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
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::Value::Null).await;

        let fs = LSPFileSystem::default();
        fs.set(
            Uri::from_str("file:///root/diagnostics.config").unwrap(),
            "hello world".to_string(),
        );
        let mut needs_diagnostic_refresh = false;

        let result = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/unknown.file").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                Some(&fs),
                &fs.keys(),
            )
            .await;

        // Since FakeToolBuilder does not know about "unknown.file", no diagnostics or registrations are expected
        assert!(result.diagnostics.is_none());
        assert_eq!(result.new_watchers.len(), 0); // No new registrations expected
        assert_eq!(result.removed_watchers.len(), 0); // No unregistrations expected
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let result = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/watcher.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                Some(&fs),
                &fs.keys(),
            )
            .await;

        // Since FakeToolBuilder knows about "watcher.config", registrations are expected
        assert!(result.diagnostics.is_none());
        assert_eq!(result.removed_watchers.len(), 1); // One unregistration expected
        assert_eq!(result.removed_watchers[0].id, "watcher-file:///root/");
        assert_eq!(result.new_watchers.len(), 1); // One new registration expected
        assert_eq!(result.new_watchers[0].id, "watcher-file:///root/");
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let result = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/tool.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                Some(&fs),
                &fs.keys(),
            )
            .await;

        // Because we passed a file system that knows about "diagnostics.config", diagnostics are expected
        assert!(result.diagnostics.is_some());
        assert_eq!(result.diagnostics.unwrap().len(), 1); // One diagnostic report expected
        assert_eq!(result.new_watchers.len(), 0); // No new registrations expected
        assert_eq!(result.removed_watchers.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // Need to refresh diagnostics

        needs_diagnostic_refresh = false;
        let result = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/tool.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                None,
                &[],
            )
            .await;

        // No file system passed, so no diagnostics expected
        assert!(result.diagnostics.is_none());
        assert_eq!(result.new_watchers.len(), 0); // No new registrations expected
        assert_eq!(result.removed_watchers.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // Need to refresh diagnostics
    }

    #[tokio::test]
    async fn test_did_change_configuration() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::json!({"some_option": true})).await;

        let fs = LSPFileSystem::default();
        fs.set(
            Uri::from_str("file:///root/diagnostics.config").unwrap(),
            "hello world".to_string(),
        );
        let mut needs_diagnostic_refresh = false;

        let result = worker
            .did_change_configuration(
                serde_json::json!({"some_option": false}),
                &mut needs_diagnostic_refresh,
                Some(&fs),
                &fs.keys(),
            )
            .await;

        // `FakeTool` restarts for a changed object option, like a real tool does, so the
        // documents it owns are linted again with the new tool
        assert_eq!(result.diagnostics.map(|diagnostics| diagnostics.len()), Some(1));
        assert_eq!(result.new_watchers.len(), 0); // No new registrations expected
        assert_eq!(result.removed_watchers.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // the restarted tool can report other diagnostics
        needs_diagnostic_refresh = false;

        let result = worker
            .did_change_configuration(
                serde_json::json!(2),
                &mut needs_diagnostic_refresh,
                Some(&fs),
                &fs.keys(),
            )
            .await;

        // Since FakeToolBuilder changes watcher patterns based on configuration, registrations are expected
        assert!(result.diagnostics.is_none());
        assert_eq!(result.removed_watchers.len(), 1); // One unregistration expected
        assert_eq!(result.removed_watchers[0].id, "watcher-file:///root/");
        assert_eq!(result.new_watchers.len(), 1); // One new registration expected
        assert_eq!(result.new_watchers[0].id, "watcher-file:///root/");
        assert!(!needs_diagnostic_refresh); // No need to refresh diagnostics

        let result = worker
            .did_change_configuration(
                serde_json::json!(3),
                &mut needs_diagnostic_refresh,
                Some(&fs),
                &fs.keys(),
            )
            .await;

        // Since FakeToolBuilder changes diagnostics based on configuration, diagnostics are expected
        assert!(result.diagnostics.is_some());
        assert_eq!(result.diagnostics.unwrap().len(), 1); // One diagnostic report expected
        assert_eq!(result.new_watchers.len(), 0); // No new registrations expected
        assert_eq!(result.removed_watchers.len(), 0); // No unregistrations expected
        assert!(needs_diagnostic_refresh); // Need to refresh diagnostics
    }

    #[tokio::test]
    async fn test_client_message_on_configuration_change() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::Value::Null).await;

        let mut needs_diagnostic_refresh = false;
        let result = worker
            .did_change_configuration(
                serde_json::json!(4),
                &mut needs_diagnostic_refresh,
                None,
                &[],
            )
            .await;

        assert!(result.diagnostics.is_none());
        assert_eq!(result.new_watchers.len(), 0);
        assert_eq!(result.removed_watchers.len(), 0);
        assert!(!needs_diagnostic_refresh);
        assert_eq!(
            result.client_messages,
            vec![ClientMessage {
                message: "Fake misconfiguration message".to_string(),
                r#type: MessageType::WARNING,
            }]
        );
    }

    #[tokio::test]
    async fn test_client_message_on_watched_file_change() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::Value::Null).await;

        let mut needs_diagnostic_refresh = false;
        let result = worker
            .did_change_watched_files(
                &FileEvent {
                    uri: Uri::from_str("file:///root/misconfiguration.config").unwrap(),
                    typ: FileChangeType::CHANGED,
                },
                &mut needs_diagnostic_refresh,
                None,
                &[],
            )
            .await;

        assert!(result.diagnostics.is_none());
        assert_eq!(result.new_watchers.len(), 0);
        assert_eq!(result.removed_watchers.len(), 0);
        assert!(!needs_diagnostic_refresh);
        assert_eq!(
            result.client_messages,
            vec![ClientMessage {
                message: "Fake misconfiguration message".to_string(),
                r#type: MessageType::WARNING,
            }]
        );
    }

    #[tokio::test]
    async fn test_code_action_collection() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        worker.start_worker(serde_json::Value::Null).await;

        let actions = worker
            .get_code_actions_or_commands(CodeActionParams {
                uri: Uri::from_str("file:///root/file.js").unwrap(),
                range: Range::default(),
                context: CodeActionContext::default(),
                is_open_document: false,
            })
            .await;

        assert_eq!(actions.len(), 0);

        let actions = worker
            .get_code_actions_or_commands(CodeActionParams {
                uri: Uri::from_str("file:///root/code_action.config").unwrap(),
                range: Range::default(),
                context: CodeActionContext::default(),
                is_open_document: false,
            })
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
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let uri = Uri::from_str("file:///root/diagnostics.config").unwrap();

        worker.start_worker(serde_json::Value::Null).await;

        let diagnostics_no_content = worker
            .run_diagnostic(TextDocument::new(&uri, LanguageId::default(), None))
            .await
            .unwrap();

        assert_eq!(diagnostics_no_content.len(), 1);
        assert_eq!(diagnostics_no_content[0].0, uri);
        assert_eq!(diagnostics_no_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_no_content[0].1[0].message,
            "Fake diagnostic for content: <no content>"
        );

        let diagnostics_with_content = worker
            .run_diagnostic(TextDocument::new(
                &uri,
                LanguageId::default(),
                Some(Arc::from("helloworld")),
            ))
            .await
            .unwrap();

        assert_eq!(diagnostics_with_content.len(), 1);
        assert_eq!(diagnostics_with_content[0].0, uri);
        assert_eq!(diagnostics_with_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_with_content[0].1[0].message,
            "Fake diagnostic for content: helloworld"
        );

        let no_diagnostics = worker
            .run_diagnostic(TextDocument::new(
                &Uri::from_str("file:///root/unknown.file").unwrap(),
                LanguageId::default(),
                None,
            ))
            .await
            .unwrap();

        assert!(no_diagnostics.is_empty());

        let error = worker
            .run_diagnostic(TextDocument::new(
                &Uri::from_str("file:///root/error.config").unwrap(),
                LanguageId::default(),
                None,
            ))
            .await
            .unwrap_err();

        assert_eq!(error, "Fake diagnostic error");
    }

    #[tokio::test]
    async fn test_run_diagnostic_on_change() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let uri = Uri::from_str("file:///root/diagnostics.config").unwrap();

        worker.start_worker(serde_json::Value::Null).await;

        let diagnostics_no_content = worker
            .run_diagnostic_on_change(TextDocument::new(&uri, LanguageId::default(), None))
            .await
            .unwrap();

        assert_eq!(diagnostics_no_content.len(), 1);
        assert_eq!(diagnostics_no_content[0].0, uri);
        assert_eq!(diagnostics_no_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_no_content[0].1[0].message,
            "Fake diagnostic for content: <no content>"
        );

        let diagnostics_with_content = worker
            .run_diagnostic_on_change(TextDocument::new(
                &uri,
                LanguageId::default(),
                Some(Arc::from("helloworld")),
            ))
            .await
            .unwrap();

        assert_eq!(diagnostics_with_content.len(), 1);
        assert_eq!(diagnostics_with_content[0].0, uri);
        assert_eq!(diagnostics_with_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_with_content[0].1[0].message,
            "Fake diagnostic for content: helloworld"
        );

        let no_diagnostics = worker
            .run_diagnostic_on_change(TextDocument::new(
                &Uri::from_str("file:///root/unknown.file").unwrap(),
                LanguageId::default(),
                None,
            ))
            .await
            .unwrap();

        assert!(no_diagnostics.is_empty());

        let error = worker
            .run_diagnostic_on_change(TextDocument::new(
                &Uri::from_str("file:///root/error.config").unwrap(),
                LanguageId::default(),
                None,
            ))
            .await
            .unwrap_err();

        assert_eq!(error, "Fake diagnostic error");
    }

    #[tokio::test]
    async fn test_run_diagnostic_on_save() {
        let worker = WorkspaceWorker::new(
            Uri::from_str("file:///root/").unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let uri = Uri::from_str("file:///root/diagnostics.config").unwrap();
        worker.start_worker(serde_json::Value::Null).await;

        let diagnostics_no_content = worker
            .run_diagnostic_on_save(TextDocument::new(&uri, LanguageId::default(), None))
            .await
            .unwrap();

        assert_eq!(diagnostics_no_content.len(), 1);
        assert_eq!(diagnostics_no_content[0].0, uri);
        assert_eq!(diagnostics_no_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_no_content[0].1[0].message,
            "Fake diagnostic for content: <no content>"
        );

        let diagnostics_with_content = worker
            .run_diagnostic_on_save(TextDocument::new(
                &uri,
                LanguageId::default(),
                Some(Arc::from("helloworld")),
            ))
            .await
            .unwrap();

        assert_eq!(diagnostics_with_content.len(), 1);
        assert_eq!(diagnostics_with_content[0].0, uri);
        assert_eq!(diagnostics_with_content[0].1.len(), 1);
        assert_eq!(
            diagnostics_with_content[0].1[0].message,
            "Fake diagnostic for content: helloworld"
        );

        let no_diagnostics = worker
            .run_diagnostic_on_save(TextDocument::new(
                &Uri::from_str("file:///root/unknown.file").unwrap(),
                LanguageId::default(),
                None,
            ))
            .await
            .unwrap();

        assert!(no_diagnostics.is_empty());

        let error = worker
            .run_diagnostic_on_save(TextDocument::new(
                &Uri::from_str("file:///root/error.config").unwrap(),
                LanguageId::default(),
                None,
            ))
            .await
            .unwrap_err();

        assert_eq!(error, "Fake diagnostic error");
    }
}
