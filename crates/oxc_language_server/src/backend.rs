use std::{borrow::Cow, sync::Arc};

use futures::future::join_all;
use rustc_hash::FxBuildHasher;
use serde_json::Value;
use tokio::sync::{OnceCell, RwLock, SetError};
use tower_lsp_server::{
    Client, LanguageServer,
    jsonrpc::{Error, ErrorCode, Result},
    ls_types::{
        CodeActionParams, CodeActionResponse, ConfigurationItem, Diagnostic,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, DocumentDiagnosticParams, DocumentDiagnosticReport,
        DocumentDiagnosticReportKind, DocumentDiagnosticReportResult, DocumentFormattingParams,
        ExecuteCommandParams, FullDocumentDiagnosticReport, InitializeParams, InitializeResult,
        InitializedParams, MessageType, RelatedFullDocumentDiagnosticReport, ServerInfo,
        TextDocumentContentChangeEvent, TextEdit, Uri,
    },
};
use tracing::{debug, error, info, warn};

use crate::{
    ConcurrentHashMap, LanguageId, ToolBuilder,
    capabilities::{Capabilities, DiagnosticMode, server_capabilities},
    file_system::LSPFileSystem,
    options::WorkspaceOption,
    worker::WorkspaceWorker,
    worker_manager::WorkerManager,
};

/// The Backend implements the LanguageServer trait to handle LSP requests and notifications.
///
/// It manages multiple WorkspaceWorkers, each corresponding to a workspace folder.
/// Depending on the client's capabilities, it can dynamically register features and start up other services.
/// The Client will send requests and notifications to the Backend, which will delegate them to the appropriate WorkspaceWorker.
/// The Backend also manages the in-memory file system for open files.
///
/// A basic flow of an Editor and Server interaction is as follows:
/// - Editor sends `initialize` request with workspace folders and client capabilities.
/// - Server responds with its capabilities.
/// - Editor sends `initialized` notification.
/// - Server registers dynamic capabilities like file watchers.
/// - Editor sends `textDocument/didOpen`, `textDocument/didChange`, `textDocument/didSave`, and `textDocument/didClose` notifications.
/// - Editor sends `shutdown` request when the user closes the editor.
/// - Editor sends `exit` notification and the server exits.
///
/// Because `initialized` is a notification, the client will not wait for a response from the server.
/// Therefore, the server must be able to handle requests and notifications that may arrive directly after `initialized` notification,
/// such as `textDocument/didOpen`.
pub struct Backend {
    // The LSP client to communicate with the editor or IDE.
    client: Client,
    // Information about the server, such as name and version.
    // The client can use this information for display or logging purposes.
    server_info: ServerInfo,
    // Manages all WorkspaceWorkers for the language server.
    // The server operates in one of two modes:
    //   - Workspace mode: one or more workspace folders (or a root URI) were
    //     provided during `initialize`. Workers are created once and updated
    //     only on `workspace/didChangeWorkspaceFolders`.
    //   - Single-file mode: no workspace folder or root URI was provided.
    //     Workers are created dynamically when a file is opened and torn down
    //     when its last open file is closed.
    pub(crate) worker_manager: WorkerManager,
    // Capabilities of the language server, set once during `initialize` request.
    // Depending on the client capabilities, the server supports different capabilities.
    capabilities: OnceCell<Capabilities>,
    // A simple in-memory file system to store the content of open files.
    // The client will send the content of in-memory files on `textDocument/didOpen` and `textDocument/didChange`.
    // This is only needed when the client supports `textDocument/formatting` request.
    file_system: Arc<RwLock<LSPFileSystem>>,
}

