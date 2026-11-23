use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};

use ignore::gitignore::Gitignore;
use rustc_hash::{FxHashMap, FxHashSet};
use tower_lsp_server::{
    gen_lsp_types::{
        CodeActionKind, CodeActionOptions, CodeActionProvider, CodeActionResponse,
        CodeActionTriggerKind, Diagnostic, DiagnosticOptions, DiagnosticProvider,
        ExecuteCommandOptions, FileChangeType, FileEvent, MessageType, Pattern, ServerCapabilities,
        Uri, WorkDoneProgressOptions, WorkspaceEdit,
    },
    jsonrpc::ErrorCode,
};
use tracing::{debug, error, warn};

use oxc_linter::{
    AllowWarnDeny, Config, ConfigStore, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore,
    FileChangeKind, FixKind, LINTABLE_EXTENSIONS, LintIgnoreMatcher, LintOptions, LintRunner,
    LintRunnerBuilder, LintServiceOptions, Linter, Oxlintrc, TsrsLintServer,
    is_tsrs_lint_executable, read_to_string, try_find_tsgolint_executable,
};

use oxc_language_server::{
    Capabilities, ClientMessage, CodeActionParams, ConcurrentHashMap, DiagnosticMode,
    DiagnosticResult, LineIndex, TextDocument, Tool, ToolBuildResult, ToolBuilder,
    ToolRestartChanges, uri_utils::uri_to_file_path,
    utils::normalize_user_config_path_to_watch_pattern,
};

use crate::lsp::utils::{config_loader_error_to_message, oxc_diagnostic_to_client_message};
use crate::{
    config_loader::{
        ConfigLoader, build_nested_configs, config_file_names, discover_configs_in_tree,
        materialize_default_plugins,
    },
    lsp::{
        code_actions::{
            CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC, CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
            apply_all_fix_code_action, apply_dangerous_fix_code_action, apply_fix_code_actions,
            fix_all_text_edit,
        },
        commands::{FIX_ALL_COMMAND_ID, FixAllCommandArgs},
        error_with_position::{
            DiagnosticReport, LinterCodeAction, create_unused_directives_report,
            generate_inverted_diagnostics, message_to_lsp_diagnostic,
        },
        lsp_file_system::LspFileSystem,
        options::{
            LintOptions as LSPLintOptions, RulesCustomization, Run, UnusedDisableDirectives,
        },
        utils::range_overlaps,
    },
    utils::normalize_path,
};

#[derive(Default)]
pub struct ServerLinterBuilder {
    external_linter: Option<ExternalLinter>,
    #[cfg(feature = "napi")]
    js_config_loader: Option<crate::js_config::JsConfigLoaderCb>,
    /// The state each workspace keeps across linter rebuilds, by workspace root.
    workspaces: Mutex<FxHashMap<Uri, Arc<WorkspaceState>>>,
}

/// What a workspace keeps across linter rebuilds (a configuration change rebuilds the linter).
#[derive(Default)]
struct WorkspaceState {
    /// The open documents with their unsaved text, by URI.
    open_documents: Mutex<FxHashMap<Uri, OpenDocument>>,
    /// The workspace's long-lived `tsrs-lint server`. Once started, it is kept for the language
    /// server's lifetime.
    tsrs_server: Mutex<Option<Arc<TsrsLintServer>>>,
}

#[derive(Clone)]
struct OpenDocument {
    path: PathBuf,
    version: i32,
    text: Arc<str>,
}

impl WorkspaceState {
    fn server(&self) -> Option<Arc<TsrsLintServer>> {
        self.tsrs_server.lock().expect("tsrs server mutex poisoned").clone()
    }

    /// The `tsrs-lint server` type-aware linting uses in this workspace, when the type-aware
    /// executable found from `root_path` is `tsrs-lint`. A new server is sent the open documents.
    ///
    /// A server that was disabled is replaced, so a configuration change tries again. The
    /// previous server is stopped when it is replaced, or when there is no `tsrs-lint` anymore.
    fn tsrs_server_for(&self, root_path: &Path) -> Option<Arc<TsrsLintServer>> {
        let Some(executable) = try_find_tsgolint_executable(root_path)
            .ok()
            .filter(|path| is_tsrs_lint_executable(path))
        else {
            self.stop_tsrs_server();
            return None;
        };
        let mut slot = self.tsrs_server.lock().expect("tsrs server mutex poisoned");
        if let Some(server) = slot.as_ref().filter(|server| {
            server.executable_path() == executable.as_path() && !server.is_disabled()
        }) {
            return Some(Arc::clone(server));
        }
        if let Some(previous) = slot.take() {
            previous.stop();
        }
        let server = Arc::new(TsrsLintServer::new(executable, root_path.to_path_buf()));
        for document in self.open_documents.lock().expect("open documents mutex poisoned").values()
        {
            server.did_open(&document.path, document.version, Arc::clone(&document.text));
        }
        *slot = Some(Arc::clone(&server));
        Some(server)
    }

    /// Stops the workspace's `tsrs-lint server`, if it has one.
    fn stop_tsrs_server(&self) {
        let server = self.tsrs_server.lock().expect("tsrs server mutex poisoned").take();
        if let Some(server) = server {
            server.stop();
        }
    }
}

impl ServerLinterBuilder {
    pub fn new(
        external_linter: Option<ExternalLinter>,
        #[cfg(feature = "napi")] js_config_loader: Option<crate::js_config::JsConfigLoaderCb>,
    ) -> Self {
        Self {
            external_linter,
            #[cfg(feature = "napi")]
            js_config_loader,
            workspaces: Mutex::default(),
        }
    }

    fn workspace(&self, root_uri: &Uri) -> Arc<WorkspaceState> {
        Arc::clone(
            self.workspaces
                .lock()
                .expect("workspaces mutex poisoned")
                .entry(root_uri.clone())
                .or_default(),
        )
    }

