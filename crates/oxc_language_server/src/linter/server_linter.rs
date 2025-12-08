use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use ignore::gitignore::Gitignore;
use log::{debug, warn};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use tower_lsp_server::{
    UriExt,
    jsonrpc::ErrorCode,
    lsp_types::{
        CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionProviderCapability,
        Diagnostic, ExecuteCommandOptions, Pattern, Range, ServerCapabilities, Uri,
        WorkDoneProgressOptions, WorkspaceEdit,
    },
};

use oxc_linter::{
    AllowWarnDeny, Config, ConfigStore, ConfigStoreBuilder, ExternalPluginStore, FixKind,
    LintIgnoreMatcher, LintOptions, Oxlintrc,
};

use crate::linter::error_with_position::LinterCodeAction;
use crate::{
    ConcurrentHashMap,
    linter::{
        LINT_CONFIG_FILE,
        code_actions::{
            CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC, apply_all_fix_code_action, apply_fix_code_actions,
            fix_all_text_edit,
        },
        commands::{FIX_ALL_COMMAND_ID, FixAllCommandArgs},
        config_walker::ConfigWalker,
        isolated_lint_handler::{IsolatedLintHandler, IsolatedLintHandlerOptions},
        options::{LintOptions as LSPLintOptions, Run, UnusedDisableDirectives},
    },
    tool::{Tool, ToolBuilder, ToolRestartChanges, ToolShutdownChanges},
    utils::normalize_path,
};

pub struct ServerLinterBuilder;

impl ServerLinterBuilder {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(root_uri: &Uri, options: serde_json::Value) -> ServerLinter {
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
        let mut nested_ignore_patterns = Vec::new();
        let (nested_configs, mut extended_paths) =
            Self::create_nested_configs(&root_path, &options, &mut nested_ignore_patterns);
        let config_path = options.config_path.as_ref().map_or(LINT_CONFIG_FILE, |v| v);
        let config = normalize_path(root_path.join(config_path));
        let oxlintrc = if config.try_exists().is_ok_and(|exists| exists) {
            if let Ok(oxlintrc) = Oxlintrc::from_file(&config) {
                oxlintrc
            } else {
                warn!("Failed to initialize oxlintrc config: {}", config.to_string_lossy());
                Oxlintrc::default()
            }
        } else {
            warn!(
                "Config file not found: {}, fallback to default config",
                config.to_string_lossy()
            );
            Oxlintrc::default()
        };

        let base_patterns = oxlintrc.ignore_patterns.clone();

        let mut external_plugin_store = ExternalPluginStore::new(false);
        let config_builder =
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc, None, &mut external_plugin_store)
                .unwrap_or_default();

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = options.use_nested_configs();
        let fix_kind = FixKind::from(options.fix_kind);

        let use_cross_module = config_builder.plugins().has_import()
            || (use_nested_config
                && nested_configs.pin().values().any(|config| config.plugins().has_import()));

