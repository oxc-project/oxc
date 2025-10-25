#[cfg(test)]
mod tests {
    use crate::command::suppressions::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_cli_flags_added() {
        use crate::command::lint::lint_command;

        // Test that suppress-all flag works
        let args: Vec<&str> = vec!["--suppress-all", "test.js"];
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_ok());
        assert!(result.unwrap().suppress_all);
    }

    #[test]
    fn test_suppressions_location_flag() {
        use crate::command::lint::lint_command;

        // Test that suppressions-location flag works
        let args: Vec<&str> = vec!["--suppressions-location", "custom-suppressions.json", "test.js"];
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_ok());
        let command = result.unwrap();
        assert_eq!(command.suppressions_location, Some(PathBuf::from("custom-suppressions.json")));
    }

    #[test]
    fn test_suppress_rule_flag() {
        use crate::command::lint::lint_command;

        // Test that suppress-rule flag works with multiple rules
        let args: Vec<&str> = vec!["--suppress-rule", "no-console", "--suppress-rule", "no-debugger", "test.js"];
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_ok());
        let command = result.unwrap();
        assert_eq!(command.suppress_rule, vec!["no-console", "no-debugger"]);
    }

    #[test]
    fn test_prune_suppressions_flag() {
        use crate::command::lint::lint_command;

        // Test that prune-suppressions flag works
        let args: Vec<&str> = vec!["--prune-suppressions", "test.js"];
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_ok());
        assert!(result.unwrap().prune_suppressions);
    }

    #[test]
    fn test_pass_on_unpruned_suppressions_flag() {
        use crate::command::lint::lint_command;

        // Test that pass-on-unpruned-suppressions flag works
        let args: Vec<&str> = vec!["--pass-on-unpruned-suppressions", "test.js"];
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_ok());
        assert!(result.unwrap().pass_on_unpruned_suppressions);
    }

    #[test]
    fn test_load_empty_suppressions_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty-suppressions.json");

        // Test loading non-existent file returns empty suppressions
        let result = load_suppressions_from_file(&file_path);
        assert!(result.is_ok());
        let suppressions = result.unwrap();
        assert!(suppressions.suppressions.is_empty());
    }

    #[test]
    fn test_write_and_load_suppressions_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test-suppressions.json");

        let original = SuppressionsFile {
            suppressions: vec![
                SuppressionEntry {
                    files: vec!["src/test.js".to_string()],
                    rules: vec!["no-console".to_string()],
                    line: 10,
                    column: 5,
                    end_line: Some(10),
                    end_column: Some(16),
                    reason: "Test suppression".to_string(),
                },
                SuppressionEntry {
                    files: vec!["src/another.js".to_string()],
                    rules: vec!["no-debugger".to_string(), "no-unused-vars".to_string()],
                    line: 20,
                    column: 0,
                    end_line: None,
                    end_column: None,
                    reason: "Multiple rules suppression".to_string(),
                },
            ],
        };

        // Write and then load back
        write_suppressions_to_file(&original, &file_path).unwrap();
        let loaded = load_suppressions_from_file(&file_path).unwrap();

        // Verify they match
        assert_eq!(loaded.suppressions.len(), 2);
        assert_eq!(loaded.suppressions[0].files, vec!["src/test.js"]);
        assert_eq!(loaded.suppressions[0].rules, vec!["no-console"]);
        assert_eq!(loaded.suppressions[0].line, 10);
        assert_eq!(loaded.suppressions[0].column, 5);

        assert_eq!(loaded.suppressions[1].files, vec!["src/another.js"]);
        assert_eq!(loaded.suppressions[1].rules, vec!["no-debugger", "no-unused-vars"]);
        assert_eq!(loaded.suppressions[1].line, 20);
    }

    #[test]
    fn test_merge_suppressions_replace_mode() {
        let existing = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["old.js".to_string()],
                rules: vec!["old-rule".to_string()],
                line: 1,
                column: 0,
                end_line: None,
                end_column: None,
                reason: "Old suppression".to_string(),
            }],
        };

        let new_suppressions = vec![SuppressionEntry {
            files: vec!["new.js".to_string()],
            rules: vec!["new-rule".to_string()],
            line: 2,
            column: 0,
            end_line: None,
            end_column: None,
            reason: "New suppression".to_string(),
        }];

        // Test replace mode
        let merged = merge_suppressions(&existing, new_suppressions, true);

        assert_eq!(merged.suppressions.len(), 1);
        assert_eq!(merged.suppressions[0].files, vec!["new.js"]);
        assert_eq!(merged.suppressions[0].rules, vec!["new-rule"]);
    }

    #[test]
    fn test_merge_suppressions_append_mode() {
        let existing = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["old.js".to_string()],
                rules: vec!["old-rule".to_string()],
                line: 1,
                column: 0,
                end_line: None,
                end_column: None,
                reason: "Old suppression".to_string(),
            }],
        };

        let new_suppressions = vec![SuppressionEntry {
            files: vec!["new.js".to_string()],
            rules: vec!["new-rule".to_string()],
            line: 2,
            column: 0,
            end_line: None,
            end_column: None,
            reason: "New suppression".to_string(),
        }];

        // Test append mode
        let merged = merge_suppressions(&existing, new_suppressions, false);

        assert_eq!(merged.suppressions.len(), 2);
        assert_eq!(merged.suppressions[0].files, vec!["old.js"]);
        assert_eq!(merged.suppressions[0].rules, vec!["old-rule"]);
        assert_eq!(merged.suppressions[1].files, vec!["new.js"]);
        assert_eq!(merged.suppressions[1].rules, vec!["new-rule"]);
    }

    #[test]
    fn test_prune_suppressions_removes_correct_entries() {
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

        // Remove indices 0 and 2 (first and third entries)
        let unused_indices = vec![0, 2];
        let pruned = prune_suppressions(&suppressions, &unused_indices);

        assert_eq!(pruned.suppressions.len(), 1);
        assert_eq!(pruned.suppressions[0].files, vec!["file2.js"]);
        assert_eq!(pruned.suppressions[0].rules, vec!["rule2"]);
    }

    #[test]
    fn test_suppression_matcher_basic_functionality() {
        use oxc_span::Span;
        use std::path::Path;

        let suppressions = SuppressionsFile {
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

        let matcher = SuppressionMatcher::new(suppressions);
        let file_path = Path::new("src/test.js");
        let span = Span::new(100, 111); // Dummy span

        // Test matching
        let matches = matcher.matches("eslint", "eslint", "no-console", span, file_path);

        // For now, this should match because of the basic implementation
        // In a real implementation, we'd need proper line/column to span conversion
        assert!(matches);
    }

    #[test]
    fn test_violation_info_creation() {
        let violation = ViolationInfo {
            file_path: "src/test.js".to_string(),
            rule_name: "no-console".to_string(),
            plugin_name: "eslint".to_string(),
            line: 10,
            column: 5,
            end_line: Some(10),
            end_column: Some(16),
            message: "Unexpected console statement.".to_string(),
            severity: "error".to_string(),
        };

        assert_eq!(violation.file_path, "src/test.js");
        assert_eq!(violation.rule_name, "no-console");
        assert_eq!(violation.plugin_name, "eslint");
        assert_eq!(violation.line, 10);
        assert_eq!(violation.column, 5);
        assert_eq!(violation.message, "Unexpected console statement.");
        assert_eq!(violation.severity, "error");
    }

    #[test]
    fn test_validate_suppression_file_valid() {
        let suppressions = SuppressionsFile {
            suppressions: vec![SuppressionEntry {
                files: vec!["test.js".to_string()],
                rules: vec!["no-console".to_string()],
                line: 10,
                column: 5,
                end_line: Some(10),
                end_column: Some(16),
                reason: "Valid suppression".to_string(),
            }],
        };

        let result = validate_suppression_file(&suppressions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_suppression_file_invalid() {
        let suppressions = SuppressionsFile {
            suppressions: vec![
                SuppressionEntry {
                    files: vec![], // Empty files - invalid
                    rules: vec!["no-console".to_string()],
                    line: 10,
                    column: 5,
                    end_line: Some(10),
                    end_column: Some(16),
                    reason: "Invalid suppression".to_string(),
                },
                SuppressionEntry {
                    files: vec!["test.js".to_string()],
                    rules: vec![], // Empty rules - invalid
                    line: 10,
                    column: 5,
                    end_line: Some(10),
                    end_column: Some(16),
                    reason: "Invalid suppression".to_string(),
                },
                SuppressionEntry {
                    files: vec!["test.js".to_string()],
                    rules: vec!["no-console".to_string()],
                    line: 10,
                    column: 5,
                    end_line: Some(10),
                    end_column: Some(16),
                    reason: "".to_string(), // Empty reason - invalid
                },
            ],
        };

        let result = validate_suppression_file(&suppressions);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
        assert!(errors[0].contains("has no files specified"));
        assert!(errors[1].contains("has no rules specified"));
        assert!(errors[2].contains("has no reason specified"));
    }

    #[test]
    fn test_suppression_tracker_usage() {
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
}