impl LanguageServer for Backend {
    /// Initialize the language server with the given parameters.
    /// This method sets up workspace workers, capabilities, and starts the
    /// [WorkspaceWorker]s if the client sent the configuration with initialization options.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#initialize>
    #[expect(deprecated)] // `params.root_uri` is deprecated, we are only falling back to it if no workspace folder is provided
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // initialization_options can be anything, so we are requesting `workspace/configuration` when no initialize options are provided
        let options = params.initialization_options.and_then(|value| {
            // the client supports the new settings object
            if let Ok(new_settings) = serde_json::from_value::<Vec<WorkspaceOption>>(value.clone())
            {
                // ToDo: validate they have the same length as params.workspace_folders
                return Some(new_settings);
            }

            // the client has deprecated settings and has a deprecated root uri.
            // handle all things like the old way
            if let (Some(deprecated_settings), Some(root_uri)) =
                (value.get("settings"), params.root_uri.as_ref())
            {
                return Some(vec![WorkspaceOption {
                    workspace_uri: root_uri.clone(),
                    options: deprecated_settings.clone(),
                }]);
            }

            // no workspace options could be generated fallback to default one or request when possible
            None
        });

        let mut capabilities = Capabilities::from(params.capabilities);
        let mut server_capabilities = server_capabilities();
        self.worker_manager
            .read_tool_builder()
            .server_capabilities(&mut server_capabilities, &mut capabilities);

        info!("initialize: {options:?}");
        info!(
            "{} version: {}",
            self.server_info.name,
            self.server_info.version.as_deref().unwrap_or("unknown")
        );
        debug!("diagnostic model: {:?}", capabilities.diagnostic_mode);

        // client sent workspace folders
        let workers = if let Some(workspace_folders) = params.workspace_folders {
            let uris: Vec<Uri> =
                workspace_folders.iter().map(|folder| folder.uri.clone()).collect();
            WorkerManager::assert_workspaces_are_valid_paths(&uris)?;

            workspace_folders
                .into_iter()
                .map(|workspace_folder| {
                    self.worker_manager
                        .create_worker(workspace_folder.uri, capabilities.diagnostic_mode.clone())
                })
                .collect()
        // client sent deprecated root uri
        } else if let Some(root_uri) = params.root_uri {
            WorkerManager::assert_workspaces_are_valid_paths(std::slice::from_ref(&root_uri))?;

            vec![self.worker_manager.create_worker(root_uri, capabilities.diagnostic_mode.clone())]
        // client is in single file mode, create no workers initially.
        // Workers will be created dynamically in did_open.
        } else {
            self.worker_manager.set_single_file_mode(true);
            vec![]
        };

        // When the client did not send our custom `initialization_options`,
        // or the client does not support `workspace/configuration` request,
        // start the linter. We do not start the linter when the client support the request,
        // we will init the linter after requesting for the workspace configuration.
        if !capabilities.workspace_configuration || options.is_some() {
            let options = options.unwrap_or_default();

            for worker in &workers {
                let option = options
                    .iter()
                    .find(|workspace_option| {
                        worker.get_root_uri() == &workspace_option.workspace_uri
                    })
                    .map(|workspace_options| workspace_options.options.clone())
                    .unwrap_or_default();

                debug!("starting worker in initialize with options: {option:?}");
                worker.start_worker(option).await;
            }
        }

        self.worker_manager.set_all_workers(workers).await;

        self.capabilities.set(capabilities).map_err(|err| {
            let message = match err {
                SetError::AlreadyInitializedError(_) => {
                    "capabilities are already initialized".into()
                }
                SetError::InitializingError(_) => "initializing error".into(),
            };

            Error { code: ErrorCode::ParseError, message, data: None }
        })?;