    /// Creates a new `ServerLinter` instance based on the provided root URI and options.
    /// Returns a tuple containing the `ServerLinter` instance and optional messages to be sent to the client.
    /// These messages will be used to inform about misconfiguration.
    ///
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(
        &self,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> (ServerLinter, Vec<ClientMessage>) {
        let options = match serde_json::from_value::<LSPLintOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };
        let root_path = uri_to_file_path(root_uri).unwrap();
        let mut external_linter = self.external_linter.as_ref();
        let mut external_plugin_store = ExternalPluginStore::new(external_linter.is_some());
        // create a set for config errors, because if the main config fails to load,
        // the same error might occur again.
        let mut client_messages = FxHashSet::default();

        // Setup JS workspace. This must be done before loading any configs
        if let Some(external_linter) = external_linter {
            let res = (external_linter.create_workspace)(root_uri.to_string());

            if let Err(err) = res {
                client_messages.insert(ClientMessage {
                    message: format!("Failed to setup JS workspace:\n{err}\n"),
                    r#type: MessageType::Error,
                });
            }
        }

        let config_path = options.config_path.as_ref().map(PathBuf::from);
        let loader = ConfigLoader::new(
            external_linter,
            &mut external_plugin_store,
            &[],
            Some(root_uri.as_ref()),
        );
        #[cfg(feature = "napi")]
        let loader = loader.with_js_config_loader(self.js_config_loader.as_ref());

        let mut oxlintrc = match loader.load_root_config(&root_path, config_path.as_ref()) {
            Ok(config) => config,
            Err(e) => {
                client_messages.insert(oxc_diagnostic_to_client_message(&e));
                Oxlintrc::default()
            }
        };
        materialize_default_plugins(&mut oxlintrc);

        let mut nested_ignore_patterns = Vec::new();
        let mut extended_paths = FxHashSet::default();
        let (nested_configs, config_client_message) = if options.use_nested_configs() {
            self.create_nested_configs(
                &root_path,
                &oxlintrc.path,
                &mut external_plugin_store,
                &mut nested_ignore_patterns,
                &mut extended_paths,
                Some(root_uri.as_ref()),
            )
        } else {
            (FxHashMap::default(), Vec::new())
        };

        client_messages.extend(config_client_message);

        let base_patterns = oxlintrc.ignore_patterns.clone();
        // Without a config file there are no patterns and the root is never consulted,
        // so the `root_path` fallback is an arbitrary placeholder.
        let base_ignore_root = oxlintrc.dir().unwrap_or(&root_path).to_path_buf();

        let config_builder = match ConfigLoader::new(
            external_linter,
            &mut external_plugin_store,
            &[],
            Some(root_uri.as_ref()),
        )
        .create_config_builder(oxlintrc)
        {
            Ok(builder) => builder,
            Err(e) => {
                client_messages.insert(ClientMessage {
                    message: format!("Failed to build config from oxlintrc:\n{e}"),
                    r#type: MessageType::Error,
                });
                ConfigStoreBuilder::default()
            }
        };

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = options.use_nested_configs();
        let fix_kind = FixKind::from(options.fix_kind);

        let use_cross_module = config_builder.plugins().has_import()
            || (use_nested_config
                && nested_configs.values().any(|config| config.plugins().has_import()));

        extended_paths.extend(config_builder.extended_paths.clone());
        let base_config = config_builder.build(&mut external_plugin_store).unwrap_or_else(|err| {
            client_messages.insert(ClientMessage {
                message: format!("Failed to build config:\n{err}"),
                r#type: MessageType::Error,
            });
            ConfigStoreBuilder::empty().build(&mut ExternalPluginStore::new(false)).unwrap()
        });

        if external_plugin_store.is_empty() {
            external_linter = None;
        }
        let config_store = ConfigStore::new(base_config, nested_configs, external_plugin_store);

        let lint_options = LintOptions {
            fix: fix_kind,
            report_unused_directive: match options.unused_disable_directives {
                Some(UnusedDisableDirectives::Allow) => None,
                Some(UnusedDisableDirectives::Warn) => Some(AllowWarnDeny::Warn),
                Some(UnusedDisableDirectives::Deny) => Some(AllowWarnDeny::Deny),
                None => match config_store.report_unused_disable_directives() {
                    Some(severity) if severity.is_warn_deny() => Some(severity),
                    _ => None,
                },
            },
            with_ignore_fixes: true,
            ..Default::default()
        };

        let type_aware = options.type_aware.unwrap_or(config_store.type_aware_enabled());
        let config_store_clone = config_store.clone();
        let workspace = self.workspace(root_uri);
        let tsrs_server = if type_aware {
            workspace.tsrs_server_for(&root_path)
        } else {
            workspace.stop_tsrs_server();
            None
        };

        // Send JS plugins config to JS side
        if let Some(external_linter) = external_linter {
            let res = config_store.external_plugin_store().setup_rule_configs(
                root_path.to_string_lossy().into_owned(),
                Some(root_uri.as_ref()),
                external_linter,
            );
            if let Err(err) = res {
                client_messages.insert(ClientMessage {
                    message: format!("Failed to setup JS plugins config:\n{err}"),
                    r#type: MessageType::Error,
                });
            }
        }

        let linter = Linter::new(lint_options, config_store, external_linter.cloned())
            .with_workspace_uri(Some(root_uri.as_ref()));
        let mut lint_service_options =
            LintServiceOptions::new(root_path.clone()).with_cross_module(use_cross_module);

        if let Some(ts_path) = options.ts_config_path.as_ref() {
            let ts_path = Path::new(ts_path).to_path_buf();
            let ts_path = if ts_path.is_relative() { root_path.join(ts_path) } else { ts_path };
            if ts_path.is_file() {
                lint_service_options = lint_service_options.with_tsconfig(&ts_path);
            }
        }

        let runner = match LintRunnerBuilder::new(lint_service_options.clone(), linter)
            .with_type_aware(type_aware)
            .with_fix_kind(fix_kind)
            .with_ignore_fixes(true)
            .with_tsrs_server(tsrs_server)
            .build()
        {
            Ok(runner) => runner,
            Err(e) => {
                client_messages.insert(ClientMessage {
                    message: format!("Failed to initialize type-aware linting:\n{e}"),
                    r#type: MessageType::Error,
                });
                let linter =
                    Linter::new(lint_options, config_store_clone, external_linter.cloned())
                        .with_workspace_uri(Some(root_uri.as_ref()));
                LintRunnerBuilder::new(lint_service_options, linter)
                    .with_type_aware(false)
                    .with_fix_kind(fix_kind)
                    .with_ignore_fixes(true)
                    .build()
                    .expect("Failed to build LintRunner without type-aware linting")
            }
        };

        (
            ServerLinter::new(
                options.run,
                root_path.to_path_buf(),
                LintIgnoreMatcher::new(&base_patterns, &base_ignore_root, nested_ignore_patterns),
                Self::create_ignore_glob(&root_path),
                extended_paths,
                runner,
                fix_kind,
                lint_options.report_unused_directive,
                options.rules_customization,
                workspace,
            ),
            client_messages.into_iter().collect(),
        )
    }
}

impl ToolBuilder for ServerLinterBuilder {
    fn server_capabilities(
        &self,
        capabilities: &mut ServerCapabilities,
        backend_capabilities: &mut Capabilities,
    ) {
        capabilities.code_action_provider =
            Some(CodeActionProvider::CodeActionOptions(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::QuickFix,
                    CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
                    CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC,
                    CodeActionKind::SourceFixAll,
                ]),
                ..Default::default()
            }));

        capabilities.execute_command_provider = Some(ExecuteCommandOptions {
            commands: vec![FIX_ALL_COMMAND_ID.to_string()],
            work_done_progress_options: WorkDoneProgressOptions::default(),
        });

        // The server supports pull and push diagnostics.
        // Only use push diagnostics if the client does not support pull diagnostics,
        // or we cannot ask the client to refresh diagnostics.
        if !backend_capabilities.pull_diagnostics || !backend_capabilities.refresh_diagnostics {
            backend_capabilities.diagnostic_mode = DiagnosticMode::Push;
        } else {
            backend_capabilities.diagnostic_mode = DiagnosticMode::Pull;
        }

        // tell the client we support pull diagnostics
        capabilities.diagnostic_provider =
            if backend_capabilities.diagnostic_mode == DiagnosticMode::Pull {
                Some(DiagnosticProvider::DiagnosticOptions(DiagnosticOptions::default()))
            } else {
                None
            };
    }

    fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult {
        let (tool, client_messages) = self.build(root_uri, options);
        ToolBuildResult { tool: Box::new(tool), client_messages }
    }

    #[expect(unused)]
    fn shutdown(&self, root_uri: &Uri) {
        // The workspace's `tsrs-lint server` keeps the programs of its projects: stop it with the
        // workspace.
        let workspace = self.workspaces.lock().expect("workspaces mutex poisoned").remove(root_uri);
        if let Some(workspace) = workspace {
            workspace.stop_tsrs_server();
        }

        // We don't currently destroy JS workspaces.
        // See comment in `destroyWorkspace` in `src-js/workspace/index.ts` for explanation.
        return;

        // Destroy JS workspace
        if let Some(external_linter) = &self.external_linter {
            let res = (external_linter.destroy_workspace)(root_uri.to_string());

            if let Err(err) = res {
                error!("Failed to destroy JS workspace:\n{err}\n");
            }
        }
    }
}

impl ServerLinterBuilder {
    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    fn create_nested_configs(
        &self,
        root_path: &Path,
        base_config_path: &Path,
        external_plugin_store: &mut ExternalPluginStore,
        nested_ignore_patterns: &mut Vec<(Vec<String>, PathBuf)>,
        extended_paths: &mut FxHashSet<PathBuf>,
        workspace_uri: Option<&str>,
    ) -> (FxHashMap<PathBuf, Config>, Vec<ClientMessage>) {
        let config_paths = discover_configs_in_tree(root_path, base_config_path);

        #[cfg_attr(not(feature = "napi"), allow(unused_mut))]
        let mut loader = ConfigLoader::new(
            self.external_linter.as_ref(),
            external_plugin_store,
            &[],
            workspace_uri,
        );

        #[cfg(feature = "napi")]
        {
            loader = loader.with_js_config_loader(self.js_config_loader.as_ref());
        }

        let (configs, errors) = loader.load_discovered_with_root_dir(root_path, config_paths);

        let mut client_messages = Vec::with_capacity(errors.len());
        for error in errors {
            // show message to the LSP client
            client_messages.push(ClientMessage {
                message: config_loader_error_to_message(error),
                r#type: MessageType::Error,
            });
        }

        (
            build_nested_configs(configs, nested_ignore_patterns, Some(extended_paths)),
            client_messages,
        )
    }

