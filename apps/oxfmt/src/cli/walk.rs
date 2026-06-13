use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock, mpsc},
};

use fast_glob::glob_match;
use ignore::gitignore::Gitignore;
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::instrument;

use oxc_config::{all_paths_have_vcs_boundary, configure_walk_builder};
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};

use super::resolve::{build_global_ignore_matchers, is_ignored};
#[cfg(feature = "napi")]
use crate::core::JsConfigLoaderCb;
use crate::core::{
    ConfigResolver, FormatStrategy, NestedConfigCtx, ResolveOutcome, classify_file_kind,
    resolve_file_scope_config,
};

/// Orchestrates file discovery with nested config and ignore handling.
///
/// Constructed from CLI path arguments, which are classified into
/// target paths, glob patterns, and exclude patterns.
///
/// # Config resolution
/// Each file is formatted by its nearest config (auto-discovered upward).
/// Scopes do not inherit between parent and child.
///
/// # Walk phases
/// - Phase 1: Classify CLI paths into directory targets, file targets, and globs.
/// - Phase 2: Resolve & format file targets directly (no walk).
/// - Phase 3: Walk directory targets in parallel with on-demand nested config discovery.
///
/// # Ignore model
/// Three layers, applied in `filter_entry()` and `visit()`:
/// 1. Hardcoded VCS / `node_modules` skips
/// 2. Global ignores (`.prettierignore`, `--ignore-path`, CLI `!path`)
/// 3. Scope-local `ignorePatterns` from each resolved config
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
        root_config_resolver: ConfigResolver,
        ignore_paths: &[PathBuf],
        with_node_modules: bool,
        detect_nested: bool,
        editorconfig_path: Option<&Path>,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
        tx_entry: &mpsc::Sender<FormatStrategy>,
        tx_error: &DiagnosticSender,
    ) -> Result<bool, String> {
        let root_config_resolver = Arc::new(root_config_resolver);

        // Global ignores: .prettierignore, --ignore-path, CLI `!` patterns
        let ignore_file_matchers: Arc<[Gitignore]> = Arc::from(build_global_ignore_matchers(
            &self.cwd,
            &self.exclude_patterns,
            ignore_paths,
        )?);

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
                let Ok(metadata) = path.metadata() else { continue };
                let is_dir = metadata.is_dir();
                if is_ignored(&ignore_file_matchers, path, is_dir, true) {
                    continue;
                }
                if is_dir {
                    dirs.push(path.clone());
                } else if metadata.is_file() {
                    files.push(path.clone());
                }
            }
            (dirs, files)
        };

        // Walk-wide shared nested-config state used in both
        // - Phase 2 (direct file targets)
        // - and Phase 3 (parallel visitors)
        // Each directory's config load runs at most once walk-wide.
        let nested_config_ctx = NestedConfigCtx::new(
            editorconfig_path.map(Arc::from),
            #[cfg(feature = "napi")]
            js_config_loader.cloned(),
        );
        let mut directly_processed: FxHashSet<PathBuf> = FxHashSet::default();

        // Phase 2: Process file targets directly (no walk needed).
        if !file_targets.is_empty() {
            let mut scope_cache: FxHashMap<&Path, Arc<ConfigResolver>> = FxHashMap::default();
            for file in &file_targets {
                debug_assert!(
                    file.is_file(),
                    "Phase 1 should have classified `{}` as a regular file",
                    file.display(),
                );

                let parent = file.parent().unwrap();
                if !scope_cache.contains_key(parent) {
                    let resolved = resolve_file_scope_config(
                        file,
                        &root_config_resolver,
                        detect_nested.then_some(&nested_config_ctx),
                    )?;
                    scope_cache.insert(parent, resolved);
                }
                let config_resolver = Arc::clone(&scope_cache[parent]);

                if config_resolver.is_path_ignored(file, false) {
                    continue;
                }

                let Some(strategy) = resolve_format_strategy(
                    Arc::from(file.as_path()),
                    &config_resolver,
                    tx_error,
                    &self.cwd,
                ) else {
                    continue;
                };

                directly_processed.insert(file.clone());
                if tx_entry.send(strategy).is_err() {
                    break;
                }
            }
        }

        // Phase 3: Walk directory targets.
        let has_vcs_boundary = all_paths_have_vcs_boundary(&walk_targets, &self.cwd);
        // Build the glob matcher once for walk-time filtering.
        // When glob patterns exist, files are matched against them during `visit()`.
        // When no globs, `visit()` has zero overhead.
        let glob_matcher = (!self.glob_patterns.is_empty()).then(|| {
            Arc::new(GlobMatcher::new(self.cwd.clone(), self.glob_patterns.clone(), &walk_targets))
        });
        let directly_processed: Arc<FxHashSet<PathBuf>> = Arc::new(directly_processed);
        // Only consulted by `ensure_scope_cached` when `detect_nested` is true,
        // so skip the allocation otherwise.
        let walk_target_roots: Arc<[PathBuf]> = if detect_nested {
            Arc::from(walk_targets.clone())
        } else {
            Arc::from(Vec::<PathBuf>::new())
        };
        let fatal_error: Arc<OnceLock<String>> = Arc::new(OnceLock::new());

        walk_and_stream(
            &self.cwd,
            &walk_targets,
            has_vcs_boundary,
            with_node_modules,
            WalkFilters {
                ignore_file_matchers: Arc::clone(&ignore_file_matchers),
                glob_matcher,
                directly_processed,
            },
            WalkConfigState {
                root_config_resolver: Arc::clone(&root_config_resolver),
                nested_config_ctx: nested_config_ctx.clone(),
                detect_nested,
                walk_target_roots,
            },
            WalkSinks {
                tx_entry: tx_entry.clone(),
                tx_error: tx_error.clone(),
                fatal_error: Arc::clone(&fatal_error),
            },
        );

        // Surface any fatal error encountered inside the parallel walk, abort
        if let Some(err) = fatal_error.get() {
            return Err(err.clone());
        }

        Ok(root_config_resolver.config_dir().is_some() || nested_config_ctx.config_found())
    }
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

