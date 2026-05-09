use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use rustc_hash::FxHashMap;

mod diff;
mod tracking;

pub use tracking::{
    DiagnosticCounts, Filename, SuppressionFile, SuppressionFileState, SuppressionTracking,
};

pub use diff::DiffManager;

type StaticSuppressionMap = Arc<FxHashMap<Filename, FxHashMap<String, DiagnosticCounts>>>;

type FileSuppressionsMap = FxHashMap<String, DiagnosticCounts>;

/// Thread-safe accumulator for runtime suppression counts from both oxlint and tsgo passes.
#[derive(Debug, Default)]
pub struct RuntimeSuppressionMap {
    inner: std::sync::Mutex<FxHashMap<Filename, FileSuppressionsMap>>,
}

impl RuntimeSuppressionMap {
    /// Merge runtime counts for a file. Counts are additive across passes.
    pub fn merge_file(&self, filename: Filename, counts: FxHashMap<String, DiagnosticCounts>) {
        let mut map = self.inner.lock().unwrap();
        let entry = map.entry(filename).or_default();
        for (rule, diagnostic) in counts {
            entry.entry(rule).or_insert(DiagnosticCounts { count: 0 }).count += diagnostic.count;
        }
    }

    /// Mark a file as seen (even if it has no violations).
    pub fn mark_seen(&self, filename: Filename) {
        let mut map = self.inner.lock().unwrap();
        map.entry(filename).or_default();
    }

