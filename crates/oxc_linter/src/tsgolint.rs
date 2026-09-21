use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fmt::Write as _,
    io::{ErrorKind, Read, Write, stderr},
    iter, mem,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use oxc_allocator::Allocator;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService, Error, OxcDiagnostic, Severity};
use oxc_span::{SourceType, Span};

use super::{
    AllowWarnDeny, Config as LinterConfig, ConfigStore, DisableDirectives, ResolvedLinterState,
    read_to_string,
};

use crate::{
    CompositeFix, DirectivesStore, FixKind, Fixer, Message, PossibleFixes, RuleTimingRecord,
    RuleTimingSource, RuleTimingStore, WEBSITE_BASE_RULES_URL,
    suppression::{DiffManager, Filename},
};

/// State required to initialize the `tsgolint` linter.
///
/// # Semantics
///
/// These are the agreed rules of this module. Each one is pinned by the test named after it.
/// Update the named test together with the behaviour.
///
/// - **Construction fails only for an invalid explicit path.** [`TsGoLintState::try_new`]
///   returns an error when `OXLINT_TSGOLINT_PATH` is set but names neither an executable nor a
///   directory holding one; that directory is searched for the names a package manager writes
///   and for the bare `tsgolint` of an extracted release, which on Windows is not the same set.
///   Finding nothing at all is not an error here.
///   (`try_new_fails_only_for_an_invalid_explicit_path`)
/// - **Each file is resolved from its own directory.** The lookup walks up from the file's
///   directory and stops at the working directory; only then does it fall back to the
///   executable resolved for the working directory itself, which came from
///   `OXLINT_TSGOLINT_PATH`, a walk up to the file system root, or `PATH`. The cache is
///   consulted at every level of the walk, so a directory whose ancestor is already resolved
///   never looks at the file system above it. The working directory is only a boundary for the
///   files below it: a file outside it walks to the file system root instead, since linting a
///   sibling package is a supported use case.
///   (`per_file_walk_stops_at_cwd_and_the_cwd_walk_continues_above_it`,
///   `walk_stops_at_the_first_cached_ancestor`,
///   `a_file_outside_cwd_walks_above_it_instead_of_stopping`,
///   `lints_a_package_outside_the_working_directory`,
///   `lints_the_package_which_has_tsgolint_and_reports_the_one_which_does_not`)
/// - **A resolution is remembered for 30 seconds.** Finding one and finding none expire alike,
///   and a directory which learns its answer from an already resolved ancestor inherits that
///   ancestor's age, so a subtree expires together. A remembered executable is checked to still
///   be installed before it is reused, once per distinct executable per run.
///   (`a_resolution_is_remembered_for_thirty_seconds`,
///   `a_cached_answer_does_not_get_younger_as_it_spreads`,
///   `a_cached_hit_is_checked_before_it_is_reused`)
/// - **Nothing resolved for any file is an error, raised before anything is linted.** The CLI
///   run fails instead of quietly skipping type-aware linting, and it fails while the
///   diagnostic channel is still empty. (`nothing_resolved_is_an_error_before_linting`)
/// - **Partial resolution never aborts the run.** The groups which resolved are linted, and the
///   remaining files are reported: once per run in the CLI, listing their paths relative to the
///   working directory, and on every lint in the language server, because each
///   `publishDiagnostics` replaces the whole set for a file.
///   (`lints_the_package_which_has_tsgolint_and_reports_the_one_which_does_not`,
///   `lsp_warns_about_every_unresolved_file_on_every_lint`,
///   `run_source_warns_when_no_executable_was_found_for_the_file`)
/// - **That report is an ordinary lint warning, attached to a file.** It therefore fails
///   `--deny-warnings` and is hidden by `--quiet`, like every other warning, and every reporter
///   can place it: the ones which are not graphical print nothing for a diagnostic with no
///   source, and drop `help`, so the remedy lives in the message.
///   (`test_missing_executable_diagnostic_lists_relative_paths_and_the_remedy`,
///   `reports_the_missing_executable_in_every_format`,
///   `reports_a_group_failure_in_every_format`)
/// - **A group which fails to run is reported without hiding the others.** Every failing
///   executable is named, the failure is reported as a diagnostic so every formatter renders it
///   with the rest, and the diagnostics already produced are still printed, unused disable
///   directives included. Nothing on this path panics: a broken `tsgolint` is reported as an
///   error, which the release profile's `panic = "abort"` could not have caught anyway.
///   Linting a source always completes: the editor gets a warning on the file and keeps the
///   regular diagnostics.
///   (`joins_every_group_failure_into_one_message`,
///   `a_broken_group_becomes_a_warning_rather_than_hiding_the_others`,
///   `a_failing_group_does_not_misalign_the_results`,
///   `run_source_keeps_the_regular_messages_when_tsgolint_fails`,
///   `reports_unused_disable_directives_even_when_a_group_failed`,
///   `prints_the_diagnostics_produced_before_a_group_failed`,
///   `prints_json_diagnostics_and_the_failure_together`)
/// - **Those files keep their type-aware suppressions.** The rules `tsgolint` would have run
///   never ran rather than stopping to fire, so their recorded counts are carried over. Only the
///   rules configured at `Deny` are carried, since those are the only ones the tracker counts;
///   rules oxlint runs itself, including those under the same `typescript` plugin, are
///   untouched.
///   (`test_type_aware_skipped_file_keeps_its_type_aware_suppressions`,
///   `test_type_aware_skipped_file_still_reports_its_native_suppressions`,
///   `test_type_aware_rules_below_deny_are_not_carried_over`)
/// - **Those files also keep their type-aware disable directives.** A directive naming a rule
///   which never ran, or a bare one which covers those rules too, cannot be reported as unused,
///   because there is no way to tell whether it is needed. This holds for the files of a
///   group whose `tsgolint` failed as much as for the files which had none, and in the editor
///   as much as on the command line, quick-fix included. A directive which names a rule that
///   ran *and* one that did not keeps reporting the first, per rule, and is never offered for
///   deletion as a whole.
///   (`keeps_the_type_aware_directives_of_the_files_which_could_not_be_linted`,
///   `keeps_the_type_aware_directives_of_a_group_which_failed`,
///   `reports_only_the_rules_which_ran_of_a_mixed_directive`,
///   `run_source_records_the_rules_which_did_not_run`,
///   `a_directive_of_a_rule_which_never_ran_is_neither_reported_nor_offered_for_deletion`,
///   `a_directive_of_a_rule_which_ran_is_still_reported_with_its_fix`)
/// - **Malformed `tsgolint` output fails the group, not the process.** Output which does not
///   parse, or which announces a message larger than anything real, fails that group and leaves
///   the others alone; a missing field or an oversized announced length is rejected before any
///   allocation.
///   (`reports_malformed_tsgolint_output_as_a_group_failure`,
///   `reports_an_oversized_tsgolint_message_as_a_group_failure`,
///   `parse_single_message_rejects_a_size_above_the_maximum`)
/// - **Groups are keyed by installation, and their executable is deterministic.** The key is
///   the canonical `node_modules/oxlint-tsgolint` directory, so two shims of one installation
///   share a process; the lexicographically smallest shim is the one run.
///   (`test_executable_group_key_identifies_the_package`,
///   `test_executable_group_key_follows_the_package_manager_store`,
///   `test_group_paths_by_executable_picks_one_shim_deterministically`)
/// - **At most `min(4, available_parallelism())` groups run at once.**
///   (`concurrency_is_capped_at_four_groups`)
#[derive(Debug, Clone)]
pub struct TsGoLintState {
    /// The `tsgolint` executable resolved from [`Self::cwd`], see [`Fallback`].
    fallback: Fallback,
    /// Outcome of the per-directory lookups, see [`ExecutableCache`].
    executable_cache: Arc<ExecutableCache>,
    /// Current working directory, used for rendering paths in diagnostics.
    cwd: PathBuf,
    /// The configuration store for `tsgolint` (used to resolve configurations outside of `oxc_linter`)
    config_store: ConfigStore,
    /// If `oxlint` will output the diagnostics or not.
    /// When `silent` is true, we do not need to access the file system for nice diagnostics messages.
    silent: bool,
    /// If `true`, request that fixes be returned from `tsgolint`.
    fix: bool,
    /// If `true`, request that suggestions be returned from `tsgolint`.
    fix_suggestions: bool,
    /// When set, type-checking was decided explicitly (`--type-check`, `--type-check-only`, or
    /// the editor's `typeCheck` setting) and applies to every file handed to `tsgolint`,
    /// regardless of the `options.typeCheck` value of the config which governs it. `None` lets
    /// each config decide for the files it governs.
    type_check_override: Option<bool>,
    /// If `true`, request that per-rule debug timings be returned from `tsgolint`.
    timings: bool,
    /// If `true`, the linter will create "ignore this section / line" fixes for all diagnostics
    with_ignore_fixes: bool,
    /// If `true`, type-aware linting was requested explicitly (CLI flag or editor setting) and
    /// applies to every file, regardless of the `options.typeAware` value of the config which
    /// governs that file.
    type_aware_forced: bool,
}

impl TsGoLintState {
    /// Try to create a new `TsGoLintState`, resolving the `tsgolint` executable for `cwd`.
    ///
    /// Not finding one is *not* an error here: in a monorepo `tsgolint` is often installed only
    /// in the packages, so the executable is resolved again for each linted directory (see
    /// `resolve_executable_for_dir`). A run in which nothing is found for any
    /// file does fail, see [`TsGoLintState::lint`].
    ///
    /// # Errors
    /// Returns an error if `OXLINT_TSGOLINT_PATH` is set but does not point at an executable:
    /// a specific executable was requested and it cannot be honoured.
    pub fn try_new(
        cwd: &Path,
        config_store: ConfigStore,
        fix_kind: FixKind,
    ) -> Result<Self, String> {
        Ok(TsGoLintState {
            config_store,
            fallback: resolve_tsgolint_executable(cwd)?,
            executable_cache: Arc::new(ExecutableCache::default()),
            cwd: cwd.to_path_buf(),
            silent: false,
            fix: fix_kind.contains(FixKind::Fix),
            fix_suggestions: fix_kind.contains(FixKind::Suggestion),
            type_check_override: None,
            timings: false,
            with_ignore_fixes: false,
            type_aware_forced: false,
        })
    }

    /// A state which uses `executable` for every file, or none at all, so the tests do not
    /// depend on what the machine running them happens to have installed.
    #[cfg(test)]
    pub(crate) fn for_test(
        cwd: &Path,
        config_store: ConfigStore,
        executable: Option<PathBuf>,
    ) -> Self {
        TsGoLintState {
            config_store,
            fallback: executable
                .map_or(Fallback::None, |path| Fallback::Discovered(ResolvedExecutable::new(path))),
            executable_cache: Arc::new(ExecutableCache::default()),
            cwd: cwd.to_path_buf(),
            silent: false,
            fix: false,
            fix_suggestions: false,
            type_check_override: None,
            timings: false,
            with_ignore_fixes: false,
            type_aware_forced: true,
        }
    }

