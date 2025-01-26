use std::{fmt::Debug, path::PathBuf, str::FromStr};

use dashmap::DashMap;
use futures::future::join_all;
use globset::Glob;
use ignore::gitignore::Gitignore;
use log::{debug, error, info};
use rustc_hash::FxBuildHasher;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, OnceCell, RwLock, SetError};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
        ConfigurationItem, Diagnostic, DidChangeConfigurationParams, DidChangeTextDocumentParams,
        DidChangeWatchedFilesParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
        NumberOrString, Position, Range, ServerInfo, TextEdit, Url, WorkspaceEdit,
    },
    Client, LanguageServer, LspService, Server,
};

use oxc_linter::{ConfigStoreBuilder, FixKind, LintOptions, Linter, Oxlintrc};

use crate::capabilities::{Capabilities, CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC};
use crate::linter::error_with_position::DiagnosticReport;
use crate::linter::server_linter::ServerLinter;

mod capabilities;
mod linter;

type FxDashMap<K, V> = DashMap<K, V, FxBuildHasher>;

struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Url>>,
    server_linter: RwLock<ServerLinter>,
    diagnostics_report_map: FxDashMap<String, Vec<DiagnosticReport>>,
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
        Self { enable: true, run: Run::default(), config_path: ".oxlintrc.json".into() }
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
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        if let Some(value) = options {
            info!("initialize: {:?}", value);
            info!("language server version: {:?}", env!("CARGO_PKG_VERSION"));
            *self.options.lock().await = value;
        }

        let oxlintrc = self.init_linter_config().await;
        self.init_ignore_glob(oxlintrc).await;
        Ok(InitializeResult {
            server_info: Some(ServerInfo { name: "oxc".into(), version: None }),
            offset_encoding: None,
            capabilities: Capabilities::from(params.capabilities).into(),
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

        let current_option = &self.options.lock().await.clone();

        debug!(
            "
        configuration changed:
        incoming: {changed_options:?}
        current: {current_option:?}
        "
        );

        if current_option.get_lint_level() != changed_options.get_lint_level()
            && changed_options.get_lint_level() == SyntheticRunLevel::Disable
        {
            debug!("lint level change detected {:?}", &changed_options.get_lint_level());
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

        *self.options.lock().await = changed_options.clone();

        // revalidate the config and all open files, when lint level is not disabled and the config path is changed
        if changed_options.get_lint_level() != SyntheticRunLevel::Disable
            && changed_options
                .get_config_path()
                .is_some_and(|path| path.to_str().unwrap() != current_option.config_path)
        {
            info!("config path change detected {:?}", &changed_options.get_config_path());
            self.init_linter_config().await;
            self.revalidate_open_files().await;
        }
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
        if run_level <= SyntheticRunLevel::Disable {
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
        let is_source_fix_all_oxc = params
            .context
            .only
            .is_some_and(|only| only.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));

        let mut code_actions_vec: Vec<CodeActionOrCommand> = vec![];
        if let Some(value) = self.diagnostics_report_map.get(&uri.to_string()) {
            let reports = value
                .iter()
                .filter(|r| {
                    r.diagnostic.range == params.range
                        || range_includes(params.range, r.diagnostic.range)
                })
                .collect::<Vec<_>>();
            for report in reports {
                // TODO: Would be better if we had exact rule name from the diagnostic instead of having to parse it.
                let mut rule_name: Option<String> = None;
                if let Some(NumberOrString::String(code)) = &report.diagnostic.code {
                    let open_paren = code.chars().position(|c| c == '(');
                    let close_paren = code.chars().position(|c| c == ')');
                    if open_paren.is_some() && close_paren.is_some() {
                        rule_name =
                            Some(code[(open_paren.unwrap() + 1)..close_paren.unwrap()].to_string());
                    }
                }

                if let Some(fixed_content) = &report.fixed_content {
                    code_actions_vec.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: report.diagnostic.message.split(':').next().map_or_else(
                            || "Fix this problem".into(),
                            |s| format!("Fix this {s} problem"),
                        ),
                        kind: Some(if is_source_fix_all_oxc {
                            CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC
                        } else {
                            CodeActionKind::QUICKFIX
                        }),
                        is_preferred: Some(true),
                        edit: Some(WorkspaceEdit {
                            #[expect(clippy::disallowed_types)]
                            changes: Some(std::collections::HashMap::from([(
                                uri.clone(),
                                vec![TextEdit {
                                    range: fixed_content.range,
                                    new_text: fixed_content.code.clone(),
                                }],
                            )])),
                            ..WorkspaceEdit::default()
                        }),
                        disabled: None,
                        data: None,
                        diagnostics: None,
                        command: None,
                    }));
                }

                code_actions_vec.push(
                    // TODO: This CodeAction doesn't support disabling multiple rules by name for a given line.
                    //  To do that, we need to read `report.diagnostic.range.start.line` and check if a disable comment already exists.
                    //  If it does, it needs to be appended to instead of a completely new line inserted.
                    CodeActionOrCommand::CodeAction(CodeAction {
                        title: rule_name.clone().map_or_else(
                            || "Disable oxlint for this line".into(),
                            |s| format!("Disable {s} for this line"),
                        ),
                        kind: Some(CodeActionKind::QUICKFIX),
                        is_preferred: Some(false),
                        edit: Some(WorkspaceEdit {
                            #[expect(clippy::disallowed_types)]
                            changes: Some(std::collections::HashMap::from([(
                                uri.clone(),
                                vec![TextEdit {
                                    range: Range {
                                        start: Position {
                                            line: report.diagnostic.range.start.line,
                                            // TODO: character should be set to match the first non-whitespace character in the source text to match the existing indentation.
                                            character: 0,
                                        },
                                        end: Position {
                                            line: report.diagnostic.range.start.line,
                                            // TODO: character should be set to match the first non-whitespace character in the source text to match the existing indentation.
                                            character: 0,
                                        },
                                    },
                                    new_text: rule_name.clone().map_or_else(
                                        || "// eslint-disable-next-line\n".into(),
                                        |s| format!("// eslint-disable-next-line {s}\n"),
                                    ),
                                }],
                            )])),
                            ..WorkspaceEdit::default()
                        }),
                        disabled: None,
                        data: None,
                        diagnostics: None,
                        command: None,
                    }),
                );

                code_actions_vec.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: rule_name.clone().map_or_else(
                        || "Disable oxlint for this file".into(),
                        |s| format!("Disable {s} for this file"),
                    ),
                    kind: Some(CodeActionKind::QUICKFIX),
                    is_preferred: Some(false),
                    edit: Some(WorkspaceEdit {
                        #[expect(clippy::disallowed_types)]
                        changes: Some(std::collections::HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position { line: 0, character: 0 },
                                    end: Position { line: 0, character: 0 },
                                },
                                new_text: rule_name.clone().map_or_else(
                                    || "// eslint-disable\n".into(),
                                    |s| format!("// eslint-disable {s}\n"),
                                ),
                            }],
                        )])),
                        ..WorkspaceEdit::default()
                    }),
                    disabled: None,
                    data: None,
                    diagnostics: None,
                    command: None,
                }));
            }
        }

        if code_actions_vec.is_empty() {
            return Ok(None);
        }

        Ok(Some(code_actions_vec))
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

    async fn init_ignore_glob(&self, oxlintrc: Option<Oxlintrc>) {
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

        if let Some(oxlintrc) = oxlintrc {
            if !oxlintrc.ignore_patterns.is_empty() {
                let mut builder =
                    ignore::gitignore::GitignoreBuilder::new(oxlintrc.path.parent().unwrap());
                for entry in &oxlintrc.ignore_patterns {
                    builder.add_line(None, entry).expect("Failed to add ignore line");
                }
                gitignore_globs.push(builder.build().unwrap());
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
        join_all(self.diagnostics_report_map.iter().map(|map| {
            let url = Url::from_str(map.key()).expect("should convert to path");

            self.handle_file_update(url, None, None)
        }))
        .await;
    }

    async fn init_linter_config(&self) -> Option<Oxlintrc> {
        let Some(Some(uri)) = self.root_uri.get() else {
            return None;
        };
        let Ok(root_path) = uri.to_file_path() else {
            return None;
        };
        let mut config_path = None;
        let config = root_path.join(self.options.lock().await.get_config_path().unwrap());
        if config.exists() {
            config_path = Some(config);
        }
        if let Some(config_path) = config_path {
            let mut linter = self.server_linter.write().await;
            let config = Oxlintrc::from_file(&config_path)
                .expect("should have initialized linter with new options");
            let config_store = ConfigStoreBuilder::from_oxlintrc(true, config.clone())
                .build()
                .expect("failed to build config");
            *linter = ServerLinter::new_with_linter(
                Linter::new(LintOptions::default(), config_store).with_fix(FixKind::SafeFix),
            );
            return Some(config);
        }

        None
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
        let gitignore_globs = &(*self.gitignore_glob.lock().await);
        for gitignore in gitignore_globs {
            if let Ok(uri_path) = uri.to_file_path() {
                if !uri_path.starts_with(gitignore.path()) {
                    continue;
                }
                if gitignore.matched_path_or_any_parents(&uri_path, uri_path.is_dir()).is_ignore() {
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
    let diagnostics_report_map = FxDashMap::default();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        root_uri: OnceCell::new(),
        server_linter: RwLock::new(server_linter),
        diagnostics_report_map,
        options: Mutex::new(Options::default()),
        gitignore_glob: Mutex::new(vec![]),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

fn range_includes(range: Range, to_include: Range) -> bool {
    if range.start >= to_include.start {
        return false;
    }
    if range.end <= to_include.end {
        return false;
    }
    true
}
