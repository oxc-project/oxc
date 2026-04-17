use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use fast_glob::glob_match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::instrument;

#[cfg(feature = "napi")]
use crate::core::JsConfigLoaderCb;
use crate::core::{
    ConfigResolver, FormatFileStrategy, has_config_in_directory, utils::normalize_relative_path,
};

/// A file entry paired with its scope's config resolver.
pub struct FormatEntry {
    pub strategy: FormatFileStrategy,
    pub config_resolver: Arc<ConfigResolver>,
}

/// Orchestrates file discovery with nested config and ignore handling.
///
/// Constructed from CLI path arguments, which are classified into
/// target paths, glob patterns, and exclude patterns.
///
/// # Config resolution
/// Each file is formatted by its nearest config (auto-discovered upward).
/// Directories containing a config file form a scope boundary.
/// There is no inheritance between parent and child scopes.
///
/// When `--config` is explicitly given, nested detection is disabled entirely.
///
/// # Walk phases
/// - Phase 1: Classify CLI positional PATHs into directory targets, file targets, and globs
/// - Phase 2: File targets are processed directly (no walk)
///   - Scope is resolved upward from the file's parent, O(depth) per unique parent
///   - This helps with performance when many file targets are specified like with `husky`
///     - Processed even when nested detection is disabled (no pre-scan, no child scopes loaded)
/// - Phase 3: Directory targets are walked via a flat single-walk architecture
///   - 3-1. Pre-scan (sequential): walk directories to discover nested config file locations
///     - Required to build the scope map before streaming walk begins (see `prescan_config_locations()`)
///     - Cost: O(directories), skipped when `--config` is given
///   - 3-2. Load child scopes: load all discovered configs into a scope map
///   - 3-3. Build ancestor set (only when any scope has `ignorePatterns`):
///     - Records which directories have a config descendant, so `filter_entry()`
///       can safely skip `ignorePatterns` matches without hiding nested configs
///   - 3-4. Parallel walk: a single `WalkBuilder` walk processes all files
///
/// # Ignore model
/// Three layers, checked in `filter_entry()` (directories) and `visit()` (files).
///
/// - (1) Hardcoded skips: `.git`, `.svn`, `node_modules`, etc
///   - Always skipped in `filter_entry()`, cannot be overridden by ignore files or patterns
/// - (2) Global ignores: `.prettierignore`, `--ignore-path`, CLI `"!path"`
///   - Block both directories and files across all scopes
/// - (3) Scope-local `ignorePatterns`: each scope's config can define patterns
///   - Directories (`filter_entry`): scope is resolved per-directory, then that scope's `ignorePatterns` are checked
///     - The directory is skipped only when the pre-scan ancestor set confirms no config descendant exists
///   - Files (`visit`): scope is resolved per-parent-directory (cached), then that scope's `ignorePatterns` are checked
///     - This handles file-specific patterns (e.g. `*.generated.js`) and files inside directories that couldn't be skipped (config descendants).
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

    /// Run the walk across all scopes.
    /// And stream file to be formatted with its resolved config via the shared channel.
    ///
    /// Returns `Ok(true)` if any valid config was used.
    #[instrument(level = "debug", name = "oxfmt::walk::run", skip_all)]
    pub fn run(
        &self,
        root_config: ConfigResolver,
        ignore_paths: &[PathBuf],
        with_node_modules: bool,
        detect_nested: bool,
        editorconfig_path: Option<&Path>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
        sender: &mpsc::Sender<FormatEntry>,
    ) -> Result<bool, String> {
        let root_config_resolver = Arc::new(root_config);

        // Global ignores: .prettierignore, --ignore-path, CLI `!` patterns
        let ignore_file_matchers: Arc<[Gitignore]> = Arc::from(build_global_ignore_matchers(
            &self.cwd,
            &self.exclude_patterns,
            ignore_paths,
        )?);

        let mut any_config = root_config_resolver.config_dir().is_some();

        // Phase 1: Classify targets into directories (walk) and files (direct)
        let (walk_targets, file_targets) = {
            let mut initial_targets: FxHashSet<PathBuf> = self.paths.iter().cloned().collect();

            // When glob patterns exist, walk from cwd to find matching files during traversal.
            // Concrete file paths are still added individually as base paths.
            if !self.glob_patterns.is_empty() {
                initial_targets.insert(self.cwd.clone());
            }

            // Default to `cwd` if no positive paths were specified.
            // Exclude patterns alone should still walk, but unmatched globs should not.
            if initial_targets.is_empty() && self.glob_patterns.is_empty() {
                initial_targets.insert(self.cwd.clone());
            }

            let mut dirs = vec![];
            let mut files = vec![];
            for path in &initial_targets {
                // Base paths passed to `WalkBuilder` are not filtered by `filter_entry()`,
                // so we need to filter them here before passing to the walker.
                // This is needed for cases like `husky`, may specify ignored paths as staged files.
                // NOTE: Git ignored paths are not filtered here.
                // But it's OK because in cases like `husky`, they are never staged.
                if is_ignored(&ignore_file_matchers, path, path.is_dir(), true) {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path.clone());
                } else {
                    files.push(path.clone());
                }
            }
            (dirs, files)
        };

        // Phase 2: Process file targets directly (no walk needed)
        let mut directly_processed: FxHashSet<PathBuf> = FxHashSet::default();
        if !file_targets.is_empty() {
            // Cache scope resolution results by parent directory to avoid redundant lookups
            type ScopeEntry = Option<Arc<ConfigResolver>>;

            let mut scope_cache: FxHashMap<&Path, ScopeEntry> = FxHashMap::default();
            for file in &file_targets {
                // Skip non-existent files (WalkBuilder naturally skips these via error entries)
                if !file.is_file() {
                    continue;
                }

                // Resolve which scope (config) this file belongs to
                let file_config = if detect_nested {
                    let parent = file.parent().unwrap();
                    if !scope_cache.contains_key(parent) {
                        let cached = resolve_file_scope_config(
                            file,
                            root_config_resolver.config_dir(),
                            editorconfig_path,
                            #[cfg(feature = "napi")]
                            js_config_loader,
                        )?
                        .map(Arc::from);
                        scope_cache.insert(parent, cached);
                    }

                    match &scope_cache[parent] {
                        Some(resolver) => {
                            any_config = true;
                            Arc::clone(resolver)
                        }
                        None => Arc::clone(&root_config_resolver),
                    }
                } else {
                    Arc::clone(&root_config_resolver)
                };

                if file_config.is_path_ignored(file, false) {
                    continue;
                }
                let Ok(strategy) = FormatFileStrategy::try_from(file.clone()) else {
                    continue;
                };
                #[cfg(not(feature = "napi"))]
                if !strategy.can_format_without_external() {
                    continue;
                }

                directly_processed.insert(file.clone());
                if sender.send(FormatEntry { strategy, config_resolver: file_config }).is_err() {
                    break;
                }
            }
        }

        // Phase 3: Walk directory targets
        let directly_processed: Arc<FxHashSet<PathBuf>> = Arc::new(directly_processed);

        // Build the glob matcher once for walk-time filtering.
        // When glob patterns exist, files are matched against them during `visit()`.
        // When no globs, `visit()` has zero overhead.
        let glob_matcher = (!self.glob_patterns.is_empty()).then(|| {
            Arc::new(GlobMatcher::new(self.cwd.clone(), self.glob_patterns.clone(), &walk_targets))
        });

        // Pre-scan: discover nested config file locations
        let config_dirs = if detect_nested {
            prescan_config_locations(&walk_targets, &ignore_file_matchers, with_node_modules)
        } else {
            vec![]
        };

        let (child_scope_map, config_ancestors) = if config_dirs.is_empty() {
            // No nested configs, = only root scope
            (Arc::new(FxHashMap::default()), None)
        } else {
            // Load all child configs upfront into a scope map
            let (map, has_children) = resolve_child_scope_configs(
                &config_dirs,
                root_config_resolver.config_dir(),
                editorconfig_path,
                #[cfg(feature = "napi")]
                js_config_loader,
            )?;
            if has_children {
                any_config = true;
            }

            // Build ancestor set for directory-level ignorePatterns skipping,
            // only when any scope (root or child) has ignorePatterns.
            let ancestors = if root_config_resolver.has_ignore_patterns()
                || map.values().any(|r| r.has_ignore_patterns())
            {
                Some(build_config_ancestors(&config_dirs))
            } else {
                None
            };

            (Arc::new(map), ancestors)
        };

        walk_and_stream(
            &walk_targets,
            Arc::clone(&ignore_file_matchers),
            with_node_modules,
            glob_matcher.as_ref(),
            &root_config_resolver,
            &directly_processed,
            config_ancestors.as_ref(),
            &child_scope_map,
            sender,
        );

        Ok(any_config)
    }
}