        extended_paths.extend(config_builder.extended_paths.clone());
        let base_config = config_builder.build(&mut external_plugin_store).unwrap_or_else(|err| {
            warn!("Failed to build config: {err}");
            ConfigStoreBuilder::empty().build(&mut external_plugin_store).unwrap()
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
        let config_store = ConfigStore::new(
            base_config,
            if use_nested_config {
                let nested_configs = nested_configs.pin();
                nested_configs
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<FxHashMap<_, _>>()
            } else {
                FxHashMap::default()
            },
            external_plugin_store,
        );

        let isolated_linter = IsolatedLintHandler::new(
            lint_options,
            config_store,
            &IsolatedLintHandlerOptions {
                use_cross_module,
                type_aware: options.type_aware,
                fix_kind,
                root_path: root_path.to_path_buf(),
                tsconfig_path: options.ts_config_path.as_ref().map(|path| {
                    let path = Path::new(path).to_path_buf();
                    if path.is_relative() { root_path.join(path) } else { path }
                }),
            },
        );

        ServerLinter::new(
            options.run,
            root_path.to_path_buf(),
            isolated_linter,
            LintIgnoreMatcher::new(&base_patterns, &root_path, nested_ignore_patterns),
            Self::create_ignore_glob(&root_path),
            extended_paths,
        )
    }
}

impl ToolBuilder for ServerLinterBuilder {
    fn server_capabilities(&self, capabilities: &mut ServerCapabilities) {
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
    }
    fn build_boxed(&self, root_uri: &Uri, options: serde_json::Value) -> Box<dyn Tool> {
        Box::new(ServerLinterBuilder::build(root_uri, options))
    }
}

impl ServerLinterBuilder {
    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    fn create_nested_configs(
        root_path: &Path,
        options: &LSPLintOptions,
        nested_ignore_patterns: &mut Vec<(Vec<String>, PathBuf)>,
    ) -> (ConcurrentHashMap<PathBuf, Config>, FxHashSet<PathBuf>) {
        let mut extended_paths = FxHashSet::default();
        // nested config is disabled, no need to search for configs
        if !options.use_nested_configs() {
            return (ConcurrentHashMap::default(), extended_paths);
        }

        let paths = ConfigWalker::new(root_path).paths();
        let nested_configs =
            ConcurrentHashMap::with_capacity_and_hasher(paths.capacity(), FxBuildHasher);

        for path in paths {
            let file_path = Path::new(&path);
            let Some(dir_path) = file_path.parent() else {
                continue;
            };

            let Ok(oxlintrc) = Oxlintrc::from_file(file_path) else {
                warn!("Skipping invalid config file: {}", file_path.display());
                continue;
            };
            // Collect ignore patterns and their root
            nested_ignore_patterns.push((oxlintrc.ignore_patterns.clone(), dir_path.to_path_buf()));
            let mut external_plugin_store = ExternalPluginStore::new(false);
            let Ok(config_store_builder) = ConfigStoreBuilder::from_oxlintrc(
                false,
                oxlintrc,
                None,
                &mut external_plugin_store,
            ) else {
                warn!("Skipping config (builder failed): {}", file_path.display());
                continue;
            };
            extended_paths.extend(config_store_builder.extended_paths.clone());
            let config =
                config_store_builder.build(&mut external_plugin_store).unwrap_or_else(|err| {
                    warn!("Failed to build nested config for {}: {:?}", dir_path.display(), err);
                    ConfigStoreBuilder::empty().build(&mut external_plugin_store).unwrap()
                });
            nested_configs.pin().insert(dir_path.to_path_buf(), config);
        }

        (nested_configs, extended_paths)
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
    isolated_linter: IsolatedLintHandler,
    ignore_matcher: LintIgnoreMatcher,
    gitignore_glob: Vec<Gitignore>,
    extended_paths: FxHashSet<PathBuf>,
    code_actions: Arc<ConcurrentHashMap<Uri, Option<Vec<LinterCodeAction>>>>,
}

impl Tool for ServerLinter {
    fn name(&self) -> &'static str {
        "linter"
    }

