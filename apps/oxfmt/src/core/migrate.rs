use std::path::Path;

use crate::core::utils;

/// Read and parse .prettierignore file from current directory
/// Returns patterns with comments and empty lines filtered out
pub fn read_prettierignore(cwd: &Path) -> Vec<String> {
    let prettierignore_path = cwd.join(".prettierignore");

    match utils::read_to_string(&prettierignore_path) {
        Ok(content) => content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect(),
        Err(_) => {
            // .prettierignore doesn't exist or can't be read
            Vec::new()
        }
    }
}
