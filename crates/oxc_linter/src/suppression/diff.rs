use std::{path::Path, sync::Arc};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    Message,
    suppression::{
        DiagnosticCounts, Filename, RuleName, StaticSuppressionMap, SuppressionDiff,
        SuppressionFile, SuppressionFileState, SuppressionSender,
    },
};

pub struct DiffManager {
    tracking_map: StaticSuppressionMap,
    has_suppress_all_arg: bool,
    is_updating: bool,
    file_exists: bool,
    ignore_diff: bool,
    ts_go_rules: Arc<FxHashSet<RuleName>>,
    suppressing_ts_go: bool,
}

impl DiffManager {
    pub fn new(
        tracking_map: StaticSuppressionMap,
        is_updating: bool,
        file_exists: bool,
        ignore_diff: bool,
        ts_go_rules: Arc<FxHashSet<RuleName>>,
        suppressing_ts_go: bool,
        has_suppress_all_arg: bool,
    ) -> Self {
        Self {
            tracking_map,
            has_suppress_all_arg,
            is_updating,
            file_exists,
            ignore_diff,
            ts_go_rules,
            suppressing_ts_go,
        }
    }

    pub fn diff_file(
        &self,
        file_path: &Path,
        cwd: &Path,
        messages: Vec<Message>,
        tx_error: &DiagnosticSender,
        suppression_sender: &SuppressionSender,
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
            SuppressionFile::new(self.file_exists, self.has_suppress_all_arg, suppression_data);

        let (filtered_diagnostics, runtime_suppression_tracking) =
            DiffManager::suppress_lint_diagnostics(&suppression_file, messages);

        if let Some(suppression_detected) = runtime_suppression_tracking {
            let diffs = self.diff_filename(&suppression_file, &suppression_detected, &filename);

            if !diffs.is_empty() && !self.is_updating {
                let errors = diffs.into_iter().map(Into::into).collect();
                let diagnostics = DiagnosticService::wrap_diagnostics(cwd, file_path, "", errors);
                tx_error.send(diagnostics).unwrap();
            } else if !diffs.is_empty() && self.is_updating {
                for diff in diffs {
                    suppression_sender.send(diff.clone()).unwrap();
                }
            }
        }

        filtered_diagnostics
    }

    pub fn prune_ts_go_rules(
        &self,
        file_path: &Path,
        cwd: &Path,
        tx_error: &DiagnosticSender,
        suppression_sender: &SuppressionSender,
    ) {
        let Ok(path) = file_path.strip_prefix(cwd) else {
            return;
        };

        let filename = Filename::new(path);

        let Some(suppressions) = self.tracking_map.get(&filename) else {
            return;
        };

        let pruned_rules: Vec<&RuleName> =
            suppressions.keys().filter(|&rule| self.prune_ts_go_rule(rule)).collect();

        if self.is_updating {
            for diff in DiffManager::pruned_rule(&filename, pruned_rules) {
                suppression_sender.send(diff).unwrap();
            }
        } else {
            let errors = DiffManager::pruned_rule(&filename, pruned_rules)
                .into_iter()
                .map(Into::into)
                .collect();
            let diagnostics = DiagnosticService::wrap_diagnostics(cwd, path, "", errors);
            tx_error.send(diagnostics).unwrap();
        }
    }

    pub fn skip(&self) -> bool {
        self.ignore_diff
    }

    fn pruned_rule(filename: &Filename, rule_names: Vec<&RuleName>) -> Vec<SuppressionDiff> {
        rule_names
            .into_iter()
            .map(|rule| SuppressionDiff::PrunedRuled { file: filename.clone(), rule: rule.clone() })
            .collect()
    }

    fn prune_ts_go_rule(&self, rule_key: &RuleName) -> bool {
        self.suppressing_ts_go && self.ts_go_rules.contains(rule_key)
    }

    fn prune_oxlint_rule(&self, rule_key: &RuleName) -> bool {
        !self.suppressing_ts_go && !self.ts_go_rules.contains(rule_key)
    }

