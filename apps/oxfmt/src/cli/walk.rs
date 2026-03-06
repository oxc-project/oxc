use std::{
    path::{Path, PathBuf},
    sync::{Mutex, mpsc},
};

use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    overrides::OverrideBuilder,
};
use rustc_hash::FxHashSet;

use crate::core::{FormatFileStrategy, utils::normalize_relative_path};

pub struct Walk {
    inner: ignore::WalkParallel,
}

impl Walk {
    pub fn build(
        cwd: &Path,
        paths: &[PathBuf],
        ignore_paths: &[PathBuf],
        with_node_modules: bool,
        oxfmtrc_path: Option<&Path>,
        ignore_patterns: &[String],
        nested_ignore_patterns: Vec<(Vec<String>, PathBuf)>,
    ) -> Result<Option<Self>, String> {
        //
        // Classify and normalize specified paths
        //
        let mut target_paths = FxHashSet::default();
        let mut glob_patterns = vec![];
        let mut exclude_patterns = vec![];

        for path in paths {
            let path_str = path.to_string_lossy();

            // Instead of `oxlint`'s `--ignore-pattern=PAT`,
            // `oxfmt` supports `!` prefix in paths like Prettier.
            if path_str.starts_with('!') {
                exclude_patterns.push(path_str.to_string());
                continue;
            }

            // Normalize `./` prefix
            let normalized =
                if let Some(stripped) = path_str.strip_prefix("./") { stripped } else { &path_str };

            // Separate glob patterns from concrete paths
            if is_glob_pattern(normalized, cwd) {
                glob_patterns.push(normalized.to_string());
                continue;
            }

            // Resolve full path for concrete paths
            let full_path = if path.is_absolute() {
                path.clone()
            } else if normalized == "." {
                // NOTE: `.` and cwd behave differently, need to normalize
                cwd.to_path_buf()
            } else {
                cwd.join(normalized)
            };
            target_paths.insert(full_path);
        }

        // Expand glob patterns and add to target paths
        // NOTE: See `expand_glob_patterns()` for why we pre-expand globs here
        if !glob_patterns.is_empty() {
            target_paths.extend(expand_glob_patterns(cwd, &glob_patterns)?);
        }

        // Default to `cwd` if no positive paths were specified.
        // Exclude patterns alone should still walk, but unmatched globs should not.
        if target_paths.is_empty() && glob_patterns.is_empty() {
            target_paths.insert(cwd.to_path_buf());
        }

        //
        // Build ignores
        //
        // Matchers are split into two categories:
        // - Global: always apply regardless of nested config scope (ignore files, `!` excludes)
        // - Scoped: follow deepest-match-wins precedence (root/nested `ignorePatterns`)
        //
        // NOTE: Git ignore files are handled by `WalkBuilder` itself
        let mut global_matchers: Vec<Gitignore> = vec![];

        // 1. Handle formatter ignore files (`.prettierignore`, or `--ignore-path`)
        // These are global — always apply regardless of nested config scope
        for ignore_path in &load_ignore_paths(cwd, ignore_paths)? {
            let (gitignore, err) = Gitignore::new(ignore_path);
            if let Some(err) = err {
                return Err(format!(
                    "Failed to parse ignore file {}: {err}",
                    ignore_path.display()
                ));
            }
            global_matchers.push(gitignore);
        }

        // 2. Handle root `oxfmtrc.ignorePatterns` — scoped (only applies outside nested configs)
        let base_scoped = if !ignore_patterns.is_empty()
            && let Some(oxfmtrc_path) = oxfmtrc_path
        {
            let mut builder = GitignoreBuilder::new(
                oxfmtrc_path.parent().expect("`oxfmtrc_path` should have a parent directory"),
            );
            for pattern in ignore_patterns {
                if builder.add_line(None, pattern).is_err() {
                    return Err(format!(
                        "Failed to add ignore pattern `{pattern}` from `.ignorePatterns`"
                    ));
                }
            }
            Some(builder.build().map_err(|_| "Failed to build ignores".to_string())?)
        } else {
            None
        };

        // 3. Handle `!` prefixed paths — global (always apply)
        if !exclude_patterns.is_empty() {
            let mut builder = GitignoreBuilder::new(cwd);
            for pattern in &exclude_patterns {
                // Remove the leading `!` because `GitignoreBuilder` uses `!` as negation
                let pattern = pattern
                    .strip_prefix('!')
                    .expect("There should be a `!` prefix, already checked");
                if builder.add_line(None, pattern).is_err() {
                    return Err(format!(
                        "Failed to add ignore pattern `{pattern}` from `!` prefix"
                    ));
                }
            }
            let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
            global_matchers.push(gitignore);
        }

        //
        // Build nested ignore matcher
        //
        let nested_matcher =
            NestedIgnoreMatcher::new(global_matchers, base_scoped, nested_ignore_patterns);

        //
        // Filter positional paths by formatter ignores
        //
        // Base paths passed to `WalkBuilder` are not filtered by `filter_entry()`,
        // so we need to filter them here before passing to the walker.
        // This is needed for cases like `husky`, may specify ignored paths as staged files.
        // NOTE: Git ignored paths are not filtered here.
        // But it's OK because in cases like `husky`, they are never staged.
        let target_paths: Vec<_> = target_paths
            .into_iter()
            .filter(|path| !nested_matcher.is_ignored(path, path.is_dir(), true))
            .collect();

        // If no target paths remain after filtering, return `None`.
        // Not an error, but nothing to format, leave it to the caller how to handle.
        let Some(first_path) = target_paths.first() else {
            return Ok(None);
        };

        // Add all non-`!` prefixed paths to the walker base
        let mut inner = ignore::WalkBuilder::new(first_path);
        for path in target_paths.iter().skip(1) {
            inner.add(path);
        }

        // NOTE: If return `false` here, it will not be `visit()`ed at all
        inner.filter_entry(move |entry| {
            let Some(file_type) = entry.file_type() else {
                return false;
            };
            let is_dir = file_type.is_dir();

            if is_dir {
                // We are setting `.hidden(false)` on the `WalkBuilder` below,
                // it means we want to include hidden files and directories.
                // However, we (and also Prettier) still skip traversing certain directories.
                // https://prettier.io/docs/ignore#ignoring-files-prettierignore
                let is_ignored_dir = {
                    let dir_name = entry.file_name();
                    dir_name == ".git"
                        || dir_name == ".jj"
                        || dir_name == ".sl"
                        || dir_name == ".svn"
                        || dir_name == ".hg"
                        || (!with_node_modules && dir_name == "node_modules")
                };
                if is_ignored_dir {
                    return false;
                }
            }

            // Check ignore files, patterns (with nested config precedence)
            if nested_matcher.is_ignored(entry.path(), is_dir, false) {
                return false;
            }

            // NOTE: In addition to ignoring based on ignore files and patterns here,
            // we also apply extra filtering in the visitor `visit()` below.
            // We need to return `bool` for `filter_entry()` here,
            // but we don't want to duplicate logic in the visitor again.
            true
        });

        let inner = apply_walk_settings(&mut inner).build_parallel();
        Ok(Some(Self { inner }))
    }

