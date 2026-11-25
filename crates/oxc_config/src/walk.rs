use std::{
    collections::hash_map::Entry,
    path::{Path, PathBuf},
};

use ignore::{IncrementalIgnore, WalkBuilder};
use rustc_hash::FxHashMap;

/// Apply `ignore::WalkBuilder` settings shared between Oxlint and Oxfmt.
/// Tool-specific options such as `follow_links()` are left to the caller.
///
/// `has_vcs_boundary` should be the result of [`all_paths_have_vcs_boundary`] for the walk targets.
/// Callers that build multiple walkers from the same targets can compute it once and reuse it.
///
/// [`GitignoreChecker`] uses these same settings for walk-root checks.
pub fn configure_walk_builder(
    builder: &mut WalkBuilder,
    has_vcs_boundary: bool,
) -> &mut WalkBuilder {
    builder
        // Include hidden files to lint|format; VCS directories are skipped by each tool
        .hidden(false)
        // Ignore generic `.ignore` files
        .ignore(false)
        // Ignore the user's global gitignore
        .git_global(false)
        // Respect repository-local (nested) `.gitignore` files
        .git_ignore(true)
        // Also look up parent directories
        .parents(true)
        // Respect `$GIT_COMMON_DIR/info/exclude` as well
        .git_exclude(true)
        // Parent `.gitignore` lookup stops at the repository boundary when targets are inside a repo
        .require_git(has_vcs_boundary)
}

/// Check whether walk target paths match the Git-derived ignore rules used by the walker.
///
/// The `ignore` crate walker applies gitignore's "everything under an ignored directory is ignored" rule
/// only by pruning ignored directories during traversal, and never filters its walk roots.
/// A walk target inside an ignored directory is therefore never filtered.
/// And a bare directory pattern like `generated` matches the directory itself, not the files below it.
/// Use an incremental matcher rooted at the VCS (or filesystem) boundary to check targets first.
/// See also <https://github.com/BurntSushi/ripgrep/issues/2595>.
///
/// This is pattern-based and does not inspect Git's index.
/// A tracked file that matches an ignore pattern therefore still matches here.
///
/// NOTE: Mirrors the [`configure_walk_builder`] settings:
/// - nested `.gitignore` files,
/// - `$GIT_COMMON_DIR/info/exclude`,
/// - and parent lookup stopping at the VCS boundary when one exists
#[derive(Debug, Default)]
pub struct GitignoreChecker {
    /// Incremental matcher per VCS boundary (or filesystem root when no boundary exists).
    matchers: FxHashMap<PathBuf, IncrementalIgnore>,
    /// Whether a directory contains a `.git` or `.jj` marker.
    vcs_roots: FxHashMap<PathBuf, bool>,
}

impl GitignoreChecker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return `true` when `path` (or an ancestor directory below the VCS boundary) is
    /// matched as ignored by a `.gitignore` in one of its ancestor directories.
    ///
    /// Directory-only patterns use the path's symlink type, matching Git's behavior.
    /// A symlink to a directory is therefore treated as a symlink, not as a directory.
    /// Relative paths are resolved against `cwd`.
    pub fn is_gitignored(&mut self, path: &Path, cwd: &Path) -> bool {
        let path = resolve_against_cwd(path, cwd);
        let is_dir = path.symlink_metadata().is_ok_and(|metadata| metadata.is_dir());

        // Match from the VCS boundary so ignored ancestors between it and the target are checked.
        // Without a boundary, start at the filesystem root
        // because the walker reads parent `.gitignore` files all the way up in that case.
        let vcs_root = find_vcs_boundary(&path, &mut self.vcs_roots);
        let has_vcs_boundary = vcs_root.is_some();
        let root = vcs_root
            .unwrap_or_else(|| path.ancestors().last().unwrap_or(path.as_path()))
            .to_path_buf();
        let Ok(relative) = path.strip_prefix(&root) else { return false };

        let matcher = match self.matchers.entry(root) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let mut builder = WalkBuilder::new(entry.key());
                configure_walk_builder(&mut builder, has_vcs_boundary);
                let Some(matcher) = builder.build_matchers().pop() else { return false };
                entry.insert(matcher)
            }
        };
        matcher.matched(relative, is_dir).is_ignore()
    }

    /// Walk-root variant of [`Self::is_gitignored`]: `true` only for gitignored directory targets.
    /// `.gitignore` scopes discovery, an explicitly named file target is processed even when gitignored.
    ///
    /// Dir-ness follows symlinks (= whether the walker would descend into the target);
    /// pattern matching itself still uses the symlink type via [`Self::is_gitignored`].
    pub fn is_gitignored_walk_root(&mut self, path: &Path, cwd: &Path) -> bool {
        resolve_against_cwd(path, cwd).is_dir() && self.is_gitignored(path, cwd)
    }
}

