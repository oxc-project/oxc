use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use fast_glob::glob_match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use rustc_hash::FxHashSet;
use tracing::instrument;

#[cfg(feature = "napi")]
use crate::core::JsConfigLoaderCb;
use crate::core::{
    ConfigResolver, FormatFileStrategy, all_config_file_names, utils::normalize_relative_path,
};

/// A file entry paired with its scope's config resolver.
pub struct FormatEntry {
    pub strategy: FormatFileStrategy,
    pub config_resolver: Arc<ConfigResolver>,
}

/// Orchestrates scoped walks with nested config detection.
///
/// Walks from base paths, detects directories containing config files,
/// splits them into child scopes, and recursively walks each scope
/// with its own config.
///
/// Constructed from CLI path arguments, which are classified into
/// target paths, glob patterns, and exclude patterns.
pub struct ScopedWalker {
    cwd: PathBuf,
    paths: Vec<PathBuf>,
    glob_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl ScopedWalker {
    /// Create a new `ScopedWalker` by classifying CLI path arguments.
    ///
    /// Paths are split into target paths, glob patterns, and exclude patterns (`!` prefix).
    pub fn new(cwd: PathBuf, paths: &[PathBuf]) -> Self {
        let mut target_paths = vec![];
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

            if is_glob_pattern(normalized, &cwd) {
                glob_patterns.push(normalized.to_string());
                continue;
            }

            let full_path = if path.is_absolute() {
                path.clone()
            } else if normalized == "." {
                // NOTE: `.` and cwd behave differently, need to normalize
                cwd.clone()
            } else {
                cwd.join(normalized)
            };
            target_paths.push(full_path);
        }

        Self { cwd, paths: target_paths, glob_patterns, exclude_patterns }
    }

    /// Run the root scope walk and all nested scopes.
    ///
    /// Returns `Ok(true)` if any config was found across all scopes.
    ///
    /// `detect_nested` should be `false` when `--config` is explicitly specified.
    #[instrument(level = "debug", name = "oxfmt::walk::run", skip_all)]
    pub fn run(
        &self,
        root_config: ConfigResolver,
        ignore_paths: &[PathBuf],
        ignore_patterns: &[String],
        with_node_modules: bool,
        detect_nested: bool,
        editorconfig_path: Option<&Path>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
        sender: &mpsc::Sender<FormatEntry>,
    ) -> Result<bool, String> {
        let config_resolver = Arc::new(root_config);

        let config_candidates: Vec<String> =
            if detect_nested { all_config_file_names().collect() } else { vec![] };

        // Build ignore matchers for the root scope
        let matchers = build_root_ignore_matchers(
            &self.cwd,
            ignore_paths,
            config_resolver.config_dir(),
            ignore_patterns,
            &self.exclude_patterns,
        )?;

        // Resolve root target paths
        let mut target_paths: FxHashSet<PathBuf> = self.paths.iter().cloned().collect();

        // When glob patterns exist, walk from cwd to find matching files during traversal.
        // Concrete file paths are still added individually as base paths.
        if !self.glob_patterns.is_empty() {
            target_paths.insert(self.cwd.clone());
        }

        // Default to `cwd` if no positive paths were specified.
        // Exclude patterns alone should still walk, but unmatched globs should not.
        if target_paths.is_empty() && self.glob_patterns.is_empty() {
            target_paths.insert(self.cwd.clone());
        }

        // Base paths passed to `WalkBuilder` are not filtered by `filter_entry()`,
        // so we need to filter them here before passing to the walker.
        // This is needed for cases like `husky`, may specify ignored paths as staged files.
        // NOTE: Git ignored paths are not filtered here.
        // But it's OK because in cases like `husky`, they are never staged.
        let target_paths: Vec<_> = target_paths
            .into_iter()
            .filter(|path| !is_ignored(&matchers, path, path.is_dir(), true))
            .collect();

        // Build the glob matcher once for walk-time filtering.
        // When glob patterns exist, files are matched against them during `visit()`.
        // When no globs, `visit()` has zero overhead.
        let glob_matcher = (!self.glob_patterns.is_empty()).then(|| {
            Arc::new(GlobMatcher::new(self.cwd.clone(), self.glob_patterns.clone(), &target_paths))
        });

        let child_scopes = walk_and_stream(
            &self.cwd,
            &target_paths,
            matchers,
            with_node_modules,
            &config_candidates,
            glob_matcher.as_ref(),
            &config_resolver,
            sender,
        );

        let mut any_config = config_resolver.config_dir().is_some();
        for dir in child_scopes {
            if run_child_scope(
                &dir,
                with_node_modules,
                &config_candidates,
                glob_matcher.as_ref(),
                editorconfig_path,
                #[cfg(feature = "napi")]
                js_config_loader,
                sender,
            )? {
                any_config = true;
            }
        }

        Ok(any_config)
    }
}

