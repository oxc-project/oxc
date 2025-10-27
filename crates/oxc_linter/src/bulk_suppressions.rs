use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};

use oxc_span::Span;
use serde::{Deserialize, Serialize};

/// Represents a single suppression entry from the suppressions file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionEntry {
    pub files: Vec<String>,
    pub rules: Vec<String>,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub reason: String,
}

/// Root structure of the suppressions file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionsFile {
    pub suppressions: Vec<SuppressionEntry>,
}

impl Default for SuppressionsFile {
    fn default() -> Self {
        Self {
            suppressions: Vec::new(),
        }
    }
}

/// ESLint-style bulk suppressions format
/// Maps file paths to rules to suppression counts
/// Example: { "src/App.tsx": { "no-console": { "count": 2 }, "prefer-const": { "count": 1 } } }
pub type ESLintBulkSuppressionsData = HashMap<String, HashMap<String, ESLintRuleSuppression>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESLintRuleSuppression {
    pub count: u32,
}

/// ESLint-style bulk suppressions file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ESLintBulkSuppressionsFile {
    pub suppressions: ESLintBulkSuppressionsData,
}

impl Default for ESLintBulkSuppressionsFile {
    fn default() -> Self {
        Self {
            suppressions: HashMap::new(),
        }
    }
}

/// Tracks suppressions that have been used during linting
#[derive(Debug, Default)]
pub struct SuppressionTracker {
    /// Set of suppression indices that have been used
    used_indices: HashSet<usize>,
}

impl SuppressionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a suppression as used
    pub fn mark_used(&mut self, index: usize) {
        self.used_indices.insert(index);
    }

    /// Check if a suppression has been used
    pub fn is_used(&self, index: usize) -> bool {
        self.used_indices.contains(&index)
    }

    /// Get all unused suppression indices
    pub fn get_unused_indices(&self, total_suppressions: usize) -> Vec<usize> {
        (0..total_suppressions).filter(|i| !self.used_indices.contains(i)).collect()
    }
}

/// Efficient matcher for checking if diagnostics should be suppressed
#[derive(Debug, Clone)]
pub struct BulkSuppressions {
    suppressions: Arc<SuppressionsFile>,
    tracker: Arc<Mutex<SuppressionTracker>>,
}

impl BulkSuppressions {
    pub fn new(suppressions: SuppressionsFile) -> Self {
        Self {
            suppressions: Arc::new(suppressions),
            tracker: Arc::new(Mutex::new(SuppressionTracker::new())),
        }
    }

    /// Check if a diagnostic should be suppressed
    pub fn matches(
        &self,
        plugin_name: &str,
        plugin_prefix: &str,
        rule_name: &str,
        span: Span,
        file_path: &Path,
    ) -> bool {
        let file_path_str = file_path.to_string_lossy();

        for (index, suppression) in self.suppressions.suppressions.iter().enumerate() {
            // Check if file matches
            let file_matches = suppression.files.iter().any(|pattern| {
                // For now, do exact string matching
                // TODO: Add glob pattern matching support
                file_path_str.ends_with(pattern)
                    || pattern == "*"
                    || file_path_str.contains(pattern)
            });

            if !file_matches {
                continue;
            }

            // Check if rule matches
            let rule_matches = suppression.rules.iter().any(|rule| {
                rule == rule_name
                    || rule == &format!("{plugin_name}/{rule_name}")
                    || rule == &format!("{plugin_prefix}/{rule_name}")
            });

            if !rule_matches {
                continue;
            }

            // Check if the span position matches
            // TODO: Improve this with proper line/column to span conversion
            // For now, we'll mark any rule+file match as suppressed
            if self.position_matches(suppression, span) {
                // Mark this suppression as used (ignore lock errors for now)
                if let Ok(mut tracker) = self.tracker.lock() {
                    tracker.mark_used(index);
                }
                return true;
            }
        }

        false
    }

    /// Check if the diagnostic position matches the suppression position
    fn position_matches(&self, suppression: &SuppressionEntry, span: Span) -> bool {
        // This is a simplified implementation that matches any rule+file combination
        // In a complete implementation with source text access, we would:
        // 1. Convert suppression line/column to actual span offsets
        // 2. Check if diagnostic span overlaps with or is contained by suppression span
        // 3. Handle end_line/end_column for range-based suppressions

        // For now, we match based on rule+file combination
        // This provides the core functionality while being simple to implement

        // If the suppression has line/column info, we could do basic span checking
        // but without access to source text, we can't convert line/column to byte offsets

        // Future enhancement: pass source text or line mapping to enable proper span matching
        let _ = (suppression, span); // Use parameters to avoid warnings
        true
    }