// ---

/// Output channels shared across all visitors.
///
/// `fatal_error` records the first error seen by any visitor:
/// parallel visitors cannot bubble errors up via `?` (the `visit` callback returns `WalkState`),
/// so the walk drops errors into this slot and the orchestrator drains it after the walk completes.
/// `OnceLock::set` ensures only the first writer wins.
#[derive(Clone)]
struct WalkSinks {
    tx_entry: mpsc::Sender<FormatStrategy>,
    tx_error: DiagnosticSender,
    fatal_error: Arc<OnceLock<String>>,
}

/// Walk-wide config-resolution state shared across all visitors.
#[derive(Clone)]
struct WalkConfigState {
    root_config_resolver: Arc<ConfigResolver>,
    nested_config_ctx: NestedConfigCtx,
    detect_nested: bool,
    walk_target_roots: Arc<[PathBuf]>,
}

/// Walk-wide filter inputs (ignore matchers, glob filter, dedup set).
#[derive(Clone)]
struct WalkFilters {
    ignore_file_matchers: Arc<[Gitignore]>,
    glob_matcher: Option<Arc<GlobMatcher>>,
    directly_processed: Arc<FxHashSet<PathBuf>>,
}

/// Build a Walk, stream entries to the shared channel.
fn walk_and_stream(
    cwd: &Path,
    target_paths: &[PathBuf],
    has_vcs_boundary: bool,
    with_node_modules: bool,
    filters: WalkFilters,
    config_state: WalkConfigState,
    sinks: WalkSinks,
) {
    let Some(first_path) = target_paths.first() else {
        return;
    };

    let mut inner = ignore::WalkBuilder::new(first_path);
    for path in target_paths.iter().skip(1) {
        inner.add(path);
    }

    let filter_global = Arc::clone(&filters.ignore_file_matchers);
    let nested_config_ctx = config_state.nested_config_ctx.clone();
    inner.filter_entry(move |entry| {
        let Some(file_type) = entry.file_type() else {
            return false;
        };

        let is_dir = file_type.is_dir();
        // Check if a directory should be excluded from walking.
        //
        // Skips VCS directories (`.git`, `.svn`, etc.), `node_modules` (by default),
        // and directories matched by global ignore files (`.prettierignore`, `--ignore-path`, CLI `!`).
        if is_dir
            && (is_ignored_dir(entry.file_name(), with_node_modules)
                || is_ignored(&filter_global, entry.path(), true, false))
        {
            return false;
        }
        // File-level global ignores apply, EXCEPT for config-looking files.
        // Those must reach `visit()` so entry-based discovery can register them in `scope_by_dir`.
        // Whether they get formatted is decided separately in `visit()` by re-checking global ignore.
        if !is_dir
            && !nested_config_ctx.is_config_file(entry.path())
            && is_ignored(&filter_global, entry.path(), false, false)
        {
            return false;
        }

        // NOTE: Scope-local `ignorePatterns` are checked per-file in `visit()`, not here.
        // Glob pattern matching is also done per-file in `visit()`,
        // since patterns like `**/*.js` cannot reliably skip directories.
        true
    });

    let mut builder = WalkVisitorBuilder { cwd: Arc::from(cwd), filters, config_state, sinks };

    // Git-related settings come from the shared helper to align with Oxlint.
    // NOTE: Prettier only reads `.gitignore` in the cwd and does not respect `.git/info/exclude`.
    configure_walk_builder(&mut inner, has_vcs_boundary)
        // Do not follow symlinks like Prettier does.
        // See https://github.com/prettier/prettier/pull/14627
        .follow_links(false)
        // Use the same thread count as rayon (controlled by `--threads`)
        .threads(rayon::current_num_threads())
        .build_parallel()
        .visit(&mut builder);
}

