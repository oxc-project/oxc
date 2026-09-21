use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tower_lsp_server::ls_types::{Pattern, Uri};
use tracing::warn;

use crate::file_system::ResolvedPath;

/// Collect the ignore files named `names` from the directories between `parent_root` (inclusive)
/// and `root` (exclusive), ordered from the deepest directory to the outermost one.
///
/// A file which can not be read or compiled is skipped with a log: one broken ignore file must not
/// disable the whole chain.
pub fn ancestor_ignore_globs(parent_root: &Path, root: &Path, names: &[&str]) -> Vec<Gitignore> {
    let mut globs = vec![];

    for directory in ancestor_directories(parent_root, root).into_iter().rev() {
        for name in names {
            let ignore_file_path = directory.join(name);
            if !ignore_file_path.is_file() {
                continue;
            }

            let mut builder = GitignoreBuilder::new(&directory);
            if let Some(err) = builder.add(&ignore_file_path) {
                warn!("skipping the ignore file {}: {err}", ignore_file_path.display());
                continue;
            }
            match builder.build() {
                Ok(gitignore) => globs.push(gitignore),
                Err(err) => {
                    warn!("skipping the ignore file {}: {err}", ignore_file_path.display());
                }
            }
        }
    }

    globs
}

/// Whether the ignore chain excludes `file_path`.
///
/// `globs` must be ordered from the deepest ignore file to the outermost one.
///
/// This follows what `git` does, which is not "the first pattern which matches wins":
/// - a directory which an ignore file excludes takes everything below it with it, a `!pattern` in
///   a deeper ignore file can not re-include a file below it,
/// - but a deeper ignore file can re-include the *directory* itself, and then the files below it
///   are considered again.
pub fn is_ignored_by_globs(globs: &[Gitignore], file_path: &Path) -> bool {
    if globs.is_empty() {
        return false;
    }

    // A directory is only judged by an ignore file which sits strictly above it, and every glob
    // brings its own root: the chain of one ignore file says nothing about a sibling directory
    // governed by another one.
    let mut directories = vec![];
    let mut current = file_path.parent();
    while let Some(directory) = current {
        if !globs.iter().any(|glob| governs(glob, directory)) {
            break;
        }
        directories.push(directory);
        current = directory.parent();
    }

    // from the outermost directory down to the one holding the file
    for directory in directories.into_iter().rev() {
        if first_decisive_match(globs, directory, true) == Some(true) {
            return true;
        }
    }

    first_decisive_match(globs, file_path, false).unwrap_or(false)
}

/// The first decisive match of the chain, from the deepest ignore file to the outermost one.
///
/// `Some(true)` is ignored, `Some(false)` is explicitly re-included, `None` is no match.
fn first_decisive_match(globs: &[Gitignore], path: &Path, is_dir: bool) -> Option<bool> {
    for glob in globs {
        if !governs(glob, path) {
            continue;
        }
        match glob.matched(path, is_dir) {
            Match::Ignore(_) => return Some(true),
            Match::Whitelist(_) => return Some(false),
            Match::None => {}
        }
    }

    None
}

/// Whether the ignore file sits strictly above `path`, which is the only case in which it applies
/// to it. A `.gitignore` never excludes its own directory.
fn governs(glob: &Gitignore, path: &Path) -> bool {
    path != glob.path() && path.starts_with(glob.path())
}

/// Whether one of the watcher `patterns` matches `file_path`.
///
/// An absolute pattern is matched against the absolute path, a relative one against the path
/// relative to `root_path`, which is how the patterns are registered with the client.
///
/// `absolute_only` keeps the absolute patterns alone, the ones a tool registers for a config file
/// it extends from outside its root.
pub fn matches_watcher_patterns(
    patterns: &[Pattern],
    file_path: &Path,
    root_path: &Path,
    absolute_only: bool,
) -> bool {
    if patterns.is_empty() {
        return false;
    }

    // a glob always uses `/`, while a path uses `\` on Windows
    let absolute = file_path.to_string_lossy().cow_replace('\\', "/").into_owned();
    let relative = file_path
        .strip_prefix(root_path)
        .ok()
        .map(|relative| relative.to_string_lossy().cow_replace('\\', "/").into_owned());

    patterns.iter().any(|pattern| {
        if Path::new(pattern).is_absolute() {
            fast_glob::glob_match(pattern.cow_replace('\\', "/").as_ref(), absolute.as_str())
        } else if absolute_only {
            false
        } else {
            relative
                .as_deref()
                .is_some_and(|relative| fast_glob::glob_match(pattern.as_str(), relative))
        }
    })
}

/// Whether two root URIs point to the same directory.
///
/// Compares the normalized file paths instead of the URI strings, so a percent encoded path, a
/// trailing slash or a different casing (on Windows and macOS) still resolve to the same root.
pub fn roots_are_equal(left: &Uri, right: &Uri) -> bool {
    match (ResolvedPath::try_from(left), ResolvedPath::try_from(right)) {
        (Ok(left), Ok(right)) => left.as_path() == right.as_path(),
        // non `file://` URIs have no path to normalize
        _ => left == right,
    }
}

