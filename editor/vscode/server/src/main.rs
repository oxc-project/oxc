mod linter;
mod options;
mod walk;

use crate::linter::ServerLinter;
use std::fmt::Debug;
use std::path::PathBuf;

use futures::future::join_all;
use tokio::sync::{OnceCell, SetError};
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Url>>,
    server_linter: ServerLinter,
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
                // code_action_provider: Some(CodeActionProviderCapability::Options(
                //     CodeActionOptions {
                //         code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                //         work_done_progress_options: WorkDoneProgressOptions {
                //             work_done_progress: None,
                //         },
                //         resolve_provider: None,
                //     },
                // )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "oxc initialized.").await;

        if let Some(Some(root_uri)) = self.root_uri.get() {
            let result = self.server_linter.run_full(root_uri);

            self.publish_all_diagnostics(result).await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(Some(root_uri)) = self.root_uri.get() {
            match self.server_linter.run_single(root_uri, &params.text_document.uri) {
                Some((_, diagnostics)) => {
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, None)
                        .await;
                }
                None => {}
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(Some(root_uri)) = self.root_uri.get() {
            match self.server_linter.run_single(root_uri, &params.text_document.uri) {
                Some((_, diagnostics)) => {
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, None)
                        .await;
                }
                None => {}
            }
        }
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

    async fn publish_all_diagnostics(&self, result: Vec<(PathBuf, Vec<Diagnostic>)>) {
        join_all(result.into_iter().map(|(path, diagnostics)| {
            self.client.publish_diagnostics(Url::from_file_path(path).unwrap(), diagnostics, None)
        }))
        .await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let server_linter = ServerLinter::new();

    let (service, socket) =
        LspService::build(|client| Backend { client, root_uri: OnceCell::new(), server_linter })
            .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
