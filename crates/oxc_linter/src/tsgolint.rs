use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fmt::Write as _,
    io::{ErrorKind, Read, Write, stderr},
    iter, mem,
    path::{Path, PathBuf},
    process::{Child, ExitStatus},
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::Duration,
};

use oxc_allocator::Allocator;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use oxc_diagnostics::{
    DiagnosticSender, DiagnosticService, Error, NamedSource, OxcDiagnostic, Severity,
};
use oxc_span::{SourceType, Span};

use super::{AllowWarnDeny, ConfigStore, DisableDirectives, ResolvedLinterState, read_to_string};

use crate::{
    CompositeFix, DirectivesStore, FixKind, Fixer, Message, PossibleFixes, RuleEnum,
    RuleTimingRecord, RuleTimingSource, RuleTimingStore, WEBSITE_BASE_RULES_URL,
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
///   `OXLINT_TSGOLINT_PATH`, a walk up to the file system root, or `PATH`. The walks of one
///   plan share a memo which every visited directory is recorded in, so a directory is looked
///   at once per plan, and a directory whose ancestor is already resolved never looks at the
///   file system above it. The walks keep nothing from one plan to the next, so an installation
///   or a removal below the working directory is seen by the next lint; the executable resolved
///   for the working directory itself lives with the state, as it does without this module, and
///   is looked up once per [`TsGoLintState`]. The working directory is only a boundary for the
///   files below it: a file outside it walks to the file system root instead, since linting a
///   sibling package is a supported use case.
///   (`per_file_walk_stops_at_cwd_and_the_cwd_walk_continues_above_it`,
///   `a_plan_does_not_stat_a_directory_twice`,
///   `test_memo_records_every_visited_directory`,
///   `a_file_outside_cwd_walks_above_it_instead_of_stopping`,
///   `lints_a_package_outside_the_working_directory`,
///   `lints_the_package_which_has_tsgolint_and_reports_the_one_which_does_not`)
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
///   group whose `tsgolint` failed as much as for the files which had none, and for the files
///   whose governing config does not set `options.typeAware` in a run which is type-aware
///   elsewhere, which are never handed to `tsgolint`. It holds in the editor as much as on the
///   command line, quick-fix included. A directive which names a rule that
///   ran *and* one that did not keeps reporting the first, per rule, and is never offered for
///   deletion as a whole.
///   (`keeps_the_type_aware_directives_of_the_files_which_could_not_be_linted`,
///   `keeps_the_type_aware_directives_of_the_files_whose_config_is_not_type_aware`,
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
/// - **Groups are keyed by installation, and their executable is deterministic.** The key of a
///   `tsgolint` shim is the canonical `node_modules/oxlint-tsgolint` directory, so two shims of
///   one installation share a process; a `tsrs-lint` shim is keyed by its own canonical path,
///   so it never shares a process with the `tsgolint` installed next to it. The
///   lexicographically smallest shim is the one run.
///   (`test_executable_group_key_identifies_the_package`,
///   `test_executable_group_key_follows_the_package_manager_store`,
///   `test_executable_group_key_keeps_tsrs_lint_apart_from_tsgolint`,
///   `test_group_paths_by_executable_picks_one_shim_deterministically`)
/// - **Groups run `OXLINT_TSGOLINT_JOBS` at a time, one by default.** Every `tsgolint` instance
///   can take several GB, and the groups overlap the regular pass, so they run one after the
///   other unless the variable names a larger positive number; an empty or invalid value means
///   the default.
///   (`concurrency_follows_the_requested_cap`, `requested_jobs_reads_a_positive_number`)
/// - **Go profiling is meant for a single group.** Every process is given the same
///   `OXLINT_TSGOLINT_{ALLOCS,CPU,HEAP,TRACE}` file names, so with several groups the profiles
///   overwrite each other, and corrupt each other when `OXLINT_TSGOLINT_JOBS` lets the groups
///   overlap. Profile one installation at a time.
/// - **Groups start before the regular pass, unless it writes files; planning always precedes
///   it.** Every group is started by [`TsGoLintState::start`], the first ones at once and the
///   others as soon as a worker is free, so they overlap the regular pass instead of waiting
///   for [`TsGoLintRun::finish`]. With fixes on the regular pass rewrites files, so the groups
///   start after it, from the rewritten files.
///   (`all_groups_run_before_finish_is_called`,
///   `fix_starts_tsgolint_after_the_regular_pass`)
/// - **Dropping a started run stops every group.** The processes which are running are killed
///   and the groups which have not started never do.
///   (`dropping_a_started_run_stops_every_group`)
#[derive(Debug, Clone)]
pub struct TsGoLintState {
    /// The `tsgolint` executable resolved from [`Self::cwd`], see [`Fallback`].
    fallback: Fallback,
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
    /// If `true`, include TypeScript compiler syntactic and semantic diagnostics.
    type_check: bool,
    /// If `true`, skip type-aware lint rules while retaining TypeScript diagnostics.
    type_check_only: bool,
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
        Ok(Self::new(
            cwd,
            config_store,
            resolve_tsgolint_executable(cwd)?,
            fix_kind.contains(FixKind::Fix),
            fix_kind.contains(FixKind::Suggestion),
            false,
        ))
    }

    fn new(
        cwd: &Path,
        config_store: ConfigStore,
        fallback: Fallback,
        fix: bool,
        fix_suggestions: bool,
        type_aware_forced: bool,
    ) -> Self {
        TsGoLintState {
            config_store,
            fallback,
            cwd: cwd.to_path_buf(),
            silent: false,
            fix,
            fix_suggestions,
            type_check: false,
            type_check_only: false,
            timings: false,
            with_ignore_fixes: false,
            type_aware_forced,
        }
    }

    /// A state which uses `executable` for every file, or none at all, so the tests do not
    /// depend on what the machine running them happens to have installed.
    #[cfg(test)]
    pub(crate) fn for_test(
        cwd: &Path,
        config_store: ConfigStore,
        executable: Option<PathBuf>,
    ) -> Self {
        let fallback = executable
            .map_or(Fallback::None, |path| Fallback::Discovered(ResolvedExecutable::new(path)));
        Self::new(cwd, config_store, fallback, false, false, true)
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

    /// Set to `true` to include TypeScript compiler syntactic and semantic diagnostics.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_type_check(mut self, yes: bool) -> Self {
        self.type_check = yes;
        self
    }

    /// Set to `true` to skip type-aware lint rules.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_type_check_only(mut self, yes: bool) -> Self {
        self.type_check_only = yes;
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
        resolved_configs: &FxHashMap<PathBuf, ResolvedLinterState>,
        linted: &[Arc<OsStr>],
        not_linted: &[Arc<OsStr>],
    ) {
        type Rules = Arc<[(RuleEnum, AllowWarnDeny)]>;

        directives_store.clear_rules_not_run(linted.iter().map(|path| Path::new(path.as_ref())));
        // The names only depend on the rules, which the files governed by one config share. The
        // memo holds the rules it is keyed by, so their address is not reused by another file's.
        let mut names_by_rules: FxHashMap<usize, (Rules, Arc<FxHashSet<String>>)> =
            FxHashMap::default();
        for path in not_linted {
            let file = Path::new(path.as_ref());
            let resolved = self.resolved_config(resolved_configs, file);
            let (_, names) =
                names_by_rules.entry(Arc::as_ptr(&resolved.rules).addr()).or_insert_with(|| {
                    (Arc::clone(&resolved.rules), Arc::new(type_aware_directive_names(&resolved)))
                });
            directives_store.set_rules_not_run(file.to_path_buf(), Arc::clone(names));
        }
    }

    /// The suppression keys of the rules `tsgolint` would have run on `file`.
    ///
    /// Only the rules configured at [`AllowWarnDeny::Deny`] are included: the suppression
    /// tracker only counts error-severity diagnostics, so a rule at `warn` never has a
    /// suppression to preserve. This is computed only for the files which actually have
    /// recorded suppressions.
    fn type_aware_suppression_keys(
        &self,
        resolved_configs: &FxHashMap<PathBuf, ResolvedLinterState>,
        file: &Path,
    ) -> FxHashSet<String> {
        self.resolved_config(resolved_configs, file)
            .rules
            .iter()
            .filter(|(rule, status)| *status == AllowWarnDeny::Deny && rule.is_tsgolint_rule())
            .map(|(rule, _)| format!("{}/{}", rule.plugin_name(), rule.name()))
            .collect()
    }

    /// The config which applies to `file`: the one [`TsGoLintState::plan`] resolved, or, for
    /// `--type-check-only` which resolves none, resolved now.
    fn resolved_config<'a>(
        &self,
        resolved_configs: &'a FxHashMap<PathBuf, ResolvedLinterState>,
        file: &Path,
    ) -> Cow<'a, ResolvedLinterState> {
        resolved_configs
            .get(file)
            .map_or_else(|| Cow::Owned(self.config_store.resolve(file)), Cow::Borrowed)
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
        // What the walks of this plan learned about each directory, so a directory is looked at
        // once per plan. It is dropped with the plan: the language server plans on every lint,
        // and must notice an installation or a removal.
        let mut memo = ExecutableMemo::default();

        // The governing config of a file is looked up once: it decides whether the file is
        // linted with type-aware rules, and gives the rules `json_input` sends for it.
        let mut resolved_configs = FxHashMap::default();
        let mut not_type_aware = Vec::new();
        let mut plan = group_paths_by_executable(
            paths,
            |file| {
                let lints = if SourceType::from_path(file).is_err() {
                    // `.vue`, `.svelte`, `.astro`, ...: only the regular pass lints it
                    false
                } else if self.type_check_only {
                    // `--type-check-only` type-checks every file and runs no rule, so the config
                    // which governs a file plays no part.
                    true
                } else {
                    let config = self.config_store.get_related_config(file);
                    resolved_configs.insert(file.to_path_buf(), config.apply_overrides(file));
                    self.type_aware_forced || config.type_aware().unwrap_or(false)
                };
                if !lints {
                    // Not handed to `tsgolint`, so its type-aware rules do not run either.
                    not_type_aware.push(Arc::from(file.as_os_str()));
                }
                lints
            },
            |dir| self.resolve_executable_for_dir(dir, &mut memo),
        );
        plan.resolved_configs = resolved_configs;
        plan.not_type_aware = not_type_aware;
        plan
    }

    /// Resolve the `tsgolint` executable which should lint the files in `dir`.
    ///
    /// The lookup starts at `dir` and walks up to [`Self::cwd`], so that a package which
    /// installs its own `tsgolint` uses that one, and falls back to the executable resolved for
    /// [`Self::cwd`]. Every directory the walk looked at is recorded in `memo`, so the siblings
    /// of an already resolved directory cost nothing.
    fn resolve_executable_for_dir(
        &self,
        dir: &Path,
        memo: &mut ExecutableMemo,
    ) -> Option<ResolvedExecutable> {
        if self.fallback.is_pinned() {
            return self.fallback.executable().cloned();
        }

        resolve_with_memo(dir, walk_boundary_for(dir, &self.cwd), memo, look_in_node_modules)
            .or_else(|| self.fallback.executable().cloned())
    }

    /// Runs `tsgolint` on the files of `plan` to completion. Equivalent to [`Self::start`]
    /// followed by [`TsGoLintRun::finish`].
    ///
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
        self.start(plan).finish(
            directives_store,
            error_sender,
            file_system,
            diff_manager,
            rule_timing_store,
        )
    }

    /// Starts `tsgolint` for the groups of `plan` on a background thread, so that it can run
    /// while the regular lint pass does.
    ///
    /// The groups run `OXLINT_TSGOLINT_JOBS` at a time, one by default, each starting as soon
    /// as the one before it is done. Nothing is reported until [`TsGoLintRun::finish`], which must run
    /// after the regular pass: diagnostics are filtered by the disable directives that pass
    /// collects. Failures to run a group are reported by `finish` as well.
    pub fn start(self, plan: LintPlan) -> TsGoLintRun {
        if plan.is_empty() {
            return TsGoLintRun { started: None };
        }

        let state = Arc::new(self);
        let plan = Arc::new(plan);
        let children = Arc::new(RunningChildren::default());

        let mut senders = Vec::with_capacity(plan.groups.len());
        let mut messages = Vec::with_capacity(plan.groups.len());
        for _ in &plan.groups {
            let (sender, receiver) = mpsc::channel();
            senders.push(sender);
            messages.push(receiver);
        }

        let pool = (!plan.groups.is_empty()).then(|| {
            let state = Arc::clone(&state);
            let plan = Arc::clone(&plan);
            let children = Arc::clone(&children);
            let jobs_at_once = concurrent_groups();
            std::thread::spawn(move || {
                let jobs: Vec<GroupJob<'_>> = plan
                    .groups
                    .iter()
                    .zip(senders)
                    .enumerate()
                    .map(|(index, (group, sender))| GroupJob {
                        index,
                        group,
                        sender: Mutex::new(Some(sender)),
                    })
                    .collect();
                run_groups(&jobs, jobs_at_once, |job| {
                    state.run_group(job, &plan.resolved_configs, &children)
                })
            })
        });

        TsGoLintRun { started: Some(StartedRun { state, plan, messages, children, pool }) }
    }
}