    #[expect(clippy::filetype_is_file)]
    fn create_ignore_glob(root_path: &Path) -> Vec<Gitignore> {
        let walk = ignore::WalkBuilder::new(root_path)
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .filter_entry(|entry| {
                !(entry.file_name() == ".git"
                    && entry.file_type().is_some_and(|file_type| file_type.is_dir()))
            })
            .build()
            .flatten();

        let mut gitignore_globs = vec![];
        for entry in walk {
            if !entry.file_type().is_some_and(|v| v.is_file()) {
                continue;
            }
            let ignore_file_path = entry.path();
            if !ignore_file_path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|v| [".eslintignore", ".gitignore"].contains(&v))
            {
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

        gitignore_globs
    }
}

pub struct ServerLinter {
    run: Run,
    cwd: PathBuf,
    ignore_matcher: LintIgnoreMatcher,
    gitignore_glob: Vec<Gitignore>,
    extended_paths: FxHashSet<PathBuf>,
    code_actions: Arc<ConcurrentHashMap<Uri, Vec<LinterCodeAction>>>,
    runner: LintRunner,
    fix_kind: FixKind,
    unused_directives_severity: Option<AllowWarnDeny>,
    rules_customization: Option<RulesCustomization>,
    workspace: Arc<WorkspaceState>,
}

impl Tool for ServerLinter {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    fn handle_configuration_change(
        &self,
        builder: &dyn ToolBuilder,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges {
        let old_option = match serde_json::from_value::<LSPLintOptions>(old_options_json.clone()) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };

        let new_options = match serde_json::from_value::<LSPLintOptions>(new_options_json.clone()) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };

        if !Self::needs_restart(&old_option, &new_options) {
            return ToolRestartChanges {
                tool: None,
                watch_patterns: None,
                client_messages: Vec::new(),
            };
        }

        // get the cached files before refreshing the linter, and revalidate them after
        builder.shutdown(root_uri);
        let ToolBuildResult { tool, client_messages } =
            builder.build(root_uri, new_options_json.clone());

        let patterns = {
            if old_option.config_path == new_options.config_path
                && old_option.use_nested_configs() == new_options.use_nested_configs()
                && old_option.type_aware == new_options.type_aware
            {
                None
            } else {
                Some(tool.get_watcher_patterns(new_options_json))
            }
        };

        ToolRestartChanges { tool: Some(tool), watch_patterns: patterns, client_messages }
    }

    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern> {
        let options = match serde_json::from_value::<LSPLintOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };
        let mut watchers = if let Some(config_path) = options.config_path.as_deref() {
            vec![normalize_user_config_path_to_watch_pattern(config_path)]
        } else {
            // Watch subdirectories too only when nested config discovery is on.
            let prefix = if options.use_nested_configs() { "**/" } else { "" };
            config_file_names().into_iter().map(|name| format!("{prefix}{name}")).collect()
        };

        for path in &self.extended_paths {
            // Ignore known config files when nested config discovery handles them.
            if config_file_names().iter().any(|name| path.ends_with(name))
                && options.use_nested_configs()
            {
                continue;
            }

            let pattern = path.strip_prefix(self.cwd.clone()).unwrap_or(path);

            watchers.push(normalize_path(pattern).to_string_lossy().to_string());
        }

        if options.type_aware.unwrap_or(self.runner.has_type_aware()) {
            watchers.push("**/tsconfig*.json".to_string());
            // The `tsrs-lint server` keeps programs between requests, so it needs to know about
            // every change on disk that can affect them.
            if self.runner.has_tsrs_server() {
                watchers.push("**/*.{ts,tsx,mts,cts,js,jsx,mjs,cjs,json}".to_string());
                watchers.push("**/package.json".to_string());
            }
        }

