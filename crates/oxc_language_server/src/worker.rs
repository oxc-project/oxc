use std::vec;

use log::debug;
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::{
    UriExt,
    lsp_types::{
        CodeActionOrCommand, Diagnostic, FileEvent, FileSystemWatcher, GlobPattern, OneOf, Range,
        RelativePattern, TextEdit, Uri, WatchKind,
    },
};

use crate::{
    ConcurrentHashMap,
    code_actions::{
        apply_all_fix_code_action, apply_fix_code_actions, ignore_this_line_code_action,
        ignore_this_rule_code_action,
    },
    formatter::server_formatter::ServerFormatter,
    linter::{
        error_with_position::{DiagnosticReport, PossibleFixContent},
        server_linter::{ServerLinter, ServerLinterRun, normalize_path},
    },
    options::Options,
};

pub struct WorkspaceWorker {
    root_uri: Uri,
    server_linter: RwLock<Option<ServerLinter>>,
    server_formatter: RwLock<Option<ServerFormatter>>,
    options: Mutex<Option<Options>>,
}

impl WorkspaceWorker {
    pub fn new(root_uri: Uri) -> Self {
        Self {
            root_uri,
            server_linter: RwLock::new(None),
            server_formatter: RwLock::new(None),
            options: Mutex::new(None),
        }
    }

    pub fn get_root_uri(&self) -> &Uri {
        &self.root_uri
    }

    pub fn is_responsible_for_uri(&self, uri: &Uri) -> bool {
        if let Some(path) = uri.to_file_path() {
            return path.starts_with(self.root_uri.to_file_path().unwrap());
        }
        false
    }

    pub async fn start_worker(&self, options: &Options) {
        *self.options.lock().await = Some(options.clone());

        *self.server_linter.write().await = Some(ServerLinter::new(&self.root_uri, &options.lint));
        if options.format.experimental {
            debug!("experimental formatter enabled");
            *self.server_formatter.write().await = Some(ServerFormatter::new());
        }
    }

    // WARNING: start all programs (linter, formatter) before calling this function
    // each program can tell us customized file watcher patterns
    pub async fn init_watchers(&self) -> Vec<FileSystemWatcher> {
        let mut watchers = Vec::new();

        // clone the options to avoid locking the mutex
        let options = self.options.lock().await;
        let default_options = Options::default();
        let options = options.as_ref().unwrap_or(&default_options);
        let use_nested_configs = options.lint.use_nested_configs();

        // append the base watcher
        watchers.push(FileSystemWatcher {
            glob_pattern: GlobPattern::Relative(RelativePattern {
                base_uri: OneOf::Right(self.root_uri.clone()),
                pattern: options
                    .lint
                    .config_path
                    .as_ref()
                    .unwrap_or(&"**/.oxlintrc.json".to_owned())
                    .to_owned(),
            }),
            kind: Some(WatchKind::all()), // created, deleted, changed
        });

        let Some(root_path) = &self.root_uri.to_file_path() else {
            return watchers;
        };

        let Some(extended_paths) =
            self.server_linter.read().await.as_ref().map(|linter| linter.extended_paths.clone())
        else {
            return watchers;
        };

        for path in &extended_paths {
            // ignore .oxlintrc.json files when using nested configs
            if path.ends_with(".oxlintrc.json") && use_nested_configs {
                continue;
            }

            let pattern = path.strip_prefix(root_path).unwrap_or(path);

            watchers.push(FileSystemWatcher {
                glob_pattern: GlobPattern::Relative(RelativePattern {
                    base_uri: OneOf::Right(self.root_uri.clone()),
                    pattern: normalize_path(pattern).to_string_lossy().to_string(),
                }),
                kind: Some(WatchKind::all()), // created, deleted, changed
            });
        }

        watchers
    }

    pub async fn needs_init_options(&self) -> bool {
        self.options.lock().await.is_none()
    }

