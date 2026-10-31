use std::path::Path;

use oxc_diagnostics::Severity;
use rustc_hash::FxHashMap;

use crate::{
    Message, oxc_code_short_canonical_name,
    suppression::{
        DiagnosticCounts, Filename, RuntimeSuppressionMap, StaticSuppressionMap, SuppressionFile,
        SuppressionFileState,
    },
};

/// A split collection of lint diagnostics into suppressed/unsuppressed diagnostics.
pub struct SuppressionPartition {
    /// Messages that should be shown to the user.
    pub unsuppressed: Vec<Message>,
    /// Messages that should not be shown to the user because they are suppressed based on the baseline.
    pub suppressed: Vec<Message>,
}

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

        let (partition, runtime_counts) =
            Self::suppress_lint_diagnostics(&suppression_file, messages, false);

        if let Some(counts) = runtime_counts {
            self.runtime_map.merge_file(filename, counts);
        }

        partition.unsuppressed
    }

    /// Partition a file's messages into suppressed and unsuppressed diagnostics using the recorded baseline,
    /// without mutating runtime state.
    ///
    /// Only rules whose runtime count exactly matches the baseline are suppressed. Rules whose
    /// count increased or decreased are surfaced in full so the language server can prompt users
    /// to fix new violations or prune stale suppressions.
    pub fn partition_file(
        &self,
        file_path: &Path,
        cwd: &Path,
        messages: Vec<Message>,
    ) -> SuppressionPartition {
        if self.ignore_diff {
            return SuppressionPartition { unsuppressed: messages, suppressed: Vec::new() };
        }

        let Ok(file_path) = file_path.strip_prefix(cwd) else {
            return SuppressionPartition { unsuppressed: messages, suppressed: Vec::new() };
        };

        let filename = Filename::new(file_path);
        let suppression_data = self.tracking_map.get(&filename);
        let suppression_file =
            SuppressionFile::new(self.file_exists, self.suppress_all, suppression_data);

        Self::suppress_lint_diagnostics(&suppression_file, messages, true).0
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

    /// Partitions the input lint diagnostics into separate groups of suppressed and
    /// unsuppressed diagnostics and also returns the actual runtime counts for the diagnostics.
    ///
    /// Accepts a `require_exact_baseline` argument that determines how errors are surfaced if the runtime counts are
    /// different than the baseline count for the file:
    /// - `require_exact_baseline: true` - errors are only surfaced if the runtime counts exactly match the baseline counts.
    /// - `require_exact_baseline: false` - errors are surfaced even if the runtime counts do not exactly match the baseline counts.
    fn suppress_lint_diagnostics(
        suppression_file_state: &SuppressionFile<'_>,
        lint_diagnostics: Vec<Message>,
        require_exact_baseline: bool,
    ) -> (SuppressionPartition, Option<FxHashMap<String, DiagnosticCounts>>) {
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
            SuppressionFileState::Ignored => (
                SuppressionPartition { unsuppressed: lint_diagnostics, suppressed: Vec::new() },
                None,
            ),
            SuppressionFileState::New => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                if require_exact_baseline {
                    return (
                        SuppressionPartition {
                            unsuppressed: lint_diagnostics,
                            suppressed: Vec::new(),
                        },
                        Some(runtime_suppression_tracking),
                    );
                }

                // Error-severity diagnostics are being written to the new suppressions file, so
                // they are suppressed. Only warnings surface.
                let (suppressed, unsuppressed): (Vec<Message>, Vec<Message>) = lint_diagnostics
                    .into_iter()
                    .partition(|message| message.error.severity == Severity::Error);

                (
                    SuppressionPartition { unsuppressed, suppressed },
                    Some(runtime_suppression_tracking),
                )
            }
            SuppressionFileState::Exists => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                let Some(recorded_violations) = suppression_file_state.suppression_data() else {
                    return (
                        SuppressionPartition {
                            unsuppressed: lint_diagnostics,
                            suppressed: Vec::new(),
                        },
                        Some(runtime_suppression_tracking),
                    );
                };

                let (unsuppressed, suppressed) =
                    lint_diagnostics.into_iter().partition(|message: &Message| {
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

                        // Diagnostics are surfaced as long as we haven't exceeded the expected count based on the baseline
                        // (e.g., for LSP). However, if we require an exact baseline (like in the CLI) then diagnostics are
                        // only surfaced if the baseline exactly matches.
                        if require_exact_baseline {
                            count_runtime.count != count_file.count
                        } else {
                            // Only surface diagnostics if we've exceeded the expected count.
                            count_runtime.count > count_file.count
                        }
                    });

                (
                    SuppressionPartition { unsuppressed, suppressed },
                    Some(runtime_suppression_tracking),
                )
            }
        }
    }
}