    fn shutdown(&self) -> ToolShutdownChanges {
        ToolShutdownChanges { uris_to_clear_diagnostics: Some(self.get_cached_uris()) }
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    fn handle_configuration_change(
        &self,
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
        let new_linter = ServerLinterBuilder::build(root_uri, new_options_json.clone());

        let patterns = {
            if old_option.config_path == new_options.config_path
                && old_option.use_nested_configs() == new_options.use_nested_configs()
            {
                None
            } else {
                Some(new_linter.get_watcher_patterns(new_options_json))
            }
        };

        ToolRestartChanges { tool: Some(Box::new(new_linter)), watch_patterns: patterns }
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
        let mut watchers = vec![
            options.config_path.as_ref().unwrap_or(&"**/.oxlintrc.json".to_string()).to_owned(),
        ];

        for path in &self.extended_paths {
            // ignore .oxlintrc.json files when using nested configs
            if path.ends_with(".oxlintrc.json") && options.use_nested_configs() {
                continue;
            }

            let pattern = path.strip_prefix(self.cwd.clone()).unwrap_or(path);

            watchers.push(normalize_path(pattern).to_string_lossy().to_string());
        }
        watchers
    }

    fn handle_watched_file_change(
        &self,
        _changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        // TODO: Check if the changed file is actually a config file (including extended paths)
        let new_linter = ServerLinterBuilder::build(root_uri, options);

        ToolRestartChanges {
            tool: Some(Box::new(new_linter)),
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
        let uri = Uri::from_str(&args.uri).map_err(|_| ErrorCode::InvalidParams)?;

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
        let is_source_fix_all_oxc = only_code_action_kinds
            .is_some_and(|only| only.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));

        if is_source_fix_all_oxc {
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
    /// - If the file is not lintable or ignored, [`None`] is returned
    /// - If the file is lintable, but no diagnostics are found, an empty vector is returned
    fn run_diagnostic(&self, uri: &Uri, content: Option<&str>) -> Option<Vec<Diagnostic>> {
        self.run_file(uri, content)
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, [`None`] is returned
    /// - If the linter is not set to `OnType`, [`None`] is returned
    /// - If the file is lintable, but no diagnostics are found, an empty vector is returned
    fn run_diagnostic_on_change(
        &self,
        uri: &Uri,
        content: Option<&str>,
    ) -> Option<Vec<Diagnostic>> {
        if self.run != Run::OnType {
            return None;
        }
        self.run_diagnostic(uri, content)
    }

    /// Lint a file with the current linter
    /// - If the file is not lintable or ignored, [`None`] is returned
    /// - If the linter is not set to `OnSave`, [`None`] is returned
    /// - If the file is lintable, but no diagnostics are found, an empty vector is returned
    fn run_diagnostic_on_save(&self, uri: &Uri, content: Option<&str>) -> Option<Vec<Diagnostic>> {
        if self.run != Run::OnSave {
            return None;
        }
        self.run_diagnostic(uri, content)
    }

    fn remove_diagnostics(&self, uri: &Uri) {
        self.code_actions.pin().remove(uri);
    }
}

impl ServerLinter {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn new(
        run: Run,
        cwd: PathBuf,
        isolated_linter: IsolatedLintHandler,
        ignore_matcher: LintIgnoreMatcher,
        gitignore_glob: Vec<Gitignore>,
        extended_paths: FxHashSet<PathBuf>,
    ) -> Self {
        Self {
            run,
            cwd,
            isolated_linter,
            ignore_matcher,
            gitignore_glob,
            extended_paths,
            code_actions: Arc::new(ConcurrentHashMap::default()),
        }
    }

    fn get_cached_uris(&self) -> Vec<Uri> {
        self.code_actions.pin().keys().cloned().collect()
    }

    fn get_code_actions_for_uri(&self, uri: &Uri) -> Option<Vec<LinterCodeAction>> {
        if let Some(cached_code_actions) = self.code_actions.pin().get(uri) {
            cached_code_actions.clone()
        } else {
            self.run_file(uri, None);
            self.code_actions.pin().get(uri).and_then(std::clone::Clone::clone)
        }
    }

    fn is_ignored(&self, uri: &Uri) -> bool {
        let Some(uri_path) = uri.to_file_path() else {
            return true;
        };

        if self.ignore_matcher.should_ignore(&uri_path) {
            debug!("ignored: {uri:?}");
            return true;
        }

        for gitignore in &self.gitignore_glob {
            if !uri_path.starts_with(gitignore.path()) {
                continue;
            }
            if gitignore.matched_path_or_any_parents(&uri_path, uri_path.is_dir()).is_ignore() {
                debug!("ignored: {uri:?}");
                return true;
            }
        }
        false
    }

    /// Lint a single file, return `None` if the file is ignored.
    fn run_file(&self, uri: &Uri, content: Option<&str>) -> Option<Vec<Diagnostic>> {
        if self.is_ignored(uri) {
            return None;
        }

        let mut diagnostics = vec![];
        let mut code_actions = vec![];

        let reports = self.isolated_linter.run_single(uri, content);
        if let Some(reports) = reports {
            for report in reports {
                diagnostics.push(report.diagnostic);

                if let Some(code_action) = report.code_action {
                    code_actions.push(code_action);
                }
            }
        }

        self.code_actions.pin().insert(uri.clone(), Some(code_actions));

        Some(diagnostics)
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
    use tower_lsp_server::lsp_types::{
        CodeActionKind, CodeActionOptions, CodeActionProviderCapability, ExecuteCommandOptions,
        ServerCapabilities, WorkDoneProgressOptions,
    };

    use crate::{
        ServerLinterBuilder, ToolBuilder,
        linter::{code_actions::CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC, commands::FIX_ALL_COMMAND_ID},
    };

    #[test]
    fn test_server_capabilities_empty_capabilities() {
        let builder = ServerLinterBuilder;
        let mut capabilities = ServerCapabilities::default();

        builder.server_capabilities(&mut capabilities);

        // Should set code action provider with quickfix and source fix all kinds
        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert_eq!(code_action_kinds.len(), 2);
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
        let builder = ServerLinterBuilder;
        let mut capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::REFACTOR]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: Some(true),
            })),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities);

        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::REFACTOR));
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert_eq!(code_action_kinds.len(), 3);
                assert_eq!(options.resolve_provider, Some(true));
            }
            _ => panic!("Expected code action provider options"),
        }
    }

    #[test]
    fn test_server_capabilities_with_existing_quickfix_kind() {
        let builder = ServerLinterBuilder;
        let mut capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: None,
            })),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities);

        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert_eq!(code_action_kinds.len(), 2);
            }
            _ => panic!("Expected code action provider options"),
        }
    }

    #[test]
    fn test_server_capabilities_with_simple_code_action_provider() {
        let builder = ServerLinterBuilder;
        let mut capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities);

        // Should override with options
        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(options)) => {
                let code_action_kinds = options.code_action_kinds.as_ref().unwrap();
                assert!(code_action_kinds.contains(&CodeActionKind::QUICKFIX));
                assert!(code_action_kinds.contains(&CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
                assert_eq!(code_action_kinds.len(), 2);
            }
            _ => panic!("Expected code action provider options"),
        }
    }

    #[test]
    fn test_server_capabilities_with_existing_commands() {
        let builder = ServerLinterBuilder;
        let mut capabilities = ServerCapabilities {
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec!["existing.command".to_string()],
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: Some(true),
                },
            }),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities);

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
        let builder = ServerLinterBuilder;
        let mut capabilities = ServerCapabilities {
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![FIX_ALL_COMMAND_ID.to_string()],
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            ..Default::default()
        };

        builder.server_capabilities(&mut capabilities);

        let execute_command_provider = capabilities.execute_command_provider.as_ref().unwrap();
        assert!(execute_command_provider.commands.contains(&FIX_ALL_COMMAND_ID.to_string()));
        assert_eq!(execute_command_provider.commands.len(), 1);
    }
}

