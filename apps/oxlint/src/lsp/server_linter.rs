use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use ignore::gitignore::Gitignore;
use oxc_data_structures::rope::Rope;
use rustc_hash::{FxHashMap, FxHashSet};
use tower_lsp_server::ls_types::{DiagnosticOptions, DiagnosticServerCapabilities};
use tower_lsp_server::{
    jsonrpc::ErrorCode,
    ls_types::{
        CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionProviderCapability,
        Diagnostic, ExecuteCommandOptions, Pattern, Range, ServerCapabilities, Uri,
        WorkDoneProgressOptions, WorkspaceEdit,
    },
};
use tracing::{debug, error, warn};

use oxc_linter::{
    AllowWarnDeny, Config, ConfigStore, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore,
    FixKind, LINTABLE_EXTENSIONS, LintIgnoreMatcher, LintOptions, LintRunner, LintRunnerBuilder,
    LintServiceOptions, Linter, Oxlintrc, read_to_string,
};

use oxc_language_server::{
    Capabilities, ConcurrentHashMap, DiagnosticMode, DiagnosticResult, Tool, ToolBuilder,
    ToolRestartChanges,
};

use crate::{
    config_loader::{ConfigLoader, build_nested_configs, discover_configs_in_tree},
    lsp::{
        code_actions::{
            CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC, apply_all_fix_code_action, apply_fix_code_actions,
            fix_all_text_edit,
        },
        commands::{FIX_ALL_COMMAND_ID, FixAllCommandArgs},
        error_with_position::{
            DiagnosticReport, LinterCodeAction, create_unused_directives_report,
            generate_inverted_diagnostics, message_to_lsp_diagnostic,
        },
        lsp_file_system::LspFileSystem,
        options::{LintOptions as LSPLintOptions, Run, UnusedDisableDirectives},
        utils::normalize_path,
    },
};