        watchers
    }

    /// With a `tsrs-lint server`, only changes to lint configuration files restart the linter.
    /// Other changes (sources, `package.json`, `tsconfig.json`) are sent to the server, so its
    /// programs match the disk as a one-shot run's would. Open files are not linted again.
    fn handle_watched_file_event(
        &self,
        builder: &dyn ToolBuilder,
        file_event: &FileEvent,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        if let Some(server) = self.workspace.server().filter(|_| self.runner.has_tsrs_server())
            && let Some(path) = uri_to_file_path(&file_event.uri)
            && !self.is_lint_config_file(&path, &options)
        {
            // Changes outside the workspace are sent too: its programs can depend on files above
            // the root, such as an extended `tsconfig.json`.
            let kind = match file_event.kind {
                FileChangeType::Created => FileChangeKind::Created,
                FileChangeType::Deleted => FileChangeKind::Deleted,
                _ => FileChangeKind::Changed,
            };
            server.files_changed(&[(path.into_owned(), kind)]);
            return ToolRestartChanges {
                tool: None,
                watch_patterns: None,
                client_messages: vec![],
            };
        }
        self.handle_watched_file_change(builder, &file_event.uri, root_uri, options)
    }

    fn did_open(&self, document: &TextDocument) {
        self.update_document(document, true);
    }

    fn did_change(&self, document: &TextDocument) {
        self.update_document(document, false);
    }

    fn did_close(&self, uri: &Uri) {
        let removed = self
            .workspace
            .open_documents
            .lock()
            .expect("open documents mutex poisoned")
            .remove(uri);
        if let Some(document) = removed
            && let Some(server) = self.workspace.server()
        {
            server.did_close(&document.path);
        }
    }

    fn handle_watched_file_change(
        &self,
        builder: &dyn ToolBuilder,
        _changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        // TODO: Check if the changed file is actually a config file (including extended paths)
        builder.shutdown(root_uri);
        let ToolBuildResult { tool, client_messages } = builder.build(root_uri, options);

        ToolRestartChanges {
            tool: Some(tool),
            // TODO: update watch patterns if config_path changed, or the extended paths changed
            watch_patterns: None,
            client_messages,
        }
    }

    /// Tries to execute the given command with the provided arguments.
    /// If the command is not recognized, returns `Err(ErrorCode)`.
    /// If the command is recognized and executed it can return:
    /// - `Ok(Some(WorkspaceEdit))` if the command was executed successfully and produced a workspace edit.
    /// - `Ok(None)` if the command was executed successfully but did not produce any workspace edit.
    ///
    /// # Errors
    /// Returns an `ErrorCode::InvalidParams` if the command arguments are invalid.
    fn execute_command(
        &self,
        command: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        if command != FIX_ALL_COMMAND_ID {
            return Err(ErrorCode::InvalidParams);
        }

        let args = FixAllCommandArgs::try_from(arguments).map_err(|_| ErrorCode::InvalidParams)?;
        let uri: Uri = Uri::from_str(&args.uri).map_err(|_| ErrorCode::InvalidParams)?;

        if !self.is_responsible_for_uri(&uri) {
            return Ok(None);
        }

        // We only run the lint process when the code action is explicitly invoked and the file is not open in the editor.
        let document_version = None;
        let actions = self.get_code_actions_for_uri(
            &uri,
            Some(CodeActionTriggerKind::Invoked),
            document_version,
        );

        let Some(actions) = actions else {
            return Ok(None);
        };

        if actions.is_empty() {
            return Ok(None);
        }

        let text_edits = fix_all_text_edit(actions.into_iter());

        if text_edits.is_empty() {
            return Ok(None);
        }

        Ok(Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(uri, text_edits)])),
            document_changes: None,
            change_annotations: None,
        }))
    }

    fn get_code_actions_or_commands(&self, params: CodeActionParams) -> Vec<CodeActionResponse> {
        let actions = self.get_code_actions_for_uri(
            &params.uri,
            params.context.trigger_kind,
            params.document_version,
        );

        let Some(actions) = actions else {
            return vec![];
        };

        if actions.is_empty() {
            return vec![];
        }

        let actions = actions.into_iter().filter(|r| range_overlaps(params.range, r.range));

        // `context.only` is a special case here. ESLint behavior is if `source.fixAll` is the first element in `context.only`,
        // then only return fix all code action, and ignore other code actions, even if they are requested.
        // https://github.com/microsoft/vscode-eslint/blob/1572a25c619861a812c6593c9b130ee52361bcf0/server/src/eslintServer.ts#L587-L589
        // This works for zed editor too, it sends always with this layout: `"only": ["quickfix", "source.fixAll.oxc", "source.fixAll"]`
        // https://github.com/oxc-project/oxc-zed/issues/133#issuecomment-4007046920
        // To align more with the official LSP specs, we implement it a bit differently:
        // If no `context.only` is applied, only return the quick fix code actions.
        // If it is provided, we should loop over it and return the actions in the same order.
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionContext
        let applying_kinds = match &params.context.only {
            Some(only) => {
                // `source.fixAll` and `source.fixAll.oxc` should behave the same, filter duplicate out
                let mut seen = FxHashSet::default();
                only.iter()
                    .filter_map(|kind| {
                        if kind == &CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC
                            || kind == &CodeActionKind::SourceFixAll
                        {
                            if seen.contains(&CodeActionKind::SourceFixAll) {
                                None
                            } else {
                                seen.insert(CodeActionKind::SourceFixAll);
                                Some(CodeActionKind::SourceFixAll)
                            }
                        } else {
                            Some(kind.clone())
                        }
                    })
                    .collect::<Vec<_>>()
            }
            // if `only` is not provided, only return quickfixes
            None => vec![CodeActionKind::QuickFix],
        };

        let mut code_actions_vec: Vec<CodeActionResponse> = vec![];

        for kind in applying_kinds {
            // `CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC` was filtered out by `applying_kinds`, so we don't need to check it here.
            if kind == CodeActionKind::SourceFixAll {
                let Some(fix_all) = apply_all_fix_code_action(
                    actions.clone(),
                    params.uri.clone(),
                    self.rules_customization.as_ref(),
                ) else {
                    continue;
                };
                code_actions_vec.push(CodeActionResponse::CodeAction(fix_all));
            } else if kind == CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC {
                if !self.fix_kind.is_dangerous() {
                    warn!(
                        "Linter is not configured to provide dangerous fixes. Please set `fixKind` to `dangerous_fix` or `dangerous_fix_or_suggestion` in the server configuration to enable it."
                    );
                    continue;
                }
                let Some(fix_all) = apply_dangerous_fix_code_action(
                    actions.clone(),
                    params.uri.clone(),
                    self.rules_customization.as_ref(),
                ) else {
                    continue;
                };
                code_actions_vec.push(CodeActionResponse::CodeAction(fix_all));
            } else if kind == CodeActionKind::QuickFix {
                for action in actions.clone() {
                    let fix_actions = apply_fix_code_actions(action, &params.uri);
                    code_actions_vec
                        .extend(fix_actions.into_iter().map(CodeActionResponse::CodeAction));
                }
            }
        }

        code_actions_vec
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, an empty vector is returned
    fn run_diagnostic(&self, document: TextDocument) -> DiagnosticResult {
        Ok(vec![(document.uri.clone(), self.run_file(document.uri, document.text.as_deref())?)])
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, an empty vector is returned
    /// - If the linter is not set to `OnType`, an empty vector is returned
    fn run_diagnostic_on_change(&self, document: TextDocument) -> DiagnosticResult {
        if self.run != Run::OnType {
            return Ok(vec![]);
        }
        self.run_diagnostic(document)
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, an empty vector is returned
    /// - If the linter is not set to `OnSave`, an empty vector is returned
    fn run_diagnostic_on_save(&self, document: TextDocument) -> DiagnosticResult {
        if self.run != Run::OnSave {
            return Ok(vec![]);
        }
        self.run_diagnostic(document)
    }

    fn remove_uri_cache(&self, uri: &Uri) {
        self.code_actions.pin().remove(uri);
    }
}

impl ServerLinter {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    fn new(
        run: Run,
        cwd: PathBuf,
        ignore_matcher: LintIgnoreMatcher,
        gitignore_glob: Vec<Gitignore>,
        extended_paths: FxHashSet<PathBuf>,
        runner: LintRunner,
        fix_kind: FixKind,
        unused_directives_severity: Option<AllowWarnDeny>,
        rules_customization: Option<RulesCustomization>,
        workspace: Arc<WorkspaceState>,
    ) -> Self {
        Self {
            run,
            cwd,
            ignore_matcher,
            gitignore_glob,
            extended_paths,
            code_actions: Arc::new(ConcurrentHashMap::default()),
            runner,
            fix_kind,
            unused_directives_severity,
            rules_customization,
            workspace,
        }
    }

    /// Records an opened or changed document and sends it to the workspace's `tsrs-lint server`,
    /// whatever the run mode: the server always sees the unsaved text of every open file.
    fn update_document(&self, document: &TextDocument, open: bool) {
        let Some(text) = document.text.as_ref() else {
            return;
        };
        let Some(path) = uri_to_file_path(document.uri) else {
            return;
        };
        // Only JS/TS sources go to the type-aware linter: the one-shot path never sends partially
        // loaded files (`.vue`, `.svelte`, `.astro`) either.
        if oxc_span::SourceType::from_path(&path).is_err() || !path.starts_with(&self.cwd) {
            return;
        }
        let version = document.version.unwrap_or(0);
        let was_open = self
            .workspace
            .open_documents
            .lock()
            .expect("open documents mutex poisoned")
            .insert(
                document.uri.clone(),
                OpenDocument { path: path.to_path_buf(), version, text: Arc::clone(text) },
            )
            .is_some();
        if let Some(server) = self.workspace.server() {
            if open || !was_open {
                server.did_open(&path, version, Arc::clone(text));
            } else {
                server.did_change(&path, version, Arc::clone(text));
            }
        }
    }

    /// Whether `path` is a lint configuration file, whose change restarts the linter.
    fn is_lint_config_file(&self, path: &Path, options: &serde_json::Value) -> bool {
        if self.extended_paths.contains(path) {
            return true;
        }
        if path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|name| config_file_names().contains(&name))
        {
            return true;
        }
        let config_path = serde_json::from_value::<LSPLintOptions>(options.clone())
            .ok()
            .and_then(|options| options.config_path);
        config_path.is_some_and(|config_path| {
            let config_path = Path::new(&config_path);
            if config_path.is_absolute() {
                path == config_path
            } else {
                path == self.cwd.join(config_path)
            }
        })
    }

    fn get_code_actions_for_uri(
        &self,
        uri: &Uri,
        trigger_kind: Option<CodeActionTriggerKind>,
        document_version: Option<i32>,
    ) -> Option<Vec<LinterCodeAction>> {
        if let Some(cached_code_actions) = self.code_actions.pin().get(uri) {
            Some(cached_code_actions.clone())
        }
        // only run linting and generate code actions when the code action is explicitly invoked,
        // otherwise it will be too heavy to run linting on every file open or cursor move, which will cause performance issues and a bad user experience.
        // It is most likely that the client already sent a request, where we run the lint process and cache the code actions.
        // So we only need to run the lint process when the code action is explicitly invoked and the file is not open in the editor.
        else if trigger_kind == Some(CodeActionTriggerKind::Invoked) && document_version.is_none()
        {
            let _ = self.run_file(uri, None);
            self.code_actions.pin().get(uri).cloned()
        } else {
            None
        }
    }

    fn is_lintable_extension(path: &Path) -> bool {
        static WANTED_EXTENSIONS: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
        let wanted_exts =
            WANTED_EXTENSIONS.get_or_init(|| LINTABLE_EXTENSIONS.iter().copied().collect());

        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| wanted_exts.contains(ext))
    }

    fn is_ignored(&self, uri_path: &Path) -> bool {
        if !Self::is_lintable_extension(uri_path) {
            debug!("ignored (unsupported extension): {uri_path:?}");
            return true;
        }

        if self.ignore_matcher.should_ignore(uri_path) {
            debug!("ignored: {uri_path:?}");
            return true;
        }

        for gitignore in &self.gitignore_glob {
            if !uri_path.starts_with(gitignore.path()) {
                continue;
            }
            if gitignore.matched_path_or_any_parents(uri_path, uri_path.is_dir()).is_ignore() {
                debug!("ignored: {uri_path:?}");
                return true;
            }
        }
        false
    }

    /// Lint a single file, returning an empty diagnostics list if the file is ignored.
    fn run_file(&self, uri: &Uri, content: Option<&str>) -> Result<Vec<Diagnostic>, String> {
        let Some(uri_path) = uri_to_file_path(uri) else {
            return Ok(Vec::new());
        };
        if self.is_ignored(&uri_path) {
            return Ok(Vec::new());
        }

        let reports = self.lint_path(&uri_path, uri, content)?;

        let mut diagnostics = Vec::with_capacity(reports.len());
        // mostly all diagnostics will have code actions (fix + ignoring line/file), only following diagnostics won't:
        // - inverted diagnostics (related spans for the diagnostics)
        // - diagnostics with span(0,0) and no fixes
        // - tsgolint internal diagnostics
        let mut code_actions = vec![];
        for report in reports {
            diagnostics.push(report.diagnostic);

            if let Some(code_action) = report.code_action {
                code_actions.push(code_action);
            }
        }

        self.code_actions.pin().insert(uri.clone(), code_actions);

        Ok(diagnostics)
    }

    fn lint_path(
        &self,
        path: &Path,
        uri: &Uri,
        content: Option<&str>,
    ) -> Result<Vec<DiagnosticReport>, String> {
        debug!("lint {}", path.display());

        let source_text = if let Some(content) = content {
            content
        } else {
            &read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?
        };

        let mut fs = LspFileSystem::default();
        fs.add_file(path.to_path_buf(), Arc::from(source_text));

        let line_index = LineIndex::new(source_text);
        let mut messages: Vec<DiagnosticReport> = self
            .runner
            .run_source(&[Arc::from(path.as_os_str())], &fs)
            .into_iter()
            .filter_map(|message| {
                message_to_lsp_diagnostic(
                    message,
                    uri,
                    &line_index,
                    self.rules_customization.as_ref(),
                )
            })
            .collect();

        messages.append(&mut generate_inverted_diagnostics(&messages, uri));

        // Take directives once to avoid separate get/remove lock acquisitions.
        let directives = self.runner.directives_coordinator().take(path);

        // Add unused directives if configured
        if let Some(severity) = self.unused_directives_severity
            && let Some(directives) = directives
        {
            messages.extend(create_unused_directives_report(&directives, severity, &line_index));
        }
        Ok(messages)
    }

    fn needs_restart(old_options: &LSPLintOptions, new_options: &LSPLintOptions) -> bool {
        old_options.config_path != new_options.config_path
            || old_options.ts_config_path != new_options.ts_config_path
            || old_options.use_nested_configs() != new_options.use_nested_configs()
            || old_options.fix_kind != new_options.fix_kind
            || old_options.unused_disable_directives != new_options.unused_disable_directives
            // TODO: only the TsgoLinter needs to be dropped or created
            || old_options.type_aware != new_options.type_aware
    }

    /// Check if the linter is responsible for the given URI.
    /// e.g. root URI: file:///path/to/root
    ///      responsible for: file:///path/to/root/file.js
    ///      not responsible for: file:///path/to/other/file.js
    fn is_responsible_for_uri(&self, uri: &Uri) -> bool {
        if let Some(path) = uri_to_file_path(uri) {
            return path.starts_with(&self.cwd);
        }
        false
    }
}

