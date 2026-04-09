use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use fast_glob::glob_match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use rustc_hash::FxHashSet;

use crate::core::{FormatFileStrategy, utils::normalize_relative_path};

pub struct Walk {
    inner: Option<ignore::WalkParallel>,
    glob_matcher: Option<Arc<GlobMatcher>>,
}

impl Walk {
    pub fn build(
        cwd: &Path,
        paths: &[PathBuf],
        ignore_paths: &[PathBuf],
        with_node_modules: bool,
        config_dir: Option<&Path>,
        ignore_patterns: &[String],
    ) -> Result<Self, String> {
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

            // Normalize `./` prefix (and any consecutive slashes, e.g. `.//src/app.js`)
            let normalized = if let Some(stripped) = path_str.strip_prefix("./") {
                stripped.trim_start_matches('/')
            } else {
                &path_str
            };

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

        // When glob patterns exist, walk from cwd to find matching files during traversal.
        // Concrete file paths are still added individually as base paths.
        if !glob_patterns.is_empty() {
            target_paths.insert(cwd.to_path_buf());
        }

        // Default to `cwd` if no positive paths were specified.
        // Exclude patterns alone should still walk, but unmatched globs should not.
        if target_paths.is_empty() && glob_patterns.is_empty() {
            target_paths.insert(cwd.to_path_buf());
        }

        //
        // Build ignores
        //
        // Use multiple matchers, each with correct root for pattern resolution:
        // - Ignore files: root = parent directory of the ignore file
        // - `.ignorePatterns`: root = parent directory of config file
        // - Exclude paths (`!` prefix): root = cwd
        //
        // NOTE: Git ignore files are handled by `WalkBuilder` itself
        let mut matchers: Vec<Gitignore> = vec![];

        // 1. Handle formatter ignore files (`.prettierignore`, or `--ignore-path`)
        // Patterns are relative to the ignore file location
        for ignore_path in &load_ignore_paths(cwd, ignore_paths)? {
            let (gitignore, err) = Gitignore::new(ignore_path);
            if let Some(err) = err {
                return Err(format!(
                    "Failed to parse ignore file {}: {err}",
                    ignore_path.display()
                ));
            }
            matchers.push(gitignore);
        }

        // 2. Handle `oxfmtrc.ignorePatterns`
        // Patterns are relative to the config file location
        if !ignore_patterns.is_empty()
            && let Some(config_dir) = config_dir
        {
            let mut builder = GitignoreBuilder::new(config_dir);
            for pattern in ignore_patterns {
                if builder.add_line(None, pattern).is_err() {
                    return Err(format!(
                        "Failed to add ignore pattern `{pattern}` from `.ignorePatterns`"
                    ));
                }
            }
            let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
            matchers.push(gitignore);
        }

        // 3. Handle `!` prefixed paths
        // These are relative to cwd
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
            matchers.push(gitignore);
        }

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
            .filter(|path| !is_ignored(&matchers, path, path.is_dir(), true))
            .collect();

        // If no target paths remain after filtering, return an empty walker.
        // Not an error, but nothing to format, leave it to the caller how to handle.
        let Some(first_path) = target_paths.first() else {
            return Ok(Self { inner: None, glob_matcher: None });
        };

        // Build the glob matcher for walk-time filtering.
        // When glob patterns exist, files are matched against them during `visit()`.
        // When no globs, `visit()` has zero overhead.
        let glob_matcher = if glob_patterns.is_empty() {
            None
        } else {
            Some(Arc::new(GlobMatcher::new(cwd.to_path_buf(), glob_patterns, &target_paths)))
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
                if is_ignored_dir(entry.file_name(), with_node_modules) {
                    return false;
                }
            }

            // Check ignore files, patterns
            if is_ignored(&matchers, entry.path(), is_dir, false) {
                return false;
            }

            // NOTE: Glob pattern matching is NOT done here in `filter_entry()`.
            // Glob patterns like `**/*.js` cannot be used to skip directories,
            // since any directory could contain matching files at any depth.
            // Glob filtering is instead done per-file in the visitor `visit()` below.
            //
            // In addition to ignoring based on ignore files and patterns here,
            // we also apply extra filtering in `visit()`.
            // We need to return `bool` for `filter_entry()` here,
            // but we don't want to duplicate logic in the visitor again.
            true
        });

        let inner = inner
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
            .build_parallel();

        Ok(Self { inner: Some(inner), glob_matcher })
    }

    /// Stream entries through a channel as they are discovered.
    /// If no target paths remain (empty walker), returns an immediately-closed channel.
    pub fn stream_entries(self) -> mpsc::Receiver<FormatFileStrategy> {
        let (sender, receiver) = mpsc::channel::<FormatFileStrategy>();

        if let Some(inner) = self.inner {
            // Spawn the walk operation in a separate thread
            rayon::spawn(move || {
                let mut builder = WalkVisitorBuilder { sender, glob_matcher: self.glob_matcher };
                inner.visit(&mut builder);
                // Channel will be closed when builder is dropped
            });
        }
        // else: sender is dropped here, receiver will yield nothing

        receiver
    }
}

// ---

