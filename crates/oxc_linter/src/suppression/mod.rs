use std::{ffi::OsStr, path::Path};

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
        if !path.exists() {
            return Ok(Self::default());
        }

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

    pub fn save(&self) -> Result<(), OxcDiagnostic> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct SuppressionManager {
    pub suppressions_by_file: SuppressionTracking,
    pub runtime_suppressions: SuppressionTracking,
}

impl Default for SuppressionManager {
    fn default() -> Self {
        Self {
            suppressions_by_file: SuppressionTracking::default(),
            runtime_suppressions: SuppressionTracking::default(),
        }
    }
}

impl SuppressionManager {
    // reads the `oxlint-suppressions.json` from the disk
    pub fn load(path: &Path) -> Result<Self, OxcDiagnostic> {
        let suppression_file = SuppressionTracking::from_file(path)?;

        Ok(Self {
            suppressions_by_file: suppression_file,
            runtime_suppressions: SuppressionTracking::default(),
        })
    }

    pub fn get_suppression_per_file(
        &self,
        path: &Path,
    ) -> Option<&FxHashMap<RuleName, DiagnosticCounts>> {
        let filename = Filename::new(path);

        self.suppressions_by_file.suppressions.get(&filename)
    }

    fn is_suppressed(&self, filename: &Filename, rule_name: &RuleName) -> bool {
        let Some(runtime_suppressions) = self.runtime_suppressions.suppressions.get(filename)
        else {
            return false;
        };

        let Some(runtime_violations) = runtime_suppressions.get(rule_name) else {
            return false;
        };

        let Some(file_suppressions) = self.suppressions_by_file.suppressions.get(filename) else {
            return false;
        };

        let Some(file_violations) = file_suppressions.get(rule_name) else {
            return false;
        };

        if file_violations.count >= runtime_violations.count {
            return false;
        }

        true
    }

    // Adds a suppression for the given file, plugin, and rule.
    pub fn add_violation(&mut self, file_path: &Path, plugin_name: &str, rule_name: &str) {
        let filename = Filename::new(file_path);
        let rule = RuleName::new(plugin_name, rule_name);

        if let Some(violation_count) = self
            .runtime_suppressions
            .suppressions
            .get_mut(&filename)
            .and_then(|rules| rules.get_mut(&rule))
        {
            violation_count.count += 1;
        } else {
            let violation_counts = {
                let mut init_map = FxHashMap::default();
                init_map.insert(rule, DiagnosticCounts { count: 1 });
                init_map
            };

            self.runtime_suppressions.suppressions.insert(filename, violation_counts);
        }
    }
}