        Ok(InitializeResult {
            server_info: Some(self.server_info.clone()),
            offset_encoding: None,
            capabilities: server_capabilities,
        })
    }

    /// It registers dynamic capabilities like file watchers and formatting if the client supports it.
    /// It also starts the [WorkspaceWorker]s if they did not start during initialization.
    /// If the client supports `workspace/configuration` request, it will request the configuration for each workspace folder
    /// and start the [WorkspaceWorker]s with the received configuration.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#initialized>
    async fn initialized(&self, _params: InitializedParams) {
        debug!("oxc initialized.");
        let Some(capabilities) = self.capabilities.get() else {
            return;
        };

        let workers = &*self.worker_manager.read_workers().await;
        let needed_configurations =
            ConcurrentHashMap::with_capacity_and_hasher(workers.len(), FxBuildHasher);
        let needed_configurations = needed_configurations.pin_owned();
        for worker in workers {
            if worker.needs_init_options().await {
                needed_configurations.insert(worker.get_root_uri().clone(), worker);
            }
        }

        if !needed_configurations.is_empty() {
            let configurations = if capabilities.workspace_configuration {
                self.request_workspace_configuration(needed_configurations.keys().collect()).await
            } else {
                // every worker should be initialized already in `initialize` request
                vec![serde_json::Value::Null; needed_configurations.len()]
            };

            // Snapshot all open-file URIs in one read lock so we don't need to
            // re-acquire the lock just to iterate the list of URIs. Individual
            // document lookups still take their own read lock per URI.
            let known_uris = self.file_system.read().await.keys();
            // will only be filled when using push diagnostic model
            let mut new_diagnostics = Vec::new();

            for (index, worker) in needed_configurations.values().copied().enumerate() {
                // get the configuration from the response and start the worker
                let configuration = configurations.get(index).unwrap_or(&serde_json::Value::Null);
                debug!("starting worker in initialize with options: {configuration:?}");
                worker.start_worker(configuration.clone()).await;

                // run diagnostics for all known files in the workspace of the worker.
                // This is necessary because the worker was not started before.
                // On Pull diagnostic model, we will ask the client to refresh diagnostics instead of sending them all.
                if capabilities.diagnostic_mode != DiagnosticMode::Push {
                    continue;
                }

                for uri in &known_uris {
                    // Check if this worker is the most specific one for this URI
                    let responsible_worker = WorkerManager::find_worker_for_uri(workers, uri);
                    if responsible_worker.is_none_or(|w| !std::ptr::eq(w, worker)) {
                        continue;
                    }
                    let document = {
                        let fs_guard = self.file_system.read().await;
                        fs_guard.get_document(uri)
                    };
                    let diagnostics = worker.run_diagnostic(&document).await;
                    match diagnostics {
                        Err(err) => {
                            error!(
                                "running diagnostics for {} failed: {err}",
                                document.uri.as_str()
                            );
                            if self.capabilities.get().is_some_and(|cap| cap.show_message) {
                                self.client.show_message(MessageType::ERROR, err).await;
                            }
                        }
                        Ok(diagnostics) => new_diagnostics.extend(diagnostics),
                    }
                }
            }

            if !new_diagnostics.is_empty() {
                self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
            } else if capabilities.diagnostic_mode == DiagnosticMode::Pull
                && !needed_configurations.is_empty()
            {
                debug_assert!(
                    capabilities.refresh_diagnostics,
                    "pull mode requires refresh diagnostics capability"
                );

                // In pull diagnostic model, we ask the client to refresh diagnostics
                if let Err(err) = self.client.workspace_diagnostic_refresh().await {
                    warn!("sending workspace/diagnostic/refresh failed: {err}");
                }
            }
        }

        let mut registrations = vec![];

        // init all file watchers
        if capabilities.dynamic_watchers {
            for worker in workers {
                registrations.extend(worker.init_watchers().await);
            }
        }

        if registrations.is_empty() {
            return;
        }
        if let Err(err) = self.client.register_capability(registrations).await {
            warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
        }
    }

    /// This method clears all diagnostics and the in-memory file system.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#shutdown>
    async fn shutdown(&self) -> Result<()> {
        let mut clearing_diagnostics = Vec::new();

        for worker in &*self.worker_manager.read_workers().await {
            // shutdown each worker and collect the URIs to clear diagnostics.
            // unregistering file watchers is not necessary, because the client will do it automatically on shutdown.
            // some clients (`helix`) do not expect any requests after shutdown is sent.
            let (uris, _) = worker.shutdown().await;
            clearing_diagnostics.extend(uris);
        }

        // only clear diagnostics when we are using push diagnostics
        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push)
            && !clearing_diagnostics.is_empty()
        {
            self.clear_diagnostics(clearing_diagnostics).await;
        }
        self.file_system.write().await.clear();

        Ok(())
    }

    /// This method updates the configuration of each [WorkspaceWorker] and restarts them if necessary.
    /// It also manages dynamic registrations for file watchers and formatting based on the new configuration.
    /// It will remove/add dynamic registrations if the client supports it.
    /// As an example, if a workspace changes the configuration file path, the file watcher will be updated.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_didChangeConfiguration>
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let workers = self.worker_manager.read_workers().await;
        let mut new_diagnostics = Vec::new();
        let mut removing_registrations = vec![];
        let mut adding_registrations = vec![];

        // when null, request configuration from client; otherwise, parse as per-workspace options or use as global configuration
        let options = if params.settings == Value::Null {
            None
        } else {
            serde_json::from_value::<Vec<WorkspaceOption>>(params.settings.clone()).ok().or_else(
                || {
                    // fallback to old configuration
                    // for all workers (default only one)
                    let options = workers
                        .iter()
                        .map(|worker| WorkspaceOption {
                            workspace_uri: worker.get_root_uri().clone(),
                            options: params.settings.clone(),
                        })
                        .collect();

                    Some(options)
                },
            )
        };

        // the client passed valid options.
        let resolved_options = if let Some(options) = options {
            options
            // else check if the client support workspace configuration requests
        } else if self
            .capabilities
            .get()
            .is_some_and(|capabilities| capabilities.workspace_configuration)
        {
            let configs = self
                .request_workspace_configuration(
                    workers.iter().map(WorkspaceWorker::get_root_uri).collect(),
                )
                .await;

            // Only create WorkspaceOption when the config is Some
            configs
                .into_iter()
                .enumerate()
                .map(|(index, options)| WorkspaceOption {
                    workspace_uri: workers[index].get_root_uri().clone(),
                    options,
                })
                .collect::<Vec<_>>()
        } else {
            warn!(
                "could not update the configuration for a worker. Send a custom configuration with `workspace/didChangeConfiguration` or support `workspace/configuration`."
            );
            return;
        };

        let mut needs_diagnostics_refresh = false;
        let diagnostic_mode =
            self.capabilities.get().map(|cap| cap.diagnostic_mode.clone()).unwrap_or_default();
        let fs_guard = if diagnostic_mode == DiagnosticMode::Push {
            Some(self.file_system.read().await)
        } else {
            None
        };
        let fs_ref = fs_guard.as_deref();

        for option in resolved_options {
            let Some(worker) =
                workers.iter().find(|worker| worker.get_root_uri() == &option.workspace_uri)
            else {
                continue;
            };

            let (diagnostics, registrations, unregistrations) = worker
                .did_change_configuration(option.options, &mut needs_diagnostics_refresh, fs_ref)
                .await;

            if let Some(diagnostics) = diagnostics {
                new_diagnostics.extend(diagnostics);
            }

            removing_registrations.extend(unregistrations);
            adding_registrations.extend(registrations);
        }

        if diagnostic_mode == DiagnosticMode::Push && !new_diagnostics.is_empty() {
            self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
        }

        if diagnostic_mode == DiagnosticMode::Pull && needs_diagnostics_refresh {
            // In pull diagnostic model, we ask the client to refresh diagnostics
            if let Err(err) = self.client.workspace_diagnostic_refresh().await {
                warn!("sending workspace/diagnostic/refresh failed: {err}");
            }
        }

        if !removing_registrations.is_empty()
            && let Err(err) = self.client.unregister_capability(removing_registrations).await
        {
            warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
        }
        if !adding_registrations.is_empty()
            && let Err(err) = self.client.register_capability(adding_registrations).await
        {
            warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
        }
    }

    /// This notification is sent when a configuration file of a tool changes (example: `.oxlintrc.json`).
    /// The server will re-lint the affected files and send updated diagnostics.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_didChangeWatchedFiles>
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let workers = self.worker_manager.read_workers().await;
        // ToDo: what if an empty changes flag is passed?
        debug!("watched file did change");

        let mut new_diagnostics = Vec::new();
        let mut removing_registrations = vec![];
        let mut adding_registrations = vec![];

        let mut needs_diagnostics_refresh = false;
        let diagnostic_mode =
            self.capabilities.get().map(|cap| cap.diagnostic_mode.clone()).unwrap_or_default();
        let fs_guard = if diagnostic_mode == DiagnosticMode::Push {
            Some(self.file_system.read().await)
        } else {
            None
        };
        let fs_ref = fs_guard.as_deref();

        for file_event in &params.changes {
            // We do not expect multiple changes from the same workspace folder.
            // If we should consider it, we need to map the events to the workers first,
            // to only restart the internal linter / diagnostics for once
            let Some(worker) = WorkerManager::find_worker_for_uri(&workers, &file_event.uri) else {
                continue;
            };
            let (diagnostics, registrations, unregistrations) = worker
                .did_change_watched_files(file_event, &mut needs_diagnostics_refresh, fs_ref)
                .await;

            if let Some(diagnostics) = diagnostics {
                new_diagnostics.extend(diagnostics);
            }
            removing_registrations.extend(unregistrations);
            adding_registrations.extend(registrations);
        }

        if diagnostic_mode == DiagnosticMode::Push && !new_diagnostics.is_empty() {
            self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
        }

        if diagnostic_mode == DiagnosticMode::Pull && needs_diagnostics_refresh {
            // In pull diagnostic model, we ask the client to refresh diagnostics
            if let Err(err) = self.client.workspace_diagnostic_refresh().await {
                warn!("sending workspace/diagnostic/refresh failed: {err}");
            }
        }

        if self.capabilities.get().is_some_and(|capabilities| capabilities.dynamic_watchers) {
            if !removing_registrations.is_empty()
                && let Err(err) = self.client.unregister_capability(removing_registrations).await
            {
                warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
            }

            if !adding_registrations.is_empty()
                && let Err(err) = self.client.register_capability(adding_registrations).await
            {
                warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
            }
        }
    }

    /// The server will start new [WorkspaceWorker]s for added workspace folders
    /// and stop and remove [WorkspaceWorker]s for removed workspace folders including:
    /// - clearing diagnostics
    /// - unregistering file watchers
    ///
    /// When workspace folders are added while the server is in single-file mode, the server
    /// exits single-file mode and shuts down any dynamically-created single-file workers.
    /// When all workspace folders are removed, the server enters single-file mode so that
    /// subsequent file opens will again create workers dynamically.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_didChangeWorkspaceFolders>
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let capabilities = self.capabilities.get();
        let diagnostic_mode = capabilities.map(|c| c.diagnostic_mode.clone()).unwrap_or_default();

        // === Phase 1: Update worker state (brief write lock, no async I/O) ===
        // Extract workers that need to be shut down and update the mode flags.
        let workers_to_shutdown = self
            .worker_manager
            .update_workspace_folders(&params.event.added, &params.event.removed)
            .await;

        // === Phase 2: Shut down removed workers (no lock held) ===
        let mut cleared_diagnostics = vec![];
        let mut removed_registrations = vec![];
        for worker in workers_to_shutdown {
            let (uris, unregistrations) = worker.shutdown().await;
            cleared_diagnostics.extend(uris);
            removed_registrations.extend(unregistrations);
        }

        // === Phase 3: Request configuration and start new workers (no lock held) ===
        let configurations = if capabilities.is_some_and(|c| c.workspace_configuration) {
            self.request_workspace_configuration(
                params.event.added.iter().map(|w| &w.uri).collect(),
            )
            .await
        } else {
            vec![]
        };

        let mut new_workers = vec![];
        let mut added_registrations = vec![];
        for (index, folder) in params.event.added.into_iter().enumerate() {
            let worker = self.worker_manager.create_worker(folder.uri, diagnostic_mode.clone());
            let options = configurations.get(index).unwrap_or(&serde_json::Value::Null);
            worker.start_worker(options.clone()).await;
            added_registrations.extend(worker.init_watchers().await);
            new_workers.push(worker);
        }

        // === Phase 4: Insert new workers (brief write lock, no async I/O) ===
        self.worker_manager.add_workers(new_workers).await;

        // === Phase 5: Clear diagnostics and update client watchers (no lock held) ===
        if diagnostic_mode == DiagnosticMode::Push && !cleared_diagnostics.is_empty() {
            self.clear_diagnostics(cleared_diagnostics).await;
        }

        if capabilities.is_some_and(|c| c.dynamic_watchers) {
            if !added_registrations.is_empty()
                && let Err(err) = self.client.register_capability(added_registrations).await
            {
                warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
            }

            if !removed_registrations.is_empty()
                && let Err(err) = self.client.unregister_capability(removed_registrations).await
            {
                warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
            }
        }
    }

    /// It will remove the in-memory file content, because the file is saved to disk.
    /// It will re-lint the file and send updated diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didSave>
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("oxc server did save");
        let uri = params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
            return;
        };

        if let Some(content) = params.text {
            self.file_system.write().await.set(uri.clone(), content);
        }

        let document = self.file_system.read().await.get_document(&uri);
        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push) {
            match worker.run_diagnostic_on_save(&document).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    if self.capabilities.get().is_some_and(|cap| cap.show_message) {
                        self.client.show_message(MessageType::ERROR, err).await;
                    }
                }
                Ok(diagnostics) => {
                    if !diagnostics.is_empty() {
                        self.publish_all_diagnostics(diagnostics, ConcurrentHashMap::default())
                            .await;
                    }
                }
            }
        }
    }
    /// It will update the in-memory file content if the client supports dynamic formatting.
    /// It will re-lint the file and send updated diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didChange>
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
            return;
        };
        if let Some(content) = params
            .content_changes
            .into_iter()
            .next()
            .map(|c: TextDocumentContentChangeEvent| c.text)
        {
            self.file_system.write().await.set(uri.clone(), content);
        }

        let document = self.file_system.read().await.get_document(&uri);

        // Remove the internal cache for the document.
        // When the editor requests `textDocument/codeAction`, it may use its diagnostic cache to generate actions.
        // This could cause code actions to be generated with stale diagnostics if the cache is not cleared here.
        // This should never happen, because this server expects `textDocument/diagnostic` is requested beforehand.
        // Sadly, some editors/extensions have bugs, so we need to make sure the cache is cleared on change.
        worker.remove_uri_cache(&uri).await;

        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push) {
            match worker.run_diagnostic_on_change(&document).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    if self.capabilities.get().is_some_and(|cap| cap.show_message) {
                        self.client.show_message(MessageType::ERROR, err).await;
                    }
                }
                Ok(diagnostics) => {
                    if !diagnostics.is_empty() {
                        let version_map = ConcurrentHashMap::default();
                        version_map.pin().insert(uri.clone(), params.text_document.version);
                        self.publish_all_diagnostics(diagnostics, version_map).await;
                    }
                }
            }
        }
    }

    /// It will add the in-memory file content if the client supports dynamic formatting.
    /// It will lint the file and send diagnostics, if necessary.
    ///
    /// In single file mode (no workspace was configured during initialize), a new
    /// [WorkspaceWorker] is created dynamically using the file's parent directory as
    /// the workspace root if no existing worker covers the URI.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didOpen>
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;

        // In single file mode, dynamically create a workspace worker for file:// URIs.
        if self.worker_manager.is_single_file_mode() && uri.scheme().as_str() == "file" {
            let capabilities = self.capabilities.get();
            let diagnostic_mode =
                capabilities.map(|c| c.diagnostic_mode.clone()).unwrap_or_default();
            let dynamic_watchers = capabilities.is_some_and(|c| c.dynamic_watchers);
            if let Some(registrations) = self
                .worker_manager
                .ensure_worker_for_file_uri(&uri, diagnostic_mode, dynamic_watchers)
                .await
                && !registrations.is_empty()
                && let Err(err) = self.client.register_capability(registrations).await
            {
                warn!("registering file watchers for single-file workspace failed: {err}");
            }
        }

        let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
            return;
        };

        let content = params.text_document.text;

        self.file_system.write().await.set_with_language(
            uri.clone(),
            LanguageId::new(params.text_document.language_id),
            content,
        );

        let document = self.file_system.read().await.get_document(&uri);

        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push) {
            match worker.run_diagnostic(&document).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    if self.capabilities.get().is_some_and(|cap| cap.show_message) {
                        self.client.show_message(MessageType::ERROR, err).await;
                    }
                }
                Ok(diagnostics) => {
                    if !diagnostics.is_empty() {
                        let version_map = ConcurrentHashMap::default();
                        version_map.pin().insert(uri.clone(), params.text_document.version);
                        self.publish_all_diagnostics(diagnostics, version_map).await;
                    }
                }
            }
        }
    }

    /// It will remove the in-memory file content if the client supports dynamic formatting.
    /// It will clear the diagnostics (internally) for the closed file.
    ///
    /// In single file mode, if no other open files are associated with the worker's
    /// workspace after this close, the workspace worker is shut down and removed.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didClose>
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = &params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(uri).await else {
            return;
        };

        // Clone the root URI now so we can use it after dropping the read lock.
        let worker_root_uri = if self.worker_manager.is_single_file_mode() {
            Some(worker.get_root_uri().clone())
        } else {
            None
        };

        self.file_system.write().await.remove(uri);
        worker.remove_uri_cache(&params.text_document.uri).await;

        // Drop the read lock before potentially acquiring the write lock in
        // try_shutdown_empty_workspace.
        drop(worker);

        if let Some(root_uri) = worker_root_uri {
            let open_uris = self.file_system.read().await.keys();
            let result =
                self.worker_manager.try_shutdown_empty_workspace(&root_uri, &open_uris).await;

            if let Some((uris, unregistrations)) = result {
                let diagnostic_mode = self
                    .capabilities
                    .get()
                    .map(|cap| cap.diagnostic_mode.clone())
                    .unwrap_or_default();

                if diagnostic_mode == DiagnosticMode::Push && !uris.is_empty() {
                    self.clear_diagnostics(uris).await;
                }

                if self.capabilities.get().is_some_and(|cap| cap.dynamic_watchers)
                    && !unregistrations.is_empty()
                    && let Err(err) = self.client.unregister_capability(unregistrations).await
                {
                    warn!("unregistering file watchers for single-file workspace failed: {err}");
                }
            }
        }
    }

    /// It will return code actions or commands for the given range.
    /// The client can send `context.only` to `source.fixAll.oxc` to fix all diagnostics of the file.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_codeAction>
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(uri).await else {
            return Ok(None);
        };

        let code_actions =
            worker.get_code_actions_or_commands(uri, &params.range, &params.context).await;

        if code_actions.is_empty() {
            return Ok(None);
        }

        Ok(Some(code_actions))
    }

    /// It will execute the given command with the provided arguments.
    /// Currently, only the `fixAll` command is supported.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_executeCommand>
    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        for worker in self.worker_manager.read_workers().await.iter() {
            match worker.execute_command(&params.command, params.arguments.clone()).await {
                Ok(changes) => {
                    let Some(edit) = changes else {
                        continue;
                    };

                    if !self.capabilities.get().unwrap().workspace_apply_edit {
                        return Err(Error::invalid_params(
                            "client does not support workspace apply edit",
                        ));
                    }

                    self.client.apply_edit(edit).await?;
                }
                Err(err) => return Err(Error::new(err)),
            }
        }

        Ok(None)
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = &params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(uri).await else {
            return Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport::default(),
            )));
        };

        let document = self.file_system.read().await.get_document(uri);
        let diagnostics = worker.run_diagnostic(&document).await;

        let diagnostics = match diagnostics {
            Err(err) => {
                error!("running diagnostics for {} failed: {err}", uri.as_str());
                return Err(Error {
                    code: ErrorCode::ServerError(1),
                    message: Cow::Owned(err),
                    data: None,
                });
            }
            Ok(diagnostics) => diagnostics,
        };

        let uri_diagnostics = diagnostics
            .iter()
            .filter(|(diag_uri, _)| diag_uri == uri)
            .flat_map(|(_, diags)| diags.clone())
            .collect::<Vec<_>>();

        let related_diagnostics =
            diagnostics.into_iter().filter(|(diag_uri, _)| diag_uri != uri).collect::<Vec<_>>();

        Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    items: uri_diagnostics,
                    ..Default::default()
                },
                related_documents: if related_diagnostics.is_empty() {
                    None
                } else {
                    Some(
                        related_diagnostics
                            .into_iter()
                            .map(|(diag_uri, diags)| {
                                (
                                    diag_uri,
                                    DocumentDiagnosticReportKind::Full(
                                        FullDocumentDiagnosticReport {
                                            items: diags,
                                            ..Default::default()
                                        },
                                    ),
                                )
                            })
                            .collect(),
                    )
                },
            },
        )))
    }

    /// It will return text edits to format the document if formatting is enabled for the workspace.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_formatting>
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(uri).await else {
            return Ok(None);
        };

        let document = self.file_system.read().await.get_document(uri);
        match worker.format_file(&document).await {
            Ok(edits) => {
                if edits.is_empty() {
                    return Ok(None);
                }
                Ok(Some(edits))
            }
            Err(err) => {
                Err(Error { code: ErrorCode::ServerError(1), message: Cow::Owned(err), data: None })
            }
        }
    }
}