/// The directories between `parent_root` (inclusive) and `sub_root` (exclusive), outermost first.
///
/// Used to collect the ignore files which still apply to a directory nested inside a workspace
/// folder, the way the `ignore` crate walks the parents when a tool runs from inside a package.
/// Returns an empty list when `sub_root` is not below `parent_root`.
pub fn ancestor_directories(parent_root: &Path, sub_root: &Path) -> Vec<PathBuf> {
    if sub_root == parent_root || !sub_root.starts_with(parent_root) {
        return vec![];
    }

    let mut directories = vec![];
    let mut current = sub_root.parent();
    while let Some(directory) = current {
        if !directory.starts_with(parent_root) {
            break;
        }
        directories.push(directory.to_path_buf());
        if directory == parent_root {
            break;
        }
        current = directory.parent();
    }
    directories.reverse();

    directories
}

/// Find the index of the most specific root responsible for `uri`.
///
/// When several roots contain the URI (nested workspace folders, `workingDirectories`), the root
/// with the longest matching path wins. For non `file://` URIs the first root is returned, which
/// mirrors the behaviour of rust-analyzer and typescript-language-server.
pub fn find_root_for_uri(roots: &[Uri], uri: &Uri) -> Option<usize> {
    find_root_for_uri_in(roots.iter(), uri)
}

/// Same as [`find_root_for_uri`], over an iterator of roots.
///
/// This avoids collecting the roots into a `Vec` on the hot path, where the routing runs for every
/// `textDocument/*` notification.
pub fn find_root_for_uri_in<'a>(roots: impl Iterator<Item = &'a Uri>, uri: &Uri) -> Option<usize> {
    if uri.scheme().as_str() != "file" {
        // these are in-memory files that don't have a file path
        return roots.enumerate().next().map(|(index, _)| index);
    }

    let resolved_path = ResolvedPath::try_from(uri).ok()?;
    let file_path = resolved_path.as_path();

    roots
        .enumerate()
        .filter_map(|(index, root)| {
            let resolved_root = ResolvedPath::try_from(root).ok()?;
            let root_path = resolved_root.as_path();
            if file_path.starts_with(root_path) {
                Some((index, root_path.as_os_str().len()))
            } else {
                None
            }
        })
        .max_by_key(|(_, len)| *len)
        .map(|(index, _)| index)
}