/// Return `true` when every path is inside a Git or Jujutsu repository.
///
/// A path is considered inside a repository when one of its ancestor
/// directories contains a `.git` or `.jj` entry.
/// This matches the boundary detection used by the `ignore` crate when `require_git(true)` is set.
///
/// Relative paths are resolved against `cwd`.
/// When `paths` is empty, returns `true`.
pub fn all_paths_have_vcs_boundary(paths: &[PathBuf], cwd: &Path) -> bool {
    let mut cache = FxHashMap::default();
    paths.iter().all(|path| has_vcs_boundary(path, cwd, &mut cache))
}

fn has_vcs_boundary(path: &Path, cwd: &Path, cache: &mut FxHashMap<PathBuf, bool>) -> bool {
    let path = resolve_against_cwd(path, cwd);
    find_vcs_boundary(&path, cache).is_some()
}

/// Whether `dir` contains a `.git` or `.jj` marker (directory, or file for worktrees).
fn has_vcs_marker(dir: &Path) -> bool {
    dir.join(".git").exists() || dir.join(".jj").exists()
}

fn resolve_against_cwd(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { cwd.join(path) }
}

fn find_vcs_boundary<'a>(path: &'a Path, cache: &mut FxHashMap<PathBuf, bool>) -> Option<&'a Path> {
    let start = if path.is_file() { path.parent().unwrap_or(path) } else { path };
    start.ancestors().find(|dir| {
        if let Some(&has) = cache.get(*dir) {
            return has;
        }
        let has = has_vcs_marker(dir);
        cache.insert((*dir).to_path_buf(), has);
        has
    })
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    use ignore::WalkBuilder;

    use super::{GitignoreChecker, all_paths_have_vcs_boundary, configure_walk_builder};

    fn collect_walked_js_files(root: &Path) -> Vec<String> {
        let mut builder = WalkBuilder::new(root);
        let has_boundary = all_paths_have_vcs_boundary(&[root.to_path_buf()], root);
        let mut paths: Vec<String> = configure_walk_builder(&mut builder, has_boundary)
            .build()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    return None;
                }
                let path = entry.path();
                if path.extension()? != "js" {
                    return None;
                }
                Some(path.strip_prefix(root).ok()?.to_string_lossy().to_string())
            })
            .collect();
        paths.sort();
        paths
    }

    #[test]
    fn boundary_returns_true_when_path_is_inside_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");
        let src_path = repo_path.join("src");

        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(&src_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();

        assert!(all_paths_have_vcs_boundary(&[src_path], temp_path));
    }

    #[test]
    fn boundary_returns_true_when_path_is_inside_jj_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");

        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".jj")).unwrap();

        assert!(all_paths_have_vcs_boundary(&[repo_path], temp_path));
    }

    #[test]
    fn boundary_returns_true_when_git_is_a_file_worktree_marker() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");

        fs::create_dir(&repo_path).unwrap();
        fs::write(repo_path.join(".git"), "gitdir: /tmp/worktrees/repo/.git\n").unwrap();

        assert!(all_paths_have_vcs_boundary(&[repo_path], temp_path));
    }

    #[test]
    fn gitignore_is_respected_without_git_repo() {
        // `.gitignore` should still apply when no `.git` directory is present.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("included.js"), "").unwrap();
        fs::write(temp_path.join("ignored.js"), "").unwrap();
        fs::write(temp_path.join(".gitignore"), "ignored.js\n").unwrap();

        assert!(!temp_path.join(".git").exists());
        assert_eq!(collect_walked_js_files(temp_path), vec!["included.js"]);
    }

    #[test]
    fn parent_gitignore_does_not_cross_git_repo_boundary() {
        // A parent `.gitignore` must not apply once the walk enters a nested
        // repository. The nested repo's own `.gitignore` should still apply.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");

        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(temp_path.join(".gitignore"), "*\n").unwrap();
        fs::write(repo_path.join(".gitignore"), "ignored.js\n").unwrap();
        fs::write(repo_path.join("included.js"), "").unwrap();
        fs::write(repo_path.join("ignored.js"), "").unwrap();

        assert_eq!(collect_walked_js_files(&repo_path), vec!["included.js"]);
    }

    #[test]
    fn parent_gitignore_does_not_cross_git_worktree_file_boundary() {
        // Git worktrees use a `.git` file instead of a `.git` directory. That
        // file is still a repository boundary for parent `.gitignore` lookup.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");

        fs::create_dir(&repo_path).unwrap();
        fs::write(temp_path.join(".gitignore"), "*\n").unwrap();
        fs::write(repo_path.join(".git"), "gitdir: /tmp/worktrees/repo/.git\n").unwrap();
        fs::write(repo_path.join("included.js"), "").unwrap();

        assert_eq!(collect_walked_js_files(&repo_path), vec!["included.js"]);
    }

    #[test]
    fn gitignored_respects_info_exclude_in_linked_worktree() {
        let temp_dir = tempfile::tempdir().unwrap();
        let common_git_dir = temp_dir.path().join("main").join(".git");
        let worktree_git_dir = common_git_dir.join("worktrees").join("linked");
        let worktree_path = temp_dir.path().join("linked");

        fs::create_dir_all(common_git_dir.join("info")).unwrap();
        fs::create_dir_all(&worktree_git_dir).unwrap();
        fs::create_dir(&worktree_path).unwrap();
        fs::write(worktree_path.join(".git"), format!("gitdir: {}\n", worktree_git_dir.display()))
            .unwrap();
        fs::write(worktree_git_dir.join("commondir"), "../..\n").unwrap();
        fs::write(common_git_dir.join("info").join("exclude"), "ignored.js\nignored/\n").unwrap();

        let ignored_dir = worktree_path.join("ignored");
        fs::create_dir(&ignored_dir).unwrap();
        fs::write(worktree_path.join("ignored.js"), "").unwrap();
        fs::write(worktree_path.join("included.js"), "").unwrap();

        let mut checker = GitignoreChecker::new();
        assert!(checker.is_gitignored(&worktree_path.join("ignored.js"), &worktree_path));
        assert!(checker.is_gitignored(&ignored_dir, &worktree_path));
        assert!(!checker.is_gitignored(&worktree_path.join("included.js"), &worktree_path));
    }

    #[test]
    fn gitignored_with_bare_directory_pattern() {
        // A bare directory pattern in a nested `.gitignore` must ignore walk
        // targets inside that directory, matching `git check-ignore`.
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let target = repo_path.join("sub").join("generated").join("pkg");

        fs::create_dir_all(&target).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(repo_path.join("sub").join(".gitignore"), "generated\n").unwrap();

        let mut checker = GitignoreChecker::new();
        assert!(checker.is_gitignored(&target, &repo_path));
        // Files inside the ignored tree are also ignored
        assert!(checker.is_gitignored(&target.join("index.ts"), &repo_path));
        // The ignored directory itself is also inside the ignored tree
        assert!(checker.is_gitignored(&repo_path.join("sub").join("generated"), &repo_path));
        // Sibling of the ignored directory is not
        assert!(!checker.is_gitignored(&repo_path.join("sub"), &repo_path));
    }

    #[test]
    fn walk_root_check_only_filters_directory_targets() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let target = repo_path.join("sub").join("generated").join("pkg");

        fs::create_dir_all(&target).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(repo_path.join("sub").join(".gitignore"), "generated\n").unwrap();
        fs::write(target.join("index.ts"), "").unwrap();

        let mut checker = GitignoreChecker::new();
        // Directory targets inside the ignored tree are filtered
        assert!(checker.is_gitignored_walk_root(&target, &repo_path));
        // An explicitly named file is not, even though `is_gitignored` matches it
        assert!(!checker.is_gitignored_walk_root(&target.join("index.ts"), &repo_path));
        assert!(checker.is_gitignored(&target.join("index.ts"), &repo_path));
    }

    #[test]
    fn gitignored_file_matched_by_own_directory_chain() {
        // File-level patterns must apply to the file itself: both a glob form
        // in an ancestor `.gitignore` and a pattern in the file's own directory.
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let pkg = repo_path.join("sub").join("generated").join("pkg");

        fs::create_dir_all(&pkg).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(repo_path.join("sub").join(".gitignore"), "generated/**\n").unwrap();
        fs::write(pkg.join(".gitignore"), "local.ts\n").unwrap();

        let mut checker = GitignoreChecker::new();
        assert!(checker.is_gitignored(&pkg.join("index.ts"), &repo_path));
        assert!(checker.is_gitignored(&pkg.join("local.ts"), &repo_path));
        // A directory-only pattern must not match a file.
        // Matchers are cached per checker, so use a fresh one after changing ignore files on disk.
        let dist_file = repo_path.join("dist");
        let build_dir = repo_path.join("build");
        fs::write(&dist_file, "").unwrap();
        fs::create_dir(&build_dir).unwrap();
        fs::write(repo_path.join(".gitignore"), "dist/\nbuild/\n").unwrap();
        let mut checker = GitignoreChecker::new();
        assert!(!checker.is_gitignored(&dist_file, &repo_path));
        assert!(checker.is_gitignored(&build_dir, &repo_path));
    }

    #[cfg(unix)]
    #[test]
    fn gitignored_directory_pattern_does_not_match_symlink_to_directory() {
        use std::os::unix::fs::symlink;

        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let target = repo_path.join("target");
        let link = repo_path.join("link");

        fs::create_dir_all(repo_path.join(".git")).unwrap();
        fs::create_dir(&target).unwrap();
        symlink("target", &link).unwrap();
        fs::write(repo_path.join(".gitignore"), "link/\ntarget/\n").unwrap();

        let mut checker = GitignoreChecker::new();
        assert!(!checker.is_gitignored(Path::new("link"), &repo_path));
        assert!(checker.is_gitignored(&target, &repo_path));
    }

    #[test]
    fn gitignored_respects_deeper_whitelist() {
        // A deeper `.gitignore` re-including the directory takes precedence
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let target = repo_path.join("sub").join("generated").join("pkg");

        fs::create_dir_all(&target).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(repo_path.join(".gitignore"), "generated\n").unwrap();
        fs::write(repo_path.join("sub").join(".gitignore"), "!generated\n").unwrap();

        assert!(!GitignoreChecker::new().is_gitignored(&target, &repo_path));
    }

    #[test]
    fn gitignored_stops_at_vcs_boundary() {
        // A `.gitignore` above the repository root must not apply
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("sub").join("repo");
        let target = repo_path.join("src");

        fs::create_dir_all(&target).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "repo\n").unwrap();

        assert!(!GitignoreChecker::new().is_gitignored(&target, &repo_path));
    }

    #[test]
    fn repo_gitignore_applies_when_walking_from_subdirectory() {
        // Stopping parent lookup at the repository boundary must still keep
        // repo-local parent `.gitignore` files active for subdirectory walks.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");
        let src_path = repo_path.join("src");

        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(&src_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(temp_path.join(".gitignore"), "*\n").unwrap();
        fs::write(repo_path.join(".gitignore"), "ignored.js\n").unwrap();
        fs::write(src_path.join("included.js"), "").unwrap();
        fs::write(src_path.join("ignored.js"), "").unwrap();

        assert_eq!(collect_walked_js_files(&src_path), vec!["included.js"]);
    }
}
