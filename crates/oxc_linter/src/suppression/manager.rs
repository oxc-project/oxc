use std::collections::HashMap;
use std::fs;
use std::path::Path;

use oxc_diagnostics::OxcDiagnostic;

use super::{SuppressionEntry, SuppressionFile};

#[derive(Debug, Clone)]
pub struct SuppressionManager {
    suppressions_by_file: HashMap<String, HashMap<String, SuppressionEntry>>,
    counts: HashMap<String, HashMap<String, u32>>,
}

impl Default for SuppressionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SuppressionManager {
    pub fn new() -> Self {
        Self { suppressions_by_file: HashMap::new(), counts: HashMap::new() }
    }

    pub fn load(path: &Path) -> Result<Self, OxcDiagnostic> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to read suppression file: {err}"))
        })?;

        let suppression_file: SuppressionFile = serde_json::from_str(&content).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse suppression file: {err}"))
        })?;

        Ok(Self { suppressions_by_file: suppression_file.suppressions, counts: HashMap::new() })
    }

    pub fn save(&self, path: &Path) -> Result<(), OxcDiagnostic> {
        let suppression_file = SuppressionFile {
            version: "0.1.0".to_string(),
            suppressions: self.suppressions_by_file.clone(),
        };

        let content = serde_json::to_string_pretty(&suppression_file).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to serialize suppression file: {err}"))
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                OxcDiagnostic::error(format!(
                    "Failed to create directory for suppression file: {err}"
                ))
            })?;
        }

        fs::write(path, content).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to write suppression file: {err}"))
        })?;

        Ok(())
    }

    /// Normalize file path to use forward slashes and be relative to project root
    fn normalize_file_path(&self, file_path: &Path) -> String {
        file_path.to_string_lossy().replace('\\', "/")
    }

    /// Create rule key from plugin prefix and rule name
    fn create_rule_key(&self, plugin_prefix: &str, rule_name: &str) -> String {
        format!("{}/{}", plugin_prefix, rule_name)
    }

    pub fn is_suppressed(&self, file_path: &Path, plugin_prefix: &str, rule_name: &str) -> bool {
        let rule_key = self.create_rule_key(plugin_prefix, rule_name);
        let file_key = self.normalize_file_path(file_path);

        if let Some(file_suppressions) = self.suppressions_by_file.get(&file_key) {
            if let Some(entry) = file_suppressions.get(&rule_key) {
                // Check if we've already seen this many violations for this rule in this file
                let current_count =
                    self.counts.get(&file_key).and_then(|rules| rules.get(&rule_key)).unwrap_or(&0);
                return *current_count < entry.count;
            }
        }
        false
    }

    pub fn record_violation(&mut self, file_path: &Path, plugin_prefix: &str, rule_name: &str) {
        let rule_key = self.create_rule_key(plugin_prefix, rule_name);
        let file_key = self.normalize_file_path(file_path);

        self.counts
            .entry(file_key)
            .or_default()
            .entry(rule_key)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn add_suppression(
        &mut self,
        file_path: &Path,
        plugin_prefix: &str,
        rule_name: &str,
        count: u32,
    ) {
        let rule_key = self.create_rule_key(plugin_prefix, rule_name);
        let file_key = self.normalize_file_path(file_path);

        self.suppressions_by_file
            .entry(file_key)
            .or_default()
            .insert(rule_key, SuppressionEntry { count });
    }

    pub fn get_suppression_count(
        &self,
        file_path: &Path,
        plugin_prefix: &str,
        rule_name: &str,
    ) -> Option<u32> {
        let rule_key = self.create_rule_key(plugin_prefix, rule_name);
        let file_key = self.normalize_file_path(file_path);

        self.suppressions_by_file.get(&file_key)?.get(&rule_key).map(|entry| entry.count)
    }

    pub fn reset_counts(&mut self) {
        self.counts.clear();
    }

    pub fn prune_unused(&mut self) {
        // Remove suppressions that have no actual violations found
        self.suppressions_by_file.retain(|file_path, rules| {
            rules.retain(|rule_key, suppression_entry| {
                let actual_count =
                    self.counts.get(file_path).and_then(|rules| rules.get(rule_key)).unwrap_or(&0);
                *actual_count > 0 && *actual_count <= suppression_entry.count
            });
            !rules.is_empty()
        });
    }

    pub fn get_all_files(&self) -> Vec<String> {
        self.suppressions_by_file.keys().cloned().collect()
    }

    pub fn get_suppressions_for_file(
        &self,
        file_path: &Path,
    ) -> Option<&HashMap<String, SuppressionEntry>> {
        let file_key = self.normalize_file_path(file_path);
        self.suppressions_by_file.get(&file_key)
    }
}