/// A group to run, and where to send what its `tsgolint` writes.
struct GroupJob<'a> {
    /// The position of the group in the plan.
    index: usize,
    group: &'a LintGroup,
    /// Taken by the worker which runs the group, so that the channel closes as soon as this
    /// group is done instead of when every group is.
    sender: Mutex<Option<mpsc::Sender<Result<TsGoLintMessage, String>>>>,
}

/// The `tsgolint` processes of a run which are still alive, so that dropping the run can stop
/// them.
#[derive(Default)]
struct RunningChildren(Mutex<RunningChildrenState>);

#[derive(Default)]
struct RunningChildrenState {
    cancelled: bool,
    /// By group index.
    children: FxHashMap<usize, Child>,
}

impl RunningChildren {
    fn is_cancelled(&self) -> bool {
        lock(&self.0).cancelled
    }

    /// Registering under the same lock as the cancellation closes the race between a worker
    /// checking [`Self::is_cancelled`] and spawning its process.
    fn register(&self, index: usize, mut child: Child) {
        let mut state = lock(&self.0);
        if state.cancelled {
            let _ = child.kill();
        }
        state.children.insert(index, child);
    }

    fn kill(&self, index: usize) {
        if let Some(child) = lock(&self.0).children.get_mut(&index) {
            let _ = child.kill();
        }
    }

    fn take(&self, index: usize) -> Option<Child> {
        lock(&self.0).children.remove(&index)
    }

    /// Kills every running process. They stay registered, so that the worker which spawned each
    /// of them still reaps it.
    fn cancel(&self) {
        let mut state = lock(&self.0);
        state.cancelled = true;
        for child in state.children.values_mut() {
            let _ = child.kill();
        }
    }
}