#[derive(Default)]
pub struct ServerLinterBuilder {
    external_linter: Option<ExternalLinter>,
    #[cfg(feature = "napi")]
    js_config_loader: Option<crate::js_config::JsConfigLoaderCb>,
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
        }
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ServerLinter {
        let options = match serde_json::from_value::<LSPLintOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPLintOptions from JSON: {e}. Falling back to default options."
                );
                LSPLintOptions::default()
            }
        };
        let root_path = root_uri.to_file_path().unwrap();
        let mut external_linter = self.external_linter.as_ref();
        let mut external_plugin_store = ExternalPluginStore::new(external_linter.is_some());

        // Setup JS workspace. This must be done before loading any configs
        if let Some(external_linter) = external_linter {
            let res = (external_linter.create_workspace)(root_uri.as_str().to_string());

            if let Err(err) = res {
                error!("Failed to setup JS workspace:\n{err}\n");
            }
        }

        let mut nested_ignore_patterns = Vec::new();
        let mut extended_paths = FxHashSet::default();
        let nested_configs = if options.use_nested_configs() {
            self.create_nested_configs(
                &root_path,
                &mut external_plugin_store,
                &mut nested_ignore_patterns,
                &mut extended_paths,
                Some(root_uri.as_str()),
            )
        } else {
            FxHashMap::default()
        };

        let config_path = options.config_path.as_ref().filter(|p| !p.is_empty()).map(PathBuf::from);
        let loader = ConfigLoader::new(
            external_linter,
            &mut external_plugin_store,
            &[],
            Some(root_uri.as_str()),
        );
        #[cfg(feature = "napi")]
        let loader = loader.with_js_config_loader(self.js_config_loader.as_ref());

        let oxlintrc =
            match loader.load_root_config_with_ancestor_search(&root_path, config_path.as_ref()) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to load config: {e}");
                    Oxlintrc::default()
                }
            };

        let base_patterns = oxlintrc.ignore_patterns.clone();

        let config_builder = match ConfigStoreBuilder::from_oxlintrc(
            false,
            oxlintrc,
            external_linter,
            &mut external_plugin_store,
            Some(root_uri.as_str()),
        ) {
            Ok(builder) => builder,
            Err(e) => {
                warn!("Failed to build config from oxlintrc: {e}");
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
            warn!("Failed to build config: {err}");
            ConfigStoreBuilder::empty().build(&mut ExternalPluginStore::new(false)).unwrap()
        });

        let lint_options = LintOptions {
            fix: fix_kind,
            report_unused_directive: match options.unused_disable_directives {
                UnusedDisableDirectives::Allow => None, // or AllowWarnDeny::Allow, should be the same?
                UnusedDisableDirectives::Warn => Some(AllowWarnDeny::Warn),
                UnusedDisableDirectives::Deny => Some(AllowWarnDeny::Deny),
            },
            ..Default::default()
        };
        if external_plugin_store.is_empty() {
            external_linter = None;
        }

        let config_store = ConfigStore::new(base_config, nested_configs, external_plugin_store);
        let config_store_clone = config_store.clone();

        // Send JS plugins config to JS side
        if let Some(external_linter) = external_linter {
            let res = config_store.external_plugin_store().setup_rule_configs(
                root_path.to_string_lossy().into_owned(),
                Some(root_uri.as_str()),
                external_linter,
            );
            if let Err(err) = res {
                error!("Failed to setup JS plugins config:\n{err}\n");
            }
        }

        let linter = Linter::new(lint_options, config_store, external_linter.cloned())
            .with_workspace_uri(Some(root_uri.as_str()));
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
            .with_type_aware(options.type_aware)
            .with_fix_kind(fix_kind)
            .build()
        {
            Ok(runner) => runner,
            Err(e) => {
                warn!("Failed to initialize type-aware linting: {e}");
                let linter =
                    Linter::new(lint_options, config_store_clone, external_linter.cloned())
                        .with_workspace_uri(Some(root_uri.as_str()));
                LintRunnerBuilder::new(lint_service_options, linter)
                    .with_type_aware(false)
                    .with_fix_kind(fix_kind)
                    .build()
                    .expect("Failed to build LintRunner without type-aware linting")
            }
        };

        ServerLinter::new(
            options.run,
            root_path.to_path_buf(),
            LintIgnoreMatcher::new(&base_patterns, &root_path, nested_ignore_patterns),
            Self::create_ignore_glob(&root_path),
            extended_paths,
            runner,
            lint_options.report_unused_directive,
        )
    }
}

impl ToolBuilder for ServerLinterBuilder {
    fn server_capabilities(
        &self,
        capabilities: &mut ServerCapabilities,
        backend_capabilities: &mut Capabilities,
    ) {
        let mut code_action_kinds = capabilities
            .code_action_provider
            .as_ref()
            .and_then(|cap| match cap {
                CodeActionProviderCapability::Simple(_) => None,
                CodeActionProviderCapability::Options(options) => options.code_action_kinds.clone(),
            })
            .unwrap_or_default();

        if !code_action_kinds.contains(&CodeActionKind::QUICKFIX) {
            code_action_kinds.push(CodeActionKind::QUICKFIX);
        }
        if !code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC) {
            code_action_kinds.push(CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC);
        }
        if !code_action_kinds.contains(&CodeActionKind::SOURCE_FIX_ALL) {
            code_action_kinds.push(CodeActionKind::SOURCE_FIX_ALL);
        }