#[cfg(test)]
mod tests_builder {
    use tower_lsp_server::gen_lsp_types::{CodeActionKind, CodeActionProvider, ServerCapabilities};

    use oxc_language_server::{Capabilities, DiagnosticMode, ToolBuilder};

    use crate::lsp::{
        code_actions::{
            CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC, CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
        },
        commands::FIX_ALL_COMMAND_ID,
        server_linter::ServerLinterBuilder,
    };

    #[test]
    fn test_server_capabilities_default_providers() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities::default();

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        // Should set code action provider with quickfix and source fix all kinds
        match &capabilities.code_action_provider {
            Some(CodeActionProvider::CodeActionOptions(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QuickFix));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC));
                assert!(code_action_kinds.contains(&CodeActionKind::SourceFixAll));
                assert_eq!(code_action_kinds.len(), 4);
            }
            _ => panic!("Expected code action provider options"),
        }

        // Should set execute command provider with fix all command
        let execute_command_provider = capabilities.execute_command_provider.as_ref().unwrap();
        assert!(execute_command_provider.commands.contains(&FIX_ALL_COMMAND_ID.to_string()));
        assert_eq!(execute_command_provider.commands.len(), 1);
    }

    #[test]
    fn test_server_capabilities_diagnostic_mode() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = Capabilities {
            pull_diagnostics: true,
            refresh_diagnostics: true,
            ..Default::default()
        };
        let mut server_capabilities = ServerCapabilities::default();
        builder.server_capabilities(&mut server_capabilities, &mut capabilities);
        assert_eq!(capabilities.diagnostic_mode, DiagnosticMode::Pull);

        let mut capabilities = Capabilities {
            pull_diagnostics: false,
            refresh_diagnostics: true,
            ..Default::default()
        };
        let mut server_capabilities = ServerCapabilities::default();
        builder.server_capabilities(&mut server_capabilities, &mut capabilities);
        assert_eq!(capabilities.diagnostic_mode, DiagnosticMode::Push);

        let mut capabilities = Capabilities {
            pull_diagnostics: true,
            refresh_diagnostics: false,
            ..Default::default()
        };
        let mut server_capabilities = ServerCapabilities::default();
        builder.server_capabilities(&mut server_capabilities, &mut capabilities);
        assert_eq!(capabilities.diagnostic_mode, DiagnosticMode::Push);

        let mut capabilities = Capabilities {
            pull_diagnostics: false,
            refresh_diagnostics: false,
            ..Default::default()
        };
        let mut server_capabilities = ServerCapabilities::default();
        builder.server_capabilities(&mut server_capabilities, &mut capabilities);
        assert_eq!(capabilities.diagnostic_mode, DiagnosticMode::Push);
    }
}

#[cfg(test)]
mod test_watchers {
    mod init_watchers {
        use crate::lsp::tester::Tester;
        use serde_json::json;

        #[test]
        fn test_default_options() {
            let patterns =
                Tester::new("fixtures/lsp/watchers/default", json!({})).get_watcher_patterns();

            assert_eq!(patterns.len(), 4);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/.oxlintrc.jsonc".to_string());
            assert_eq!(patterns[2], "**/oxlint.config.ts".to_string());
            assert_eq!(patterns[3], "**/oxlint.config.mts".to_string());
        }

        #[test]
        fn test_custom_config_path() {
            let patterns = Tester::new(
                "fixtures/lsp/watchers/default",
                json!({
                    "configPath": "configs/lint.json"
                }),
            )
            .get_watcher_patterns();

            assert_eq!(patterns.len(), 1);
            assert_eq!(patterns[0], "configs/lint.json".to_string());
        }

        #[test]
        fn test_linter_extends_configs() {
            let patterns = Tester::new("fixtures/lsp/watchers/linter_extends", json!({}))
                .get_watcher_patterns();

            // The `.oxlintrc.json` extends `./lint.json` -> 5 watchers
            // (json, jsonc, ts, mts, lint.json)
            assert_eq!(patterns.len(), 5);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/.oxlintrc.jsonc".to_string());
            assert_eq!(patterns[2], "**/oxlint.config.ts".to_string());
            assert_eq!(patterns[3], "**/oxlint.config.mts".to_string());
            assert_eq!(patterns[4], "lint.json".to_string());
        }

