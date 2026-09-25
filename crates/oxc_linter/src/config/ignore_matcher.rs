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
        let base_gi = if base_patterns.is_empty() {
            None
        } else {
            let mut builder = GitignoreBuilder::new(base_root);
            for pat in base_patterns {
                let _ = builder.add_line(None, pat);
            }
            builder.build().ok()
        };

        // Sort nested configs deepest-to-shallowest for correct precedence
        nested.sort_by_cached_key(|(_, root)| std::cmp::Reverse(root.components().count()));
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
    /// Checks nested configs deepest-to-shallowest, so the deepest config wins.
    pub fn should_ignore(&self, path: &Path) -> bool {
        self.evaluate(path, true)
    }

    /// Core ignore evaluation.
    ///
    /// Finds the deepest nested config whose directory is an ancestor of `path`
    /// (or equal to it, when `include_equal` is `true`). That config's
    /// `ignorePatterns` govern `path` exclusively, *unless* the nested config's
    /// own directory is itself ignored by a shallower config — in that case the
    /// nested config never takes effect and `path` stays ignored. When no nested
    /// config applies, the base (root) config decides.
    ///
    /// This makes a root config's directory-level `ignorePatterns` (e.g.
    /// `"vendored"`) exclude a whole subtree even when that subtree contains its
    /// own nested config, while still letting a nested config in a *non-ignored*
    /// directory override the root's patterns for its own files.
    /// See <https://github.com/oxc-project/oxc/issues/23182>.
    fn evaluate(&self, path: &Path, include_equal: bool) -> bool {
        for (ignore, root) in &self.nested {
            let covers = path.starts_with(root) && (include_equal || path != root);
            if covers {
                // If this nested config's own directory is excluded by a shallower
                // config, the nested config must not "un-ignore" its subtree.
                if self.evaluate(root, false) {
                    return true;
                }
                return ignore
                    .as_ref()
                    .is_some_and(|gi| gi.matched_path_or_any_parents(path, false).is_ignore());
            }
        }
        self.base.as_ref().is_some_and(|base| {
            path.starts_with(base.path())
                && base.matched_path_or_any_parents(path, false).is_ignore()
        })
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

    #[test]
    fn test_base_ignores_directory_with_nested_config() {
        // Root config ignores the whole `vendored` directory. A nested config
        // living *inside* that ignored directory must not resurrect it.
        // https://github.com/oxc-project/oxc/issues/23182
        let base_patterns = vec!["vendored".to_string(), "vendored/**".to_string()];
        let base_root = Path::new("/repo");

        // Nested config inside the ignored directory, with no ignore patterns.
        let nested = (vec![], PathBuf::from("/repo/vendored/pkg"));

        let matcher = LintIgnoreMatcher::new(&base_patterns, base_root, vec![nested]);

        // Files under the ignored directory stay ignored despite the nested config.
        assert!(matcher.should_ignore(Path::new("/repo/vendored/pkg/src/file.ts")));
        assert!(matcher.should_ignore(Path::new("/repo/vendored/pkg/index.ts")));

        // Files outside the ignored directory are unaffected.
        assert!(!matcher.should_ignore(Path::new("/repo/src/file.ts")));
    }

    #[test]
    fn test_nested_config_in_non_ignored_dir_overrides_base() {
        // Root ignores all `*.ts`, but a nested config in a directory that is
        // itself *not* ignored may un-ignore its own files.
        let base_patterns = vec!["**/*.ts".to_string()];
        let base_root = Path::new("/repo");

        let nested = (vec![], PathBuf::from("/repo/pkg"));

        let matcher = LintIgnoreMatcher::new(&base_patterns, base_root, vec![nested]);

        // `pkg` directory itself is not matched by `**/*.ts`, so its nested config
        // takes effect and un-ignores the `.ts` files within it.
        assert!(!matcher.should_ignore(Path::new("/repo/pkg/file.ts")));
        // Files outside the nested config are still ignored by the base.
        assert!(matcher.should_ignore(Path::new("/repo/file.ts")));
    }

    #[test]
    fn test_lint_file_outside_root() {
        let base_patterns = vec!["pattern".to_string()];
        let base_root = Path::new("/repo1");

        let matcher = LintIgnoreMatcher::new(&base_patterns, base_root, vec![]);

        // Test that path outside root shouldn't be ignored.
        assert!(!matcher.should_ignore(Path::new("/repo2/pattern/file.ts")));
    }
}