impl TsGoLintState {
    /// Run `tsgolint` for one group, sending what it writes to the group's channel as it
    /// arrives so that it never blocks on a full pipe. Returns once the process has exited,
    /// with its status; `None` when there was nothing to send it.
    ///
    /// # Errors
    /// A human-readable error message indicating why the group could not be run.
    fn run_group(
        &self,
        job: &GroupJob<'_>,
        resolved_configs: &FxHashMap<PathBuf, ResolvedLinterState>,
        children: &RunningChildren,
    ) -> Result<Option<ExitStatus>, String> {
        let Some(sender) = lock(&job.sender).take() else {
            return Err("tsgolint group was already run".to_string());
        };
        if children.is_cancelled() {
            return Err("tsgolint run was cancelled".to_string());
        }

        let LintGroup { executable, paths } = job.group;
        let json_input = self.json_input(paths, None, resolved_configs);
        if json_input.configs.is_empty() {
            return Ok(None);
        }

        let mut child = self.spawn_tsgolint(executable, &json_input)?;
        let Some(stdout) = child.stdout.take() else {
            discard_child(&mut child);
            return Err("could not open tsgolint stdout".to_string());
        };
        children.register(job.index, child);

        // Message processing stops at the first error, so stop reading there too.
        for msg in TsGoLintMessageStream::new(stdout) {
            let is_error =
                !matches!(msg, Ok(TsGoLintMessage::Diagnostic(_) | TsGoLintMessage::Timing(_)));
            if sender.send(msg).is_err() {
                // Nobody reads this group any more, the run was dropped.
                children.kill(job.index);
                break;
            }
            if is_error {
                break;
            }
        }
        // The stream, and with it stdout, is gone: a `tsgolint` which keeps writing after
        // malformed output gets a broken pipe instead of blocking the wait below.
        drop(sender);

        let Some(mut child) = children.take(job.index) else {
            return Err("could not find the tsgolint process".to_string());
        };
        child.wait().map(Some).map_err(|error| format!("could not await tsgolint: {error}"))
    }
}

/// A `tsgolint` run started by [`TsGoLintState::start`].
pub struct TsGoLintRun {
    /// `None` when there was nothing to lint.
    started: Option<StartedRun>,
}

/// How each group's process ended, in group order.
type GroupExits = Vec<Result<Option<ExitStatus>, String>>;

struct StartedRun {
    state: Arc<TsGoLintState>,
    plan: Arc<LintPlan>,
    /// What each group's `tsgolint` wrote, in group order, decoded as it arrives.
    messages: Vec<mpsc::Receiver<Result<TsGoLintMessage, String>>>,
    children: Arc<RunningChildren>,
    /// Runs the groups, and returns how each of them ended. `None` when there are no groups.
    pool: Option<JoinHandle<GroupExits>>,
}

impl TsGoLintRun {
    /// Waits for every group to finish and reports its diagnostics, filtered by the disable
    /// directives that the regular lint pass collected, then applies its fixes.
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    pub fn finish(
        mut self,
        directives_store: &DirectivesStore,
        error_sender: DiagnosticSender,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        diff_manager: &Arc<DiffManager>,
        rule_timing_store: Option<&RuleTimingStore>,
    ) -> Result<(), String> {
        let Some(StartedRun { state, plan, messages, pool, .. }) = self.started.take() else {
            return Ok(());
        };
        let LintPlan { groups, unresolved, resolved_configs, .. } = &*plan;

        // The files which could not be linted keep the suppressions recorded for their
        // type-aware rules, instead of looking like suppressions which no longer fire.
        for path in plan.skipped_paths() {
            let file = Path::new(path.as_ref());
            diff_manager.collect_type_aware_skipped_file(file, &state.cwd, || {
                state.type_aware_suppression_keys(resolved_configs, file)
            });
        }

        let disable_directives_map = directives_store.map();
        let outputs: Vec<Result<TsGoLintOutput, String>> = groups
            .iter()
            .zip(messages)
            .map(|(group, messages)| {
                // The regular pass is over, so nothing else needs the directives while a group
                // is consumed.
                let disable_directives_map = lock(&disable_directives_map);
                state.collect_group(
                    group,
                    messages,
                    resolved_configs,
                    &disable_directives_map,
                    error_sender.clone(),
                    diff_manager,
                )
            })
            .collect();

        let exits = pool.map_or_else(Vec::new, |pool| {
            pool.join().unwrap_or_else(|_| {
                vec![Err("tsgolint group did not run".to_string()); groups.len()]
            })
        });

        let results: Vec<Result<(), String>> = exits
            .into_iter()
            .zip(outputs)
            .map(|(exit, output)| {
                group_result(exit, output).map(|output| {
                    state.apply_group_output(
                        output,
                        file_system,
                        diff_manager,
                        rule_timing_store,
                        &error_sender,
                    );
                })
            })
            .collect();

        let (failed, linted): (Vec<_>, Vec<_>) =
            results.iter().zip(groups).partition(|(result, _)| result.is_err());
        let mut not_linted = plan.skipped_paths().cloned().collect::<Vec<_>>();
        not_linted.extend(failed.iter().flat_map(|(_, group)| group.paths.iter().cloned()));
        let linted =
            linted.iter().flat_map(|(_, group)| group.paths.iter().cloned()).collect::<Vec<_>>();
        state.record_type_aware_coverage(directives_store, resolved_configs, &linted, &not_linted);

        // Reported once, after the groups which could run did, so that a missing executable
        // never hides the diagnostics of the packages which do have one.
        if let Some(first) = unresolved.first() {
            // Best effort: the receiver is only gone when the run is already being torn down.
            let _ = error_sender.send(attach_to_file(
                missing_executable_diagnostic(unresolved, &state.cwd),
                Path::new(first.as_ref()),
                &state.cwd,
            ));
        }

        // A group which could not run is reported here rather than left to the caller, because
        // this is where the files it was meant to lint are still known.
        for (group, error) in results
            .iter()
            .zip(groups)
            .filter_map(|(result, group)| result.as_ref().err().map(|error| (group, error)))
        {
            let Some(first) = group.paths.first() else { continue };
            let _ = error_sender.send(attach_to_file(
                group_failure_diagnostic(group, error),
                Path::new(first.as_ref()),
                &state.cwd,
            ));
        }

        drop(error_sender);

        // Every group was run; report all the ones which failed, not just the first.
        join_group_errors(groups, results)
    }
}

impl TsGoLintState {
    /// Report the diagnostics `tsgolint` wrote for one group as they arrive, and collect what
    /// the fixes need.
    ///
    /// # Errors
    /// A human-readable error message indicating why the output could not be processed.
    fn collect_group(
        &self,
        group: &LintGroup,
        messages: mpsc::Receiver<Result<TsGoLintMessage, String>>,
        resolved_configs: &FxHashMap<PathBuf, ResolvedLinterState>,
        disable_directives_map: &FxHashMap<PathBuf, DisableDirectives>,
        error_sender: DiagnosticSender,
        diff_manager: &Arc<DiffManager>,
    ) -> Result<TsGoLintOutput, String> {
        let should_fix = self.fix || self.fix_suggestions;
        let mut diagnostic_handler =
            DiagnosticHandler::new(self.cwd.clone(), self.silent, should_fix, error_sender);
        let mut timings = vec![];

        for msg in messages {
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
                                    tsgolint_rule_severity(resolved_config, &tsgolint_diagnostic.rule)
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

                            if should_skip_diagnostic(
                                disable_directives_map,
                                path,
                                &tsgolint_diagnostic,
                            ) {
                                continue;
                            }

                            diagnostic_handler.handle_rule_diagnostic(
                                tsgolint_diagnostic,
                                severity,
                                diff_manager.skip(),
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

        // The group's files, in input order: `TsGoLintState::plan` kept only the files
        // `tsgolint` lints, and kept their order.
        let all_paths: Vec<PathBuf> = group.paths.iter().map(PathBuf::from).collect();
        Ok(TsGoLintOutput {
            messages_requiring_fixes: diagnostic_handler
                .into_messages_requiring_fixes(diff_manager, all_paths),
            timings,
        })
    }

    /// Record the timings of a group, apply its fixes and report what they left.
    fn apply_group_output(
        &self,
        output: TsGoLintOutput,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        diff_manager: &Arc<DiffManager>,
        rule_timing_store: Option<&RuleTimingStore>,
        error_sender: &DiagnosticSender,
    ) {
        let cwd = &self.cwd;
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
                let _ = error_sender.send(vec![
                    OxcDiagnostic::error(format!(
                        "Failed to write file {} with error \"{error}\"",
                        path.display()
                    ))
                    .into(),
                ]);
            }

            if fix_result.messages.is_empty() {
                diff_manager.collect_empty_file(path.as_path(), cwd);
            } else {
                let source_for_diagnostics: &str =
                    if fix_result.fixed { &fix_result.fixed_code } else { &source_text };

                let filtered_messages: Vec<OxcDiagnostic> = if diff_manager.skip() {
                    fix_result.messages.into_iter().map(Into::into).collect()
                } else {
                    diff_manager
                        .collect_file(
                            path.as_path(),
                            cwd,
                            fix_result.messages.into_iter().collect(),
                        )
                        .into_iter()
                        .map(Into::into)
                        .collect()
                };

                let diagnostics = DiagnosticService::wrap_diagnostics(
                    cwd,
                    &path,
                    source_for_diagnostics,
                    filtered_messages,
                );
                let _ = error_sender.send(diagnostics);
            }
        }
    }
}

