use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use rustc_hash::FxHashMap;

/// Apply `ignore::WalkBuilder` settings shared between Oxlint and Oxfmt.
/// Tool-specific options such as `follow_links()` are left to the caller.
///
/// `has_vcs_boundary` should be the result of [`all_paths_have_vcs_boundary`] for the walk targets.
/// Callers that build multiple walkers from the same targets can compute it once and reuse it.
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
        // Respect `.git/info/exclude` as well
        .git_exclude(true)
        // Parent `.gitignore` lookup stops at the repository boundary when targets are inside a repo
        .require_git(has_vcs_boundary)
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
    let path = if path.is_absolute() { path.to_path_buf() } else { cwd.join(path) };
    let start = if path.is_file() { path.parent().unwrap_or(&path) } else { path.as_path() };

    // Cache per-directory marker presence so sibling paths reuse stat results.
    start.ancestors().any(|dir| {
        if let Some(&has) = cache.get(dir) {
            return has;
        }
        let has = dir.join(".git").exists() || dir.join(".jj").exists();
        cache.insert(dir.to_path_buf(), has);
        has
    })
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    use ignore::WalkBuilder;

    use super::{all_paths_have_vcs_boundary, configure_walk_builder};

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
