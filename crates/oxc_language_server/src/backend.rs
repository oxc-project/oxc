use std::{str::FromStr, sync::Arc};

use futures::future::join_all;
use log::{debug, info, warn};
use rustc_hash::FxBuildHasher;
use tokio::sync::{OnceCell, RwLock, SetError};
use tower_lsp_server::{
    Client, LanguageServer,
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        CodeActionParams, CodeActionResponse, ConfigurationItem, Diagnostic,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, DocumentFormattingParams, ExecuteCommandParams,
        InitializeParams, InitializeResult, InitializedParams, Registration, ServerInfo, TextEdit,
        Unregistration, Uri, WorkspaceEdit,
    },
};

use crate::{
    ConcurrentHashMap,
    capabilities::Capabilities,
    code_actions::CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
    commands::{FIX_ALL_COMMAND_ID, FixAllCommandArgs},
    file_system::LSPFileSystem,
    linter::server_linter::ServerLinterRun,
    options::{Options, WorkspaceOption},
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
pub struct Backend {
    // The LSP client to communicate with the editor or IDE.
    client: Client,
    // Each Workspace has it own worker with Linter (and in the future the formatter).
    // We must respect each program inside with its own root folder
    // and can not use shared programmes across multiple workspaces.
    // Each Workspace can have its own server configuration and program root configuration.
    // WorkspaceWorkers are only written on 2 occasions:
    // 1. `initialize` request with workspace folders
    // 2. `workspace/didChangeWorkspaceFolders` request
    workspace_workers: Arc<RwLock<Vec<WorkspaceWorker>>>,
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
        let server_version = env!("CARGO_PKG_VERSION");
        // initialization_options can be anything, so we are requesting `workspace/configuration` when no initialize options are provided
        let options = params.initialization_options.and_then(|mut value| {
            // the client supports the new settings object
            if let Ok(new_settings) = serde_json::from_value::<Vec<WorkspaceOption>>(value.clone())
            {
                // ToDo: validate they have the same length as params.workspace_folders
                return Some(new_settings);
            }

            let deprecated_settings =
                serde_json::from_value::<Options>(value.get_mut("settings")?.take()).ok();

            // the client has deprecated settings and has a deprecated root uri.
            // handle all things like the old way
            if deprecated_settings.is_some() && params.root_uri.is_some() {
                return Some(vec![WorkspaceOption {
                    workspace_uri: params.root_uri.clone().unwrap(),
                    options: deprecated_settings.unwrap(),
                }]);
            }

            // no workspace options could be generated fallback to default one or request when possible
            None
        });

        info!("initialize: {options:?}");
        info!("language server version: {server_version}");

        let capabilities = Capabilities::from(params.capabilities);

        // client sent workspace folders
        let workers = if let Some(workspace_folders) = &params.workspace_folders {
            workspace_folders
                .iter()
                .map(|workspace_folder| WorkspaceWorker::new(workspace_folder.uri.clone()))
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
            for worker in &workers {
                let option = &options
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .find(|workspace_option| {
                        worker.is_responsible_for_uri(&workspace_option.workspace_uri)
                    })
                    .map(|workspace_options| workspace_options.options.clone())
                    .unwrap_or_default();

                worker.start_worker(option).await;
            }
        }

        *self.workspace_workers.write().await = workers;

        self.capabilities.set(capabilities.clone()).map_err(|err| {
            let message = match err {
                SetError::AlreadyInitializedError(_) => {
                    "capabilities are already initialized".into()
                }
                SetError::InitializingError(_) => "initializing error".into(),
            };

            Error { code: ErrorCode::ParseError, message, data: None }
        })?;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "oxc".into(),
                version: Some(server_version.to_string()),
            }),
            offset_encoding: None,
            capabilities: capabilities.into(),
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
                vec![Some(Options::default()); needed_configurations.len()]
            };
            let default_options = Options::default();

            for (index, worker) in needed_configurations.values().enumerate() {
                let configuration =
                    configurations.get(index).unwrap_or(&None).as_ref().unwrap_or(&default_options);

                worker.start_worker(configuration).await;
            }
        }

        let mut registrations = vec![];

        // init all file watchers
        if capabilities.dynamic_watchers {
            for worker in workers {
                registrations.extend(worker.init_watchers().await);
            }
        }

        if capabilities.dynamic_formatting {
            // check if one workspace has formatting enabled
            let mut started_worker = false;
            for worker in workers {
                if worker.has_active_formatter().await {
                    started_worker = true;
                    break;
                }
            }

            if started_worker {
                registrations.push(Registration {
                    id: "dynamic-formatting".to_string(),
                    method: "textDocument/formatting".to_string(),
                    register_options: None,
                });
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
        self.clear_all_diagnostics().await;
        if self.capabilities.get().is_some_and(|option| option.dynamic_formatting) {
            self.file_system.write().await.clear();
        }
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

        // new valid configuration is passed
        let options = serde_json::from_value::<Vec<WorkspaceOption>>(params.settings.clone())
            .ok()
            .or_else(|| {
                // fallback to old configuration
                let options = serde_json::from_value::<Options>(params.settings).ok()?;

                // for all workers (default only one)
                let options = workers
                    .iter()
                    .map(|worker| WorkspaceOption {
                        workspace_uri: worker.get_root_uri().clone(),
                        options: options.clone(),
                    })
                    .collect();

                Some(options)
            });

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
                .iter()
                .enumerate()
                // filter out results where the client did not return a configuration
                .filter_map(|(index, config)| {
                    config.as_ref().map(|options| WorkspaceOption {
                        workspace_uri: workers[index].get_root_uri().clone(),
                        options: options.clone(),
                    })
                })
                .collect::<Vec<_>>()
        } else {
            warn!(
                "could not update the configuration for a worker. Send a custom configuration with `workspace/didChangeConfiguration` or support `workspace/configuration`."
            );
            return;
        };

        let mut global_formatting_added = false;

        for option in resolved_options {
            let Some(worker) =
                workers.iter().find(|worker| worker.is_responsible_for_uri(&option.workspace_uri))
            else {
                continue;
            };

            let (diagnostics, registrations, unregistrations, formatter_activated) =
                worker.did_change_configuration(&option.options).await;

            if formatter_activated && self.capabilities.get().is_some_and(|c| c.dynamic_formatting)
            {
                global_formatting_added = true;
            }

            if let Some(diagnostics) = diagnostics {
                new_diagnostics.extend(diagnostics);
            }

            removing_registrations.extend(unregistrations);
            adding_registrations.extend(registrations);
        }

        if !new_diagnostics.is_empty() {
            self.publish_all_diagnostics(&new_diagnostics).await;
        }

        // override the existing formatting registration
        // do not remove the registration, because other workspaces might still need it
        if global_formatting_added {
            adding_registrations.push(Registration {
                id: "dynamic-formatting".to_string(),
                method: "textDocument/formatting".to_string(),
                register_options: None,
            });
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

        let mut all_diagnostics = Vec::new();

        for file_event in &params.changes {
            // We do not expect multiple changes from the same workspace folder.
            // If we should consider it, we need to map the events to the workers first,
            // to only restart the internal linter / diagnostics for once
            let Some(worker) =
                workers.iter().find(|worker| worker.is_responsible_for_uri(&file_event.uri))
            else {
                continue;
            };
            let Some(diagnostics) = worker.did_change_watched_files(file_event).await else {
                continue;
            };

            all_diagnostics.extend(diagnostics);
        }

        if !all_diagnostics.is_empty() {
            self.publish_all_diagnostics(&all_diagnostics).await;
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
            cleared_diagnostics.extend(worker.get_clear_diagnostics().await);
            removed_registrations.push(Unregistration {
                id: format!("watcher-{}", worker.get_root_uri().as_str()),
                method: "workspace/didChangeWatchedFiles".to_string(),
            });
            workers.remove(index);
        }

        self.publish_all_diagnostics(&cleared_diagnostics).await;

        let default_options = Options::default();

        // client support `workspace/configuration` request
        if self.capabilities.get().is_some_and(|capabilities| capabilities.workspace_configuration)
        {
            let configurations = self
                .request_workspace_configuration(
                    params.event.added.iter().map(|w| &w.uri).collect(),
                )
                .await;

            for (index, folder) in params.event.added.iter().enumerate() {
                let worker = WorkspaceWorker::new(folder.uri.clone());
                // get the configuration from the response and init the linter
                let options = configurations.get(index).unwrap_or(&None);
                let options = options.as_ref().unwrap_or(&default_options);

                worker.start_worker(options).await;

                added_registrations.extend(worker.init_watchers().await);
                workers.push(worker);
            }
        // client does not support the request
        } else {
            for folder in params.event.added {
                let worker = WorkspaceWorker::new(folder.uri);
                // use default options
                worker.start_worker(&default_options).await;
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
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };

        if self.capabilities.get().is_some_and(|option| option.dynamic_formatting) {
            // saving the file means we can read again from the file system
            self.file_system.write().await.remove(uri);
        }

        if let Some(diagnostics) = worker.lint_file(uri, None, ServerLinterRun::OnSave).await {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    None,
                )
                .await;
        }
    }
    /// It will update the in-memory file content if the client supports dynamic formatting.
    /// It will re-lint the file and send updated diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didChange>
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };
        let content = params.content_changes.first().map(|c| c.text.clone());

        if self.capabilities.get().is_some_and(|option| option.dynamic_formatting)
            && let Some(content) = &content
        {
            self.file_system.write().await.set(uri, content.clone());
        }

        if let Some(diagnostics) = worker.lint_file(uri, content, ServerLinterRun::OnType).await {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    Some(params.text_document.version),
                )
                .await;
        }
    }

    /// It will add the in-memory file content if the client supports dynamic formatting.
    /// It will lint the file and send diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didOpen>
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.read().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };

        let content = params.text_document.text;

        if self.capabilities.get().is_some_and(|option| option.dynamic_formatting) {
            self.file_system.write().await.set(uri, content.clone());
        }

        if let Some(diagnostics) =
            worker.lint_file(uri, Some(content), ServerLinterRun::Always).await
        {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    Some(params.text_document.version),
                )
                .await;
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
        if self.capabilities.get().is_some_and(|option| option.dynamic_formatting) {
            self.file_system.write().await.remove(uri);
        }
        worker.remove_diagnostics(&params.text_document.uri).await;
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

        let is_source_fix_all_oxc = params
            .context
            .only
            .is_some_and(|only| only.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));

        let code_actions =
            worker.get_code_actions_or_commands(uri, &params.range, is_source_fix_all_oxc).await;

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
        if params.command == FIX_ALL_COMMAND_ID {
            if !self.capabilities.get().unwrap().workspace_apply_edit {
                return Err(Error::invalid_params("client does not support workspace apply edit"));
            }

            let args =
                FixAllCommandArgs::try_from(params.arguments).map_err(Error::invalid_params)?;

            let uri = &Uri::from_str(&args.uri).unwrap();
            let workers = self.workspace_workers.read().await;
            let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri))
            else {
                return Ok(None);
            };

            let text_edits = worker.get_diagnostic_text_edits(uri).await;

            self.client
                .apply_edit(WorkspaceEdit {
                    #[expect(clippy::disallowed_types)]
                    changes: Some(std::collections::HashMap::from([(uri.clone(), text_edits)])),
                    document_changes: None,
                    change_annotations: None,
                })
                .await?;

            return Ok(None);
        }

        Err(Error::invalid_request())
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
        Ok(worker.format_file(uri, self.file_system.read().await.get(uri)).await)
    }
}