    /// Stream entries through a channel as they are discovered
    pub fn stream_entries(self) -> mpsc::Receiver<FormatFileStrategy> {
        let (sender, receiver) = mpsc::channel::<FormatFileStrategy>();

        // Spawn the walk operation in a separate thread
        rayon::spawn(move || {
            let mut builder = WalkBuilder { sender };
            self.inner.visit(&mut builder);
            // Channel will be closed when builder is dropped
        });

        receiver
    }
}

// ---

/// Handles ignore matching with nested config precedence.
///
/// Two categories of matchers:
/// - **Global** (ignore files like `.prettierignore`, `!` CLI excludes): always checked.
/// - **Scoped** (`ignorePatterns` from configs): deepest nested config wins, root only
///   applies outside any nested scope. Follows the same pattern as oxlint's `LintIgnoreMatcher`
///   in `crates/oxc_linter/src/config/ignore_matcher.rs`.
struct NestedIgnoreMatcher {
    /// Always-apply matchers (ignore files, `!` CLI excludes).
    global: Vec<Gitignore>,
    /// Root `ignorePatterns` — only applies outside any nested config scope.
    base_scoped: Option<Gitignore>,
    /// Nested `ignorePatterns` sorted deepest-to-shallowest.
    nested_scoped: Vec<(Option<Gitignore>, PathBuf)>,
}

