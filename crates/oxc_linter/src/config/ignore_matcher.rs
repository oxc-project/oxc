use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Holds ignore matchers for base and nested configs, for fast filtering in lint.rs

#[derive(Debug)]
pub struct LintIgnoreMatcher {
    base: Option<Gitignore>,
    nested: Vec<(Option<Gitignore>, PathBuf)>,
}

impl LintIgnoreMatcher {
    /// Create a matcher from the base patterns and all nested patterns.
    /// Accepts patterns directly, builds Gitignore internally.
    pub fn new(
        base_patterns: &[String],
        base_root: &Path,
        mut nested: Vec<(Vec<String>, PathBuf)>,
    ) -> Self {
        let base_gi = {
            let mut builder = GitignoreBuilder::new(base_root);
            for pat in base_patterns {
                let _ = builder.add_line(None, pat);
            }
            builder.build().ok()
        };

        // Sort nested configs deepest-to-shallowest for correct precedence
        nested.sort_unstable_by(|a, b| {
            let a_len = a.1.components().count();
            let b_len = b.1.components().count();
            b_len.cmp(&a_len)
        });
        let nested = nested
            .into_iter()
            .map(|(patterns, root)| {
                if patterns.is_empty() {
                    (None, root)
                } else {
                    let mut builder = GitignoreBuilder::new(&root);
                    for pat in &patterns {
                        let _ = builder.add_line(None, pat);
                    }
                    (builder.build().ok(), root)
                }
            })
            .collect();
        Self { base: base_gi, nested }
    }

    /// Returns true if the path should be ignored by any config.
    /// Checks nested configs deepest-to-shallowest, so deepest config wins.
    pub fn should_ignore(&self, path: &Path) -> bool {
        // If a nested config matches, only use its ignore patterns (do not fall back to base)
        for (ignore, root) in &self.nested {
            if path.starts_with(root) {
                return ignore
                    .as_ref()
                    .is_some_and(|gi| gi.matched_path_or_any_parents(path, false).is_ignore());
            }
        }
        self.base
            .as_ref()
            .is_some_and(|base| base.matched_path_or_any_parents(path, false).is_ignore())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_deepest_config_precedence() {
        // Base ignores all *.js
        let base_patterns = vec!["*.js".to_string()];
        let base_root = Path::new("/repo");

        let nested1 = (vec![], PathBuf::from("/repo/all_allowed"));
        let nested2 = (vec!["*.ts".to_string()], PathBuf::from("/repo/all_allowed/ts"));
        let nested3 = (vec!["*.js".to_string()], PathBuf::from("/repo/all_allowed/ts/js"));

        let matcher =
            LintIgnoreMatcher::new(&base_patterns, base_root, vec![nested1, nested2, nested3]);

        // Path in /repo/all_allowed/ts/js should be ignored by nested3 (deepest)
        assert!(matcher.should_ignore(Path::new("/repo/all_allowed/ts/js/file.js")));
        assert!(!matcher.should_ignore(Path::new("/repo/all_allowed/ts/js/file.ts")));

        // Path in /repo/all_allowed/ts should be ignored by nested2 for *.ts, base for *.js
        assert!(!matcher.should_ignore(Path::new("/repo/all_allowed/ts/file.js")));
        assert!(matcher.should_ignore(Path::new("/repo/all_allowed/ts/file.ts")));

        // Path in /repo/a should be ignored by base for *.js, not for *.ts
        assert!(!matcher.should_ignore(Path::new("/repo/all_allowed/file.js")));
        assert!(!matcher.should_ignore(Path::new("/repo/all_allowed/file.ts")));

        // Path outside any nested config, only base applies
        assert!(matcher.should_ignore(Path::new("/repo/file.js")));
        assert!(!matcher.should_ignore(Path::new("/repo/file.ts")));
    }
}