// ---

/// Run a child scope walk. Returns whether this scope had a config.
#[instrument(level = "debug", name = "oxfmt::walk::run_child_scope", skip_all, fields(scope = %scope_dir.display()))]
fn run_child_scope(
    scope_dir: &Path,
    with_node_modules: bool,
    config_candidates: &[String],
    glob_matcher: Option<&Arc<GlobMatcher>>,
    editorconfig_path: Option<&Path>,
    #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    sender: &mpsc::Sender<FormatEntry>,
) -> Result<bool, String> {
    let mut config_resolver = ConfigResolver::from_config(
        scope_dir,
        // NOTE: Let it auto-discovery (not a direct path) config,
        // so that `discover_config()` can skip invalid candidates
        // (e.g. `vite.config.ts` without a `.fmt` field)
        // and continue searching upward.
        None,
        editorconfig_path,
        #[cfg(feature = "napi")]
        js_config_loader,
    )
    .map_err(|err| format!("Failed to load config in {}: {err}", scope_dir.display()))?;
    let ignore_patterns = config_resolver
        .build_and_validate()
        .map_err(|err| format!("Failed to parse config in {}: {err}", scope_dir.display()))?;

    let has_config = config_resolver.config_dir().is_some();

    let config_resolver = Arc::new(config_resolver);
    let matchers = build_ignore_patterns_matcher(scope_dir, &ignore_patterns)?;

    let child_scopes = walk_and_stream(
        scope_dir,
        &[scope_dir.to_path_buf()],
        matchers,
        with_node_modules,
        config_candidates,
        glob_matcher,
        &config_resolver,
        sender,
    );

    // Recurse into further nested scopes
    let mut any_config = has_config;
    for dir in child_scopes {
        if run_child_scope(
            &dir,
            with_node_modules,
            config_candidates,
            glob_matcher,
            editorconfig_path,
            #[cfg(feature = "napi")]
            js_config_loader,
            sender,
        )? {
            any_config = true;
        }
    }

    Ok(any_config)
}

/// Build a Walk, stream entries to the shared channel, and return child config dirs.
fn walk_and_stream(
    scope_dir: &Path,
    target_paths: &[PathBuf],
    matchers: Vec<Gitignore>,
    with_node_modules: bool,
    config_candidates: &[String],
    glob_matcher: Option<&Arc<GlobMatcher>>,
    config_resolver: &Arc<ConfigResolver>,
    sender: &mpsc::Sender<FormatEntry>,
) -> Vec<PathBuf> {
    let Some(first_path) = target_paths.first() else {
        return vec![];
    };

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

        // We are setting `.hidden(false)` on the `WalkBuilder` below,
        // it means we want to include hidden files and directories.
        // However, we (and also Prettier) still skip traversing certain directories.
        // https://prettier.io/docs/ignore#ignoring-files-prettierignore
        if is_dir && is_ignored_dir(entry.file_name(), with_node_modules) {
            return false;
        }
        if is_ignored(&matchers, entry.path(), is_dir, false) {
            return false;
        }

        // NOTE: Glob pattern matching is NOT done here in `filter_entry()`.
        // Glob patterns like `**/*.js` cannot be used to skip directories,
        // since any directory could contain matching files at any depth.
        // Glob filtering is instead done per-file in the visitor `visit()` below.
        true
    });

    let child_config_dirs: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(vec![]));

    let mut builder = WalkVisitorBuilder {
        sender: sender.clone(),
        config_resolver: Arc::clone(config_resolver),
        glob_matcher: glob_matcher.cloned(),
        scope_dir: scope_dir.to_path_buf(),
        config_candidates: config_candidates.to_vec(),
        child_config_dirs: Arc::clone(&child_config_dirs),
    };

    // Use the same thread count as rayon (controlled by `--threads`)
    let num_of_threads = rayon::current_num_threads();
    inner
        .threads(num_of_threads)
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
        .build_parallel()
        .visit(&mut builder);

    std::mem::take(&mut *child_config_dirs.lock().unwrap())
}

// ---