    /// Set to `true` to skip file system reads.
    /// When `silent` is true, we do not need to access the file system for nice diagnostics messages.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_silent(mut self, yes: bool) -> Self {
        self.silent = yes;
        self
    }

    /// Decide TypeScript compiler diagnostics explicitly, for every file handed to `tsgolint`:
    /// `Some(true)` type-checks them all (`--type-check`, `--type-check-only`, the editor's
    /// `typeCheck: true`), `Some(false)` type-checks none of them (the editor's
    /// `typeCheck: false`). This never changes *which* files are handed to `tsgolint`; that is
    /// decided by type-aware linting.
    ///
    /// Default is `None`: each file follows the `options.typeCheck` of the config which governs
    /// it.
    #[must_use]
    pub fn with_type_check_override(mut self, type_check: Option<bool>) -> Self {
        self.type_check_override = type_check;
        self
    }

    /// Set to `true` to request that per-rule debug timings be returned from `tsgolint`.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_timings(mut self, yes: bool) -> Self {
        self.timings = yes;
        self
    }

    #[must_use]
    pub fn with_ignore_fixes(mut self, yes: bool) -> Self {
        self.with_ignore_fixes = yes;
        self
    }

    /// Set to `true` when type-aware linting was requested explicitly (`--type-aware`,
    /// `--type-check-only`, or the editor's `typeAware` setting). Every file is then linted with
    /// type-aware rules, instead of only the files whose config enables `options.typeAware`.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_type_aware_forced(mut self, yes: bool) -> Self {
        self.type_aware_forced = yes;
        self
    }

    /// Whether a file governed by `config` should be linted with type-aware rules.
    fn is_type_aware(&self, config: &LinterConfig) -> bool {
        self.type_aware_forced || config.type_aware().unwrap_or(false)
    }

    /// Whether a file governed by `config` should report TypeScript compiler diagnostics.
    fn is_type_check(&self, config: &LinterConfig) -> bool {
        self.type_check_override.unwrap_or_else(|| config.type_check().unwrap_or(false))
    }

    /// Whether nothing was resolved for the working directory, so that every file depends on
    /// what is installed next to it.
    ///
    /// That is not fatal: in a monorepo, `tsgolint` is often installed only in the packages, so
    /// files may still resolve to an executable of their own. Those which do not are reported,
    /// see `missing_executable_diagnostic`.
    pub fn has_no_fallback(&self) -> bool {
        self.fallback.executable().is_none()
    }

    /// Record, for every file in `paths`, whether its type-aware rules ran.
    ///
    /// A directive naming a rule which never ran is not unused, and neither is a suppression
    /// recorded for it; both would otherwise be reported and offered for deletion. Files which
    /// were linted clear any earlier record, because the editor lints the same file again and
    /// again and `tsgolint` may have appeared in between.
    fn record_type_aware_coverage(
        &self,
        directives_store: &DirectivesStore,
        linted: &[Arc<OsStr>],
        not_linted: &[Arc<OsStr>],
    ) {
        for path in not_linted {
            let file = Path::new(path.as_ref());
            directives_store
                .set_rules_not_run(file.to_path_buf(), self.type_aware_directive_names(file));
        }
        for path in linted {
            let file = Path::new(path.as_ref());
            directives_store.set_rules_not_run(file.to_path_buf(), FxHashSet::default());
        }
    }

    /// Every name a disable directive may use for the type-aware rules which govern `file`.
    ///
    /// A user writes whichever spelling their editor suggested, and
    /// [`should_skip_diagnostic`] accepts all of them, so the set which must not be reported as
    /// unused has to accept all of them too.
    pub(crate) fn type_aware_directive_names(&self, file: &Path) -> FxHashSet<String> {
        self.config_store
            .resolve(file)
            .rules
            .iter()
            .filter(|(rule, status)| status.is_warn_deny() && rule.is_tsgolint_rule())
            .flat_map(|(rule, _)| {
                let name = rule.name();
                [
                    name.to_string(),
                    format!("{}/{name}", rule.plugin_name()),
                    format!("typescript-eslint/{name}"),
                    format!("@typescript-eslint/{name}"),
                ]
            })
            .collect()
    }

    /// The suppression keys of the rules `tsgolint` would have run on `file`.
    ///
    /// Only the rules configured at [`AllowWarnDeny::Deny`] are included: the suppression
    /// tracker only counts error-severity diagnostics, so a rule at `warn` never has a
    /// suppression to preserve. This is computed only for the files which actually have
    /// recorded suppressions, which is why the config is resolved here.
    fn type_aware_suppression_keys(&self, file: &Path) -> FxHashSet<String> {
        self.config_store
            .resolve(file)
            .rules
            .iter()
            .filter(|(rule, status)| *status == AllowWarnDeny::Deny && rule.is_tsgolint_rule())
            .map(|(rule, _)| format!("{}/{}", rule.plugin_name(), rule.name()))
            .collect()
    }

    /// Plan a `tsgolint` run, failing when not a single file can be linted with type-aware
    /// rules. A run which requested them must not pass, and the failure is raised before
    /// anything is linted, while the diagnostic channel is still empty.
    ///
    /// # Errors
    /// Returns an error if no `tsgolint` executable could be found for any of the files.
    pub fn try_plan(&self, paths: &[Arc<OsStr>]) -> Result<LintPlan, String> {
        let plan = self.plan(paths);
        if plan.groups.is_empty() && !plan.unresolved.is_empty() {
            return Err(no_executable_error(plan.unresolved.len()));
        }
        Ok(plan)
    }

    /// Plan a `tsgolint` run: keep the files `tsgolint` has to lint and group them by the
    /// installation which should lint them.
    ///
    /// Everything is computed in a single pass: whether a file is linted at all (the same
    /// filtering as [`TsGoLintState::json_input`], which therefore does not repeat it) and which
    /// installation it belongs to.
    fn plan(&self, paths: &[Arc<OsStr>]) -> LintPlan {
        // A recorded executable can have been uninstalled since it was recorded, so it is
        // checked before being reused; once per distinct executable, not once per file.
        let mut installed: FxHashMap<PathBuf, bool> = FxHashMap::default();

        group_paths_by_executable(
            paths,
            |file| {
                SourceType::from_path(file).is_ok()
                    && self.is_type_aware(self.config_store.get_related_config(file))
            },
            |dir| self.resolve_executable_for_dir(dir, &mut installed),
        )
    }

    /// Resolve the `tsgolint` executable which should lint the files in `dir`.
    ///
    /// The lookup starts at `dir` and walks up to [`Self::cwd`], so that a package which
    /// installs its own `tsgolint` uses that one, and falls back to the executable resolved for
    /// [`Self::cwd`]. Every directory the walk looked at is cached, so the siblings of an
    /// already resolved directory cost nothing.
    fn resolve_executable_for_dir(
        &self,
        dir: &Path,
        installed: &mut FxHashMap<PathBuf, bool>,
    ) -> Option<ResolvedExecutable> {
        if self.fallback.is_pinned() {
            return self.fallback.executable().cloned();
        }

        let mut still_installed =
            |path: &Path| *installed.entry(path.to_path_buf()).or_insert_with(|| path.exists());

        self.executable_cache
            .resolve(
                dir,
                walk_boundary_for(dir, &self.cwd),
                Instant::now(),
                look_in_node_modules,
                &mut still_installed,
            )
            .or_else(|| self.fallback.executable().cloned())
    }

    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    pub fn lint(
        self,
        plan: LintPlan,
        directives_store: &DirectivesStore,
        error_sender: DiagnosticSender,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        diff_manager: &Arc<DiffManager>,
        rule_timing_store: Option<&RuleTimingStore>,
    ) -> Result<(), String> {
        let LintPlan { groups, unresolved } = plan;
        if groups.is_empty() && unresolved.is_empty() {
            return Ok(());
        }

        // The files which could not be linted keep the suppressions recorded for their
        // type-aware rules, instead of looking like suppressions which no longer fire.
        for path in &unresolved {
            let file = Path::new(path.as_ref());
            diff_manager.collect_type_aware_skipped_file(file, &self.cwd, || {
                self.type_aware_suppression_keys(file)
            });
        }

        let disable_directives_map = &directives_store.map();
        let results = run_groups(&groups, |group| {
            self.lint_group(
                group,
                disable_directives_map,
                error_sender.clone(),
                file_system,
                diff_manager,
                rule_timing_store,
            )
        });

        let (failed, linted): (Vec<_>, Vec<_>) =
            results.iter().zip(&groups).partition(|(result, _)| result.is_err());
        let mut not_linted = unresolved.clone();
        not_linted.extend(failed.iter().flat_map(|(_, group)| group.paths.iter().cloned()));
        let linted =
            linted.iter().flat_map(|(_, group)| group.paths.iter().cloned()).collect::<Vec<_>>();
        self.record_type_aware_coverage(directives_store, &linted, &not_linted);

        // Reported once, after the groups which could run did, so that a missing executable
        // never hides the diagnostics of the packages which do have one.
        if let Some(first) = unresolved.first() {
            // Best effort: the receiver is only gone when the run is already being torn down.
            let _ = error_sender.send(attach_to_file(
                missing_executable_diagnostic(&unresolved, &self.cwd),
                Path::new(first.as_ref()),
                &self.cwd,
            ));
        }

        // A group which could not run is reported here rather than left to the caller, because
        // this is where the files it was meant to lint are still known.
        for (group, error) in results
            .iter()
            .zip(&groups)
            .filter_map(|(result, group)| result.as_ref().err().map(|error| (group, error)))
        {
            let Some(first) = group.paths.first() else { continue };
            let _ = error_sender.send(attach_to_file(
                group_failure_diagnostic(group, error),
                Path::new(first.as_ref()),
                &self.cwd,
            ));
        }

        drop(error_sender);

        // Every group was run; report all the ones which failed, not just the first.
        join_group_errors(&groups, results)
    }

    /// Lint `paths` with a single `tsgolint` executable.
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    fn lint_group(
        &self,
        group: &LintGroup,
        disable_directives_map: &Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
        error_sender: DiagnosticSender,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        diff_manager: &Arc<DiffManager>,
        rule_timing_store: Option<&RuleTimingStore>,
    ) -> Result<(), String> {
        let LintGroup { executable, paths } = group;
        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();

        let json_input = self.json_input(paths, None, &mut resolved_configs);
        if json_input.configs.is_empty() {
            return Ok(());
        }

        // Already filtered by `TsGoLintState::plan`.
        let all_paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

        let should_fix = self.fix || self.fix_suggestions;
        let cwd = self.cwd.clone();
        let handler_cwd = self.cwd.clone();
        let silent = self.silent;
        let sender_for_fixes = error_sender.clone();

        let diff_manager_clone_to_ts_go = Arc::<DiffManager>::clone(diff_manager);
        let disable_directives_map = Arc::clone(disable_directives_map);

        let mut child = self.spawn_tsgolint(executable, &json_input)?;

        let handler = std::thread::spawn(move || {
            let Some(stdout) = child.stdout.take() else {
                return Err("could not open tsgolint stdout".to_string());
            };

            // Process stdout stream in a separate thread to send diagnostics as they arrive
            let stdout_handler = std::thread::spawn(move || -> Result<TsGoLintOutput, String> {
                let mut diagnostic_handler =
                    DiagnosticHandler::new(handler_cwd, silent, should_fix, error_sender);
                let mut timings = vec![];

                let msg_iter = TsGoLintMessageStream::new(stdout);

                for msg in msg_iter {
                    match msg {
                        Ok(TsGoLintMessage::Error(err)) => {
                            return Err(err.error);
                        }
                        Ok(TsGoLintMessage::Diagnostic(tsgolint_diagnostic)) => {
                            match tsgolint_diagnostic {
                                TsGoLintDiagnostic::Rule(tsgolint_diagnostic) => {
                                    let path = &tsgolint_diagnostic.file_path;

                                    let severity = resolved_configs
                                            .get(path)
                                            .or_else(|| {
                                                debug_assert!(false, "resolved_configs should have an entry for every file we linted {tsgolint_diagnostic:?}");
                                                None
                                            })
                                            .and_then(|resolved_config| {
                                                resolved_config
                                                    .rules
                                                    .iter()
                                                    .find(|(rule, _)| {
                                                        rule.name() == tsgolint_diagnostic.rule
                                                    })
                                                    .map(|(_, status)| *status)
                                            })
                                            .or_else(|| {
                                                debug_assert!(false, "resolved_config should have a matching rule for every diagnostic we received {tsgolint_diagnostic:?}");
                                                None
                                            });
                                    let Some(severity) = severity else {
                                        // If the severity is not found, we should not report
                                        // the diagnostic
                                        continue;
                                    };

                                    // Locked per diagnostic: holding it for the whole stream
                                    // would serialize the groups against each other.
                                    if should_skip_diagnostic(
                                        &lock(&disable_directives_map),
                                        path,
                                        &tsgolint_diagnostic,
                                    ) {
                                        continue;
                                    }

                                    diagnostic_handler.handle_rule_diagnostic(
                                        tsgolint_diagnostic,
                                        severity,
                                        diff_manager_clone_to_ts_go.skip(),
                                    );
                                }
                                TsGoLintDiagnostic::Internal(e) => {
                                    diagnostic_handler.handle_internal_diagnostic(e);
                                }
                            }
                        }
                        Ok(TsGoLintMessage::Timing(payload)) => {
                            timings.extend(payload.rules);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(TsGoLintOutput {
                    messages_requiring_fixes: diagnostic_handler
                        .into_messages_requiring_fixes(&diff_manager_clone_to_ts_go, all_paths),
                    timings,
                })
            });

            // Wait for process to complete and stdout processing to finish
            let exit_status =
                child.wait().map_err(|error| format!("could not await tsgolint: {error}"))?;
            let stdout_result = stdout_handler.join();

            if !exit_status.success() {
                return Err(
                    if let Some(err) = &stdout_result.ok().and_then(std::result::Result::err) {
                        format!("{exit_status}, error: {err}")
                    } else {
                        format!("{exit_status}")
                    },
                );
            }

            match stdout_result {
                Ok(Ok(messages)) => Ok(messages),
                Ok(Err(err)) => Err(format!("{exit_status}, error: {err}")),
                Err(_) => Err("Failed to join stdout processing thread".to_string()),
            }
        });

        match handler.join() {
            Ok(Ok(output)) => {
                let TsGoLintOutput { messages_requiring_fixes, timings } = output;
                if let Some(rule_timing_store) = rule_timing_store {
                    rule_timing_store.merge(timings.into_iter().map(|timing| RuleTimingRecord {
                        source: RuleTimingSource::TypeAware,
                        plugin_name: "typescript".to_string(),
                        rule_name: timing.rule_name,
                        duration: Duration::from_nanos(timing.duration),
                        calls: timing.calls,
                    }));
                }

                for (path, source_text, messages) in messages_requiring_fixes {
                    let source_type = SourceType::from_path(&path)
                        .ok()
                        .map(|st| if st.is_javascript() { st.with_jsx(true) } else { st });
                    let fix_result = Fixer::new(&source_text, messages, source_type).fix();

                    if fix_result.fixed
                        && let Err(error) = file_system.write_file(&path, &fix_result.fixed_code)
                    {
                        let _ = sender_for_fixes.send(vec![
                            OxcDiagnostic::error(format!(
                                "Failed to write file {} with error \"{error}\"",
                                path.display()
                            ))
                            .into(),
                        ]);
                    }

                    if fix_result.messages.is_empty() {
                        diff_manager.collect_empty_file(path.as_path(), &cwd);
                    } else {
                        let source_for_diagnostics: &str =
                            if fix_result.fixed { &fix_result.fixed_code } else { &source_text };

                        let filtered_messages: Vec<OxcDiagnostic> = if diff_manager.skip() {
                            fix_result.messages.into_iter().map(Into::into).collect()
                        } else {
                            diff_manager
                                .collect_file(
                                    path.as_path(),
                                    &cwd,
                                    fix_result.messages.into_iter().collect(),
                                )
                                .into_iter()
                                .map(Into::into)
                                .collect()
                        };

                        let diagnostics = DiagnosticService::wrap_diagnostics(
                            &cwd,
                            &path,
                            source_for_diagnostics,
                            filtered_messages,
                        );
                        let _ = sender_for_fixes.send(diagnostics);
                    }
                }
                Ok(())
            }
            Ok(Err(err)) => Err(format!("Error running tsgolint: {err}")),
            Err(err) => Err(format!("Error running tsgolint: {err:?}")),
        }
    }

    /// Spawn the tsgolint process with the given input.
    fn spawn_tsgolint(
        &self,
        executable: &Path,
        json_input: &Payload,
    ) -> Result<std::process::Child, String> {
        let mut cmd = std::process::Command::new(executable);
        cmd.arg("headless")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(stderr());

        if self.timings {
            cmd.args(["-debug", "timings"]);
        }

        if self.fix {
            cmd.arg("-fix");
        }

        if self.fix_suggestions {
            cmd.arg("-fix-suggestions");
        }

        if let Ok(trace_file) = std::env::var("OXLINT_TSGOLINT_TRACE") {
            cmd.arg(format!("-trace={trace_file}"));
        }
        if let Ok(cpuprof_file) = std::env::var("OXLINT_TSGOLINT_CPU") {
            cmd.arg(format!("-cpuprof={cpuprof_file}"));
        }
        if let Ok(heap_file) = std::env::var("OXLINT_TSGOLINT_HEAP") {
            cmd.arg(format!("-heap={heap_file}"));
        }
        if let Ok(allocs_file) = std::env::var("OXLINT_TSGOLINT_ALLOCS") {
            cmd.arg(format!("-allocs={allocs_file}"));
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return Err(format!(
                    "Failed to spawn tsgolint from path `{}`, with error: {e}",
                    executable.display()
                ));
            }
        };

        let Some(mut stdin) = child.stdin.take() else {
            return Err("could not open tsgolint stdin".to_string());
        };

        let json = serde_json::to_string(json_input)
            .map_err(|error| format!("could not serialize the tsgolint payload: {error}"))?;
        if let Err(e) = stdin.write_all(json.as_bytes())
            && e.kind() != ErrorKind::BrokenPipe
        {
            return Err(format!("Failed to write to tsgolint stdin: {e}"));
        }
        drop(stdin);

        Ok(child)
    }

    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    pub fn lint_source(
        &self,
        paths: &[Arc<OsStr>],
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        directives_store: &DirectivesStore,
    ) -> Result<Vec<Message>, String> {
        // Each package may pin its own `tsgolint` version, so run one process per distinct
        // installation, each one receiving only the files it is responsible for.
        let LintPlan { groups, unresolved } = self.plan(paths);
        if groups.is_empty() && unresolved.is_empty() {
            return Ok(vec![]);
        }

        let disable_directives_map = &directives_store.map();
        let results = run_groups(&groups, |group| {
            self.lint_source_group(group, file_system, disable_directives_map)
        });

        // Merged in group order, which `plan` made deterministic.
        let mut messages = vec![];
        let mut failures = Vec::new();
        let mut linted = Vec::new();
        let mut not_linted = unresolved.clone();
        for (result, group) in results.into_iter().zip(&groups) {
            match result {
                Ok(group_messages) => {
                    messages.extend(group_messages);
                    linted.extend(group.paths.iter().cloned());
                }
                Err(error) => {
                    not_linted.extend(group.paths.iter().cloned());
                    failures.push((group, error));
                }
            }
        }
        self.record_type_aware_coverage(directives_store, &linted, &not_linted);

        // The run only fails when there is nothing to show: one broken installation must not
        // remove the diagnostics of the packages which do work, so its failure is reported as a
        // warning on the files it was meant to lint. With no group at all there is nothing
        // which failed, only files to warn about.
        if !groups.is_empty() && failures.len() == groups.len() {
            return Err(failures
                .iter()
                .map(|(group, error)| format!("{}: {error}", group.executable.display()))
                .collect::<Vec<_>>()
                .join("\n"));
        }
        messages.extend(failures.iter().map(|(group, error)| group_failure_message(group, error)));

        // The regular rules ran and are reported as usual; the files without an executable get
        // a warning of their own.
        messages.extend(missing_executable_messages(&unresolved));

        Ok(messages)
    }

    /// Lint `paths` from their in-memory sources with a single `tsgolint` executable.
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    fn lint_source_group(
        &self,
        group: &LintGroup,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        disable_directives_map: &Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
    ) -> Result<Vec<Message>, String> {
        let LintGroup { executable, paths } = group;
        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();
        let mut source_overrides: FxHashMap<String, String> = FxHashMap::default();
        let allocator = Allocator::default();

        // Read all sources into overrides
        for path in paths {
            let path_ref = Path::new(path.as_ref());
            let Ok(source_text) = file_system.read_to_arena_str(path_ref, &allocator) else {
                return Err(format!(
                    "Failed to read source text for file: {}",
                    path.to_string_lossy()
                ));
            };
            source_overrides.insert(path.to_string_lossy().to_string(), source_text.to_string());
        }

        let json_input =
            self.json_input(paths, Some(source_overrides.clone()), &mut resolved_configs);
        if json_input.configs.is_empty() {
            // No file in this batch is governed by a config which enables type-aware linting.
            return Ok(vec![]);
        }

        //  Get the file name of the first path for internal diagnostic filtering
        let path_file_name =
            Path::new(paths[0].as_ref()).file_name().unwrap_or_default().to_os_string();

        let mut child = self.spawn_tsgolint(executable, &json_input)?;
        let Some(stdout) = child.stdout.take() else {
            return Err("could not open tsgolint stdout".to_string());
        };
        let diagnostics = (|| -> Result<Vec<Message>, String> {
            let msg_iter = TsGoLintMessageStream::new(stdout);
            let mut result = vec![];

            for msg in msg_iter {
                match msg {
                    Ok(TsGoLintMessage::Error(err)) => {
                        return Err(err.error);
                    }
                    Ok(TsGoLintMessage::Diagnostic(tsgolint_diagnostic)) => {
                        match tsgolint_diagnostic {
                            TsGoLintDiagnostic::Rule(tsgolint_diagnostic) => {
                                let path = tsgolint_diagnostic.file_path.clone();
                                let Some(resolved_config) = resolved_configs.get(&path) else {
                                    // If we don't have a resolved config for this path, skip it. We should always
                                    // have a resolved config though, since we processed them already above.
                                    continue;
                                };

                                let severity =
                                    resolved_config.rules.iter().find_map(|(rule, status)| {
                                        if rule.name() == tsgolint_diagnostic.rule {
                                            Some(*status)
                                        } else {
                                            None
                                        }
                                    });
                                let Some(severity) = severity else {
                                    // If the severity is not found, we should not report the diagnostic
                                    continue;
                                };

                                if should_skip_diagnostic(
                                    &lock(disable_directives_map),
                                    &path,
                                    &tsgolint_diagnostic,
                                ) {
                                    continue;
                                }

                                // Use the corresponding source override text
                                let Some(source_text_owned) = source_overrides
                                    .get(&path.to_string_lossy().to_string())
                                    .cloned()
                                else {
                                    // should never happen, because we populated source_overrides above
                                    continue;
                                };

                                let mut message = Message::from_tsgo_lint_diagnostic(
                                    tsgolint_diagnostic,
                                    &source_text_owned,
                                );

                                if self.with_ignore_fixes {
                                    message.add_ignore_fix(0, &source_text_owned);
                                }

                                message.error.severity = if severity == AllowWarnDeny::Deny {
                                    Severity::Error
                                } else {
                                    Severity::Warning
                                };

                                result.push(message);
                            }
                            TsGoLintDiagnostic::Internal(e) => {
                                let span = e
                                    .file_path
                                    .as_ref()
                                    .is_some_and(|f| {
                                        f.file_name().unwrap_or_default() == path_file_name
                                    })
                                    .then_some(e.span)
                                    .flatten()
                                    .unwrap_or_default();
                                let mut diagnostic: OxcDiagnostic = e.into();
                                diagnostic = diagnostic.with_label(span);
                                result.push(Message::new(diagnostic, PossibleFixes::None));
                            }
                        }
                    }
                    Ok(TsGoLintMessage::Timing(_)) => {}
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Ok(result)
        })();

        // Kill the child process if it's still running to avoid zombie processes
        let _ = child.kill();
        let _ = child.wait();

        diagnostics
    }

    /// Create a JSON input for STDIN of tsgolint.
    ///
    /// # Payload semantics
    ///
    /// The payload is a list of config groups, each listing the absolute paths of the files it
    /// applies to, the rules to run on them, and an optional `type_check` flag. `tsgolint`
    /// interprets them as follows, and `oxlint` is written against that contract:
    ///
    /// 1. A payload where `type_check` is absent from every group behaves exactly like it did
    ///    before the field existed.
    /// 2. The last group listing a file defines that file's whole config, its rules and its
    ///    `type_check` together; there is no inheritance between groups. A `type_check` absent
    ///    from that group means `true`. `oxlint` never lists a file in two groups, and always
    ///    sends the field rather than relying on that default, so the payload says outright which
    ///    files are type-checked.
    /// 3. `type_check: false` excludes that group's files from every TypeScript diagnostic
    ///    attached to them, syntactic and parse ones included. Type-aware *rules* still run on
    ///    those files; only the compiler's own diagnostics are dropped.
    /// 4. `report_syntactic` and `report_semantic` stay run-wide selectors for which *kinds* of
    ///    diagnostic are reported, and `type_check` selects which *files* report them. So
    ///    `type_check: true` is a no-op unless at least one of the two is set, which is why
    ///    `oxlint` sets both as soon as a single emitted file is type-checked.
    /// 5. A config group is identified by `(rules, type_check)`: files whose `type_check` differs
    ///    must never share a group, even when their rules are identical, because a group carries
    ///    exactly one value of the field.
    ///
    /// ```json
    /// {
    ///   "version": 2,
    ///   "configs": [
    ///     {
    ///       "file_paths": ["/absolute/path/to/file.ts"],
    ///       "rules": [{ "name": "no-floating-promises" }],
    ///       "type_check": true
    ///     }
    ///   ],
    ///   "report_syntactic": true,
    ///   "report_semantic": true
    /// }
    /// ```
    #[inline]
    fn json_input(
        &self,
        paths: &[Arc<OsStr>],
        source_overrides: Option<FxHashMap<String, String>>,
        resolved_configs: &mut FxHashMap<PathBuf, ResolvedLinterState>,
    ) -> Payload {
        // Keyed by `(rules, type_check)`: a group carries a single `type_check` value, so files
        // which disagree about it stay in separate groups even with identical rules.
        let mut config_groups: FxHashMap<(BTreeSet<Rule>, bool), Vec<String>> =
            FxHashMap::default();

        // `paths` is already filtered by `TsGoLintState::plan`: every entry has a recognized
        // source type and is governed by a config which enables type-aware linting.
        for path in paths {
            let path_buf = PathBuf::from(path);
            // Finding the config which governs a file walks its ancestors, so look it up once:
            // `typeCheck` and the overrides both come from that one config.
            let config = self.config_store.get_related_config(&path_buf);
            let type_check = self.is_type_check(config);
            let file_path = path.to_string_lossy().to_string();

            let resolved_config = resolved_configs
                .entry(path_buf.clone())
                .or_insert_with(|| config.apply_overrides(&path_buf));

            let rules: BTreeSet<Rule> = resolved_config
                .rules
                .iter()
                .filter_map(|(rule, status)| {
                    if status.is_warn_deny() && rule.is_tsgolint_rule() {
                        let rule_name = rule.name().to_string();
                        let options = match rule.to_configuration() {
                            Some(Ok(config)) => Some(config),
                            Some(Err(_)) | None => None,
                        };
                        Some(Rule { name: rule_name, options })
                    } else {
                        None
                    }
                })
                .collect();

            config_groups.entry((rules, type_check)).or_default().push(file_path);
        }

        // `type_check` only selects the files, never the kinds of diagnostic, so it is a no-op
        // unless the run also enables a kind. Both are enabled as soon as one file is
        // type-checked.
        let any_type_check = config_groups.keys().any(|(_, type_check)| *type_check);

        Payload {
            version: 2,
            configs: config_groups
                .into_iter()
                .map(|((rules, type_check), file_paths)| Config {
                    file_paths,
                    rules: rules.into_iter().collect(),
                    type_check,
                })
                .collect(),
            source_overrides,
            report_syntactic: any_type_check,
            report_semantic: any_type_check,
        }
    }
}

/// Represents the input JSON to `tsgolint`, like:
///
/// ```json
/// {
///   "version": 2,
///   "configs": [
///     {
///       "file_paths": ["/absolute/path/to/file.ts", "/another/file.ts"],
///       "rules": [
///         { "name": "rule-1" },
///         { "name": "another-rule" },
///       ],
///       "type_check": true
///     }
///   ]
/// }
/// ```
///
/// See [`TsGoLintState::json_input`] for how `tsgolint` interprets the fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub version: i32,
    pub configs: Vec<Config>,
    pub source_overrides: Option<FxHashMap<String, String>>,
    pub report_syntactic: bool,
    pub report_semantic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Absolute path to the file to lint
    pub file_paths: Vec<String>,
    /// List of rules to apply to this file
    /// Example: `["no-floating-promises"]`
    pub rules: Vec<Rule>,
    /// Whether this group's files report TypeScript compiler diagnostics.
    ///
    /// Always serialized: `tsgolint` treats an absent `type_check` as `true`, so leaving it out
    /// would type-check a group which set it to `false`. See [`TsGoLintState::json_input`].
    #[serde(default = "type_check_default")]
    pub type_check: bool,
}

/// `tsgolint` type-checks a config group whose `type_check` is absent.
fn type_check_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by name
        match self.name.cmp(&other.name) {
            std::cmp::Ordering::Equal => {
                // If names are equal, compare by serialized options
                // Serialize to canonical JSON string for comparison
                let self_options = self.options.as_ref().map(|v| serde_json::to_string(v).ok());
                let other_options = other.options.as_ref().map(|v| serde_json::to_string(v).ok());
                self_options.cmp(&other_options)
            }
            other_ordering => other_ordering,
        }
    }
}

