use std::{borrow::Cow, sync::Arc};

use futures::future::join_all;
use rustc_hash::FxHashSet;
use serde_json::Value;
use tokio::sync::{OnceCell, SetError};
use tower_lsp_server::{
    Client, LanguageServer,
    jsonrpc::{Error, ErrorCode, Result},
    ls_types::{
        CodeActionParams, CodeActionResponse, ConfigurationItem, Diagnostic,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, DocumentDiagnosticParams, DocumentDiagnosticReport,
        DocumentDiagnosticReportKind, DocumentDiagnosticReportResult, DocumentFormattingParams,
        ExecuteCommandParams, FileChangeType, FullDocumentDiagnosticReport, InitializeParams,
        InitializeResult, InitializedParams, MessageType, Pattern, Registration,
        RelatedFullDocumentDiagnosticReport, ServerInfo, TextDocumentContentChangeEvent, TextEdit,
        Unregistration, Uri, WorkspaceEdit,
    },
};
use tracing::{debug, error, info, warn};

use crate::{
    ClientMessage, ConcurrentHashMap, LanguageId,
    capabilities::{Capabilities, DiagnosticMode, server_capabilities},
    file_system::LSPFileSystem,
    file_system::ResolvedPath,
    options::WorkspaceOption,
    utils::{find_root_for_uri, roots_are_equal},
    worker::WorkspaceWorker,
    worker_manager::WorkerManager,
    working_directories::{
        WORKING_DIRECTORIES_OPTION, is_below_ignored_directory, sub_worker_options,
    },
};

/// Everything a worker reconciliation produced and the caller has to forward to the client.
#[derive(Default)]
struct WorkerChanges {
    diagnostics: Vec<(Uri, Vec<Diagnostic>)>,
    cleared_diagnostics: Vec<Uri>,
    registrations: Vec<Registration>,
    unregistrations: Vec<Unregistration>,
    client_messages: Vec<ClientMessage>,
    /// Whether an open document is now handled by a different worker, so the client has to ask for
    /// its diagnostics again in pull mode.
    needs_diagnostics_refresh: bool,
    /// Roots whose excluded set changed, so their tool has to be rebuilt. Already rebuilt when the
    /// caller requested the reconciliation to do it.
    rebuild_roots: Vec<Uri>,
    /// Roots of the sub workers which the reconciliation created.
    added_roots: Vec<Uri>,
    /// Roots whose documents changed owner and still have to be revalidated.
    changed_roots: Vec<Uri>,
}

