use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use oxc_span::Span;

use crate::{
    AllowWarnDeny, ConfigStore, ConfigStore, DisableDirectives, ExternalLinter, LintService,
    LintService, LintServiceOptions, LintServiceOptions, Linter, Linter, TsGoLintState,
    TsGoLintState,
};

/// Unified runner that orchestrates both regular (oxc) and type-aware (tsgolint) linting
/// with centralized disable directives handling.
pub struct LintRunner {
    /// Regular oxc linter
    regular_linter: Option<Linter>,
    /// Type-aware tsgolint
    type_aware_linter: Option<TsGoLintState>,
    /// Shared disable directives coordinator
    directives_store: DirectivesStore,
    /// Configuration store
    _config_store: ConfigStore,
    /// Current working directory
    cwd: PathBuf,
    /// Whether to run in silent mode (for CLI)
    _silent: bool,
}

/// Manages disable directives across all linting engines
pub struct DirectivesStore {
    /// Map of file paths to their disable directives
    map: Arc<DashMap<PathBuf, DisableDirectives>>,
}

impl DirectivesStore {
    /// Create a new directives coordinator
    pub fn new() -> Self {
        Self { map: Arc::new(DashMap::new()) }
    }

    /// Get the underlying map (for sharing with LintService)
    pub fn map(&self) -> Arc<DashMap<PathBuf, DisableDirectives>> {
        Arc::clone(&self.map)
    }

    /// Check if a diagnostic should be disabled
    pub fn should_disable(&self, path: &Path, rule: &str, span: Span) -> bool {
        if let Some(entry) = self.map.get(path) {
            let directives = entry.value();
            // Check with various rule name formats
            directives.contains(rule, span)
                || directives.contains(&format!("typescript-eslint/{rule}"), span)
                || directives.contains(&format!("@typescript-eslint/{rule}"), span)
        } else {
            false
        }
    }

    /// Insert disable directives for a file
    pub fn insert(&self, path: PathBuf, directives: DisableDirectives) {
        self.map.insert(path, directives);
    }

    /// Get disable directives for a file
    pub fn get(
        &self,
        path: &Path,
    ) -> Option<dashmap::mapref::one::Ref<'_, PathBuf, DisableDirectives>> {
        self.map.get(path)
    }

    /// Report unused disable directives
    ///
    /// # Panics
    /// Panics if sending to the error channel fails.
    pub fn report_unused(&self, severity: AllowWarnDeny, cwd: &Path, tx_error: &DiagnosticSender) {
        use crate::RuleCommentType;

        for entry in self.map.iter() {
            let (path, directives) = (entry.key(), entry.value());
            let unused = directives.collect_unused_disable_comments();

            let unused_enable = directives.unused_enable_comments();

            if !unused.is_empty() || !unused_enable.is_empty() {
                let mut diagnostics = Vec::new();
                let message_for_disable =
                    "Unused eslint-disable directive (no problems were reported).";

                for unused_comment in unused {
                    let span = unused_comment.span;
                    match unused_comment.r#type {
                        RuleCommentType::All => {
                            diagnostics.push(
                                OxcDiagnostic::warn(message_for_disable)
                                    .with_label(span)
                                    .with_severity(if severity == AllowWarnDeny::Deny {
                                        oxc_diagnostics::Severity::Error
                                    } else {
                                        oxc_diagnostics::Severity::Warning
                                    }),
                            );
                        }
                        RuleCommentType::Single(rules) => {
                            for rule in rules {
                                let rule_message = format!(
                                    "Unused eslint-disable directive (no problems were reported from {}).",
                                    rule.rule_name
                                );
                                diagnostics.push(
                                    OxcDiagnostic::warn(rule_message)
                                        .with_label(rule.name_span)
                                        .with_severity(if severity == AllowWarnDeny::Deny {
                                            oxc_diagnostics::Severity::Error
                                        } else {
                                            oxc_diagnostics::Severity::Warning
                                        }),
                                );
                            }
                        }
                    }
                }

                // Report unused enable directives
                for (rule_name, span) in unused_enable {
                    let message = if let Some(rule_name) = rule_name {
                        format!(
                            "Unused eslint-enable directive (no matching eslint-disable found for {rule_name})."
                        )
                    } else {
                        "Unused eslint-enable directive (no matching eslint-disable found)."
                            .to_string()
                    };
                    diagnostics.push(OxcDiagnostic::warn(message).with_label(*span).with_severity(
                        if severity == AllowWarnDeny::Deny {
                            oxc_diagnostics::Severity::Error
                        } else {
                            oxc_diagnostics::Severity::Warning
                        },
                    ));
                }

                if !diagnostics.is_empty() {
                    let source_text = std::fs::read_to_string(path.as_path()).unwrap_or_default();
                    let wrapped = DiagnosticService::wrap_diagnostics(
                        cwd,
                        path.clone(),
                        &source_text,
                        diagnostics,
                    );
                    tx_error.send((path.clone(), wrapped)).unwrap();
                }
            }
        }
    }

    /// Clear all disable directives
    pub fn clear(&self) {
        self.map.clear();
    }
}

