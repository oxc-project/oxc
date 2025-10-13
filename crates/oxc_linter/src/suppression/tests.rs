#[cfg(test)]
mod tests {
    use crate::suppression::{SuppressionManager, ThreadSafeSuppressionManager};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::thread;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_suppression_manager_new() {
        let manager = SuppressionManager::new();
        assert_eq!(manager.get_all_files().len(), 0);
    }

    #[test]
    fn test_add_suppression() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        manager.add_suppression(file_path, "eslint", "no-console", 3);

        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-console"), Some(3));
        assert_eq!(manager.get_all_files().len(), 1);
    }

    #[test]
    fn test_add_suppression_with_zero_count() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        // Adding suppression with count 0 should be ignored
        manager.add_suppression(file_path, "eslint", "no-console", 0);

        // Should not be added to suppressions
        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-console"), None);
        assert_eq!(manager.get_all_files().len(), 0);

        // Adding valid suppression should work
        manager.add_suppression(file_path, "eslint", "no-console", 2);
        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-console"), Some(2));
        assert_eq!(manager.get_all_files().len(), 1);
    }

    #[test]
    fn test_is_suppressed() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        manager.add_suppression(file_path, "eslint", "no-console", 2);

        // Should be suppressed for first 2 violations
        assert!(manager.is_suppressed(file_path, "eslint", "no-console"));
        manager.record_violation(file_path, "eslint", "no-console");

        assert!(manager.is_suppressed(file_path, "eslint", "no-console"));
        manager.record_violation(file_path, "eslint", "no-console");

        // Should not be suppressed for 3rd violation
        assert!(!manager.is_suppressed(file_path, "eslint", "no-console"));
    }

    #[test]
    fn test_file_path_normalization() {
        let mut manager = SuppressionManager::new();
        let file_path1 = Path::new("src\\test.js"); // Windows-style path
        let file_path2 = Path::new("src/test.js"); // Unix-style path

        manager.add_suppression(file_path1, "eslint", "no-console", 1);

        // Should work with both path styles due to normalization
        assert_eq!(manager.get_suppression_count(file_path2, "eslint", "no-console"), Some(1));
    }

    #[test]
    fn test_save_and_load() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        manager.add_suppression(file_path, "eslint", "no-console", 2);
        manager.add_suppression(file_path, "@typescript-eslint", "no-unused-vars", 1);

        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        manager.save(temp_file.path()).unwrap();

        // Load from file
        let loaded_manager = SuppressionManager::load(temp_file.path()).unwrap();

        assert_eq!(
            loaded_manager.get_suppression_count(file_path, "eslint", "no-console"),
            Some(2)
        );
        assert_eq!(
            loaded_manager.get_suppression_count(file_path, "@typescript-eslint", "no-unused-vars"),
            Some(1)
        );
    }

    #[test]
    fn test_prune_unused() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        // Add suppressions
        manager.add_suppression(file_path, "eslint", "no-console", 2);
        manager.add_suppression(file_path, "eslint", "no-debugger", 1);

        // Record only one violation
        manager.record_violation(file_path, "eslint", "no-console");

        // Prune unused
        manager.prune_unused();

        // Only the used suppression should remain
        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-console"), Some(2));
        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-debugger"), None);
    }

    #[test]
    fn test_reset_counts() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        manager.add_suppression(file_path, "eslint", "no-console", 2);
        manager.record_violation(file_path, "eslint", "no-console");

        // Should be suppressed
        assert!(manager.is_suppressed(file_path, "eslint", "no-console"));

        manager.reset_counts();

        // Should still be suppressed after reset
        assert!(manager.is_suppressed(file_path, "eslint", "no-console"));
    }

    #[test]
    fn test_thread_safe_suppression_manager() {
        let manager = SuppressionManager::new();
        let thread_safe = ThreadSafeSuppressionManager::new(manager);

        let file_path = Path::new("src/test.js");

        // Test basic operations work in thread-safe context
        thread_safe.record_violation(file_path, "eslint", "no-console");
        assert!(!thread_safe.is_suppressed(file_path, "eslint", "no-console"));
    }

    #[test]
    fn test_concurrent_access() {
        let manager = SuppressionManager::new();
        let thread_safe = Arc::new(ThreadSafeSuppressionManager::new(manager));

        let handles: Vec<_> = (0..10).map(|i| {
            let manager_clone = Arc::clone(&thread_safe);
            thread::spawn(move || {
                let file_path = PathBuf::from(format!("src/test{}.js", i));

                // Each thread records violations concurrently
                for j in 0..5 {
                    manager_clone.record_violation(&file_path, "eslint", &format!("rule-{}", j));
                    manager_clone.is_suppressed(&file_path, "eslint", &format!("rule-{}", j));
                }
            })
        }).collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Test passes if no panics occurred
    }

    #[test]
    fn test_add_violations_from_diagnostics() {
        let mut manager = SuppressionManager::new();

        let violations = vec![
            (PathBuf::from("src/test1.js"), "eslint".to_string(), "no-console".to_string()),
            (PathBuf::from("src/test1.js"), "eslint".to_string(), "no-console".to_string()),
            (PathBuf::from("src/test1.js"), "@typescript-eslint".to_string(), "no-unused-vars".to_string()),
            (PathBuf::from("src/test2.js"), "eslint".to_string(), "no-console".to_string()),
        ];

        manager.add_violations_from_diagnostics(&violations);

        // Check that violations were correctly counted and added as suppressions
        assert_eq!(manager.get_suppression_count(Path::new("src/test1.js"), "eslint", "no-console"), Some(2));
        assert_eq!(manager.get_suppression_count(Path::new("src/test1.js"), "@typescript-eslint", "no-unused-vars"), Some(1));
        assert_eq!(manager.get_suppression_count(Path::new("src/test2.js"), "eslint", "no-console"), Some(1));
    }

    #[test]
    fn test_path_normalization_edge_cases() {
        let mut manager = SuppressionManager::new();

        // Test various path formats
        let paths = vec![
            Path::new("src/test.js"),
            Path::new("src\\test.js"),
            Path::new("./src/test.js"),
            Path::new("src/./test.js"),
            Path::new("src/../src/test.js"),
        ];

        for (i, path) in paths.iter().enumerate() {
            manager.add_suppression(path, "eslint", "no-console", i as u32 + 1);
        }

        // All paths should normalize to the same key and overwrite each other
        // The last one should have count 5
        assert_eq!(manager.get_suppression_count(Path::new("src/test.js"), "eslint", "no-console"), Some(5));
    }

    #[test]
    fn test_suppression_file_io() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        let mut manager = SuppressionManager::new();
        manager.add_suppression(Path::new("src/test.js"), "eslint", "no-console", 3);
        manager.add_suppression(Path::new("lib/utils.ts"), "@typescript-eslint", "no-unused-vars", 1);

        // Save to file
        manager.save(temp_path).unwrap();

        // Load from file
        let loaded_manager = SuppressionManager::load(temp_path).unwrap();

        // Verify data was preserved
        assert_eq!(loaded_manager.get_suppression_count(Path::new("src/test.js"), "eslint", "no-console"), Some(3));
        assert_eq!(loaded_manager.get_suppression_count(Path::new("lib/utils.ts"), "@typescript-eslint", "no-unused-vars"), Some(1));
    }

    #[test]
    fn test_prune_unused_suppressions() {
        let mut manager = SuppressionManager::new();
        let file_path = Path::new("src/test.js");

        // Add suppressions
        manager.add_suppression(file_path, "eslint", "no-console", 2);
        manager.add_suppression(file_path, "eslint", "no-unused-vars", 1);

        // Record some violations but not all
        manager.record_violation(file_path, "eslint", "no-console");

        // Prune unused suppressions
        manager.prune_unused();

        // Only the rule with recorded violations should remain
        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-console"), Some(2));
        assert_eq!(manager.get_suppression_count(file_path, "eslint", "no-unused-vars"), None);
    }

    #[test]
    fn test_end_to_end_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let suppression_file_path = temp_dir.path().join("suppressions.json");

        // Step 1: Generate initial suppressions
        let mut manager = SuppressionManager::new();
        let violations = vec![
            (PathBuf::from("src/app.js"), "eslint".to_string(), "no-console".to_string()),
            (PathBuf::from("src/app.js"), "eslint".to_string(), "no-console".to_string()),
            (PathBuf::from("src/utils.js"), "eslint".to_string(), "no-unused-vars".to_string()),
        ];

        manager.add_violations_from_diagnostics(&violations);
        manager.save(&suppression_file_path).unwrap();

        // Step 2: Load suppressions for linting
        let loaded_manager = SuppressionManager::load(&suppression_file_path).unwrap();
        assert_eq!(loaded_manager.get_all_files().len(), 2);

        // Step 3: Test suppression behavior
        let thread_safe = ThreadSafeSuppressionManager::new(loaded_manager);

        // First violation should be suppressed
        assert!(thread_safe.is_suppressed(Path::new("src/app.js"), "eslint", "no-console"));
        thread_safe.record_violation(Path::new("src/app.js"), "eslint", "no-console");

        // Second violation should be suppressed
        assert!(thread_safe.is_suppressed(Path::new("src/app.js"), "eslint", "no-console"));
        thread_safe.record_violation(Path::new("src/app.js"), "eslint", "no-console");

        // Third violation should NOT be suppressed
        assert!(!thread_safe.is_suppressed(Path::new("src/app.js"), "eslint", "no-console"));
    }
}
