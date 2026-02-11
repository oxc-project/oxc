use std::{ffi::OsStr, fs, hash::BuildHasherDefault, path::Path, sync::Arc};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{Message, read_to_string};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DiagnosticCounts {
    pub count: usize,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(default)]
pub struct Filename(String);

impl Filename {
    pub fn new(path: &Path) -> Self {
        Self(path.as_os_str().to_string_lossy().to_string())
    }
}

impl std::fmt::Display for Filename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(default)]
pub struct RuleName(String);

#[derive(Debug, Default, Clone, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub struct SuppressionId(pub Filename, pub RuleName);

impl SuppressionId {
    pub fn new(path: &Path, plugin_name: &str, rule_name: &str) -> Self {
        Self(Filename::new(path), RuleName::new(plugin_name, rule_name))
    }
}

impl RuleName {
    pub fn new(plugin_name: &str, rule_name: &str) -> Self {
        let compose_key = format!("{plugin_name}/{rule_name}");

        Self(compose_key)
    }
}

impl std::fmt::Display for RuleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SuppressionTracking {
    version: String,
    suppressions: FxHashMap<Filename, FxHashMap<RuleName, DiagnosticCounts>>,
}

impl Default for SuppressionTracking {
    fn default() -> Self {
        Self { version: "0.1.0".to_string(), suppressions: FxHashMap::default() }
    }
}

impl SuppressionTracking {
    pub fn from_file(path: &Path) -> Result<Self, OxcDiagnostic> {
        let string = read_to_string(path).map_err(|e| {
            OxcDiagnostic::error(format!(
                "Failed to parse suppression rules file {} with error {e:?}",
                path.display()
            ))
        })?;

        let json = serde_json::from_str::<serde_json::Value>(&string).map_err(|error| {
            let ext = path.extension().and_then(OsStr::to_str);
            let err = match ext {
                // syntax error
                Some(ext) if ext == "json" => error.to_string(),
                Some(_) => "Only JSON suppression rules files are supported".to_string(),
                None => {
                    format!(
                        "{error}, if the configuration is not a JSON file, please use JSON instead."
                    )
                }
            };
            OxcDiagnostic::error(format!(
                "Failed to parse oxlint config {}.\n{err}",
                path.display()
            ))
        })?;

        let config = Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to parse rules suppression file with error {err:?}"
            ))
        })?;

        Ok(config)
    }

    pub fn update(&mut self, diff: SuppressionDiff) {
        match diff {
            SuppressionDiff::Increased { file, rule, from, to } => {
                self.suppressions.get_mut(&file).unwrap().get_mut(&rule).unwrap().count = to;
            }
            SuppressionDiff::Decreased { file, rule, from, to } => {
                self.suppressions.get_mut(&file).unwrap().get_mut(&rule).unwrap().count = to;
            }
            SuppressionDiff::PrunedRuled { file, rule } => {
                let file_map = self.suppressions.get(&file).unwrap();

                if file_map.len() == 1 {
                    self.suppressions.remove(&file);
                } else {
                    self.suppressions.get_mut(&file).unwrap().remove(&rule);
                }
            }
            SuppressionDiff::Appeared { file, rule, count } => {
                if let Some(file) = self.suppressions.get_mut(&file) {
                    file.insert(rule, DiagnosticCounts { count });
                } else {
                    let mut file_diagnostic: FxHashMap<RuleName, DiagnosticCounts> =
                        FxHashMap::default();
                    file_diagnostic.insert(rule, DiagnosticCounts { count });
                    self.suppressions.insert(file, file_diagnostic);
                }
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), OxcDiagnostic> {
        let content = serde_json::to_string_pretty(&self).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to serialize suppression file: {err}"))
        })?;

        fs::write(path, content).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to write suppression file: {err}"))
        })?;

        Ok(())
    }
}

type StaticSuppressionMap = papaya::HashMap<
    Arc<Filename>,
    FxHashMap<RuleName, DiagnosticCounts>,
    BuildHasherDefault<FxHasher>,
>;

#[derive(Clone, Debug)]
pub struct SuppressionManager {
    pub suppressions_by_file: SuppressionTracking,
    pub suppression_key_set: FxHashSet<(Filename, RuleName)>,
    pub concurrent_suppression_by_file: StaticSuppressionMap,
    suppress_all: bool,
    prune_suppression: bool,
    //If the source of truth exists
    file_exists: bool,
}

#[derive(Debug, Clone)]
pub enum SuppressionDiff {
    Increased { file: Filename, rule: RuleName, from: usize, to: usize },
    Decreased { file: Filename, rule: RuleName, from: usize, to: usize },
    PrunedRuled { file: Filename, rule: RuleName },
    Appeared { file: Filename, rule: RuleName, count: usize },
}

