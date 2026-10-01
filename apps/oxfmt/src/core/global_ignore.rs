use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::core::utils;

/// Resolve ignore file paths from `--ignore-path` or defaults.
///
/// Called early (before walk) to validate that specified ignore files exist.
/// Empty `ignore_paths` falls back to `<cwd>/.prettierignore`.
pub fn resolve_ignore_paths(cwd: &Path, ignore_paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    if !ignore_paths.is_empty() {
        let mut result = Vec::with_capacity(ignore_paths.len());
        for path in ignore_paths {
            let path = utils::normalize_relative_path(cwd, path);
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

/// Build global ignore matchers from ignore files and CLI exclude patterns.
///
/// These are scope-independent and block both files and directories across all scopes.
/// Each matcher has its own root for pattern resolution:
/// - ignore files use their parent dir
/// - excludes use `cwd`
///
/// Git ignore files are handled by `WalkBuilder` itself.
pub fn build_global_ignore_matchers(
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

/// Check if a path should be ignored by any of the matchers.
/// A path is ignored if any matcher says it's ignored (and not whitelisted in that same matcher).
///
/// When `check_ancestors: true`, a path below a parent directory ignored by a matcher is ignored too,
/// even if the same matcher whitelists the path itself, see [`matches_with_ancestors`].
/// This is more expensive, but necessary when paths (to be ignored) are passed directly via CLI arguments.
/// For normal walking, walk is done in a top-down manner, so only the current path needs to be checked.
pub fn is_ignored(
    matchers: &[Gitignore],
    path: &Path,
    is_dir: bool,
    check_ancestors: bool,
) -> bool {
    matchers.iter().any(|matcher| {
        if check_ancestors {
            matches_with_ancestors(matcher, path, is_dir)
        } else {
            matcher.matched(path, is_dir).is_ignore()
        }
    })
}

/// Check if a path is ignored by the matcher, or by one of its parent directories below the matcher's root.
///
/// Like Git and Prettier, a negated pattern cannot re-include a path if one of its parent directories is excluded.
/// This gives the same result as a top-down walk, which does not descend into an ignored directory.
/// A path outside the matcher's root is never ignored.
pub fn matches_with_ancestors(matcher: &Gitignore, path: &Path, is_dir: bool) -> bool {
    let root = matcher.path();
    path.starts_with(root)
        && (matcher.matched(path, is_dir).is_ignore()
            || path
                .ancestors()
                .skip(1)
                .take_while(|ancestor| *ancestor != root && ancestor.starts_with(root))
                .any(|ancestor| matcher.matched(ancestor, true).is_ignore()))
}
