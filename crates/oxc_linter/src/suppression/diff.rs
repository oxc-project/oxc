use std::path::Path;

use oxc_diagnostics::Severity;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    Message, oxc_code_short_canonical_name,
    suppression::{
        DiagnosticCounts, Filename, RuntimeSuppressionMap, StaticSuppressionMap, SuppressionFile,
        SuppressionFileState,
    },
};

pub struct DiffManager {
    tracking_map: StaticSuppressionMap,
    runtime_map: RuntimeSuppressionMap,
    suppress_all: bool,
    file_exists: bool,
    ignore_diff: bool,
}

impl DiffManager {
    pub fn new(
        tracking_map: StaticSuppressionMap,
        file_exists: bool,
        ignore_diff: bool,
        suppress_all: bool,
    ) -> Self {
        Self {
            tracking_map,
            runtime_map: RuntimeSuppressionMap::default(),
            suppress_all,
            file_exists,
            ignore_diff,
        }
    }

    /// Process messages for a file: filter suppressed diagnostics and accumulate runtime counts.
    /// Returns the filtered messages (only new/increased violations shown to the user).
    pub fn collect_file(
        &self,
        file_path: &Path,
        cwd: &Path,
        messages: Vec<Message>,
    ) -> Vec<Message> {
        if self.ignore_diff {
            return messages;
        }

        let Ok(file_path) = file_path.strip_prefix(cwd) else {
            return messages;
        };

        let filename = Filename::new(file_path);
        let suppression_data = self.tracking_map.get(&filename);
        let suppression_file =
            SuppressionFile::new(self.file_exists, self.suppress_all, suppression_data);

        let (filtered_diagnostics, runtime_counts) =
            Self::suppress_lint_diagnostics(&suppression_file, messages);

        if let Some(counts) = runtime_counts {
            self.runtime_map.merge_file(filename, counts);
        }

        filtered_diagnostics
    }

    /// Mark that a file was seen but produced no violations (e.g. all fixed).
    /// This ensures we track it as "empty" rather than "unseen".
    pub fn collect_empty_file(&self, file_path: &Path, cwd: &Path) {
        if self.ignore_diff {
            return;
        }

        let Ok(file_path) = file_path.strip_prefix(cwd) else {
            return;
        };

        let filename = Filename::new(file_path);
        self.runtime_map.mark_seen(filename);
    }

    /// Record that the type-aware pass could not run for `file_path`, because no `tsgolint`
    /// executable could be found for it.
    ///
    /// The regular pass has already marked the file as seen, so the suppressions recorded for
    /// the rules `tsgolint` would have run would otherwise look like suppressions which stopped
    /// firing, and be pruned or reported as unpruned. Carrying their recorded counts over
    /// leaves them exactly as they are: nothing is reported for them, and nothing is rewritten.
    ///
    /// `type_aware_rules` holds the suppression keys of those rules, and only those: oxlint runs
    /// rules of its own under the same plugin, whose suppressions are genuinely stale when they
    /// stop firing.
    pub fn collect_type_aware_skipped_file(
        &self,
        file_path: &Path,
        cwd: &Path,
        type_aware_rules: impl FnOnce() -> FxHashSet<String>,
    ) {
        if self.ignore_diff {
            return;
        }

        let Ok(file_path) = file_path.strip_prefix(cwd) else {
            return;
        };

        let filename = Filename::new(file_path);
        let Some(recorded) = self.tracking_map.get(&filename) else {
            // Nothing was recorded for this file, so nothing can go stale.
            return;
        };

        // The config which governs this file is only resolved at this point.
        let type_aware_rules = type_aware_rules();
        let carried: FxHashMap<String, DiagnosticCounts> = recorded
            .iter()
            .filter(|(rule, _)| type_aware_rules.contains(rule.as_str()))
            .map(|(rule, counts)| (rule.clone(), counts.clone()))
            .collect();

        if !carried.is_empty() {
            self.runtime_map.merge_file(filename, carried);
        }
    }

    pub fn skip(&self) -> bool {
        self.ignore_diff
    }

    /// Return the accumulated runtime map for final diff computation.
    pub fn into_runtime_map(self) -> RuntimeSuppressionMap {
        self.runtime_map
    }

