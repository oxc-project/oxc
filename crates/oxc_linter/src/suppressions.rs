//! Manages the suppressed violations.
//!
//! This module implements bulk suppressions functionality similar to ESLint's
//! `eslint-suppressions.json` file. It allows teams to gradually adopt stricter
//! linting rules by suppressing existing violations in a separate file.
//!
//! ## File Format
//!
//! The suppressions file is a JSON file with the following structure:
//!
//! ```json
//! {
//!   "src/file1.js": {
//!     "no-undef": { "count": 2 },
//!     "no-unused-vars": { "count": 1 }
//!   },
//!   "src/file2.js": {
//!     "no-console": { "count": 5 }
//!   }
//! }
//! ```
//!
//! File paths are stored relative to the current working directory, always using
//! POSIX-style forward slashes for cross-platform consistency.

use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use oxc_diagnostics::{OxcDiagnostic, Severity};
use serde::{Deserialize, Serialize};

use crate::Message;

/// Default filename for the suppressions file.
pub const DEFAULT_SUPPRESSIONS_FILE: &str = "oxlint-suppressions.json";

/// Represents a count of suppressed violations for a single rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleSuppression {
    pub count: usize,
}

/// Type alias for the suppressions file format.
/// Maps file paths to rule suppressions.
/// `{ "file/path.js": { "rule-name": { "count": 5 } } }`
pub type SuppressedViolations = IndexMap<String, IndexMap<String, RuleSuppression>>;

/// Result of applying suppressions to lint results.
#[derive(Debug)]
pub struct ApplySuppressionsResult {
    /// The filtered messages after applying suppressions.
    pub messages: Vec<Message>,
    /// Suppressions that were not matched against any violations (unused).
    pub unused: SuppressedViolations,
}

/// Manages the suppressed violations.
///
/// This service handles loading, saving, and applying suppressions from a JSON file.
/// It follows ESLint's suppressions file format and behavior.
#[derive(Debug)]
pub struct SuppressionsService {
    /// The location of the suppressions file.
    file_path: PathBuf,
    /// The current working directory.
    cwd: PathBuf,
}

impl SuppressionsService {
    /// Creates a new instance of SuppressionsService.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The location of the suppressions file.
    /// * `cwd` - The current working directory.
    pub fn new(file_path: PathBuf, cwd: PathBuf) -> Self {
        Self { file_path, cwd }
    }

    /// Returns the path to the suppressions file.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Updates the suppressions file based on the current violations and the provided rules.
    ///
    /// If no rules are provided, all violations are suppressed.
    ///
    /// # Arguments
    ///
    /// * `results` - A map of file paths to their lint messages.
    /// * `rules` - Optional list of specific rules to suppress. If `None`, all rules are suppressed.
    ///
    /// # Errors
    ///
    /// Returns an error if the suppressions file cannot be loaded or saved.
    pub fn suppress(
        &self,
        results: &IndexMap<&Path, Vec<Message>>,
        rules: Option<&[String]>,
    ) -> io::Result<()> {
        let mut suppressions = self.load()?;

        for (file_path, messages) in results {
            let relative_file_path = self.get_relative_file_path(file_path);
            let violations_by_rule = Self::count_violations_by_rule(messages);

            for (rule_id, suppression) in violations_by_rule {
                // Skip rules not in the filter list (if provided)
                if let Some(rules) = rules {
                    if !rules.iter().any(|r| r == &rule_id) {
                        continue;
                    }
                }

                suppressions
                    .entry(relative_file_path.clone())
                    .or_default()
                    .insert(rule_id, suppression);
            }
        }

        self.save(&suppressions)
    }