impl Into<OxcDiagnostic> for SuppressionDiff {
    fn into(self) -> OxcDiagnostic {
        let message = match self {
            SuppressionDiff::Increased { file, rule, from, to } => {
                format!("The {rule} errors in {file} have increased from {from} to {to}.")
            }
            SuppressionDiff::Decreased { file, rule, from, to } => {
                format!("The {rule} errors in {file} have decreased from {from} to {to}.")
            }
            SuppressionDiff::PrunedRuled { file, rule } => {
                format!("All {rule} errors has been pruned in {file}.")
            }
            SuppressionDiff::Appeared { file, rule, count } => {
                format!("New {rule} error have appeared {count} times in {file}.")
            }
        };

        OxcDiagnostic::error(message)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SuppressionFileState {
    Ignored,
    New,
    Exists,
}

#[derive(Debug)]
pub struct SuppressionFile<'a> {
    state: SuppressionFileState,
    suppression_data: Option<&'a FxHashMap<RuleName, DiagnosticCounts>>,
}

impl<'a> Default for SuppressionFile<'a> {
    fn default() -> Self {
        Self { state: SuppressionFileState::Ignored, suppression_data: None }
    }
}

impl<'a> SuppressionFile<'a> {
    pub fn new<'b>(
        file_exists: bool,
        suppress_all: bool,
        suppression_data: Option<&'b FxHashMap<RuleName, DiagnosticCounts>>,
    ) -> Self
    where
        'b: 'a,
    {
        if !file_exists && !suppress_all {
            return Self { state: SuppressionFileState::Ignored, suppression_data: None };
        }

        if !file_exists && suppress_all {
            return Self { state: SuppressionFileState::New, suppression_data: None };
        }

        Self { state: SuppressionFileState::Exists, suppression_data }
    }

    pub fn suppression_state(&self) -> &SuppressionFileState {
        &self.state
    }
}

impl SuppressionManager {
    pub fn load(
        path: &Path,
        suppress_all: bool,
        prune_suppression: bool,
    ) -> Result<Self, OxcDiagnostic> {
        let concurrent_papaya = papaya::HashMap::builder()
            .hasher(BuildHasherDefault::default())
            .resize_mode(papaya::ResizeMode::Blocking)
            .build();

        if !path.exists() {
            return Ok(Self {
                suppressions_by_file: SuppressionTracking::default(),
                suppression_key_set: FxHashSet::default(),
                concurrent_suppression_by_file: concurrent_papaya,
                file_exists: false,
                prune_suppression,
                suppress_all,
            });
        }

        let suppression_file = SuppressionTracking::from_file(path)?;

        let mut set: FxHashSet<(Filename, RuleName)> = FxHashSet::default();

        let mut keys_iterator = suppression_file.suppressions.keys().into_iter();

        while let Some(file_key) = keys_iterator.next() {
            let mut values_iterator =
                suppression_file.suppressions.get(file_key).unwrap().keys().into_iter();

            while let Some(rule_name_key) = values_iterator.next() {
                set.insert((file_key.clone(), rule_name_key.clone()));
            }

            concurrent_papaya.pin().insert(
                Arc::new(file_key.clone()),
                suppression_file.suppressions.get(file_key).unwrap().to_owned(),
            );
        }

        Ok(Self {
            suppressions_by_file: suppression_file,
            suppression_key_set: set,
            concurrent_suppression_by_file: concurrent_papaya,
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

        let mut keys_iterator = self.suppressions_by_file.suppressions.keys().into_iter();

        while let Some(file_key) = keys_iterator.next() {
            concurrent_papaya.pin().insert(
                Arc::new(file_key.clone()),
                self.suppressions_by_file.suppressions.get(file_key).unwrap().to_owned(),
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
        suppression_file_state: SuppressionFile<'_>,
        runtime_suppression: &FxHashMap<RuleName, DiagnosticCounts>,
        filename: &Filename,
    ) -> Vec<SuppressionDiff> {
        let static_suppression = match suppression_file_state.suppression_state() {
            SuppressionFileState::Ignored => return vec![],
            SuppressionFileState::New => FxHashMap::default(),
            SuppressionFileState::Exists => {
                if let Some(data) = suppression_file_state.suppression_data {
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

        let static_suppression_keys = FxHashSet::from_iter(static_suppression.keys());
        let runtime_suppression_keys = FxHashSet::from_iter(runtime_suppression.keys());

        let mut pruned_rules = static_suppression_keys.difference(&runtime_suppression_keys);
        let mut new_violations = runtime_suppression_keys.difference(&static_suppression_keys);
        let mut existing_violations =
            static_suppression_keys.intersection(&runtime_suppression_keys);

        while let Some(rule_key) = pruned_rules.next() {
            diff.push(SuppressionDiff::PrunedRuled {
                file: filename.clone(),
                rule: (*rule_key).clone(),
            });
        }

        while let Some(rule_key) = new_violations.next() {
            let runtime_diagnostic = runtime_suppression.get(rule_key).unwrap();

            diff.push(SuppressionDiff::Appeared {
                file: filename.clone(),
                rule: (*rule_key).clone(),
                count: runtime_diagnostic.count,
            });
        }

        while let Some(rule_key) = existing_violations.next() {
            let file_diagnostic = static_suppression.get(rule_key).unwrap();
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
                    rule: (*rule_key).clone(), // Deref??? es un string por debajo al final
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
            diagnostics.iter().for_each(|message| {
                let Some(SuppressionId(_, rule_name)) = &message.suppression_id else {
                    return;
                };

                if let Some(violation_count) = suppression_tracking.get_mut(&rule_name) {
                    violation_count.count += 1;
                } else {
                    suppression_tracking.insert(rule_name.clone(), DiagnosticCounts { count: 1 });
                }
            });

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

                let Some(recorded_violations) = suppression_file_state.suppression_data else {
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