    fn suppress_lint_diagnostics(
        suppression_file_state: &SuppressionFile<'_>,
        lint_diagnostics: Vec<Message>,
    ) -> (Vec<Message>, Option<FxHashMap<String, DiagnosticCounts>>) {
        let build_suppression_map = |diagnostics: &Vec<Message>| {
            let mut suppression_tracking: FxHashMap<String, DiagnosticCounts> =
                FxHashMap::default();
            for message in diagnostics {
                // Only consider error severity messages for suppression tracking
                if message.error.severity != Severity::Error {
                    continue;
                }

                let Some(key) = oxc_code_short_canonical_name(&message.error.code) else {
                    continue;
                };

                suppression_tracking.entry(key).or_insert(DiagnosticCounts { count: 0 }).count += 1;
            }

            suppression_tracking
        };

        match suppression_file_state.suppression_state() {
            SuppressionFileState::Ignored => (lint_diagnostics, None),
            SuppressionFileState::New => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                // Filter out error-severity diagnostics — they are being written
                // to the new suppressions file. Only warnings pass through.
                let filtered = lint_diagnostics
                    .into_iter()
                    .filter(|message| message.error.severity != Severity::Error)
                    .collect();

                (filtered, Some(runtime_suppression_tracking))
            }
            SuppressionFileState::Exists => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                let Some(recorded_violations) = suppression_file_state.suppression_data() else {
                    return (lint_diagnostics, Some(runtime_suppression_tracking));
                };

                let diagnostics_filtered = lint_diagnostics
                    .into_iter()
                    .filter(|message| {
                        // Warnings are not suppressed — always pass through
                        if message.error.severity != Severity::Error {
                            return true;
                        }

                        let Some(key) = oxc_code_short_canonical_name(&message.error.code) else {
                            return true;
                        };

                        let Some(count_file) = recorded_violations.get(&key) else {
                            return true;
                        };

                        let Some(count_runtime) = runtime_suppression_tracking.get(&key) else {
                            return false;
                        };

                        count_file.count < count_runtime.count
                    })
                    .collect();

                (diagnostics_filtered, Some(runtime_suppression_tracking))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        path::{Path, PathBuf},
        sync::Arc,
    };

    use oxc_diagnostics::OxcDiagnostic;
    use rustc_hash::{FxHashMap, FxHashSet};

    use crate::{
        Message, PossibleFixes,
        suppression::{DiagnosticCounts, DiffManager, Filename, SuppressionManager},
    };

    /// A rule `tsgolint` runs. Note that it lives under the same plugin as the rules oxlint
    /// runs itself, which is why the carry-over cannot key off the scope.
    const TYPE_AWARE_RULE: &str = "typescript/no-floating-promises";
    /// A rule oxlint runs itself, under that same plugin.
    const NATIVE_TYPESCRIPT_RULE: &str = "typescript/no-explicit-any";
    const NATIVE_ESLINT_RULE: &str = "no-debugger";
    const FILE: &str = "packages/b/src/index.ts";

    type StaticMap = Arc<FxHashMap<Filename, FxHashMap<String, DiagnosticCounts>>>;

    fn suppressions(rules: &[&str]) -> StaticMap {
        let recorded =
            rules.iter().map(|rule| ((*rule).to_string(), DiagnosticCounts { count: 1 })).collect();

        let mut map = FxHashMap::default();
        map.insert(Filename::new(Path::new(FILE)), recorded);
        Arc::new(map)
    }

    /// The keys `tsgolint` would have produced for this file, as
    /// `TsGoLintState::type_aware_suppression_keys` computes them from the resolved config.
    fn type_aware_rules() -> FxHashSet<String> {
        std::iter::once(TYPE_AWARE_RULE.to_string()).collect()
    }

    fn diagnostic(scope: &'static str, rule: &'static str) -> Message {
        Message::new(
            OxcDiagnostic::error("something").with_error_code(scope, rule),
            PossibleFixes::None,
        )
    }

    /// A file whose package has no `tsgolint` is still linted by the regular pass, so it counts
    /// as seen. The suppressions of the rules `tsgolint` would have run must not look like
    /// suppressions which stopped firing.
    #[test]
    fn test_type_aware_skipped_file_keeps_its_type_aware_suppressions() {
        let cwd = PathBuf::from("/workspace");
        let file = cwd.join(FILE);
        let static_map = suppressions(&[TYPE_AWARE_RULE, NATIVE_ESLINT_RULE]);

        let diff_manager = DiffManager::new(Arc::clone(&static_map), true, false, false);
        // The regular pass ran and found the `no-debugger` violation.
        diff_manager.collect_file(&file, &cwd, vec![diagnostic("eslint", "no-debugger")]);
        // No `tsgolint` could be found for this file.
        diff_manager.collect_type_aware_skipped_file(&file, &cwd, type_aware_rules);

        let runtime_map = diff_manager.into_runtime_map().into_inner();
        let counts = &runtime_map[&Filename::new(Path::new(FILE))];
        assert_eq!(counts[NATIVE_ESLINT_RULE].count, 1);
        assert_eq!(
            counts[TYPE_AWARE_RULE].count, 1,
            "the recorded count of a rule tsgolint would have run should be carried over"
        );

        // Nothing is reported as stale.
        let (errors, has_unused) =
            SuppressionManager::compute_diagnostics(&static_map, &runtime_map);
        assert!(!has_unused, "{errors:?}");
        assert!(errors.is_empty(), "{errors:?}");

        // Nothing is pruned away.
        let pruned = SuppressionManager::compute_prune(&static_map, &runtime_map, &cwd);
        let pruned_rules = &pruned[&Filename::new(Path::new(FILE))];
        assert_eq!(pruned_rules[TYPE_AWARE_RULE].count, 1);
        assert_eq!(pruned_rules[NATIVE_ESLINT_RULE].count, 1);
    }