    /// Consume into the inner map.
    pub fn into_inner(self) -> FxHashMap<Filename, FileSuppressionsMap> {
        self.inner.into_inner().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OxlintSuppressionFileAction {
    None,
    Updated,
    Exists,
    Created,
    HasUnprunedSuppressions,
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

#[derive(Debug)]
pub struct SuppressionManager {
    pub suppressions_by_file: Option<SuppressionTracking>,
    pub file_action: OxlintSuppressionFileAction,
    suppression_file_path: PathBuf,
    suppress_all: bool,
    prune_suppression: bool,
    //If the source of truth exists
    file_exists: bool,
}

impl SuppressionManager {
    pub fn load(cwd: &Path, file_path: &str, suppress_all: bool, prune_suppression: bool) -> Self {
        let suppression_file_path = cwd.join(file_path);
        let file_exists = suppression_file_path.exists();

        if !file_exists {
            let file_action = if suppress_all {
                OxlintSuppressionFileAction::Created
            } else {
                OxlintSuppressionFileAction::None
            };

            let suppressions_by_file =
                if suppress_all { Some(SuppressionTracking::default()) } else { None };

            return Self {
                suppressions_by_file,
                file_action,
                suppression_file_path,
                suppress_all,
                prune_suppression,
                file_exists,
            };
        }

        match SuppressionTracking::from_file(&suppression_file_path, cwd) {
            Ok(suppression_file) => Self {
                suppressions_by_file: Some(suppression_file),
                file_action: OxlintSuppressionFileAction::Exists,
                suppression_file_path,
                suppress_all,
                prune_suppression,
                file_exists,
            },
            Err(err) => Self {
                suppressions_by_file: None,
                file_action: OxlintSuppressionFileAction::Malformed(err),
                suppression_file_path,
                suppress_all,
                prune_suppression,
                file_exists,
            },
        }
    }

    /// Build a shared `DiffManager` that both oxlint and tsgo passes can write into.
    pub fn build_diff(&self) -> Arc<DiffManager> {
        let diff_manager = DiffManager::new(
            self.concurrent_map(),
            self.file_exists,
            self.file_action.ignore(),
            self.suppress_all,
        );

        Arc::new(diff_manager)
    }

    /// Finalize: compute new suppression state from static file + merged runtime map,
    /// then either update the suppression file or report diagnostics.
    ///
    /// # Panics
    /// Panics if `DiffManager` has any outstanding references to it still.
    pub fn finalize(
        &mut self,
        diff_manager: Arc<DiffManager>,
        tx_error: &DiagnosticSender,
        cwd: &Path,
    ) -> Result<(), OxcDiagnostic> {
        // Nothing to do if there's no suppression file and we're not creating one
        if self.suppressions_by_file.is_none() && !self.suppress_all {
            return Ok(());
        }

        let diff_manager = Arc::into_inner(diff_manager)
            .expect("DiffManager still has outstanding Arc references");
        let runtime_map = diff_manager.into_runtime_map().into_inner();

        let static_map = self.concurrent_map();

        if self.is_updating_file() {
            let new_map = if self.suppress_all {
                Self::compute_suppress(&static_map, &runtime_map)
            } else {
                Self::compute_prune(&static_map, &runtime_map, cwd)
            };
            self.suppressions_by_file = Some(SuppressionTracking::from_map(new_map));
            self.has_been_updated();
            self.write()
        } else {
            // Read-only mode: report diagnostics for any differences
            let (errors, has_unused) = Self::compute_diagnostics(&static_map, &runtime_map);
            if !errors.is_empty() {
                let diagnostics =
                    DiagnosticService::wrap_diagnostics(cwd, Path::new(""), "", errors);
                tx_error.send(diagnostics).unwrap();
            }
            if has_unused {
                self.file_action = OxlintSuppressionFileAction::HasUnprunedSuppressions;
            }
            Ok(())
        }
    }

    /// Suppress mode: overwrite counts with runtime values for seen files.
    /// Unseen files keep their static values.
    fn compute_suppress(
        static_map: &StaticSuppressionMap,
        runtime_map: &FxHashMap<Filename, FileSuppressionsMap>,
    ) -> FxHashMap<Filename, FileSuppressionsMap> {
        let mut result: FxHashMap<Filename, FileSuppressionsMap> = static_map.as_ref().clone();

        for (filename, runtime_rules) in runtime_map {
            if runtime_rules.is_empty() {
                // File was seen but had no error-level diagnostics — don't create a new entry
                // for it, but keep any existing static entry.
                continue;
            }
            // For seen files, replace all rules with the runtime counts.
            // This ensures that rules which are no longer error-severity
            // (e.g. warnings) are removed from the suppression file.
            result.insert(filename.clone(), runtime_rules.clone());
        }

        result
    }

    /// Prune mode: for seen files, remove rules absent in runtime and decrease counts.
    /// Also remove entries for files that no longer exist on disk.
    fn compute_prune(
        static_map: &StaticSuppressionMap,
        runtime_map: &FxHashMap<Filename, FileSuppressionsMap>,
        cwd: &Path,
    ) -> FxHashMap<Filename, FileSuppressionsMap> {
        let mut result: FxHashMap<Filename, FileSuppressionsMap> = FxHashMap::default();

        for (filename, static_rules) in static_map.iter() {
            if let Some(runtime_rules) = runtime_map.get(filename) {
                // File was linted — only keep rules that still fire, with min count
                let mut file_rules = FileSuppressionsMap::default();
                for (rule, static_count) in static_rules {
                    if let Some(runtime_count) = runtime_rules.get(rule) {
                        let count = static_count.count.min(runtime_count.count);
                        if count > 0 {
                            file_rules.insert(rule.clone(), DiagnosticCounts { count });
                        }
                    }
                    // Rule not in runtime = pruned, don't include
                }
                if !file_rules.is_empty() {
                    result.insert(filename.clone(), file_rules);
                }
            } else {
                // File not linted this run — check if it still exists on disk
                let file_path = cwd.join(filename.to_string());
                if file_path.exists() {
                    result.insert(filename.clone(), static_rules.clone());
                }
                // File doesn't exist on disk — drop it (deleted file cleanup)
            }
        }

        result
    }

    /// Read-only mode: generate at most two summary diagnostics.
    /// One for unused/stale suppressions (suggest prune), one for new violations (suggest suppress).
    fn compute_diagnostics(
        static_map: &StaticSuppressionMap,
        runtime_map: &FxHashMap<Filename, FileSuppressionsMap>,
    ) -> (Vec<OxcDiagnostic>, bool) {
        let mut has_unused = false;
        let mut has_new = false;

        for (filename, static_rules) in static_map.iter() {
            if let Some(runtime_rules) = runtime_map.get(filename) {
                for (rule, static_count) in static_rules {
                    if let Some(runtime_count) = runtime_rules.get(rule) {
                        if static_count.count > runtime_count.count {
                            has_unused = true;
                        } else if static_count.count < runtime_count.count {
                            has_new = true;
                        }
                    } else {
                        has_unused = true;
                    }
                    if has_unused && has_new {
                        break;
                    }
                }

                if !(has_unused && has_new) {
                    for rule in runtime_rules.keys() {
                        if !static_rules.contains_key(rule) {
                            has_new = true;
                            break;
                        }
                    }
                }
            } else if runtime_map.contains_key(filename) {
                has_unused = true;
            }
            if has_unused && has_new {
                break;
            }
        }

        if !has_new {
            for (filename, runtime_rules) in runtime_map {
                // Skip files with no error-level violations, because they only had warnings,
                // which are not tracked in suppressions.
                if !runtime_rules.is_empty() && !static_map.contains_key(filename) {
                    has_new = true;
                    break;
                }
            }
        }

        let mut errors = vec![];

        if has_unused {
            errors.push(
                OxcDiagnostic::error("There are suppressions that do not occur anymore.")
                    .with_help("Run `oxlint --prune-suppressions` to remove unused suppressions."),
            );
        }

        if has_new {
            errors.push(
                OxcDiagnostic::error(
                    "There are new violations not covered by the suppressions file.",
                )
                .with_help("Run `oxlint --suppress-all` to update the suppressions file."),
            );
        }

        (errors, has_unused)
    }

    fn has_been_updated(&mut self) {
        if self.file_action == OxlintSuppressionFileAction::Exists {
            self.file_action = OxlintSuppressionFileAction::Updated;
        }
    }

    fn concurrent_map(&self) -> StaticSuppressionMap {
        self.suppressions_by_file.as_ref().map(|f| Arc::clone(f.suppressions())).unwrap_or_default()
    }

    fn is_updating_file(&self) -> bool {
        self.suppress_all || self.prune_suppression
    }

    fn write(&self) -> Result<(), OxcDiagnostic> {
        let Some(file) = self.suppressions_by_file.as_ref() else {
            return Err(OxcDiagnostic::error(
                "You can't prune error messages if a bulk suppression file is malformed.",
            ));
        };

        file.save(&self.suppression_file_path)
    }
}