/// Diagnostic kind discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosticKind {
    Rule = 0,
    Internal = 1,
}

impl Serialize for DiagnosticKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for DiagnosticKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(DiagnosticKind::Rule),
            1 => Ok(DiagnosticKind::Internal),
            _ => Err(serde::de::Error::custom(format!("Invalid DiagnosticKind value: {value}"))),
        }
    }
}

/// Represents the raw output binary data from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsGoLintDiagnosticPayload {
    pub kind: DiagnosticKind,
    pub range: Option<Range>,
    pub message: RuleMessage,
    pub file_path: Option<String>,
    // Only for kind="rule"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<Suggestion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labeled_ranges: Vec<LabeledRange>,
}

/// Represents the error payload from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsGoLintErrorPayload {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsGoLintTimingPayload {
    pub rules: Vec<TsGoLintRuleTiming>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsGoLintRuleTiming {
    pub rule_name: String,
    pub duration: u64,
    pub calls: u64,
}

#[derive(Debug, Clone)]
pub enum TsGoLintMessage {
    Diagnostic(TsGoLintDiagnostic),
    Error(TsGoLintError),
    Timing(TsGoLintTimingPayload),
}

#[derive(Debug, Clone)]
pub enum TsGoLintDiagnostic {
    Rule(TsGoLintRuleDiagnostic),
    Internal(TsGoLintInternalDiagnostic),
}

#[derive(Debug, Clone)]
pub struct TsGoLintRuleDiagnostic {
    pub span: Span,
    pub rule: String,
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
    pub suggestions: Vec<Suggestion>,
    pub labeled_ranges: Vec<LabeledRange>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TsGoLintInternalDiagnostic {
    pub message: RuleMessage,
    pub span: Option<Span>,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TsGoLintError {
    pub error: String,
}

impl From<TsGoLintDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintDiagnostic) -> Self {
        match val {
            TsGoLintDiagnostic::Rule(d) => d.into(),
            TsGoLintDiagnostic::Internal(d) => d.into(),
        }
    }
}

impl From<TsGoLintRuleDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintRuleDiagnostic) -> Self {
        let mut d = OxcDiagnostic::warn(val.message.description)
            .with_url(format!("{}/{}/{}.html", WEBSITE_BASE_RULES_URL, "typescript", val.rule))
            .with_error_code(TSGOLINT_DIAGNOSTIC_SCOPE, val.rule);
        if let Some(help) = val.message.help {
            d = d.with_help(help);
        }
        if val.labeled_ranges.is_empty() {
            d = d.with_label(val.span);
        } else {
            let labels = val
                .labeled_ranges
                .into_iter()
                .map(|lr| Span::new(lr.range.pos, lr.range.end).label(lr.label));
            d = d.with_labels(labels);
            // If the main span is empty, don't add it as a label since it doesn't have any meaning (means tsgolint sent nothing).
            // Just use the labeled ranges that were passed instead.
            if !val.span.is_unspanned() {
                d = d.and_label(val.span.primary());
            }
        }
        d
    }
}
impl From<TsGoLintInternalDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintInternalDiagnostic) -> Self {
        let mut d = OxcDiagnostic::error(val.message.description)
            .with_error_code(TSGOLINT_DIAGNOSTIC_SCOPE, val.message.id);
        if let Some(help) = val.message.help {
            d = d.with_help(help);
        }
        if val.file_path.is_some()
            && let Some(span) = val.span
        {
            d = d.with_label(span);
        }
        d
    }
}

impl Message {
    /// Converts a `TsGoLintDiagnostic` into a `Message` with possible fixes.
    fn from_tsgo_lint_diagnostic(mut val: TsGoLintRuleDiagnostic, source_text: &str) -> Self {
        let fix = if val.fixes.is_empty() {
            None
        } else {
            let fix_vec = mem::take(&mut val.fixes)
                .into_iter()
                .map(|fix| crate::fixer::Fix {
                    content: Cow::Owned(fix.text),
                    span: Span::new(fix.range.pos, fix.range.end),
                    message: None,
                    kind: FixKind::Fix,
                })
                .collect();

            Some(CompositeFix::merge_fixes(fix_vec, source_text))
        };

        let suggestions = mem::take(&mut val.suggestions).into_iter().map(|suggestion| {
            let fix_vec = suggestion
                .fixes
                .into_iter()
                .map(|fix| crate::fixer::Fix {
                    content: Cow::Owned(fix.text),
                    span: Span::new(fix.range.pos, fix.range.end),
                    message: None,
                    kind: FixKind::Suggestion,
                })
                .collect();

            CompositeFix::merge_fixes(fix_vec, source_text)
                .with_message(suggestion.message.description)
        });

        let possible_fixes = PossibleFixes::from_iter(iter::chain(fix, suggestions));

        Self::new(val.into(), possible_fixes)
    }
}