    /// Removes old, unused suppressions for violations that no longer occur.
    ///
    /// # Arguments
    ///
    /// * `results` - A map of file paths to their lint messages.
    ///
    /// # Errors
    ///
    /// Returns an error if the suppressions file cannot be loaded or saved.
    pub fn prune(&self, results: &IndexMap<&Path, Vec<Message>>) -> io::Result<()> {
        let mut suppressions = self.load()?;
        let ApplySuppressionsResult { unused, .. } =
            self.apply_suppressions(results, &suppressions);

        for (file, rules) in &unused {
            if let Some(file_suppressions) = suppressions.get_mut(file) {
                for (rule, unused_suppression) in rules {
                    if let Some(current_suppression) = file_suppressions.get(rule) {
                        let suppressions_count = current_suppression.count;
                        let violations_count = unused_suppression.count;

                        if suppressions_count == violations_count {
                            // Remove unused rules entirely
                            file_suppressions.swap_remove(rule);
                        } else {
                            // Update the count to match the new number of violations
                            if let Some(s) = file_suppressions.get_mut(rule) {
                                s.count = s.count.saturating_sub(violations_count);
                            }
                        }
                    }
                }

                // Cleanup files with no rules
                if file_suppressions.is_empty() {
                    suppressions.swap_remove(file);
                }
            }
        }

        // Remove entries for files that no longer exist
        let files_to_remove: Vec<String> = suppressions
            .keys()
            .filter(|file| {
                let absolute_path = self.cwd.join(file);
                !absolute_path.exists()
            })
            .cloned()
            .collect();

        for file in files_to_remove {
            suppressions.swap_remove(&file);
        }

        self.save(&suppressions)
    }

    /// Checks the provided suppressions against the lint results.
    ///
    /// For each file, counts the number of violations per rule.
    /// For each rule in each file, compares the number of violations against
    /// the counter from the suppressions file.
    ///
    /// If the number of violations is less than or equal to the counter,
    /// messages are suppressed and not included in the result.
    /// Otherwise, all violations are reported as usual.
    ///
    /// # Arguments
    ///
    /// * `results` - A map of file paths to their lint messages.
    /// * `suppressions` - The loaded suppressions data.
    ///
    /// # Returns
    ///
    /// A struct containing the filtered messages and unused suppressions.
    pub fn apply_suppressions(
        &self,
        results: &IndexMap<&Path, Vec<Message>>,
        suppressions: &SuppressedViolations,
    ) -> ApplySuppressionsResult {
        let mut all_messages = Vec::new();
        let mut unused: SuppressedViolations = IndexMap::default();

        for (file_path, messages) in results {
            let relative_file_path = self.get_relative_file_path(file_path);

            let Some(file_suppressions) = suppressions.get(&relative_file_path) else {
                // No suppressions for this file, include all messages
                all_messages.extend(messages.iter().cloned());
                continue;
            };

            let violations_by_rule = Self::count_violations_by_rule(messages);

            // Track which rules were suppressed
            let mut suppressed_rules = std::collections::HashSet::new();

            for (rule_id, violation) in &violations_by_rule {
                if let Some(suppression) = file_suppressions.get(rule_id) {
                    let suppressions_count = suppression.count;
                    let violations_count = violation.count;

                    // Suppress messages if the number of violations is less than or equal to the suppressions count
                    if violations_count <= suppressions_count {
                        suppressed_rules.insert(rule_id.clone());

                        // Track unused suppressions (when violations < suppressions)
                        if violations_count < suppressions_count {
                            unused.entry(relative_file_path.clone()).or_default().insert(
                                rule_id.clone(),
                                RuleSuppression { count: suppressions_count - violations_count },
                            );
                        }
                    }
                }
            }

            // Mark as unused all the suppressions that were not matched against a rule
            for (rule_id, suppression) in file_suppressions {
                if !violations_by_rule.contains_key(rule_id) {
                    unused
                        .entry(relative_file_path.clone())
                        .or_default()
                        .insert(rule_id.clone(), suppression.clone());
                }
            }

            // Filter out suppressed messages
            let filtered_messages: Vec<Message> = messages
                .iter()
                .filter(|msg| match Self::get_rule_id(msg) {
                    Some(ref id) => !suppressed_rules.contains(id),
                    None => true,
                })
                .cloned()
                .collect();

            all_messages.extend(filtered_messages);
        }

        ApplySuppressionsResult { messages: all_messages, unused }
    }

