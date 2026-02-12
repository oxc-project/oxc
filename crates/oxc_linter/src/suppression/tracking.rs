use std::{ffi::OsStr, fs, path::Path};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashMap;
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

type FileSuppressionsMap = FxHashMap<RuleName, DiagnosticCounts>;
type AllSuppressionsMap = FxHashMap<Filename, FileSuppressionsMap>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SuppressionTracking {
    version: String,
    suppressions: AllSuppressionsMap,
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
                Some("json") => error.to_string(),
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

    pub fn suppressions(&self) -> &AllSuppressionsMap {
        &self.suppressions
    }

    pub fn update(&mut self, diff: SuppressionDiff) {
        match diff {
            SuppressionDiff::Increased { file, rule, from: _, to }
            | SuppressionDiff::Decreased { file, rule, from: _, to } => {
                self.suppressions.get_mut(&file).unwrap().get_mut(&rule).unwrap().count = to;
            }
            SuppressionDiff::PrunedRuled { file, rule } => {
                let file_map = &self.suppressions[&file];

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
                    let mut file_diagnostic: FileSuppressionsMap = FxHashMap::default();
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

#[derive(Debug, Clone)]
pub enum SuppressionDiff {
    Increased { file: Filename, rule: RuleName, from: usize, to: usize },
    Decreased { file: Filename, rule: RuleName, from: usize, to: usize },
    PrunedRuled { file: Filename, rule: RuleName },
    Appeared { file: Filename, rule: RuleName, count: usize },
}

impl From<SuppressionDiff> for OxcDiagnostic {
    fn from(val: SuppressionDiff) -> Self {
        let help = match &val {
            SuppressionDiff::Increased { file: _, rule: _, from: _, to: _ }
            | SuppressionDiff::Appeared { file: _, rule: _, count: _ } => {
                "Update `oxlint-suppressions.json` file running `oxlint --suppress--all`"
            }
            SuppressionDiff::Decreased { file: _, rule: _, from: _, to: _ }
            | SuppressionDiff::PrunedRuled { file: _, rule: _ } => {
                "Update `oxlint-suppressions.json` file running `oxlint --prune-suppressions`"
            }
        };

        let message = match val {
            SuppressionDiff::Increased { file, rule, from, to } => {
                format!("The number of '{rule}' errors in {file} increased from {from} to {to}.")
            }
            SuppressionDiff::Decreased { file, rule, from, to } => {
                format!("The number of '{rule}' errors in {file} decreased from {from} to {to}.")
            }
            SuppressionDiff::PrunedRuled { file, rule } => {
                format!("The '{rule}' rule has been pruned from {file}.")
            }
            SuppressionDiff::Appeared { file, rule, count } => {
                let s = if count == 1 { "" } else { "s" };
                format!("{count} new '{rule}' error{s} appeared in {file}.")
            }
        };

        OxcDiagnostic::error(message).with_help(help)
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
    suppression_data: Option<&'a FileSuppressionsMap>,
}

impl Default for SuppressionFile<'_> {
    fn default() -> Self {
        Self { state: SuppressionFileState::Ignored, suppression_data: None }
    }
}

impl<'a> SuppressionFile<'a> {
    pub fn new<'b>(
        file_exists: bool,
        suppress_all: bool,
        suppression_data: Option<&'b FileSuppressionsMap>,
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

    pub fn suppression_data(&self) -> Option<&'a FileSuppressionsMap> {
        self.suppression_data
    }
}