struct WalkVisitorBuilder {
    cwd: Arc<Path>,
    filters: WalkFilters,
    config_state: WalkConfigState,
    sinks: WalkSinks,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkVisitorBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor {
            cwd: Arc::clone(&self.cwd),
            filters: self.filters.clone(),
            config_state: self.config_state.clone(),
            sinks: self.sinks.clone(),
            scope_cache: FxHashMap::default(),
        })
    }
}

struct WalkVisitor {
    cwd: Arc<Path>,
    filters: WalkFilters,
    config_state: WalkConfigState,
    sinks: WalkSinks,
    /// Visitor-local cache: parent dir → (resolved scope, parent_ignored flag).
    scope_cache: FxHashMap<PathBuf, (Arc<ConfigResolver>, bool)>,
}

impl WalkVisitor {
    /// Record the first fatal error seen by any visitor.
    fn record_fatal(&self, err: String) {
        let _ = self.sinks.fatal_error.set(err);
    }

    /// Resolve and cache `parent`'s scope.
    ///
    /// Pass 1 (lookup) walks `parent.ancestors()`, accumulating dirs without cache hit into `visited`.
    /// Pass 2 (race-rescue probe) probes every dir in `visited` (closer-first).
    /// Even when Pass 1 hit an outer ancestor,
    /// this protects `nearest-config-wins` against parallel visitors that may register a closer config concurrently.
    fn ensure_scope_cached(&mut self, parent: &Path) -> Result<(), String> {
        // Case 1: Already cached by this visitor
        if self.scope_cache.contains_key(parent) {
            return Ok(());
        }

        let root_config_resolver = &self.config_state.root_config_resolver;

        // Case 2: Nested config disabled, shares the root scope, no probe needed
        if !self.config_state.detect_nested {
            let parent_ignored = root_config_resolver.is_path_ignored(parent, true);
            self.scope_cache
                .insert(parent.to_path_buf(), (Arc::clone(root_config_resolver), parent_ignored));
            return Ok(());
        }

        // Limit the ancestor directories to probe up to the closest walk root ancestor
        let probe_root: Option<&Path> = self
            .config_state
            .walk_target_roots
            .iter()
            .map(PathBuf::as_path)
            .filter(|t| parent.starts_with(t))
            .max_by_key(|t| t.components().count());
        let root_config_dir = root_config_resolver.config_dir();

        // Pass 1: cheap ancestor lookup (no probe).
        let mut visited: Vec<PathBuf> = vec![];
        let mut hit_via_lookup: Option<Arc<ConfigResolver>> = None;
        for dir in parent.ancestors() {
            // Check this dir's scope via:
            // (1) this visitor's cache
            if let Some((r, _)) = self.scope_cache.get(dir) {
                hit_via_lookup = Some(Arc::clone(r));
                break;
            }
            // (2) root config
            if Some(dir) == root_config_dir {
                hit_via_lookup = Some(Arc::clone(root_config_resolver));
                break;
            }
            // (3) shared cache, covered by other visitors' probes
            if let Some(r) = self.config_state.nested_config_ctx.lookup_scope(dir) {
                hit_via_lookup = Some(r);
                break;
            }
            // Otherwise, accumulate for Pass 2 probe and keep looking up.
            visited.push(dir.to_path_buf());

            if Some(dir) == probe_root {
                break;
            }
        }

        // Pass 2: race-rescue probe across `visited`, closer-first.
        //
        // Runs even when Pass 1 hit an outer ancestor:
        // A parallel visitor may have registered that outer dir before this visitor's Pass 1 ran,
        // while a closer config on disk hasn't been registered yet.
        // Probing closer-first catches it and preserves nearest-config-wins.
        let mut found_closer: Option<(PathBuf, Arc<ConfigResolver>)> = None;
        // Initial value covers the no-break case: if the loop completes without
        // finding a closer config, every entry in `visited` is probed-and-None.
        let mut probed_none_count = visited.len();
        for (i, dir) in visited.iter().enumerate() {
            if let Some(loaded) = self.config_state.nested_config_ctx.probe_dir(dir)? {
                found_closer = Some((dir.clone(), loaded));
                probed_none_count = i;
                break;
            }
        }
        visited.truncate(probed_none_count);

        // Merge priority: Pass 2 closer wins over Pass 1 hit.
        // Pass 1's hit may come from a parallel visitor that registered an outer ancestor,
        // closer config confirmed by Pass 2 always overrides it.
        let (resolved_scope_dir, resolver) = match (found_closer, hit_via_lookup) {
            (Some((dir, loaded)), _) => (Some(dir), loaded),
            (None, Some(r)) => (None, r),
            (None, None) => (None, Arc::clone(root_config_resolver)),
        };

        // Cache every dir that now has a confirmed scope:
        // (1) the dir that owns the resolved scope
        // (2) probed-and-None dirs (legitimate negative cache for dirs strictly below the resolved scope)
        // (3) `parent` itself
        // `or_insert_with_key` makes inserts idempotent, earlier writes win.
        let dirs_to_cache = resolved_scope_dir
            .into_iter()
            .chain(visited)
            .chain(std::iter::once(parent.to_path_buf()));
        for dir in dirs_to_cache {
            self.scope_cache.entry(dir).or_insert_with_key(|d| {
                let parent_ignored = resolver.is_path_ignored(d, true);
                (Arc::clone(&resolver), parent_ignored)
            });
        }

        Ok(())
    }

