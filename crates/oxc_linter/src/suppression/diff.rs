use std::path::Path;

use oxc_diagnostics::Severity;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    Message, oxc_code_short_canonical_name,
    suppression::{
        DiagnosticCounts, Filename, RuntimeSuppressionMap, StaticSuppressionMap, SuppressionFile,
        SuppressionFileState, SuppressionRules,
    },
};

pub struct DiffManager {
    tracking_map: StaticSuppressionMap,
    runtime_map: RuntimeSuppressionMap,
    suppress_all: bool,
    suppress_rules: SuppressionRules,
    file_exists: bool,
    ignore_diff: bool,
}

impl DiffManager {
    pub(crate) fn new(
        tracking_map: StaticSuppressionMap,
        file_exists: bool,
        ignore_diff: bool,
        suppress_all: bool,
        suppress_rules: SuppressionRules,
    ) -> Self {
        Self {
            tracking_map,
            runtime_map: RuntimeSuppressionMap::default(),
            suppress_all,
            suppress_rules,
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
        let is_suppressing = self.suppress_all || !self.suppress_rules.is_empty();
        let suppression_file =
            SuppressionFile::new(self.file_exists, is_suppressing, suppression_data);

        let (filtered_diagnostics, runtime_counts) = Self::suppress_lint_diagnostics(
            &suppression_file,
            messages,
            self.suppress_all,
            &self.suppress_rules,
        );

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
        suppress_all: bool,
        suppress_rules: &FxHashSet<String>,
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

                let filtered = lint_diagnostics
                    .into_iter()
                    .filter(|message| {
                        if message.error.severity != Severity::Error {
                            return true;
                        }
                        let Some(key) = oxc_code_short_canonical_name(&message.error.code) else {
                            return true;
                        };
                        !suppress_all && !suppress_rules.contains(&key)
                    })
                    .collect();

                (filtered, Some(runtime_suppression_tracking))
            }
            SuppressionFileState::Exists => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);
                let recorded_violations = suppression_file_state.suppression_data();

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

                        if suppress_all || suppress_rules.contains(&key) {
                            return false;
                        }

                        let Some(recorded_violations) = recorded_violations else {
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