#[cfg(test)]
mod test_watchers {
    mod init_watchers {
        use crate::linter::tester::Tester;
        use serde_json::json;

        #[test]
        fn test_default_options() {
            let patterns =
                Tester::new("fixtures/linter/watchers/default", json!({})).get_watcher_patterns();

            assert_eq!(patterns.len(), 1);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
        }

        #[test]
        fn test_custom_config_path() {
            let patterns = Tester::new(
                "fixtures/linter/watchers/default",
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
            let patterns = Tester::new("fixtures/linter/watchers/linter_extends", json!({}))
                .get_watcher_patterns();

            // The `.oxlintrc.json` extends `./lint.json -> 2 watchers
            assert_eq!(patterns.len(), 2);
            assert_eq!(patterns[0], "**/.oxlintrc.json".to_string());
            assert_eq!(patterns[1], "lint.json".to_string());
        }

        #[test]
        fn test_linter_extends_custom_config_path() {
            let patterns = Tester::new(
                "fixtures/linter/watchers/linter_extends",
                json!({
                    "configPath": ".oxlintrc.json"
                }),
            )
            .get_watcher_patterns();

            assert_eq!(patterns.len(), 2);
            assert_eq!(patterns[0], ".oxlintrc.json".to_string());
            assert_eq!(patterns[1], "lint.json".to_string());
        }
    }

    mod handle_configuration_change {
        use crate::{ToolRestartChanges, linter::tester::Tester};
        use serde_json::json;

        #[test]
        fn test_no_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new("fixtures/linter/watchers/default", json!({}))
                    .handle_configuration_change(json!({}));

            assert!(watch_patterns.is_none());
        }

        #[test]
        fn test_lint_config_path_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new("fixtures/linter/watchers/default", json!({}))
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
                Tester::new("fixtures/linter/watchers/default", json!({}))
                    .handle_configuration_change(json!({
                        // run is the only option that does not require a restart
                        "run": "onSave"
                    }));

            assert!(watch_patterns.is_none());
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use crate::linter::{
        options::LintOptions,
        server_linter::ServerLinterBuilder,
        tester::{Tester, get_file_path},
    };