        #[test]
        fn test_linter_extends_custom_config_path() {
            let patterns = Tester::new(
                "fixtures/lsp/watchers/linter_extends",
                json!({
                    "configPath": ".oxlintrc.json"
                }),
            )
            .get_watcher_patterns();

            assert_eq!(patterns.len(), 2);
            assert_eq!(patterns[0], ".oxlintrc.json".to_string());
            assert_eq!(patterns[1], "lint.json".to_string());
        }

        #[test]
        fn test_linter_with_type_aware() {
            let patterns = Tester::new(
                "fixtures/lsp/watchers/default",
                json!({
                    "typeAware": true
                }),
            )
            .get_watcher_patterns();

            assert_eq!(patterns.len(), 5);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/.oxlintrc.jsonc".to_string());
            assert_eq!(patterns[2], "**/oxlint.config.ts".to_string());
            assert_eq!(patterns[3], "**/oxlint.config.mts".to_string());
            assert_eq!(patterns[4], "**/tsconfig*.json".to_string());
        }
    }

    mod handle_configuration_change {
        use crate::lsp::tester::Tester;
        use oxc_language_server::ToolRestartChanges;
        use serde_json::json;

        #[test]
        fn test_no_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new("fixtures/lsp/watchers/default", json!({}))
                    .handle_configuration_change(json!({}));

            assert!(watch_patterns.is_none());
        }

        #[test]
        fn test_lint_config_path_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new("fixtures/lsp/watchers/default", json!({}))
                    .handle_configuration_change(json!({
                        "configPath": "configs/lint.json"
                    }));

