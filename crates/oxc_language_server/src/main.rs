use code_actions::{
    CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC, apply_all_fix_code_action, apply_fix_code_action,
    ignore_this_line_code_action, ignore_this_rule_code_action,
};
use commands::LSP_COMMANDS;
use futures::future::join_all;
use globset::Glob;
use ignore::gitignore::Gitignore;
use linter::{config_walker::ConfigWalker, isolated_lint_handler::IsolatedLintHandlerOptions};
use log::{debug, error, info};
use oxc_linter::{ConfigStore, ConfigStoreBuilder, FixKind, LintOptions, Linter, Oxlintrc};
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::sync::{Mutex, OnceCell, RwLock, SetError};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server, UriExt,
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        CodeActionOrCommand, CodeActionParams, CodeActionResponse, ConfigurationItem, Diagnostic,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        ExecuteCommandParams, FileChangeType, InitializeParams, InitializeResult,
        InitializedParams, Range, ServerInfo, Uri,
    },
};

use crate::capabilities::Capabilities;
use crate::linter::error_with_position::DiagnosticReport;
use crate::linter::server_linter::ServerLinter;

mod capabilities;
mod code_actions;
mod commands;
mod linter;

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const OXC_CONFIG_FILE: &str = ".oxlintrc.json";

struct Backend {
    client: Client,
    root_uri: OnceCell<Option<Uri>>,
    server_linter: RwLock<ServerLinter>,
    diagnostics_report_map: ConcurrentHashMap<String, Vec<DiagnosticReport>>,
    options: Mutex<Options>,
    gitignore_glob: Mutex<Vec<Gitignore>>,
    nested_configs: ConcurrentHashMap<PathBuf, ConfigStore>,
}
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum Run {
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
        self.init(params.root_uri)?;
        let options = params.initialization_options.and_then(|mut value| {
            let settings = value.get_mut("settings")?.take();
            serde_json::from_value::<Options>(settings).ok()
        });

        if let Some(value) = options {
            info!("initialize: {value:?}");
            info!("language server version: {:?}", env!("CARGO_PKG_VERSION"));
            *self.options.lock().await = value;
        }

        self.init_nested_configs().await;
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

        *self.options.lock().await = changed_options.clone();

        if changed_options.use_nested_configs() != current_option.use_nested_configs() {
            self.nested_configs.pin().clear();
            self.init_nested_configs().await;
        }

        if Self::needs_linter_restart(current_option, &changed_options) {
            self.init_linter_config().await;
            self.revalidate_open_files().await;
        }
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        debug!("watched file did change");
        if self.options.lock().await.use_nested_configs() {
            let nested_configs = self.nested_configs.pin();

            params.changes.iter().for_each(|x| {
                let Some(file_path) = x.uri.to_file_path() else {
                    info!("Unable to convert {:?} to a file path", x.uri);
                    return;
                };
                let Some(file_name) = file_path.file_name() else {
                    info!("Unable to retrieve file name from {file_path:?}");
                    return;
                };

                if file_name != OXC_CONFIG_FILE {
                    return;
                }

                let Some(dir_path) = file_path.parent() else {
                    info!("Unable to retrieve parent from {file_path:?}");
                    return;
                };

                // spellchecker:off -- "typ" is accurate
                if x.typ == FileChangeType::CREATED || x.typ == FileChangeType::CHANGED {
                    // spellchecker:on
                    let oxlintrc =
                        Oxlintrc::from_file(&file_path).expect("Failed to parse config file");
                    let config_store_builder = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc)
                        .expect("Failed to create config store builder");
                    let config_store =
                        config_store_builder.build().expect("Failed to build config store");
                    nested_configs.insert(dir_path.to_path_buf(), config_store);
                // spellchecker:off -- "typ" is accurate
                } else if x.typ == FileChangeType::DELETED {
                    // spellchecker:on
                    nested_configs.remove(&dir_path.to_path_buf());
                }
            });
        }

        self.init_linter_config().await;
        self.revalidate_open_files().await;
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
        // drop as fast as possible
        let run_level = { self.options.lock().await.run };
        if run_level != Run::OnSave {
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
        let run_level = { self.options.lock().await.run };
        if run_level != Run::OnType {
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
        if self.is_ignored(&params.text_document.uri).await {
            return;
        }
        self.handle_file_update(params.text_document.uri, None, Some(params.text_document.version))
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.diagnostics_report_map.pin().remove(&uri);
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let report_map = self.diagnostics_report_map.pin();
        let Some(value) = report_map.get(&uri.to_string()) else {
            return Ok(None);
        };

        let reports = value.iter().filter(|r| {
            r.diagnostic.range == params.range || range_overlaps(params.range, r.diagnostic.range)
        });

        let is_source_fix_all_oxc = params
            .context
            .only
            .is_some_and(|only| only.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));

        if is_source_fix_all_oxc {
            return Ok(apply_all_fix_code_action(reports, uri)
                .map(|code_actions| vec![CodeActionOrCommand::CodeAction(code_actions)]));
        }

        let mut code_actions_vec: Vec<CodeActionOrCommand> = vec![];

        for report in reports {
            if let Some(fix_action) = apply_fix_code_action(report, uri) {
                code_actions_vec.push(CodeActionOrCommand::CodeAction(fix_action));
            }

            code_actions_vec
                .push(CodeActionOrCommand::CodeAction(ignore_this_line_code_action(report, uri)));

            code_actions_vec
                .push(CodeActionOrCommand::CodeAction(ignore_this_rule_code_action(report, uri)));
        }

        if code_actions_vec.is_empty() {
            return Ok(None);
        }

        Ok(Some(code_actions_vec))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        let command = LSP_COMMANDS.iter().find(|c| c.command_id() == params.command);
        match command {
            Some(c) => c.execute(self, params.arguments).await,
            None => Err(Error::invalid_request()),
        }
    }
}