    #[test]
    fn test_create_nested_configs_with_disabled_nested_configs() {
        let mut nested_ignore_patterns = Vec::new();
        let (configs, _) = ServerLinterBuilder::create_nested_configs(
            Path::new("/root/"),
            &LintOptions { disable_nested_config: true, ..LintOptions::default() },
            &mut nested_ignore_patterns,
        );

        assert!(configs.is_empty());
    }

    #[test]
    fn test_create_nested_configs() {
        let mut nested_ignore_patterns = Vec::new();
        let (configs, _) = ServerLinterBuilder::create_nested_configs(
            &get_file_path("fixtures/linter/init_nested_configs"),
            &LintOptions::default(),
            &mut nested_ignore_patterns,
        );
        let configs = configs.pin();
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
        Tester::new("fixtures/linter/no_errors", json!({}))
            .test_and_snapshot_single_file("hello_world.js");
    }

    #[test]
    fn test_no_console() {
        Tester::new("fixtures/linter/deny_no_console", json!({}))
            .test_and_snapshot_single_file("hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        Tester::new("fixtures/linter/issue_9958", json!({}))
            .test_and_snapshot_single_file("issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        Tester::new("fixtures/linter/regexp_feature", json!({}))
            .test_and_snapshot_single_file("index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new("fixtures/linter/astro", json!({}))
            .test_and_snapshot_single_file("debugger.astro");
        Tester::new("fixtures/linter/vue", json!({})).test_and_snapshot_single_file("debugger.vue");
        Tester::new("fixtures/linter/svelte", json!({}))
            .test_and_snapshot_single_file("debugger.svelte");
        // ToDo: fix Tester to work only with Uris and do not access the file system
        // Tester::new("fixtures/linter/nextjs").test_and_snapshot_single_file("%5B%5B..rest%5D%5D/debugger.ts");
    }

    #[test]
    fn test_invalid_syntax_file() {
        Tester::new("fixtures/linter/invalid_syntax", json!({}))
            .test_and_snapshot_multiple_file(&["debugger.ts", "invalid.vue"]);
    }

    #[test]
    fn test_cross_module_debugger() {
        Tester::new("fixtures/linter/cross_module", json!({}))
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        Tester::new("fixtures/linter/cross_module", json!({}))
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        Tester::new("fixtures/linter/cross_module_nested_config", json!({}))
            .test_and_snapshot_multiple_file(&["dep-a.ts", "folder/folder-dep-a.ts"]);
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        Tester::new("fixtures/linter/cross_module_extended_config", json!({}))
            .test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_multiple_suggestions() {
        Tester::new(
            "fixtures/linter/multiple_suggestions",
            json!({
                "fixKind": "safe_fix_or_suggestion"
            }),
        )
        .test_and_snapshot_single_file("forward_ref.ts");
    }

    #[test]
    fn test_report_unused_directives() {
        Tester::new(
            "fixtures/linter/unused_disabled_directives",
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
            "fixtures/linter/tsgolint/unused_disabled_directives",
            json!({
                "unusedDisableDirectives": "deny",
                "typeAware": true
            }),
        )
        .test_and_snapshot_single_file("test.ts");
    }

    #[test]
    fn test_root_ignore_patterns() {
        let tester = Tester::new("fixtures/linter/ignore_patterns", json!({}));
        tester.test_and_snapshot_multiple_file(&[
            "ignored-file.ts",
            "another_config/not-ignored-file.ts",
        ]);
    }

    #[test]
    fn test_ts_alias() {
        Tester::new(
            "fixtures/linter/ts_path_alias",
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
            "fixtures/linter/tsgolint",
            json!({
                "typeAware": true,
                "fixKind": "all"
            }),
        );
        tester.test_and_snapshot_single_file("no-floating-promises/index.ts");
    }

    #[test]
    fn test_ignore_js_plugins() {
        let tester = Tester::new("fixtures/linter/js_plugins", json!({}));
        tester.test_and_snapshot_single_file("index.js");
    }

    // https://github.com/oxc-project/oxc/issues/14565
    #[test]
    fn test_issue_14565() {
        let tester = Tester::new(
            "fixtures/linter/issue_14565",
            json!({
                "run": "onSave"
            }),
        );
        tester.test_and_snapshot_single_file("foo-bar.astro");
    }
}