// TODO: Should this be removed and replaced with a `Span`?
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Range {
    pub pos: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMessage {
    pub id: String,
    pub description: String,
    pub help: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fix {
    pub text: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabeledRange {
    pub label: String,
    pub range: Range,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    Error = 0,
    Diagnostic = 1,
    Timing = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidMessageType(pub u8);

impl std::fmt::Display for InvalidMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid message type: {}", self.0)
    }
}

impl std::error::Error for InvalidMessageType {}

impl TryFrom<u8> for MessageType {
    type Error = InvalidMessageType;

    fn try_from(value: u8) -> Result<Self, InvalidMessageType> {
        match value {
            0 => Ok(Self::Error),
            1 => Ok(Self::Diagnostic),
            2 => Ok(Self::Timing),
            _ => Err(InvalidMessageType(value)),
        }
    }
}

/// The largest message `tsgolint` may announce, 256 MiB.
///
/// Far beyond any real diagnostic, and small enough that a desynchronized stream fails instead
/// of allocating the size its four header bytes announce.
const MAX_MESSAGE_SIZE: usize = 256 * 1024 * 1024;

/// Iterator that streams messages from tsgolint stdout.
struct TsGoLintMessageStream {
    stdout: std::process::ChildStdout,
    buffer: Vec<u8>,
}

impl TsGoLintMessageStream {
    fn new(stdout: std::process::ChildStdout) -> TsGoLintMessageStream {
        TsGoLintMessageStream { stdout, buffer: Vec::with_capacity(8192) }
    }
}

impl Iterator for TsGoLintMessageStream {
    type Item = Result<TsGoLintMessage, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read_buf = [0u8; 8192];

        loop {
            // Try to parse a complete message from the existing buffer
            let mut cursor = std::io::Cursor::new(self.buffer.as_slice());

            if cursor.position() < self.buffer.len() as u64 {
                match parse_single_message(&mut cursor) {
                    Ok(message) => {
                        // Successfully parsed a message, remove it from buffer
                        #[expect(clippy::cast_possible_truncation)]
                        self.buffer.drain(..cursor.position() as usize);
                        return Some(Ok(message));
                    }
                    Err(TsGoLintMessageParseError::IncompleteData) => {}
                    Err(e) => {
                        return Some(Err(e.to_string()));
                    }
                }
            }

            // Read more data from stdout
            match self.stdout.read(&mut read_buf) {
                Ok(0) => {
                    return None;
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&read_buf[..n]);
                }
                Err(e) => {
                    return Some(Err(format!("Failed to read from tsgolint stdout: {e}")));
                }
            }
        }
    }
}

enum TsGoLintMessageParseError {
    IncompleteData,
    InvalidMessageType(InvalidMessageType),
    InvalidErrorPayload(serde_json::Error),
    InvalidDiagnosticPayload(serde_json::Error),
    InvalidTimingPayload(serde_json::Error),
    /// The announced payload is larger than anything `tsgolint` could legitimately send, so the
    /// stream is out of sync rather than large.
    PayloadTooLarge(usize),
    /// A rule diagnostic without the field which makes it one.
    IncompleteRuleDiagnostic(&'static str),
}

impl std::fmt::Display for TsGoLintMessageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsGoLintMessageParseError::IncompleteData => write!(f, "Incomplete data"),
            TsGoLintMessageParseError::InvalidMessageType(e) => write!(f, "{e}"),
            TsGoLintMessageParseError::InvalidErrorPayload(e) => {
                write!(f, "Failed to parse tsgolint error payload: {e}")
            }
            TsGoLintMessageParseError::InvalidDiagnosticPayload(e) => {
                write!(f, "Failed to parse tsgolint diagnostic payload: {e}")
            }
            TsGoLintMessageParseError::PayloadTooLarge(size) => {
                write!(f, "tsgolint announced a {size} byte message, which cannot be right")
            }
            TsGoLintMessageParseError::IncompleteRuleDiagnostic(field) => {
                write!(f, "tsgolint sent a rule diagnostic without its `{field}`")
            }
            TsGoLintMessageParseError::InvalidTimingPayload(e) => {
                write!(f, "Failed to parse tsgolint timing payload: {e}")
            }
        }
    }
}

/// Cache for source text to avoid reading the same file multiple times.
#[derive(Default)]
struct SourceTextCache(FxHashMap<PathBuf, String>);

impl SourceTextCache {
    fn get_or_insert(&mut self, path: &Path) -> &str {
        self.0
            .entry(path.to_path_buf())
            .or_insert_with(|| read_to_string(path).unwrap_or_default())
            .as_str()
    }
}

struct TsGoLintOutput {
    messages_requiring_fixes: Vec<(PathBuf, String, Vec<Message>)>,
    timings: Vec<TsGoLintRuleTiming>,
}

/// Handles streaming and collecting diagnostics from tsgolint.
struct DiagnosticHandler {
    cwd: PathBuf,
    silent: bool,
    should_fix: bool,
    source_text_cache: SourceTextCache,
    error_sender: DiagnosticSender,
    /// Messages requiring fixes, grouped by file path: messages.
    messages_requiring_fixes: FxHashMap<PathBuf, Vec<Message>>,
    messages_not_requiring_fixes: FxHashMap<PathBuf, Vec<(TsGoLintRuleDiagnostic, AllowWarnDeny)>>,
}

impl DiagnosticHandler {
    fn new(cwd: PathBuf, silent: bool, should_fix: bool, error_sender: DiagnosticSender) -> Self {
        Self {
            cwd,
            silent,
            should_fix,
            source_text_cache: SourceTextCache::default(),
            error_sender,
            messages_requiring_fixes: FxHashMap::default(),
            messages_not_requiring_fixes: FxHashMap::default(),
        }
    }

    fn get_source_text(&mut self, path: &Path) -> &str {
        if self.silent && !self.should_fix {
            // The source text is not needed in silent mode, the diagnostic isn't printed.
            ""
        } else {
            self.source_text_cache.get_or_insert(path)
        }
    }

    fn handle_rule_diagnostic(
        &mut self,
        diagnostic: TsGoLintRuleDiagnostic,
        severity: AllowWarnDeny,
        ignore_suppression: bool,
    ) {
        let path = diagnostic.file_path.clone();
        let has_fixes =
            self.should_fix && (!diagnostic.fixes.is_empty() || !diagnostic.suggestions.is_empty());

        if has_fixes {
            // Collect for later fix application
            let mut message =
                Message::from_tsgo_lint_diagnostic(diagnostic, self.get_source_text(&path));
            message.error.severity =
                if severity == AllowWarnDeny::Deny { Severity::Error } else { Severity::Warning };

            let entry = self.messages_requiring_fixes.entry(path).or_default();

            entry.push(message);
        } else if !ignore_suppression {
            let entry = self.messages_not_requiring_fixes.entry(path).or_default();

            entry.push((diagnostic, severity));
        } else {
            self.send_diagnostic(&path, diagnostic.into(), severity);
        }
    }

    fn handle_internal_diagnostic(&mut self, e: TsGoLintInternalDiagnostic) {
        let file_path = e.file_path.clone();
        let oxc_diagnostic: OxcDiagnostic = e.into();

        let diagnostics = if let Some(ref file_path) = file_path {
            let source_text = self.get_source_text(file_path).to_string();
            DiagnosticService::wrap_diagnostics(
                &self.cwd,
                file_path,
                &source_text,
                vec![oxc_diagnostic],
            )
        } else {
            vec![oxc_diagnostic.into()]
        };

        let _ = self.error_sender.send(diagnostics);
    }

    fn send_diagnostic(
        &mut self,
        path: &Path,
        oxc_diagnostic: OxcDiagnostic,
        severity: AllowWarnDeny,
    ) {
        let source_text = self.get_source_text(path).to_string();
        let oxc_diagnostic = oxc_diagnostic.with_severity(if severity == AllowWarnDeny::Deny {
            Severity::Error
        } else {
            Severity::Warning
        });
        let diagnostics = DiagnosticService::wrap_diagnostics(
            &self.cwd,
            path,
            &source_text,
            vec![oxc_diagnostic],
        );
        let _ = self.error_sender.send(diagnostics);
    }

    /// Consume the handler and return collected messages requiring fixes.
    fn into_messages_requiring_fixes(
        self,
        diff_manager: &Arc<DiffManager>,
        paths: Vec<PathBuf>,
    ) -> Vec<(PathBuf, String, Vec<Message>)> {
        let Self { messages_requiring_fixes, mut source_text_cache, should_fix, silent, .. } = self;

        if !diff_manager.skip() {
            for path in paths {
                if let Some(messages) = self.messages_not_requiring_fixes.get(&path) {
                    let source_text = source_text_cache.0.remove(&path).unwrap_or_else(|| {
                        if !silent || should_fix {
                            read_to_string(&path).unwrap_or_default()
                        } else {
                            String::new()
                        }
                    });

                    let filtered_messages: Vec<OxcDiagnostic> = diff_manager
                        .collect_file(
                            path.as_path(),
                            self.cwd.as_path(),
                            messages
                                .iter()
                                .cloned()
                                .map(|(diagnostic, severity)| {
                                    let mut message = Message::from_tsgo_lint_diagnostic(
                                        diagnostic,
                                        source_text.as_str(),
                                    );
                                    message.error.severity = if severity == AllowWarnDeny::Deny {
                                        Severity::Error
                                    } else {
                                        Severity::Warning
                                    };
                                    message
                                })
                                .collect(),
                        )
                        .into_iter()
                        .map(Into::into)
                        .collect();

                    let diagnostics = DiagnosticService::wrap_diagnostics(
                        &self.cwd,
                        &path,
                        source_text.as_str(),
                        filtered_messages,
                    );
                    let _ = self.error_sender.send(diagnostics);
                } else if !messages_requiring_fixes.contains_key(&path) {
                    diff_manager.collect_empty_file(path.as_path(), self.cwd.as_path());
                }
            }
        }

        messages_requiring_fixes
            .into_iter()
            .map(|(path, messages)| {
                let source_text = source_text_cache.0.remove(&path).unwrap_or_else(|| {
                    if !silent || should_fix {
                        read_to_string(&path).unwrap_or_default()
                    } else {
                        String::new()
                    }
                });
                (path, source_text, messages)
            })
            .collect()
    }
}

fn should_skip_diagnostic(
    disable_directives_map: &FxHashMap<PathBuf, DisableDirectives>,
    path: &Path,
    tsgolint_diagnostic: &TsGoLintRuleDiagnostic,
) -> bool {
    let span = if tsgolint_diagnostic.span.is_unspanned() {
        tsgolint_diagnostic
            .labeled_ranges
            .first()
            .map_or(tsgolint_diagnostic.span, |range| Span::new(range.range.pos, range.range.end))
    } else {
        tsgolint_diagnostic.span
    };

    if let Some(directives) = disable_directives_map.get(path) {
        directives.contains(&tsgolint_diagnostic.rule, span)
            || directives.contains(&format!("typescript-eslint/{}", tsgolint_diagnostic.rule), span)
            || directives
                .contains(&format!("@typescript-eslint/{}", tsgolint_diagnostic.rule), span)
    } else {
        debug_assert!(
            false,
            "missing disable_directives_map entry for {}; expected directives to be collected for every linted file",
            path.display()
        );
        false
    }
}

/// Parses a single message from the binary tsgolint output.
// Messages are encoded as follows:
// | Payload Size (uint32 LE) - 4 bytes | Message Type (uint8) - 1 byte | Payload |
fn parse_single_message(
    cursor: &mut std::io::Cursor<&[u8]>,
) -> Result<TsGoLintMessage, TsGoLintMessageParseError> {
    let mut size_bytes = [0u8; 4];
    if cursor.read_exact(&mut size_bytes).is_err() {
        return Err(TsGoLintMessageParseError::IncompleteData);
    }
    let size = u32::from_le_bytes(size_bytes) as usize;
    // A desynchronized stream can announce any size; allocating before validating would turn a
    // malformed message into an out-of-memory abort.
    if size > MAX_MESSAGE_SIZE {
        return Err(TsGoLintMessageParseError::PayloadTooLarge(size));
    }

    let mut message_type_byte = [0u8; 1];
    if cursor.read_exact(&mut message_type_byte).is_err() {
        return Err(TsGoLintMessageParseError::IncompleteData);
    }

    let message_type = MessageType::try_from(message_type_byte[0])
        .map_err(TsGoLintMessageParseError::InvalidMessageType)?;

    let mut payload_bytes = vec![0u8; size];
    if cursor.read_exact(&mut payload_bytes).is_err() {
        return Err(TsGoLintMessageParseError::IncompleteData);
    }
    let payload_str = String::from_utf8_lossy(&payload_bytes);

    match message_type {
        MessageType::Error => {
            let error_payload = serde_json::from_str::<TsGoLintErrorPayload>(&payload_str)
                .map_err(TsGoLintMessageParseError::InvalidErrorPayload)?;

            Ok(TsGoLintMessage::Error(TsGoLintError { error: error_payload.error }))
        }
        MessageType::Diagnostic => {
            let diagnostic_payload =
                serde_json::from_str::<TsGoLintDiagnosticPayload>(&payload_str)
                    .map_err(TsGoLintMessageParseError::InvalidDiagnosticPayload)?;

            Ok(TsGoLintMessage::Diagnostic(match diagnostic_payload.kind {
                DiagnosticKind::Rule => {
                    TsGoLintDiagnostic::Rule(TsGoLintRuleDiagnostic {
                        rule: diagnostic_payload
                            .rule
                            .ok_or(TsGoLintMessageParseError::IncompleteRuleDiagnostic("rule"))?,
                        span: diagnostic_payload.range.map_or_else(
                            || {
                                debug_assert!(false, "Range must be present for rule diagnostics");
                                Span::default()
                            },
                            |range| Span::new(range.pos, range.end),
                        ),
                        message: diagnostic_payload.message,
                        fixes: diagnostic_payload.fixes,
                        suggestions: diagnostic_payload.suggestions,
                        labeled_ranges: diagnostic_payload.labeled_ranges,
                        file_path: PathBuf::from(diagnostic_payload.file_path.ok_or(
                            TsGoLintMessageParseError::IncompleteRuleDiagnostic("file_path"),
                        )?),
                    })
                }
                DiagnosticKind::Internal => {
                    TsGoLintDiagnostic::Internal(TsGoLintInternalDiagnostic {
                        message: diagnostic_payload.message,
                        span: diagnostic_payload.range.map(|range| Span::new(range.pos, range.end)),
                        file_path: diagnostic_payload.file_path.map(PathBuf::from),
                    })
                }
            }))
        }
        MessageType::Timing => {
            let timing_payload = serde_json::from_str::<TsGoLintTimingPayload>(&payload_str)
                .map_err(TsGoLintMessageParseError::InvalidTimingPayload)?;

            Ok(TsGoLintMessage::Timing(timing_payload))
        }
    }
}

/// The error code scope of every diagnostic `tsgolint` produces, and therefore the plugin
/// under which its rules are known to the rest of the linter.
pub const TSGOLINT_DIAGNOSTIC_SCOPE: &str = "typescript";

/// The file names the `tsgolint` executable can have inside `node_modules/.bin`.
///
/// Executing a sub-command in Windows needs a `cmd` or `ps1` extension.
/// Since `cmd` is the most compatible one with older systems, we use that one first,
/// then check for `exe` which is also common. Bun, for example, does not create a `cmd`
/// file but still produces an `exe` file (<https://github.com/oxc-project/oxc/issues/13784>).
#[cfg(windows)]
pub const TSGOLINT_EXECUTABLE_NAMES: &[&str] = &["tsgolint.CMD", "tsgolint.exe"];
#[cfg(not(windows))]
pub const TSGOLINT_EXECUTABLE_NAMES: &[&str] = &["tsgolint"];

/// Lock `mutex`, ignoring poisoning.
///
/// The groups run concurrently and share this state; a group which panics while holding a lock
/// must not block the others. Nothing guarded here can be left half-written by a panic: every
/// guarded value is a plain map or set.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// How long a resolution is remembered before the file system is looked at again.
///
/// Both answers expire: without it the language server would walk the file system on every
/// keystroke in a package which has no `tsgolint`, and would never notice a
/// `pnpm add oxlint-tsgolint`, an uninstall, or one installed closer to a file than the one
/// already recorded.
const RESOLUTION_TTL: Duration = Duration::from_secs(30);