    /// Get the tracker for checking unused suppressions
    pub fn tracker(&self) -> Arc<Mutex<SuppressionTracker>> {
        self.tracker.clone()
    }

    /// Get the suppressions
    pub fn suppressions(&self) -> &SuppressionsFile {
        &self.suppressions
    }

    /// Get unused suppressions after linting is complete
    pub fn get_unused_suppressions(&self) -> Vec<usize> {
        if let Ok(tracker) = self.tracker.lock() {
            tracker.get_unused_indices(self.suppressions.suppressions.len())
        } else {
            // If we can't lock the tracker, assume no unused suppressions
            Vec::new()
        }
    }
}

/// ESLint-style bulk suppressions matcher
#[derive(Debug, Clone)]
pub struct ESLintBulkSuppressions {
    suppressions: Arc<ESLintBulkSuppressionsFile>,
    usage_tracker: Arc<Mutex<HashMap<String, HashMap<String, u32>>>>, // file -> rule -> used_count
}

impl ESLintBulkSuppressions {
    pub fn new(suppressions: ESLintBulkSuppressionsFile) -> Self {
        Self {
            suppressions: Arc::new(suppressions),
            usage_tracker: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a diagnostic should be suppressed
    pub fn matches(
        &self,
        plugin_name: &str,
        plugin_prefix: &str,
        rule_name: &str,
        _span: Span,
        file_path: &Path,
    ) -> bool {
        let file_path_str = file_path.to_string_lossy();

        // Try to find the file in suppressions (exact match or relative path match)
        let file_key = self.find_file_key(&file_path_str);

        if let Some(file_key) = file_key {
            if let Some(rules) = self.suppressions.suppressions.get(file_key) {
                // Try different rule name formats
                let possible_rule_names = [
                    rule_name,
                    &format!("{plugin_name}/{rule_name}"),
                    &format!("{plugin_prefix}/{rule_name}"),
                ];

                for &rule_key in &possible_rule_names {
                    if let Some(rule_suppression) = rules.get(rule_key) {
                        // Check if we still have suppressions left for this rule
                        if let Ok(mut tracker) = self.usage_tracker.lock() {
                            let file_tracker = tracker.entry(file_key.clone()).or_insert_with(HashMap::new);
                            let used_count = file_tracker.entry(rule_key.to_string()).or_insert(0);

                            if *used_count < rule_suppression.count {
                                *used_count += 1;
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Find the file key in suppressions data, handling both exact matches and relative paths
    fn find_file_key(&self, file_path: &str) -> Option<&String> {
        // First try exact match
        if self.suppressions.suppressions.contains_key(file_path) {
            return self.suppressions.suppressions.keys().find(|k| *k == file_path);
        }

        // Try to find by matching the end of the path (for relative paths)
        self.suppressions.suppressions.keys().find(|key| {
            file_path.ends_with(key.as_str()) || key.ends_with(file_path)
        })
    }

    /// Get unused suppressions after linting is complete
    pub fn get_unused_suppressions(&self) -> Vec<(String, String, u32)> {
        let mut unused = Vec::new();

        if let Ok(tracker) = self.usage_tracker.lock() {
            for (file_path, file_rules) in &self.suppressions.suppressions {
                for (rule_name, rule_suppression) in file_rules {
                    let used_count = tracker
                        .get(file_path)
                        .and_then(|rules| rules.get(rule_name))
                        .copied()
                        .unwrap_or(0);

                    if used_count < rule_suppression.count {
                        unused.push((file_path.clone(), rule_name.clone(), rule_suppression.count - used_count));
                    }
                }
            }
        }

        unused
    }

    /// Get the suppressions data
    pub fn suppressions(&self) -> &ESLintBulkSuppressionsFile {
        &self.suppressions
    }
}

/// Load suppressions from a JSON file
pub fn load_suppressions_from_file(path: &Path) -> Result<SuppressionsFile, std::io::Error> {
    if !path.exists() {
        return Ok(SuppressionsFile::default());
    }

    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
    })
}

/// Load ESLint-style bulk suppressions from a JSON file
pub fn load_eslint_suppressions_from_file(path: &Path) -> Result<ESLintBulkSuppressionsFile, std::io::Error> {
    if !path.exists() {
        return Ok(ESLintBulkSuppressionsFile::default());
    }

    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_suppressions_basic_functionality() {
        let suppressions_file = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["test.js".to_string()],
                rules: vec!["no-console".to_string()],
                line: 10,
                column: 5,
                end_line: Some(10),
                end_column: Some(16),
                reason: "Test suppression".to_string(),
            }],
        };

        let bulk_suppressions = BulkSuppressions::new(suppressions_file);
        let file_path = Path::new("src/test.js");
        let span = Span::new(100, 111); // Dummy span

        // Test matching
        let matches = bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path);
        assert!(matches);

        // Check that suppression was marked as used
        let unused = bulk_suppressions.get_unused_suppressions();
        assert!(unused.is_empty());
    }

    #[test]
    fn test_load_suppressions_from_nonexistent_file() {
        let file_path = Path::new("/nonexistent/path/suppressions.json");

        let result = load_suppressions_from_file(&file_path).unwrap();
        assert!(result.suppressions.is_empty());
    }

    #[test]
    fn test_suppression_tracker() {
        let mut tracker = SuppressionTracker::new();

        // Initially no suppressions are used
        assert!(!tracker.is_used(0));
        assert!(!tracker.is_used(1));

        // Mark some suppressions as used
        tracker.mark_used(0);
        tracker.mark_used(2);

        assert!(tracker.is_used(0));
        assert!(!tracker.is_used(1));
        assert!(tracker.is_used(2));

        // Get unused indices
        let unused = tracker.get_unused_indices(5);
        assert_eq!(unused, vec![1, 3, 4]);
    }

    #[test]
    fn test_eslint_bulk_suppressions_basic_functionality() {
        let mut suppressions_data = HashMap::new();
        let mut file_rules = HashMap::new();
        file_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 2 });
        file_rules.insert("prefer-const".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data.insert("src/test.js".to_string(), file_rules);

        let suppressions_file = ESLintBulkSuppressionsFile {
            suppressions: suppressions_data,
        };

        let bulk_suppressions = ESLintBulkSuppressions::new(suppressions_file);
        let file_path = Path::new("src/test.js");
        let span = Span::new(100, 111);

        // Test first suppression for no-console
        assert!(bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path));

        // Test second suppression for no-console
        assert!(bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path));

        // Test third suppression for no-console should fail (only 2 allowed)
        assert!(!bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path));

