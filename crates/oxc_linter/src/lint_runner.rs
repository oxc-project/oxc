use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService};
use oxc_span::Span;

use crate::{
    AllowWarnDeny, ConfigStore, DisableDirectives, FixKind, LintService, LintServiceOptions,
    Linter, Message, OsFileSystem, RuleTimingStore, TsGoLintState, suppression::DiffManager,
    tsgolint::type_aware_failure_message,
};

/// Why a lint run could not be completed.
///
/// The two cases are reported differently: a planning failure happened before anything was
/// linted, so there is nothing to show alongside it, while a failure during the run leaves
/// diagnostics which must still reach the user.
#[derive(Debug)]
pub enum LintRunError {
    /// The run could not start. Nothing was linted, and nothing was sent to the diagnostic
    /// channel.
    Planning(String),
    /// The run started and then failed. Everything linted before the failure is already on
    /// its way to the diagnostic channel.
    Running(String),
}

impl std::fmt::Display for LintRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintRunError::Planning(message) | LintRunError::Running(message) => {
                write!(f, "{message}")
            }
        }
    }
}

/// Unified runner that orchestrates both regular (oxc) and type-aware (tsgolint) linting
/// with centralized disable directives handling.
pub struct LintRunner {
    /// Regular oxc linter
    lint_service: LintService,
    /// Type-aware tsgolint
    type_aware_linter: Option<TsGoLintState>,
    /// Shared disable directives coordinator
    directives_store: DirectivesStore,
    /// Current working directory
    cwd: PathBuf,
    type_check_only: bool,
    /// Config store, used to resolve per-file options such as `reportUnusedDisableDirectives`
    config_store: ConfigStore,
    /// Explicit `reportUnusedDisableDirectives` severity coming from the CLI or the editor.
    /// `Some(AllowWarnDeny::Allow)` means "explicitly turned off"; `None` falls back to the config.
    report_unused_directive_override: Option<AllowWarnDeny>,
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
    /// Rules whose pass did not run for a file, so their directives cannot be called unused.
    /// Keyed by file, holding every spelling a directive may use for those rules.
    rules_not_run: Arc<Mutex<FxHashMap<PathBuf, FxHashSet<String>>>>,
}