/// What a walk concluded about a directory, and when.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Lookup {
    /// The executable that walk reached, if any.
    found: Option<ResolvedExecutable>,
    /// When the walk which produced this answer ran. Directories which learn their answer from
    /// an already resolved ancestor inherit its instant, so a whole subtree expires together
    /// instead of one level at a time.
    resolved_at: Instant,
}

impl Lookup {
    /// Whether this entry can still be trusted at `now`.
    fn is_fresh(&self, now: Instant) -> bool {
        now.duration_since(self.resolved_at) < RESOLUTION_TTL
    }
}

/// Outcome of the `node_modules/.bin/tsgolint` lookups, keyed by the directory the lookup
/// started from.
///
/// Entries are returned as they were recorded; whether one is still usable is
/// [`Lookup::is_fresh`], which the walk applies.
#[derive(Debug, Default)]
struct ExecutableCache(Mutex<FxHashMap<PathBuf, Lookup>>);

impl ExecutableCache {
    /// What was recorded for `dir`, if anything.
    fn get(&self, dir: &Path) -> Option<Lookup> {
        lock(&self.0).get(dir).cloned()
    }

    /// Record `found` for every directory a walk looked at, as of `resolved_at`.
    ///
    /// A walk answers the question for all of them at once: each visited directory would have
    /// reached the same executable, or the same absence of one.
    fn record(
        &self,
        visited: &[PathBuf],
        found: Option<&ResolvedExecutable>,
        resolved_at: Instant,
    ) {
        let mut cache = lock(&self.0);
        for dir in visited {
            cache.insert(dir.clone(), Lookup { found: found.cloned(), resolved_at });
        }
    }

    /// Resolve `start_dir` by walking up to `stop_at`, consulting the cache at every level.
    ///
    /// A fresh answer for any directory on the way answers the whole walk, so the siblings of
    /// an already resolved directory stop as soon as they reach it, and inherit when that
    /// answer was reached. Only the directories this walk actually looked at are recorded,
    /// leaving the rest as they were.
    ///
    /// `look` is the file system check and `still_installed` validates a recorded executable,
    /// both taken as arguments so they can be counted in tests and memoized by the caller.
    fn resolve(
        &self,
        start_dir: &Path,
        stop_at: Option<&Path>,
        now: Instant,
        mut look: impl FnMut(&Path) -> WalkStep,
        still_installed: &mut impl FnMut(&Path) -> bool,
    ) -> Option<ResolvedExecutable> {
        let mut visited = Vec::new();
        // Answers learned from an ancestor keep that ancestor's age.
        let mut resolved_at = now;

        let found = walk_for_tsgolint(start_dir, stop_at, |dir| {
            if let Some(cached) = self.get(dir).filter(|cached| cached.is_fresh(now)) {
                // An executable recorded earlier can have been uninstalled since; that entry is
                // no better than no entry at all.
                let usable =
                    cached.found.as_ref().is_none_or(|resolved| still_installed(&resolved.path));
                if usable {
                    resolved_at = cached.resolved_at;
                    return WalkStep::Stop(cached.found);
                }
            }

            visited.push(dir.to_path_buf());
            look(dir)
        });

        self.record(&visited, found.as_ref(), resolved_at);
        found
    }
}

/// Only a handful of paths are listed; naming every file of a large monorepo is not helpful.
const MAX_REPORTED_UNRESOLVED_PATHS: usize = 5;

/// What a `tsgolint` run will do: which files each installation lints, and which files no
/// installation could be found for.
#[derive(Debug, Default)]
pub struct LintPlan {
    /// One entry per distinct installation, ordered deterministically.
    groups: Vec<LintGroup>,
    /// Files which have no `tsgolint` to lint them. They are reported, never dropped.
    unresolved: Vec<Arc<OsStr>>,
}

/// The files handed to one `tsgolint` process.
#[derive(Debug)]
struct LintGroup {
    /// The executable to run for these files.
    executable: PathBuf,
    /// Absolute paths of the files this executable lints.
    paths: Vec<Arc<OsStr>>,
}

/// The `tsgolint` executable used for the files which have none of their own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Fallback {
    /// Pinned through `OXLINT_TSGOLINT_PATH`: this executable was explicitly requested, so it
    /// is used for every file and no per-directory lookup is performed.
    Env(ResolvedExecutable),
    /// Found from the working directory. A file may still have a closer one of its own.
    Discovered(ResolvedExecutable),
    /// Nothing was found for the working directory.
    #[default]
    None,
}

impl Fallback {
    fn executable(&self) -> Option<&ResolvedExecutable> {
        match self {
            Fallback::Env(resolved) | Fallback::Discovered(resolved) => Some(resolved),
            Fallback::None => None,
        }
    }

    /// Whether this executable must be used for every file, regardless of what is installed
    /// next to it.
    fn is_pinned(&self) -> bool {
        matches!(self, Fallback::Env(_))
    }
}

/// A `tsgolint` executable together with the identity of the installation it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedExecutable {
    /// Identifies the installation, so that two shims which run the same binary are one group.
    key: PathBuf,
    /// The path to actually execute.
    path: PathBuf,
}

impl ResolvedExecutable {
    fn new(path: PathBuf) -> Self {
        Self { key: executable_group_key(&path), path }
    }
}

/// How many `tsgolint` processes run at once.
const MAX_CONCURRENT_GROUPS: usize = 4;

/// Run `lint` over every group, at most [`MAX_CONCURRENT_GROUPS`] at a time, and return the
/// results in group order.
///
/// `tsgolint` is itself parallel, so one process per installation would oversubscribe the
/// machine on a large monorepo; a handful is enough to overlap their startup and I/O. Workers
/// take the next group as soon as they are free, so one slow package does not hold the others
/// back, and each result is tagged with its group before the work starts, so a group which
/// fails cannot shift the others' results onto the wrong groups.
fn run_groups<T: Send>(
    groups: &[LintGroup],
    lint: impl Fn(&LintGroup) -> Result<T, String> + Sync,
) -> Vec<Result<T, String>> {
    // One installation is the overwhelmingly common case; run it here rather than spawning a
    // thread to wait on.
    if let [group] = groups {
        return vec![lint(group)];
    }

    let workers = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(MAX_CONCURRENT_GROUPS)
        .min(groups.len());

    let next_group = AtomicUsize::new(0);
    let lint = &lint;
    let next_group = &next_group;

    let collected: Vec<(usize, Result<T, String>)> = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            handles.push(scope.spawn(move || {
                let mut results = Vec::new();
                loop {
                    let index = next_group.fetch_add(1, Ordering::Relaxed);
                    let Some(group) = groups.get(index) else { break };

                    // Tagged with its index before the work starts, so the results cannot end
                    // up against the wrong groups.
                    results.push((index, lint(group)));
                }
                results
            }));
        }

        handles.into_iter().filter_map(|handle| handle.join().ok()).flatten().collect()
    });

    // One entry per group, in group order, regardless of what happened to the workers.
    let mut collected: FxHashMap<usize, Result<T, String>> = collected.into_iter().collect();
    (0..groups.len())
        .map(|index| {
            collected
                .remove(&index)
                .unwrap_or_else(|| Err("tsgolint group did not run".to_string()))
        })
        .collect()
}

/// Report every group which failed, one line each, instead of only the first.
///
/// `results` is in group order, as [`run_groups`] returns it.
fn join_group_errors(groups: &[LintGroup], results: Vec<Result<(), String>>) -> Result<(), String> {
    let errors = results
        .into_iter()
        .zip(groups)
        .filter_map(|(result, group)| {
            result.err().map(|error| format!("{}: {error}", group.executable.display()))
        })
        .collect::<Vec<_>>();

    if errors.is_empty() { Ok(()) } else { Err(errors.join("\n")) }
}

/// Keep the paths `keep` accepts and group them into one [`LintGroup`] per distinct `tsgolint`
/// installation.
///
/// `resolve` maps a directory to the executable which lints the files in it; it is expected to
/// answer repeated questions cheaply, which [`ExecutableCache`] takes care of. Groups are sorted
/// by executable path, so that the order in which they are run and merged is deterministic.
/// Files `resolve` finds no executable for are collected separately instead of being dropped.
fn group_paths_by_executable(
    paths: &[Arc<OsStr>],
    keep: impl Fn(&Path) -> bool,
    mut resolve: impl FnMut(&Path) -> Option<ResolvedExecutable>,
) -> LintPlan {
    let mut groups: FxHashMap<PathBuf, LintGroup> = FxHashMap::default();
    let mut unresolved: Vec<Arc<OsStr>> = Vec::new();

    for path in paths {
        let file = Path::new(path.as_ref());
        if !keep(file) {
            continue;
        }

        let Some(resolved) = resolve(file.parent().unwrap_or(file)) else {
            unresolved.push(Arc::clone(path));
            continue;
        };

        let group = groups
            .entry(resolved.key)
            .or_insert_with(|| LintGroup { executable: resolved.path.clone(), paths: Vec::new() });
        // Several shims can lead to one installation; always run the same one.
        if resolved.path < group.executable {
            group.executable = resolved.path;
        }
        group.paths.push(Arc::clone(path));
    }

    let mut groups: Vec<LintGroup> = groups.into_values().collect();
    groups.sort_unstable_by(|a, b| a.executable.cmp(&b.executable));

    LintPlan { groups, unresolved }
}

/// How to get a `tsgolint`, appended to every message about a missing executable.
const MISSING_EXECUTABLE_HELP: &str = "Add the `oxlint-tsgolint` package to the corresponding project, or set the `OXLINT_TSGOLINT_PATH` environment variable to a `tsgolint` executable.";

/// The single warning reported for all the files no `tsgolint` executable could be found for.
///
/// Paths are rendered relative to `cwd`, like every other diagnostic.
fn missing_executable_diagnostic(unresolved: &[Arc<OsStr>], cwd: &Path) -> OxcDiagnostic {
    let mut listed = String::new();
    for path in unresolved.iter().take(MAX_REPORTED_UNRESOLVED_PATHS) {
        let path = Path::new(path.as_ref());
        let path = path.strip_prefix(cwd).unwrap_or(path);
        // `Filename` renders a path the way the suppressions file does, with `/` separators on
        // every platform.
        let _ = write!(listed, "\n  {}", Filename::new(path));
    }
    if unresolved.len() > MAX_REPORTED_UNRESOLVED_PATHS {
        let _ =
            write!(listed, "\n  ... and {} more", unresolved.len() - MAX_REPORTED_UNRESOLVED_PATHS);
    }

    OxcDiagnostic::warn(format!(
        "Could not find a `tsgolint` executable for {} file(s), they were not linted with type-aware rules:{listed}\n{MISSING_EXECUTABLE_HELP}",
        unresolved.len()
    ))
}

/// Attach `diagnostic` to `file`, so that every reporter can place it.
///
/// The graphical reporter accepts a bare diagnostic, but `unix`, `checkstyle`, `gitlab`
/// and `junit` render one as an empty line with no file and no position. A source and a label
/// at the top of the file give them something to print.
fn attach_to_file(diagnostic: OxcDiagnostic, file: &Path, cwd: &Path) -> Vec<Error> {
    let source_text = read_to_string(file).unwrap_or_default();
    DiagnosticService::wrap_diagnostics(
        cwd,
        file,
        &source_text,
        vec![diagnostic.with_label(Span::default())],
    )
}

/// The error reported for a group whose `tsgolint` could not be run.
fn group_failure_diagnostic(group: &LintGroup, error: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "`{}` could not lint this file: {error}\n{MISSING_EXECUTABLE_HELP}",
        group.executable.display()
    ))
}

/// The error reported when no file at all could be linted with type-aware rules.
///
/// Unlike a partial failure this is fatal: type-aware linting did not run, and a run which
/// requested it must not pass.
fn no_executable_error(unresolved: usize) -> String {
    format!(
        "Could not find a `tsgolint` executable for any of the {unresolved} file(s) to lint with type-aware rules.\n{MISSING_EXECUTABLE_HELP}"
    )
}

/// The warning the editor gets when type-aware linting could not run for a file at all.
///
/// The regular rules did run, so their diagnostics are kept; the message reports why the
/// type-aware ones are missing.
pub fn type_aware_failure_message(error: &str) -> Message {
    let diagnostic =
        OxcDiagnostic::warn(format!("This file was not linted with type-aware rules: {error}"))
            .with_help(MISSING_EXECUTABLE_HELP)
            .with_label(Span::default());

    Message::new(diagnostic, PossibleFixes::None)
}

/// The warning the editor gets for a group whose `tsgolint` could not be run.
///
/// It names the first file of the group, so the message points at a file the user can see, and
/// the executable, so the installation to fix is identified.
fn group_failure_message(group: &LintGroup, error: &str) -> Message {
    let file = group
        .paths
        .first()
        .map(|path| Path::new(path.as_ref()).display().to_string())
        .unwrap_or_default();

    let diagnostic = OxcDiagnostic::warn(format!(
        "`{}` could not lint `{file}`, so it was not checked with type-aware rules: {error}",
        group.executable.display()
    ))
    .with_help(MISSING_EXECUTABLE_HELP)
    .with_label(Span::default());

    Message::new(diagnostic, PossibleFixes::None)
}

/// The warnings the editor gets for the files no `tsgolint` executable could be found for: one
/// per file, rebuilt on every lint.
///
/// They are deliberately not deduplicated across runs: each `publishDiagnostics` replaces the
/// whole set for a file, so a warning reported only the first time would flash and vanish.
fn missing_executable_messages(unresolved: &[Arc<OsStr>]) -> Vec<Message> {
    unresolved
        .iter()
        .map(|_| {
            let diagnostic = OxcDiagnostic::warn(
                "Could not find a `tsgolint` executable for this file, it was not linted with type-aware rules.",
            )
            .with_help(MISSING_EXECUTABLE_HELP)
            .with_label(Span::default());

            Message::new(diagnostic, PossibleFixes::None)
        })
        .collect()
}

/// Identify the `tsgolint` installation `executable` belongs to.
///
/// Two packages which depend on the same `tsgolint` version must end up in a single group: pnpm
/// links every identical version to one store directory, and npm/yarn hoist it to a single path,
/// so the package directory identifies the installation while the `node_modules/.bin` shim does
/// not. Falls back to the canonical path of the executable itself, then to the raw path when the
/// file system cannot be queried.
fn executable_group_key(executable: &Path) -> PathBuf {
    // `<dir>/node_modules/.bin/tsgolint` -> `<dir>/node_modules/oxlint-tsgolint`
    if let Some(node_modules) = executable.parent().and_then(Path::parent)
        && let Ok(package_dir) = std::fs::canonicalize(node_modules.join("oxlint-tsgolint"))
    {
        return package_dir;
    }

    std::fs::canonicalize(executable).unwrap_or_else(|_| executable.to_path_buf())
}

/// Resolve the `tsgolint` executable an explicit `OXLINT_TSGOLINT_PATH` points at.
///
/// The value may name the executable itself or the directory holding it.
///
/// # Errors
/// Returns an error if it names neither.
fn tsgolint_executable_from_env_value(value: &OsStr) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    let displayed = path.display();

    if path.is_dir() {
        // An explicit directory is looked at more loosely than a `node_modules/.bin`: on Windows
        // that one only holds the shims a package manager wrote, while a user pointing at a
        // directory is usually pointing at the extracted release, whose binary has no extension.
        if let Some(found) = tsgolint_in_dir(&path).or_else(|| {
            let bare = path.join("tsgolint");
            bare.is_file().then_some(bare)
        }) {
            return Ok(found);
        }
        return Err(format!(
            "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to directory '{displayed}' but 'tsgolint' binary not found inside"
        ));
    }
    if path.is_file() {
        return Ok(path);
    }
    Err(format!(
        "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to '{displayed}' which does not exist"
    ))
}