    /// Format eligibility:
    /// - Resolve scope for `path`
    /// - Apply scope-local `ignorePatterns` / glob / kind filters
    /// - Dispatch to format workers
    fn dispatch_format(&mut self, path: PathBuf) -> ignore::WalkState {
        let parent = path.parent().expect("walk yields absolute paths");

        if let Err(err) = self.ensure_scope_cached(parent) {
            self.record_fatal(err);
            return ignore::WalkState::Quit;
        }

        let (resolver, parent_ignored) = &self.scope_cache[parent];

        // Scope-local `ignorePatterns`:
        // - parent dir (cached) catches directory patterns like `lib`
        // - file-level catches patterns like `temp.js`
        if *parent_ignored || resolver.is_path_ignored(&path, false) {
            return ignore::WalkState::Continue;
        }
        if let Some(glob_matcher) = &self.filters.glob_matcher
            && !glob_matcher.matches(&path)
        {
            return ignore::WalkState::Continue;
        }
        let Some(strategy) =
            resolve_format_strategy(Arc::from(path), resolver, &self.sinks.tx_error, &self.cwd)
        else {
            return ignore::WalkState::Continue;
        };

        if self.sinks.tx_entry.send(strategy).is_err() {
            return ignore::WalkState::Quit;
        }

        ignore::WalkState::Continue
    }
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        let entry = match entry {
            Ok(e) => e,
            Err(_err) => return ignore::WalkState::Skip,
        };
        let Some(file_type) = entry.file_type() else {
            return ignore::WalkState::Continue;
        };

        // Nothing to do at dir-level in `visit()`.
        // Whether to walk into it is decided by `filter_entry()`.
        if file_type.is_dir() {
            return ignore::WalkState::Continue;
        }
        // Skip non-regular entries (symlinks of any kind, sockets, etc.) to
        // match the walker's `follow_links(false)` behavior.
        #[expect(clippy::filetype_is_file)]
        if !file_type.is_file() {
            return ignore::WalkState::Continue;
        }

