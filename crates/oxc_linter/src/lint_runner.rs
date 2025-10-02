use std::{ffi::OsStr, sync::Arc};

use oxc_diagnostics::DiagnosticSender;

use crate::{ConfigStore, LintService, LintServiceOptions, Linter, TsGoLintState};

/// Unified runner that orchestrates both regular (oxc) and type-aware (tsgolint) linting.
pub struct LintRunner {
    /// Regular oxc linter
    regular_linter: Option<Linter>,
    /// Type-aware tsgolint
    type_aware_linter: Option<TsGoLintState>,
    /// Lint service options
    lint_service_options: LintServiceOptions,
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
    ) -> Result<(), String> {
        if let Some(linter) = self.regular_linter.take() {
            let tx_error_clone = tx_error.clone();
            let files_clone = files.to_owned();
            let lint_service_options = self.lint_service_options;

            rayon::spawn(move || {
                let mut lint_service = LintService::new(linter, lint_service_options);
                lint_service.with_paths(files_clone);
                if let Some(fs) = file_system {
                    lint_service.with_file_system(fs);
                }

                lint_service.run(&tx_error_clone);
            });
        }

        if let Some(type_aware_linter) = self.type_aware_linter.take() {
            type_aware_linter.lint(files, tx_error)?;
        } else {
            drop(tx_error);
        }

        Ok(())
    }

    /// Check if type-aware linting is enabled
    pub fn has_type_aware(&self) -> bool {
        self.type_aware_linter.is_some()
    }
}