/// What looking at one directory concluded, during a walk.
enum WalkStep {
    /// This directory answers the walk, with an executable or with the absence of one.
    Stop(Option<ResolvedExecutable>),
    /// Nothing here; keep climbing.
    Climb,
}

/// Walk up from `start_dir`, asking `look` about every directory on the way.
///
/// The walk stops after `stop_at` when one is given (it is asked about too, but nothing above
/// it is), and at the file system root otherwise.
fn walk_for_tsgolint(
    start_dir: &Path,
    stop_at: Option<&Path>,
    mut look: impl FnMut(&Path) -> WalkStep,
) -> Option<ResolvedExecutable> {
    let mut current_dir = start_dir;
    loop {
        if let WalkStep::Stop(found) = look(current_dir) {
            return found;
        }
        if stop_at.is_some_and(|stop_at| current_dir == stop_at) {
            return None;
        }

        current_dir = current_dir.parent()?;
    }
}

/// Where a walk which starts at `dir` must stop.
///
/// The working directory is the boundary for the files below it: the target is the package of
/// the project which owns the file, and climbing past the project would pick up an unrelated
/// installation. A file outside the working directory has no such boundary, because linting
/// `../other-package` is a supported use case, so its walk goes up to the file system root
/// instead, and falls back only if that finds nothing either.
fn walk_boundary_for<'a>(dir: &Path, cwd: &'a Path) -> Option<&'a Path> {
    dir.starts_with(cwd).then_some(cwd)
}

/// Look for `node_modules/.bin/tsgolint` in `dir`, as a [`WalkStep`].
fn look_in_node_modules(dir: &Path) -> WalkStep {
    tsgolint_in_node_modules(dir)
        .map_or(WalkStep::Climb, |found| WalkStep::Stop(Some(ResolvedExecutable::new(found))))
}

fn find_tsgolint_from_cwd(cwd: &Path) -> Option<ResolvedExecutable> {
    walk_for_tsgolint(cwd, None, look_in_node_modules)
        .or_else(|| find_tsgolint_in_path().map(ResolvedExecutable::new))
}

/// Look for the `tsgolint` executable inside `dir/node_modules/.bin`.
fn tsgolint_in_node_modules(dir: &Path) -> Option<PathBuf> {
    tsgolint_in_dir(&dir.join("node_modules").join(".bin"))
}

/// Look for any of [`TSGOLINT_EXECUTABLE_NAMES`] directly inside `dir`.
fn tsgolint_in_dir(dir: &Path) -> Option<PathBuf> {
    TSGOLINT_EXECUTABLE_NAMES.iter().map(|file| dir.join(file)).find(|path| path.exists())
}

/// Search the system `PATH` for the `tsgolint` executable.
///
/// This supports package managers that install binaries globally and make them available
/// via `PATH`.
fn find_tsgolint_in_path() -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    std::env::split_paths(&path_env)
        .flat_map(|dir| TSGOLINT_EXECUTABLE_NAMES.iter().map(move |file| dir.join(file)))
        .find(|candidate| candidate.is_file())
}

/// Tries to find the `tsgolint` executable for `cwd`, and reports whether it was pinned through
/// `OXLINT_TSGOLINT_PATH` (in which case it must be used for every file), reading that variable
/// from the environment.
///
/// # Errors
/// See [`resolve_tsgolint_executable_with`].
fn resolve_tsgolint_executable(cwd: &Path) -> Result<Fallback, String> {
    resolve_tsgolint_executable_with(cwd, std::env::var_os("OXLINT_TSGOLINT_PATH").as_deref())
}

/// Tries to find the `tsgolint` executable for `cwd`, given the value of
/// `OXLINT_TSGOLINT_PATH`. In priority order, this will check:
/// 1. `explicit_path`, when the variable is set.
/// 2. The `tsgolint` binary in a `node_modules/.bin` directory, starting at `cwd` and moving
///    upwards to the file system root.
/// 3. The system `PATH`.
///
/// Returns `Ok(None)` when none of them has one: a monorepo often installs `tsgolint` only in
/// its packages, which [`TsGoLintState::resolve_executable_for_dir`] finds later.
///
/// # Errors
/// Returns an error only when `explicit_path` is set but names neither an executable nor a
/// directory holding one: an explicitly requested executable is never silently replaced by
/// another one, while having none at all is not an error here.
fn resolve_tsgolint_executable_with(
    cwd: &Path,
    explicit_path: Option<&OsStr>,
) -> Result<Fallback, String> {
    if let Some(explicit_path) = explicit_path {
        return tsgolint_executable_from_env_value(explicit_path)
            .map(|executable| Fallback::Env(ResolvedExecutable::new(executable)));
    }

    Ok(find_tsgolint_from_cwd(cwd).map_or(Fallback::None, Fallback::Discovered))
}

#[cfg(test)]
mod test {
    use std::{
        ffi::OsStr,
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{Duration, Instant},
    };

    use oxc_diagnostics::{LabeledSpan, OxcCode, Severity};
    use oxc_span::Span;

    use crate::{
        fixer::{FixKind, Message, PossibleFixes},
        tsgolint::{Fix, LabeledRange, Range, RuleMessage, Suggestion, TsGoLintRuleDiagnostic},
    };

    use super::{
        ExecutableCache, Fallback, LintGroup, LintPlan, MAX_MESSAGE_SIZE, RESOLUTION_TTL,
        ResolvedExecutable, TSGOLINT_DIAGNOSTIC_SCOPE, TSGOLINT_EXECUTABLE_NAMES,
        TsGoLintMessageParseError, executable_group_key, find_tsgolint_from_cwd,
        group_failure_message, group_paths_by_executable, join_group_errors, look_in_node_modules,
        missing_executable_diagnostic, missing_executable_messages, no_executable_error,
        parse_single_message, resolve_tsgolint_executable_with, run_groups, walk_boundary_for,
    };

    /// Stand in for the per-plan validation, which the tests below do not exercise.
    fn always_installed(_: &Path) -> bool {
        true
    }

    /// Resolve `start_dir` the way production does, boundary included, through a cache of its
    /// own.
    fn resolve_for_dir(start_dir: &Path, cwd: &Path) -> Option<PathBuf> {
        ExecutableCache::default()
            .resolve(
                start_dir,
                walk_boundary_for(start_dir, cwd),
                Instant::now(),
                look_in_node_modules,
                &mut always_installed,
            )
            .map(|resolved| resolved.path)
    }

    /// Create a fake `tsgolint` executable in `<dir>/node_modules/.bin`, using the same file
    /// name production code looks for, and return its path.
    fn create_fake_tsgolint(dir: &Path) -> PathBuf {
        let bin_dir = dir.join("node_modules").join(".bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let executable = bin_dir.join(TSGOLINT_EXECUTABLE_NAMES[0]);
        std::fs::write(&executable, "").unwrap();
        executable
    }

    fn arc_path(path: &Path) -> Arc<OsStr> {
        Arc::from(path.as_os_str())
    }

    #[test]
    fn test_find_tsgolint_for_dir() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let a_src = root_path.join("packages").join("a").join("src");
        let b_src = root_path.join("packages").join("b").join("src");
        std::fs::create_dir_all(&a_src).unwrap();
        std::fs::create_dir_all(&b_src).unwrap();

        // Only `packages/a` has `tsgolint` installed.
        let a_executable = create_fake_tsgolint(&root_path.join("packages").join("a"));

        assert_eq!(resolve_for_dir(&a_src, root_path), Some(a_executable.clone()));
        assert_eq!(resolve_for_dir(&b_src, root_path), None);

        // Installing it at the root makes it visible from both packages, but `packages/a`
        // keeps using its own.
        let root_executable = create_fake_tsgolint(root_path);
        assert_eq!(resolve_for_dir(&a_src, root_path), Some(a_executable));
        assert_eq!(resolve_for_dir(&b_src, root_path), Some(root_executable));
    }

    #[test]
    fn per_file_walk_stops_at_cwd_and_the_cwd_walk_continues_above_it() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // The executable lives above the working directory.
        let executable = create_fake_tsgolint(root_path);

        let cwd = root_path.join("workspace");
        let src = cwd.join("packages").join("b").join("src");
        std::fs::create_dir_all(&src).unwrap();

        // The per-file walk stops at the working directory.
        assert_eq!(resolve_for_dir(&src, &cwd), None);
        // The working directory's own walk keeps going up.
        assert_eq!(find_tsgolint_from_cwd(&cwd).map(|resolved| resolved.path), Some(executable));

