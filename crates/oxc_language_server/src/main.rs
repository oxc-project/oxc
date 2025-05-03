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
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        ExecuteCommandParams, InitializeParams, InitializeResult, InitializedParams, ServerInfo,
        Uri, WorkspaceEdit,
    },
};
// #
use capabilities::Capabilities;
use code_actions::CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC;
use commands::{FIX_ALL_COMMAND_ID, FixAllCommandArgs};
use worker::WorkspaceWorker;

mod capabilities;
mod code_actions;
mod commands;
mod linter;
#[cfg(test)]
mod tester;
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
    #[expect(deprecated)] // TODO: FIXME
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        // ToDo: add support for multiple workspace folders
        // maybe fallback when the client does not support it
        let root_worker =
            WorkspaceWorker::new(&params.root_uri.unwrap(), options.clone().unwrap_or_default());

        *self.workspace_workers.lock().await = vec![root_worker];

        if let Some(value) = options {
            info!("initialize: {value:?}");
            info!("language server version: {:?}", env!("CARGO_PKG_VERSION"));
        }

        let capabilities = Capabilities::from(params.capabilities);
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

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let workers = self.workspace_workers.lock().await;
        let new_diagnostics: papaya::HashMap<String, Vec<Diagnostic>, FxBuildHasher> =
            ConcurrentHashMap::default();

        // when we have only workspace folder, apply to it
        // ToDo: check if this is really safe because the client could still pass an empty settings
        if workers.len() == 1 {
            let worker = workers.first().unwrap();
            let Some(diagnostics) = worker.did_change_configuration(params.settings).await else {
                return;
            };
            for (uri, reports) in &diagnostics.pin() {
                new_diagnostics
                    .pin()
                    .insert(uri.clone(), reports.iter().map(|d| d.diagnostic.clone()).collect());
            }

        // else check if the client support workspace configuration requests so we can only restart only the needed workers
        } else if self
            .capabilities
            .get()
            .is_some_and(|capabilities| capabilities.workspace_configuration)
        {
            let mut config_items = vec![];
            for worker in workers.iter() {
                let Some(uri) = worker.get_root_uri() else {
                    continue;
                };
                // ToDo: this is broken in VSCode. Check how we can get the language server configuration from the client
                // changing `section` to `oxc` will return the client configuration.
                config_items.push(ConfigurationItem {
                    scope_uri: Some(uri),
                    section: Some("oxc_language_server".into()),
                });
            }

            let Ok(configs) = self.client.configuration(config_items).await else {
                debug!("failed to get configuration");
                return;
            };

            // we expect that the client is sending all the configuration items in order and completed
            // this is a LSP specification and errors should be reported on the client side
            for (index, worker) in workers.iter().enumerate() {
                let config = &configs[index];
                let Some(diagnostics) = worker.did_change_configuration(config.clone()).await
                else {
                    continue;
                };

                for (uri, reports) in &diagnostics.pin() {
                    new_diagnostics.pin().insert(
                        uri.clone(),
                        reports.iter().map(|d| d.diagnostic.clone()).collect(),
                    );
                }
            }

            // we have multiple workspace folders and the client does not support workspace configuration requests
            // we assume that every workspace is under effect
        } else {
            for worker in workers.iter() {
                let Some(diagnostics) =
                    worker.did_change_configuration(params.settings.clone()).await
                else {
                    continue;
                };

                for (uri, reports) in &diagnostics.pin() {
                    new_diagnostics.pin().insert(
                        uri.clone(),
                        reports.iter().map(|d| d.diagnostic.clone()).collect(),
                    );
                }
            }
        }

        if new_diagnostics.is_empty() {
            return;
        }

        let x = &new_diagnostics
            .pin()
            .into_iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();

        self.publish_all_diagnostics(x).await;
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

    async fn initialized(&self, _params: InitializedParams) {
        debug!("oxc initialized.");
    }

    async fn shutdown(&self) -> Result<()> {
        self.clear_all_diagnostics().await;
        Ok(())
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
