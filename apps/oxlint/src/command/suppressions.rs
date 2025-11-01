use std::{collections::HashSet, fs, path::Path, rc::Rc};

use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_span::Span;
// serde types are now imported from oxc_linter

use crate::result::CliRunResult;
pub use oxc_linter::{
    BulkSuppressions, ESLintBulkSuppressions, ESLintBulkSuppressionsFile, ESLintRuleSuppression,
    SuppressionEntry, SuppressionsFile, load_eslint_suppressions_from_file,
    load_suppressions_from_file,
};

/// Default name for the suppressions file
pub const DEFAULT_SUPPRESSIONS_FILE: &str = "oxlint-suppressions.json";

/// Information about a linting violation to be suppressed
#[derive(Debug, Clone)]
pub struct ViolationInfo {
    pub file_path: String,
    pub rule_name: String,
    pub plugin_name: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub message: String,
    pub severity: String,
}

// SuppressionEntry and SuppressionsFile are now imported from oxc_linter

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
#[derive(Debug)]
pub struct SuppressionMatcher {
    suppressions: Rc<SuppressionsFile>,
    tracker: Rc<std::cell::RefCell<SuppressionTracker>>,
}

impl SuppressionMatcher {
    pub fn new(suppressions: SuppressionsFile) -> Self {
        Self {
            suppressions: Rc::new(suppressions),
            tracker: Rc::new(std::cell::RefCell::new(SuppressionTracker::new())),
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
            let source_file = self
                .suppressions
                .suppressions
                .first()
                .map(|_| {
                    // TODO: Get actual source file for line/column conversion
                    // For now, we'll do a simple span-based check
                    true
                })
                .unwrap_or(false);

            if source_file {
                // Mark this suppression as used
                self.tracker.borrow_mut().mark_used(index);
                return true;
            }
        }

        false
    }

    /// Get the tracker for checking unused suppressions
    pub fn tracker(&self) -> Rc<std::cell::RefCell<SuppressionTracker>> {
        self.tracker.clone()
    }

    /// Get the suppressions
    pub fn suppressions(&self) -> &SuppressionsFile {
        &self.suppressions
    }
}

/// Load suppressions from a file - wrapper around oxc_linter version with error conversion
pub fn load_suppressions_file_cli(path: &Path) -> Result<SuppressionsFile, CliRunResult> {
    load_suppressions_from_file(path).map_err(|_| CliRunResult::SuppressionFileInvalid)
}

/// Write suppressions to a file
pub fn write_suppressions_to_file(
    suppressions: &SuppressionsFile,
    path: &Path,
) -> Result<(), CliRunResult> {
    let content = serde_json::to_string_pretty(suppressions)
        .map_err(|err| {
            eprintln!("Failed to serialize suppressions to JSON: {}", err);
            CliRunResult::SuppressionGenerationFailed
        })?;

    fs::write(path, content).map_err(|err| {
        eprintln!("Failed to write suppressions file to {}: {}", path.display(), err);
        CliRunResult::SuppressionGenerationFailed
    })
}

/// Merge new suppressions with existing ones
pub fn merge_suppressions(
    existing: &SuppressionsFile,
    new_suppressions: Vec<SuppressionEntry>,
    replace_all: bool,
) -> SuppressionsFile {
    if replace_all {
        SuppressionsFile { suppressions: new_suppressions }
    } else {
        let mut merged = existing.clone();
        merged.suppressions.extend(new_suppressions);
        merged
    }
}

/// Remove unused suppressions from a suppressions file
pub fn prune_suppressions(
    suppressions: &SuppressionsFile,
    unused_indices: &[usize],
) -> SuppressionsFile {
    let mut pruned = suppressions.clone();

    // Remove in reverse order to maintain correct indices
    let mut sorted_indices = unused_indices.to_vec();
    sorted_indices.sort_by(|a, b| b.cmp(a));

    for &index in &sorted_indices {
        if index < pruned.suppressions.len() {
            pruned.suppressions.remove(index);
        }
    }

    pruned
}

/// Create diagnostics for unused suppressions
pub fn create_unused_suppressions_diagnostics(
    suppressions: &SuppressionsFile,
    unused_indices: &[usize],
    severity: Severity,
) -> Vec<OxcDiagnostic> {
    unused_indices
        .iter()
        .filter_map(|&index| {
            suppressions.suppressions.get(index).map(|suppression| {
                let message = format!(
                    "Unused suppression for rule '{}' (no violations found)",
                    suppression.rules.join(", ")
                );

                // Create a span for the suppression entry
                // Note: This is a placeholder - in a real implementation,
                // we'd need to track the original source locations
                let span = Span::new(0, 1);

                OxcDiagnostic::error(message).with_label(span).with_severity(severity)
            })
        })
        .collect()
}

