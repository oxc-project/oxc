use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::Message;

mod tracking;

pub use tracking::{
    DiagnosticCounts, Filename, RuleName, SuppressionDiff, SuppressionFile, SuppressionFileState,
    SuppressionTracking,
};

type StaticSuppressionMap = Arc<FxHashMap<Filename, FxHashMap<RuleName, DiagnosticCounts>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OxlintSuppressionFileAction {
    None,
    Updated,
    Exists,
    Created,
    Malformed(OxcDiagnostic),
    UnableToPerformFsOperation(OxcDiagnostic),
}

impl OxlintSuppressionFileAction {
    fn ignore(&self) -> bool {
        *self != OxlintSuppressionFileAction::Created
            && *self != OxlintSuppressionFileAction::Updated
            && *self != OxlintSuppressionFileAction::Exists
    }
}

pub type SuppressionSender = mpsc::Sender<SuppressionDiff>;
pub type SuppressionReceiver = mpsc::Receiver<SuppressionDiff>;

#[derive(Debug)]
pub struct SuppressionManager {
    pub suppressions_by_file: Option<SuppressionTracking>,
    pub manager_status: OxlintSuppressionFileAction,
    suppression_path: PathBuf,
    suppress_all: bool,
    prune_suppression: bool,
    //If the source of truth exists
    file_exists: bool,
    receiver: SuppressionReceiver,
}

impl SuppressionManager {
    pub fn load(
        path: &Path,
        suppress_all: bool,
        prune_suppression: bool,
    ) -> (Self, SuppressionSender) {
        let (sender, receiver): (SuppressionSender, SuppressionReceiver) =
            std::sync::mpsc::channel();

        if !path.exists() {
            let manager_status = if suppress_all {
                OxlintSuppressionFileAction::Created
            } else {
                OxlintSuppressionFileAction::None
            };

            let suppressions_by_file =
                if suppress_all { Some(SuppressionTracking::default()) } else { None };

            return (
                Self {
                    suppressions_by_file,
                    manager_status,
                    file_exists: false,
                    suppression_path: path.into(),
                    prune_suppression,
                    suppress_all,
                    receiver,
                },
                sender,
            );
        }

        match SuppressionTracking::from_file(path) {
            Ok(suppression_file) => (
                Self {
                    suppressions_by_file: Some(suppression_file),
                    manager_status: OxlintSuppressionFileAction::Exists,
                    suppression_path: path.into(),
                    file_exists: true,
                    prune_suppression,
                    suppress_all,
                    receiver,
                },
                sender,
            ),
            Err(err) => (
                Self {
                    suppressions_by_file: None,
                    manager_status: OxlintSuppressionFileAction::Malformed(err),
                    suppression_path: path.into(),
                    file_exists: true,
                    prune_suppression,
                    suppress_all,
                    receiver,
                },
                sender,
            ),
        }
    }

    pub fn ignore(&self) -> bool {
        self.manager_status.ignore()
    }

    pub fn has_been_updated(&mut self) {
        if self.manager_status == OxlintSuppressionFileAction::Exists {
            self.manager_status = OxlintSuppressionFileAction::Updated;
        }
    }

    pub fn concurrent_map(&self) -> StaticSuppressionMap {
        self.suppressions_by_file.as_ref().map(|f| Arc::clone(f.suppressions())).unwrap_or_default()
    }

    pub fn is_updating_file(&self) -> bool {
        self.suppress_all || self.prune_suppression
    }

    pub fn exists_suppression_file(&self) -> bool {
        self.file_exists
    }

    pub fn update(&mut self, diff: SuppressionDiff) {
        let Some(file) = self.suppressions_by_file.as_mut() else {
            return;
        };

        file.update(diff);
    }

    pub fn write(&self) -> Result<(), OxcDiagnostic> {
        if !self.file_exists && self.prune_suppression {
            return Err(OxcDiagnostic::error(
                "You can't prune error messages if a bulk suppression file doesn't exist.",
            ));
        }

        let Some(file) = self.suppressions_by_file.as_ref() else {
            return Err(OxcDiagnostic::error(
                "You can't prune error messages if a bulk suppression file is malformed.",
            ));
        };

        file.save(&self.suppression_path)
    }

    pub fn report_suppression(&mut self) -> Result<(), OxcDiagnostic> {
        let mut have_at_least_one_diff = false;
        while let Ok(diff) = self.receiver.recv() {
            if !have_at_least_one_diff {
                have_at_least_one_diff = true;
            }

            self.update(diff);
        }

        if have_at_least_one_diff && self.is_updating_file() {
            self.has_been_updated();
            self.write()
        } else {
            Ok(())
        }
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
            let Some(runtime_diagnostic) = runtime_suppression.get(rule_key) else {
                continue;
            };

            diff.push(SuppressionDiff::Appeared {
                file: filename.clone(),
                rule: (*rule_key).clone(),
                count: runtime_diagnostic.count,
            });
        }

        for rule_key in existing_violations {
            let file_diagnostic = &static_suppression[rule_key];
            let Some(runtime_diagnostic) = runtime_suppression.get(rule_key) else {
                continue;
            };

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
