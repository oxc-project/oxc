use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

#[cfg(feature = "napi")]
use crate::core::JsConfigLoaderCb;
use crate::core::{ConfigResolver, utils::normalize_relative_path};

/// Resolve ignore file paths from CLI args or defaults.
///
/// Called early (before walk) to validate that specified ignore files exist.
pub(super) fn resolve_ignore_paths(
    cwd: &Path,
    ignore_paths: &[PathBuf],
) -> Result<Vec<PathBuf>, String> {
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

/// Build global ignore matchers from ignore files and CLI exclude patterns.
///
/// These are scope-independent and block both files and directories across all scopes.
/// Each matcher has its own root for pattern resolution:
/// - ignore files use their parent dir
/// - excludes use `cwd`
///
/// Git ignore files are handled by `WalkBuilder` itself.
pub(super) fn build_global_ignore_matchers(
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
/// When `check_ancestors: true`, also checks if any parent directory is ignored.
/// This is more expensive, but necessary when paths (to be ignored) are passed directly via CLI arguments.
/// For normal walking, walk is done in a top-down manner, so only the current path needs to be checked.
pub(super) fn is_ignored(
    matchers: &[Gitignore],
    path: &Path,
    is_dir: bool,
    check_ancestors: bool,
) -> bool {
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

/// Resolve the nearest config scope for a file target.
/// Returns `None` if the file belongs to the root scope.
pub(super) fn resolve_file_scope_config(
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
