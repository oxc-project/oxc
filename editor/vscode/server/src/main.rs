mod linter;
mod options;
mod walk;

use crate::linter::{DiagnosticReport, ServerLinter};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;

use dashmap::DashMap;
use futures::future::join_all;
use tokio::sync::{OnceCell, SetError};
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, Diagnostic, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, MessageType, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkDoneProgressOptions, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Url>>,
    server_linter: ServerLinter,
    diagnostics_report_map: DashMap<String, Vec<DiagnosticReport>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.init(params.root_uri)?;

        Ok(InitializeResult {
            server_info: Some(ServerInfo { name: "oxc".into(), version: None }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
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

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "oxc initialized.").await;

        if let Some(Some(root_uri)) = self.root_uri.get() {
            let result = self.server_linter.run_full(root_uri);

            self.publish_all_diagnostics(
                &result
                    .into_iter()
                    .map(|(p, d)| (p, d.into_iter().map(|d| d.diagnostic).collect()))
                    .collect(),
            )
            .await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.handle_file_update(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.handle_file_update(params.text_document.uri).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.handle_file_update(params.text_document.uri).await;
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
        join_all(result.iter().map(|(path, diagnostics)| {
            self.client.publish_diagnostics(
                Url::from_file_path(path).unwrap(),
                diagnostics.clone(),
                None,
            )
        }))
        .await;
    }

    async fn handle_file_update(&self, uri: Url) {
        if let Some(Some(root_uri)) = self.root_uri.get() {
            if let Some(diagnostics) = self.server_linter.run_single(root_uri, &uri) {
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
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