impl DirectivesStore {
    /// Create a new directives coordinator
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(FxHashMap::default())),
            rules_not_run: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// Record that `rules` were never run for `path`, so their directives must not be reported
    /// as unused.
    ///
    /// `rules` holds every spelling a directive may use for them, because a user may disable
    /// `no-floating-promises`, `typescript/no-floating-promises` or
    /// `@typescript-eslint/no-floating-promises` and mean the same rule.
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn set_rules_not_run(&self, path: PathBuf, rules: FxHashSet<String>) {
        let mut not_run =
            self.rules_not_run.lock().expect("DirectivesStore mutex poisoned in set_rules_not_run");
        if rules.is_empty() {
            // The pass ran this time, so any record from an earlier run is stale; the editor
            // lints the same file repeatedly.
            not_run.remove(&path);
        } else {
            not_run.insert(path, rules);
        }
    }

    /// The rules whose pass did not run for `path`, if any.
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn rules_not_run(&self, path: &Path) -> Option<FxHashSet<String>> {
        self.rules_not_run
            .lock()
            .expect("DirectivesStore mutex poisoned in rules_not_run")
            .get(path)
            .cloned()
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
    /// `severity_for` resolves the severity to use for a given file, so that configs nested in a
    /// monorepo can set `reportUnusedDisableDirectives` per package. Returning `None` (or
    /// [`AllowWarnDeny::Allow`]) skips the file.
    ///
    /// # Panics
    /// Panics if the mutex is poisoned or if sending to the error channel fails.
    pub fn report_unused(
        &self,
        severity_for: impl Fn(&Path) -> Option<AllowWarnDeny>,
        cwd: &Path,
        tx_error: &DiagnosticSender,
    ) {
        use crate::create_unused_directives_diagnostics;

        let map = self.map.lock().expect("DirectivesStore mutex poisoned in report_unused");
        let rules_not_run =
            self.rules_not_run.lock().expect("DirectivesStore mutex poisoned in report_unused");
        for (path, directives) in map.iter() {
            let Some(severity) =
                severity_for(path.as_path()).filter(|severity| severity.is_warn_deny())
            else {
                continue;
            };
            let not_run = rules_not_run.get(path);
            let diagnostics = create_unused_directives_diagnostics(directives, severity, |rule| {
                let Some(not_run) = not_run else { return true };
                // A bare directive covers the rules which did not run too, so it cannot be
                // reported as unused either; there is no way to tell whether it is needed.
                rule.is_some_and(|rule| !not_run.contains(rule))
            });

            if !diagnostics.is_empty() {
                let source_text = std::fs::read_to_string(path.as_path()).unwrap_or_default();
                let wrapped = DiagnosticService::wrap_diagnostics(
                    cwd,
                    path.clone(),
                    &source_text,
                    diagnostics,
                );
                tx_error.send(wrapped).expect("failed to send unused directive diagnostics");
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

    /// Remove disable directives for a specific file
    ///
    /// This should be called before re-linting a file to ensure stale directives
    /// from previous linting runs are not used if the new linting run fails to
    /// produce directives (e.g., due to parse errors).
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn remove(&self, path: &Path) {
        self.map.lock().expect("DirectivesStore mutex poisoned in remove").remove(path);
    }

    /// Take and remove disable directives for a specific file in a single lock acquisition.
    ///
    /// # Panics
    /// Panics if the mutex is poisoned.
    pub fn take(&self, path: &Path) -> Option<DisableDirectives> {
        self.map.lock().expect("DirectivesStore mutex poisoned in take").remove(path)
    }
}

impl Default for DirectivesStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for LintRunner
pub struct LintRunnerBuilder {
    regular_linter: Linter,
    type_aware_enabled: bool,
    type_aware_forced: bool,
    type_check_override: Option<bool>,
    lint_service_options: LintServiceOptions,
    silent: bool,
    fix_kind: FixKind,
    type_check_only: bool,
    timings: bool,
    with_ignore_fixes: bool,
}

impl LintRunnerBuilder {
    pub fn new(lint_service_options: LintServiceOptions, linter: Linter) -> Self {
        Self {
            regular_linter: linter,
            type_aware_enabled: false,
            type_aware_forced: false,
            type_check_override: None,
            lint_service_options,
            silent: false,
            fix_kind: FixKind::None,
            type_check_only: false,
            timings: false,
            with_ignore_fixes: false,
        }
    }

    #[must_use]
    pub fn with_type_aware(mut self, enabled: bool) -> Self {
        self.type_aware_enabled = enabled;
        self
    }

    /// Mark type-aware linting as explicitly requested (`--type-aware`, `--type-check-only`, or
    /// the editor's `typeAware` setting), which applies it to every file instead of only the files
    /// whose config enables `options.typeAware`.
    #[must_use]
    pub fn with_type_aware_forced(mut self, forced: bool) -> Self {
        self.type_aware_forced = forced;
        self
    }

    /// Decide TypeScript compiler diagnostics explicitly for every file handed to `tsgolint`:
    /// `Some(true)` from `--type-check` / `--type-check-only` / the editor's `typeCheck: true`,
    /// `Some(false)` from the editor's `typeCheck: false`. `None` lets the config which governs
    /// each file decide.
    #[must_use]
    pub fn with_type_check_override(mut self, type_check: Option<bool>) -> Self {
        self.type_check_override = type_check;
        self
    }

    #[must_use]
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    #[must_use]
    pub fn with_fix_kind(mut self, fix_kind: FixKind) -> Self {
        self.fix_kind = fix_kind;
        self
    }

    #[must_use]
    pub fn with_type_check_only(mut self, type_check_only: bool) -> Self {
        self.type_check_only = type_check_only;
        self
    }

    #[must_use]
    pub fn with_timings(mut self, timings: bool) -> Self {
        self.timings = timings;
        self
    }

    #[must_use]
    pub fn with_ignore_fixes(mut self, with_ignore_fixes: bool) -> Self {
        self.with_ignore_fixes = with_ignore_fixes;
        self
    }

    /// # Errors
    /// Returns an error if the type-aware linter fails to initialize.
    pub fn build(self) -> Result<LintRunner, String> {
        let type_aware_linter = if self.type_aware_enabled {
            let state = TsGoLintState::try_new(
                self.lint_service_options.cwd(),
                self.regular_linter.config.clone(),
                self.fix_kind,
            )?;
            Some(
                state
                    .with_silent(self.silent)
                    .with_type_check_override(self.type_check_override)
                    .with_timings(self.timings)
                    .with_ignore_fixes(self.with_ignore_fixes)
                    .with_type_aware_forced(self.type_aware_forced),
            )
        } else {
            None
        };

        Ok(self.assemble(type_aware_linter))
    }

    /// Build around a `tsgolint` chosen by the caller, so the tests do not depend on what the
    /// machine running them happens to have installed.
    #[cfg(test)]
    pub fn build_with_tsgolint(self, executable: Option<PathBuf>) -> LintRunner {
        let state = TsGoLintState::for_test(
            self.lint_service_options.cwd(),
            self.regular_linter.config.clone(),
            executable,
        );
        self.assemble(Some(state))
    }

    fn assemble(self, type_aware_linter: Option<TsGoLintState>) -> LintRunner {
        let directives_coordinator = DirectivesStore::new();

        let cwd = self.lint_service_options.cwd().to_path_buf();
        let config_store = self.regular_linter.config.clone();
        let report_unused_directive_override =
            self.regular_linter.options().report_unused_directive;
        let mut lint_service = LintService::new(self.regular_linter, self.lint_service_options);
        lint_service.set_disable_directives_map(directives_coordinator.map());

        LintRunner {
            lint_service,
            type_aware_linter,
            directives_store: directives_coordinator,
            cwd,
            type_check_only: self.type_check_only,
            config_store,
            report_unused_directive_override,
        }
    }
}

impl LintRunner {
    /// Create a new builder for LintRunner
    pub fn builder(lint_service_options: LintServiceOptions, linter: Linter) -> LintRunnerBuilder {
        LintRunnerBuilder::new(lint_service_options, linter)
    }

    /// Run both regular and type-aware linting on files.
    ///
    /// The runner is returned whether or not type-aware linting succeeded, because the disable
    /// directives it collected are still reported.
    pub fn lint_files<const TIMINGS: bool>(
        mut self,
        files: &[Arc<OsStr>],
        tx_error: DiagnosticSender,
        diff_manager: &Arc<DiffManager>,
        rule_timing_store: Option<&RuleTimingStore>,
    ) -> (Self, Result<(), LintRunError>) {
        let fs: &(dyn crate::RuntimeFileSystem + Sync + Send) = &OsFileSystem;

        // Resolve the `tsgolint` executables before anything is linted: when not a single file
        // can be linted with type-aware rules the run fails, and it must do so while the
        // diagnostic channel is still empty, or the regular diagnostics already sent into it
        // would be dropped unread.
        let type_aware_plan = match &self.type_aware_linter {
            Some(type_aware_linter) => match type_aware_linter.try_plan(files) {
                Ok(plan) => Some(plan),
                Err(error) => return (self, Err(LintRunError::Planning(error))),
            },
            None => None,
        };

        if self.type_check_only {
            self.lint_service.collect_parse_diagnostics(fs, files.to_owned(), &tx_error);
        } else {
            self.lint_service.run::<TIMINGS>(
                fs,
                files.to_owned(),
                &tx_error,
                diff_manager,
                rule_timing_store,
            );
        }

        let Some((type_aware_linter, plan)) = self.type_aware_linter.take().zip(type_aware_plan)
        else {
            drop(tx_error);
            return (self, Ok(()));
        };

        let result = type_aware_linter.lint(
            plan,
            &self.directives_store,
            tx_error,
            fs,
            diff_manager,
            rule_timing_store,
        );

        // The runner comes back either way: the disable directives were collected before the
        // failure, and the caller still has to report the unused ones.
        (self, result.map_err(LintRunError::Running))
    }

    /// Run both regular and type-aware linting on files
    /// # Errors
    /// Returns an error if type-aware linting fails.
    pub fn run_source(
        &self,
        files: &[Arc<OsStr>],
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
    ) -> Vec<Message> {
        let mut messages = self.lint_service.run_source(file_system, files.to_owned());

        if let Some(type_aware_linter) = &self.type_aware_linter {
            // The editor lints one file per call, so a failure here is the failure of the only
            // installation there was. It is reported as a warning on that file: the regular
            // rules did run, and dropping their diagnostics because `tsgolint` is broken would
            // wrongly show the file as clean.
            match type_aware_linter.lint_source(files, file_system, &self.directives_store) {
                Ok(type_aware_messages) => messages.extend(type_aware_messages),
                Err(error) => messages.push(type_aware_failure_message(&error)),
            }
        }

        messages
    }

    /// Report unused disable directives.
    ///
    /// The severity is resolved per file: the explicit CLI/editor setting wins, otherwise the
    /// config which governs the file (root or nested) decides.
    pub fn report_unused_directives(&self, tx_error: &DiagnosticSender) {
        self.directives_store.report_unused(
            |path| self.report_unused_directive_for(path),
            &self.cwd,
            tx_error,
        );
    }

    /// The `reportUnusedDisableDirectives` severity that applies to `path`.
    pub fn report_unused_directive_for(&self, path: &Path) -> Option<AllowWarnDeny> {
        self.report_unused_directive_override
            .or_else(|| self.config_store.report_unused_disable_directives_for(path))
    }

    /// Whether any config in this run reports unused disable directives.
    pub fn reports_unused_directives(&self) -> bool {
        self.report_unused_directive_override.is_some_and(AllowWarnDeny::is_warn_deny)
            || self.config_store.reports_unused_disable_directives()
    }

    /// Get the directives coordinator for external use
    pub fn directives_coordinator(&self) -> &DirectivesStore {
        &self.directives_store
    }

    /// Check if type-aware linting is enabled
    pub fn has_type_aware(&self) -> bool {
        self.type_aware_linter.is_some()
    }

    /// Whether type-aware linting is enabled but nothing was found for the working directory.
    ///
    /// The run still proceeds: files in a package which installs its own `tsgolint` are linted
    /// with it, and the others are reported. Editors use this to report it once, up front,
    /// instead of leaving the user to infer it from the per-file warnings.
    pub fn type_aware_has_no_fallback(&self) -> bool {
        self.type_aware_linter.as_ref().is_some_and(TsGoLintState::has_no_fallback)
    }
}

#[cfg(test)]
mod test {
    use std::{
        ffi::OsStr,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use oxc_diagnostics::Severity;
    use rustc_hash::FxHashMap;

    use crate::{
        ConfigStore, ConfigStoreBuilder, ExternalPluginStore, LintOptions, LintServiceOptions,
        Linter, OsFileSystem, RuntimeFileSystem,
        lint_runner::{LintRunner, LintRunnerBuilder},
    };

    /// The rule which makes the fixture below report something, so that the tests can tell
    /// "the regular pass ran" from "nothing ran".
    const REGULAR_RULE: &str = "no-debugger";

    fn config_store() -> ConfigStore {
        let mut plugins = ExternalPluginStore::default();
        let config = ConfigStoreBuilder::default().build(&mut plugins).unwrap();
        ConfigStore::new(config, FxHashMap::default(), plugins)
    }

    /// A workspace with one type-aware file, and the `tsgolint` the runner should use for it.
    fn runner_for(cwd: &Path, executable: Option<PathBuf>) -> LintRunner {
        let linter = Linter::new(LintOptions::default(), config_store(), None);
        LintRunnerBuilder::new(LintServiceOptions::new(cwd.to_path_buf()), linter)
            .with_type_aware(true)
            .with_type_aware_forced(true)
            .build_with_tsgolint(executable)
    }

    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("index.ts");
        std::fs::write(&file, "debugger;\n").unwrap();
        (temp, file)
    }

    fn lint(runner: &LintRunner, file: &Path) -> Vec<crate::Message> {
        let fs: &(dyn RuntimeFileSystem + Sync + Send) = &OsFileSystem;
        runner.run_source(&[Arc::from(file.as_os_str()) as Arc<OsStr>], fs)
    }

    /// The editor must keep the regular diagnostics when no `tsgolint` could be found, and
    /// report why the type-aware ones are missing.
    #[test]
    fn run_source_warns_when_no_executable_was_found_for_the_file() {
        let (temp, file) = fixture();
        let runner = runner_for(temp.path(), None);

        let messages = lint(&runner, &file);

        assert!(
            messages.iter().any(|message| message.error.code.to_string().contains(REGULAR_RULE)),
            "the regular rules should still be reported: {messages:?}"
        );
        // Specifically the "nothing was found for this file" warning: with no group at all
        // there is no group failure to report, so the generic one would be wrong.
        let warning = messages
            .iter()
            .find(|message| {
                message
                    .error
                    .message
                    .contains("Could not find a `tsgolint` executable for this file")
            })
            .unwrap_or_else(|| panic!("the file should be warned about: {messages:?}"));
        assert_eq!(warning.error.severity, Severity::Warning);
        assert!(
            warning.error.help.as_ref().is_some_and(|help| help.contains("oxlint-tsgolint")),
            "the warning should say what to do: {:?}",
            warning.error.help
        );
    }

    /// The editor asks the store which rules never ran for a file, so that its unused-directive
    /// report and the delete quick-fix behind it leave those directives alone.
    #[test]
    fn run_source_records_the_rules_which_did_not_run() {
        let (temp, file) = fixture();
        let runner = runner_for(temp.path(), None);

        lint(&runner, &file);

        let not_run = runner
            .directives_coordinator()
            .rules_not_run(&file)
            .expect("the file was not type-aware linted");
        // Every spelling a directive may use, because the editor sees whichever one was typed.
        for name in [
            "no-floating-promises",
            "typescript/no-floating-promises",
            "typescript-eslint/no-floating-promises",
            "@typescript-eslint/no-floating-promises",
        ] {
            assert!(not_run.contains(name), "{name} should be recorded: {not_run:?}");
        }
        // A rule oxlint runs itself did run, regardless of the `tsgolint` failure.
        assert!(!not_run.contains("no-debugger"), "{not_run:?}");
    }

    /// A `tsgolint` which cannot run removes the type-aware rules only, not the regular ones.
    #[test]
    fn run_source_keeps_the_regular_messages_when_tsgolint_fails() {
        let (temp, file) = fixture();
        // A path which is not an executable at all, so spawning it fails.
        let runner = runner_for(temp.path(), Some(temp.path().join("not-an-executable")));

        let messages = lint(&runner, &file);

        assert!(
            messages.iter().any(|message| message.error.code.to_string().contains(REGULAR_RULE)),
            "the regular rules should still be reported: {messages:?}"
        );
        let warning = messages
            .iter()
            .find(|message| message.error.message.contains("type-aware rules"))
            .expect("the failure should be reported on the file");
        assert_eq!(warning.error.severity, Severity::Warning);
    }
}