impl NestedIgnoreMatcher {
    fn new(
        global: Vec<Gitignore>,
        base_scoped: Option<Gitignore>,
        mut nested_patterns: Vec<(Vec<String>, PathBuf)>,
    ) -> Self {
        // Sort deepest-to-shallowest for correct precedence
        nested_patterns
            .sort_unstable_by(|a, b| b.1.components().count().cmp(&a.1.components().count()));

        let nested_scoped = nested_patterns
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

        Self { global, base_scoped, nested_scoped }
    }

    /// Check if a path should be ignored.
    ///
    /// When `check_ancestors: true`, also checks if any parent directory is ignored.
    /// This is more expensive, but necessary when paths are passed directly via CLI arguments.
    fn is_ignored(&self, path: &Path, is_dir: bool, check_ancestors: bool) -> bool {
        // Global matchers always apply (ignore files, `!` excludes)
        if check_ignored_by(&self.global, path, is_dir, check_ancestors) {
            return true;
        }

        // Scoped matchers: deepest nested wins, root only outside nested scope
        for (ignore, root) in &self.nested_scoped {
            if path.starts_with(root) {
                return check_ignored_by_single(ignore.as_ref(), path, is_dir, check_ancestors);
            }
        }
        check_ignored_by_single(self.base_scoped.as_ref(), path, is_dir, check_ancestors)
    }
}

/// Check if any matcher in the list ignores the path.
fn check_ignored_by(
    matchers: &[Gitignore],
    path: &Path,
    is_dir: bool,
    check_ancestors: bool,
) -> bool {
    for matcher in matchers {
        let matched = if check_ancestors {
            if !path.starts_with(matcher.path()) {
                continue;
            }
            matcher.matched_path_or_any_parents(path, is_dir)
        } else {
            matcher.matched(path, is_dir)
        };
        if matched.is_ignore() && !matched.is_whitelist() {
            return true;
        }
    }
    false
}

/// Check if a single optional matcher ignores the path.
fn check_ignored_by_single(
    matcher: Option<&Gitignore>,
    path: &Path,
    is_dir: bool,
    check_ancestors: bool,
) -> bool {
    matcher.is_some_and(|gi| {
        if check_ancestors {
            gi.matched_path_or_any_parents(path, is_dir).is_ignore()
        } else {
            let m = gi.matched(path, is_dir);
            m.is_ignore() && !m.is_whitelist()
        }
    })
}

/// Check if a path string looks like a glob pattern.
/// Glob-like characters are also valid path characters on some environments.
/// If the path actually exists on disk, it is treated as a concrete path.
/// e.g. `{config}.js`, `[id].tsx`
fn is_glob_pattern(s: &str, cwd: &Path) -> bool {
    let has_glob_chars = s.contains('*') || s.contains('?') || s.contains('[') || s.contains('{');
    has_glob_chars && !cwd.join(s).exists()
}