        let path = entry.into_path();

        // Skip files already formatted as Phase 2 direct file targets
        if self.filters.directly_processed.contains(&path) {
            return ignore::WalkState::Continue;
        }

        let parent = path.parent().expect("walk yields absolute paths");

        // `filter_entry` exempts config-looking files from file-level global ignore,
        // so discovery can see them.
        // Here we (1) register the parent dir's scope when nested detection is on,
        // and (2) re-apply the global ignore for format eligibility.
        let is_config_file = self.config_state.nested_config_ctx.is_config_file(&path);

        if is_config_file
            && self.config_state.detect_nested
            && let Err(err) = self.config_state.nested_config_ctx.probe_dir(parent)
        {
            self.record_fatal(err);
            return ignore::WalkState::Quit;
        }

        if is_config_file && is_ignored(&self.filters.ignore_file_matchers, &path, false, false) {
            return ignore::WalkState::Continue;
        }

        self.dispatch_format(path)
    }
}

// ---

/// Classify `path`, resolve its scope, and return the format strategy if any.
///
/// `None` means "not a formatting target" or "missing plugin"; resolve errors
/// are reported via `tx_error` and also yield `None` so callers can move on
/// to the next file.
#[expect(clippy::needless_pass_by_value)] // caller has no further use for `path`
fn resolve_format_strategy(
    path: Arc<Path>,
    resolver: &ConfigResolver,
    tx_error: &DiagnosticSender,
    cwd: &Path,
) -> Option<FormatStrategy> {
    let kind = classify_file_kind(Arc::clone(&path))?;
    match resolver.resolve(kind) {
        Ok(ResolveOutcome::Format(strategy)) => Some(strategy),
        Ok(ResolveOutcome::MissingPlugin(_)) => None,
        Err(err) => {
            // Report a per-file config resolve error via the diagnostic channel.
            let diagnostics = DiagnosticService::wrap_diagnostics(
                cwd,
                &path,
                "",
                vec![
                    OxcDiagnostic::error(format!(
                        "Invalid resolved configuration for {}",
                        path.display()
                    ))
                    .with_help(err),
                ],
            );
            let _ = tx_error.send(diagnostics);
            None
        }
    }
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

// ---

#[cfg(test)]
mod tests_scope_resolution {
    #[cfg(feature = "napi")]
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::{fs, sync::mpsc};

    use tempfile::TempDir;

    use super::*;

    fn write_config(dir: &Path, contents: &str) {
        fs::write(dir.join(".oxfmtrc.json"), contents).expect("write config");
    }

    fn make_ctx() -> NestedConfigCtx {
        NestedConfigCtx::new(
            None,
            #[cfg(feature = "napi")]
            None,
        )
    }

    /// Minimal `WalkVisitor` for exercising `ensure_scope_cached`.
    /// Channels / filters are dummies — the test never sends or applies them.
    fn make_visitor(walk_root: &Path, ctx: NestedConfigCtx) -> WalkVisitor {
        let mut root_resolver =
            ConfigResolver::from_json_config(None, None).expect("default resolver");
        root_resolver.build_and_validate().expect("validate default");

        let (tx_entry, _rx_entry) = mpsc::channel();
        let (tx_error, _rx_error) = mpsc::channel();

        WalkVisitor {
            cwd: Arc::from(walk_root),
            filters: WalkFilters {
                ignore_file_matchers: Arc::from(Vec::<Gitignore>::new()),
                glob_matcher: None,
                directly_processed: Arc::new(FxHashSet::default()),
            },
            config_state: WalkConfigState {
                root_config_resolver: Arc::new(root_resolver),
                nested_config_ctx: ctx,
                detect_nested: true,
                walk_target_roots: Arc::from(vec![walk_root.to_path_buf()]),
            },
            sinks: WalkSinks { tx_entry, tx_error, fatal_error: Arc::new(OnceLock::new()) },
            scope_cache: FxHashMap::default(),
        }
    }

    /// Race simulation: another visitor already registered an outer ancestor
    /// in `scope_by_dir`, but a closer dir has a config file on disk that
    /// has NOT yet been registered. The Pass 2 race-rescue probe must pick
    /// the closer config — this is the entire reason the probe exists even
    /// when Pass 1 already had a lookup hit.
    #[test]
    fn race_rescue_picks_closer_config_when_outer_already_registered() {
        let tmp = TempDir::new().expect("tempdir");
        let outer = tmp.path().join("repo");
        let closer = outer.join("src");
        let leaf_parent = closer.join("sub");
        fs::create_dir_all(&leaf_parent).expect("create dirs");

        write_config(&outer, r#"{ "printWidth": 100 }"#);
        write_config(&closer, r#"{ "printWidth": 60 }"#);

        let ctx = make_ctx();
        // Pre-register outer as if another visitor got there first.
        ctx.probe_dir(&outer).expect("probe outer").expect("outer config");

        let mut visitor = make_visitor(tmp.path(), ctx);
        visitor.ensure_scope_cached(&leaf_parent).expect("resolve");

        let (resolver, _) = &visitor.scope_cache[&leaf_parent];
        assert_eq!(
            resolver.config_dir(),
            Some(closer.as_path()),
            "race-rescue must pick the closer config even when an outer ancestor \
             was already registered in scope_by_dir before this visitor ran"
        );
    }

    /// When no closer config exists, the rescue probe returns `None` for
    /// every visited dir and the outer lookup hit is retained. Guards
    /// against an off-by-one rescue that drops the legitimate outer hit.
    #[test]
    fn race_rescue_falls_back_to_outer_hit_when_no_closer_config_exists() {
        let tmp = TempDir::new().expect("tempdir");
        let outer = tmp.path().join("repo");
        let middle = outer.join("src");
        let leaf_parent = middle.join("sub");
        fs::create_dir_all(&leaf_parent).expect("create dirs");

        write_config(&outer, r#"{ "printWidth": 100 }"#);

        let ctx = make_ctx();
        ctx.probe_dir(&outer).expect("probe outer").expect("outer config");

        let mut visitor = make_visitor(tmp.path(), ctx);
        visitor.ensure_scope_cached(&leaf_parent).expect("resolve");

        let (resolver, _) = &visitor.scope_cache[&leaf_parent];
        assert_eq!(
            resolver.config_dir(),
            Some(outer.as_path()),
            "with no closer config on disk, the outer lookup hit must be retained"
        );
    }

    /// When Pass 2 finds a closer config at `visited[i]` and breaks,
    /// entries `visited[i+1..]` were never probed. They must NOT be cached
    /// with the closer's resolver — they sit ABOVE the closer dir in the
    /// ancestor chain and do not share its scope.
    ///
    /// Example: parent = /repo/src has a direct config. Pass 1 collects
    /// `visited = [/repo/src, /repo, walk_root]`. Pass 2 breaks at
    /// /repo/src. /repo and walk_root were not probed — caching them with
    /// /repo/src's resolver would later misroute a file at /repo/other.ts
    /// (whose parent /repo would hit the early-return in
    /// `ensure_scope_cached`) through /repo/src's config.
    #[test]
    fn ancestors_above_closer_must_not_inherit_its_scope() {
        let tmp = TempDir::new().expect("tempdir");
        let outer = tmp.path().join("repo");
        let closer = outer.join("src");
        fs::create_dir_all(&closer).expect("create dirs");

        write_config(&closer, r#"{ "printWidth": 60 }"#);

        let ctx = make_ctx();
        let mut visitor = make_visitor(tmp.path(), ctx);
        visitor.ensure_scope_cached(&closer).expect("resolve");

        if let Some((resolver, _)) = visitor.scope_cache.get(outer.as_path()) {
            assert_ne!(
                resolver.config_dir(),
                Some(closer.as_path()),
                "ancestor /repo (above closer /repo/src) must not be cached with \
                 /repo/src's resolver — files at /repo/other.ts would otherwise be \
                 formatted with the wrong config"
            );
        }
    }

    /// Fast path: `parent` itself has a direct config — Pass 2's first
    /// probe hits and breaks. No outer-ancestor state is involved.
    #[test]
    fn parent_with_direct_config_resolves_to_itself() {
        let tmp = TempDir::new().expect("tempdir");
        let parent_dir = tmp.path().join("repo").join("src");
        fs::create_dir_all(&parent_dir).expect("create dirs");

        write_config(&parent_dir, r#"{ "printWidth": 60 }"#);

        let ctx = make_ctx();
        let mut visitor = make_visitor(tmp.path(), ctx);
        visitor.ensure_scope_cached(&parent_dir).expect("resolve");

        let (resolver, _) = &visitor.scope_cache[&parent_dir];
        assert_eq!(resolver.config_dir(), Some(parent_dir.as_path()));
    }

    /// Phase 2 (direct file targets) must return the pre-built root resolver
    /// by `Arc::ptr_eq` when ancestor walk reaches `root_config_dir` — i.e.,
    /// it must NOT re-load the root config through `config_load_cache`.
    /// For JS root configs that path would otherwise invoke the NAPI loader
    /// a second time on every Phase 2 file.
    #[test]
    fn phase2_at_root_config_dir_returns_same_arc_without_reloading() {
        let tmp = TempDir::new().expect("tempdir");
        let repo = tmp.path().join("repo");
        let src = repo.join("src");
        fs::create_dir_all(&src).expect("create dirs");
        write_config(&repo, r#"{ "printWidth": 80 }"#);

        let target_file = src.join("file.ts");
        fs::write(&target_file, "").expect("write target");

        let mut root_resolver =
            ConfigResolver::from_json_config(Some(&repo.join(".oxfmtrc.json")), None)
                .expect("load root config");
        root_resolver.build_and_validate().expect("validate root config");
        let root_resolver = Arc::new(root_resolver);

        let ctx = make_ctx();
        let resolved = resolve_file_scope_config(&target_file, &root_resolver, Some(&ctx))
            .expect("resolve file scope");

        assert!(
            Arc::ptr_eq(&resolved, &root_resolver),
            "Phase 2 must return the pre-built root Arc directly (no config_load_cache round-trip)"
        );
    }

    /// `config_load_cache` + `OnceLock` must dedupe NAPI loader invocations
    /// across concurrent `probe_dir` callers on the same dir.
    ///
    /// This is the single most important guarantee for large repos with JS
    /// configs: an N-thread walk that all reach the same nested
    /// `oxfmt.config.ts` must trigger the JS loader exactly once, not N
    /// times. E2E tests cannot observe NAPI call counts.
    #[cfg(feature = "napi")]
    #[test]
    fn napi_loader_called_once_per_dir_across_concurrent_probes() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("repo");
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(dir.join("oxfmt.config.ts"), "export default {};").expect("write config");

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_in_cb = Arc::clone(&counter);
        let cb: JsConfigLoaderCb = Arc::new(move |_path: String| {
            counter_in_cb.fetch_add(1, Ordering::Relaxed);
            Ok(serde_json::json!({}))
        });

        let ctx = NestedConfigCtx::new(None, Some(cb));

        std::thread::scope(|s| {
            for _ in 0..8 {
                let ctx_local = ctx.clone();
                let dir_local = dir.clone();
                s.spawn(move || {
                    ctx_local.probe_dir(&dir_local).expect("probe").expect("loaded");
                });
            }
        });

        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "NAPI loader must be invoked exactly once even with 8 concurrent probes"
        );
    }

    /// A broken JS config's `Err` must be cached so concurrent / repeated
    /// `probe_dir` calls don't retry the failing load. Without `Err`
    /// caching, every visitor that walked through a broken-config dir would
    /// re-invoke the NAPI loader.
    #[cfg(feature = "napi")]
    #[test]
    fn napi_loader_err_is_cached_and_not_retried() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("repo");
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(dir.join("oxfmt.config.ts"), "broken").expect("write config");

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_in_cb = Arc::clone(&counter);
        let cb: JsConfigLoaderCb = Arc::new(move |_path: String| {
            counter_in_cb.fetch_add(1, Ordering::Relaxed);
            Err("simulated load failure".to_string())
        });

        let ctx = NestedConfigCtx::new(None, Some(cb));

        let err1 = ctx.probe_dir(&dir).expect_err("first probe should error");
        let err2 = ctx.probe_dir(&dir).expect_err("second probe should hit cached Err");

        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "loader must not be re-invoked after an Err is cached"
        );
        assert_eq!(err1, err2, "both errors must come from the same cached load");
        assert!(
            err1.contains("simulated load failure"),
            "cached Err must surface the underlying loader message, got: {err1}"
        );
    }
}
