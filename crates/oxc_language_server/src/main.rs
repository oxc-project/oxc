mod linter;

use std::{fmt::Debug, path::PathBuf, str::FromStr};

use dashmap::DashMap;
use futures::future::join_all;
use globset::Glob;
use ignore::gitignore::Gitignore;
use log::{debug, error, info};
use oxc_linter::{FixKind, LinterBuilder, Oxlintrc};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, OnceCell, RwLock, SetError};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, ConfigurationItem, Diagnostic,
        DiagnosticOptions, DiagnosticServerCapabilities, DidChangeConfigurationParams,
        DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, DocumentDiagnosticParams, DocumentDiagnosticReport,
        DocumentDiagnosticReportResult, FullDocumentDiagnosticReport, InitializeParams,
        InitializeResult, InitializedParams, OneOf, RelatedFullDocumentDiagnosticReport,
        ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
        Url, WorkDoneProgressOptions, WorkspaceEdit, WorkspaceFoldersServerCapabilities,
        WorkspaceServerCapabilities,
    },
    Client, LanguageServer, LspService, Server,
};

use crate::linter::{DiagnosticReport, ServerLinter};

struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Url>>,
    server_linter: RwLock<ServerLinter>,
    document_content_cache: DashMap<String, String>,
    diagnostics_report_map: DashMap<String, Vec<DiagnosticReport>>,
    options: Mutex<Options>,
    gitignore_glob: Mutex<Vec<Gitignore>>,
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
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        if let Some(value) = options {
            info!("initialize: {:?}", value);
            info!("language server version: {:?}", env!("CARGO_PKG_VERSION"));
            *self.options.lock().await = value;
        }
        self.init_linter_config().await;
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
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("oxc".into()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
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
            let opened_files = self.document_content_cache.iter().map(|k| k.key().to_string());
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
            self.diagnostics_report_map.clear();
        }
        *self.options.lock().await = changed_options;
    }

    async fn did_change_watched_files(&self, _params: DidChangeWatchedFilesParams) {
        debug!("watched file did change");
        self.init_linter_config().await;
        self.revalidate_open_files().await;
    }

    async fn initialized(&self, _params: InitializedParams) {
        debug!("oxc initialized.");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.document_content_cache
            .insert(params.text_document.uri.to_string(), params.text_document.text);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_content_cache.remove(&params.text_document.uri.to_string());
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let content = params.content_changes.first().map(|c| c.text.clone());

        if let Some(content) = content {
            self.document_content_cache.insert(params.text_document.uri.to_string(), content);
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        // the file is ignored, return empty result
        if self.is_ignored(&params.text_document.uri).await {
            return Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport::default(),
                },
            )));
        }

        let content = self
            .document_content_cache
            .get(&params.text_document.uri.to_string())
            .map(|entry| entry.value().to_owned());

        let Some(result) = self.lint_uri(&params.text_document.uri, content).await else {
            return Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport::default(),
                },
            )));
        };

        Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    items: result.into_iter().map(|report| report.diagnostic).collect(),
                    ..FullDocumentDiagnosticReport::default()
                },
            },
        )))
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
                        #[expect(clippy::disallowed_types)]
                        changes: Some(std::collections::HashMap::from([(
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

        let walk = ignore::WalkBuilder::new(uri.path())
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .build()
            .flatten();

        let mut gitignore_globs = self.gitignore_glob.lock().await;
        for entry in walk {
            let ignore_file_path = entry.path();
            if !ignore_file_glob_set.is_match(ignore_file_path) {
                continue;
            }

            if let Some(ignore_file_dir) = ignore_file_path.parent() {
                let mut builder = ignore::gitignore::GitignoreBuilder::new(ignore_file_dir);
                builder.add(ignore_file_path);
                if let Ok(gitignore) = builder.build() {
                    gitignore_globs.push(gitignore);
                }
            }
        }
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

    async fn revalidate_open_files(&self) {
        join_all(self.document_content_cache.iter().map(|map| {
            self.lint_file_and_publish_diagnostic(
                Url::from_str(map.key()).unwrap(),
                Some(map.value().clone()),
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
                LinterBuilder::from_oxlintrc(
                    true,
                    Oxlintrc::from_file(&config_path)
                        .expect("should have initialized linter with new options"),
                )
                .with_fix(FixKind::SafeFix)
                .build(),
            );
        }
    }

    async fn lint_uri(&self, uri: &Url, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        let Some(Some(_root_uri)) = self.root_uri.get() else {
            return None;
        };

        let result = self.server_linter.read().await.run_single(&uri, content);

        if result.is_some() {
            self.diagnostics_report_map.insert(uri.to_string(), result.clone().unwrap());
        }

        result
    }
    async fn lint_file_and_publish_diagnostic(
        &self,
        uri: Url,
        content: Option<String>,
        version: Option<i32>,
    ) {
        if let Some(diagnostics) = self.lint_uri(&uri, content).await {
            self.client
                .publish_diagnostics(
                    uri.clone(),
                    diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                    version,
                )
                .await;
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
        let gitignore_globs = &(*self.gitignore_glob.lock().await);
        for gitignore in gitignore_globs {
            if let Ok(uri_path) = uri.to_file_path() {
                if !uri_path.starts_with(gitignore.path()) {
                    continue;
                }

                let path = PathBuf::from(uri.path());
                let ignored =
                    gitignore.matched_path_or_any_parents(&path, path.is_dir()).is_ignore();
                if ignored {
                    debug!("ignored: {uri}");
                    return true;
                }
            }
        }
        false
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let server_linter = ServerLinter::new();
    let diagnostics_report_map = DashMap::new();
    let document_content_cache = DashMap::new();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        root_uri: OnceCell::new(),
        server_linter: RwLock::new(server_linter),
        document_content_cache,
        diagnostics_report_map,
        options: Mutex::new(Options::default()),
        gitignore_glob: Mutex::new(vec![]),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