impl Default for DirectivesStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for LintRunner
pub struct LintRunnerBuilder {
    regular_linter: Option<Linter>,
    type_aware_enabled: bool,
    config_store: ConfigStore,
    cwd: PathBuf,
    silent: bool,
}

impl LintRunnerBuilder {
    pub fn new(cwd: PathBuf, config_store: ConfigStore) -> Self {
        Self { regular_linter: None, type_aware_enabled: false, config_store, cwd, silent: false }
    }

    #[must_use]
    pub fn with_linter(mut self, linter: Linter) -> Self {
        self.regular_linter = Some(linter);
        self
    }

    #[must_use]
    pub fn with_type_aware(mut self, enabled: bool) -> Self {
        self.type_aware_enabled = enabled;
        self
    }

    #[must_use]
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// # Errors
    /// Returns an error if the type-aware linter fails to initialize.
    pub fn build(self) -> Result<LintRunner, String> {
        let directives_coordinator = DirectivesStore::new();

        let type_aware_linter = if self.type_aware_enabled {
            match TsGoLintState::try_new(
                &self.cwd,
                self.config_store.clone(),
                directives_coordinator.map(),
            ) {
                Ok(state) => Some(state.with_silent(self.silent)),
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        Ok(LintRunner {
            regular_linter: self.regular_linter,
            type_aware_linter,
            directives_store: directives_coordinator,
            _config_store: self.config_store,
            cwd: self.cwd,
            _silent: self.silent,
        })
    }
}

impl LintRunner {
    /// Create a new builder for LintRunner
    pub fn builder(cwd: PathBuf, config_store: ConfigStore) -> LintRunnerBuilder {
        LintRunnerBuilder::new(cwd, config_store)
    }

    /// Run both regular and type-aware linting on files
    /// # Errors
    /// Returns an error if type-aware linting fails.
    pub fn lint_files(
        mut self,
        files: &[Arc<OsStr>],
        tx_error: DiagnosticSender,
        lint_service_options: LintServiceOptions,
        file_system: Option<Box<dyn crate::RuntimeFileSystem + Sync + Send>>,
    ) -> Result<Self, String> {
        // Phase 1: Regular linting (collects disable directives)
        if let Some(linter) = self.regular_linter.take() {
            let regular_handle = {
                let tx_error = tx_error.clone();
                let files = files.to_owned();
                let directives_map = self.directives_store.map();

                std::thread::spawn(move || {
                    let mut lint_service = LintService::new(linter, lint_service_options);
                    lint_service.with_paths(files);
                    lint_service.set_disable_directives_map(directives_map);

                    // Set custom file system if provided
                    if let Some(fs) = file_system {
                        lint_service.with_file_system(fs);
                    }

                    lint_service.run(&tx_error);
                })
            };

            // Wait for regular linting to complete before type-aware linting
            regular_handle.join().map_err(|_| "Regular linting thread panicked")?;
        }

        // Phase 2: Type-aware linting (uses collected disable directives)
        if let Some(type_aware_linter) = self.type_aware_linter.take() {
            type_aware_linter.lint(files, tx_error)?;
        }

        Ok(self)
    }

    /// Report unused disable directives
    pub fn report_unused_directives(
        &self,
        severity: Option<AllowWarnDeny>,
        tx_error: &DiagnosticSender,
    ) {
        if let Some(severity) = severity {
            self.directives_store.report_unused(severity, &self.cwd, tx_error);
        }
    }

    /// Get the directives coordinator for external use
    pub fn directives_coordinator(&self) -> &DirectivesStore {
        &self.directives_store
    }

    /// Check if type-aware linting is enabled
    pub fn has_type_aware(&self) -> bool {
        self.type_aware_linter.is_some()
    }
}