/// Build ignore matchers for the root scope from all sources:
/// ignore files, config ignorePatterns, and CLI `!` exclude patterns.
///
/// Each matcher has its own root for pattern resolution:
/// ignore files use their parent dir, ignorePatterns use config dir, excludes use `cwd`.
/// Git ignore files are handled by `WalkBuilder` itself.
fn build_root_ignore_matchers(
    cwd: &Path,
    ignore_paths: &[PathBuf],
    config_dir: Option<&Path>,
    ignore_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<Vec<Gitignore>, String> {
    let mut matchers: Vec<Gitignore> = vec![];

    // 1. Formatter ignore files (.prettierignore, --ignore-path)
    // Paths are already resolved and validated by `resolve_ignore_paths()`
    for ignore_path in ignore_paths {
        let (gitignore, err) = Gitignore::new(ignore_path);
        if let Some(err) = err {
            return Err(format!("Failed to parse ignore file {}: {err}", ignore_path.display()));
        }
        matchers.push(gitignore);
    }

    // 2. oxfmtrc.ignorePatterns (relative to config file location)
    if let Some(config_dir) = config_dir {
        matchers.extend(build_ignore_patterns_matcher(config_dir, ignore_patterns)?);
    }

    // 3. `!` prefixed paths (CLI excludes, relative to cwd)
    if !exclude_patterns.is_empty() {
        let mut builder = GitignoreBuilder::new(cwd);
        for pattern in exclude_patterns {
            // Remove the leading `!` because `GitignoreBuilder` uses `!` as negation
            let pattern =
                pattern.strip_prefix('!').expect("There should be a `!` prefix, already checked");
            if builder.add_line(None, pattern).is_err() {
                return Err(format!("Failed to add ignore pattern `{pattern}` from `!` prefix"));
            }
        }
        let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
        matchers.push(gitignore);
    }

    Ok(matchers)
}

/// Build ignore matchers from config `ignorePatterns`.
/// Patterns are resolved relative to the config file's directory.
fn build_ignore_patterns_matcher(
    config_dir: &Path,
    ignore_patterns: &[String],
) -> Result<Vec<Gitignore>, String> {
    if ignore_patterns.is_empty() {
        return Ok(vec![]);
    }

    let mut builder = GitignoreBuilder::new(config_dir);
    for pattern in ignore_patterns {
        if builder.add_line(None, pattern).is_err() {
            return Err(format!("Failed to add ignore pattern `{pattern}` from `.ignorePatterns`"));
        }
    }
    let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;

    Ok(vec![gitignore])
}

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

/// Resolve ignore file paths from CLI args or defaults.
///
/// Called early (before walk) to validate that specified ignore files exist.
pub fn resolve_ignore_paths(cwd: &Path, ignore_paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
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

    // Default: search for .prettierignore in cwd
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

        let relative = path.strip_prefix(&self.cwd).unwrap_or(path).to_string_lossy();
        self.glob_patterns.iter().any(|pattern| glob_match(pattern, relative.as_ref()))
    }
}

// ---

struct WalkVisitorBuilder {
    sender: mpsc::Sender<FormatEntry>,
    config_resolver: Arc<ConfigResolver>,
    glob_matcher: Option<Arc<GlobMatcher>>,
    scope_dir: PathBuf,
    config_candidates: Vec<String>,
    child_config_dirs: Arc<Mutex<Vec<PathBuf>>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkVisitorBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor {
            sender: self.sender.clone(),
            config_resolver: Arc::clone(&self.config_resolver),
            glob_matcher: self.glob_matcher.clone(),
            scope_dir: self.scope_dir.clone(),
            config_candidates: self.config_candidates.clone(),
            child_config_dirs: Arc::clone(&self.child_config_dirs),
        })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<FormatEntry>,
    config_resolver: Arc<ConfigResolver>,
    glob_matcher: Option<Arc<GlobMatcher>>,
    scope_dir: PathBuf,
    config_candidates: Vec<String>,
    child_config_dirs: Arc<Mutex<Vec<PathBuf>>>,
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                let Some(file_type) = entry.file_type() else {
                    return ignore::WalkState::Continue;
                };

                // If nested config files in subdirectories is detected
                if file_type.is_dir() {
                    if !self.config_candidates.is_empty() && entry.path() != self.scope_dir {
                        let dir_path = entry.path();
                        if self.config_candidates.iter().any(|name| dir_path.join(name).exists()) {
                            self.child_config_dirs.lock().unwrap().push(dir_path.to_path_buf());
                            // Skip walking into this directory, it will be walked separately as a child scope
                            return ignore::WalkState::Skip;
                        }
                    }
                    return ignore::WalkState::Continue;
                }

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

                    let entry = FormatEntry {
                        strategy,
                        config_resolver: Arc::clone(&self.config_resolver),
                    };
                    if self.sender.send(entry).is_err() {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