impl Backend {
    /// Create a new Backend with the given client.
    /// The Backend will manage multiple [WorkspaceWorker]s and their configurations.
    /// It also holds the capabilities of the language server and an in-memory file system.
    /// The client is used to communicate with the LSP client.
    pub fn new(client: Client, server_info: ServerInfo, tool: Arc<dyn ToolBuilder>) -> Self {
        Self {
            client,
            server_info,
            worker_manager: WorkerManager::new(tool),
            capabilities: OnceCell::new(),
            file_system: Arc::new(RwLock::new(LSPFileSystem::default())),
        }
    }

    /// Request the workspace configuration from the client
    /// and return the options for each workspace folder.
    /// The check if the client support workspace configuration, should be done before.
    async fn request_workspace_configuration(&self, uris: Vec<&Uri>) -> Vec<serde_json::Value> {
        let length = uris.len();
        let config_items = uris
            .into_iter()
            .map(|uri| ConfigurationItem {
                scope_uri: Some(uri.clone()),
                section: Some("oxc_language_server".into()),
            })
            .collect::<Vec<_>>();

        let Ok(configs) = self.client.configuration(config_items).await else {
            debug!("failed to get configuration");
            // return none for each workspace folder
            return vec![serde_json::Value::Null; length];
        };

        debug_assert!(
            configs.len() == length,
            "the number of configuration items should be the same as the number of workspace folders"
        );

        configs
    }

    async fn clear_diagnostics(&self, uris: Vec<Uri>) {
        self.publish_all_diagnostics(
            uris.into_iter().map(|uri| (uri, vec![])).collect(),
            ConcurrentHashMap::default(),
        )
        .await;
    }

    /// Publish diagnostics for all files.
    async fn publish_all_diagnostics(
        &self,
        result: Vec<(Uri, Vec<Diagnostic>)>,
        version_map: ConcurrentHashMap<Uri, i32>,
    ) {
        join_all(result.into_iter().map(|(uri, diagnostics)| {
            let version = version_map.pin().get(&uri).copied();
            self.client.publish_diagnostics(uri, diagnostics, version)
        }))
        .await;
    }
}
