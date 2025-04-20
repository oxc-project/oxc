use code_actions::CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC;
use commands::{FIX_ALL_COMMAND_ID, FixAllCommandArgs};
use futures::future::join_all;
use log::{debug, info};
use oxc_linter::FixKind;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};
use tokio::sync::{Mutex, OnceCell, SetError};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        CodeActionParams, CodeActionResponse, ConfigurationItem, Diagnostic,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, ExecuteCommandParams, InitializeParams, InitializeResult,
        InitializedParams, ServerInfo, Uri, WorkspaceEdit,
    },
};
use worker::WorkspaceWorker;

use crate::capabilities::Capabilities;

mod capabilities;
mod code_actions;
mod commands;
mod linter;
mod worker;

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const OXC_CONFIG_FILE: &str = ".oxlintrc.json";

struct Backend {
    client: Client,
    // Each Workspace has it own worker with Linter (and in the future the formatter).
    // We must respect each program inside with its own root folder
    // and can not use shared programmes across multiple workspaces.
    // Each Workspace can have its own server configuration and program root configuration.
    workspace_workers: Mutex<Vec<WorkspaceWorker>>,
    capabilities: OnceCell<Capabilities>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Run {
    OnSave,
    #[default]
    OnType,
}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Options {
    run: Run,
    config_path: Option<String>,
    flags: FxHashMap<String, String>,
}

impl Options {
    fn use_nested_configs(&self) -> bool {
        !self.flags.contains_key("disable_nested_config") || self.config_path.is_some()
    }

    fn fix_kind(&self) -> FixKind {
        self.flags.get("fix_kind").map_or(FixKind::SafeFix, |kind| match kind.as_str() {
            "safe_fix" => FixKind::SafeFix,
            "safe_fix_or_suggestion" => FixKind::SafeFixOrSuggestion,
            "dangerous_fix" => FixKind::DangerousFix,
            "dangerous_fix_or_suggestion" => FixKind::DangerousFixOrSuggestion,
            "none" => FixKind::None,
            "all" => FixKind::All,
            _ => {
                info!("invalid fix_kind flag `{kind}`, fallback to `safe_fix`");
                FixKind::SafeFix
            }
        })
    }
}

impl LanguageServer for Backend {
    #[expect(deprecated)] // `params.root_uri` is deprecated, we are only falling back to it if no workspace folder is provided
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // initialization_options can be anything, so we are requesting `workspace/configuration` when no initialize options are provided
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        if let Some(value) = &options {
            info!("initialize: {value:?}");
            info!("language server version: {:?}", env!("CARGO_PKG_VERSION"));
        }

        let capabilities = Capabilities::from(params.capabilities);

        if let Some(workspace_folder) = params.workspace_folders.as_ref() {
            if workspace_folder.is_empty() {
                return Err(Error::invalid_params("workspace folder is empty"));
            }

            let mut workers = vec![];
            // when we have only one workspace folder and the client already passed the configuration
            if workspace_folder.len() == 1 && options.is_some() {
                let root_worker =
                    WorkspaceWorker::new(&workspace_folder.first().unwrap().uri, options.unwrap());
                workers.push(root_worker);
                // else check if the client support workspace configuration requests
                // and we can request the configuration for each workspace folder
            } else if capabilities.workspace_configuration {
                let configs = self
                    .request_workspace_configuration(
                        workspace_folder.iter().map(|w| w.uri.clone()).collect(),
                    )
                    .await;
                for (index, folder) in workspace_folder.iter().enumerate() {
                    let workspace_options = configs
                        .get(index)
                        // when there is no valid index fallback to the initialize options
                        .unwrap_or(&options)
                        .clone()
                        // no valid index or initialize option, still fallback to default
                        .unwrap_or_default();

                    workers.push(WorkspaceWorker::new(&folder.uri, workspace_options));
                }
            } else {
                for folder in workspace_folder {
                    workers.push(WorkspaceWorker::new(
                        &folder.uri,
                        options.clone().unwrap_or_default(),
                    ));
                }
            }

            *self.workspace_workers.lock().await = workers;
        // fallback to root uri if no workspace folder is provided
        } else if let Some(root_uri) = params.root_uri.as_ref() {
            // use the initialize options if the client does not support workspace configuration or already provided one
            let root_options = if options.is_some() {
                options.clone().unwrap()
            // check if the client support workspace configuration requests
            } else if capabilities.workspace_configuration {
                let configs = self.request_workspace_configuration(vec![root_uri.clone()]).await;
                configs
                    .first()
                    // options is already none, no need to pass it here
                    .unwrap_or(&None)
                    // no valid index or initialize option, still fallback to default
                    .clone()
                    .unwrap_or_default()
            // no initialize options provided and the client does not support workspace configuration
            // fallback to default
            } else {
                Options::default()
            };

            let root_worker = WorkspaceWorker::new(root_uri, root_options);
            *self.workspace_workers.lock().await = vec![root_worker];
        // one of the two (workspace folder or root_uri) must be provided
        } else {
            return Err(Error::invalid_params("no workspace folder or root uri"));
        }

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
            server_info: Some(ServerInfo { name: "oxc".into(), version: None }),
            offset_encoding: None,
            capabilities: capabilities.into(),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        debug!("oxc initialized.");
    }