impl Backend {
    /// Create a new Backend with the given client.
    /// The Backend will manage multiple [WorkspaceWorker]s and their configurations.
    /// It also holds the capabilities of the language server and an in-memory file system.
    /// The client is used to communicate with the LSP client.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_workers: Arc::new(RwLock::new(vec![])),
            capabilities: OnceCell::new(),
            file_system: Arc::new(RwLock::new(LSPFileSystem::default())),
        }
    }

    /// Request the workspace configuration from the client
    /// and return the options for each workspace folder.
    /// The check if the client support workspace configuration, should be done before.
    async fn request_workspace_configuration(&self, uris: Vec<&Uri>) -> Vec<Option<Options>> {
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
            return vec![None; length];
        };

        let mut options = vec![];
        for config in configs {
            options.push(serde_json::from_value::<Options>(config).ok());
        }

        debug_assert!(
            options.len() == length,
            "the number of configuration items should be the same as the number of workspace folders"
        );

        options
    }

    /// Clears all diagnostics for workspace folders
    async fn clear_all_diagnostics(&self) {
        let mut cleared_diagnostics = vec![];
        let workers = &*self.workspace_workers.read().await;
        for worker in workers {
            cleared_diagnostics.extend(worker.get_clear_diagnostics().await);
        }
        self.publish_all_diagnostics(&cleared_diagnostics).await;
    }

    /// Publish diagnostics for all files.
    async fn publish_all_diagnostics(&self, result: &[(String, Vec<Diagnostic>)]) {
        join_all(result.iter().map(|(path, diagnostics)| {
            self.client.publish_diagnostics(Uri::from_str(path).unwrap(), diagnostics.clone(), None)
        }))
        .await;
    }
}