impl Backend {
    fn init(&self, root_uri: Option<Uri>) -> Result<()> {
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

        let walk = ignore::WalkBuilder::new(uri.to_file_path().unwrap())
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

    async fn clear_all_diagnostics(&self) {
        let cleared_diagnostics = self
            .diagnostics_report_map
            .pin()
            .keys()
            .map(|uri| (uri.clone(), vec![]))
            .collect::<Vec<_>>();
        self.publish_all_diagnostics(&cleared_diagnostics).await;
    }

    #[expect(clippy::ptr_arg)]
    async fn publish_all_diagnostics(&self, result: &Vec<(String, Vec<Diagnostic>)>) {
        join_all(result.iter().map(|(path, diagnostics)| {
            self.client.publish_diagnostics(Uri::from_str(path).unwrap(), diagnostics.clone(), None)
        }))
        .await;
    }

    async fn revalidate_open_files(&self) {
        join_all(self.diagnostics_report_map.pin_owned().keys().map(|key| {
            let url = Uri::from_str(key).expect("should convert to path");

            self.handle_file_update(url, None, None)
        }))
        .await;
    }

    fn needs_linter_restart(old_options: &Options, new_options: &Options) -> bool {
        old_options.config_path != new_options.config_path
            || old_options.use_nested_configs() != new_options.use_nested_configs()
            || old_options.fix_kind() != new_options.fix_kind()
    }

    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    async fn init_nested_configs(&self) {
        let Some(Some(uri)) = self.root_uri.get() else {
            return;
        };
        let Some(root_path) = uri.to_file_path() else {
            return;
        };

        // nested config is disabled, no need to search for configs
        if !self.options.lock().await.use_nested_configs() {
            return;
        }

        let paths = ConfigWalker::new(&root_path).paths();
        let nested_configs = self.nested_configs.pin();

        for path in paths {
            let file_path = Path::new(&path);
            let Some(dir_path) = file_path.parent() else {
                continue;
            };

            let oxlintrc = Oxlintrc::from_file(file_path).expect("Failed to parse config file");
            let config_store_builder = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc)
                .expect("Failed to create config store builder");
            let config_store = config_store_builder.build().expect("Failed to build config store");
            nested_configs.insert(dir_path.to_path_buf(), config_store);
        }
    }