    pub async fn has_active_formatter(&self) -> bool {
        self.server_formatter.read().await.is_some()
    }

    pub async fn remove_diagnostics(&self, uri: &Uri) {
        let server_linter_guard = self.server_linter.read().await;
        let Some(server_linter) = server_linter_guard.as_ref() else {
            return;
        };
        server_linter.remove_diagnostics(uri);
    }

    async fn refresh_server_linter(&self) {
        let options = self.options.lock().await;
        let default_options = Options::default();
        let lint_options = &options.as_ref().unwrap_or(&default_options).lint;
        let server_linter = ServerLinter::new(&self.root_uri, lint_options);

        *self.server_linter.write().await = Some(server_linter);
    }

    pub async fn lint_file(
        &self,
        uri: &Uri,
        content: Option<String>,
        run_type: ServerLinterRun,
    ) -> Option<Vec<DiagnosticReport>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return None;
        };

        server_linter.run_single(uri, content, run_type).await
    }

    pub async fn format_file(&self, uri: &Uri, content: Option<String>) -> Option<Vec<TextEdit>> {
        let Some(server_formatter) = &*self.server_formatter.read().await else {
            return None;
        };

        server_formatter.run_single(uri, content)
    }

    async fn revalidate_diagnostics(
        &self,
        uris: Vec<Uri>,
    ) -> ConcurrentHashMap<String, Vec<DiagnosticReport>> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return ConcurrentHashMap::default();
        };

        server_linter.revalidate_diagnostics(uris).await
    }

    pub async fn get_clear_diagnostics(&self) -> Vec<(String, Vec<Diagnostic>)> {
        self.server_linter
            .read()
            .await
            .as_ref()
            .map(|server_linter| {
                server_linter
                    .get_cached_files_of_diagnostics()
                    .iter()
                    .map(|uri| (uri.to_string(), vec![]))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub async fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        range: &Range,
        is_source_fix_all_oxc: bool,
    ) -> Vec<CodeActionOrCommand> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return vec![];
        };

        let value = if let Some(cached_diagnostics) = server_linter.get_cached_diagnostics(uri) {
            cached_diagnostics
        } else {
            let diagnostics = server_linter.run_single(uri, None, ServerLinterRun::Always).await;
            diagnostics.unwrap_or_default()
        };

        if value.is_empty() {
            return vec![];
        }

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
            let mut append_ignore_code_actions = true;

            if let Some(fix_actions) = apply_fix_code_actions(report, uri) {
                // do not append ignore code actions when the error is the ignore action
                if fix_actions
                    .first()
                    .as_ref()
                    .is_some_and(|fix| fix.title == "remove unused disable directive")
                {
                    append_ignore_code_actions = false;
                }
                code_actions_vec
                    .extend(fix_actions.into_iter().map(CodeActionOrCommand::CodeAction));
            }

            if append_ignore_code_actions {
                code_actions_vec.push(CodeActionOrCommand::CodeAction(
                    ignore_this_line_code_action(report, uri),
                ));

                code_actions_vec.push(CodeActionOrCommand::CodeAction(
                    ignore_this_rule_code_action(report, uri),
                ));
            }
        }

        code_actions_vec
    }

    /// This function is used for executing the `oxc.fixAll` command
    pub async fn get_diagnostic_text_edits(&self, uri: &Uri) -> Vec<TextEdit> {
        let Some(server_linter) = &*self.server_linter.read().await else {
            return vec![];
        };
        let value = if let Some(cached_diagnostics) = server_linter.get_cached_diagnostics(uri) {
            cached_diagnostics
        } else {
            let diagnostics = server_linter.run_single(uri, None, ServerLinterRun::Always).await;
            diagnostics.unwrap_or_default()
        };

        if value.is_empty() {
            return vec![];
        }

        let mut text_edits = vec![];

        for report in value {
            let fix = match &report.fixed_content {
                PossibleFixContent::None => None,
                PossibleFixContent::Single(fixed_content) => Some(fixed_content),
                // For multiple fixes, we take the first one as a representative fix.
                // Applying all possible fixes at once is not possible in this context.
                PossibleFixContent::Multiple(multi) => multi.first(),
            };

            if let Some(fixed_content) = &fix {
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
        _file_event: &FileEvent,
    ) -> Option<ConcurrentHashMap<String, Vec<DiagnosticReport>>> {
        let files = {
            let server_linter_guard = self.server_linter.read().await;
            let server_linter = server_linter_guard.as_ref()?;
            server_linter.get_cached_files_of_diagnostics()
        };
        self.refresh_server_linter().await;
        Some(self.revalidate_diagnostics(files).await)
    }

    pub async fn did_change_configuration(
        &self,
        changed_options: &Options,
    ) -> (
        // Diagnostic reports that need to be revalidated
        Option<ConcurrentHashMap<String, Vec<DiagnosticReport>>>,
        // File system watcher for lint config changes
        Option<FileSystemWatcher>,
        // Is true, when the formatter was added to the workspace worker
        bool,
    ) {
        // Scope the first lock so it is dropped before the second lock
        let current_option = {
            let options_guard = self.options.lock().await;
            options_guard.clone()
        }
        .unwrap_or_default();

        debug!(
            "
        configuration changed:
        incoming: {changed_options:?}
        current: {current_option:?}
        "
        );

        {
            let mut options_guard = self.options.lock().await;
            *options_guard = Some(changed_options.clone());
        }

        let mut formatting = false;
        if current_option.format.experimental != changed_options.format.experimental {
            if changed_options.format.experimental {
                debug!("experimental formatter enabled");
                *self.server_formatter.write().await = Some(ServerFormatter::new());
                formatting = true;
            } else {
                debug!("experimental formatter disabled");
                *self.server_formatter.write().await = None;
            }
        }

        if ServerLinter::needs_restart(&current_option.lint, &changed_options.lint) {
            let files = {
                let server_linter_guard = self.server_linter.read().await;
                let server_linter = server_linter_guard.as_ref();
                if let Some(server_linter) = server_linter {
                    server_linter.get_cached_files_of_diagnostics()
                } else {
                    vec![]
                }
            };
            self.refresh_server_linter().await;

            if current_option.lint.config_path != changed_options.lint.config_path {
                return (
                    Some(self.revalidate_diagnostics(files).await),
                    Some(FileSystemWatcher {
                        glob_pattern: GlobPattern::Relative(RelativePattern {
                            base_uri: OneOf::Right(self.root_uri.clone()),
                            pattern: changed_options
                                .lint
                                .config_path
                                .as_ref()
                                .unwrap_or(&"**/.oxlintrc.json".to_string())
                                .to_owned(),
                        }),
                        kind: Some(WatchKind::all()), // created, deleted, changed
                    }),
                    formatting,
                );
            }

            return (Some(self.revalidate_diagnostics(files).await), None, formatting);
        }

        (None, None, formatting)
    }
}

fn range_overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && a.end >= b.start
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_get_root_uri() {
        let worker = WorkspaceWorker::new(Uri::from_str("file:///root/").unwrap());

        assert_eq!(worker.get_root_uri(), &Uri::from_str("file:///root/").unwrap());
    }

    #[test]
    fn test_is_responsible() {
        let worker = WorkspaceWorker::new(Uri::from_str("file:///path/to/root").unwrap());

        assert!(
            worker.is_responsible_for_uri(&Uri::from_str("file:///path/to/root/file.js").unwrap())
        );
        assert!(worker.is_responsible_for_uri(
            &Uri::from_str("file:///path/to/root/folder/file.js").unwrap()
        ));
        assert!(
            !worker
                .is_responsible_for_uri(&Uri::from_str("file:///path/to/other/file.js").unwrap())
        );
    }
}