    /// Loads the suppressions file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    /// Returns an empty map if the file does not exist.
    pub fn load(&self) -> io::Result<SuppressedViolations> {
        match fs::read_to_string(&self.file_path) {
            Ok(data) => serde_json::from_str(&data).map_err(|e| {
                io::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Failed to parse suppressions file at {}: {e}",
                        self.file_path.display()
                    ),
                )
            }),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(SuppressedViolations::default()),
            Err(err) => Err(err),
        }
    }

    /// Updates the suppressions file.
    ///
    /// # Arguments
    ///
    /// * `suppressions` - The suppressions to save.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self, suppressions: &SuppressedViolations) -> io::Result<()> {
        let json = serde_json::to_string_pretty(suppressions)?;
        fs::write(&self.file_path, json)
    }

    /// Counts the violations by rule, ignoring warnings.
    ///
    /// Only counts messages with severity `Error` and a rule ID.
    ///
    /// # Arguments
    ///
    /// * `messages` - The messages to count.
    ///
    /// # Returns
    ///
    /// A map of rule IDs to their violation counts.
    pub fn count_violations_by_rule(messages: &[Message]) -> IndexMap<String, RuleSuppression> {
        let mut counts: IndexMap<String, RuleSuppression> = IndexMap::default();

        for message in messages {
            // Only count errors, not warnings (ESLint uses severity === 2)
            if message.error.severity != Severity::Error {
                continue;
            }

            if let Some(rule_id) = Self::get_rule_id(message) {
                counts.entry(rule_id.to_string()).or_insert(RuleSuppression { count: 0 }).count +=
                    1;
            }
        }

        counts
    }

    /// Returns the relative path of a file to the current working directory.
    ///
    /// Always in POSIX format for consistency and interoperability.
    fn get_relative_file_path(&self, file_path: &Path) -> String {
        let relative = file_path.strip_prefix(&self.cwd).unwrap_or(file_path);

        // Convert to POSIX format (forward slashes)
        relative.to_string_lossy().replace('\\', "/")
    }

    /// Extracts the rule ID from a message.
    ///
    /// Returns the rule ID in the format "scope(number)" (e.g., "eslint(no-console)")
    /// or just "number" if no scope is present.
    fn get_rule_id(message: &Message) -> Option<String> {
        let code = &message.error.code;
        let number = code.number.as_ref()?;

        Some(match &code.scope {
            Some(scope) => format!("{scope}({number})"),
            None => number.to_string(),
        })
    }

    /// Checks if the suppressions file exists.
    pub fn exists(&self) -> bool {
        self.file_path.exists()
    }
}

/// Creates a diagnostic for unused suppressions.
pub fn create_unused_suppressions_diagnostic(unused: &SuppressedViolations) -> OxcDiagnostic {
    let mut details = String::new();
    for (file, rules) in unused {
        for (rule, suppression) in rules {
            details.push_str(&format!("  {file}: {rule} (count: {})\n", suppression.count));
        }
    }

    OxcDiagnostic::warn(
        "There are suppressions left that do not occur anymore. Consider re-running the command with `--prune-suppressions`."
    ).with_help(format!("Unused suppressions:\n{details}"))
}

#[cfg(test)]
mod tests {
    use oxc_diagnostics::OxcDiagnostic;
    use oxc_span::Span;

    use super::*;
    use crate::fixer::PossibleFixes;

    fn create_message(rule_id: &'static str, severity: Severity) -> Message {
        Message::new(
            OxcDiagnostic::error("test message")
                .with_error_code("eslint", rule_id)
                .with_severity(severity)
                .with_label(Span::default()),
            PossibleFixes::None,
        )
    }

    #[test]
    fn test_count_violations_by_rule() {
        let messages = vec![
            create_message("no-undef", Severity::Error),
            create_message("no-undef", Severity::Error),
            create_message("no-console", Severity::Error),
            create_message("no-unused-vars", Severity::Warning), // Should be ignored
        ];

        let counts = SuppressionsService::count_violations_by_rule(&messages);

        assert_eq!(counts.get("eslint(no-undef)").unwrap().count, 2);
        assert_eq!(counts.get("eslint(no-console)").unwrap().count, 1);
        assert!(counts.get("eslint(no-unused-vars)").is_none()); // Warnings are not counted
    }