            assert!(watch_patterns.is_some());
            assert_eq!(watch_patterns.as_ref().unwrap().len(), 1);
            assert_eq!(watch_patterns.unwrap()[0], "configs/lint.json".to_string());
        }

        #[test]
        fn test_lint_other_option_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new("fixtures/lsp/watchers/default", json!({}))
                    .handle_configuration_change(json!({
                        // run is the only option that does not require a restart
                        "run": "onSave"
                    }));

            assert!(watch_patterns.is_none());
        }

        #[test]
        fn test_lint_type_aware_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new("fixtures/lsp/watchers/default", json!({}))
                    .handle_configuration_change(json!({
                        "typeAware": true
                    }));
            assert!(watch_patterns.is_some());
            assert_eq!(watch_patterns.as_ref().unwrap().len(), 5);
            assert_eq!(watch_patterns.as_ref().unwrap()[0], "**/.oxlintrc.json".to_string());
            assert_eq!(watch_patterns.as_ref().unwrap()[1], "**/.oxlintrc.jsonc".to_string());
            assert_eq!(watch_patterns.as_ref().unwrap()[2], "**/oxlint.config.ts".to_string());
            assert_eq!(watch_patterns.as_ref().unwrap()[3], "**/oxlint.config.mts".to_string());
            assert_eq!(watch_patterns.as_ref().unwrap()[4], "**/tsconfig*.json".to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::PathBuf};

    use oxc_language_server::{CodeActionParams, LanguageId, TextDocument, Tool};
    use oxc_linter::ExternalPluginStore;
    use rustc_hash::FxHashSet;
    use serde_json::json;
    use tower_lsp_server::gen_lsp_types::{
        CodeActionContext, CodeActionKind, CodeActionTriggerKind, Position, Range,
    };

    use crate::lsp::{
        code_actions::{
            CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC, CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
        },
        server_linter::ServerLinterBuilder,
        tester::{Tester, get_file_path},
    };

    fn code_action_params(
        uri: &tower_lsp_server::gen_lsp_types::Uri,
        range: Range,
        context: CodeActionContext,
    ) -> CodeActionParams {
        CodeActionParams { uri: uri.clone(), range, context, document_version: None }
    }

    #[test]
    fn test_create_nested_configs() {
        let builder = ServerLinterBuilder::default();
        let mut nested_ignore_patterns = Vec::new();
        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut extended_paths = FxHashSet::default();
        let base_config_path = get_file_path("fixtures/lsp/init_nested_configs/.oxlintrc.json");
        let (configs, client_message) = builder.create_nested_configs(
            &get_file_path("fixtures/lsp/init_nested_configs"),
            &base_config_path,
            &mut external_plugin_store,
            &mut nested_ignore_patterns,
            &mut extended_paths,
            None,
        );
        let mut configs_dirs = configs.keys().collect::<Vec<&PathBuf>>();
        // sorting the key because for consistent tests results
        configs_dirs.sort();

        assert_eq!(configs_dirs.len(), 2);
        assert!(configs_dirs[1].ends_with("deep2"));
        assert!(configs_dirs[0].ends_with("deep1"));
        assert_eq!(client_message, []);
    }

    #[test]
    fn test_create_ignore_glob_skips_git_dir() {
        let root_dir = tempfile::tempdir().unwrap();
        let app_dir = root_dir.path().join("apps").join("foo");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(app_dir.join(".gitignore"), "dist/\n").unwrap();

        let git_dir = root_dir.path().join(".git").join("info");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join(".gitignore"), "refs/\n").unwrap();

        let ignore_globs = ServerLinterBuilder::create_ignore_glob(root_dir.path());

        assert_eq!(ignore_globs.len(), 1);
        assert!(ignore_globs[0].path().starts_with(&app_dir));
    }

    #[test]
    fn test_fix_all_dangerous_returns_dangerous_fix_action() {
        let tester =
            Tester::new("fixtures/lsp/dangerous_fix", json!({ "fixKind": "dangerous_fix" }));
        let linter = tester.create_linter();
        let range = Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX));
        let uri = tester.get_file_uri("unused_var.js");
        let _ = linter
            .run_diagnostic(TextDocument {
                uri: &uri,
                version: None,
                language_id: LanguageId::default(),
                text: Some("let a = 1;".into()),
            })
            .unwrap();

        // source.fixAll should only return safe fixes, not dangerous ones
        let safe_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext {
                only: Some(vec![CodeActionKind::SourceFixAll]),
                ..Default::default()
            },
        ));
        assert!(safe_actions.is_empty(), "source.fixAll should not apply dangerous fixes");

        // source.fixAllDangerous.oxc should return dangerous fix actions when fix_kind is dangerous
        let dangerous_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext {
                only: Some(vec![CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC]),
                ..Default::default()
            },
        ));
        assert!(
            !dangerous_actions.is_empty(),
            "source.fixAllDangerous.oxc should return dangerous fix action when fix_kind is dangerous"
        );
    }

    #[test]
    fn test_code_action_context_only() {
        // this directory doesn't matter because we are directly calling the linter methods, and not relying on the file system for this test.
        let tester = Tester::new("fixtures/lsp/code_action", json!({}));
        let linter = tester.create_linter();
        let range = Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX));
        let uri = tester.get_file_uri("quickfix.js");
        let _ = linter
            .run_diagnostic(TextDocument {
                uri: &uri,
                version: None,
                language_id: LanguageId::default(),
                text: Some("if (foo == NaN) {}".into()),
            })
            .unwrap();
        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext::default(),
        ));
        assert_eq!(
            code_actions.len(),
            3,
            "Default Context: Should return 3 code actions: 1 rule fix + 2 ignore actions"
        );

        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext { only: Some(vec![CodeActionKind::QuickFix]), ..Default::default() },
        ));

        assert_eq!(
            code_actions.len(),
            3,
            "Quickfix Context: Should return 3 code actions: 1 rule fix + 2 ignore actions"
        );

        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext {
                only: Some(vec![CodeActionKind::QuickFix, CodeActionKind::SourceFixAll]),
                ..Default::default()
            },
        ));

        assert_eq!(
            code_actions.len(),
            4,
            "Quickfix & FixAll Context: Should return 4 code actions: 1 rule fix + 2 ignore actions, and 1 fix all action"
        );

        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext {
                only: Some(vec![
                    CodeActionKind::QuickFix,
                    CodeActionKind::SourceFixAll,
                    CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
                ]),
                ..Default::default()
            },
        ));

        assert_eq!(
            code_actions.len(),
            4,
            "Quickfix & FixAll Context: Should return 4 code actions even if both `source.fixAll` and `source.fixAll.oxc` are requested,
            because they are the same action, we should filter out duplicates."
        );
    }

    #[test]
    fn test_code_action_context_trigger_kind_default() {
        // this directory doesn't matter because we are directly calling the linter methods, and not relying on the file system for this test.
        let tester = Tester::new("fixtures/lsp/code_action", json!({}));
        let linter = tester.create_linter();
        let range = Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX));
        let uri = tester.get_file_uri("quickfix.js");
        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext::default(),
        ));
        assert_eq!(
            code_actions.len(),
            0,
            "Default Context: Should return 0 code actions before running the file"
        );

        let _ = linter
            .run_diagnostic(TextDocument {
                uri: &uri,
                version: None,
                text: Some("debugger;".into()),
                language_id: LanguageId::default(),
            })
            .unwrap();
        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext::default(),
        ));

        assert_eq!(
            code_actions.len(),
            3,
            "Default Context after running file: Should return 3 code actions: 1 rule fix + 2 ignore actions"
        );
    }

    #[test]
    fn test_code_action_context_trigger_kind_invoked() {
        let tester = Tester::new("fixtures/lsp/code_action", json!({}));
        let linter = tester.create_linter();
        let range = Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX));
        let uri = tester.get_file_uri("trigger-kind-invoked.js");
        let code_actions = linter.get_code_actions_or_commands(code_action_params(
            &uri,
            range,
            CodeActionContext {
                trigger_kind: Some(CodeActionTriggerKind::Invoked),
                ..Default::default()
            },
        ));
        assert_eq!(
            code_actions.len(),
            3,
            "Invoked Context: Should return 3 code actions: 1 rule fix + 2 ignore actions, Even if the file was not linted before."
        );
    }

    #[test]
    fn test_no_errors() {
        Tester::new("fixtures/lsp/no_errors", json!({}))
            .test_and_snapshot_single_file("hello_world.js");
    }

    #[test]
    fn test_no_console() {
        Tester::new("fixtures/lsp/deny_no_console", json!({}))
            .test_and_snapshot_single_file("hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        Tester::new("fixtures/lsp/issue_9958", json!({})).test_and_snapshot_single_file("issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        Tester::new("fixtures/lsp/regexp_feature", json!({}))
            .test_and_snapshot_single_file("index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new("fixtures/lsp/frameworks", json!({})).test_and_snapshot_multiple_file(&[
            "astro/debugger.astro",
            "vue/debugger.vue",
            "svelte/debugger.svelte",
            "nextjs/[[..rest]]/debugger.ts",
        ]);
    }

    #[test]
    fn test_invalid_syntax_file() {
        Tester::new("fixtures/lsp/invalid_syntax", json!({}))
            .test_and_snapshot_multiple_file(&["debugger.ts", "invalid.vue"]);
    }

    #[test]
    fn test_cross_module_debugger() {
        Tester::new("fixtures/lsp/cross_module", json!({}))
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        Tester::new("fixtures/lsp/cross_module", json!({}))
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        Tester::new("fixtures/lsp/cross_module_nested_config", json!({}))
            .test_and_snapshot_multiple_file(&["dep-a.ts", "folder/folder-dep-a.ts"]);
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        Tester::new("fixtures/lsp/cross_module_extended_config", json!({}))
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_multiple_suggestions() {
        Tester::new(
            "fixtures/lsp/multiple_suggestions",
            json!({
                "fixKind": "safe_fix_or_suggestion"
            }),
        )
        .test_and_snapshot_single_file("forward_ref.ts");
    }

    #[test]
    fn test_report_unused_directives() {
        Tester::new(
            "fixtures/lsp/unused_disabled_directives",
            json!({
                "unusedDisableDirectives": "deny"
            }),
        )
        .test_and_snapshot_single_file("test.js");
    }

    #[test]
    fn test_allow_unused_directives() {
        Tester::new(
            "fixtures/lsp/allow_unused_disabled_directives",
            json!({
                "unusedDisableDirectives": "allow"
            }),
        )
        .test_and_snapshot_single_file("test.js");
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_report_tsgolint_unused_directives() {
        Tester::new(
            "fixtures/lsp/tsgolint/unused_disabled_directives",
            json!({
                "unusedDisableDirectives": "deny",
                "typeAware": true
            }),
        )
        .test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_root_ignore_patterns() {
        let tester = Tester::new("fixtures/lsp/ignore_patterns", json!({}));
        tester.test_and_snapshot_multiple_file(&[
            "ignored-file.ts",
            "another_config/not-ignored-file.ts",
        ]);
    }

    #[test]
    fn test_ts_alias() {
        Tester::new(
            "fixtures/lsp/ts_path_alias",
            json!({
                "tsConfigPath": "./deep/tsconfig.json"
            }),
        )
        .test_and_snapshot_single_file("deep/src/dep-a.ts");
    }

    #[test]
    #[cfg(not(target_endian = "big"))] // TODO: tsgolint doesn't support big endian?
    fn test_tsgo_lint() {
        let tester = Tester::new(
            "fixtures/lsp/tsgolint",
            json!({
                "typeAware": true,
                "fixKind": "all"
            }),
        );
        tester.test_and_snapshot_single_file("no-floating-promises/index.ts");
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_config_file_type_aware_used_when_lsp_not_set() {
        let tester = Tester::new("fixtures/lsp/tsgolint/type_aware_config", json!({}));
        tester.test_and_snapshot_single_file("test.ts");
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_config_file_type_aware_disabled_when_lsp_set() {
        let tester =
            Tester::new("fixtures/lsp/tsgolint/type_aware_config", json!({ "typeAware": false }));
        tester.test_and_snapshot_single_file("test-with-lsp-config.ts");
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_nested_config_file_type_aware_is_rejected() {
        let tester = Tester::new("fixtures/lsp/tsgolint/nested_type_aware", json!({}));
        tester.test_and_snapshot_multiple_file(&["test-root.ts", "nested/test.ts"]);
    }

    #[test]
    fn test_ignore_js_plugins() {
        let tester = Tester::new("fixtures/lsp/js_plugins", json!({}));
        tester.test_and_snapshot_single_file("index.js");
    }

    // https://github.com/oxc-project/oxc/issues/14565
    #[test]
    fn test_issue_14565() {
        let tester = Tester::new(
            "fixtures/lsp/issue_14565",
            json!({
                "run": "onSave"
            }),
        );
        tester.test_and_snapshot_single_file("foo-bar.astro");
    }

    #[test]
    fn test_rules_customization_severity() {
        let tester = Tester::new(
            "fixtures/lsp/rules_customization/severity",
            json!({
                "rulesCustomization": {
                    "no-debugger": {
                        "severity": "warn"
                    },
                    "no-console": {
                        "severity": "off"
                    }
                }
            }),
        );
        tester.test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_rules_customization_severity_with_eslint_prefix() {
        let tester = Tester::new(
            "fixtures/lsp/rules_customization/severity",
            json!({
                "rulesCustomization": {
                    "eslint/no-debugger": {
                        "severity": "warn"
                    },
                    "eslint/no-console": {
                        "severity": "off"
                    }
                }
            }),
        );
        tester.test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_rules_customization_autofix() {
        let tester = Tester::new(
            "fixtures/lsp/rules_customization/autofix",
            json!({
                "rulesCustomization": {
                    "no-debugger": {
                        "autofix": false
                    }
                }
            }),
        );
        tester.test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_rules_customization_autofix_with_eslint_prefix() {
        let tester = Tester::new(
            "fixtures/lsp/rules_customization/autofix",
            json!({
                "rulesCustomization": {
                    "eslint/no-debugger": {
                        "autofix": false
                    }
                }
            }),
        );
        tester.test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_issue_22758() {
        let tester = Tester::new("fixtures/lsp/issue_22758/apps/api", json!({}));
        tester.test_and_snapshot_single_file("app/auth/guard/firebase.ts");
    }
}

/// Type-aware linting with `tsrs-lint`, found in a fixture's `node_modules/.bin`.
#[cfg(all(test, unix, not(target_endian = "big")))]
mod test_tsrs_lint {
    use std::{
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use oxc_language_server::{LanguageId, TextDocument, Tool, uri_utils::file_path_to_uri};
    use serde_json::json;
    use tower_lsp_server::gen_lsp_types::{Code, Diagnostic, FileChangeType, FileEvent, Message};

    use crate::lsp::{
        server_linter::ServerLinterBuilder,
        tester::{Tester, get_file_path},
    };

    /// The fixture whose `node_modules/.bin/tsrs-lint` links to a real `tsrs-lint`.
    const SERVER_FIXTURE: &str = "fixtures/lsp/tsrs_server";
    /// The fixture whose `node_modules/.bin/tsrs-lint` is a script that fails.
    const FAILURE_FIXTURE: &str = "fixtures/lsp/tsrs_failure";
    /// The watcher pattern for sources, watched only with a `tsrs-lint server`.
    const SOURCES_PATTERN: &str = "**/*.{ts,tsx,mts,cts,js,jsx,mjs,cjs,json}";

    /// Links `node_modules/.bin/tsrs-lint` in [`SERVER_FIXTURE`] to the `tsrs-lint` named by
    /// `OXLINT_TSRS_LINT_TEST_BIN`. Returns `false`, and the test is skipped, when it is unset.
    ///
    /// The link is replaced atomically, so tests (and test processes) running at once always see
    /// a complete link.
    #[expect(clippy::print_stderr, reason = "reports a skipped test")]
    fn link_tsrs_lint() -> bool {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let Some(binary) = std::env::var_os("OXLINT_TSRS_LINT_TEST_BIN") else {
            eprintln!("skipping tsrs-lint test: OXLINT_TSRS_LINT_TEST_BIN is not set");
            return false;
        };
        let bin_dir = get_file_path(SERVER_FIXTURE).join("node_modules/.bin");
        std::fs::create_dir_all(&bin_dir).expect("failed to create node_modules/.bin");
        let temporary = bin_dir.join(format!(
            "tsrs-lint.{}.{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::os::unix::fs::symlink(Path::new(&binary), &temporary)
            .expect("failed to link tsrs-lint");
        std::fs::rename(&temporary, bin_dir.join("tsrs-lint")).expect("failed to link tsrs-lint");
        true
    }

    fn codes(diagnostics: &[Diagnostic]) -> Vec<String> {
        diagnostics
            .iter()
            .map(|diagnostic| match &diagnostic.code {
                Some(Code::String(code)) => code.clone(),
                Some(Code::Int(code)) => code.to_string(),
                None => "None".to_string(),
            })
            .collect()
    }

    /// The text of `index.ts` with the unsaved `statement` appended.
    fn unsaved_text(statement: &str) -> String {
        let on_disk = std::fs::read_to_string(get_file_path(&format!("{SERVER_FIXTURE}/index.ts")))
            .expect("failed to read index.ts");
        assert!(!on_disk.contains("load();"));
        format!("{on_disk}\n{statement}\n")
    }

    #[test]
    fn test_tsrs_server_lints_unsaved_text() {
        if !link_tsrs_lint() {
            return;
        }
        let options = json!({ "typeAware": true });
        let tester = Tester::new(SERVER_FIXTURE, options.clone());
        let linter = tester.create_linter();
        assert!(linter.runner.has_tsrs_server());
        assert!(linter.get_watcher_patterns(options).contains(&SOURCES_PATTERN.to_string()));

        // The server lints the open document's unsaved text, which has a floating promise.
        tester.test_and_snapshot_open_file(
            &linter,
            "index.ts",
            &unsaved_text("load();"),
            "index.ts_server",
        );
        let server = linter.workspace.server().expect("no tsrs-lint server");
        assert!(!server.is_disabled());

        // An edit that fixes it reaches the server.
        let uri = tester.get_file_uri("index.ts");
        let fixed = unsaved_text("void load();");
        let document = TextDocument {
            uri: &uri,
            language_id: LanguageId::default(),
            text: Some(fixed.into()),
            version: Some(2),
        };
        linter.did_change(&document);
        let diagnostics = linter.run_diagnostic(document).expect("lint failed");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(codes(&diagnostics[0].1), [] as [String; 0]);
        assert!(!server.is_disabled());
    }

    /// A change to a file outside the workspace root reaches the server: the workspace's programs
    /// can import it.
    #[test]
    #[expect(clippy::print_stderr, reason = "reports a skipped test")]
    fn test_tsrs_server_sees_changes_outside_the_workspace() {
        let Some(binary) = std::env::var_os("OXLINT_TSRS_LINT_TEST_BIN") else {
            eprintln!("skipping tsrs-lint test: OXLINT_TSRS_LINT_TEST_BIN is not set");
            return;
        };
        // `workspace/index.ts` imports `load` from `shared/load.ts`, next to the workspace.
        let temp = tempfile::tempdir().expect("failed to create a temporary directory");
        let dir = temp.path().canonicalize().expect("failed to resolve the temporary directory");
        let root = dir.join("workspace");
        let shared = dir.join("shared/load.ts");
        std::fs::create_dir_all(root.join("node_modules/.bin"))
            .expect("failed to create the workspace");
        std::fs::create_dir_all(dir.join("shared")).expect("failed to create shared");
        std::os::unix::fs::symlink(Path::new(&binary), root.join("node_modules/.bin/tsrs-lint"))
            .expect("failed to link tsrs-lint");
        std::fs::copy(
            get_file_path(&format!("{SERVER_FIXTURE}/.oxlintrc.json")),
            root.join(".oxlintrc.json"),
        )
        .expect("failed to copy .oxlintrc.json");
        std::fs::write(
            root.join("index.ts"),
            "import { load } from \"../shared/load\";\n\nload();\n",
        )
        .expect("failed to write index.ts");
        std::fs::write(
            &shared,
            "export async function load(): Promise<number> {\n  return 1;\n}\n",
        )
        .expect("failed to write load.ts");

        let options = json!({ "typeAware": true });
        let builder = ServerLinterBuilder::default();
        let root_uri = file_path_to_uri(&root).expect("invalid workspace path");
        let (linter, _) = builder.build(&root_uri, options.clone());
        assert!(linter.runner.has_tsrs_server());
        let uri = file_path_to_uri(root.join("index.ts")).expect("invalid index.ts path");
        let lint = || {
            let document = TextDocument {
                uri: &uri,
                language_id: LanguageId::default(),
                text: None,
                version: None,
            };
            let diagnostics = linter.run_diagnostic(document).expect("lint failed");
            assert_eq!(diagnostics.len(), 1);
            codes(&diagnostics[0].1)
        };

        assert_eq!(lint(), ["typescript(no-floating-promises)"]);

        // `load` no longer returns a promise.
        std::fs::write(&shared, "export function load(): number {\n  return 1;\n}\n")
            .expect("failed to write load.ts");
        let event = FileEvent {
            uri: file_path_to_uri(&shared).expect("invalid load.ts path"),
            kind: FileChangeType::Changed,
        };
        let changes = linter.handle_watched_file_event(&builder, &event, &root_uri, options);
        assert!(changes.tool.is_none());

        assert_eq!(lint(), [] as [String; 0]);
    }

    /// A `tsrs-lint` without server mode adds one diagnostic, and keeps the others.
    #[test]
    fn test_tsrs_server_failure_keeps_regular_diagnostics() {
        let tester = Tester::new(FAILURE_FIXTURE, json!({ "typeAware": true }));
        let linter = tester.create_linter();
        assert!(linter.runner.has_tsrs_server());
        let uri = tester.get_file_uri("index.ts");

        // The server is not started again after the first failure, which is reported each time.
        for _ in 0..2 {
            let diagnostics = linter
                .run_diagnostic(TextDocument::new(&uri, LanguageId::default(), None))
                .expect("lint failed");
            assert_eq!(diagnostics.len(), 1);
            let diagnostics = &diagnostics[0].1;
            assert_eq!(codes(diagnostics), ["eslint(no-debugger)", ""]);
            let Message::String(failure) = &diagnostics[1].message else {
                panic!("expected a string message");
            };
            assert!(failure.starts_with("Type-aware linting failed"), "{failure}");
            assert!(failure.contains("exited before the server handshake"), "{failure}");
        }
        assert!(linter.workspace.server().expect("no tsrs-lint server").is_disabled());
    }
}
