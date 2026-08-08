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

        let (surfaced, _suppressed, runtime_counts) =
            Self::partition_lint_diagnostics(&suppression_file, messages);

        if let Some(counts) = runtime_counts {
            self.runtime_map.merge_file(filename, counts);
        }

        surfaced
    }

    /// Partition a file's messages into `(surfaced, suppressed)` using the recorded baseline,
    /// without mutating runtime state.
    ///
    /// This mirrors [`Self::collect_file`]'s per-rule count semantics, but instead of dropping
    /// suppressed diagnostics it returns them alongside the surfaced ones. The language server
    /// uses this to render suppressed violations as faded diagnostics rather than hiding them.
    pub fn partition_file(
        &self,
        file_path: &Path,
        cwd: &Path,
        messages: Vec<Message>,
    ) -> (Vec<Message>, Vec<Message>) {
        if self.ignore_diff {
            return (messages, Vec::new());
        }

        let Ok(file_path) = file_path.strip_prefix(cwd) else {
            return (messages, Vec::new());
        };

        let filename = Filename::new(file_path);
        let suppression_data = self.tracking_map.get(&filename);
        let suppression_file =
            SuppressionFile::new(self.file_exists, self.suppress_all, suppression_data);

        let (surfaced, suppressed, _runtime_counts) =
            Self::partition_lint_diagnostics(&suppression_file, messages);

        (surfaced, suppressed)
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

    /// Partition messages into `(surfaced, suppressed, runtime_counts)` for a file.
    ///
    /// `surfaced` are the diagnostics that should be reported (new/increased violations plus all
    /// warnings); `suppressed` are the error-severity diagnostics covered by the baseline. Callers
    /// that only care about surfaced diagnostics (e.g. the CLI) discard `suppressed`; callers that
    /// want to render suppressed diagnostics differently (e.g. the language server) keep them.
    fn partition_lint_diagnostics(
        suppression_file_state: &SuppressionFile<'_>,
        lint_diagnostics: Vec<Message>,
    ) -> (Vec<Message>, Vec<Message>, Option<FxHashMap<String, DiagnosticCounts>>) {
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
            SuppressionFileState::Ignored => (lint_diagnostics, Vec::new(), None),
            SuppressionFileState::New => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                // Error-severity diagnostics are being written to the new suppressions file, so
                // they are suppressed. Only warnings surface.
                let (suppressed, surfaced): (Vec<Message>, Vec<Message>) = lint_diagnostics
                    .into_iter()
                    .partition(|message| message.error.severity == Severity::Error);

                (surfaced, suppressed, Some(runtime_suppression_tracking))
            }
            SuppressionFileState::Exists => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                let Some(recorded_violations) = suppression_file_state.suppression_data() else {
                    return (lint_diagnostics, Vec::new(), Some(runtime_suppression_tracking));
                };

                let is_surfaced = |message: &Message| {
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
                };

                let (surfaced, suppressed): (Vec<Message>, Vec<Message>) =
                    lint_diagnostics.into_iter().partition(is_surfaced);

                (surfaced, suppressed, Some(runtime_suppression_tracking))
            }
        }
    }
}
