#![allow(unused)]
mod linter;
mod options;
mod walk;

use crate::linter::{DiagnosticReport, ServerLinter};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;

use dashmap::DashMap;
use futures::future::join_all;
use tokio::sync::{Mutex, OnceCell, SetError};
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, Diagnostic, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
    OneOf, Registration, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkDoneProgressOptions, WorkspaceEdit,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Url>>,
    server_linter: ServerLinter,
    diagnostics_report_map: DashMap<String, Vec<DiagnosticReport>>,
    options: Mutex<Options>,
}
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum Run {
    OnSave,
    #[default]
    OnType,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Options {
    run: Run,
    enable: bool,
}

impl Options {
    fn get_lint_level(&self) -> SyntheticRunLevel {
        if self.enable {
            match self.run {
                Run::OnSave => SyntheticRunLevel::OnSave,
                Run::OnType => SyntheticRunLevel::OnType,
            }
        } else {
            SyntheticRunLevel::Disable
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum SyntheticRunLevel {
    Disable,
    OnSave,
    OnType,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.init(params.root_uri)?;
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        if let Some(value) = options {
            debug!("initialize: {:?}", value);
            *self.options.lock().await = value;
        }
        Ok(InitializeResult {
            server_info: Some(ServerInfo { name: "oxc".into(), version: None }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        resolve_provider: None,
                    },
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let changed_options = match serde_json::from_value::<Options>(params.settings) {
            Ok(option) => option,
            Err(err) => {
                error!("error parsing settings: {:?}", err);
                return;
            }
        };
        debug!("{:?}", &changed_options.get_lint_level());
        if changed_options.get_lint_level() == SyntheticRunLevel::Disable {
            // clear all exists diagnostics when linter is disabled
            let opened_files = self.diagnostics_report_map.iter().map(|k| k.key().to_string());
            let cleared_diagnostics = opened_files
                .into_iter()
                .map(|uri| {
                    (
                        // should convert successfully, case the key is from `params.document.uri`
                        Url::from_str(&uri)
                            .ok()
                            .and_then(|url| url.to_file_path().ok())
                            .expect("should convert to path"),
                        vec![],
                    )
                })
                .collect::<Vec<_>>();
            self.publish_all_diagnostics(&cleared_diagnostics).await;
        }
        *self.options.lock().await = changed_options;
    }

    async fn initialized(&self, params: InitializedParams) {
        debug!("oxc initialized.");

        if let Some(Some(root_uri)) = self.root_uri.get() {
            self.server_linter.make_plugin(root_uri);
            // let result = self.server_linter.run_full(root_uri);

            // self.publish_all_diagnostics(
            // &result
            // .into_iter()
            // .map(|(p, d)| (p, d.into_iter().map(|d| d.diagnostic).collect()))
            // .collect(),
            // )
            // .await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("oxc server did save");
        // drop as fast as possible
        let run_level = { self.options.lock().await.get_lint_level() };
        if run_level < SyntheticRunLevel::OnSave {
            return;
        }
        self.handle_file_update(params.text_document.uri, None).await;
    }

    /// When the document changed, it may not be written to disk, so we should
    /// get the file context from the language client
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let run_level = { self.options.lock().await.get_lint_level() };
        if run_level < SyntheticRunLevel::OnType {
            return;
        }
        let content = params.content_changes.first().map(|c| c.text.clone());
        self.handle_file_update(params.text_document.uri, content).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let run_level = { self.options.lock().await.get_lint_level() };
        if run_level < SyntheticRunLevel::OnType {
            return;
        }
        self.handle_file_update(params.text_document.uri, None).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.diagnostics_report_map.remove(&uri);
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;

        if let Some(value) = self.diagnostics_report_map.get(&uri.to_string()) {
            if let Some(report) = value
                .iter()
                .find(|r| r.diagnostic.range == params.range && r.fixed_content.is_some())
            {
                let title =
                    report.diagnostic.message.split(':').next().map_or_else(
                        || "Fix this problem".into(),
                        |s| format!("Fix this {s} problem"),
                    );

                let fixed_content = report.fixed_content.clone().unwrap();

                return Ok(Some(vec![CodeActionOrCommand::CodeAction(CodeAction {
                    title,
                    kind: Some(CodeActionKind::QUICKFIX),
                    is_preferred: Some(true),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri,
                            vec![TextEdit {
                                range: fixed_content.range,
                                new_text: fixed_content.code,
                            }],
                        )])),
                        ..WorkspaceEdit::default()
                    }),
                    disabled: None,
                    data: None,
                    diagnostics: None,
                    command: None,
                })]));
            }
        }

        Ok(None)
    }
}

impl Backend {
    fn init(&self, root_uri: Option<Url>) -> Result<()> {
        self.root_uri.set(root_uri).map_err(|err| {
            let message = match err {
                SetError::AlreadyInitializedError(_) => "root uri already initialized".into(),
                SetError::InitializingError(_) => "initializing error".into(),
            };

            Error { code: ErrorCode::ParseError, message, data: None }
        })
    }

    #[allow(clippy::ptr_arg)]
    async fn publish_all_diagnostics(&self, result: &Vec<(PathBuf, Vec<Diagnostic>)>) {
        debug!("{:?}", result);
        join_all(result.iter().map(|(path, diagnostics)| {
            self.client.publish_diagnostics(
                Url::from_file_path(path).unwrap(),
                diagnostics.clone(),
                None,
            )
        }))
        .await;
    }

    async fn handle_file_update(&self, uri: Url, content: Option<String>) {
        if let Some(Some(root_uri)) = self.root_uri.get() {
            self.server_linter.make_plugin(root_uri);
            if let Some(diagnostics) = self.server_linter.run_single(root_uri, &uri, content) {
                self.client
                    .publish_diagnostics(
                        uri.clone(),
                        diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                        None,
                    )
                    .await;

                self.diagnostics_report_map.insert(uri.to_string(), diagnostics);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let server_linter = ServerLinter::new();
    let diagnostics_report_map = DashMap::new();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        root_uri: OnceCell::new(),
        server_linter,
        diagnostics_report_map,
        options: tokio::sync::Mutex::new(Options::default()),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
