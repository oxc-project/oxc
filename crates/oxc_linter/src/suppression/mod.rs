use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Mutex,
};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::read_to_string;

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
}

#[derive(Clone, Debug)]
pub struct SuppressionManager {
    pub suppressions_by_file: SuppressionTracking,
    pub suppression_key_set: FxHashSet<(Filename, RuleName)>,
    suppress_all: bool,
    prune_suppression: bool,
    //If the source of truth exists
    file_exists: bool,
}

impl Default for SuppressionManager {
    fn default() -> Self {
        Self {
            suppressions_by_file: SuppressionTracking::default(),
            suppression_key_set: FxHashSet::default(),
            suppress_all: false,
            prune_suppression: false,
            file_exists: false,
        }
    }
}

#[derive(Debug)]
pub enum SuppressionDiff {
    Increased { file: Filename, rule: RuleName, from: usize, to: usize },
    Decreased { file: Filename, rule: RuleName, from: usize, to: usize },
    PrunedRuled { file: Filename, rule: RuleName },
    Appeared { file: Filename, rule: RuleName },
}

pub enum SuppressionFileState<'a> {
    Ignored,
    New,
    Exists { file_suppressions: Option<&'a FxHashMap<RuleName, DiagnosticCounts>> },
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
                suppression_key_set: FxHashSet::default(),
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
        }

        Ok(Self {
            suppressions_by_file: suppression_file,
            suppression_key_set: set,
            file_exists: true,
            prune_suppression,
            suppress_all,
        })
    }

    pub fn get_suppression_per_file(&self, path: &Path) -> SuppressionFileState<'_> {
        if !self.file_exists && !self.suppress_all {
            return SuppressionFileState::Ignored;
        }

        if !self.file_exists && self.suppress_all {
            return SuppressionFileState::New;
        }

        let filename = Filename::new(path);

        SuppressionFileState::Exists {
            file_suppressions: self.suppressions_by_file.suppressions.get(&filename),
        }
    }

    pub fn diff(
        &self,
        runtime_suppressions: &FxHashMap<Filename, FxHashMap<RuleName, DiagnosticCounts>>,
        runtime_set: &FxHashSet<(Filename, RuleName)>,
    ) -> Vec<SuppressionDiff> {
        let mut diff: Vec<SuppressionDiff> = vec![];

        if self.suppression_key_set.is_empty() && runtime_set.is_empty() {
            return diff;
        }

        if self.suppression_key_set.is_empty() {
            return runtime_set
                .iter()
                .map(|(file_key, rule_key)| SuppressionDiff::Appeared {
                    file: file_key.clone(),
                    rule: rule_key.clone(),
                })
                .collect();
        }

        if runtime_set.is_empty() {
            return self
                .suppression_key_set
                .iter()
                .map(|(file_key, rule_key)| SuppressionDiff::PrunedRuled {
                    file: file_key.clone(),
                    rule: rule_key.clone(),
                })
                .collect();
        }

        let mut pruned_rules = self.suppression_key_set.difference(runtime_set);
        let mut new_violations = runtime_set.difference(&self.suppression_key_set);
        let mut existing_violations = self.suppression_key_set.intersection(runtime_set);

        println!("pruned {:?}", pruned_rules);
        println!("new_violations {:?}", new_violations);
        println!("existing_violations {:?}", existing_violations);

        while let Some((file_key, rule_key)) = pruned_rules.next() {
            diff.push(SuppressionDiff::PrunedRuled {
                file: file_key.clone(),
                rule: rule_key.clone(),
            });
        }

        while let Some((file_key, rule_key)) = new_violations.next() {
            diff.push(SuppressionDiff::Appeared { file: file_key.clone(), rule: rule_key.clone() });
        }

        while let Some((file_key, rule_key)) = existing_violations.next() {
            let file_diagnostic = self
                .suppressions_by_file
                .suppressions
                .get(file_key)
                .unwrap()
                .get(rule_key)
                .unwrap();
            let runtime_diagnostic =
                runtime_suppressions.get(file_key).unwrap().get(rule_key).unwrap();

            if file_diagnostic.count > runtime_diagnostic.count {
                diff.push(SuppressionDiff::Decreased {
                    file: file_key.clone(),
                    rule: rule_key.clone(),
                    from: file_diagnostic.count,
                    to: runtime_diagnostic.count,
                });
            } else if file_diagnostic.count < runtime_diagnostic.count {
                diff.push(SuppressionDiff::Increased {
                    file: file_key.clone(),
                    rule: rule_key.clone(),
                    from: file_diagnostic.count,
                    to: runtime_diagnostic.count,
                });
            }
        }

        diff
    }
}