impl WorkerChanges {
    fn extend(&mut self, other: Self) {
        self.diagnostics.extend(other.diagnostics);
        self.cleared_diagnostics.extend(other.cleared_diagnostics);
        self.registrations.extend(other.registrations);
        self.unregistrations.extend(other.unregistrations);
        self.client_messages.extend(other.client_messages);
        self.needs_diagnostics_refresh |= other.needs_diagnostics_refresh;
        self.rebuild_roots.extend(other.rebuild_roots);
        self.added_roots.extend(other.added_roots);
        self.changed_roots.extend(other.changed_roots);
    }
}

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
    file_system: Arc<LSPFileSystem>,
    // Messages collected during `initialize` that must be deferred until `initialized`,
    // because the LSP spec forbids server-to-client communication before the initialize response is sent.
    pending_initialization_messages: OnceCell<Vec<ClientMessage>>,
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
        let options = params.initialization_options.and_then(|mut value| {
            // the client supports the new settings object
            if let Ok(new_settings) = serde_json::from_value::<Vec<WorkspaceOption>>(value.clone())
            {
                // ToDo: validate they have the same length as params.workspace_folders
                return Some(new_settings);
            }

            // the client has deprecated settings and has a deprecated root uri.
            // handle all things like the old way
            if value.get("settings").is_some()
                && let (Some(deprecated_settings), Some(root_uri)) =
                    (value.get_mut("settings"), params.root_uri.as_ref())
            {
                return Some(vec![WorkspaceOption {
                    workspace_uri: root_uri.clone(),
                    options: deprecated_settings.take(),
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
        let mut workers = if let Some(workspace_folders) = params.workspace_folders {
            let uris: Vec<&Uri> = workspace_folders.iter().map(|folder| &folder.uri).collect();
            WorkerManager::assert_workspaces_are_valid_paths(uris)?;

            workspace_folders
                .into_iter()
                .map(|workspace_folder| {
                    self.worker_manager
                        .create_worker(workspace_folder.uri, capabilities.diagnostic_mode.clone())
                })
                .collect()
        // client sent deprecated root uri
        } else if let Some(root_uri) = params.root_uri {
            WorkerManager::assert_workspaces_are_valid_paths(vec![&root_uri])?;

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
        let mut client_messages = vec![];
        if !capabilities.workspace_configuration || options.is_some() {
            let options = options.unwrap_or_default();

            // a directory the client opened as its own workspace folder never gets a second,
            // shadowing sub worker
            let folder_roots =
                workers.iter().map(|worker| worker.get_root_uri().clone()).collect::<Vec<_>>();
            let mut claimed_roots = folder_roots.clone();
            let mut sub_workers = vec![];
            for worker in &workers {
                let option = options
                    .iter()
                    .find(|workspace_option| {
                        worker.get_root_uri() == &workspace_option.workspace_uri
                    })
                    .map(|workspace_options| workspace_options.options.clone())
                    .unwrap_or_default();

                debug!("starting worker in initialize with options: {option:?}");

                let (workers, messages) = self
                    .worker_manager
                    .start_folder_worker(
                        worker,
                        option,
                        &capabilities.diagnostic_mode,
                        &claimed_roots,
                        &folder_roots,
                    )
                    .await;
                claimed_roots.extend(workers.iter().map(|w| w.get_root_uri().clone()));
                sub_workers.extend(workers);
                client_messages.extend(messages);
            }

            // A `workingDirectories` sub worker is an ordinary worker. Appending them is enough,
            // because a file URI is routed to the worker with the longest matching root path.
            workers.extend(sub_workers);
        }

        client_messages.extend(
            self.worker_manager.start_manager(workers, capabilities.diagnostic_mode.clone()).await,
        );

        if !client_messages.is_empty() {
            let _ = self.pending_initialization_messages.set(client_messages);
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

        let mut client_messages =
            self.pending_initialization_messages.get().cloned().unwrap_or_default();
        let Some(capabilities) = self.capabilities.get() else {
            return;
        };

        // Collect the workspace folder workers which have not been started in `initialize`.
        // The read lock must not be held while starting them, because that inserts the
        // `workingDirectories` sub workers, which needs the write lock.
        let needed_configurations = {
            let workspace_workers = self.worker_manager.read_workspace_workers().await;
            let mut uris = Vec::with_capacity(workspace_workers.len());
            for worker in workspace_workers.iter() {
                if worker.needs_init_options().await {
                    uris.push(worker.get_root_uri().clone());
                }
            }
            uris
        };

        if !needed_configurations.is_empty() {
            let configurations = if capabilities.workspace_configuration {
                self.request_workspace_configuration(needed_configurations.iter().collect()).await
            } else {
                // every worker should be initialized already in `initialize` request
                vec![serde_json::Value::Null; needed_configurations.len()]
            };

            let mut sub_workers = vec![];
            {
                let workspace_workers = self.worker_manager.read_workspace_workers().await;
                // a directory the client opened as its own workspace folder never gets a second,
                // shadowing sub worker
                let mut claimed_roots = workspace_workers
                    .iter()
                    .map(|worker| worker.get_root_uri().clone())
                    .collect::<Vec<_>>();
                // the folders are read from the guard which is already held: `start_folder_worker`
                // must not take the lock a second time
                let folder_roots = workspace_workers
                    .iter()
                    .filter(|worker| !worker.is_sub_worker())
                    .map(|worker| worker.get_root_uri().clone())
                    .collect::<Vec<_>>();
                for (index, root_uri) in needed_configurations.iter().enumerate() {
                    let Some(worker) = workspace_workers.iter().find(|worker| {
                        !worker.is_sub_worker() && worker.get_root_uri() == root_uri
                    }) else {
                        continue;
                    };
                    // get the configuration from the response and start the worker
                    let configuration =
                        configurations.get(index).unwrap_or(&serde_json::Value::Null);
                    debug!("starting worker in initialize with options: {configuration:?}");

                    let (workers, messages) = self
                        .worker_manager
                        .start_folder_worker(
                            worker,
                            configuration.clone(),
                            &capabilities.diagnostic_mode,
                            &claimed_roots,
                            &folder_roots,
                        )
                        .await;
                    claimed_roots.extend(workers.iter().map(|w| w.get_root_uri().clone()));
                    sub_workers.extend(workers);
                    client_messages.extend(messages);
                }
            }
            self.worker_manager.add_workers(sub_workers).await;

            // All open-file URIs
            let known_uris = self.file_system.keys();
            // will only be filled when using push diagnostic model
            let mut new_diagnostics = Vec::new();

            // run diagnostics for all known files in the workspace of the worker.
            // This is necessary because the worker was not started before.
            // On Pull diagnostic model, we will ask the client to refresh diagnostics instead of sending them all.
            if capabilities.diagnostic_mode == DiagnosticMode::Push {
                for uri in &known_uris {
                    // Check if this worker is the most specific one for this URI
                    let Some(worker) = self.worker_manager.get_worker_for_uri(uri).await else {
                        continue;
                    };
                    let document = self.file_system.get_document(uri);
                    let diagnostics = worker.run_diagnostic(document).await;
                    match diagnostics {
                        Err(err) => {
                            error!("running diagnostics for {} failed: {err}", uri.as_str());
                            client_messages
                                .push(ClientMessage { r#type: MessageType::ERROR, message: err });
                        }
                        Ok(diagnostics) => new_diagnostics.extend(diagnostics),
                    }
                }
            }

            if !new_diagnostics.is_empty() {
                self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
            } else if capabilities.diagnostic_mode == DiagnosticMode::Pull {
                debug_assert!(
                    capabilities.refresh_diagnostics,
                    "pull mode requires refresh diagnostics capability"
                );

                // In pull diagnostic model, we ask the client to refresh diagnostics
                self.spawn_diagnostic_refresh();
            }
        }

        self.send_client_messages(client_messages).await;

        let mut registrations = vec![];

        // init all file watchers
        if capabilities.dynamic_watchers {
            for worker in self.worker_manager.read_workspace_workers().await.iter() {
                registrations.extend(worker.init_watchers().await);
            }

            // The watcher logic does handle per workspace, the dynamic worker is targeting the root (`file:///`).
            // We do not want to include them, or the client will watch the whole file system, which can be very expensive.
            // Current Assumption: The user does not modify files outside the workspace + the outside config file at the same session.
            // If this becomes a problem in the future, we can consider adding dynamic watchers for the dynamic worker as well,
            // but we need to optimize the logic.
            // Example optimization: Only return registered config paths instead of watching the workspace root.
            // Adding a "add" watcher for the root should still be applied.
            // if let Some(dynamic_worker) = self.worker_manager.read_dynamic_worker().await {
            //     registrations.extend(dynamic_worker.init_watchers().await);
            // }
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
        let clearing_diagnostics = self.worker_manager.stop_manager().await;

        // only clear diagnostics when we are using push diagnostics
        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push)
            && !clearing_diagnostics.is_empty()
        {
            self.clear_diagnostics(clearing_diagnostics).await;
        }
        self.file_system.clear();

        Ok(())
    }

    /// This method updates the configuration of each [WorkspaceWorker] and restarts them if necessary.
    /// It also manages dynamic registrations for file watchers and formatting based on the new configuration.
    /// It will remove/add dynamic registrations if the client supports it.
    /// As an example, if a workspace changes the configuration file path, the file watcher will be updated.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_didChangeConfiguration>
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let mut changes = WorkerChanges::default();
        let mut new_diagnostics = Vec::new();

        // `workingDirectories` sub workers are invisible to the client, only the workspace folders
        // are addressed by a configuration change.
        let folder_uris = {
            let workers = self.worker_manager.read_workspace_workers().await;
            workers
                .iter()
                .filter(|worker| !worker.is_sub_worker())
                .map(|worker| worker.get_root_uri().clone())
                .collect::<Vec<_>>()
        };

        // when null, request configuration from client; otherwise, parse as per-workspace options or use as global configuration
        let options = if params.settings == Value::Null {
            None
        } else {
            serde_json::from_value::<Vec<WorkspaceOption>>(params.settings.clone()).ok().or_else(
                || {
                    // fallback to old configuration
                    // for all workers (default only one)
                    let options = folder_uris
                        .iter()
                        .map(|workspace_uri| WorkspaceOption {
                            workspace_uri: workspace_uri.clone(),
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
            let configs = self.request_workspace_configuration(folder_uris.iter().collect()).await;

            // Only create WorkspaceOption when the config is Some
            configs
                .into_iter()
                .enumerate()
                .map(|(index, options)| WorkspaceOption {
                    workspace_uri: folder_uris[index].clone(),
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
        let fs = if diagnostic_mode == DiagnosticMode::Push {
            Some(self.file_system.as_ref())
        } else {
            None
        };

        // First reconcile the `workingDirectories` sub workers of every workspace folder. This has
        // to happen before the folder worker handles the option change, so its tool is rebuilt with
        // the new set of excluded roots.
        let mut auto_changed_folders = vec![];
        let mut restarted_roots: Vec<Uri> = vec![];
        // roots whose tool restarted without requesting new watchers, with the patterns they had
        // before: when their excluded roots also changed, the patterns of the rebuilt tool are
        // diffed against these and re-registered when they differ
        let mut restarted_keeping_watchers: Vec<(Uri, Vec<Pattern>)> = vec![];
        // the documents a worker already re-linted in this notification, per worker root
        let mut relinted: Vec<(Uri, Vec<Uri>)> = vec![];
        for option in &resolved_options {
            // Resolving the option walks the file system, so it only runs when its value actually
            // changed: an unrelated configuration change resolves to the very same roots. This is
            // also what decides whether the validation warnings are repeated.
            let option_changed = {
                let workers = self.worker_manager.read_workspace_workers().await;
                let previous = match workers.iter().find(|worker| {
                    !worker.is_sub_worker()
                        && roots_are_equal(worker.get_root_uri(), &option.workspace_uri)
                }) {
                    Some(worker) => Some(worker.get_options().await),
                    None => None,
                };

                previous.is_none_or(|previous| {
                    previous.get(WORKING_DIRECTORIES_OPTION)
                        != option.options.get(WORKING_DIRECTORIES_OPTION)
                })
            };
            if !option_changed {
                continue;
            }

            // The tools are not rebuilt here: the workers are about to handle the option change
            // themselves, and a tool which restarts for it does not have to be built twice.
            let (reconciled, auto_changed) = self
                .reconcile_sub_workers(
                    &option.workspace_uri,
                    &option.options,
                    &diagnostic_mode,
                    true,
                    false,
                )
                .await;
            changes.extend(reconciled);

            if auto_changed {
                auto_changed_folders.push(option.workspace_uri.clone());
            }
        }

        // Then hand the new options to the workspace folder worker and to its surviving sub
        // workers. A sub worker never sees the `workingDirectories` option itself, so it can not
        // spawn further working directories.
        {
            let workers = self.worker_manager.read_workspace_workers().await;
            let open_uris = self.file_system.keys();
            // route every open document once, the workers below group it by their own index
            let routes = WorkerManager::route_uris(&workers, &open_uris);

            for option in &resolved_options {
                let sub_options = sub_worker_options(&option.options);

                for (index, worker) in workers.iter().enumerate() {
                    let worker_options = if worker
                        .get_parent_uri()
                        .is_some_and(|parent| roots_are_equal(parent, &option.workspace_uri))
                    {
                        sub_options.clone()
                    } else if !worker.is_sub_worker()
                        && roots_are_equal(worker.get_root_uri(), &option.workspace_uri)
                    {
                        option.options.clone()
                    } else {
                        continue;
                    };

                    // only revalidate the documents this worker is responsible for
                    let owned_uris = WorkerManager::owned_uris(&routes, &open_uris, index);
                    let patterns_before = worker.watcher_patterns().await;
                    let result = worker
                        .did_change_configuration(
                            worker_options,
                            &mut needs_diagnostics_refresh,
                            fs,
                            &owned_uris,
                        )
                        .await;

                    if let Some(diagnostics) = result.diagnostics {
                        // only the documents the worker actually published for: one which came
                        // back clean still needs the diagnostics of its previous owner cleared
                        relinted.push((
                            worker.get_root_uri().clone(),
                            diagnostics.iter().map(|(uri, _)| uri.clone()).collect(),
                        ));
                        new_diagnostics.extend(diagnostics);
                    }

                    let tool_changed_watchers =
                        !result.new_watchers.is_empty() || !result.removed_watchers.is_empty();

                    if result.tool_restarted {
                        // the tool was rebuilt with the new context already
                        restarted_roots.push(worker.get_root_uri().clone());
                        if !tool_changed_watchers {
                            restarted_keeping_watchers
                                .push((worker.get_root_uri().clone(), patterns_before));
                        }
                    }
                    changes.unregistrations.extend(result.removed_watchers);
                    changes.registrations.extend(result.new_watchers);
                    changes.client_messages.extend(result.client_messages);

                    // The `auto` flag of this worker flipped, which adds or removes the
                    // `**/package.json` watcher. The tool knows nothing about it, so force the
                    // re-registration when it did not already request one.
                    if auto_changed_folders
                        .iter()
                        .any(|folder| roots_are_equal(folder, worker.get_root_uri()))
                        && !tool_changed_watchers
                    {
                        let (unregistrations, registrations) = worker.refresh_watchers().await;
                        changes.unregistrations.extend(unregistrations);
                        changes.registrations.extend(registrations);
                    }
                }
            }
        }

        // A worker whose excluded roots changed has to be rebuilt, unless its tool restarted for
        // the option change anyway: that restart already used the new build context.
        let rebuild_roots = changes
            .rebuild_roots
            .iter()
            .filter(|root| {
                !restarted_roots.iter().any(|restarted| roots_are_equal(restarted, root))
            })
            .cloned()
            .collect::<Vec<_>>();
        if !rebuild_roots.is_empty() {
            let (client_messages, unregistrations, registrations) =
                self.worker_manager.rebuild_workers(&rebuild_roots).await;
            changes.client_messages.extend(client_messages);
            changes.unregistrations.extend(unregistrations);
            changes.registrations.extend(registrations);
        }

        // A tool which restarted for the option change was rebuilt with the new excluded roots,
        // but it reported no watcher change: its patterns are derived from those roots, so they
        // are re-registered here. `rebuild_workers` does the same diff for the others.
        let refresh_roots = restarted_keeping_watchers
            .into_iter()
            .filter(|(root, _)| {
                changes.rebuild_roots.iter().any(|changed| roots_are_equal(changed, root))
            })
            .collect::<Vec<_>>();
        if !refresh_roots.is_empty() {
            let (unregistrations, registrations) =
                self.worker_manager.refresh_worker_watchers(&refresh_roots).await;
            changes.unregistrations.extend(unregistrations);
            changes.registrations.extend(registrations);
        }

        // Only now, with every tool rebuilt, are the documents which changed owner revalidated.
        // A document its new owner already re-linted above is skipped, unless that owner was
        // rebuilt afterwards and its diagnostics are stale.
        let already_linted = relinted
            .into_iter()
            .filter(|(root, _)| !rebuild_roots.iter().any(|rebuilt| roots_are_equal(rebuilt, root)))
            .flat_map(|(_, uris)| uris)
            .collect::<FxHashSet<_>>();
        self.revalidate_changed_owners(&mut changes, &diagnostic_mode, &already_linted).await;

        new_diagnostics.extend(std::mem::take(&mut changes.diagnostics));
        needs_diagnostics_refresh |= changes.needs_diagnostics_refresh;

        if diagnostic_mode == DiagnosticMode::Push {
            if !changes.cleared_diagnostics.is_empty() {
                self.clear_diagnostics(std::mem::take(&mut changes.cleared_diagnostics)).await;
            }
            if !new_diagnostics.is_empty() {
                self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
            }
        }

        if diagnostic_mode == DiagnosticMode::Pull && needs_diagnostics_refresh {
            // In pull diagnostic model, we ask the client to refresh diagnostics
            self.spawn_diagnostic_refresh();
        }

        if !changes.unregistrations.is_empty()
            && let Err(err) = self
                .client
                .unregister_capability(std::mem::take(&mut changes.unregistrations))
                .await
        {
            warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
        }
        if !changes.registrations.is_empty()
            && let Err(err) =
                self.client.register_capability(std::mem::take(&mut changes.registrations)).await
        {
            warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
        }

        self.send_client_messages(changes.client_messages).await;
    }

    /// This notification is sent when a configuration file of a tool changes (example: `.oxlintrc.json`).
    /// The server will re-lint the affected files and send updated diagnostics.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#workspace_didChangeWatchedFiles>
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        // ToDo: what if an empty changes flag is passed?
        debug!("watched file did change");

        let mut changes = WorkerChanges::default();
        let mut new_diagnostics = Vec::new();

        let mut needs_diagnostics_refresh = false;
        let diagnostic_mode =
            self.capabilities.get().map(|cap| cap.diagnostic_mode.clone()).unwrap_or_default();
        let dynamic_watchers =
            self.capabilities.get().is_some_and(|capabilities| capabilities.dynamic_watchers);
        let fs = if diagnostic_mode == DiagnosticMode::Push {
            Some(self.file_system.as_ref())
        } else {
            None
        };

        // Creating or deleting a watched file (a tool config file, or a `package.json` when
        // `workingDirectories` is in `auto` mode) can add or remove a working directory. Collect
        // the workspace folders to reconcile first, so a notification carrying several events for
        // the same folder only reconciles it once.
        let mut folders_to_reconcile: Vec<(Uri, serde_json::Value)> = vec![];
        // the events which reconcile a workspace folder: only they are already accounted for by
        // the rebuild, every other event of the same notification still reaches every worker
        let mut reconciling_uris: Vec<Uri> = vec![];
        for file_event in &params.changes {
            // Only a created or deleted file can add or remove a working directory, and only a
            // file the workspace folder actually watches (a `package.json` of the `auto` mode, or
            // one of its configuration files) can be that file. Resolving the option walks the
            // file system, so everything else must not reach it.
            if !matches!(file_event.typ, FileChangeType::CREATED | FileChangeType::DELETED) {
                continue;
            }
            // Only a `package.json` or a configuration file of the tool can change the result, and
            // that is decided by the file *names*, not by the watcher patterns: a setup with an
            // explicit `configPath` watches one file but still gains a working directory when a
            // config appears somewhere else.
            if !self.worker_manager.is_project_root_marker(&file_event.uri) {
                continue;
            }
            let Ok(file_path) = ResolvedPath::try_from(&file_event.uri) else {
                continue;
            };

            // A file can sit in several nested workspace folders, and each of them may declare its
            // own `workingDirectories`: the marker reconciles every one of them, not only the
            // nearest.
            for (parent_uri, options) in
                self.worker_manager.folder_workers_containing(&file_event.uri).await
            {
                if options.get(WORKING_DIRECTORIES_OPTION).is_none_or(serde_json::Value::is_null) {
                    continue;
                }
                if folders_to_reconcile.iter().any(|(uri, _)| roots_are_equal(uri, &parent_uri)) {
                    if !reconciling_uris.contains(&file_event.uri) {
                        reconciling_uris.push(file_event.uri.clone());
                    }
                    continue;
                }
                // `**/package.json` also matches inside `node_modules`, which the LSP glob syntax
                // can not exclude, so those events are dropped here, relative to the folder.
                let Ok(parent_path) = ResolvedPath::try_from(&parent_uri) else {
                    continue;
                };
                if is_below_ignored_directory(parent_path.as_path(), file_path.as_path()) {
                    continue;
                }

                if !reconciling_uris.contains(&file_event.uri) {
                    reconciling_uris.push(file_event.uri.clone());
                }
                folders_to_reconcile.push((parent_uri, options));
            }
        }

        // This has to happen before the events are handed to the workers, so a workspace folder
        // worker is rebuilt with the new set of excluded roots. The option value did not change,
        // so its validation warnings are not repeated.
        for (parent_uri, options) in folders_to_reconcile {
            // the option value did not change here, so the `auto` flag can not flip and the
            // watcher patterns of the worker stay the same
            let (reconciled, _auto_changed) = self
                .reconcile_sub_workers(&parent_uri, &options, &diagnostic_mode, false, true)
                .await;
            changes.extend(reconciled);
        }

        // A worker which the reconciliation just built or rebuilt already read the file which
        // triggered it, it must not be built a second time for that event. The other events of
        // the same notification are unrelated to the rebuild and still reach it.
        let mut rebuilt_roots = std::mem::take(&mut changes.rebuild_roots);
        rebuilt_roots.extend(std::mem::take(&mut changes.added_roots));

        let open_uris = self.file_system.keys();
        // the documents which were linted in this notification, so the revalidation of the
        // rebuilt workers below does not lint them a second time
        let mut already_linted = FxHashSet::default();

        for file_event in &params.changes {
            // A `package.json` is watched for the `auto` detection only. It is not a tool
            // configuration file, so it must not reach the tool of the workspace folder, nor the
            // tool of any of its sub workers.
            if self.worker_manager.is_detection_only_event(&file_event.uri).await {
                continue;
            }

            // We do not expect multiple changes from the same workspace folder.
            // If we should consider it, we need to map the events to the workers first,
            // to only restart the internal linter / diagnostics for once.
            //
            // Events below `node_modules` or `.git` are noise: `**/package.json` matches thousands
            // of them. They only reach a worker whose tool explicitly watches that exact file, for
            // example a config it extends from a dependency, and are never broadcast.
            //
            // Only the workers the change can affect see it: the ones whose root contains the
            // file, the ones rooted below the directory holding it (a config there governs them,
            // which is how a `workingDirectories` sub worker resolves its own config), and the
            // ones which registered it as an absolute watcher pattern, which is how a tool
            // watches a config file it extends from outside its root.
            //
            // A change which affects no worker at all can still be a config file shared by every
            // workspace, for example one in the home directory, so it is handed to all of them.
            let workers = self.worker_manager.read_workspace_workers().await;
            let mut targets = vec![];
            for (index, worker) in workers.iter().enumerate() {
                if worker.watches_uri(&file_event.uri).await {
                    targets.push(index);
                }
            }
            if targets.is_empty() {
                // Events below `node_modules` or `.git` are noise: `**/package.json` matches
                // thousands of them. They only reach a worker whose tool explicitly watches that
                // exact file, and are never broadcast.
                if Self::is_ignored_below_any(&workers, &file_event.uri) {
                    continue;
                }
                targets = (0..workers.len()).collect();
            }

            // A worker the reconciliation just (re)built read the new state already, but only
            // for the event which triggered that reconciliation. This is applied after the
            // fallback above, so skipping the only interested worker does not turn the event into
            // a broadcast.
            if reconciling_uris.contains(&file_event.uri) {
                targets.retain(|index| {
                    !rebuilt_roots
                        .iter()
                        .any(|root| roots_are_equal(root, workers[*index].get_root_uri()))
                });
            }

            // route every open document once, the workers below group it by their own index
            let routes = WorkerManager::route_uris(&workers, &open_uris);

            for index in targets {
                let worker = &workers[index];
                // only revalidate the documents this worker is responsible for
                let owned_uris = WorkerManager::owned_uris(&routes, &open_uris, index);
                let result = worker
                    .did_change_watched_files(
                        file_event,
                        &mut needs_diagnostics_refresh,
                        fs,
                        &owned_uris,
                    )
                    .await;

                if let Some(diagnostics) = result.diagnostics {
                    // only the documents the worker actually published for, see above
                    already_linted.extend(diagnostics.iter().map(|(uri, _)| uri.clone()));
                    new_diagnostics.extend(diagnostics);
                }
                changes.unregistrations.extend(result.removed_watchers);
                changes.registrations.extend(result.new_watchers);
                changes.client_messages.extend(result.client_messages);
            }
        }

        // The documents of a worker the reconciliation rebuilt, and the ones which changed owner,
        // are linted by their owner now, unless an event of this notification linted them already.
        self.revalidate_changed_owners(&mut changes, &diagnostic_mode, &already_linted).await;

        new_diagnostics.extend(std::mem::take(&mut changes.diagnostics));
        needs_diagnostics_refresh |= changes.needs_diagnostics_refresh;

        if diagnostic_mode == DiagnosticMode::Push {
            if !changes.cleared_diagnostics.is_empty() {
                self.clear_diagnostics(std::mem::take(&mut changes.cleared_diagnostics)).await;
            }
            if !new_diagnostics.is_empty() {
                self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
            }
        }

        if diagnostic_mode == DiagnosticMode::Pull && needs_diagnostics_refresh {
            // In pull diagnostic model, we ask the client to refresh diagnostics
            self.spawn_diagnostic_refresh();
        }

        if dynamic_watchers {
            if !changes.unregistrations.is_empty()
                && let Err(err) = self
                    .client
                    .unregister_capability(std::mem::take(&mut changes.unregistrations))
                    .await
            {
                warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
            }

            if !changes.registrations.is_empty()
                && let Err(err) = self
                    .client
                    .register_capability(std::mem::take(&mut changes.registrations))
                    .await
            {
                warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
            }
        }

        self.send_client_messages(changes.client_messages).await;
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
        let changed_folder_paths = params
            .event
            .added
            .iter()
            .chain(params.event.removed.iter())
            .map(|folder| folder.uri.clone())
            .collect::<Vec<_>>();
        let diagnostic_mode = capabilities.map(|c| c.diagnostic_mode.clone()).unwrap_or_default();

        // === Phase 1: Update worker state (brief write lock, no async I/O) ===
        // Extract workers that need to be shut down and update the mode flags.
        let workers_to_shutdown = self
            .worker_manager
            .update_workspace_folders(&params.event.added, &params.event.removed)
            .await;

        // === Phase 2: Shut down removed workers (no lock held) ===
        let mut cleared_diagnostics: Vec<Uri> = vec![];
        let mut removed_registrations = vec![];
        let mut client_messages = vec![];
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
        // a directory the client opened as its own workspace folder never gets a second,
        // shadowing sub worker
        let mut claimed_roots = self
            .worker_manager
            .read_workspace_workers()
            .await
            .iter()
            .map(|worker| worker.get_root_uri().clone())
            .collect::<Vec<_>>();
        // every workspace folder root, the ones this notification adds included
        let mut folder_roots = self.worker_manager.folder_roots().await;
        folder_roots.extend(params.event.added.iter().map(|folder| folder.uri.clone()));
        claimed_roots.extend(params.event.added.iter().map(|folder| folder.uri.clone()));

        for (index, folder) in params.event.added.into_iter().enumerate() {
            let worker = self.worker_manager.create_worker(folder.uri, diagnostic_mode.clone());
            let options = configurations.get(index).unwrap_or(&serde_json::Value::Null);

            // starting the worker also creates one sub worker per `workingDirectories` entry
            let (sub_workers, messages) = self
                .worker_manager
                .start_folder_worker(
                    &worker,
                    options.clone(),
                    &diagnostic_mode,
                    &claimed_roots,
                    &folder_roots,
                )
                .await;
            claimed_roots.extend(sub_workers.iter().map(|w| w.get_root_uri().clone()));
            client_messages.extend(messages);

            added_registrations.extend(worker.init_watchers().await);
            new_workers.push(worker);

            for sub_worker in sub_workers {
                added_registrations.extend(sub_worker.init_watchers().await);
                new_workers.push(sub_worker);
            }
        }

        // === Phase 4: Insert new workers (brief write lock, no async I/O) ===
        self.worker_manager.add_workers(new_workers).await;

        // === Phase 5: Reconcile the `workingDirectories` sub workers (no lock held) ===
        // A workspace folder opened on a root which a sub worker served replaces it, and a folder
        // which was removed hands its root back to a sub worker. The option value did not change,
        // so its validation warnings are not repeated.
        // Only a folder which contains, or sits inside, one of the folders which were added or
        // removed can have gained or lost a working directory.
        let changed_paths = changed_folder_paths
            .iter()
            .filter_map(|uri| ResolvedPath::try_from(uri).ok())
            .collect::<Vec<_>>();

        let mut new_diagnostics = vec![];
        let mut needs_diagnostics_refresh = false;
        let mut reconcile_changes = WorkerChanges::default();
        for (parent_uri, options) in self.folders_with_working_directories().await {
            let Ok(parent_path) = ResolvedPath::try_from(&parent_uri) else {
                continue;
            };
            let related = changed_paths.iter().any(|changed| {
                let (changed, parent) = (changed.as_path(), parent_path.as_path());
                changed.starts_with(parent) || parent.starts_with(changed)
            });
            if !related {
                continue;
            }

            let (reconciled, _auto_changed) = self
                .reconcile_sub_workers(&parent_uri, &options, &diagnostic_mode, false, true)
                .await;

            reconcile_changes.extend(reconciled);
        }

        // The documents of the workers a removed workspace folder shut down are handed over with
        // the ones of the reconciliation, so a document a new owner republishes for below is not
        // cleared first: that would publish the same document twice and make the editor flicker.
        reconcile_changes.cleared_diagnostics.extend(std::mem::take(&mut cleared_diagnostics));

        // the documents which changed owner, and the ones of the rebuilt workers, are linted by
        // their owner now that every tool is up to date
        self.revalidate_changed_owners(
            &mut reconcile_changes,
            &diagnostic_mode,
            &FxHashSet::default(),
        )
        .await;

        new_diagnostics.extend(reconcile_changes.diagnostics);
        cleared_diagnostics.extend(reconcile_changes.cleared_diagnostics);
        added_registrations.extend(reconcile_changes.registrations);
        removed_registrations.extend(reconcile_changes.unregistrations);
        client_messages.extend(reconcile_changes.client_messages);
        needs_diagnostics_refresh |= reconcile_changes.needs_diagnostics_refresh;

        // === Phase 6: Clear diagnostics and update client watchers (no lock held) ===
        if diagnostic_mode == DiagnosticMode::Push {
            if !cleared_diagnostics.is_empty() {
                self.clear_diagnostics(cleared_diagnostics).await;
            }
            if !new_diagnostics.is_empty() {
                self.publish_all_diagnostics(new_diagnostics, ConcurrentHashMap::default()).await;
            }
        }

        if diagnostic_mode == DiagnosticMode::Pull && needs_diagnostics_refresh {
            // In pull diagnostic model, we ask the client to refresh diagnostics
            self.spawn_diagnostic_refresh();
        }

        if capabilities.is_some_and(|c| c.dynamic_watchers) {
            // Unregister first: a watcher id is derived from the root, and a root which changes
            // owner (a sub worker replaced by a workspace folder worker, or the other way around)
            // produces an unregistration and a registration for the very same id.
            if !removed_registrations.is_empty()
                && let Err(err) = self.client.unregister_capability(removed_registrations).await
            {
                warn!("sending unregisterCapability.didChangeWatchedFiles failed: {err}");
            }

            if !added_registrations.is_empty()
                && let Err(err) = self.client.register_capability(added_registrations).await
            {
                warn!("sending registerCapability.didChangeWatchedFiles failed: {err}");
            }
        }

        // === Phase 7: Show messages to the client ===
        self.send_client_messages(client_messages).await;
    }

    /// It will save the in-memory file content, because non file-system files needs to be keep tracked too, and can not be accessed by the OS file system.
    /// It will re-lint the file and send updated diagnostics, if necessary.
    ///
    /// See: <https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_didSave>
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("oxc server did save");
        let uri = params.text_document.uri;
        if let Some(content) = params.text {
            self.file_system.set(uri.clone(), content);
        }

        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push) {
            let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
                return;
            };
            let document = self.file_system.get_document(&uri);
            match worker.run_diagnostic_on_save(document).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    self.client.show_message(MessageType::ERROR, err).await;
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
        if let Some(content) = params
            .content_changes
            .into_iter()
            .next()
            .map(|c: TextDocumentContentChangeEvent| c.text)
        {
            self.file_system.set(uri.clone(), content);
        }

        let document = self.file_system.get_document(&uri);

        let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
            return;
        };
        // Remove the internal cache for the document.
        // When the editor requests `textDocument/codeAction`, it may use its diagnostic cache to generate actions.
        // This could cause code actions to be generated with stale diagnostics if the cache is not cleared here.
        // This should never happen, because this server expects `textDocument/diagnostic` is requested beforehand.
        // Sadly, some editors/extensions have bugs, so we need to make sure the cache is cleared on change.
        worker.remove_uri_cache(&uri).await;

        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push) {
            match worker.run_diagnostic_on_change(document).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    self.client.show_message(MessageType::ERROR, err).await;
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
            let (registrations, client_messages) = self
                .worker_manager
                .ensure_worker_for_file_uri(&uri, diagnostic_mode, dynamic_watchers)
                .await;

            if let Some(registrations) = registrations
                && let Err(err) = self.client.register_capability(vec![registrations]).await
            {
                warn!("registering file watchers for single-file workspace failed: {err}");
            }

            self.send_client_messages(client_messages).await;
        }

        let content = params.text_document.text;

        self.file_system.set_with_language(
            uri.clone(),
            LanguageId::new(params.text_document.language_id),
            content,
        );

        if self.capabilities.get().is_some_and(|cap| cap.diagnostic_mode == DiagnosticMode::Push) {
            let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
                return;
            };

            let document = self.file_system.get_document(&uri);

            match worker.run_diagnostic(document).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    self.client.show_message(MessageType::ERROR, err).await;
                }
                Ok(diagnostics) => {
                    if !diagnostics.is_empty() {
                        let version_map = ConcurrentHashMap::default();
                        version_map.pin().insert(uri, params.text_document.version);
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
        self.file_system.remove(uri);

        let Some(worker) = self.worker_manager.get_worker_for_uri(uri).await else {
            return;
        };

        worker.remove_uri_cache(&params.text_document.uri).await;

        // Clone the root URI now so we can use it after dropping the read lock.
        let worker_root_uri = if self.worker_manager.is_single_file_mode() {
            Some(worker.get_root_uri().clone())
        } else {
            None
        };

        // Drop the read lock before potentially acquiring the write lock in
        // try_shutdown_empty_workspace.
        drop(worker);

        if let Some(root_uri) = worker_root_uri {
            let open_uris = self.file_system.keys();
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
        let uri = params.text_document.uri;
        let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
            return Ok(None);
        };

        let is_open_document = self.file_system.is_open(&uri);

        let params = crate::CodeActionParams {
            uri,
            range: params.range,
            context: params.context,
            is_open_document,
        };

        let code_actions = worker.get_code_actions_or_commands(params).await;

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
        // at the moment we only support `fixAll` command, which returns text-edits.
        // move this check when we support other type of commands
        if !self.capabilities.get().unwrap().workspace_apply_edit {
            return Err(Error::invalid_params("client does not support workspace apply edit"));
        }
        // Collect all edits under a brief read lock, then release it
        // before performing client RPCs to avoid blocking writers.
        // TODO: recheck if we should return the first found workspace edit instead of merging edits from all workers.
        // This can cause edit conflicts when the same line/column on the same file is edited by different workers.
        let edits: Vec<WorkspaceEdit> = {
            let mut edits = Vec::new();
            {
                let workers = self.worker_manager.read_workspace_workers().await;
                for worker in workers.iter() {
                    match worker.execute_command(&params.command, params.arguments.clone()).await {
                        Ok(Some(edit)) => edits.push(edit),
                        Ok(None) => {}
                        Err(err) => return Err(Error::new(err)),
                    }
                }
            }

            {
                if let Some(worker) = self.worker_manager.read_dynamic_worker() {
                    match worker.execute_command(&params.command, params.arguments.clone()).await {
                        Ok(Some(edit)) => edits.push(edit),
                        Ok(None) => {}
                        Err(err) => return Err(Error::new(err)),
                    }
                }
            }

            edits
        };

        if !edits.is_empty() {
            for edit in edits {
                self.client.apply_edit(edit).await?;
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

        let document = self.file_system.get_document(uri);
        let diagnostics = worker.run_diagnostic(document).await;

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

        let document = self.file_system.get_document(uri);
        match worker.format_file(document).await {
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
    pub fn new(client: Client, server_info: ServerInfo, worker_manager: WorkerManager) -> Self {
        Self {
            client,
            server_info,
            worker_manager,
            capabilities: OnceCell::new(),
            file_system: Arc::new(LSPFileSystem::default()),
            pending_initialization_messages: OnceCell::new(),
        }
    }

    /// Ask the client to refresh pull diagnostics, without awaiting its reply
    /// inside the calling handler.
    ///
    /// `workspace/diagnostic/refresh` is a server-to-client *request*: the
    /// client may do arbitrary work before responding — typically re-pulling
    /// `textDocument/diagnostic` from this very server. Awaiting the reply
    /// inside a notification handler therefore pins one of the transport's
    /// concurrency slots (4 by default in tower-lsp-server) for the whole
    /// round-trip. Once every slot is pinned this way, the server can no
    /// longer service the diagnostic pulls the client is waiting on before it
    /// replies: a circular wait with no timeout and no recovery. The refresh
    /// is advisory and nothing depends on its result beyond logging, so it is
    /// detached instead.
    fn spawn_diagnostic_refresh(&self) {
        let client = self.client.clone();
        tokio::spawn(async move {
            if let Err(err) = client.workspace_diagnostic_refresh().await {
                warn!("sending workspace/diagnostic/refresh failed: {err}");
            }
        });
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

    /// Whether `uri` is below a `node_modules` or `.git` directory of one of the worker roots.
    fn is_ignored_below_any(workers: &[WorkspaceWorker], uri: &Uri) -> bool {
        let Ok(file) = ResolvedPath::try_from(uri) else {
            return false;
        };
        let file_path = file.as_path();

        workers.iter().any(|worker| {
            worker
                .get_root_path()
                .is_some_and(|root_path| is_below_ignored_directory(root_path, file_path))
        })
    }

    /// The workspace folder workers which declare `workingDirectories`, with their options.
    async fn folders_with_working_directories(&self) -> Vec<(Uri, serde_json::Value)> {
        let workers = self.worker_manager.read_workspace_workers().await;
        let mut folders = vec![];

        for worker in workers.iter().filter(|worker| !worker.is_sub_worker()) {
            let options = worker.get_options().await;
            if options.get(WORKING_DIRECTORIES_OPTION).is_none_or(serde_json::Value::is_null) {
                continue;
            }
            folders.push((worker.get_root_uri().clone(), options));
        }

        folders
    }

    /// Recompute the `workingDirectories` of the workspace folder worker `parent_uri` and
    /// reconcile its sub workers.
    ///
    /// Shuts down the sub workers which disappeared. The tools whose build context changed were
    /// already rebuilt by the manager, so the open documents whose responsible worker changed can
    /// be revalidated right away: the ones a new sub worker took over, and the ones orphaned by a
    /// removed sub worker.
    ///
    /// Returns everything the caller has to forward to the client, and whether the `auto` flag of
    /// the workspace folder worker flipped.
    async fn reconcile_sub_workers(
        &self,
        parent_uri: &Uri,
        options: &serde_json::Value,
        diagnostic_mode: &DiagnosticMode,
        report_warnings: bool,
        rebuild_now: bool,
    ) -> (WorkerChanges, bool) {
        let dynamic_watchers =
            self.capabilities.get().is_some_and(|capabilities| capabilities.dynamic_watchers);
        let sync = self
            .worker_manager
            .sync_sub_workers(
                parent_uri,
                options,
                diagnostic_mode,
                dynamic_watchers,
                report_warnings,
                rebuild_now,
            )
            .await;

        let mut changes = WorkerChanges {
            registrations: sync.registrations,
            unregistrations: sync.unregistrations,
            client_messages: sync.client_messages,
            rebuild_roots: sync.rebuild_roots,
            added_roots: sync.added_roots.clone(),
            ..WorkerChanges::default()
        };

        let mut changed_roots = sync.added_roots;
        for worker in sync.removed {
            changed_roots.push(worker.get_root_uri().clone());
            let (uris, unregistrations) = worker.shutdown().await;
            changes.cleared_diagnostics.extend(uris);
            changes.unregistrations.extend(unregistrations);
        }

        // A worker the reconciliation rebuilt reads other configuration files now, so its open
        // documents have to be linted again exactly like the ones which changed owner. The caller
        // revalidates, once, when it knows which documents it linted itself: a rebuild and an
        // event of the same notification must not lint the same document twice.
        changed_roots.extend(changes.rebuild_roots.iter().cloned());

        changes.changed_roots = changed_roots;

        (changes, sync.auto_changed)
    }

    /// Revalidate the open documents whose responsible worker changed.
    ///
    /// Without this a document keeps the diagnostics of its previous owner. The new owner
    /// publishes for it, so its previous diagnostics are not cleared first: that would publish the
    /// same document twice and make the editor flicker.
    ///
    /// `already_linted` holds the documents their new owner linted earlier in the same
    /// notification with an up to date tool: linting them again would only publish the very same
    /// diagnostics twice.
    async fn revalidate_changed_owners(
        &self,
        changes: &mut WorkerChanges,
        diagnostic_mode: &DiagnosticMode,
        already_linted: &FxHashSet<Uri>,
    ) {
        let changed_roots = std::mem::take(&mut changes.changed_roots);
        if changed_roots.is_empty() {
            return;
        }

        let affected = self
            .file_system
            .keys()
            .into_iter()
            .filter(|uri| {
                uri.scheme().as_str() == "file" && find_root_for_uri(&changed_roots, uri).is_some()
            })
            .collect::<Vec<_>>();

        // in pull mode the client has to ask for their diagnostics again
        changes.needs_diagnostics_refresh |= !affected.is_empty();

        if *diagnostic_mode != DiagnosticMode::Push {
            return;
        }

        let mut revalidated = FxHashSet::default();
        for uri in affected {
            if already_linted.contains(&uri) {
                // its new owner published for it already
                revalidated.insert(uri);
                continue;
            }
            let Some(worker) = self.worker_manager.get_worker_for_uri(&uri).await else {
                continue;
            };
            match worker.run_diagnostic(self.file_system.get_document(&uri)).await {
                Err(err) => {
                    error!("running diagnostics for {} failed: {err}", uri.as_str());
                    changes
                        .client_messages
                        .push(ClientMessage { r#type: MessageType::ERROR, message: err });
                }
                Ok(diagnostics) => {
                    for (uri, _) in &diagnostics {
                        revalidated.insert(uri.clone());
                    }
                    changes.diagnostics.extend(diagnostics);
                }
            }
        }

        changes.cleared_diagnostics.retain(|uri| !revalidated.contains(uri));
    }

    /// Send multiple messages to the client, if any.
    /// Will cap the number of messages to 5, to avoid flooding the client.
    async fn send_client_messages(&self, messages: Vec<ClientMessage>) {
        let max_messages = 5;
        let messages_to_send = if messages.len() > max_messages {
            let extra_message = ClientMessage {
                r#type: MessageType::WARNING,
                message: format!(
                    "{} more messages not shown. See LSP logs for details.",
                    messages.len() - max_messages + 1
                ),
            };
            let mut messages_to_send =
                messages.into_iter().take(max_messages - 1).collect::<Vec<_>>();
            messages_to_send.push(extra_message);
            messages_to_send
        } else {
            messages
        };

        join_all(
            messages_to_send
                .into_iter()
                .map(|message| self.client.show_message(message.r#type, message.message)),
        )
        .await;
    }
}
