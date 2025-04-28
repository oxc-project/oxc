use std::{
    path::{Path, PathBuf},
    str::FromStr,
    vec,
};

use globset::Glob;
use ignore::gitignore::Gitignore;
use log::{debug, info, warn};
use oxc_linter::{ConfigStore, ConfigStoreBuilder, LintOptions, Linter, Oxlintrc};
use rustc_hash::{FxBuildHasher, FxHashMap};
use tokio::sync::{Mutex, OnceCell, RwLock};
use tower_lsp_server::{
    UriExt,
    lsp_types::{CodeActionOrCommand, Diagnostic, FileChangeType, FileEvent, Range, TextEdit, Uri},
};

use crate::{
    ConcurrentHashMap, OXC_CONFIG_FILE, Options, Run,
    code_actions::{
        apply_all_fix_code_action, apply_fix_code_action, ignore_this_line_code_action,
        ignore_this_rule_code_action,
    },
    linter::{
        config_walker::ConfigWalker, error_with_position::DiagnosticReport,
        isolated_lint_handler::IsolatedLintHandlerOptions, server_linter::ServerLinter,
    },
};

pub struct WorkspaceWorker {
    root_uri: OnceCell<Uri>,
    server_linter: RwLock<ServerLinter>,
    diagnostics_report_map: RwLock<ConcurrentHashMap<String, Vec<DiagnosticReport>>>,
    options: Mutex<Options>,
    gitignore_glob: Mutex<Vec<Gitignore>>,
    nested_configs: RwLock<ConcurrentHashMap<PathBuf, ConfigStore>>,
}

impl WorkspaceWorker {
    pub fn new(root_uri: &Uri, options: Options) -> Self {
        let root_uri_cell = OnceCell::new();
        root_uri_cell.set(root_uri.clone()).unwrap();

        let nested_configs = Self::init_nested_configs(root_uri, &options);
        let (server_linter, oxlintrc) =
            Self::init_linter_config(root_uri, &options, &nested_configs);

        Self {
            root_uri: root_uri_cell,
            server_linter: RwLock::new(server_linter),
            diagnostics_report_map: RwLock::new(ConcurrentHashMap::default()),
            options: Mutex::new(options),
            gitignore_glob: Mutex::new(Self::init_ignore_glob(root_uri, &oxlintrc)),
            nested_configs: RwLock::const_new(nested_configs),
        }
    }

    pub fn get_root_uri(&self) -> Option<Uri> {
        self.root_uri.get().cloned()
    }

    pub fn is_responsible_for_uri(&self, uri: &Uri) -> bool {
        if let Some(root_uri) = self.root_uri.get() {
            if let Some(path) = uri.to_file_path() {
                return path.starts_with(root_uri.to_file_path().unwrap());
            }
        }
        false
    }

    pub async fn remove_diagnostics(&self, uri: &Uri) {
        self.diagnostics_report_map.read().await.pin().remove(&uri.to_string());
    }

    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    fn init_nested_configs(
        root_uri: &Uri,
        options: &Options,
    ) -> ConcurrentHashMap<PathBuf, ConfigStore> {
        // nested config is disabled, no need to search for configs
        if options.use_nested_configs() {
            return ConcurrentHashMap::default();
        }

        let root_path = root_uri.to_file_path().expect("Failed to convert URI to file path");

        let paths = ConfigWalker::new(&root_path).paths();
        let nested_configs =
            ConcurrentHashMap::with_capacity_and_hasher(paths.capacity(), FxBuildHasher);

        for path in paths {
            let file_path = Path::new(&path);
            let Some(dir_path) = file_path.parent() else {
                continue;
            };

            let oxlintrc = Oxlintrc::from_file(file_path).expect("Failed to parse config file");
            let config_store_builder = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc)
                .expect("Failed to create config store builder");
            let config_store = config_store_builder.build().expect("Failed to build config store");
            nested_configs.pin().insert(dir_path.to_path_buf(), config_store);
        }