/// Validate that a suppressions file is well-formed
pub fn validate_suppression_file(suppressions: &SuppressionsFile) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for (index, suppression) in suppressions.suppressions.iter().enumerate() {
        if suppression.files.is_empty() {
            errors.push(format!("Suppression entry {index} has no files specified"));
        }

        if suppression.rules.is_empty() {
            errors.push(format!("Suppression entry {index} has no rules specified"));
        }

        if suppression.reason.trim().is_empty() {
            errors.push(format!("Suppression entry {index} has no reason specified"));
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_nonexistent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.json");

        let result = load_suppressions_from_file(&file_path).unwrap();
        assert!(result.suppressions.is_empty());
    }

    #[test]
    fn test_write_and_load_suppressions() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test-suppressions.json");

        let suppressions = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["test.js".to_string()],
                rules: vec!["no-console".to_string()],
                line: 1,
                column: 0,
                end_line: Some(1),
                end_column: Some(10),
                reason: "Test suppression".to_string(),
            }],
        };

        write_suppressions_to_file(&suppressions, &file_path).unwrap();
        let loaded = load_suppressions_from_file(&file_path).unwrap();

        assert_eq!(loaded.suppressions.len(), 1);
        assert_eq!(loaded.suppressions[0].files, vec!["test.js"]);
        assert_eq!(loaded.suppressions[0].rules, vec!["no-console"]);
    }

    #[test]
    fn test_merge_suppressions_replace() {
        let existing = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["old.js".to_string()],
                rules: vec!["no-debugger".to_string()],
                line: 1,
                column: 0,
                end_line: None,
                end_column: None,
                reason: "Old suppression".to_string(),
            }],
        };

        let new_suppressions = vec![SuppressionEntry {
            files: vec!["new.js".to_string()],
            rules: vec!["no-console".to_string()],
            line: 2,
            column: 0,
            end_line: None,
            end_column: None,
            reason: "New suppression".to_string(),
        }];

        let merged = merge_suppressions(&existing, new_suppressions, true);

        assert_eq!(merged.suppressions.len(), 1);
        assert_eq!(merged.suppressions[0].files, vec!["new.js"]);
    }

    #[test]
    fn test_merge_suppressions_extend() {
        let existing = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["old.js".to_string()],
                rules: vec!["no-debugger".to_string()],
                line: 1,
                column: 0,
                end_line: None,
                end_column: None,
                reason: "Old suppression".to_string(),
            }],
        };

        let new_suppressions = vec![SuppressionEntry {
            files: vec!["new.js".to_string()],
            rules: vec!["no-console".to_string()],
            line: 2,
            column: 0,
            end_line: None,
            end_column: None,
            reason: "New suppression".to_string(),
        }];

        let merged = merge_suppressions(&existing, new_suppressions, false);

        assert_eq!(merged.suppressions.len(), 2);
        assert_eq!(merged.suppressions[0].files, vec!["old.js"]);
        assert_eq!(merged.suppressions[1].files, vec!["new.js"]);
    }

    #[test]
    fn test_prune_suppressions() {
        let suppressions = SuppressionsFile {
            suppressions: vec![
                SuppressionEntry {
                    files: vec!["file1.js".to_string()],
                    rules: vec!["rule1".to_string()],
                    line: 1,
                    column: 0,
                    end_line: None,
                    end_column: None,
                    reason: "Suppression 1".to_string(),
                },
                SuppressionEntry {
                    files: vec!["file2.js".to_string()],
                    rules: vec!["rule2".to_string()],
                    line: 2,
                    column: 0,
                    end_line: None,
                    end_column: None,
                    reason: "Suppression 2".to_string(),
                },
                SuppressionEntry {
                    files: vec!["file3.js".to_string()],
                    rules: vec!["rule3".to_string()],
                    line: 3,
                    column: 0,
                    end_line: None,
                    end_column: None,
                    reason: "Suppression 3".to_string(),
                },
            ],
        };

        // Remove indices 0 and 2
        let pruned = prune_suppressions(&suppressions, &[0, 2]);

        assert_eq!(pruned.suppressions.len(), 1);
        assert_eq!(pruned.suppressions[0].files, vec!["file2.js"]);
    }
}