    async fn shutdown(&self) -> Result<()> {
        self.clear_all_diagnostics().await;
        Ok(())
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let workers = self.workspace_workers.lock().await;
        let params_options = serde_json::from_value::<Options>(params.settings).ok();

        // when we have only workspace folder and the client provided us the configuration
        // we can just update the worker with the new configuration
        if workers.len() == 1 && params_options.is_some() {
            let worker = workers.first().unwrap();
            worker.did_change_configuration(&params_options.unwrap()).await;

        // else check if the client support workspace configuration requests so we can only restart only the needed workers
        } else if self
            .capabilities
            .get()
            .is_some_and(|capabilities| capabilities.workspace_configuration)
        {
            let configs = self
                .request_workspace_configuration(
                    workers.iter().map(worker::WorkspaceWorker::get_root_uri).collect(),
                )
                .await;
            // we expect that the client is sending all the configuration items in order and completed
            // this is a LSP specification and errors should be reported on the client side
            for (index, worker) in workers.iter().enumerate() {
                // get the index or fallback to the initialize options
                let config = configs.get(index).unwrap_or(&params_options);

                // change anything
                let Some(config) = config else {
                    continue;
                };

                worker.did_change_configuration(config).await;
            }

            // we have multiple workspace folders and the client does not support workspace configuration requests
            // the client must provide a configuration change or else we do not know what to do
        } else if params_options.is_some() {
            for worker in workers.iter() {
                worker.did_change_configuration(&params_options.clone().unwrap()).await;
            }
        }
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let workers = self.workspace_workers.lock().await;
        // ToDo: what if an empty changes flag is passed?
        debug!("watched file did change");
        let all_diagnostics: papaya::HashMap<String, Vec<Diagnostic>, FxBuildHasher> =
            ConcurrentHashMap::default();
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

            for (key, value) in &diagnostics.pin() {
                all_diagnostics
                    .pin()
                    .insert(key.clone(), value.iter().map(|d| d.diagnostic.clone()).collect());
            }
        }

        if all_diagnostics.is_empty() {
            return;
        }

        let x = &all_diagnostics
            .pin()
            .into_iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();

        self.publish_all_diagnostics(x).await;
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let mut workers = self.workspace_workers.lock().await;
        let mut cleared_diagnostics = vec![];
        for folder in params.event.removed {
            let Some((index, worker)) = workers
                .iter()
                .enumerate()
                .find(|(_, worker)| worker.is_responsible_for_uri(&folder.uri))
            else {
                continue;
            };
            cleared_diagnostics.extend(worker.get_clear_diagnostics().await);
            workers.remove(index);
        }

        self.publish_all_diagnostics(&cleared_diagnostics).await;

        if self.capabilities.get().is_some_and(|capabilities| capabilities.workspace_configuration)
        {
            let configurations = self
                .request_workspace_configuration(
                    params.event.added.iter().map(|w| w.uri.clone()).collect(),
                )
                .await;

            for (index, folder) in params.event.added.iter().enumerate() {
                let option = configurations.get(index).unwrap_or(&None);
                let option = option.clone().unwrap_or(Options::default());

                workers.push(WorkspaceWorker::new(&folder.uri, option));
            }
        } else {
            for folder in params.event.added {
                workers.push(WorkspaceWorker::new(&folder.uri, Options::default()));
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("oxc server did save");
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.lock().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };
        if !worker.should_lint_on_run_type(Run::OnSave).await {
            return;
        }
        if let Some(diagnostics) = worker.lint_file(uri, None).await {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    None,
                )
                .await;
        }
    }

    /// When the document changed, it may not be written to disk, so we should
    /// get the file context from the language client
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.lock().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };
        if !worker.should_lint_on_run_type(Run::OnType).await {
            return;
        }
        let content = params.content_changes.first().map(|c| c.text.clone());
        if let Some(diagnostics) = worker.lint_file(uri, content).await {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    Some(params.text_document.version),
                )
                .await;
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.lock().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };

        let content = params.text_document.text;
        if let Some(diagnostics) = worker.lint_file(uri, Some(content)).await {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    Some(params.text_document.version),
                )
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.lock().await;
        let Some(worker) = workers.iter().find(|worker| worker.is_responsible_for_uri(uri)) else {
            return;
        };
        worker.remove_diagnostics(&params.text_document.uri).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let workers = self.workspace_workers.lock().await;
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
            let workers = self.workspace_workers.lock().await;
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
}

impl Backend {
    /// Request the workspace configuration from the client
    /// and return the options for each workspace folder.
    /// The check if the client support workspace configuration, should be done before.
    async fn request_workspace_configuration(&self, uris: Vec<Uri>) -> Vec<Option<Options>> {
        let length = uris.len();
        let config_items = uris
            .into_iter()
            .map(|uri| ConfigurationItem {
                scope_uri: Some(uri),
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
    // clears all diagnostics for workspace folders
    async fn clear_all_diagnostics(&self) {
        let mut cleared_diagnostics = vec![];
        for worker in self.workspace_workers.lock().await.iter() {
            cleared_diagnostics.extend(worker.get_clear_diagnostics().await);
        }
        self.publish_all_diagnostics(&cleared_diagnostics).await;
    }

    async fn publish_all_diagnostics(&self, result: &[(String, Vec<Diagnostic>)]) {
        join_all(result.iter().map(|(path, diagnostics)| {
            self.client.publish_diagnostics(Uri::from_str(path).unwrap(), diagnostics.clone(), None)
        }))
        .await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        workspace_workers: Mutex::new(vec![]),
        capabilities: OnceCell::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