    #[test]
    fn test_get_relative_file_path() {
        let service =
            SuppressionsService::new(PathBuf::from("suppressions.json"), PathBuf::from("/project"));

        let result = service.get_relative_file_path(Path::new("/project/src/file.js"));
        assert_eq!(result, "src/file.js");

        let result = service.get_relative_file_path(Path::new("/other/file.js"));
        assert_eq!(result, "/other/file.js");
    }

    #[test]
    fn test_apply_suppressions_no_suppressions() {
        let service =
            SuppressionsService::new(PathBuf::from("suppressions.json"), PathBuf::from("/project"));

        let messages = vec![create_message("no-undef", Severity::Error)];
        let mut results: IndexMap<&Path, Vec<Message>> = IndexMap::default();
        results.insert(Path::new("/project/src/file.js"), messages);

        let suppressions = SuppressedViolations::default();
        let result = service.apply_suppressions(&results, &suppressions);

        assert_eq!(result.messages.len(), 1);
        assert!(result.unused.is_empty());
    }

    #[test]
    fn test_apply_suppressions_with_matching_suppressions() {
        let service =
            SuppressionsService::new(PathBuf::from("suppressions.json"), PathBuf::from("/project"));

        let messages = vec![
            create_message("no-undef", Severity::Error),
            create_message("no-undef", Severity::Error),
        ];
        let mut results: IndexMap<&Path, Vec<Message>> = IndexMap::default();
        results.insert(Path::new("/project/src/file.js"), messages);

        let mut suppressions = SuppressedViolations::default();
        let mut rule_suppressions = IndexMap::default();
        rule_suppressions.insert("eslint(no-undef)".to_string(), RuleSuppression { count: 2 });
        suppressions.insert("src/file.js".to_string(), rule_suppressions);

        let result = service.apply_suppressions(&results, &suppressions);

        assert_eq!(result.messages.len(), 0); // All messages suppressed
        assert!(result.unused.is_empty()); // Exact match, no unused
    }

    #[test]
    fn test_apply_suppressions_with_unused_suppressions() {
        let service =
            SuppressionsService::new(PathBuf::from("suppressions.json"), PathBuf::from("/project"));

        let messages = vec![create_message("no-undef", Severity::Error)];
        let mut results: IndexMap<&Path, Vec<Message>> = IndexMap::default();
        results.insert(Path::new("/project/src/file.js"), messages);

        let mut suppressions = SuppressedViolations::default();
        let mut rule_suppressions = IndexMap::default();
        rule_suppressions.insert("eslint(no-undef)".to_string(), RuleSuppression { count: 3 }); // More than actual
        suppressions.insert("src/file.js".to_string(), rule_suppressions);

        let result = service.apply_suppressions(&results, &suppressions);

        assert_eq!(result.messages.len(), 0); // Message suppressed
        assert_eq!(
            result.unused.get("src/file.js").unwrap().get("eslint(no-undef)").unwrap().count,
            2
        ); // 3 - 1 = 2 unused
    }

    #[test]
    fn test_apply_suppressions_violations_exceed_suppressions() {
        let service =
            SuppressionsService::new(PathBuf::from("suppressions.json"), PathBuf::from("/project"));

        let messages = vec![
            create_message("no-undef", Severity::Error),
            create_message("no-undef", Severity::Error),
            create_message("no-undef", Severity::Error),
        ];
        let mut results: IndexMap<&Path, Vec<Message>> = IndexMap::default();
        results.insert(Path::new("/project/src/file.js"), messages);

        let mut suppressions = SuppressedViolations::default();
        let mut rule_suppressions = IndexMap::default();
        rule_suppressions.insert("eslint(no-undef)".to_string(), RuleSuppression { count: 2 }); // Less than actual
        suppressions.insert("src/file.js".to_string(), rule_suppressions);

        let result = service.apply_suppressions(&results, &suppressions);

        // When violations exceed suppressions, ALL violations are reported (ESLint behavior)
        assert_eq!(result.messages.len(), 3);
        assert!(result.unused.is_empty());
    }
}