    async fn init_linter_config(&self) -> Option<Oxlintrc> {
        let Some(Some(uri)) = self.root_uri.get() else {
            return None;
        };
        let root_path = uri.to_file_path()?;
        let relative_config_path = self.options.lock().await.config_path.clone();
        let oxlintrc = if relative_config_path.is_some() {
            let config = root_path.join(relative_config_path.unwrap());
            if config.try_exists().expect("Could not get fs metadata for config") {
                if let Ok(oxlintrc) = Oxlintrc::from_file(&config) {
                    oxlintrc
                } else {
                    error!("Failed to initialize oxlintrc config: {}", config.to_string_lossy());
                    Oxlintrc::default()
                }
            } else {
                error!(
                    "Config file not found: {}, fallback to default config",
                    config.to_string_lossy()
                );
                Oxlintrc::default()
            }
        } else {
            Oxlintrc::default()
        };

        // clone because we are returning it for ignore builder
        let config_builder =
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc.clone()).unwrap_or_default();

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = self.options.lock().await.use_nested_configs();

        let use_cross_module = if use_nested_config {
            self.nested_configs.pin().values().any(|config| config.plugins().has_import())
        } else {
            config_builder.plugins().has_import()
        };

        let config_store = config_builder.build().expect("Failed to build config store");

        let lint_options =
            LintOptions { fix: self.options.lock().await.fix_kind(), ..Default::default() };

        let linter = if use_nested_config {
            let nested_configs = self.nested_configs.pin();
            let nested_configs_copy: FxHashMap<PathBuf, ConfigStore> = nested_configs
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<FxHashMap<_, _>>();

            Linter::new_with_nested_configs(lint_options, config_store, nested_configs_copy)
        } else {
            Linter::new(lint_options, config_store)
        };

        *self.server_linter.write().await = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions { use_cross_module, root_path: root_path.to_path_buf() },
        );

        Some(oxlintrc)
    }

    async fn handle_file_update(&self, uri: Uri, content: Option<String>, version: Option<i32>) {
        if let Some(Some(_root_uri)) = self.root_uri.get() {
            let diagnostics = self.server_linter.read().await.run_single(&uri, content);
            if let Some(diagnostics) = diagnostics {
                self.client
                    .publish_diagnostics(
                        uri.clone(),
                        diagnostics.clone().into_iter().map(|d| d.diagnostic).collect(),
                        version,
                    )
                    .await;

                self.diagnostics_report_map.pin().insert(uri.to_string(), diagnostics);
            }
        }
    }

    async fn is_ignored(&self, uri: &Uri) -> bool {
        let Some(Some(root_uri)) = self.root_uri.get() else {
            return false;
        };

        // The file is not under current workspace
        if !uri.to_file_path().unwrap().starts_with(root_uri.to_file_path().unwrap()) {
            return false;
        }
        let gitignore_globs = &(*self.gitignore_glob.lock().await);
        for gitignore in gitignore_globs {
            if let Some(uri_path) = uri.to_file_path() {
                if !uri_path.starts_with(gitignore.path()) {
                    continue;
                }
                if gitignore.matched_path_or_any_parents(&uri_path, uri_path.is_dir()).is_ignore() {
                    debug!("ignored: {uri:?}");
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

    let server_linter = ServerLinter::new(IsolatedLintHandlerOptions {
        use_cross_module: false,
        root_path: PathBuf::new(),
    });
    let diagnostics_report_map = ConcurrentHashMap::default();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        root_uri: OnceCell::new(),
        server_linter: RwLock::new(server_linter),
        diagnostics_report_map,
        options: Mutex::new(Options::default()),
        gitignore_glob: Mutex::new(vec![]),
        nested_configs: ConcurrentHashMap::default(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

fn range_overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && a.end >= b.start
}
