#[cfg(test)]
mod tests {
    use crate::suppression::SuppressionManager;
    use std::path::Path;
    use tempfile::NamedTempFile;

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
        let file_path1 = Path::new("src\\test.js");  // Windows-style path
        let file_path2 = Path::new("src/test.js");   // Unix-style path
        
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
        
        assert_eq!(loaded_manager.get_suppression_count(file_path, "eslint", "no-console"), Some(2));
        assert_eq!(loaded_manager.get_suppression_count(file_path, "@typescript-eslint", "no-unused-vars"), Some(1));
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
}