        nested_configs
    }

    async fn refresh_nested_configs(&self) {
        let options = self.options.lock().await;
        let nested_configs = Self::init_nested_configs(self.root_uri.get().unwrap(), &options);

        *self.nested_configs.write().await = nested_configs;
    }

    fn init_linter_config(
        root_uri: &Uri,
        options: &Options,
        nested_configs: &ConcurrentHashMap<PathBuf, ConfigStore>,
    ) -> (ServerLinter, Oxlintrc) {
        let root_path = root_uri.to_file_path().unwrap();
        let relative_config_path = options.config_path.clone();
        let oxlintrc = if relative_config_path.is_some() {
            let config = root_path.join(relative_config_path.unwrap());
            if config.try_exists().expect("Could not get fs metadata for config") {
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
            }
        } else {
            Oxlintrc::default()
        };

        // clone because we are returning it for ignore builder
        let config_builder =
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc.clone()).unwrap_or_default();

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = options.use_nested_configs();

        let use_cross_module = if use_nested_config {
            nested_configs.pin().values().any(|config| config.plugins().has_import())
        } else {
            config_builder.plugins().has_import()
        };

        let config_store = config_builder.build().expect("Failed to build config store");

        let lint_options = LintOptions { fix: options.fix_kind(), ..Default::default() };

        let linter = if use_nested_config {
            let nested_configs = nested_configs.pin();
            let nested_configs_copy: FxHashMap<PathBuf, ConfigStore> = nested_configs
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<FxHashMap<_, _>>();

            Linter::new_with_nested_configs(lint_options, config_store, nested_configs_copy)
        } else {
            Linter::new(lint_options, config_store)
        };

        let server_linter = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions { use_cross_module, root_path: root_path.to_path_buf() },
        );

        (server_linter, oxlintrc)
    }

    async fn refresh_linter_config(&self) {
        let options = self.options.lock().await;
        let nested_configs = self.nested_configs.read().await;
        let (server_linter, _) =
            Self::init_linter_config(self.root_uri.get().unwrap(), &options, &nested_configs);

        *self.server_linter.write().await = server_linter;
    }

    fn init_ignore_glob(root_uri: &Uri, oxlintrc: &Oxlintrc) -> Vec<Gitignore> {
        let mut builder = globset::GlobSetBuilder::new();
        // Collecting all ignore files
        builder.add(Glob::new("**/.eslintignore").unwrap());
        builder.add(Glob::new("**/.gitignore").unwrap());

        let ignore_file_glob_set = builder.build().unwrap();

        let walk = ignore::WalkBuilder::new(root_uri.to_file_path().unwrap())
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .build()
            .flatten();

        let mut gitignore_globs = vec![];
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

        if !oxlintrc.ignore_patterns.is_empty() {
            let mut builder =
                ignore::gitignore::GitignoreBuilder::new(oxlintrc.path.parent().unwrap());
            for entry in &oxlintrc.ignore_patterns {
                builder.add_line(None, entry).expect("Failed to add ignore line");
            }
            gitignore_globs.push(builder.build().unwrap());
        }

        gitignore_globs
    }

    fn needs_linter_restart(old_options: &Options, new_options: &Options) -> bool {
        old_options.config_path != new_options.config_path
            || old_options.use_nested_configs() != new_options.use_nested_configs()
            || old_options.fix_kind() != new_options.fix_kind()
    }

    pub async fn should_lint_on_run_type(&self, current_run: Run) -> bool {
        let run_level = { self.options.lock().await.run };

        run_level == current_run
    }

    pub async fn lint_file(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if self.is_ignored(uri).await {
            return None;
        }

        Some(self.update_diagnostics(uri, content).await)
    }

    async fn update_diagnostics(
        &self,
        uri: &Uri,
        content: Option<String>,
    ) -> Vec<DiagnosticReport> {
        let diagnostics = self.server_linter.read().await.run_single(uri, content);
        if let Some(diagnostics) = diagnostics {
            self.diagnostics_report_map
                .read()
                .await
                .pin()
                .insert(uri.to_string(), diagnostics.clone());

            return diagnostics;
        }

        vec![]
    }

    async fn revalidate_diagnostics(&self) -> ConcurrentHashMap<String, Vec<DiagnosticReport>> {
        let diagnostics_map = ConcurrentHashMap::with_capacity_and_hasher(
            self.diagnostics_report_map.read().await.len(),
            FxBuildHasher,
        );
        let linter = self.server_linter.read().await;
        for uri in self.diagnostics_report_map.read().await.pin().keys() {
            if let Some(diagnostics) = linter.run_single(&Uri::from_str(uri).unwrap(), None) {
                diagnostics_map.pin().insert(uri.clone(), diagnostics);
            }
        }

        *self.diagnostics_report_map.write().await = diagnostics_map.clone();

        diagnostics_map
    }

    pub async fn get_clear_diagnostics(&self) -> Vec<(String, Vec<Diagnostic>)> {
        self.diagnostics_report_map
            .read()
            .await
            .pin()
            .keys()
            .map(|uri| (uri.clone(), vec![]))
            .collect::<Vec<_>>()
    }

    pub async fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        range: &Range,
        is_source_fix_all_oxc: bool,
    ) -> Vec<CodeActionOrCommand> {
        let report_map = self.diagnostics_report_map.read().await;
        let report_map_ref = report_map.pin();
        let Some(value) = report_map_ref.get(&uri.to_string()) else {
            return vec![];
        };

        let reports = value
            .iter()
            .filter(|r| r.diagnostic.range == *range || range_overlaps(*range, r.diagnostic.range));

        if is_source_fix_all_oxc {
            return apply_all_fix_code_action(reports, uri).map_or(vec![], |code_actions| {
                vec![CodeActionOrCommand::CodeAction(code_actions)]
            });
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

        code_actions_vec
    }

    pub async fn get_diagnostic_text_edits(&self, uri: &Uri) -> Vec<TextEdit> {
        let report_map = self.diagnostics_report_map.read().await;
        let report_map_ref = report_map.pin();
        let Some(value) = report_map_ref.get(&uri.to_string()) else {
            return vec![];
        };

        let mut text_edits = vec![];

        for report in value {
            if let Some(fixed_content) = &report.fixed_content {
                text_edits.push(TextEdit {
                    range: fixed_content.range,
                    new_text: fixed_content.code.clone(),
                });
            }
        }

        text_edits
    }

    pub async fn did_change_watched_files(
        &self,
        file_event: &FileEvent,
    ) -> Option<ConcurrentHashMap<String, Vec<DiagnosticReport>>> {
        if self.options.lock().await.use_nested_configs() {
            let nested_configs = self.nested_configs.read().await;
            let nested_configs = nested_configs.pin();
            let Some(file_path) = file_event.uri.to_file_path() else {
                info!("Unable to convert {:?} to a file path", file_event.uri);
                return None;
            };
            let Some(file_name) = file_path.file_name() else {
                info!("Unable to retrieve file name from {file_path:?}");
                return None;
            };

            if file_name != OXC_CONFIG_FILE {
                return None;
            }

            let Some(dir_path) = file_path.parent() else {
                info!("Unable to retrieve parent from {file_path:?}");
                return None;
            };

            // spellchecker:off -- "typ" is accurate
            if file_event.typ == FileChangeType::CREATED
                || file_event.typ == FileChangeType::CHANGED
            {
                // spellchecker:on
                let oxlintrc =
                    Oxlintrc::from_file(&file_path).expect("Failed to parse config file");
                let config_store_builder = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc)
                    .expect("Failed to create config store builder");
                let config_store =
                    config_store_builder.build().expect("Failed to build config store");
                nested_configs.insert(dir_path.to_path_buf(), config_store);
            // spellchecker:off -- "typ" is accurate
            } else if file_event.typ == FileChangeType::DELETED {
                // spellchecker:on
                nested_configs.remove(&dir_path.to_path_buf());
            }
        }

        self.refresh_linter_config().await;
        Some(self.revalidate_diagnostics().await)
    }

    pub async fn did_change_configuration(
        &self,
        options: serde_json::value::Value,
    ) -> Option<ConcurrentHashMap<String, Vec<DiagnosticReport>>> {
        let changed_options = serde_json::from_value::<Options>(options).unwrap_or_default();

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
            self.refresh_nested_configs().await;
        }

        if Self::needs_linter_restart(current_option, &changed_options) {
            self.refresh_linter_config().await;
            return Some(self.revalidate_diagnostics().await);
        }

        None
    }

    async fn is_ignored(&self, uri: &Uri) -> bool {
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

fn range_overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && a.end >= b.start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_get_root_uri() {
        let worker =
            WorkspaceWorker::new(&Uri::from_str("file://root/").unwrap(), Options::default());

        assert_eq!(worker.get_root_uri(), Some(Uri::from_str("file://root/").unwrap()));
    }

    #[test]
    fn test_is_responsible() {
        let worker = WorkspaceWorker::new(
            &Uri::from_str("file://path/to/root").unwrap(),
            Options::default(),
        );

        assert!(
            worker.is_responsible_for_uri(&Uri::from_str("file://path/to/root/file.js").unwrap())
        );
        assert!(
            worker.is_responsible_for_uri(
                &Uri::from_str("file://path/to/root/folder/file.js").unwrap()
            )
        );
        assert!(
            !worker.is_responsible_for_uri(&Uri::from_str("file://path/to/other/file.js").unwrap())
        );
    }
}
