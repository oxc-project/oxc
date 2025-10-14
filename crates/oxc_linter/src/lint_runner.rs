use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rustc_hash::FxHashMap;

use oxc_diagnostics::{DiagnosticSender, DiagnosticService};
use oxc_span::Span;

use crate::{
    AllowWarnDeny, ConfigStore, DisableDirectives, LintService, LintServiceOptions, Linter,
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
    /// Lint service options
    lint_service_options: LintServiceOptions,
}

/// Manages disable directives across all linting engines.
///
/// This coordinator stores disable directives for each file and provides
/// thread-safe access using a `Mutex<HashMap>`. The map is shared via `Arc`
/// with `LintService` instances to enable consistent directive handling
/// across regular and type-aware linting.
pub struct DirectivesStore {
    /// Map of file paths to their disable directives
    map: Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
}

impl DirectivesStore {
    /// Create a new directives coordinator
    pub fn new() -> Self {
        Self { map: Arc::new(Mutex::new(FxHashMap::default())) }
    }

    /// Get the underlying map (for sharing with LintService)
    pub fn map(&self) -> Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>> {
        Arc::clone(&self.map)
    }

    /// Check if a diagnostic should be disabled
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn should_disable(&self, path: &Path, rule: &str, span: Span) -> bool {
        let map = self.map.lock().expect("DirectivesStore mutex poisoned in should_disable");
        if let Some(directives) = map.get(path) {
            // Check with various rule name formats
            directives.contains(rule, span)
                || directives.contains(&format!("typescript-eslint/{rule}"), span)
                || directives.contains(&format!("@typescript-eslint/{rule}"), span)
        } else {
            false
        }
    }

    /// Insert disable directives for a file
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn insert(&self, path: PathBuf, directives: DisableDirectives) {
        self.map.lock().expect("DirectivesStore mutex poisoned in insert").insert(path, directives);
    }

    /// Get disable directives for a file
    ///
    /// Returns a clone of the directives for the given path, if they exist.
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn get(&self, path: &Path) -> Option<DisableDirectives> {
        self.map.lock().expect("DirectivesStore mutex poisoned in get").get(path).cloned()
    }

    /// Report unused disable directives
    ///
    /// # Panics
    /// Panics if the mutex is poisoned or if sending to the error channel fails.
    pub fn report_unused(&self, severity: AllowWarnDeny, cwd: &Path, tx_error: &DiagnosticSender) {
        use crate::create_unused_directives_diagnostics;

        let map = self.map.lock().expect("DirectivesStore mutex poisoned in report_unused");
        for (path, directives) in map.iter() {
            let diagnostics = create_unused_directives_diagnostics(directives, severity);

            if !diagnostics.is_empty() {
                let source_text = std::fs::read_to_string(path.as_path()).unwrap_or_default();
                let wrapped = DiagnosticService::wrap_diagnostics(
                    cwd,
                    path.clone(),
                    &source_text,
                    diagnostics,
                );
                tx_error
                    .send((path.clone(), wrapped))
                    .expect("failed to send unused directive diagnostics");
            }
        }
    }

    /// Clear all disable directives
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn clear(&self) {
        self.map.lock().expect("DirectivesStore mutex poisoned in clear").clear();
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
    lint_service_options: LintServiceOptions,
    silent: bool,
}

impl LintRunnerBuilder {
    pub fn new(lint_service_options: LintServiceOptions, config_store: ConfigStore) -> Self {
        Self {
            regular_linter: None,
            type_aware_enabled: false,
            config_store,
            lint_service_options,
            silent: false,
        }
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
            match TsGoLintState::try_new(self.lint_service_options.cwd(), self.config_store.clone())
            {
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
            lint_service_options: self.lint_service_options,
        })
    }
}

impl LintRunner {
    /// Create a new builder for LintRunner
    pub fn builder(
        lint_service_options: LintServiceOptions,
        config_store: ConfigStore,
    ) -> LintRunnerBuilder {
        LintRunnerBuilder::new(lint_service_options, config_store)
    }

    /// Run both regular and type-aware linting on files
    /// # Errors
    /// Returns an error if type-aware linting fails.
    pub fn lint_files(
        mut self,
        files: &[Arc<OsStr>],
        tx_error: DiagnosticSender,
        file_system: Option<Box<dyn crate::RuntimeFileSystem + Sync + Send>>,
        suppression_manager: Option<crate::suppression::ThreadSafeSuppressionManager>,
    ) -> Result<Self, String> {
        // Phase 1: Regular linting (collects disable directives)
        if let Some(linter) = self.regular_linter.take() {
            let files = files.to_owned();
            let directives_map = self.directives_store.map();
            let lint_service_options = self.lint_service_options.clone();

            let mut lint_service = LintService::new(linter, lint_service_options);
            lint_service.with_paths(files);
            lint_service.set_disable_directives_map(directives_map);

            // Set suppression manager if provided
            if let Some(suppression_mgr) = suppression_manager.clone() {
                lint_service.set_suppression_manager(suppression_mgr);
            }

            // Set custom file system if provided
            if let Some(fs) = file_system {
                lint_service.with_file_system(fs);
            }

            lint_service.run(&tx_error);
        }

        if let Some(type_aware_linter) = self.type_aware_linter.take() {
            type_aware_linter.lint(files, self.directives_store.map(), tx_error)?;
        } else {
            drop(tx_error);
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
            self.directives_store.report_unused(
                severity,
                self.lint_service_options.cwd(),
                tx_error,
            );
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