// NOTE: Why pre-expand globs?
// An alternative approach would be:
// - to always walk the entire `cwd`
// - and filter by both concrete paths and glob patterns
//
// However, this would be inefficient for common use cases
// like `oxfmt src/a.js` or pre-commit hooks that specify only staged files.
//
// Pre-expanding globs allows us to walk only the necessary paths.
// And this only happens if glob patterns are specified.
//
// NOTE: Why not use `ignore::Overrides` in the main walk?
// `ignore::Overrides` have the highest priority in the `ignore` crate,
// so files matching the glob would be collected even if they're in `.gitignore`!
/// Expand glob patterns to concrete file paths.
fn expand_glob_patterns(cwd: &Path, patterns: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut ob = OverrideBuilder::new(cwd);
    for pattern in patterns {
        ob.add(pattern).map_err(|e| format!("Invalid glob pattern `{pattern}`: {e}"))?;
    }
    let overrides = ob.build().map_err(|e| format!("Failed to build glob overrides: {e}"))?;

    let mut builder = ignore::WalkBuilder::new(cwd);
    builder.overrides(overrides);

    let paths = Mutex::new(vec![]);
    apply_walk_settings(&mut builder).build_parallel().run(|| {
        Box::new(|entry| {
            match entry {
                Ok(entry) => {
                    // Align with main walk: only include files
                    #[expect(clippy::filetype_is_file)]
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        paths.lock().unwrap().push(entry.into_path());
                    }
                    ignore::WalkState::Continue
                }
                Err(_err) => ignore::WalkState::Skip,
            }
        })
    });

    Ok(paths.into_inner().unwrap())
}

fn load_ignore_paths(cwd: &Path, ignore_paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    // If specified, resolve absolute paths and check existence
    if !ignore_paths.is_empty() {
        let mut result = Vec::with_capacity(ignore_paths.len());
        for path in ignore_paths {
            let path = normalize_relative_path(cwd, path);
            if !path.exists() {
                return Err(format!("{}: File not found", path.display()));
            }
            result.push(path);
        }
        return Ok(result);
    }

    // Else, search for default ignore files in cwd
    // These are optional, do not error if not found
    Ok(std::iter::once(".prettierignore")
        .filter_map(|file_name| {
            let path = cwd.join(file_name);
            path.exists().then_some(path)
        })
        .collect())
}

/// Apply common walk settings.
/// This ensures consistent behavior across glob expansion and main walk.
fn apply_walk_settings(builder: &mut ignore::WalkBuilder) -> &mut ignore::WalkBuilder {
    builder
        // Do not follow symlinks like Prettier does.
        // See https://github.com/prettier/prettier/pull/14627
        .follow_links(false)
        // Include hidden files and directories except those we explicitly skip
        .hidden(false)
        // Do not respect `.ignore` file
        .ignore(false)
        // Do not search upward
        // NOTE: Prettier only searches current working directory
        .parents(false)
        // Also do not respect globals
        .git_global(false)
        // But respect downward nested `.gitignore` files
        // NOTE: Prettier does not: https://github.com/prettier/prettier/issues/4081
        .git_ignore(true)
        // Also do not respect `.git/info/exclude`
        .git_exclude(false)
        // Git is not required
        .require_git(false)
}

// ---

struct WalkBuilder {
    sender: mpsc::Sender<FormatFileStrategy>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor { sender: self.sender.clone() })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<FormatFileStrategy>,
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                let Some(file_type) = entry.file_type() else {
                    return ignore::WalkState::Continue;
                };

                // Use `is_file()` to detect symlinks to the directory named `.js`
                #[expect(clippy::filetype_is_file)]
                if file_type.is_file() {
                    // Determine this file should be handled or NOT
                    // Tier 1 = `.js`, `.tsx`, etc: JS/TS files supported by `oxc_formatter`
                    // Tier 2 = `.toml`, etc: Some files supported by `oxfmt` directly
                    // Tier 3 = `.html`, `.json`, etc: Other files supported by Prettier
                    // (Tier 4 = `.astro`, `.svelte`, etc: Other files supported by Prettier plugins)
                    // Everything else: Ignored
                    let Ok(strategy) = FormatFileStrategy::try_from(entry.into_path()) else {
                        return ignore::WalkState::Continue;
                    };

                    #[cfg(not(feature = "napi"))]
                    if !strategy.can_format_without_external() {
                        return ignore::WalkState::Continue;
                    }

                    // Send each entry immediately through the channel
                    // If send fails, the receiver has been dropped, so stop walking
                    if self.sender.send(strategy).is_err() {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