// ---

/// Build global ignore matchers from ignore files and CLI exclude patterns.
///
/// These are scope-independent and block both files and directories across all scopes.
/// Each matcher has its own root for pattern resolution:
/// - ignore files use their parent dir
/// - excludes use `cwd`
///
/// Git ignore files are handled by `WalkBuilder` itself.
fn build_global_ignore_matchers(
    cwd: &Path,
    exclude_patterns: &[String],
    ignore_paths: &[PathBuf],
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

    // 2. `!` prefixed paths (CLI excludes, relative to cwd)
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

/// Resolve the nearest config scope for a file target.
/// Returns `None` if the file belongs to the root scope.
fn resolve_file_scope_config(
    file: &Path,
    root_config_dir: Option<&Path>,
    editorconfig_path: Option<&Path>,
    #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
) -> Result<Option<ConfigResolver>, String> {
    let Some(parent) = file.parent() else {
        return Ok(None);
    };

    let mut resolver = ConfigResolver::from_config(
        parent,
        None,
        editorconfig_path,
        #[cfg(feature = "napi")]
        js_config_loader,
    )
    .map_err(|err| format!("Failed to load config for {}: {err}", file.display()))?;

    // No config found, or same as root → belongs to root scope
    if resolver.config_dir().is_none() || resolver.config_dir() == root_config_dir {
        return Ok(None);
    }

    resolver
        .build_and_validate()
        .map_err(|err| format!("Failed to parse config for {}: {err}", file.display()))?;
    Ok(Some(resolver))
}

// ---

/// Check if a path string looks like a glob pattern.
/// Glob-like characters are also valid path characters on some environments.
/// If the path actually exists on disk, it is treated as a concrete path.
/// e.g. `{config}.js`, `[id].tsx`
fn is_glob_pattern(s: &str, cwd: &Path) -> bool {
    let has_glob_chars = s.contains('*') || s.contains('?') || s.contains('[') || s.contains('{');
    has_glob_chars && !cwd.join(s).exists()
}

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

/// Pre-scan directories within walk targets to discover nested config file locations.
///
/// A separate pass is required because the flat single-walk needs all configs
/// loaded into a scope map before `visit()` can resolve file scopes.
///
/// This also preserves the streaming architecture (walk → channel → format workers),
/// which keeps format workers busy sooner.
/// This is critical because current bottleneck is non-JS file formatting via external JS runtime.
///
/// The returned directories are used to:
/// 1. Load child scope configs (`resolve_child_scope_configs()`)
/// 2. Build an ancestor set for `filter_entry()` directory skipping
///
/// Uses a sequential (non-parallel) walk since the workload is small
/// (only directory entries are processed, files are skipped).
#[instrument(level = "debug", name = "oxfmt::walk::prescan_config_locations", skip_all)]
fn prescan_config_locations(
    target_paths: &[PathBuf],
    ignore_file_matchers: &Arc<[Gitignore]>,
    with_node_modules: bool,
) -> Vec<PathBuf> {
    let Some(first) = target_paths.first() else {
        return vec![];
    };

    let mut builder = ignore::WalkBuilder::new(first);
    for path in target_paths.iter().skip(1) {
        builder.add(path);
    }

    let matchers = Arc::clone(ignore_file_matchers);
    // Only descend into directories; skip all files
    builder.filter_entry(move |entry| {
        entry.file_type().is_some_and(|ft| ft.is_dir())
            && !is_walk_excluded_dir(entry, &matchers, with_node_modules)
    });

    let mut config_dirs = vec![];
    for entry in configure_walk_builder(builder).build().flatten() {
        let dir = entry.path();
        if has_config_in_directory(dir) {
            config_dirs.push(dir.to_path_buf());
        }
    }

    config_dirs
}

/// Load all child scope configs discovered by pre-scan.
///
/// Returns the scope map and a flag indicating whether any child scope had a config.
/// Skips directories whose resolved config matches the root config dir
/// (they belong to the root scope, not a child scope).
#[instrument(level = "debug", name = "oxfmt::walk::load_child_scopes", skip_all)]
fn resolve_child_scope_configs(
    config_dirs: &[PathBuf],
    root_config_dir: Option<&Path>,
    editorconfig_path: Option<&Path>,
    #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
) -> Result<(FxHashMap<PathBuf, Arc<ConfigResolver>>, bool), String> {
    let mut map = FxHashMap::default();
    let mut has_children = false;
    for config_dir in config_dirs {
        let mut resolver = ConfigResolver::from_config(
            config_dir,
            // NOTE: Let it auto-discover (not a direct path) config,
            // so that `discover_config()` can skip invalid candidates
            // (e.g. `vite.config.ts` without a `.fmt` field)
            // and continue searching upward.
            None,
            editorconfig_path,
            #[cfg(feature = "napi")]
            js_config_loader,
        )
        .map_err(|err| format!("Failed to load config in {}: {err}", config_dir.display()))?;

        // No config found, or same as root → belongs to root scope
        if resolver.config_dir().is_none() || resolver.config_dir() == root_config_dir {
            continue;
        }

        resolver
            .build_and_validate()
            .map_err(|err| format!("Failed to parse config in {}: {err}", config_dir.display()))?;

        has_children = true;
        map.insert(config_dir.clone(), Arc::from(resolver));
    }

    Ok((map, has_children))
}

/// Build the ancestor set from discovered config directories.
/// Enables O(1) "has config descendant?" lookups in `filter_entry()`,
/// allowing `ignorePatterns` directories to be skipped when no nested config exists beneath them.
fn build_config_ancestors(config_dirs: &[PathBuf]) -> Arc<FxHashSet<PathBuf>> {
    let mut ancestors = FxHashSet::default();
    for config_dir in config_dirs {
        for ancestor in config_dir.ancestors() {
            if !ancestors.insert(ancestor.to_path_buf()) {
                break;
            }
        }
    }
    Arc::new(ancestors)
}

// ---

/// Build a Walk, stream entries to the shared channel.
#[expect(clippy::needless_pass_by_value)] // Arcs are moved into closures/structs
fn walk_and_stream(
    target_paths: &[PathBuf],
    ignore_file_matchers: Arc<[Gitignore]>,
    with_node_modules: bool,
    glob_matcher: Option<&Arc<GlobMatcher>>,
    root_config_resolver: &Arc<ConfigResolver>,
    directly_processed: &Arc<FxHashSet<PathBuf>>,
    config_ancestors: Option<&Arc<FxHashSet<PathBuf>>>,
    child_scope_map: &Arc<FxHashMap<PathBuf, Arc<ConfigResolver>>>,
    sender: &mpsc::Sender<FormatEntry>,
) {
    let Some(first_path) = target_paths.first() else {
        return;
    };

    let mut inner = ignore::WalkBuilder::new(first_path);
    for path in target_paths.iter().skip(1) {
        inner.add(path);
    }

    let filter_global = Arc::clone(&ignore_file_matchers);
    let filter_root_resolver = Arc::clone(root_config_resolver);
    let filter_ancestors = config_ancestors.cloned();
    let filter_child_scopes = Arc::clone(child_scope_map);
    // NOTE: If return `false` here, it will not be `visit()`ed at all
    inner.filter_entry(move |entry| {
        let Some(file_type) = entry.file_type() else {
            return false;
        };
        let is_dir = file_type.is_dir();

        if is_dir && is_walk_excluded_dir(entry, &filter_global, with_node_modules) {
            return false;
        }
        // Global ignores also apply to files (e.g. `*.test.js` in .prettierignore)
        if !is_dir && is_ignored(&filter_global, entry.path(), false, false) {
            return false;
        }
        // Check scope-specific `ignorePatterns` for directories.
        // Resolves which scope (root or child) this directory belongs to,
        // then checks that scope's ignorePatterns.
        // Safe to skip when pre-scan confirms no config descendant exists beneath.
        if is_dir && filter_ancestors.as_ref().is_some_and(|a| !a.contains(entry.path())) {
            let resolver: &ConfigResolver = entry
                .path()
                .ancestors()
                .find_map(|a| filter_child_scopes.get(a).map(AsRef::as_ref))
                .unwrap_or(&filter_root_resolver);
            if resolver.is_path_ignored(entry.path(), true) {
                return false;
            }
        }

        // NOTE: Glob pattern matching is NOT done here in `filter_entry()`.
        // Glob patterns like `**/*.js` cannot be used to skip directories,
        // since any directory could contain matching files at any depth.
        // Glob filtering is instead done per-file in the visitor `visit()` below.
        true
    });

    let mut builder = WalkVisitorBuilder {
        sender: sender.clone(),
        root_config_resolver: Arc::clone(root_config_resolver),
        glob_matcher: glob_matcher.cloned(),
        child_scope_map: Arc::clone(child_scope_map),
        directly_processed: Arc::clone(directly_processed),
    };

    let num_of_threads = rayon::current_num_threads();
    configure_walk_builder(inner)
        // Use the same thread count as rayon (controlled by `--threads`)
        .threads(num_of_threads)
        .build_parallel()
        .visit(&mut builder);
}

/// Configure a `WalkBuilder` with the standard settings matching Prettier's behavior.
///
/// Shared between the pre-scan walk and the main parallel walk.
fn configure_walk_builder(mut builder: ignore::WalkBuilder) -> ignore::WalkBuilder {
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
        .require_git(false);
    builder
}

struct WalkVisitorBuilder {
    sender: mpsc::Sender<FormatEntry>,
    root_config_resolver: Arc<ConfigResolver>,
    glob_matcher: Option<Arc<GlobMatcher>>,
    child_scope_map: Arc<FxHashMap<PathBuf, Arc<ConfigResolver>>>,
    /// Files already processed as direct file targets (for dedup with walk results).
    directly_processed: Arc<FxHashSet<PathBuf>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkVisitorBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor {
            sender: self.sender.clone(),
            root_config_resolver: Arc::clone(&self.root_config_resolver),
            glob_matcher: self.glob_matcher.clone(),
            child_scope_map: Arc::clone(&self.child_scope_map),
            directly_processed: Arc::clone(&self.directly_processed),
            scope_cache: FxHashMap::default(),
        })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<FormatEntry>,
    root_config_resolver: Arc<ConfigResolver>,
    glob_matcher: Option<Arc<GlobMatcher>>,
    child_scope_map: Arc<FxHashMap<PathBuf, Arc<ConfigResolver>>>,
    directly_processed: Arc<FxHashSet<PathBuf>>,
    /// Cache: parent dir → (resolved config, parent_ignored flag).
    scope_cache: FxHashMap<PathBuf, (Arc<ConfigResolver>, bool)>,
}