        // Test single suppression for prefer-const
        assert!(bulk_suppressions.matches("eslint", "eslint", "prefer-const", span, file_path));

        // Test second suppression for prefer-const should fail (only 1 allowed)
        assert!(!bulk_suppressions.matches("eslint", "eslint", "prefer-const", span, file_path));
    }

    #[test]
    fn test_eslint_bulk_suppressions_file_matching() {
        let mut suppressions_data = HashMap::new();
        let mut file_rules = HashMap::new();
        file_rules.insert("no-unused-vars".to_string(), ESLintRuleSuppression { count: 5 });
        suppressions_data.insert("App.tsx".to_string(), file_rules);

        let suppressions_file = ESLintBulkSuppressionsFile {
            suppressions: suppressions_data,
        };

        let bulk_suppressions = ESLintBulkSuppressions::new(suppressions_file);
        let span = Span::new(100, 111);

        // Test exact file match
        let file_path = Path::new("App.tsx");
        assert!(bulk_suppressions.matches("typescript", "@typescript-eslint", "no-unused-vars", span, file_path));

        // Test relative path match
        let file_path = Path::new("src/App.tsx");
        assert!(bulk_suppressions.matches("typescript", "@typescript-eslint", "no-unused-vars", span, file_path));

        // Test full path match
        let file_path = Path::new("/Users/user/project/src/App.tsx");
        assert!(bulk_suppressions.matches("typescript", "@typescript-eslint", "no-unused-vars", span, file_path));

        // Test non-matching file
        let file_path = Path::new("Different.tsx");
        assert!(!bulk_suppressions.matches("typescript", "@typescript-eslint", "no-unused-vars", span, file_path));
    }

    #[test]
    fn test_eslint_bulk_suppressions_rule_name_formats() {
        // Test bare rule name format
        let mut suppressions_data1 = HashMap::new();
        let mut file_rules1 = HashMap::new();
        file_rules1.insert("no-unused-vars".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data1.insert("test.ts".to_string(), file_rules1);

        let bulk_suppressions1 = ESLintBulkSuppressions::new(ESLintBulkSuppressionsFile {
            suppressions: suppressions_data1,
        });
        let file_path = Path::new("test.ts");
        let span = Span::new(100, 111);

        // Test matching with bare rule name
        assert!(bulk_suppressions1.matches("typescript", "@typescript-eslint", "no-unused-vars", span, file_path));

        // Test @typescript-eslint/ prefix format
        let mut suppressions_data2 = HashMap::new();
        let mut file_rules2 = HashMap::new();
        file_rules2.insert("@typescript-eslint/no-unused-vars".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data2.insert("test.ts".to_string(), file_rules2);

        let bulk_suppressions2 = ESLintBulkSuppressions::new(ESLintBulkSuppressionsFile {
            suppressions: suppressions_data2,
        });

        // Test matching with plugin prefix
        assert!(bulk_suppressions2.matches("typescript", "@typescript-eslint", "no-unused-vars", span, file_path));

        // Test typescript/ prefix format
        let mut suppressions_data3 = HashMap::new();
        let mut file_rules3 = HashMap::new();
        file_rules3.insert("typescript/no-unused-vars".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data3.insert("test.ts".to_string(), file_rules3);

        let bulk_suppressions3 = ESLintBulkSuppressions::new(ESLintBulkSuppressionsFile {
            suppressions: suppressions_data3,
        });

        // Test matching with different plugin name
        assert!(bulk_suppressions3.matches("typescript", "typescript", "no-unused-vars", span, file_path));
    }

    #[test]
    fn test_eslint_bulk_suppressions_unused_tracking() {
        let mut suppressions_data = HashMap::new();
        let mut file_rules = HashMap::new();
        file_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 3 });
        file_rules.insert("prefer-const".to_string(), ESLintRuleSuppression { count: 2 });
        suppressions_data.insert("test.js".to_string(), file_rules);

        let suppressions_file = ESLintBulkSuppressionsFile {
            suppressions: suppressions_data,
        };

        let bulk_suppressions = ESLintBulkSuppressions::new(suppressions_file);
        let file_path = Path::new("test.js");
        let span = Span::new(100, 111);

        // Use only one no-console suppression
        bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path);

        // Don't use any prefer-const suppressions

        let unused = bulk_suppressions.get_unused_suppressions();

        // Should have 2 unused no-console and 2 unused prefer-const
        assert_eq!(unused.len(), 2);

        let mut unused_by_rule: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for (_, rule, count) in &unused {
            unused_by_rule.insert(rule.as_str(), *count);
        }

        assert_eq!(unused_by_rule.get("no-console"), Some(&2));
        assert_eq!(unused_by_rule.get("prefer-const"), Some(&2));
    }

    #[test]
    fn test_load_eslint_suppressions_from_nonexistent_file() {
        let file_path = Path::new("/nonexistent/path/eslint-suppressions.json");

        let result = load_eslint_suppressions_from_file(&file_path).unwrap();
        assert!(result.suppressions.is_empty());
    }

    #[test]
    fn test_eslint_suppressions_serialization() {
        // Create test suppressions
        let mut suppressions_data = HashMap::new();
        let mut file_rules = HashMap::new();
        file_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 2 });
        file_rules.insert("@typescript-eslint/no-unused-vars".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data.insert("src/App.tsx".to_string(), file_rules);

        let suppressions = ESLintBulkSuppressionsFile {
            suppressions: suppressions_data,
        };

        // Test serialization
        let json_content = serde_json::to_string_pretty(&suppressions).unwrap();
        assert!(json_content.contains("src/App.tsx"));
        assert!(json_content.contains("no-console"));
        assert!(json_content.contains("@typescript-eslint/no-unused-vars"));

        // Test deserialization
        let parsed: ESLintBulkSuppressionsFile = serde_json::from_str(&json_content).unwrap();
        assert_eq!(parsed.suppressions.len(), 1);
        assert!(parsed.suppressions.contains_key("src/App.tsx"));

        let file_rules = parsed.suppressions.get("src/App.tsx").unwrap();
        assert_eq!(file_rules.get("no-console").unwrap().count, 2);
        assert_eq!(file_rules.get("@typescript-eslint/no-unused-vars").unwrap().count, 1);
    }
}