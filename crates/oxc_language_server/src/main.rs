mod linter;
mod options;

use crate::linter::{DiagnosticReport, ServerLinter};
use globset::Glob;
use ignore::gitignore::Gitignore;
use log::{debug, error, info};
use oxc_linter::{LintOptions, Linter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;

use dashmap::DashMap;
use futures::future::join_all;
use tokio::sync::{Mutex, OnceCell, RwLock, SetError};
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, ConfigurationItem, Diagnostic,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, OneOf, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkDoneProgressOptions, WorkspaceEdit,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Url>>,
    server_linter: RwLock<ServerLinter>,
    diagnostics_report_map: DashMap<String, Vec<DiagnosticReport>>,
    options: Mutex<Options>,
    gitignore_glob: Mutex<Option<Gitignore>>,
}
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum Run {
    OnSave,
    #[default]
    OnType,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Options {
    run: Run,
    enable: bool,
    config_path: String,
}

impl Default for Options {
    fn default() -> Self {
        Self { enable: true, run: Run::default(), config_path: ".eslintrc".into() }
    }
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
    fn get_config_path(&self) -> Option<PathBuf> {
        if self.config_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(&self.config_path))
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
        self.init_ignore_glob().await;
        self.init_linter_config().await;
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        if let Some(value) = options {
            info!("initialize: {:?}", value);
            info!("language server version: {:?}", env!("CARGO_PKG_VERSION"));
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
        let changed_options =
            if let Ok(options) = serde_json::from_value::<Options>(params.settings) {
                options
            } else {
                // Fallback if some client didn't took changed configuration in params of `workspace/configuration`
                let Some(options) = self
                    .client
                    .configuration(vec![ConfigurationItem {
                        scope_uri: None,
                        section: Some("oxc_language_server".into()),
                    }])
                    .await
                    .ok()
                    .and_then(|mut config| config.first_mut().map(serde_json::Value::take))
                    .and_then(|value| serde_json::from_value::<Options>(value).ok())
                else {
                    error!("Can't fetch `oxc_language_server` configuration");
                    return;
                };
                options
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

    async fn initialized(&self, _params: InitializedParams) {
        debug!("oxc initialized.");
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
        let uri = params.text_document.uri;
        if self.is_ignored(&uri).await {
            return;
        }
        self.handle_file_update(uri, None, None).await;
    }

    /// When the document changed, it may not be written to disk, so we should
    /// get the file context from the language client
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let run_level = { self.options.lock().await.get_lint_level() };
        if run_level < SyntheticRunLevel::OnType {
            return;
        }

        let uri = &params.text_document.uri;
        if self.is_ignored(uri).await {
            return;
        }
        let content = params.content_changes.first().map(|c| c.text.clone());
        self.handle_file_update(
            params.text_document.uri,
            content,
            Some(params.text_document.version),
        )
        .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let run_level = { self.options.lock().await.get_lint_level() };
        if run_level < SyntheticRunLevel::OnType {
            return;
        }
        if self.is_ignored(&params.text_document.uri).await {
            return;
        }
        self.handle_file_update(params.text_document.uri, None, Some(params.text_document.version))
            .await;
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
        })?;

        Ok(())
    }

    async fn init_ignore_glob(&self) {
        let uri = self
            .root_uri
            .get()
            .expect("The root uri should be initialized already")
            .as_ref()
            .expect("should get uri");
        let mut builder = globset::GlobSetBuilder::new();
        // Collecting all ignore files
        builder.add(Glob::new("**/.eslintignore").unwrap());
        builder.add(Glob::new("**/.gitignore").unwrap());

        let ignore_file_glob_set = builder.build().unwrap();

        let mut gitignore_builder = ignore::gitignore::GitignoreBuilder::new(uri.path());
        let walk = ignore::WalkBuilder::new(uri.path())
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .build();
        for entry in walk.flatten() {
            if ignore_file_glob_set.is_match(entry.path()) {
                gitignore_builder.add(entry.path());
            }
        }

        *self.gitignore_glob.lock().await = gitignore_builder.build().ok();
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

    async fn init_linter_config(&self) {
        let Some(Some(uri)) = self.root_uri.get() else {
            return;
        };
        let Ok(root_path) = uri.to_file_path() else {
            return;
        };
        let mut config_path = None;
        let config = root_path.join(self.options.lock().await.get_config_path().unwrap());
        if config.exists() {
            config_path = Some(config);
        }
        if let Some(config_path) = config_path {
            let mut linter = self.server_linter.write().await;
            *linter = ServerLinter::new_with_linter(
                Linter::from_options(
                    LintOptions::default().with_fix(true).with_config_path(Some(config_path)),
                )
                .expect("should have initialized linter with new options"),
            );
        }
    }

    async fn handle_file_update(&self, uri: Url, content: Option<String>, version: Option<i32>) {
        if let Some(Some(_root_uri)) = self.root_uri.get() {
            if let Some(diagnostics) = self.server_linter.read().await.run_single(&uri, content) {
                self.client
                    .publish_diagnostics(
                        uri.clone(),
                        diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                        version,
                    )
                    .await;

                self.diagnostics_report_map.insert(uri.to_string(), diagnostics);
            }
        }
    }

    async fn is_ignored(&self, uri: &Url) -> bool {
        let Some(Some(root_uri)) = self.root_uri.get() else {
            return false;
        };

        // The file is not under current workspace
        if !uri.path().starts_with(root_uri.path()) {
            return false;
        }
        let Some(ref gitignore_globs) = *self.gitignore_glob.lock().await else {
            return false;
        };
        let path = PathBuf::from(uri.path());
        let ignored = gitignore_globs.matched_path_or_any_parents(&path, path.is_dir()).is_ignore();
        if ignored {
            debug!("ignored: {uri}");
        }
        ignored
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
        server_linter: RwLock::new(server_linter),
        diagnostics_report_map,
        options: Mutex::new(Options::default()),
        gitignore_glob: Mutex::new(None),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