/// Check if a path should be ignored by any of the matchers.
/// A path is ignored if any matcher says it's ignored (and not whitelisted in that same matcher).
///
/// When `check_ancestors: true`, also checks if any parent directory is ignored.
/// This is more expensive, but necessary when paths (to be ignored) are passed directly via CLI arguments.
/// For normal walking, walk is done in a top-down manner, so only the current path needs to be checked.
fn is_ignored(matchers: &[Gitignore], path: &Path, is_dir: bool, check_ancestors: bool) -> bool {
    for matcher in matchers {
        let matched = if check_ancestors {
            // `matched_path_or_any_parents()` panics if path is not under matcher's root.
            // Skip this matcher if the path is outside its scope.
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

/// Check if a directory should be skipped during walking.
/// VCS internal directories are always skipped, and `node_modules` is skipped by default.
fn is_ignored_dir(dir_name: &OsStr, with_node_modules: bool) -> bool {
    dir_name == ".git"
        || dir_name == ".jj"
        || dir_name == ".sl"
        || dir_name == ".svn"
        || dir_name == ".hg"
        || (!with_node_modules && dir_name == "node_modules")
}

/// Check if a path string looks like a glob pattern.
/// Glob-like characters are also valid path characters on some environments.
/// If the path actually exists on disk, it is treated as a concrete path.
/// e.g. `{config}.js`, `[id].tsx`
fn is_glob_pattern(s: &str, cwd: &Path) -> bool {
    let has_glob_chars = s.contains('*') || s.contains('?') || s.contains('[') || s.contains('{');
    has_glob_chars && !cwd.join(s).exists()
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

// ---

/// Matches file paths against glob patterns during walk.
///
/// When glob patterns are specified via CLI args,
/// files are matched against them during the walk's `visit()` callback.
///
/// Uses `fast_glob::glob_match` instead of `ignore::Overrides` because
/// overrides have the highest priority in the `ignore` crate and would bypass `.gitignore` rules.
///
/// Also tracks concrete target paths (non-glob) because when globs are present,
/// cwd is added as a base path, which means concrete paths can be visited twice.
/// (as direct base paths and during the cwd walk)
/// This struct handles both acceptance and dedup of those paths via `matches()`.
struct GlobMatcher {
    /// cwd for computing relative paths for glob matching.
    cwd: PathBuf,
    /// Normalized glob pattern strings for matching via `fast_glob::glob_match`.
    glob_patterns: Vec<String>,
    /// Concrete target paths (absolute) specified via CLI.
    /// These are always accepted even when glob filtering is active.
    concrete_paths: FxHashSet<PathBuf>,
    /// Tracks seen concrete paths to avoid duplicates (visited both as
    /// direct base paths and via cwd walk).
    seen: Mutex<FxHashSet<PathBuf>>,
}

impl GlobMatcher {
    fn new(cwd: PathBuf, glob_patterns: Vec<String>, target_paths: &[PathBuf]) -> Self {
        // Normalize glob patterns: patterns without `/` are prefixed with `**/`
        // to match at any depth (gitignore/prettier semantics).
        // e.g., `*.js` → `**/*.js`, `foo/**/*.js` stays as-is.
        let glob_patterns = glob_patterns
            .into_iter()
            .map(|pat| if pat.contains('/') { pat } else { format!("**/{pat}") })
            .collect();
        // Store concrete paths (excluding cwd itself) for dedup and acceptance.
        let concrete_paths = target_paths.iter().filter(|p| p.as_path() != cwd).cloned().collect();
        Self { cwd, glob_patterns, concrete_paths, seen: Mutex::new(FxHashSet::default()) }
    }

    /// Returns `true` if the path matches any glob pattern or is a concrete target path.
    /// Concrete paths are deduplicated (they can appear both as direct base paths and via cwd walk).
    fn matches(&self, path: &Path) -> bool {
        // Accept concrete paths (explicitly specified via CLI), with dedup
        if self.concrete_paths.contains(path) {
            return self.seen.lock().unwrap().insert(path.to_path_buf());
        }

        // Match against glob patterns using cwd-relative path
        let relative = path.strip_prefix(&self.cwd).unwrap_or(path).to_string_lossy();
        self.glob_patterns.iter().any(|pattern| glob_match(pattern, relative.as_ref()))
    }
}

// ---

struct WalkVisitorBuilder {
    sender: mpsc::Sender<FormatFileStrategy>,
    glob_matcher: Option<Arc<GlobMatcher>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkVisitorBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor {
            sender: self.sender.clone(),
            glob_matcher: self.glob_matcher.clone(),
        })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<FormatFileStrategy>,
    glob_matcher: Option<Arc<GlobMatcher>>,
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
                    let path = entry.into_path();

                    // When glob matcher is active,
                    // only accept files that match glob patterns or explicitly specified concrete paths.
                    if let Some(glob_matcher) = &self.glob_matcher
                        && !glob_matcher.matches(&path)
                    {
                        return ignore::WalkState::Continue;
                    }

                    // Otherwise, all files are specified as base paths

                    // Tier 1 = `.js`, `.tsx`, etc: JS/TS files supported by `oxc_formatter`
                    // Tier 2 = `.toml`, etc: Some files supported by `oxfmt` directly
                    // Tier 3 = `.html`, `.json`, etc: Other files supported by Prettier
                    // (Tier 4 = `.astro`, `.svelte`, etc: Other files supported by Prettier plugins)
                    // Everything else: Ignored
                    let Ok(strategy) = FormatFileStrategy::try_from(path) else {
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