/// Normalize the user config path to a watch pattern that can be used to watch for changes.
///
/// Watch pattern like `./oxlintrc.json` is not supported by some editors (VS Code), so we need to normalize it to `oxlintrc.json`.
pub fn normalize_user_config_path_to_watch_pattern(config_path: &str) -> String {
    let path = config_path.cow_replace('\\', "/");
    let path = path.cow_replace("/./", "/");
    let path = path.strip_prefix("./").unwrap_or(&path);

    let mut out = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            // escape path characters that have special meaning in glob patterns
            '*' | '?' | '[' | ']' | '{' | '}' | ',' | '!' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A path which is absolute on every platform: `Path::is_absolute` needs a drive prefix on
    /// Windows, so a `/` rooted path alone would never take the absolute pattern branch there.
    fn absolute(path: &str) -> String {
        if cfg!(windows) { format!("C:{path}") } else { path.to_string() }
    }

    #[test]
    fn test_roots_are_equal() {
        let plain: Uri = "file:///repo/packages/a".parse().unwrap();
        // a trailing slash and a percent encoded separator resolve to the same directory
        assert!(roots_are_equal(&plain, &"file:///repo/packages/a/".parse().unwrap()));
        assert!(roots_are_equal(&plain, &"file:///repo/packages%2Fa".parse().unwrap()));
        assert!(!roots_are_equal(&plain, &"file:///repo/packages/b".parse().unwrap()));

        // non `file://` URIs fall back to a string comparison
        let untitled: Uri = "untitled:///Untitled-1".parse().unwrap();
        assert!(roots_are_equal(&untitled, &"untitled:///Untitled-1".parse().unwrap()));
        assert!(!roots_are_equal(&untitled, &"untitled:///Untitled-2".parse().unwrap()));
    }

    #[test]
    fn test_is_ignored_by_globs() {
        fn glob(directory: &str, patterns: &[&str]) -> Gitignore {
            let mut builder = GitignoreBuilder::new(directory);
            for pattern in patterns {
                builder.add_line(None, pattern).unwrap();
            }
            builder.build().unwrap()
        }

        // a `!pattern` can not re-include a file below an excluded directory
        let globs = [glob("/repo/packages/a", &["!index.ts"]), glob("/repo", &["dist/"])];
        assert!(is_ignored_by_globs(&globs, Path::new("/repo/packages/a/dist/index.ts")));

        // but it can re-include the directory itself, and then the files below it count again
        let globs = [glob("/repo/packages/a", &["!dist/"]), glob("/repo", &["dist/"])];
        assert!(!is_ignored_by_globs(&globs, Path::new("/repo/packages/a/dist/index.ts")));

        // a plain match on the file itself
        let globs = [glob("/repo", &["*.log"])];
        assert!(is_ignored_by_globs(&globs, Path::new("/repo/packages/a/debug.log")));
        assert!(!is_ignored_by_globs(&globs, Path::new("/repo/packages/a/index.ts")));

        assert!(!is_ignored_by_globs(&[], Path::new("/repo/index.ts")));

        // every glob brings its own root: an ignore file in a sibling directory says nothing
        // about this file, and it must not stop the one which does from being evaluated
        let globs = [glob("/repo/packages/a", &["dist/"]), glob("/repo/apps", &["*.log"])];
        assert!(is_ignored_by_globs(&globs, Path::new("/repo/packages/a/dist/index.ts")));
        assert!(!is_ignored_by_globs(&globs, Path::new("/repo/packages/a/src/index.ts")));
        assert!(is_ignored_by_globs(&globs, Path::new("/repo/apps/web/debug.log")));
    }

    #[test]
    fn test_matches_watcher_patterns() {
        let root = absolute("/repo");
        let root_path = Path::new(&root);
        let file = absolute("/repo/packages/a/.oxlintrc.json");
        let file_path = Path::new(&file);

        // relative patterns are matched against the path relative to the root
        assert!(matches_watcher_patterns(
            &["**/.oxlintrc.json".to_string()],
            file_path,
            root_path,
            false
        ));
        assert!(!matches_watcher_patterns(&["*.json".to_string()], file_path, root_path, false));

        // absolute patterns are matched against the absolute path
        assert!(matches_watcher_patterns(std::slice::from_ref(&file), file_path, root_path, true));
        // a relative pattern is skipped when only the absolute ones count
        assert!(!matches_watcher_patterns(
            &["**/.oxlintrc.json".to_string()],
            file_path,
            root_path,
            true
        ));
        // a file outside the root only matches an absolute pattern
        assert!(!matches_watcher_patterns(
            &["**/.oxlintrc.json".to_string()],
            Path::new(&absolute("/other/.oxlintrc.json")),
            root_path,
            false
        ));

        assert!(!matches_watcher_patterns(&[], file_path, root_path, false));
    }

    #[test]
    fn test_matches_watcher_patterns_normalizes_separators() {
        let root = absolute("/repo");
        let root_path = Path::new(&root);

        // a Windows path and a Windows pattern still match the `/` separated glob
        assert!(matches_watcher_patterns(
            &[absolute("/repo/packages/a/.oxlintrc.json")],
            Path::new(&absolute(r"/repo\packages\a\.oxlintrc.json")),
            root_path,
            false
        ));
        assert!(matches_watcher_patterns(
            &[absolute(r"/repo\packages\a\.oxlintrc.json")],
            Path::new(&absolute("/repo/packages/a/.oxlintrc.json")),
            root_path,
            false
        ));
    }

    #[test]
    fn test_ancestor_directories() {
        assert_eq!(
            ancestor_directories(Path::new("/repo"), Path::new("/repo/packages/a")),
            vec![PathBuf::from("/repo"), PathBuf::from("/repo/packages")]
        );
        // the sub root itself is never included
        assert!(ancestor_directories(Path::new("/repo"), Path::new("/repo")).is_empty());
        // an unrelated directory has no ancestors inside the parent root
        assert!(ancestor_directories(Path::new("/repo"), Path::new("/other")).is_empty());
    }

    #[test]
    fn test_find_root_for_uri() {
        let roots: Vec<Uri> = vec![
            "file:///repo".parse().unwrap(),
            "file:///repo/packages/a".parse().unwrap(),
            "file:///other".parse().unwrap(),
        ];

        // the longest matching root wins
        assert_eq!(
            find_root_for_uri(&roots, &"file:///repo/packages/a/src/x.ts".parse().unwrap()),
            Some(1)
        );
        assert_eq!(find_root_for_uri(&roots, &"file:///repo/x.ts".parse().unwrap()), Some(0));
        assert_eq!(find_root_for_uri(&roots, &"file:///other/x.ts".parse().unwrap()), Some(2));
        // a similarly named sibling does not match
        assert_eq!(find_root_for_uri(&roots, &"file:///repo-2/x.ts".parse().unwrap()), None);
        // non `file://` URIs use the first root
        assert_eq!(find_root_for_uri(&roots, &"untitled:///Untitled-1".parse().unwrap()), Some(0));
        assert_eq!(find_root_for_uri(&[], &"untitled:///Untitled-1".parse().unwrap()), None);
    }

    #[test]
    fn test_normalize_user_config_path_to_watch_pattern() {
        assert_eq!(normalize_user_config_path_to_watch_pattern("./oxlintrc.json"), "oxlintrc.json");
        assert_eq!(
            normalize_user_config_path_to_watch_pattern(".\\oxlintrc.json"),
            "oxlintrc.json"
        );
        assert_eq!(normalize_user_config_path_to_watch_pattern("oxlintrc.json"), "oxlintrc.json");
        assert_eq!(
            normalize_user_config_path_to_watch_pattern("/home/oxlintrc.json"),
            "/home/oxlintrc.json"
        );
        assert_eq!(
            normalize_user_config_path_to_watch_pattern("C:\\home\\oxlintrc.json"),
            "C:/home/oxlintrc.json"
        );
    }
}