        // override code action kinds if the code action provider is already set
        capabilities.code_action_provider =
            Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(code_action_kinds),
                work_done_progress_options: capabilities
                    .code_action_provider
                    .as_ref()
                    .and_then(|cap| match cap {
                        CodeActionProviderCapability::Simple(_) => None,
                        CodeActionProviderCapability::Options(options) => {
                            Some(options.work_done_progress_options)
                        }
                    })
                    .unwrap_or_default(),
                resolve_provider: capabilities.code_action_provider.as_ref().and_then(|cap| {
                    match cap {
                        CodeActionProviderCapability::Simple(_) => None,
                        CodeActionProviderCapability::Options(options) => options.resolve_provider,
                    }
                }),
            }));

        let mut commands = capabilities
            .execute_command_provider
            .as_ref()
            .map_or(vec![], |opts| opts.commands.clone());

        if !commands.contains(&FIX_ALL_COMMAND_ID.to_string()) {
            commands.push(FIX_ALL_COMMAND_ID.to_string());
        }

        capabilities.execute_command_provider = Some(ExecuteCommandOptions {
            commands,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: capabilities
                    .execute_command_provider
                    .as_ref()
                    .and_then(|provider| provider.work_done_progress_options.work_done_progress),
            },
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
                Some(DiagnosticServerCapabilities::Options(DiagnosticOptions::default()))
            } else {
                None
            };
    }

    fn build_boxed(&self, root_uri: &Uri, options: serde_json::Value) -> Box<dyn Tool> {
        Box::new(self.build(root_uri, options))
    }

    #[expect(unused)]
    fn shutdown(&self, root_uri: &Uri) {
        // We don't currently destroy workspaces.
        // See comment in `destroyWorkspace` in `src-js/workspace/index.ts` for explanation.
        return;

        // Destroy JS workspace
        if let Some(external_linter) = &self.external_linter {
            let res = (external_linter.destroy_workspace)(root_uri.as_str().to_string());

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
        external_plugin_store: &mut ExternalPluginStore,
        nested_ignore_patterns: &mut Vec<(Vec<String>, PathBuf)>,
        extended_paths: &mut FxHashSet<PathBuf>,
        workspace_uri: Option<&str>,
    ) -> FxHashMap<PathBuf, Config> {
        let config_paths = discover_configs_in_tree(root_path);

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

        let (configs, errors) = loader.load_discovered(config_paths);

        for error in errors {
            if let Some(path) = error.path() {
                warn!("Skipping config file {}: {:?}", path.display(), error);
            } else {
                warn!("Skipping config file: {:?}", error);
            }
        }

        build_nested_configs(configs, nested_ignore_patterns, Some(extended_paths))
    }

    #[expect(clippy::filetype_is_file)]
    fn create_ignore_glob(root_path: &Path) -> Vec<Gitignore> {
        let walk = ignore::WalkBuilder::new(root_path)
            .ignore(true)
            .hidden(false)
            .git_global(false)
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
    code_actions: Arc<ConcurrentHashMap<Uri, Option<Vec<LinterCodeAction>>>>,
    runner: LintRunner,
    unused_directives_severity: Option<AllowWarnDeny>,
}

impl Tool for ServerLinter {
    fn name(&self) -> &'static str {
        "linter"
    }

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
            return ToolRestartChanges { tool: None, watch_patterns: None };
        }

        // get the cached files before refreshing the linter, and revalidate them after
        builder.shutdown(root_uri);
        let new_linter = builder.build_boxed(root_uri, new_options_json.clone());

        let patterns = {
            if old_option.config_path == new_options.config_path
                && old_option.use_nested_configs() == new_options.use_nested_configs()
                && old_option.type_aware == new_options.type_aware
            {
                None
            } else {
                Some(new_linter.get_watcher_patterns(new_options_json))
            }
        };

        ToolRestartChanges { tool: Some(new_linter), watch_patterns: patterns }
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
        let mut watchers = match options.config_path.as_deref() {
            Some("") | None => {
                // Watch both JSON and TS config files
                vec!["**/.oxlintrc.json".to_string(), "**/oxlint.config.ts".to_string()]
            }
            Some(v) => vec![v.to_string()],
        };

        for path in &self.extended_paths {
            // ignore .oxlintrc.json and oxlint.config.ts files when using nested configs
            if (path.ends_with(".oxlintrc.json") || path.ends_with("oxlint.config.ts"))
                && options.use_nested_configs()
            {
                continue;
            }

            let pattern = path.strip_prefix(self.cwd.clone()).unwrap_or(path);

            watchers.push(normalize_path(pattern).to_string_lossy().to_string());
        }

        if options.type_aware {
            watchers.push("**/tsconfig*.json".to_string());
        }

        watchers
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
        let new_linter = builder.build_boxed(root_uri, options);

        ToolRestartChanges {
            tool: Some(new_linter),
            // TODO: update watch patterns if config_path changed, or the extended paths changed
            watch_patterns: None,
        }
    }

    /// Check if the linter should know about the given command
    fn is_responsible_for_command(&self, command: &str) -> bool {
        command == FIX_ALL_COMMAND_ID
    }

    /// Tries to execute the given command with the provided arguments.
    /// If the command is not recognized, returns `Ok(None)`.
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
            return Ok(None);
        }

        let args = FixAllCommandArgs::try_from(arguments).map_err(|_| ErrorCode::InvalidParams)?;
        let uri: Uri = args.uri.parse().map_err(|_| ErrorCode::InvalidParams)?;

        if !self.is_responsible_for_uri(&uri) {
            return Ok(None);
        }

        let actions = self.get_code_actions_for_uri(&uri);

        let Some(actions) = actions else {
            return Ok(None);
        };

        if actions.is_empty() {
            return Ok(None);
        }

        let text_edits = fix_all_text_edit(actions.into_iter());

        Ok(Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(uri, text_edits)])),
            document_changes: None,
            change_annotations: None,
        }))
    }

    fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        range: &Range,
        only_code_action_kinds: Option<&Vec<CodeActionKind>>,
    ) -> Vec<CodeActionOrCommand> {
        let actions = self.get_code_actions_for_uri(uri);

        let Some(actions) = actions else {
            return vec![];
        };

        if actions.is_empty() {
            return vec![];
        }

        let actions =
            actions.into_iter().filter(|r| r.range == *range || range_overlaps(*range, r.range));
        // if `source.fixAll.oxc` or `source.fixAll` is requested, return a single code action that applies all fixes
        let is_source_fix_all = only_code_action_kinds.is_some_and(|only| {
            only.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC)
                || only.contains(&CodeActionKind::SOURCE_FIX_ALL)
        });

        if is_source_fix_all {
            return apply_all_fix_code_action(actions, uri.clone())
                .map_or(vec![], |code_actions| {
                    vec![CodeActionOrCommand::CodeAction(code_actions)]
                });
        }

        let mut code_actions_vec: Vec<CodeActionOrCommand> = vec![];

        for action in actions {
            let fix_actions = apply_fix_code_actions(action, uri);
            code_actions_vec.extend(fix_actions.into_iter().map(CodeActionOrCommand::CodeAction));
        }

        code_actions_vec
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, an empty vector is returned
    fn run_diagnostic(&self, uri: &Uri, content: Option<&str>) -> DiagnosticResult {
        Ok(vec![(uri.clone(), self.run_file(uri, content)?)])
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, an empty vector is returned
    /// - If the linter is not set to `OnType`, an empty vector is returned
    fn run_diagnostic_on_change(&self, uri: &Uri, content: Option<&str>) -> DiagnosticResult {
        if self.run != Run::OnType {
            return Ok(vec![]);
        }
        self.run_diagnostic(uri, content)
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, an empty vector is returned
    /// - If the linter is not set to `OnSave`, an empty vector is returned
    fn run_diagnostic_on_save(&self, uri: &Uri, content: Option<&str>) -> DiagnosticResult {
        if self.run != Run::OnSave {
            return Ok(vec![]);
        }
        self.run_diagnostic(uri, content)
    }

    fn remove_uri_cache(&self, uri: &Uri) {
        self.code_actions.pin().remove(uri);
    }
}

impl ServerLinter {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn new(
        run: Run,
        cwd: PathBuf,
        ignore_matcher: LintIgnoreMatcher,
        gitignore_glob: Vec<Gitignore>,
        extended_paths: FxHashSet<PathBuf>,
        runner: LintRunner,
        unused_directives_severity: Option<AllowWarnDeny>,
    ) -> Self {
        Self {
            run,
            cwd,
            ignore_matcher,
            gitignore_glob,
            extended_paths,
            code_actions: Arc::new(ConcurrentHashMap::default()),
            runner,
            unused_directives_severity,
        }
    }

    fn get_code_actions_for_uri(&self, uri: &Uri) -> Option<Vec<LinterCodeAction>> {
        if let Some(cached_code_actions) = self.code_actions.pin().get(uri) {
            cached_code_actions.clone()
        } else {
            let _ = self.run_file(uri, None);
            self.code_actions.pin().get(uri).and_then(std::clone::Clone::clone)
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
        let Some(uri_path) = uri.to_file_path() else {
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
        // - unused directives diagnostics
        let mut code_actions = vec![];
        for report in reports {
            diagnostics.push(report.diagnostic);

            if let Some(code_action) = report.code_action {
                code_actions.push(code_action);
            }
        }

        self.code_actions.pin().insert(uri.clone(), Some(code_actions));

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

        let rope = &Rope::from_str(source_text);

        let mut fs = LspFileSystem::default();
        fs.add_file(path.to_path_buf(), Arc::from(source_text));

        let mut messages: Vec<DiagnosticReport> =
            match self.runner.run_source(&[Arc::from(path.as_os_str())], &fs) {
                Ok(results) => results
                    .into_iter()
                    .map(|message| message_to_lsp_diagnostic(message, uri, source_text, rope))
                    .collect(),
                Err(e) => {
                    // clear disable directives on error to prevent stale directives
                    self.runner.directives_coordinator().remove(path);
                    return Err(e);
                }
            };

        messages.append(&mut generate_inverted_diagnostics(&messages, uri));

        // Add unused directives if configured
        if let Some(severity) = self.unused_directives_severity
            && let Some(directives) = self.runner.directives_coordinator().get(path)
        {
            messages.extend(create_unused_directives_report(
                &directives,
                severity,
                source_text,
                rope,
            ));
        }

        // Clear any stale directives because they are no longer needed.
        // This prevents using outdated directive spans if the new linting run fails.
        self.runner.directives_coordinator().remove(path);

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
        if let Some(path) = uri.to_file_path() {
            return path.starts_with(&self.cwd);
        }
        false
    }
}

fn range_overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && a.end >= b.start
}

#[cfg(test)]
mod tests_builder {
    use tower_lsp_server::ls_types::{
        CodeActionKind, CodeActionOptions, CodeActionProviderCapability, ExecuteCommandOptions,
        ServerCapabilities, WorkDoneProgressOptions,
    };

    use oxc_language_server::{Capabilities, DiagnosticMode, ToolBuilder};

    use crate::lsp::{
        code_actions::CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC, commands::FIX_ALL_COMMAND_ID,
        server_linter::ServerLinterBuilder,
    };

    #[test]
    fn test_server_capabilities_empty_capabilities() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities::default();

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        // Should set code action provider with quickfix and source fix all kinds
        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert!(code_action_kinds.contains(&CodeActionKind::SOURCE_FIX_ALL));
                assert_eq!(code_action_kinds.len(), 3);
            }
            _ => panic!("Expected code action provider options"),
        }

        // Should set execute command provider with fix all command
        let execute_command_provider = capabilities.execute_command_provider.as_ref().unwrap();
        assert!(execute_command_provider.commands.contains(&FIX_ALL_COMMAND_ID.to_string()));
        assert_eq!(execute_command_provider.commands.len(), 1);
    }

    #[test]
    fn test_server_capabilities_with_existing_code_action_kinds() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::REFACTOR]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: Some(true),
            })),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::REFACTOR));
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert!(code_action_kinds.contains(&CodeActionKind::SOURCE_FIX_ALL));
                assert_eq!(code_action_kinds.len(), 4);
                assert_eq!(options.resolve_provider, Some(true));
            }
            _ => panic!("Expected code action provider options"),
        }
    }

    #[test]
    fn test_server_capabilities_with_existing_quickfix_kind() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: None,
            })),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert!(code_action_kinds.contains(&CodeActionKind::SOURCE_FIX_ALL));
                assert_eq!(code_action_kinds.len(), 3);
            }
            _ => panic!("Expected code action provider options"),
        }
    }

    #[test]
    fn test_server_capabilities_with_simple_code_action_provider() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        // Should override with options
        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert!(code_action_kinds.contains(&CodeActionKind::SOURCE_FIX_ALL));
                assert_eq!(code_action_kinds.len(), 3);
            }
            _ => panic!("Expected code action provider options"),
        }
    }

    #[test]
    fn test_server_capabilities_with_existing_commands() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities {
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec!["existing.command".to_string()],
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: Some(true),
                },
            }),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        let execute_command_provider = capabilities.execute_command_provider.as_ref().unwrap();
        assert!(execute_command_provider.commands.contains(&"existing.command".to_string()));
        assert!(execute_command_provider.commands.contains(&FIX_ALL_COMMAND_ID.to_string()));
        assert_eq!(execute_command_provider.commands.len(), 2);
        assert_eq!(
            execute_command_provider.work_done_progress_options.work_done_progress,
            Some(true)
        );
    }

    #[test]
    fn test_server_capabilities_with_existing_fix_all_command() {
        let builder = ServerLinterBuilder::default();
        let mut capabilities = ServerCapabilities {
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![FIX_ALL_COMMAND_ID.to_string()],
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

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

            assert_eq!(patterns.len(), 2);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/oxlint.config.ts".to_string());
        }

        #[test]
        fn test_empty_string_config_path() {
            let patterns = Tester::new(
                "fixtures/lsp/watchers/default",
                json!({
                    "configPath": ""
                }),
            )
            .get_watcher_patterns();

            assert_eq!(patterns.len(), 2);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/oxlint.config.ts".to_string());
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

            // The `.oxlintrc.json` extends `./lint.json` -> 3 watchers (json, ts, lint.json)
            assert_eq!(patterns.len(), 3);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/oxlint.config.ts".to_string());
            assert_eq!(patterns[2], "lint.json".to_string());
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

            assert_eq!(patterns.len(), 3);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "**/oxlint.config.ts".to_string());
            assert_eq!(patterns[2], "**/tsconfig*.json".to_string());
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
            assert_eq!(watch_patterns.as_ref().unwrap().len(), 3);
            assert_eq!(watch_patterns.as_ref().unwrap()[0], "**/.oxlintrc.json".to_string());
            assert_eq!(watch_patterns.as_ref().unwrap()[1], "**/oxlint.config.ts".to_string());
            assert_eq!(watch_patterns.as_ref().unwrap()[2], "**/tsconfig*.json".to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use oxc_linter::ExternalPluginStore;
    use rustc_hash::FxHashSet;
    use serde_json::json;

    use crate::lsp::{
        server_linter::ServerLinterBuilder,
        tester::{Tester, get_file_path},
    };

    #[test]
    fn test_create_nested_configs() {
        let builder = ServerLinterBuilder::default();
        let mut nested_ignore_patterns = Vec::new();
        let mut external_plugin_store = ExternalPluginStore::new(false);
        let mut extended_paths = FxHashSet::default();
        let configs = builder.create_nested_configs(
            &get_file_path("fixtures/lsp/init_nested_configs"),
            &mut external_plugin_store,
            &mut nested_ignore_patterns,
            &mut extended_paths,
            None,
        );
        let mut configs_dirs = configs.keys().collect::<Vec<&PathBuf>>();
        // sorting the key because for consistent tests results
        configs_dirs.sort();

        assert!(configs_dirs.len() == 3);
        assert!(configs_dirs[2].ends_with("deep2"));
        assert!(configs_dirs[1].ends_with("deep1"));
        assert!(configs_dirs[0].ends_with("init_nested_configs"));
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
}