    /// The rules oxlint runs itself did run, even under the `typescript` plugin, so their
    /// suppressions are stale as usual when they stop firing.
    #[test]
    fn test_type_aware_skipped_file_still_reports_its_native_suppressions() {
        let cwd = PathBuf::from("/workspace");
        let file = cwd.join(FILE);
        let static_map = suppressions(&[TYPE_AWARE_RULE, NATIVE_TYPESCRIPT_RULE]);

        let diff_manager = DiffManager::new(Arc::clone(&static_map), true, false, false);
        // `typescript/no-explicit-any` ran and no longer fires.
        diff_manager.collect_file(&file, &cwd, vec![diagnostic("eslint", "no-debugger")]);
        diff_manager.collect_type_aware_skipped_file(&file, &cwd, type_aware_rules);

        let runtime_map = diff_manager.into_runtime_map().into_inner();
        let counts = &runtime_map[&Filename::new(Path::new(FILE))];
        assert_eq!(counts[TYPE_AWARE_RULE].count, 1, "the type-aware rule is carried over");
        assert!(
            !counts.contains_key(NATIVE_TYPESCRIPT_RULE),
            "a rule oxlint runs itself must not be carried over"
        );

        let (_, has_unused) = SuppressionManager::compute_diagnostics(&static_map, &runtime_map);
        assert!(has_unused, "the stale native suppression should still be reported");

        let pruned = SuppressionManager::compute_prune(&static_map, &runtime_map, &cwd);
        let pruned_rules = &pruned[&Filename::new(Path::new(FILE))];
        assert_eq!(pruned_rules[TYPE_AWARE_RULE].count, 1);
        assert!(
            !pruned_rules.contains_key(NATIVE_TYPESCRIPT_RULE),
            "the stale native suppression should still be pruned"
        );
    }

    /// The tracker only counts error-severity diagnostics, so a type-aware rule configured at
    /// `warn` never had a suppression of its own to preserve. One recorded against it is stale
    /// like any other.
    #[test]
    fn test_type_aware_rules_below_deny_are_not_carried_over() {
        let cwd = PathBuf::from("/workspace");
        let file = cwd.join(FILE);
        let static_map = suppressions(&[TYPE_AWARE_RULE, NATIVE_ESLINT_RULE]);

        let diff_manager = DiffManager::new(Arc::clone(&static_map), true, false, false);
        diff_manager.collect_file(&file, &cwd, vec![diagnostic("eslint", "no-debugger")]);
        // `TsGoLintState::type_aware_suppression_keys` keeps only the `Deny` rules, so a rule
        // configured at `warn` is simply absent from the set.
        diff_manager.collect_type_aware_skipped_file(&file, &cwd, FxHashSet::default);

        let runtime_map = diff_manager.into_runtime_map().into_inner();
        assert!(!runtime_map[&Filename::new(Path::new(FILE))].contains_key(TYPE_AWARE_RULE));

        let (_, has_unused) = SuppressionManager::compute_diagnostics(&static_map, &runtime_map);
        assert!(has_unused, "a suppression which cannot be produced any more is stale");

        let pruned = SuppressionManager::compute_prune(&static_map, &runtime_map, &cwd);
        assert!(!pruned[&Filename::new(Path::new(FILE))].contains_key(TYPE_AWARE_RULE));
    }

    /// Without the carry-over the type-aware suppression is reported as unpruned and rewritten
    /// away, which is exactly what the first test prevents.
    #[test]
    fn test_type_aware_suppressions_go_stale_when_the_file_is_not_marked_as_skipped() {
        let cwd = PathBuf::from("/workspace");
        let file = cwd.join(FILE);
        let static_map = suppressions(&[TYPE_AWARE_RULE, NATIVE_ESLINT_RULE]);

        let diff_manager = DiffManager::new(Arc::clone(&static_map), true, false, false);
        diff_manager.collect_file(&file, &cwd, vec![diagnostic("eslint", "no-debugger")]);

        let runtime_map = diff_manager.into_runtime_map().into_inner();

        let (_, has_unused) = SuppressionManager::compute_diagnostics(&static_map, &runtime_map);
        assert!(has_unused);

        let pruned = SuppressionManager::compute_prune(&static_map, &runtime_map, &cwd);
        let pruned_rules = &pruned[&Filename::new(Path::new(FILE))];
        assert!(!pruned_rules.contains_key(TYPE_AWARE_RULE));
    }
}
