use std::{hash::BuildHasherDefault, path::Path, sync::Arc};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use crate::Message;

mod tracking;

pub use tracking::{
    DiagnosticCounts, Filename, RuleName, SuppressionDiff, SuppressionFile, SuppressionFileState,
    SuppressionId, SuppressionTracking,
};

type StaticSuppressionMap = papaya::HashMap<
    Arc<Filename>,
    FxHashMap<RuleName, DiagnosticCounts>,
    BuildHasherDefault<FxHasher>,
>;

#[derive(Clone, Debug)]
pub struct SuppressionManager {
    pub suppressions_by_file: SuppressionTracking,
    suppress_all: bool,
    prune_suppression: bool,
    //If the source of truth exists
    file_exists: bool,
}

impl SuppressionManager {
    pub fn load(
        path: &Path,
        suppress_all: bool,
        prune_suppression: bool,
    ) -> Result<Self, OxcDiagnostic> {
        if !path.exists() {
            return Ok(Self {
                suppressions_by_file: SuppressionTracking::default(),
                file_exists: false,
                prune_suppression,
                suppress_all,
            });
        }

        let suppression_file = SuppressionTracking::from_file(path)?;

        Ok(Self {
            suppressions_by_file: suppression_file,
            file_exists: true,
            prune_suppression,
            suppress_all,
        })
    }

    pub fn concurrent_map(&self) -> StaticSuppressionMap {
        let concurrent_papaya = papaya::HashMap::builder()
            .hasher(BuildHasherDefault::default())
            .resize_mode(papaya::ResizeMode::Blocking)
            .build();

        if !self.file_exists {
            return concurrent_papaya;
        }

        for file_key in self.suppressions_by_file.suppressions().keys() {
            concurrent_papaya.pin().insert(
                Arc::new(file_key.clone()),
                self.suppressions_by_file.suppressions()[file_key].clone(),
            );
        }

        concurrent_papaya
    }

    pub fn is_updating_file(&self) -> bool {
        self.suppress_all || self.prune_suppression
    }

    pub fn exists_suppression_file(&self) -> bool {
        self.file_exists
    }

    pub fn update(&mut self, diff: SuppressionDiff) {
        self.suppressions_by_file.update(diff);
    }

    pub fn write(&self, path: &Path) -> Result<(), OxcDiagnostic> {
        if !self.file_exists && self.prune_suppression {
            return Err(OxcDiagnostic::error(
                "You can't prune error messages if a bulk suppression file doesn't exist.",
            ));
        }

        self.suppressions_by_file.save(path)
    }

    pub fn diff_filename(
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

        let mut diff = vec![];

        if static_suppression.is_empty() && runtime_suppression.is_empty() {
            return diff;
        }

        let static_suppression_keys = static_suppression.keys().collect::<FxHashSet<_>>();
        let runtime_suppression_keys = runtime_suppression.keys().collect::<FxHashSet<_>>();

        let pruned_rules = static_suppression_keys.difference(&runtime_suppression_keys);
        let new_violations = runtime_suppression_keys.difference(&static_suppression_keys);
        let existing_violations = static_suppression_keys.intersection(&runtime_suppression_keys);

        for rule_key in pruned_rules {
            diff.push(SuppressionDiff::PrunedRuled {
                file: filename.clone(),
                rule: (*rule_key).clone(),
            });
        }

        for rule_key in new_violations {
            let runtime_diagnostic = runtime_suppression.get(rule_key).unwrap();

            diff.push(SuppressionDiff::Appeared {
                file: filename.clone(),
                rule: (*rule_key).clone(),
                count: runtime_diagnostic.count,
            });
        }

        for rule_key in existing_violations {
            let file_diagnostic = &static_suppression[rule_key];
            let runtime_diagnostic = runtime_suppression.get(rule_key).unwrap();

            if file_diagnostic.count > runtime_diagnostic.count {
                diff.push(SuppressionDiff::Decreased {
                    file: filename.clone(),
                    rule: (*rule_key).clone(),
                    from: file_diagnostic.count,
                    to: runtime_diagnostic.count,
                });
            } else if file_diagnostic.count < runtime_diagnostic.count {
                diff.push(SuppressionDiff::Increased {
                    file: filename.clone(),
                    rule: (*rule_key).clone(),
                    from: file_diagnostic.count,
                    to: runtime_diagnostic.count,
                });
            }
        }

        diff
    }

    pub fn suppress_lint_diagnostics(
        suppression_file_state: &SuppressionFile<'_>,
        lint_diagnostics: Vec<Message>,
    ) -> (Vec<Message>, Option<FxHashMap<RuleName, DiagnosticCounts>>) {
        let build_suppression_map = |diagnostics: &Vec<Message>| {
            let mut suppression_tracking: FxHashMap<RuleName, DiagnosticCounts> =
                FxHashMap::default();
            for message in diagnostics {
                let Some(SuppressionId(_, rule_name)) = &message.suppression_id else {
                    continue;
                };

                if let Some(violation_count) = suppression_tracking.get_mut(rule_name) {
                    violation_count.count += 1;
                } else {
                    suppression_tracking.insert(rule_name.clone(), DiagnosticCounts { count: 1 });
                }
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
                        let Some(SuppressionId(_, rule_name)) = &message.suppression_id else {
                            return false;
                        };

                        let Some(count_file) = recorded_violations.get(rule_name) else {
                            return true;
                        };

                        let Some(count_runtime) = runtime_suppression_tracking.get(rule_name)
                        else {
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
