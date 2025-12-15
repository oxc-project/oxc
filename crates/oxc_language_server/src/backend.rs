use std::sync::Arc;

use futures::future::join_all;
use log::{debug, info, warn};
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
        DidSaveTextDocumentParams, DocumentFormattingParams, ExecuteCommandParams,
        InitializeParams, InitializeResult, InitializedParams, ServerInfo, TextEdit, Uri,
    },
};

use crate::{
    ConcurrentHashMap, ToolBuilder,
    capabilities::{Capabilities, server_capabilities},
    file_system::LSPFileSystem,
    options::WorkspaceOption,
    worker::WorkspaceWorker,
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
    // The available tool builders to create tools like linters and formatters.
    tool_builders: Vec<Box<dyn ToolBuilder>>,
    // Each Workspace has it own worker with Linter (and in the future the formatter).
    // We must respect each program inside with its own root folder
    // and can not use shared programmes across multiple workspaces.
    // Each Workspace can have its own server configuration and program root configuration.
    // WorkspaceWorkers are only written on 2 occasions:
    // 1. `initialize` request with workspace folders
    // 2. `workspace/didChangeWorkspaceFolders` request
    pub(crate) workspace_workers: Arc<RwLock<Vec<WorkspaceWorker>>>,
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

        info!("initialize: {options:?}");
        info!(
            "{} version: {}",
            self.server_info.name,
            self.server_info.version.as_deref().unwrap_or("unknown")
        );

        let capabilities = Capabilities::from(params.capabilities);

        // client sent workspace folders
        let workers = if let Some(workspace_folders) = params.workspace_folders {
            workspace_folders
                .into_iter()
                .map(|workspace_folder| WorkspaceWorker::new(workspace_folder.uri))
                .collect()
        // client sent deprecated root uri
        } else if let Some(root_uri) = params.root_uri {
            vec![WorkspaceWorker::new(root_uri)]
        // client is in single file mode, create no workers
        } else {
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
                        worker.is_responsible_for_uri(&workspace_option.workspace_uri)
                    })
                    .map(|workspace_options| workspace_options.options.clone())
                    .unwrap_or_default();

                worker.start_worker(option, &self.tool_builders).await;
            }
        }

        *self.workspace_workers.write().await = workers;

        let mut server_capabilities = server_capabilities();
        for tool_builder in &self.tool_builders {
            tool_builder.server_capabilities(&mut server_capabilities);
        }

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

        let workers = &*self.workspace_workers.read().await;
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

            let known_files = self.file_system.read().await.keys();
            let mut new_diagnostics = Vec::new();

            for (index, worker) in needed_configurations.values().enumerate() {
                // get the configuration from the response and start the worker
                let configuration = configurations.get(index).unwrap_or(&serde_json::Value::Null);
                worker.start_worker(configuration.clone(), &self.tool_builders).await;

                // run diagnostics for all known files in the workspace of the worker.
                // This is necessary because the worker was not started before.
                for uri in &known_files {
                    if !worker.is_responsible_for_uri(uri) {
                        continue;
                    }
                    let content = self.file_system.read().await.get(uri);
                    let diagnostics = worker.run_diagnostic(uri, content.as_deref()).await;
                    new_diagnostics.extend(diagnostics);
                }
            }

            if !new_diagnostics.is_empty() {
                self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
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

    /// This method clears all diagnostics and the in-memory file system if dynamic formatting is enabled.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#shutdown>
    async fn shutdown(&self) -> Result<()> {
        let mut clearing_diagnostics = Vec::new();
        let mut removed_registrations = Vec::new();

        for worker in &*self.workspace_workers.read().await {
            let (uris, unregistrations) = worker.shutdown().await;
            clearing_diagnostics.extend(uris);
            removed_registrations.extend(unregistrations);
        }
        if !clearing_diagnostics.is_empty() {
            self.clear_diagnostics(clearing_diagnostics).await;
        }
        if self.capabilities.get().unwrap().dynamic_watchers
            && !removed_registrations.is_empty()
            && let Err(err) = self.client.unregister_capability(removed_registrations).await
        {
            warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
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
        let workers = self.workspace_workers.read().await;
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

        for option in resolved_options {
            let Some(worker) =
                workers.iter().find(|worker| worker.is_responsible_for_uri(&option.workspace_uri))
            else {
                continue;
            };

            let (diagnostics, registrations, unregistrations) = worker
                .did_change_configuration(option.options, &*self.file_system.read().await)
                .await;

            if let Some(diagnostics) = diagnostics {
                new_diagnostics.extend(diagnostics);
            }

            removing_registrations.extend(unregistrations);
            adding_registrations.extend(registrations);
        }

        if !new_diagnostics.is_empty() {
            self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
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
        let workers = self.workspace_workers.read().await;
        // ToDo: what if an empty changes flag is passed?
        debug!("watched file did change");

        let mut new_diagnostics = Vec::new();
        let mut removing_registrations = vec![];
        let mut adding_registrations = vec![];

        for file_event in &params.changes {
            // We do not expect multiple changes from the same workspace folder.
            // If we should consider it, we need to map the events to the workers first,
            // to only restart the internal linter / diagnostics for once
            let Some(worker) =
                workers.iter().find(|worker| worker.is_responsible_for_uri(&file_event.uri))
            else {
                continue;
            };
            let (diagnostics, registrations, unregistrations) =
                worker.did_change_watched_files(file_event, &*self.file_system.read().await).await;

            if let Some(diagnostics) = diagnostics {
                new_diagnostics.extend(diagnostics);
            }
            removing_registrations.extend(unregistrations);
            adding_registrations.extend(registrations);
        }

        if !new_diagnostics.is_empty() {
            self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
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
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_didChangeWorkspaceFolders>
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let mut workers = self.workspace_workers.write().await;
        let mut cleared_diagnostics = vec![];
        let mut added_registrations = vec![];
        let mut removed_registrations = vec![];

        for folder in params.event.removed {
            let Some((index, worker)) = workers
                .iter()
                .enumerate()
                .find(|(_, worker)| worker.is_responsible_for_uri(&folder.uri))
            else {
                continue;
            };
            let (uris, unregistrations) = worker.shutdown().await;
            cleared_diagnostics.extend(uris);
            removed_registrations.extend(unregistrations);
            workers.remove(index);
        }

        self.clear_diagnostics(cleared_diagnostics).await;

        // client support `workspace/configuration` request
        if self.capabilities.get().is_some_and(|capabilities| capabilities.workspace_configuration)
        {
            let configurations = self
                .request_workspace_configuration(
                    params.event.added.iter().map(|w| &w.uri).collect(),
                )
                .await;

            for (index, folder) in params.event.added.into_iter().enumerate() {
                let worker = WorkspaceWorker::new(folder.uri);
                // get the configuration from the response and init the linter
                let options = configurations.get(index).unwrap_or(&serde_json::Value::Null);
                worker.start_worker(options.clone(), &self.tool_builders).await;

                added_registrations.extend(worker.init_watchers().await);
                workers.push(worker);
            }
        // client does not support the request
        } else {
            for folder in params.event.added {
                let worker = WorkspaceWorker::new(folder.uri);
                // use default options
                worker.start_worker(serde_json::Value::Null, &self.tool_builders).await;
                added_registrations.extend(worker.init_watchers().await);
                workers.push(worker);
            }
        }

        // tell client to stop / start watching for files
        if self.capabilities.get().is_some_and(|capabilities| capabilities.dynamic_watchers) {
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
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(&uri)) else {
            return;
        };

        let diagnostics = worker.run_diagnostic_on_save(&uri, params.text.as_deref()).await;
        if !diagnostics.is_empty() {
            self.publish_all_diagnostics(diagnostics, ConcurrentHashMap::default()).await;
        }
    }
    /// It will update the in-memory file content if the client supports dynamic formatting.
    /// It will re-lint the file and send updated diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didChange>
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(&uri)) else {
            return;
        };
        let content = params.content_changes.first().map(|c| c.text.clone());

        if let Some(content) = &content {
            self.file_system.write().await.set(uri.clone(), content.clone());
        }

        let diagnostics = worker.run_diagnostic_on_change(&uri, content.as_deref()).await;
        if !diagnostics.is_empty() {
            let version_map = ConcurrentHashMap::default();
            version_map.pin().insert(uri.clone(), params.text_document.version);
            self.publish_all_diagnostics(diagnostics, version_map).await;
        }
    }

    /// It will add the in-memory file content if the client supports dynamic formatting.
    /// It will lint the file and send diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didOpen>
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(&uri)) else {
            return;
        };

        let content = params.text_document.text;

        self.file_system.write().await.set(uri.clone(), content.clone());

        let diagnostics = worker.run_diagnostic(&uri, Some(&content)).await;
        if !diagnostics.is_empty() {
            let version_map = ConcurrentHashMap::default();
            version_map.pin().insert(uri.clone(), params.text_document.version);
            self.publish_all_diagnostics(diagnostics, version_map).await;
        }
    }

    /// It will remove the in-memory file content if the client supports dynamic formatting.
    /// It will clear the diagnostics (internally) for the closed file.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didClose>
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };

        self.file_system.write().await.remove(uri);
        worker.remove_uri_cache(&params.text_document.uri).await;
    }

    /// It will return code actions or commands for the given range.
    /// The client can send `context.only` to `source.fixAll.oxc` to fix all diagnostics of the file.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_codeAction>
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return Ok(None);
        };

        let code_actions =
            worker.get_code_actions_or_commands(uri, &params.range, params.context.only).await;

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
        for worker in self.workspace_workers.read().await.iter() {
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

    /// It will return text edits to format the document if formatting is enabled for the workspace.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_formatting>
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return Ok(None);
        };
        Ok(worker.format_file(uri, self.file_system.read().await.get(uri).as_deref()).await)
    }
}

impl Backend {
    /// Create a new Backend with the given client.
    /// The Backend will manage multiple [WorkspaceWorker]s and their configurations.
    /// It also holds the capabilities of the language server and an in-memory file system.
    /// The client is used to communicate with the LSP client.
    pub fn new(client: Client, server_info: ServerInfo, tools: Vec<Box<dyn ToolBuilder>>) -> Self {
        Self {
            client,
            server_info,
            tool_builders: tools,
            workspace_workers: Arc::new(RwLock::new(vec![])),
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