/// Combine how a group's process ended with what its output turned out to be.
///
/// The output is reported first when the process failed, because it is the better explanation.
fn group_result(
    exit: Result<Option<ExitStatus>, String>,
    output: Result<TsGoLintOutput, String>,
) -> Result<TsGoLintOutput, String> {
    let status = match exit {
        Err(error) => return Err(error),
        Ok(None) => {
            return Ok(TsGoLintOutput { messages_requiring_fixes: vec![], timings: vec![] });
        }
        Ok(Some(status)) => status,
    };

    if !status.success() {
        return Err(format!(
            "Error running tsgolint: {}",
            match output {
                Err(error) => format!("{status}, error: {error}"),
                Ok(_) => format!("{status}"),
            }
        ));
    }

    output.map_err(|error| format!("Error running tsgolint: {status}, error: {error}"))
}

/// Kills a `tsgolint` process which cannot be used and reaps it, so that it neither keeps
/// running nor stays a zombie for as long as `oxlint` or the language server does.
fn discard_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

impl Drop for TsGoLintRun {
    /// Stops every group when the run is dropped without [`TsGoLintRun::finish`] (e.g. the
    /// regular pass unwound), so no process outlives the run and no group which has not started
    /// ever does.
    ///
    /// The pool is not joined: on Windows, killing a `.CMD` shim does not close the pipe its
    /// child holds, so the join could wait for `tsgolint` to finish. Its workers reap the killed
    /// processes themselves.
    fn drop(&mut self) {
        if let Some(run) = self.started.take() {
            run.children.cancel();
        }
    }
}

impl TsGoLintState {
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