        // The boundary itself is still looked at.
        assert!(resolve_for_dir(&src, root_path).is_some());
    }

    /// Only an explicitly requested executable which cannot be used is fatal; having none at
    /// all is left to the per-file lookup.
    #[test]
    fn try_new_fails_only_for_an_invalid_explicit_path() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // A directory which does not hold a `tsgolint`.
        let error = resolve_tsgolint_executable_with(root_path, Some(root_path.as_os_str()))
            .expect_err("a directory without the binary is not usable");
        assert!(error.contains("OXLINT_TSGOLINT_PATH"), "{error}");

        // A path which does not exist at all.
        let missing = root_path.join("nowhere");
        let error = resolve_tsgolint_executable_with(root_path, Some(missing.as_os_str()))
            .expect_err("a path which does not exist is not usable");
        assert!(error.contains("OXLINT_TSGOLINT_PATH"), "{error}");

        // A directory holding the binary under its bare name, which is how the release is
        // shipped on every platform, including the one whose shims carry an extension.
        let release_dir = root_path.join("release");
        std::fs::create_dir_all(&release_dir).unwrap();
        let bare = release_dir.join("tsgolint");
        std::fs::write(&bare, "").unwrap();
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(release_dir.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(bare.clone())),
            "an explicit directory should be searched for the bare name too"
        );

        // A directory holding it under one of the names a package manager writes.
        let executable = create_fake_tsgolint(root_path);
        let bin_dir = executable.parent().unwrap();
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(bin_dir.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(executable.clone())),
            "the directory should be searched for every name of {TSGOLINT_EXECUTABLE_NAMES:?}"
        );

        // And the executable named directly.
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(bare.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(bare))
        );
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(executable.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(executable))
        );

        // Without an explicit path, finding nothing is not an error.
        assert!(resolve_tsgolint_executable_with(root_path, None).is_ok());
    }

    /// The editor replaces the whole diagnostic set for a file on every lint, so the warning
    /// has to be repeated instead of reported once.
    #[test]
    fn lsp_warns_about_every_unresolved_file_on_every_lint() {
        let unresolved =
            [arc_path(Path::new("/packages/b/x.ts")), arc_path(Path::new("/packages/b/y.ts"))];

        for _ in 0..3 {
            let messages = missing_executable_messages(&unresolved);
            assert_eq!(messages.len(), unresolved.len(), "one warning per file, every time");
            for message in &messages {
                assert_eq!(message.error.severity, Severity::Warning);
                let help = message.error.help.as_ref().expect("the warning should say what to do");
                assert!(help.contains("oxlint-tsgolint"), "{help}");
                assert!(help.contains("OXLINT_TSGOLINT_PATH"), "{help}");
            }
        }
    }

    /// A walk out of the working directory would climb to the file system root, looking into
    /// directories which have nothing to do with the project.
    #[test]
    fn a_file_outside_cwd_walks_above_it_instead_of_stopping() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // A sibling package of the working directory, with its own `tsgolint`.
        let sibling = root_path.join("packages").join("a");
        let sibling_src = sibling.join("src");
        std::fs::create_dir_all(&sibling_src).unwrap();
        let executable = create_fake_tsgolint(&sibling);

        let cwd = root_path.join("packages").join("b");
        std::fs::create_dir_all(&cwd).unwrap();

        // `oxlint --type-aware ../a`, run from `packages/b`.
        assert!(!sibling_src.starts_with(&cwd), "the file should be outside the cwd");
        assert_eq!(walk_boundary_for(&sibling_src, &cwd), None, "and so have no boundary");

        let mut looked_at = Vec::new();
        let found = ExecutableCache::default().resolve(
            &sibling_src,
            walk_boundary_for(&sibling_src, &cwd),
            Instant::now(),
            |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            },
            &mut always_installed,
        );

        assert_eq!(found.map(|resolved| resolved.path), Some(executable));
        assert_eq!(looked_at, vec![sibling_src, sibling], "the walk should climb out of the cwd");

        // A file below the working directory still stops there.
        let inside = cwd.join("src");
        std::fs::create_dir_all(&inside).unwrap();
        assert_eq!(walk_boundary_for(&inside, &cwd), Some(cwd.as_path()));
        assert_eq!(resolve_for_dir(&inside, &cwd), None);
    }

    #[test]
    fn a_cached_hit_is_checked_before_it_is_reused() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let package = root_path.join("packages").join("a");
        let src = package.join("src");
        std::fs::create_dir_all(&src).unwrap();
        let executable = create_fake_tsgolint(&package);

        let cache = ExecutableCache::default();
        let now = Instant::now();
        assert_eq!(
            cache
                .resolve(&src, Some(root_path), now, look_in_node_modules, &mut always_installed,)
                .map(|resolved| resolved.path),
            Some(executable.clone())
        );

        // `pnpm remove oxlint-tsgolint`, without restarting the editor.
        std::fs::remove_file(&executable).unwrap();

        let mut looked_at = Vec::new();
        let found = cache.resolve(
            &src,
            Some(root_path),
            now,
            |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            },
            &mut |path: &Path| path.exists(),
        );

        assert_eq!(found, None, "the recorded executable is gone, so nothing resolves");
        assert!(!looked_at.is_empty(), "the stale entry should be re-resolved rather than trusted");
    }

    /// In the editor, one broken installation must not remove the diagnostics of the packages
    /// which do work; its failure is reported as a warning instead.
    #[test]
    fn a_broken_group_becomes_a_warning_rather_than_hiding_the_others() {
        let file = PathBuf::from("/packages/b/src/index.ts");
        let group = LintGroup {
            executable: PathBuf::from("/packages/b/node_modules/.bin/tsgolint"),
            paths: vec![arc_path(&file)],
        };

        let message = group_failure_message(&group, "exit status: 1");

        assert_eq!(message.error.severity, Severity::Warning);
        let text = message.error.message.to_string();
        assert!(text.contains("/packages/b/src/index.ts"), "{text}");
        assert!(text.contains("/packages/b/node_modules/.bin/tsgolint"), "{text}");
        assert!(text.contains("exit status: 1"), "{text}");
    }

    /// Two broken installations are both reported, not just the first.
    #[test]
    fn joins_every_group_failure_into_one_message() {
        let groups = ["/a/tsgolint", "/b/tsgolint", "/c/tsgolint"]
            .into_iter()
            .map(|path| LintGroup { executable: PathBuf::from(path), paths: vec![] })
            .collect::<Vec<_>>();

        let error = join_group_errors(
            &groups,
            vec![Err("exit status: 1".to_string()), Ok(()), Err("broken pipe".to_string())],
        )
        .expect_err("two groups failed");

        assert_eq!(error, "/a/tsgolint: exit status: 1\n/c/tsgolint: broken pipe");

        // The group which succeeded is not mentioned, and an all-clear run has no error.
        assert!(!error.contains("/b/tsgolint"), "{error}");
        assert!(join_group_errors(&groups, vec![Ok(()), Ok(()), Ok(())]).is_ok());
    }

    /// A group which fails must not shift the results of the others onto the wrong groups.
    #[test]
    fn a_failing_group_does_not_misalign_the_results() {
        let groups = ["/a/tsgolint", "/b/tsgolint", "/c/tsgolint"]
            .into_iter()
            .map(|path| LintGroup { executable: PathBuf::from(path), paths: vec![] })
            .collect::<Vec<_>>();

        let results = run_groups(&groups, |group| {
            if group.executable.ends_with("b/tsgolint") {
                return Err("exit status: 1".to_string());
            }
            Ok(group.executable.clone())
        });

        assert_eq!(results.len(), groups.len(), "one entry per group, always");
        assert_eq!(results[0].as_deref().map(Path::to_path_buf), Ok(groups[0].executable.clone()));
        assert_eq!(results[1].as_ref().err().map(String::as_str), Some("exit status: 1"));
        assert_eq!(results[2].as_deref().map(Path::to_path_buf), Ok(groups[2].executable.clone()));
    }

    /// `tsgolint` saturates the machine on its own, so only a handful run at once.
    #[test]
    fn concurrency_is_capped_at_four_groups() {
        let groups = (0..16)
            .map(|index| LintGroup {
                executable: PathBuf::from(format!("/bin/tsgolint-{index}")),
                paths: vec![],
            })
            .collect::<Vec<_>>();

        let running = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);

        let results = run_groups(&groups, |group| {
            let now = running.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            // Long enough for the other workers to pile up if the cap did not hold.
            std::thread::sleep(Duration::from_millis(5));
            running.fetch_sub(1, Ordering::SeqCst);
            Ok(group.executable.clone())
        });

        let expected_cap =
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get).min(4);
        let peak = peak.load(Ordering::SeqCst);
        assert!(peak <= expected_cap, "at most {expected_cap} groups may run at once, saw {peak}");

        // Every group ran, and the results came back in group order.
        let executables = results.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(
            executables,
            groups.iter().map(|group| group.executable.clone()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_executable_group_key_identifies_the_package() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let executable = create_fake_tsgolint(root_path);
        // Without the package directory, the executable itself identifies the installation.
        assert_eq!(executable_group_key(&executable), std::fs::canonicalize(&executable).unwrap());

        let package_dir = root_path.join("node_modules").join("oxlint-tsgolint");
        std::fs::create_dir_all(&package_dir).unwrap();
        assert_eq!(executable_group_key(&executable), std::fs::canonicalize(&package_dir).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn test_executable_group_key_follows_the_package_manager_store() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // What pnpm does: every package depending on the same version links to one store entry.
        let store = root_path.join("store").join("oxlint-tsgolint");
        std::fs::create_dir_all(&store).unwrap();

        let mut keys = vec![];
        for package in ["a", "b"] {
            let package_root = root_path.join("packages").join(package);
            let executable = create_fake_tsgolint(&package_root);
            std::os::unix::fs::symlink(
                &store,
                package_root.join("node_modules").join("oxlint-tsgolint"),
            )
            .unwrap();
            keys.push(executable_group_key(&executable));
        }

        // Two different shims, one installation, so one group.
        assert_eq!(keys[0], keys[1]);
    }

    #[test]
    fn test_group_paths_by_executable() {
        let a = PathBuf::from("/packages/a/src/x.ts");
        let b = PathBuf::from("/packages/b/src/y.ts");
        let c = PathBuf::from("/packages/b/src/z.ts");
        let without = PathBuf::from("/packages/c/src/w.ts");
        let skipped = PathBuf::from("/packages/b/src/tsconfig.json");

        let a_executable = PathBuf::from("/packages/a/node_modules/.bin/tsgolint");
        let b_executable = PathBuf::from("/packages/b/node_modules/.bin/tsgolint");

        let paths =
            [arc_path(&a), arc_path(&b), arc_path(&c), arc_path(&without), arc_path(&skipped)];

        let mut resolved_dirs = vec![];
        let LintPlan { groups, unresolved } = group_paths_by_executable(
            &paths,
            |path| path.extension().is_some_and(|extension| extension == "ts"),
            |dir| {
                resolved_dirs.push(dir.to_path_buf());
                let executable = if dir.starts_with("/packages/a") {
                    a_executable.clone()
                } else if dir.starts_with("/packages/b") {
                    b_executable.clone()
                } else {
                    return None;
                };
                // Stand in for `executable_group_key`, which would hit the file system.
                Some(ResolvedExecutable { key: executable.clone(), path: executable })
            },
        );

        // Two distinct executables produce two groups, each with only its own files.
        assert_eq!(groups.len(), 2);
        let LintGroup { executable, paths } = &groups[0];
        assert_eq!(*executable, a_executable);
        assert_eq!(*paths, vec![arc_path(&a)]);
        let LintGroup { executable, paths } = &groups[1];
        assert_eq!(*executable, b_executable);
        assert_eq!(*paths, vec![arc_path(&b), arc_path(&c)]);

        // Files without any executable are collected instead of being dropped.
        assert_eq!(unresolved, vec![arc_path(&without)]);

        // The file `keep` rejects is never resolved.
        assert!(
            !resolved_dirs.iter().any(|dir| dir == Path::new("/packages/b/src/tsconfig.json")),
            "{resolved_dirs:?}"
        );
    }

    #[test]
    fn test_group_paths_by_executable_picks_one_shim_deterministically() {
        let a = PathBuf::from("/packages/a/src/x.ts");
        let b = PathBuf::from("/packages/b/src/y.ts");
        let key = PathBuf::from("/store/oxlint-tsgolint");

        // Two shims of one installation: the same one runs regardless of the order of the files.
        let shim_for = |dir: &Path| ResolvedExecutable {
            key: key.clone(),
            path: dir.join("node_modules").join(".bin").join("tsgolint"),
        };

        let forwards = group_paths_by_executable(
            &[arc_path(&a), arc_path(&b)],
            |_| true,
            |dir| Some(shim_for(dir)),
        );
        let backwards = group_paths_by_executable(
            &[arc_path(&b), arc_path(&a)],
            |_| true,
            |dir| Some(shim_for(dir)),
        );

        assert_eq!(forwards.groups.len(), 1);
        assert_eq!(backwards.groups.len(), 1);
        assert_eq!(forwards.groups[0].executable, backwards.groups[0].executable);
        assert_eq!(
            forwards.groups[0].executable,
            PathBuf::from("/packages/a/src/node_modules/.bin/tsgolint")
        );
        assert!(forwards.unresolved.is_empty());
    }

    #[test]
    fn test_executable_cache_records_every_visited_directory() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let package = root_path.join("packages").join("a");
        let deep = package.join("src").join("nested");
        std::fs::create_dir_all(&deep).unwrap();
        let executable = create_fake_tsgolint(&package);

        let cache = ExecutableCache::default();
        let now = Instant::now();
        let mut looked_at = Vec::new();
        let found = cache.resolve(
            &deep,
            Some(root_path),
            now,
            |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            },
            &mut always_installed,
        );

        assert_eq!(found.as_ref().map(|resolved| &resolved.path), Some(&executable));
        // The walk stopped as soon as it found one.
        assert_eq!(looked_at, vec![deep, package.join("src"), package.clone()]);

        // Every directory it climbed now answers on its own.
        for dir in &looked_at {
            let cached = cache.get(dir).unwrap_or_else(|| {
                panic!("{} should be cached", dir.display());
            });
            assert_eq!(cached.found.map(|resolved| resolved.path), Some(executable.clone()));
        }
    }

    /// A walk which reaches a directory the cache already knows about stops there, instead of
    /// asking the file system all the way up again.
    #[test]
    fn walk_stops_at_the_first_cached_ancestor() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let package = root_path.join("packages").join("a");
        let src = package.join("src");
        let first = src.join("first");
        let second = src.join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        let executable = create_fake_tsgolint(&package);

        let cache = ExecutableCache::default();
        let now = Instant::now();

        cache.resolve(&first, Some(root_path), now, look_in_node_modules, &mut always_installed);

        // A sibling only looks at itself before finding `src` in the cache.
        let mut looked_at = Vec::new();
        let found = cache.resolve(
            &second,
            Some(root_path),
            now,
            |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            },
            &mut always_installed,
        );

        assert_eq!(found.map(|resolved| resolved.path), Some(executable));
        assert_eq!(looked_at, vec![second], "nothing above the cached ancestor should be probed");

        // The same holds for a directory already resolved: no file system access at all.
        let mut looked_at = Vec::new();
        cache.resolve(
            &first,
            Some(root_path),
            now,
            |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            },
            &mut always_installed,
        );
        assert!(looked_at.is_empty(), "a cached directory should not be probed again");
    }

    /// A cached miss stops the walk too, until it expires.
    #[test]
    fn a_resolution_is_remembered_for_thirty_seconds() {
        let cache = ExecutableCache::default();
        let missing = PathBuf::from("/packages/b/src");
        let found = PathBuf::from("/packages/a/src");
        let now = Instant::now();

        let resolved = ResolvedExecutable {
            key: PathBuf::from("/store/oxlint-tsgolint"),
            path: PathBuf::from("/packages/a/node_modules/.bin/tsgolint"),
        };
        cache.record(std::slice::from_ref(&missing), None, now);
        cache.record(std::slice::from_ref(&found), Some(&resolved), now);

        for dir in [&missing, &found] {
            let recorded = cache.get(dir).expect("the walk should have been recorded");
            // Trusted for a while, so the file system is left alone.
            assert!(recorded.is_fresh(now));
            assert!(recorded.is_fresh(now + (RESOLUTION_TTL / 2)));
            // Once expired, installing, moving or removing `tsgolint` is picked up.
            assert!(!recorded.is_fresh(now + RESOLUTION_TTL));
        }

        // The group key is remembered too, so it is not recomputed on every file.
        assert_eq!(cache.get(&found).unwrap().found, Some(resolved));
    }

    /// A directory which learns its answer from an already resolved ancestor must expire with
    /// it, not thirty seconds after it was asked.
    #[test]
    fn a_cached_answer_does_not_get_younger_as_it_spreads() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let src = root_path.join("packages").join("a").join("src");
        std::fs::create_dir_all(&src).unwrap();
        create_fake_tsgolint(root_path);

        let cache = ExecutableCache::default();
        let resolved_at = Instant::now();
        cache.resolve(
            root_path,
            Some(root_path),
            resolved_at,
            look_in_node_modules,
            &mut always_installed,
        );

        // Half a TTL later a child asks, and inherits the root's answer.
        let later = resolved_at + (RESOLUTION_TTL / 2);
        cache.resolve(&src, Some(root_path), later, look_in_node_modules, &mut always_installed);

        // The age is inherited too, so both expire together.
        let child = cache.get(&src).expect("the child should be recorded");
        assert_eq!(child.resolved_at, resolved_at, "the child should inherit the root's instant");
        assert!(!child.is_fresh(resolved_at + RESOLUTION_TTL));
    }

    #[test]
    fn test_missing_executable_diagnostic_lists_relative_paths_and_the_remedy() {
        let cwd = PathBuf::from("/workspace");
        let paths = (0..7)
            .map(|index| arc_path(&cwd.join("packages").join("b").join(format!("{index}.ts"))))
            .collect::<Vec<_>>();

        let diagnostic = missing_executable_diagnostic(&paths, &cwd);
        let message = diagnostic.message.to_string();

        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(message.contains("7 file(s)"), "{message}");
        // Relative to `cwd`, like every other diagnostic.
        assert!(message.contains("packages/b/0.ts"), "{message}");
        assert!(!message.contains("/workspace"), "{message}");
        // Only the first few are listed, the rest is counted.
        assert!(!message.contains("packages/b/5.ts"), "{message}");
        assert!(message.contains("... and 2 more"), "{message}");

        // The remedy lives in the message, not in `help`, because `unix`, `checkstyle`,
        // `gitlab` and `junit` print the message and drop everything else.
        assert!(message.contains("oxlint-tsgolint"), "{message}");
        assert!(message.contains("OXLINT_TSGOLINT_PATH"), "{message}");
        assert!(diagnostic.help.is_none(), "{:?}", diagnostic.help);
    }

    /// The suppression keys are built from the plugin a rule belongs to, while the diagnostics
    /// `tsgolint` reports are built from [`TSGOLINT_DIAGNOSTIC_SCOPE`]. They have to agree, or
    /// the suppressions of a file which could not be linted would not be recognized.
    #[test]
    fn test_tsgolint_rules_live_under_the_scope_of_their_diagnostics() {
        let tsgolint_rules = crate::rules::RULES.iter().filter(|rule| rule.is_tsgolint_rule());

        let mut seen = 0;
        for rule in tsgolint_rules {
            assert_eq!(
                rule.plugin_name(),
                TSGOLINT_DIAGNOSTIC_SCOPE,
                "{} should be reported under `{TSGOLINT_DIAGNOSTIC_SCOPE}`",
                rule.name()
            );
            seen += 1;
        }
        assert!(seen > 0, "there should be some tsgolint rules to check");
    }

    #[test]
    fn test_no_executable_error_says_what_to_do() {
        let error = no_executable_error(3);

        assert!(error.contains("3 file(s)"), "{error}");
        assert!(error.contains("oxlint-tsgolint"), "{error}");
        assert!(error.contains("OXLINT_TSGOLINT_PATH"), "{error}");
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_basic() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: Some("Some help".into()),
            },
            fixes: vec![],
            suggestions: vec![],
            labeled_ranges: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.error.message, "Some description");
        assert_eq!(message.error.severity, Severity::Warning);
        assert_eq!(message.span, Span::new(0, 10));
        assert_eq!(
            message.error.code,
            OxcCode { scope: Some("typescript".into()), number: Some("some_rule".into()) }
        );
        assert!(!message.error.labels.is_empty());
        assert_eq!(message.error.labels.len(), 1);
        assert_eq!(message.error.labels[0], LabeledSpan::new(None, 0, 10));
        assert_eq!(message.error.help, Some("Some help".into()));
        assert!(message.fixes.is_empty());
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_fixes() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![
                Fix { text: "fixed".into(), range: Range { pos: 0, end: 5 } },
                Fix { text: "hello".into(), range: Range { pos: 5, end: 10 } },
            ],
            suggestions: vec![],
            labeled_ranges: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.fixes.len(), 1);
        assert_eq!(
            message.fixes,
            PossibleFixes::Single(crate::fixer::Fix {
                content: "fixedhello".into(),
                span: Span::new(0, 10),
                message: None,
                kind: FixKind::Fix
            })
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_multiple_suggestions() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![],
            suggestions: vec![
                Suggestion {
                    message: RuleMessage {
                        id: "some_id".into(),
                        description: "Suggestion 1".into(),
                        help: None,
                    },
                    fixes: vec![Fix { text: "hello".into(), range: Range { pos: 0, end: 5 } }],
                },
                Suggestion {
                    message: RuleMessage {
                        id: "some_id".into(),
                        description: "Suggestion 2".into(),
                        help: None,
                    },
                    fixes: vec![
                        Fix { text: "hello".into(), range: Range { pos: 0, end: 5 } },
                        Fix { text: "world".into(), range: Range { pos: 5, end: 10 } },
                    ],
                },
            ],
            labeled_ranges: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(
            message.fixes,
            PossibleFixes::Multiple(vec![
                crate::fixer::Fix {
                    content: "hello".into(),
                    span: Span::new(0, 5),
                    message: Some("Suggestion 1".into()),
                    kind: FixKind::Suggestion
                },
                crate::fixer::Fix {
                    content: "helloworld".into(),
                    span: Span::new(0, 10),
                    message: Some("Suggestion 2".into()),
                    kind: FixKind::Suggestion
                },
            ])
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_fix_and_suggestions() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![Fix { text: "fixed".into(), range: Range { pos: 0, end: 5 } }],
            suggestions: vec![Suggestion {
                message: RuleMessage {
                    id: "some_id".into(),
                    description: "Suggestion 1".into(),
                    help: None,
                },
                fixes: vec![Fix { text: "Suggestion 1".into(), range: Range { pos: 0, end: 5 } }],
            }],
            labeled_ranges: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.fixes.len(), 2);
        assert_eq!(
            message.fixes,
            PossibleFixes::Multiple(vec![
                crate::fixer::Fix {
                    content: "fixed".into(),
                    span: Span::new(0, 5),
                    message: None,
                    kind: FixKind::Fix
                },
                crate::fixer::Fix {
                    content: "Suggestion 1".into(),
                    span: Span::new(0, 5),
                    message: Some("Suggestion 1".into()),
                    kind: FixKind::Suggestion,
                },
            ])
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_labeled_ranges() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![],
            suggestions: vec![],
            labeled_ranges: vec![
                LabeledRange { label: "Label 1".into(), range: Range { pos: 0, end: 5 } },
                LabeledRange { label: "Label 2".into(), range: Range { pos: 5, end: 10 } },
            ],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert!(!message.error.labels.is_empty());
        let labels = &message.error.labels;
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0], LabeledSpan::new(Some("Label 1".into()), 0, 5));
        assert_eq!(labels[1], LabeledSpan::new(Some("Label 2".into()), 5, 5));
    }

    #[test]
    fn test_diagnostic_payload_deserialize_without_fixes_or_suggestions() {
        use super::TsGoLintDiagnosticPayload;

        // Test payload with both fixes and suggestions omitted
        let json = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "no-unused-vars",
            "message": {
                "id": "msg_id",
                "description": "Variable is unused",
                "help": null
            },
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.fixes.len(), 0);
        assert_eq!(payload.suggestions.len(), 0);
        assert_eq!(payload.rule, Some("no-unused-vars".to_string()));

        // Test payload with only fixes omitted
        let json_with_suggestions = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "no-unused-vars",
            "message": {
                "id": "msg_id",
                "description": "Variable is unused",
                "help": null
            },
            "suggestions": [
                {
                    "message": {
                        "id": "suggestion_id",
                        "description": "Remove unused variable",
                        "help": null
                    },
                    "fixes": []
                }
            ],
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload =
            serde_json::from_str(json_with_suggestions).unwrap();
        assert_eq!(payload.fixes.len(), 0);
        assert_eq!(payload.suggestions.len(), 1);

        // Test payload with only suggestions omitted
        let json_with_fixes = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "no-unused-vars",
            "message": {
                "id": "msg_id",
                "description": "Variable is unused",
                "help": null
            },
            "fixes": [
                {
                    "text": "fixed",
                    "range": {"pos": 0, "end": 5}
                }
            ],
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload = serde_json::from_str(json_with_fixes).unwrap();
        assert_eq!(payload.fixes.len(), 1);
        assert_eq!(payload.suggestions.len(), 0);
    }

    #[test]
    fn test_diagnostic_payload_deserialize_with_labeled_ranges() {
        use super::TsGoLintDiagnosticPayload;

        let json = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "some_rule",
            "message": {
                "id": "some_id",
                "description": "Some description",
                "help": null
            },
            "labeled_ranges": [
                {
                    "label": "Label 1",
                    "range": {"pos": 0, "end": 5}
                },
                {
                    "label": "Label 2",
                    "range": {"pos": 5, "end": 10}
                }
            ],
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.labeled_ranges.len(), 2);
        assert_eq!(payload.labeled_ranges[0].label, "Label 1");
        assert_eq!(payload.labeled_ranges[0].range.pos, 0);
        assert_eq!(payload.labeled_ranges[0].range.end, 5);
        assert_eq!(payload.labeled_ranges[1].label, "Label 2");
        assert_eq!(payload.labeled_ranges[1].range.pos, 5);
        assert_eq!(payload.labeled_ranges[1].range.end, 10);
    }

    #[test]
    fn test_timing_message_deserialize() {
        use super::{TsGoLintMessage, parse_single_message};

        let payload = serde_json::json!({
            "rules": [
                {"rule_name": "no-floating-promises", "duration": 123_456_u64, "calls": 7_u64}
            ]
        })
        .to_string();
        let payload = payload.as_bytes();

        let mut bytes = Vec::new();
        let payload_len: u32 = payload.len().try_into().expect("payload length should fit in u32");
        bytes.extend_from_slice(&payload_len.to_le_bytes());
        bytes.push(2);
        bytes.extend_from_slice(payload);

        let mut cursor = std::io::Cursor::new(bytes.as_slice());
        let message = parse_single_message(&mut cursor).unwrap_or_else(|err| panic!("{err}"));

        let TsGoLintMessage::Timing(payload) = message else {
            panic!("expected timing message");
        };
        assert_eq!(payload.rules.len(), 1);
        assert_eq!(payload.rules[0].rule_name, "no-floating-promises");
        assert_eq!(payload.rules[0].duration, 123_456);
        assert_eq!(payload.rules[0].calls, 7);
    }

    #[test]
    fn test_btreeset_preserves_rules_with_different_options() {
        use super::Rule;
        use std::collections::BTreeSet;

        // Create two rules with the same name but different options
        let rule1 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": true})),
        };

        let rule2 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": false})),
        };

        let rule3 = Rule { name: "no-floating-promises".to_string(), options: None };

        // Insert into BTreeSet
        let mut rules = BTreeSet::new();
        rules.insert(rule1.clone());
        rules.insert(rule2.clone());
        rules.insert(rule3.clone());

        // All three distinct rules should be preserved
        assert_eq!(rules.len(), 3, "BTreeSet should preserve all rules with different options");

        // Verify all rules are present
        assert!(rules.contains(&rule1), "Rule with ignoreVoid: true should be present");
        assert!(rules.contains(&rule2), "Rule with ignoreVoid: false should be present");
        assert!(rules.contains(&rule3), "Rule with no options should be present");
    }

    #[test]
    fn test_btreeset_deduplicates_identical_rules() {
        use super::Rule;
        use std::collections::BTreeSet;

        let rule1 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": true})),
        };

        let rule2 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": true})),
        };

        let mut rules = BTreeSet::new();
        rules.insert(rule1);
        rules.insert(rule2);

        // Identical rules should be deduplicated
        assert_eq!(rules.len(), 1, "BTreeSet should deduplicate identical rules");
    }

    /// The announced size is checked before the payload buffer is created, so four arbitrary
    /// bytes cannot trigger an allocation the process will not survive.
    ///
    /// Only the header is fed in: if the check were made after the allocation, the parser would
    /// report the truncated payload instead.
    #[test]
    fn parse_single_message_rejects_a_size_above_the_maximum() {
        let mut header = u32::MAX.to_le_bytes().to_vec();
        header.push(1); // `MessageType::Diagnostic`
        let mut cursor = std::io::Cursor::new(header.as_slice());

        let Err(error) = parse_single_message(&mut cursor) else {
            panic!("an impossible size should not parse");
        };
        assert!(
            matches!(error, TsGoLintMessageParseError::PayloadTooLarge(size) if size == u32::MAX as usize),
            "expected the size itself to be rejected, got: {error}"
        );
        assert_eq!(
            error.to_string(),
            "tsgolint announced a 4294967295 byte message, which cannot be right"
        );

        // The largest size still allowed is not rejected; it merely runs out of payload.
        let mut header = u32::try_from(MAX_MESSAGE_SIZE).unwrap().to_le_bytes().to_vec();
        header.push(1);
        let mut cursor = std::io::Cursor::new(header.as_slice());
        assert!(
            matches!(
                parse_single_message(&mut cursor),
                Err(TsGoLintMessageParseError::IncompleteData)
            ),
            "the boundary itself should be accepted"
        );
    }
}