    fn diff_filename(
        &self,
        suppression_file_state: &SuppressionFile<'_>,
        runtime_suppression: &FxHashMap<RuleName, DiagnosticCounts>,
        filename: &Filename,
    ) -> Vec<SuppressionDiff> {
        let static_suppression = match suppression_file_state.suppression_state() {
            SuppressionFileState::Ignored => return vec![],
            SuppressionFileState::New => FxHashMap::default(),
            SuppressionFileState::Exists => {
                if let Some(data) = suppression_file_state.suppression_data() {
                    data.to_owned()
                } else {
                    FxHashMap::default()
                }
            }
        };

        let mut diffs = vec![];

        if static_suppression.is_empty() && runtime_suppression.is_empty() {
            return diffs;
        }

        let static_suppression_keys = static_suppression.keys().collect::<FxHashSet<_>>();
        let runtime_suppression_keys = runtime_suppression.keys().collect::<FxHashSet<_>>();

        let pruned_rules = static_suppression_keys.difference(&runtime_suppression_keys);
        let new_violations = runtime_suppression_keys.difference(&static_suppression_keys);
        let existing_violations = static_suppression_keys.intersection(&runtime_suppression_keys);

        for &rule_key in pruned_rules {
            if self.prune_ts_go_rule(rule_key) || self.prune_oxlint_rule(rule_key) {
                diffs.push(SuppressionDiff::PrunedRuled {
                    file: filename.clone(),
                    rule: rule_key.clone(),
                });
            }
        }

        for &rule_key in new_violations {
            let Some(runtime_diagnostic) = runtime_suppression.get(rule_key) else {
                continue;
            };

            diffs.push(SuppressionDiff::Appeared {
                file: filename.clone(),
                rule: rule_key.clone(),
                count: runtime_diagnostic.count,
            });
        }

        for &rule_key in existing_violations {
            let file_diagnostic = &static_suppression[rule_key];
            let Some(runtime_diagnostic) = runtime_suppression.get(rule_key) else {
                continue;
            };

            if file_diagnostic.count > runtime_diagnostic.count {
                diffs.push(SuppressionDiff::Decreased {
                    file: filename.clone(),
                    rule: rule_key.clone(),
                    from: file_diagnostic.count,
                    to: runtime_diagnostic.count,
                });
            } else if file_diagnostic.count < runtime_diagnostic.count {
                diffs.push(SuppressionDiff::Increased {
                    file: filename.clone(),
                    rule: rule_key.clone(),
                    from: file_diagnostic.count,
                    to: runtime_diagnostic.count,
                });
            }
        }

        diffs
    }

    fn suppress_lint_diagnostics(
        suppression_file_state: &SuppressionFile<'_>,
        lint_diagnostics: Vec<Message>,
    ) -> (Vec<Message>, Option<FxHashMap<RuleName, DiagnosticCounts>>) {
        let build_suppression_map = |diagnostics: &Vec<Message>| {
            let mut suppression_tracking: FxHashMap<RuleName, DiagnosticCounts> =
                FxHashMap::default();
            for message in diagnostics {
                let Ok(key) = RuleName::try_from(message) else {
                    continue;
                };

                suppression_tracking
                    .entry(key)
                    .or_insert(DiagnosticCounts { count: 0 }) // Make a default
                    .count += 1;
            }

            suppression_tracking
        };

        match suppression_file_state.suppression_state() {
            SuppressionFileState::Ignored => (lint_diagnostics, None),
            SuppressionFileState::New => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                (lint_diagnostics, Some(runtime_suppression_tracking))
            }
            SuppressionFileState::Exists => {
                let runtime_suppression_tracking = build_suppression_map(&lint_diagnostics);

                let Some(recorded_violations) = suppression_file_state.suppression_data() else {
                    return (lint_diagnostics, Some(runtime_suppression_tracking));
                };

                let diagnostics_filtered = lint_diagnostics
                    .into_iter()
                    .filter(|message| {
                        let Ok(key) = RuleName::try_from(message) else {
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