impl WalkVisitor {
    /// Ensure the scope for `parent` is cached, resolving it if needed.
    ///
    /// Walks `parent.ancestors()` to find the nearest child scope in the pre-loaded map.
    /// Falls back to root scope if no child scope is found.
    /// Caches the resolved scope and pre-computed `parent_ignored` flag.
    fn ensure_scope_cached(&mut self, parent: &Path) {
        if self.scope_cache.contains_key(parent) {
            return;
        }

        let resolver = parent
            .ancestors()
            .find_map(|a| self.child_scope_map.get(a))
            .cloned()
            .unwrap_or_else(|| Arc::clone(&self.root_config_resolver));

        let parent_ignored = resolver.is_path_ignored(parent, true);
        self.scope_cache.insert(parent.to_path_buf(), (resolver, parent_ignored));
    }
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                let Some(file_type) = entry.file_type() else {
                    return ignore::WalkState::Continue;
                };
                if file_type.is_dir() {
                    return ignore::WalkState::Continue;
                }

                // Use `is_file()` to detect symlinks to the directory named `.js`
                #[expect(clippy::filetype_is_file)]
                if file_type.is_file() {
                    let path = entry.into_path();

                    // Skip files already processed as direct file targets
                    if self.directly_processed.contains(&path) {
                        return ignore::WalkState::Continue;
                    }

                    // Resolve this file's scope (cached per parent directory)
                    let parent = path.parent().expect("walk yields absolute paths");
                    self.ensure_scope_cached(parent);
                    let (resolver, parent_ignored) = &self.scope_cache[parent];

                    // Check scope's `ignorePatterns` per-file.
                    //
                    // Two-level check:
                    // 1. Parent directory (cached): catches directory patterns like "lib" that match all files under lib/.
                    //    Uses check_ancestors so "lib" also catches files under lib/packages/.
                    // 2. File-level: catches file-specific patterns like "temp.js".
                    if *parent_ignored || resolver.is_path_ignored(&path, false) {
                        return ignore::WalkState::Continue;
                    }

                    // When glob matcher is active,
                    // only accept files that match glob patterns or explicitly specified concrete paths.
                    if let Some(glob_matcher) = &self.glob_matcher
                        && !glob_matcher.matches(&path)
                    {
                        return ignore::WalkState::Continue;
                    }

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

                    if self
                        .sender
                        .send(FormatEntry { strategy, config_resolver: Arc::clone(resolver) })
                        .is_err()
                    {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}

// ---

/// Check if a directory should be excluded from walking.
///
/// Skips VCS directories (`.git`, `.svn`, etc.), `node_modules` (by default),
/// and directories matched by global ignore files (`.prettierignore`, `--ignore-path`, CLI `!`).
fn is_walk_excluded_dir(
    entry: &ignore::DirEntry,
    global_matchers: &[Gitignore],
    with_node_modules: bool,
) -> bool {
    is_ignored_dir(entry.file_name(), with_node_modules)
        || is_ignored(global_matchers, entry.path(), true, false)
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
/// We set `.hidden(false)` on `WalkBuilder` to include hidden files,
/// but still skip these specific directories (matching Prettier's behavior).
/// <https://prettier.io/docs/ignore#ignoring-files-prettierignore>
fn is_ignored_dir(dir_name: &OsStr, with_node_modules: bool) -> bool {
    dir_name == ".git"
        || dir_name == ".jj"
        || dir_name == ".sl"
        || dir_name == ".svn"
        || dir_name == ".hg"
        || (!with_node_modules && dir_name == "node_modules")
}