        for (env_name, flag) in PROFILING_ENV {
            if let Ok(file) = std::env::var(env_name) {
                cmd.arg(format!("-{flag}={file}"));
            }
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

        // The process exists from here on: it must not outlive an error.
        let Some(mut stdin) = child.stdin.take() else {
            discard_child(&mut child);
            return Err("could not open tsgolint stdin".to_string());
        };

        let json = match serde_json::to_string(json_input) {
            Ok(json) => json,
            Err(error) => {
                discard_child(&mut child);
                return Err(format!("could not serialize the tsgolint payload: {error}"));
            }
        };
        if let Err(e) = stdin.write_all(json.as_bytes())
            && e.kind() != ErrorKind::BrokenPipe
        {
            discard_child(&mut child);
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
        let plan = self.plan(paths);
        if plan.is_empty() {
            return Ok(vec![]);
        }
        let LintPlan { groups, unresolved, resolved_configs, .. } = &plan;

        let disable_directives_map = &directives_store.map();
        let results = run_groups(groups, concurrent_groups(), |group| {
            self.lint_source_group(group, resolved_configs, file_system, disable_directives_map)
        });

        // Merged in group order, which `plan` made deterministic.
        let mut messages = vec![];
        let mut failures = Vec::new();
        let mut linted = Vec::new();
        let mut not_linted = plan.skipped_paths().cloned().collect::<Vec<_>>();
        for (result, group) in results.into_iter().zip(groups) {
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
        self.record_type_aware_coverage(directives_store, resolved_configs, &linted, &not_linted);

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
        messages.extend(missing_executable_messages(unresolved));

        Ok(messages)
    }

    /// Lint `paths` from their in-memory sources with a single `tsgolint` executable.
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    fn lint_source_group(
        &self,
        group: &LintGroup,
        resolved_configs: &FxHashMap<PathBuf, ResolvedLinterState>,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        disable_directives_map: &Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
    ) -> Result<Vec<Message>, String> {
        let LintGroup { executable, paths } = group;
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

        let json_input = self.json_input(paths, Some(source_overrides.clone()), resolved_configs);
        if json_input.configs.is_empty() {
            // No file in this batch is governed by a config which enables type-aware linting.
            return Ok(vec![]);
        }

        //  Get the file name of the first path for internal diagnostic filtering
        let path_file_name =
            Path::new(paths[0].as_ref()).file_name().unwrap_or_default().to_os_string();

        let mut child = self.spawn_tsgolint(executable, &json_input)?;
        let Some(stdout) = child.stdout.take() else {
            discard_child(&mut child);
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

                                let severity = tsgolint_rule_severity(
                                    resolved_config,
                                    &tsgolint_diagnostic.rule,
                                );
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
        discard_child(&mut child);

        diagnostics
    }

    /// Create a JSON input for STDIN of tsgolint in this format:
    ///
    /// ```json
    /// {
    ///   "files": [
    ///     {
    ///       "file_path": "/absolute/path/to/file.ts",
    ///       "rules": ["rule-1", "another-rule"]
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// `paths` is already filtered by [`TsGoLintState::plan`]: every entry has a recognized
    /// source type and is governed by a config which enables type-aware linting, and
    /// `resolved_configs` holds the config `plan` resolved for it.
    #[inline]
    fn json_input(
        &self,
        paths: &[Arc<OsStr>],
        source_overrides: Option<FxHashMap<String, String>>,
        resolved_configs: &FxHashMap<PathBuf, ResolvedLinterState>,
    ) -> Payload {
        if self.type_check_only {
            let file_paths: Vec<String> =
                paths.iter().map(|path| path.to_string_lossy().into_owned()).collect();
            return Payload {
                version: 2,
                configs: if file_paths.is_empty() {
                    vec![]
                } else {
                    vec![Config { file_paths, rules: vec![] }]
                },
                source_overrides,
                report_syntactic: self.type_check,
                report_semantic: self.type_check,
            };
        }

        let mut config_groups: FxHashMap<BTreeSet<Rule>, Vec<String>> = FxHashMap::default();

        for path in paths {
            let path_buf = PathBuf::from(path);
            let file_path = path.to_string_lossy().to_string();

            let resolved_config = self.resolved_config(resolved_configs, &path_buf);

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

            config_groups.entry(rules).or_default().push(file_path);
        }

        Payload {
            version: 2,
            configs: config_groups
                .into_iter()
                .map(|(rules, file_paths)| Config {
                    file_paths,
                    rules: rules.into_iter().collect(),
                })
                .collect(),
            source_overrides,
            report_syntactic: self.type_check,
            report_semantic: self.type_check,
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
///       ]
///     }
///   ]
/// }
/// ```
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
    /// One source per file, shared by every diagnostic of that file that is sent as it arrives.
    named_sources: FxHashMap<PathBuf, Arc<NamedSource<String>>>,
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
            named_sources: FxHashMap::default(),
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
        let oxc_diagnostic = oxc_diagnostic.with_severity(if severity == AllowWarnDeny::Deny {
            Severity::Error
        } else {
            Severity::Warning
        });
        let source = self.named_source(path);
        // Best effort: the receiver is only gone when the run is already being torn down.
        let _ = self.error_sender.send(vec![oxc_diagnostic.with_source_code(source)]);
    }

    fn named_source(&mut self, path: &Path) -> Arc<NamedSource<String>> {
        if let Some(source) = self.named_sources.get(path) {
            return Arc::clone(source);
        }
        let cwd = self.cwd.clone();
        let source = DiagnosticService::named_source(cwd, path, self.get_source_text(path));
        self.named_sources.insert(path.to_path_buf(), Arc::clone(&source));
        source
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

/// Every name a disable directive may use for the type-aware rules of `resolved`.
///
/// A user writes whichever spelling their editor suggested, and [`should_skip_diagnostic`]
/// accepts all of them, so the set which must not be reported as unused has to accept all of them
/// too.
fn type_aware_directive_names(resolved: &ResolvedLinterState) -> FxHashSet<String> {
    resolved
        .rules
        .iter()
        .filter(|(rule, status)| status.is_warn_deny() && rule.is_tsgolint_rule())
        .flat_map(|(rule, _)| {
            let name = rule.name();
            std::iter::once(format!("{}/{name}", rule.plugin_name()))
                .chain(directive_spellings(name).map(Cow::into_owned))
        })
        .collect()
}

/// The spellings a disable directive may use for the `tsgolint` rule `name`: the bare name and the
/// two prefixes of `typescript-eslint`.
pub fn directive_spellings(name: &str) -> [Cow<'_, str>; 3] {
    [
        Cow::Borrowed(name),
        Cow::Owned(format!("typescript-eslint/{name}")),
        Cow::Owned(format!("@typescript-eslint/{name}")),
    ]
}

/// The severity of the `tsgolint` rule `rule` in `resolved`, `None` when it is not configured.
fn tsgolint_rule_severity(resolved: &ResolvedLinterState, rule: &str) -> Option<AllowWarnDeny> {
    resolved
        .rules
        .iter()
        .find(|(candidate, _)| candidate.is_tsgolint_rule() && candidate.name() == rule)
        .map(|(_, status)| *status)
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
        directive_spellings(&tsgolint_diagnostic.rule)
            .iter()
            .any(|spelling| directives.contains(spelling, span))
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

/// The environment variables which request Go profiling, with the flag of `tsgolint` each one
/// forwards its value to.
const PROFILING_ENV: [(&str, &str); 4] = [
    ("OXLINT_TSGOLINT_ALLOCS", "allocs"),
    ("OXLINT_TSGOLINT_CPU", "cpuprof"),
    ("OXLINT_TSGOLINT_HEAP", "heap"),
    ("OXLINT_TSGOLINT_TRACE", "trace"),
];

/// Whether Go profiling was requested with one of the [`PROFILING_ENV`] variables.
fn go_profiling_requested() -> bool {
    PROFILING_ENV.iter().any(|(env_name, _)| std::env::var(env_name).is_ok())
}

/// The file names the type-aware linter executable can have inside a directory such as
/// `node_modules/.bin`, in order of preference.
///
/// `tsrs-lint` is preferred over `tsgolint`, unless Go profiling is requested (see
/// [`go_profiling_requested`]): `tsrs-lint` does not support its flags, so only `tsgolint` is
/// considered then.
///
/// Executing a sub-command in Windows needs a `cmd` or `ps1` extension.
/// Since `cmd` is the most compatible one with older systems, we use that one first,
/// then check for `exe` which is also common. Bun, for example, does not create a `cmd`
/// file but still produces an `exe` file (<https://github.com/oxc-project/oxc/issues/13784>).
fn type_aware_executable_names() -> &'static [&'static str] {
    let go_profiling = go_profiling_requested();
    // The order of these lists is a preference, not an alphabetical order.
    #[cfg(windows)]
    {
        if go_profiling {
            &["tsgolint.CMD", "tsgolint.exe"]
        } else {
            &["tsrs-lint.CMD", "tsrs-lint.exe", "tsgolint.CMD", "tsgolint.exe"]
        }
    }
    #[cfg(not(windows))]
    {
        if go_profiling { &["tsgolint"] } else { &["tsrs-lint", "tsgolint"] }
    }
}

/// Lock `mutex`, ignoring poisoning.
///
/// The groups run concurrently and share this state; a group which panics while holding a lock
/// must not block the others. Nothing guarded here can be left half-written by a panic: every
/// guarded value is a plain map or set.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the walks of one plan concluded, keyed by the directory each answer is about: the
/// executable a walk from that directory reaches, or the absence of one.
type ExecutableMemo = FxHashMap<PathBuf, Option<ResolvedExecutable>>;

/// Resolve `start_dir` by walking up to `stop_at`, consulting `memo` at every level.
///
/// An answer recorded for any directory on the way answers the whole walk, so the siblings of an
/// already resolved directory stop as soon as they reach it. Only the directories this walk
/// looked at are recorded, each with the answer of the walk: every one of them would have
/// reached the same executable, or the same absence of one.
///
/// `look` is the file system check, taken as an argument so it can be counted in tests.
fn resolve_with_memo(
    start_dir: &Path,
    stop_at: Option<&Path>,
    memo: &mut ExecutableMemo,
    mut look: impl FnMut(&Path) -> WalkStep,
) -> Option<ResolvedExecutable> {
    let mut visited = Vec::new();

    let found = walk_for_tsgolint(start_dir, stop_at, |dir| {
        if let Some(known) = memo.get(dir) {
            return WalkStep::Stop(known.clone());
        }
        visited.push(dir.to_path_buf());
        look(dir)
    });

    for dir in visited {
        memo.insert(dir, found.clone());
    }
    found
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
    /// Files which `tsgolint` is not asked to lint, because the config which governs them does
    /// not set `options.typeAware` and the run did not force it, or because it cannot take their
    /// extension (`.vue`, `.svelte`, ...). Their type-aware rules did not run either, which is
    /// recorded like it is for the unresolved files.
    not_type_aware: Vec<Arc<OsStr>>,
    /// The config which applies to each file `tsgolint` lints, and to each of
    /// [`Self::not_type_aware`], resolved while planning so that the governing config of a file
    /// is looked up once. Empty for `--type-check-only`, which sends no rules.
    resolved_configs: FxHashMap<PathBuf, ResolvedLinterState>,
}

impl LintPlan {
    /// Whether there is nothing to run and nothing to report.
    fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.unresolved.is_empty() && self.not_type_aware.is_empty()
    }

    /// The files `tsgolint` did not lint, whose type-aware rules did not run: the unresolved
    /// ones, then the ones it was not asked to lint.
    fn skipped_paths(&self) -> impl Iterator<Item = &Arc<OsStr>> {
        self.unresolved.iter().chain(&self.not_type_aware)
    }
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

    /// An executable which is an installation of its own, without looking at the file system.
    #[cfg(test)]
    fn standalone(path: PathBuf) -> Self {
        Self { key: path.clone(), path }
    }
}

/// How many `tsgolint` processes run at once when `OXLINT_TSGOLINT_JOBS` does not say.
const DEFAULT_CONCURRENT_GROUPS: usize = 1;

/// How many groups to run at once, from the value of `OXLINT_TSGOLINT_JOBS`: a positive integer,
/// or the default for anything else.
fn requested_jobs(value: Option<&str>) -> usize {
    value
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|jobs| *jobs > 0)
        .unwrap_or(DEFAULT_CONCURRENT_GROUPS)
}

/// [`requested_jobs`] for the environment, read like the other `OXLINT_TSGOLINT_*` variables.
fn concurrent_groups() -> usize {
    requested_jobs(std::env::var("OXLINT_TSGOLINT_JOBS").ok().as_deref())
}

/// Run `lint` over every group, at most `cap` at a time, and return the results in group order.
///
/// `tsgolint` is itself parallel and can take several GB, so one process per installation would
/// oversubscribe a large monorepo; a few overlap their startup and I/O. Workers
/// take the next group as soon as they are free, so one slow package does not hold the others
/// back, and each result is tagged with its group before the work starts, so a group which
/// fails cannot shift the others' results onto the wrong groups.
fn run_groups<G: Sync, T: Send>(
    groups: &[G],
    cap: usize,
    lint: impl Fn(&G) -> Result<T, String> + Sync,
) -> Vec<Result<T, String>> {
    // One installation is the overwhelmingly common case; run it here rather than spawning a
    // thread to wait on.
    if let [group] = groups {
        return vec![lint(group)];
    }

    let workers = cap.max(1).min(groups.len());

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
/// answer repeated questions cheaply, which [`ExecutableMemo`] takes care of. Groups are sorted
/// by executable path, so that the order in which they are run and merged is deterministic.
/// Files `resolve` finds no executable for are collected separately instead of being dropped.
fn group_paths_by_executable(
    paths: &[Arc<OsStr>],
    mut keep: impl FnMut(&Path) -> bool,
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

    LintPlan {
        groups,
        unresolved,
        not_type_aware: Vec::new(),
        resolved_configs: FxHashMap::default(),
    }
}

/// How to get a `tsgolint`, appended to every message about a missing executable.
const MISSING_EXECUTABLE_HELP: &str = "Add the `tsrs` or `oxlint-tsgolint` package to the corresponding project, or set the `OXLINT_TSGOLINT_PATH` environment variable to a type-aware linter executable.";

/// What to do instead when Go profiling was requested, which only `tsgolint` supports.
const GO_PROFILING_HELP: &str = "Go profiling requires the `oxlint-tsgolint` package: add it to the corresponding project, or set the `OXLINT_TSGOLINT_PATH` environment variable to a `tsgolint` executable.";

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
    let help = if go_profiling_requested() { GO_PROFILING_HELP } else { MISSING_EXECUTABLE_HELP };
    format!(
        "Could not find a `tsgolint` executable for any of the {unresolved} file(s) to lint with type-aware rules.\n{help}"
    )
}

/// A warning for the editor about a file which could not be linted with type-aware rules, with the
/// help on how to install `tsgolint`.
fn editor_warning(text: String) -> Message {
    attached(OxcDiagnostic::warn(text).with_help(MISSING_EXECUTABLE_HELP))
}

/// `diagnostic` as a message without a fix, labelled at the start of the file so the editor has a
/// position to show it at.
fn attached(diagnostic: OxcDiagnostic) -> Message {
    Message::new(diagnostic.with_label(Span::default()), PossibleFixes::None)
}

/// The warning the editor gets when type-aware linting could not run for a file at all.
///
/// The regular rules did run, so their diagnostics are kept; the message reports why the
/// type-aware ones are missing.
pub fn type_aware_failure_message(error: &str) -> Message {
    editor_warning(format!("This file was not linted with type-aware rules: {error}"))
}

/// The warning the editor gets for a group whose `tsgolint` could not be run.
///
/// It names the first file of the group, so the message points at a file the user can see, and
/// the executable, so the installation to fix is identified.
fn group_failure_message(group: &LintGroup, error: &str) -> Message {
    let file =
        group.paths.first().map_or_default(|path| Path::new(path.as_ref()).display().to_string());

    editor_warning(format!(
        "`{}` could not lint `{file}`, so it was not checked with type-aware rules: {error}",
        group.executable.display()
    ))
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
            editor_warning(
                "Could not find a `tsgolint` executable for this file, it was not linted with type-aware rules."
                    .to_string(),
            )
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
///
/// Only a `tsgolint` shim is identified by the `oxlint-tsgolint` package: a `tsrs-lint` shim
/// installed next to it belongs to another installation, and must not share its group.
fn executable_group_key(executable: &Path) -> PathBuf {
    // `<dir>/node_modules/.bin/tsgolint` -> `<dir>/node_modules/oxlint-tsgolint`
    if executable.file_stem().is_some_and(|stem| stem == "tsgolint")
        && let Some(node_modules) = executable.parent().and_then(Path::parent)
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
        let expected =
            if go_profiling_requested() { "'tsgolint'" } else { "'tsrs-lint' or 'tsgolint'" };
        return Err(format!(
            "Failed to find type-aware linter executable: OXLINT_TSGOLINT_PATH points to directory '{displayed}' but no {expected} binary found inside"
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
    /// This directory answers the walk, with an executable or with the absence of one, as a
    /// directory the plan already walked through does.
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

/// Look for any of [`type_aware_executable_names`] directly inside `dir`.
fn tsgolint_in_dir(dir: &Path) -> Option<PathBuf> {
    type_aware_executable_names().iter().map(|file| dir.join(file)).find(|path| path.is_file())
}

/// Search the system `PATH` for the `tsgolint` executable.
///
/// This supports package managers that install binaries globally and make them available
/// via `PATH`.
fn find_tsgolint_in_path() -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    std::env::split_paths(&path_env)
        .flat_map(|dir| type_aware_executable_names().iter().map(move |file| dir.join(file)))
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
    #[cfg(unix)]
    use std::io::{BufRead as _, BufReader, Read as _};
    use std::{
        ffi::OsStr,
        path::{Path, PathBuf},
        sync::{
            Arc, Barrier,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use oxc_diagnostics::{LabeledSpan, OxcCode, Severity};
    use oxc_span::Span;

    use crate::{
        fixer::{FixKind, Message, PossibleFixes},
        tsgolint::{Fix, LabeledRange, Range, RuleMessage, Suggestion, TsGoLintRuleDiagnostic},
    };

    use super::{
        ExecutableMemo, Fallback, LintGroup, LintPlan, MAX_MESSAGE_SIZE, ResolvedExecutable,
        TSGOLINT_DIAGNOSTIC_SCOPE, TsGoLintMessageParseError, executable_group_key,
        find_tsgolint_from_cwd, group_failure_message, group_paths_by_executable,
        join_group_errors, look_in_node_modules, missing_executable_diagnostic,
        missing_executable_messages, no_executable_error, parse_single_message, requested_jobs,
        resolve_tsgolint_executable_with, resolve_with_memo, run_groups,
        type_aware_executable_names, walk_boundary_for,
    };

    /// Resolve `start_dir` the way production does, boundary included, through a memo of its
    /// own.
    fn resolve_for_dir(start_dir: &Path, cwd: &Path) -> Option<PathBuf> {
        resolve_with_memo(
            start_dir,
            walk_boundary_for(start_dir, cwd),
            &mut ExecutableMemo::default(),
            look_in_node_modules,
        )
        .map(|resolved| resolved.path)
    }

    /// Create a fake `tsgolint` executable in `<dir>/node_modules/.bin`, using the same file
    /// name production code looks for, and return its path.
    fn create_fake_tsgolint(dir: &Path) -> PathBuf {
        let bin_dir = dir.join("node_modules").join(".bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let name = type_aware_executable_names()
            .iter()
            .find(|name| name.starts_with("tsgolint"))
            .expect("a `tsgolint` name is always listed");
        let executable = bin_dir.join(name);
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
    fn an_explicit_path_without_a_tsgolint_is_an_error() {
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
    }

    #[test]
    fn without_an_explicit_path_finding_nothing_is_not_an_error() {
        let root = tempfile::tempdir().unwrap();

        assert!(resolve_tsgolint_executable_with(root.path(), None).is_ok());
    }

    #[test]
    fn an_explicit_directory_is_searched_for_every_name_of_the_executable() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // A directory holding the binary under its bare name, which is how the release is
        // shipped on every platform, including the one whose shims carry an extension.
        let release_dir = root_path.join("release");
        std::fs::create_dir_all(&release_dir).unwrap();
        let bare = release_dir.join("tsgolint");
        std::fs::write(&bare, "").unwrap();
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(release_dir.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(bare)),
            "an explicit directory should be searched for the bare name too"
        );

        // A directory holding it under one of the names a package manager writes.
        let executable = create_fake_tsgolint(root_path);
        let bin_dir = executable.parent().unwrap();
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(bin_dir.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(executable)),
            "the directory should be searched for every name of {:?}",
            type_aware_executable_names()
        );
    }

    #[test]
    fn an_explicit_path_can_name_the_executable_directly() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let bare = root_path.join("tsgolint");
        std::fs::write(&bare, "").unwrap();
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(bare.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(bare))
        );

        let executable = create_fake_tsgolint(root_path);
        assert_eq!(
            resolve_tsgolint_executable_with(root_path, Some(executable.as_os_str())).unwrap(),
            Fallback::Env(ResolvedExecutable::new(executable))
        );
    }

    /// The editor shows the diagnostics of a file on their own, so every unresolved file gets its
    /// own warning, which says what to do.
    #[test]
    fn missing_executable_gives_one_warning_per_unresolved_file() {
        let unresolved =
            [arc_path(Path::new("/packages/b/x.ts")), arc_path(Path::new("/packages/b/y.ts"))];

        let messages = missing_executable_messages(&unresolved);

        assert_eq!(messages.len(), unresolved.len());
        for message in &messages {
            assert_eq!(message.error.severity, Severity::Warning);
            let help = message.error.help.as_ref().expect("the warning should say what to do");
            assert!(help.contains("oxlint-tsgolint"), "{help}");
            assert!(help.contains("OXLINT_TSGOLINT_PATH"), "{help}");
        }
    }

    /// A file outside the working directory has no boundary for its walk, which climbs above the
    /// working directory to the package that owns the file. A file below the working directory
    /// still stops there, so its walk does not reach the file system root.
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
        let found = resolve_with_memo(
            &sibling_src,
            walk_boundary_for(&sibling_src, &cwd),
            &mut ExecutableMemo::default(),
            |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            },
        );

        assert_eq!(found.map(|resolved| resolved.path), Some(executable));
        assert_eq!(looked_at, vec![sibling_src, sibling], "the walk should climb out of the cwd");

        // A file below the working directory still stops there.
        let inside = cwd.join("src");
        std::fs::create_dir_all(&inside).unwrap();
        assert_eq!(walk_boundary_for(&inside, &cwd), Some(cwd.as_path()));
        assert_eq!(resolve_for_dir(&inside, &cwd), None);
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

        let results = run_groups(&groups, 2, |group| {
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

    /// `tsgolint` saturates the machine on its own, so only as many run at once as asked for.
    /// Each group waits for `cap - 1` others to be running too, so a run which is not
    /// concurrent up to the cap never finishes.
    #[test]
    fn concurrency_follows_the_requested_cap() {
        let groups = (0..16)
            .map(|index| LintGroup {
                executable: PathBuf::from(format!("/bin/tsgolint-{index}")),
                paths: vec![],
            })
            .collect::<Vec<_>>();

        let running = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);

        for cap in [1, 2, 4] {
            running.store(0, Ordering::SeqCst);
            peak.store(0, Ordering::SeqCst);
            let together = Barrier::new(cap);

            let results = run_groups(&groups, cap, |group| {
                let now = running.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                together.wait();
                running.fetch_sub(1, Ordering::SeqCst);
                Ok(group.executable.clone())
            });

            let peak = peak.load(Ordering::SeqCst);
            assert_eq!(peak, cap, "{cap} groups should run at once, saw {peak}");

            // Every group ran, and the results came back in group order.
            let executables = results.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(
                executables,
                groups.iter().map(|group| group.executable.clone()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn requested_jobs_reads_a_positive_number() {
        assert_eq!(requested_jobs(Some("3")), 3);
        assert_eq!(requested_jobs(Some(" 2 ")), 2);
        for value in [None, Some(""), Some("0"), Some("x"), Some("-1")] {
            assert_eq!(requested_jobs(value), 1, "{value:?}");
        }
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
        let LintPlan { groups, unresolved, .. } = group_paths_by_executable(
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
                Some(ResolvedExecutable::standalone(executable))
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

    /// `plan` keeps the files in the order they were given, leaves out the ones whose config
    /// does not enable type-aware linting, and resolves the config of each kept file once, for
    /// `json_input` to group them by rule set.
    #[test]
    fn test_plan_keeps_the_type_aware_files_in_input_order() {
        let cwd = Path::new("/oxc-tsgolint-plan-test");
        let config_store = crate::ConfigStore::from_oxlintrc_json(
            serde_json::json!({
                "options": { "typeAware": true },
                "rules": { "typescript/no-floating-promises": "error" }
            }),
            [
                (
                    cwd.join("b"),
                    serde_json::json!({
                        "options": { "typeAware": true },
                        "categories": { "correctness": "off" }
                    }),
                ),
                (cwd.join("skip"), serde_json::json!({ "options": { "typeAware": false } })),
            ],
        );

        // No executable anywhere, so every kept file ends up in `unresolved`, in planning order.
        let state =
            super::TsGoLintState::for_test(cwd, config_store, None).with_type_aware_forced(false);
        let paths: Vec<Arc<OsStr>> = ["a.ts", "b/x.ts", "c.ts", "skip/y.ts", "b/z.ts", "d.vue"]
            .iter()
            .map(|path| arc_path(&cwd.join(path)))
            .collect();
        let plan = state.plan(&paths);

        let kept = ["a.ts", "b/x.ts", "c.ts", "b/z.ts"].map(|path| arc_path(&cwd.join(path)));
        assert_eq!(plan.unresolved, kept);
        // The file of the config which is not type-aware is resolved too, to record its rules.
        // So is a file `tsgolint` cannot take, which no config is resolved for while planning.
        assert_eq!(
            plan.not_type_aware,
            ["skip/y.ts", "d.vue"].map(|path| arc_path(&cwd.join(path)))
        );
        assert_eq!(plan.resolved_configs.len(), kept.len() + 1);

        let payload = state.json_input(&plan.unresolved, None, &plan.resolved_configs);
        assert_eq!(payload.configs.len(), 2);
    }

    /// `--type-check-only` type-checks every file: the config which governs a file, even one
    /// with `typeAware: false`, does not take it out of the run, and no config is resolved.
    #[test]
    fn test_type_check_only_plan_keeps_every_file() {
        let cwd = Path::new("/oxc-tsgolint-type-check-only-test");
        let config_store = crate::ConfigStore::from_oxlintrc_json(
            serde_json::json!({ "options": { "typeAware": true } }),
            [(cwd.join("skip"), serde_json::json!({ "options": { "typeAware": false } }))],
        );

        let state = super::TsGoLintState::for_test(cwd, config_store, None)
            .with_type_aware_forced(false)
            .with_type_check_only(true);
        let paths: Vec<Arc<OsStr>> =
            ["a.ts", "skip/y.ts"].iter().map(|path| arc_path(&cwd.join(path))).collect();
        let plan = state.plan(&paths);

        assert_eq!(plan.unresolved, paths);
        assert!(plan.resolved_configs.is_empty());
    }

    /// The files of a group keep the order they were given in, even interleaved with the files
    /// of another group: `lint_group` hands them to `tsgolint`, and reports, in that order.
    #[test]
    fn test_group_paths_by_executable_keeps_input_order() {
        let a1 = PathBuf::from("/packages/a/src/1.ts");
        let b1 = PathBuf::from("/packages/b/src/1.ts");
        let a2 = PathBuf::from("/packages/a/src/2.ts");
        let b2 = PathBuf::from("/packages/b/src/2.ts");
        let a3 = PathBuf::from("/packages/a/src/0.ts");

        let LintPlan { groups, .. } = group_paths_by_executable(
            &[arc_path(&a1), arc_path(&b1), arc_path(&a2), arc_path(&b2), arc_path(&a3)],
            |_| true,
            |dir| {
                let package = dir.parent()?;
                let executable = package.join("node_modules/.bin/tsgolint");
                Some(ResolvedExecutable::standalone(executable))
            },
        );

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].paths, vec![arc_path(&a1), arc_path(&a2), arc_path(&a3)]);
        assert_eq!(groups[1].paths, vec![arc_path(&b1), arc_path(&b2)]);
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
        assert_eq!(forwards.unresolved, []);
    }

    #[test]
    fn test_memo_records_every_visited_directory() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let package = root_path.join("packages").join("a");
        let deep = package.join("src").join("nested");
        std::fs::create_dir_all(&deep).unwrap();
        let executable = create_fake_tsgolint(&package);

        let mut memo = ExecutableMemo::default();
        let mut looked_at = Vec::new();
        let found = resolve_with_memo(&deep, Some(root_path), &mut memo, |dir| {
            looked_at.push(dir.to_path_buf());
            look_in_node_modules(dir)
        });

        assert_eq!(found.as_ref().map(|resolved| &resolved.path), Some(&executable));
        // The walk stopped as soon as it found one.
        assert_eq!(looked_at, vec![deep, package.join("src"), package.clone()]);

        // Every directory it climbed now answers on its own, and nothing else was recorded.
        assert_eq!(memo.len(), looked_at.len());
        for dir in &looked_at {
            let recorded = memo.get(dir).unwrap_or_else(|| {
                panic!("{} should be recorded", dir.display());
            });
            assert_eq!(recorded.as_ref().map(|resolved| &resolved.path), Some(&executable));
        }
    }

    /// The absence of an executable is recorded too, so a package which has none is not walked
    /// again for every one of its files.
    #[test]
    fn test_memo_records_a_miss() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let src = root_path.join("packages").join("b").join("src");
        std::fs::create_dir_all(&src).unwrap();

        let mut memo = ExecutableMemo::default();
        assert_eq!(resolve_with_memo(&src, Some(root_path), &mut memo, look_in_node_modules), None);

        assert_eq!(memo.get(&src), Some(&None));
        assert_eq!(memo.get(root_path), Some(&None));
    }

    /// The files of one plan share a memo: a walk which reaches a directory an earlier walk
    /// recorded stops there, instead of asking the file system all the way up again, and a
    /// directory which was resolved is not looked at again.
    #[test]
    fn a_plan_does_not_stat_a_directory_twice() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let package = root_path.join("packages").join("a");
        let src = package.join("src");
        let first = src.join("first");
        let second = src.join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        let executable = create_fake_tsgolint(&package);

        let mut memo = ExecutableMemo::default();
        let mut looked_at = Vec::new();
        let mut resolve = |dir: &Path| {
            resolve_with_memo(dir, Some(root_path), &mut memo, |dir| {
                looked_at.push(dir.to_path_buf());
                look_in_node_modules(dir)
            })
        };

        assert!(resolve(&first).is_some());
        // A sibling only looks at itself before finding `src` in the memo.
        assert_eq!(resolve(&second).map(|resolved| resolved.path), Some(executable));
        // The same holds for a directory already resolved: no file system access at all.
        assert!(resolve(&first).is_some());

        let mut expected = vec![first.clone(), src, package, second];
        let mut seen = looked_at.clone();
        expected.sort();
        seen.sort();
        assert_eq!(seen, expected, "every directory should be looked at exactly once");
        assert_eq!(looked_at.len(), 4);
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
    #[test]
    fn test_executable_group_key_keeps_tsrs_lint_apart_from_tsgolint() {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        let tsgolint = create_fake_tsgolint(root_path);
        let package_dir = root_path.join("node_modules").join("oxlint-tsgolint");
        std::fs::create_dir_all(&package_dir).unwrap();
        let tsrs_lint = tsgolint.parent().unwrap().join(if cfg!(windows) {
            "tsrs-lint.CMD"
        } else {
            "tsrs-lint"
        });
        std::fs::write(&tsrs_lint, "").unwrap();

        assert_eq!(executable_group_key(&tsgolint), std::fs::canonicalize(&package_dir).unwrap());
        assert_eq!(executable_group_key(&tsrs_lint), std::fs::canonicalize(&tsrs_lint).unwrap());
    }

    /// Write an executable shell script, standing in for `tsgolint`.
    #[cfg(unix)]
    fn write_script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// A config store whose default rules include the type-aware ones.
    #[cfg(unix)]
    fn type_aware_config_store() -> crate::ConfigStore {
        let mut plugins = crate::ExternalPluginStore::default();
        let config = crate::ConfigStoreBuilder::default().build(&mut plugins).unwrap();
        crate::ConfigStore::new(config, rustc_hash::FxHashMap::default(), plugins)
    }

    /// `count` packages of one `.ts` file each, every one with a `tsgolint` of its own running
    /// `script(package)`, and the files to lint.
    #[cfg(unix)]
    fn packages_with_scripts(
        temp: &Path,
        count: usize,
        script: impl Fn(&Path) -> String,
    ) -> (Vec<PathBuf>, Vec<Arc<OsStr>>) {
        let mut packages = vec![];
        let mut files = vec![];
        for index in 0..count {
            let package = temp.join(format!("pkg-{index}"));
            write_script(&package.join("node_modules/.bin/tsgolint"), &script(&package));
            let file = package.join("index.ts");
            std::fs::write(&file, "export {};\n").unwrap();
            files.push(arc_path(&file));
            packages.push(package);
        }
        (packages, files)
    }

    /// A named pipe in `dir`, which the fake `tsgolint` scripts write to: a test learns what they
    /// did by reading it, instead of polling for it.
    #[cfg(unix)]
    fn make_fifo(dir: &Path) -> PathBuf {
        let fifo = dir.join("announcements");
        let status = std::process::Command::new("mkfifo").arg(&fifo).status().unwrap();
        assert!(status.success(), "mkfifo failed: {status}");
        fifo
    }

    /// Every group runs while the caller is still busy, one after the other, not when it asks
    /// for the result.
    #[cfg(unix)]
    #[test]
    fn all_groups_run_before_finish_is_called() {
        let temp = tempfile::tempdir().unwrap();
        let fifo = make_fifo(temp.path());
        // Opened for reading and writing, so that the pipe never ends between two groups.
        let announcements = std::fs::OpenOptions::new().read(true).write(true).open(&fifo).unwrap();
        let (packages, files) = packages_with_scripts(temp.path(), 6, |_| {
            format!(
                "while IFS= read -r _l; do :; done\necho started > \"{}\"\nexit 0\n",
                fifo.display()
            )
        });

        let state = super::TsGoLintState::for_test(temp.path(), type_aware_config_store(), None);
        let plan = state.try_plan(&files).unwrap();
        assert_eq!(plan.groups.len(), packages.len());
        let run = state.start(plan);

        // Every group announces itself once it ran, and `finish` was not called yet.
        let mut announcements = BufReader::new(announcements);
        for _ in &packages {
            let mut line = String::new();
            announcements.read_line(&mut line).unwrap();
            assert_eq!(line, "started\n");
        }

        let (tx, _rx) = std::sync::mpsc::channel();
        let diff_manager =
            Arc::new(crate::suppression::DiffManager::new(Arc::default(), false, true, false));
        run.finish(&crate::DirectivesStore::new(), tx, &crate::OsFileSystem, &diff_manager, None)
            .unwrap();
    }

    /// Dropping the run, as happens when the regular pass unwinds, must not leave `tsgolint`
    /// processes behind, and a group which had not started must never start.
    #[cfg(unix)]
    #[test]
    fn dropping_a_started_run_stops_every_group() {
        let temp = tempfile::tempdir().unwrap();
        let fifo = make_fifo(temp.path());
        // Each process keeps the pipe open until it dies, so the end of the pipe is the death of
        // every process which announced itself.
        let (packages, files) = packages_with_scripts(temp.path(), 2, |_| {
            format!("exec 3> \"{}\"\necho started >&3\nexec sleep 600\n", fifo.display())
        });

        let state = super::TsGoLintState::for_test(temp.path(), type_aware_config_store(), None);
        let plan = state.try_plan(&files).unwrap();
        let run = state.start(plan);

        // Groups may be capped below the number of packages: the others have not started.
        let running = super::concurrent_groups().min(packages.len());
        let mut announcements = BufReader::new(std::fs::File::open(&fifo).unwrap());
        for _ in 0..running {
            let mut line = String::new();
            announcements.read_line(&mut line).unwrap();
            assert_eq!(line, "started\n");
        }

        drop(run);

        let mut announced_later = String::new();
        announcements.read_to_string(&mut announced_later).unwrap();
        assert_eq!(announced_later, "", "a group which had not started must never start");
    }
}
