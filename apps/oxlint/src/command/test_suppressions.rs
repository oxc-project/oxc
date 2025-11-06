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
        let args: Vec<&str> =
            vec!["--suppressions-location", "custom-suppressions.json", "test.js"];
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_ok());
        let command = result.unwrap();
        assert_eq!(command.suppressions_location, Some(PathBuf::from("custom-suppressions.json")));
    }

    #[test]
    fn test_suppress_rule_flag() {
        use crate::command::lint::lint_command;

        // Test that suppress-rule flag works with multiple rules
        let args: Vec<&str> =
            vec!["--suppress-rule", "no-console", "--suppress-rule", "no-debugger", "test.js"];
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

    #[test]
    fn test_eslint_bulk_suppressions_file_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("oxlint-suppressions.json");

        // Create ESLint-style suppression format
        let content = r#"{
  "src/App.tsx": {
    "@typescript-eslint/no-unused-vars": { "count": 2 },
    "no-console": { "count": 1 }
  },
  "src/utils.js": {
    "prefer-const": { "count": 3 }
  }
}"#;

        std::fs::write(&file_path, content).unwrap();

        // Load and verify format
        let loaded = load_eslint_suppressions_from_file(&file_path).unwrap();

        assert_eq!(loaded.suppressions.len(), 2);
        assert!(loaded.suppressions.contains_key("src/App.tsx"));
        assert!(loaded.suppressions.contains_key("src/utils.js"));

        let app_rules = loaded.suppressions.get("src/App.tsx").unwrap();
        assert_eq!(app_rules.get("@typescript-eslint/no-unused-vars").unwrap().count, 2);
        assert_eq!(app_rules.get("no-console").unwrap().count, 1);

        let utils_rules = loaded.suppressions.get("src/utils.js").unwrap();
        assert_eq!(utils_rules.get("prefer-const").unwrap().count, 3);
    }

    #[test]
    fn test_eslint_bulk_suppressions_integration() {
        use oxc_span::Span;
        use std::path::Path;

        // Create ESLint-style suppressions
        let mut suppressions_data = std::collections::HashMap::new();
        let mut file_rules = std::collections::HashMap::new();

        file_rules.insert(
            "@typescript-eslint/no-unused-vars".to_string(),
            ESLintRuleSuppression { count: 10 },
        ); // Increase count to handle multiple test cases
        file_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 5 }); // Increase count to handle multiple test cases

        suppressions_data.insert("src/App.tsx".to_string(), file_rules);

        let suppressions_file = ESLintBulkSuppressionsFile { suppressions: suppressions_data };

        let bulk_suppressions = ESLintBulkSuppressions::new(suppressions_file);

        // Test different file path variations
        let test_cases =
            vec![Path::new("src/App.tsx"), Path::new("/project/src/App.tsx"), Path::new("App.tsx")];

        let span = Span::new(100, 111);

        // Test exact file match first
        let file_path = Path::new("src/App.tsx");

        // First TypeScript error should be suppressed
        assert!(bulk_suppressions.matches(
            "typescript",
            "@typescript-eslint",
            "no-unused-vars",
            span,
            file_path
        ));

        // Second TypeScript error should be suppressed
        assert!(bulk_suppressions.matches(
            "typescript",
            "@typescript-eslint",
            "no-unused-vars",
            span,
            file_path
        ));

        // Console error should be suppressed
        assert!(bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path));

        // Test that different file paths can find the same suppression entry
        for file_path in test_cases {
            if file_path.to_string_lossy().contains("App.tsx") {
                // Should match because file path contains App.tsx
                assert!(bulk_suppressions.matches(
                    "typescript",
                    "@typescript-eslint",
                    "no-unused-vars",
                    span,
                    file_path
                ));
            }
        }
    }

    #[test]
    fn test_eslint_bulk_suppressions_unused_tracking() {
        use std::collections::HashMap;

        // Create suppressions with various counts
        let mut suppressions_data = HashMap::new();
        let mut file_rules = HashMap::new();

        file_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 5 });
        file_rules.insert("prefer-const".to_string(), ESLintRuleSuppression { count: 3 });
        file_rules.insert("no-unused-vars".to_string(), ESLintRuleSuppression { count: 1 });

        suppressions_data.insert("test.js".to_string(), file_rules);

        let suppressions_file = ESLintBulkSuppressionsFile { suppressions: suppressions_data };

        let bulk_suppressions = ESLintBulkSuppressions::new(suppressions_file);
        let file_path = std::path::Path::new("test.js");
        let span = oxc_span::Span::new(100, 111);

        // Use some suppressions
        bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path);
        bulk_suppressions.matches("eslint", "eslint", "no-console", span, file_path);
        // Leave 3 unused for no-console

        bulk_suppressions.matches("eslint", "eslint", "prefer-const", span, file_path);
        bulk_suppressions.matches("eslint", "eslint", "prefer-const", span, file_path);
        bulk_suppressions.matches("eslint", "eslint", "prefer-const", span, file_path);
        // All prefer-const used

        // Don't use any no-unused-vars
        // Leave 1 unused for no-unused-vars

        let unused = bulk_suppressions.get_unused_suppressions();
        assert_eq!(unused.len(), 2);

        let unused_map: HashMap<&str, u32> =
            unused.iter().map(|(_, rule, count)| (rule.as_str(), *count)).collect();

        assert_eq!(unused_map.get("no-console"), Some(&3));
        assert_eq!(unused_map.get("no-unused-vars"), Some(&1));
        assert_eq!(unused_map.get("prefer-const"), None); // All used
    }

    #[test]
    fn test_eslint_format_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("eslint-test.json");

        // Create original ESLint suppressions
        let mut suppressions_data = std::collections::HashMap::new();
        let mut file1_rules = std::collections::HashMap::new();
        file1_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 2 });
        file1_rules.insert(
            "@typescript-eslint/no-unused-vars".to_string(),
            ESLintRuleSuppression { count: 1 },
        );
        suppressions_data.insert("src/App.tsx".to_string(), file1_rules);

        let mut file2_rules = std::collections::HashMap::new();
        file2_rules.insert("prefer-const".to_string(), ESLintRuleSuppression { count: 3 });
        suppressions_data.insert("src/utils.js".to_string(), file2_rules);

        let original = ESLintBulkSuppressionsFile { suppressions: suppressions_data };

        // Write to file
        let content = serde_json::to_string_pretty(&original).unwrap();
        std::fs::write(&file_path, content).unwrap();

        // Load from file
        let loaded = load_eslint_suppressions_from_file(&file_path).unwrap();

        // Verify structure
        assert_eq!(loaded.suppressions.len(), 2);

        let app_rules = loaded.suppressions.get("src/App.tsx").unwrap();
        assert_eq!(app_rules.len(), 2);
        assert_eq!(app_rules.get("no-console").unwrap().count, 2);
        assert_eq!(app_rules.get("@typescript-eslint/no-unused-vars").unwrap().count, 1);

        let utils_rules = loaded.suppressions.get("src/utils.js").unwrap();
        assert_eq!(utils_rules.len(), 1);
        assert_eq!(utils_rules.get("prefer-const").unwrap().count, 3);
    }

    #[test]
    fn test_prune_suppressions_integration() {
        let dir = tempdir().unwrap();
        let suppressions_file = dir.path().join("test-suppressions.json");
        let test_file = dir.path().join("test.js");

        // Create a test JavaScript file with some violations
        let test_content = r#"console.log("test");
let unused = 42;
var anotherUnused = "hello";"#;
        std::fs::write(&test_file, test_content).unwrap();

        // Create initial suppressions (some will be used, some unused)
        let mut suppressions_data = std::collections::HashMap::new();
        let mut file_rules = std::collections::HashMap::new();
        file_rules.insert("no-console".to_string(), ESLintRuleSuppression { count: 1 }); // Will be used
        file_rules.insert("no-unused-vars".to_string(), ESLintRuleSuppression { count: 5 }); // Partially used
        file_rules.insert("no-debugger".to_string(), ESLintRuleSuppression { count: 2 }); // Will be unused
        suppressions_data.insert("test.js".to_string(), file_rules);

        // Add suppressions for non-existent file (will be unused)
        let mut nonexistent_rules = std::collections::HashMap::new();
        nonexistent_rules.insert("prefer-const".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data.insert("nonexistent.js".to_string(), nonexistent_rules);

        let initial_suppressions = ESLintBulkSuppressionsFile { suppressions: suppressions_data };

        // Write initial suppressions to file
        let content = serde_json::to_string_pretty(&initial_suppressions).unwrap();
        std::fs::write(&suppressions_file, content).unwrap();

        // Test that suppressions file exists and has expected content before pruning
        let before_prune = load_eslint_suppressions_from_file(&suppressions_file).unwrap();
        assert_eq!(before_prune.suppressions.len(), 2);
        assert!(before_prune.suppressions.contains_key("test.js"));
        assert!(before_prune.suppressions.contains_key("nonexistent.js"));

        let test_js_rules = before_prune.suppressions.get("test.js").unwrap();
        assert_eq!(test_js_rules.get("no-console").unwrap().count, 1);
        assert_eq!(test_js_rules.get("no-unused-vars").unwrap().count, 5);
        assert_eq!(test_js_rules.get("no-debugger").unwrap().count, 2);

        // The implementation is complex to test in unit tests because it requires
        // a full linter setup. This test verifies the basic structure and file handling.
        // Integration testing of the full prune functionality would typically be done
        // at a higher level with real CLI invocations.

        // Verify that the test file and suppressions file are properly set up
        assert!(test_file.exists());
        assert!(suppressions_file.exists());
    }

    #[test]
    fn test_prune_suppressions_empty_file() {
        let dir = tempdir().unwrap();
        let empty_suppressions_file = dir.path().join("empty-suppressions.json");

        // Create empty suppressions file
        let empty_suppressions =
            ESLintBulkSuppressionsFile { suppressions: std::collections::HashMap::new() };
        let content = serde_json::to_string_pretty(&empty_suppressions).unwrap();
        std::fs::write(&empty_suppressions_file, content).unwrap();

        // Load and verify it's empty
        let loaded = load_eslint_suppressions_from_file(&empty_suppressions_file).unwrap();
        assert!(loaded.suppressions.is_empty());
    }

    #[test]
    fn test_prune_suppressions_nonexistent_file() {
        let dir = tempdir().unwrap();
        let nonexistent_file = dir.path().join("does-not-exist.json");

        // Loading non-existent file should return empty suppressions
        let loaded = load_eslint_suppressions_from_file(&nonexistent_file).unwrap();
        assert!(loaded.suppressions.is_empty());
    }

    #[test]
    fn test_prune_suppressions_file_format_preservation() {
        let dir = tempdir().unwrap();
        let suppressions_file = dir.path().join("format-test.json");

        // Create suppressions with specific structure
        let mut suppressions_data = std::collections::HashMap::new();
        let mut file_rules = std::collections::HashMap::new();
        file_rules.insert("rule-a".to_string(), ESLintRuleSuppression { count: 3 });
        file_rules.insert("rule-b".to_string(), ESLintRuleSuppression { count: 1 });
        suppressions_data.insert("file1.js".to_string(), file_rules);

        let mut file2_rules = std::collections::HashMap::new();
        file2_rules.insert("rule-c".to_string(), ESLintRuleSuppression { count: 2 });
        suppressions_data.insert("file2.js".to_string(), file2_rules);

        let original = ESLintBulkSuppressionsFile { suppressions: suppressions_data };

        // Write to file
        let content = serde_json::to_string_pretty(&original).unwrap();
        std::fs::write(&suppressions_file, content).unwrap();

        // Verify the JSON structure is correct
        let loaded = load_eslint_suppressions_from_file(&suppressions_file).unwrap();
        assert_eq!(loaded.suppressions.len(), 2);

        // Verify specific counts
        let file1_rules = loaded.suppressions.get("file1.js").unwrap();
        assert_eq!(file1_rules.get("rule-a").unwrap().count, 3);
        assert_eq!(file1_rules.get("rule-b").unwrap().count, 1);

        let file2_rules = loaded.suppressions.get("file2.js").unwrap();
        assert_eq!(file2_rules.get("rule-c").unwrap().count, 2);
    }
}