/// Tests for the `tsgolint` payload built by [`TsGoLintState::json_input`].
///
/// The `type_check` field is per config group, so these assert on the payload itself: the
/// released `tsgolint` binaries used by the CLI fixtures ignore the field, and would report the
/// same diagnostics either way.
#[cfg(test)]
mod json_input_test {
    use std::{ffi::OsStr, path::PathBuf, sync::Arc};

    use rustc_hash::FxHashMap;
    use serde_json::{Value, json};

    use super::{Config, Payload, TsGoLintState};
    use crate::{
        ExternalPluginStore,
        config::{ConfigStore, ConfigStoreBuilder, Oxlintrc},
    };

    const ROOT_FILE: &str = "/root/index.ts";
    const NESTED_FILE: &str = "/root/packages/app/index.ts";

    /// A config which runs a single type-aware rule, plus the given `options`.
    fn type_aware_config(options: &Value) -> Value {
        json!({
            "categories": { "correctness": "off" },
            "options": options,
            "rules": { "typescript/no-floating-promises": "error" },
        })
    }

    /// A store whose root config governs `/root` and whose nested config governs
    /// `/root/packages/app`, both running the same type-aware rule.
    fn store_with_nested_options(root_options: Value, nested_options: Value) -> ConfigStore {
        let mut external_plugin_store = ExternalPluginStore::default();
        let mut build = |options: Value| {
            let oxlintrc: Oxlintrc = serde_json::from_value(type_aware_config(&options)).unwrap();
            ConfigStoreBuilder::from_oxlintrc(
                false,
                oxlintrc,
                None,
                &mut external_plugin_store,
                None,
            )
            .unwrap()
            .build(&mut external_plugin_store)
            .unwrap()
        };
        let root = build(root_options);
        let nested = build(nested_options);
        let mut nested_configs = FxHashMap::default();
        nested_configs.insert(PathBuf::from("/root/packages/app"), nested);
        ConfigStore::new(root, nested_configs, external_plugin_store)
    }

    fn payload_for(config_store: ConfigStore, type_check_override: Option<bool>) -> Payload {
        let state = TsGoLintState::for_test(&PathBuf::from("/root"), config_store, None)
            .with_type_check_override(type_check_override);
        let paths: Vec<Arc<OsStr>> =
            vec![Arc::from(OsStr::new(ROOT_FILE)), Arc::from(OsStr::new(NESTED_FILE))];
        let mut resolved_configs = FxHashMap::default();
        state.json_input(&paths, None, &mut resolved_configs)
    }

    fn group_for<'p>(payload: &'p Payload, file_path: &str) -> &'p Config {
        payload
            .configs
            .iter()
            .find(|config| config.file_paths.iter().any(|path| path == file_path))
            .unwrap_or_else(|| panic!("no config group lists {file_path}"))
    }

    #[test]
    fn payload_splits_groups_with_different_type_check_even_with_identical_rules() {
        // Both configs enable exactly the same rule, so the only thing separating the two files
        // is `type_check`. A group carries a single value of the field, so they must not merge.
        let payload = payload_for(
            store_with_nested_options(json!({ "typeAware": true }), {
                json!({ "typeAware": true, "typeCheck": true })
            }),
            None,
        );

        assert_eq!(payload.configs.len(), 2);
        let root = group_for(&payload, ROOT_FILE);
        let nested = group_for(&payload, NESTED_FILE);
        assert_eq!(root.rules, nested.rules, "the two groups must differ only by `type_check`");
        assert!(!root.type_check);
        assert!(nested.type_check);
    }

    #[test]
    fn payload_always_serializes_type_check() {
        // `tsgolint` reads an absent `type_check` as `true`, so a `false` group which skipped the
        // field would be type-checked after all. Assert on the JSON, not on the struct, because
        // that default is what a `skip_serializing_if` would reintroduce.
        let payload = payload_for(
            store_with_nested_options(json!({ "typeAware": true }), {
                json!({ "typeAware": true, "typeCheck": true })
            }),
            None,
        );

        let json = serde_json::to_value(&payload).unwrap();
        let configs = json["configs"].as_array().expect("`configs` should be an array");
        assert_eq!(configs.len(), 2);
        let mut serialized: Vec<bool> = configs
            .iter()
            .map(|config| {
                config
                    .as_object()
                    .expect("a config group should be an object")
                    .get("type_check")
                    .unwrap_or_else(|| panic!("`type_check` missing from {config}"))
                    .as_bool()
                    .expect("`type_check` should be a JSON boolean")
            })
            .collect();
        // Both values are spelled out, rather than one of them being left to the default.
        serialized.sort_unstable();
        assert_eq!(serialized, vec![false, true]);
    }

    #[test]
    fn payload_enables_report_flags_when_any_group_type_checks() {
        // `report_syntactic` / `report_semantic` select the *kinds* of diagnostic for the whole
        // run, so `type_check: true` on a single group is a no-op without them.
        let payload = payload_for(
            store_with_nested_options(json!({ "typeAware": true }), {
                json!({ "typeAware": true, "typeCheck": true })
            }),
            None,
        );
        assert!(payload.report_syntactic);
        assert!(payload.report_semantic);

        // Neither kind is enabled when no file is type-checked.
        let payload = payload_for(
            store_with_nested_options(json!({ "typeAware": true }), json!({ "typeAware": true })),
            None,
        );
        assert!(!payload.report_syntactic);
        assert!(!payload.report_semantic);
        assert!(payload.configs.iter().all(|config| !config.type_check));
    }

    #[test]
    fn payload_type_check_is_not_inherited_by_a_nested_config() {
        // Only the *root* config enables type checking here. `options` is never inherited, so
        // the nested package takes the default and its files are sent with `type_check: false`.
        // Sharing the option requires `extends`.
        let payload = payload_for(
            store_with_nested_options(
                json!({ "typeAware": true, "typeCheck": true }),
                json!({ "typeAware": true }),
            ),
            None,
        );

        assert!(group_for(&payload, ROOT_FILE).type_check);
        assert!(!group_for(&payload, NESTED_FILE).type_check);
    }

    #[test]
    fn cli_type_check_flag_forces_every_group() {
        // `--type-check` / `--type-check-only` type-check every file, regardless of the config
        // which governs it. The two files now agree, so they share a single group.
        let payload = payload_for(
            store_with_nested_options(json!({ "typeAware": true }), {
                json!({ "typeAware": true, "typeCheck": false })
            }),
            Some(true),
        );

        assert_eq!(payload.configs.len(), 1);
        assert!(payload.configs[0].type_check);
        assert_eq!(payload.configs[0].file_paths.len(), 2);
        assert!(payload.report_syntactic);
        assert!(payload.report_semantic);
    }

    #[test]
    fn editor_type_check_false_disables_every_group() {
        // The editor's `typeCheck: false` overrides a config which turns it on. The files are
        // still linted with type-aware rules, they just report no TypeScript diagnostics, so no
        // diagnostic kind is enabled either.
        let payload = payload_for(
            store_with_nested_options(
                json!({ "typeAware": true, "typeCheck": true }),
                json!({ "typeAware": true, "typeCheck": true }),
            ),
            Some(false),
        );

        assert_eq!(payload.configs.len(), 1);
        assert!(!payload.configs[0].type_check);
        assert_eq!(payload.configs[0].file_paths.len(), 2);
        assert!(!payload.report_syntactic);
        assert!(!payload.report_semantic);
    }
}
