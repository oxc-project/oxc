use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::Config;

mod diff;
mod tracking;

pub use tracking::{
    DiagnosticCounts, Filename, RuleName, SuppressionDiff, SuppressionFile, SuppressionFileState,
    SuppressionTracking,
};

pub use diff::DiffManager;

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
    ts_go_rules: Arc<FxHashSet<RuleName>>,
}

impl SuppressionManager {
    pub fn load(
        cwd: &Path,
        file_path: &str,
        suppress_all: bool,
        prune_suppression: bool,
        lint_config: &Config,
    ) -> (Self, SuppressionSender) {
        let path = cwd.join(file_path);
        let (sender, receiver): (SuppressionSender, SuppressionReceiver) =
            std::sync::mpsc::channel();

        let ts_go_rules = lint_config
            .base
            .rules
            .iter()
            .filter_map(|(rule, _)| {
                if rule.is_tsgolint_rule() {
                    Some(RuleName::new("typescript-eslint", rule.name()))
                } else {
                    None
                }
            })
            .collect::<FxHashSet<RuleName>>();

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
                    suppression_path: path,
                    prune_suppression,
                    suppress_all,
                    receiver,
                    ts_go_rules: Arc::new(ts_go_rules),
                },
                sender,
            );
        }

        match SuppressionTracking::from_file(&path, cwd) {
            Ok(suppression_file) => (
                Self {
                    suppressions_by_file: Some(suppression_file),
                    manager_status: OxlintSuppressionFileAction::Exists,
                    suppression_path: path,
                    file_exists: true,
                    prune_suppression,
                    suppress_all,
                    receiver,
                    ts_go_rules: Arc::new(ts_go_rules),
                },
                sender,
            ),
            Err(err) => (
                Self {
                    suppressions_by_file: None,
                    manager_status: OxlintSuppressionFileAction::Malformed(err),
                    suppression_path: path,
                    file_exists: true,
                    prune_suppression,
                    suppress_all,
                    receiver,
                    ts_go_rules: Arc::new(ts_go_rules),
                },
                sender,
            ),
        }
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

    pub fn build_diff(&self, suppressing_ts_go: bool) -> Arc<DiffManager> {
        let diff_manager = DiffManager::new(
            self.concurrent_map(),
            self.is_updating_file(),
            self.file_exists,
            self.manager_status.ignore(),
            Arc::<FxHashSet<RuleName>>::clone(&self.ts_go_rules),
            suppressing_ts_go,
            self.suppress_all,
        );

        Arc::new(diff_manager)
    }

    fn has_been_updated(&mut self) {
        if self.manager_status == OxlintSuppressionFileAction::Exists {
            self.manager_status = OxlintSuppressionFileAction::Updated;
        }
    }

    fn concurrent_map(&self) -> StaticSuppressionMap {
        self.suppressions_by_file.as_ref().map(|f| Arc::clone(f.suppressions())).unwrap_or_default()
    }

    fn is_updating_file(&self) -> bool {
        self.suppress_all || self.prune_suppression
    }

    fn update(&mut self, diff: SuppressionDiff) {
        let Some(file) = self.suppressions_by_file.as_mut() else {
            return;
        };

        file.update(diff);
    }

    fn write(&self) -> Result<(), OxcDiagnostic> {
        if !self.file_exists && (self.prune_suppression && !self.suppress_all) {
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
}
