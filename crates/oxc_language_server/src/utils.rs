use std::path::Path;

use cow_utils::CowUtils;
use tower_lsp_server::gen_lsp_types::Uri;

use crate::file_system::ResolvedPath;

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

/// Whether `path` starts with `root`, comparing the components as they are spelled: ignoring the
/// case on Windows and macOS, where the file system does, byte-wise elsewhere.
pub fn is_spelled_below(path: &Path, root: &Path) -> bool {
    starts_with_components(path, root, cfg!(any(windows, target_os = "macos")))
}

fn starts_with_components(path: &Path, root: &Path, ignore_case: bool) -> bool {
    if !ignore_case {
        return path.starts_with(root);
    }
    let mut path = path.components();
    root.components().all(|root| {
        path.next().is_some_and(|component| {
            component.as_os_str().to_string_lossy().cow_to_lowercase()
                == root.as_os_str().to_string_lossy().cow_to_lowercase()
        })
    })
}

/// `path` relative to `root`, with `/` as separator on every platform, to name it in a message.
/// `path` itself when it is not below `root`.
pub fn display_path_below(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().cow_replace('\\', "/").into_owned()
}

/// Find the index of the most specific root responsible for `uri`.
///
/// When several roots contain the URI (nested workspace folders, `workingDirectories`), the root
/// with the longest matching path wins. For non `file://` URIs the first root is returned, which
/// mirrors the behaviour of rust-analyzer and typescript-language-server.
pub fn find_root_for_uri(roots: &[Uri], uri: &Uri) -> Option<usize> {
    find_root_for_uri_in(roots.iter().map(|root| ResolvedPath::try_from(root).ok()), uri)
}

/// Same as [`find_root_for_uri`], over an iterator of roots which are already resolved, `None`
/// for a root which has no path. The worker manager routes every `textDocument/*` notification
/// with it, over the root path each worker resolved once, so only the path of `uri` is resolved
/// per request.
pub(crate) fn find_root_for_uri_in<P: AsRef<Path>>(
    roots: impl Iterator<Item = Option<P>>,
    uri: &Uri,
) -> Option<usize> {
    if uri.scheme().as_str() != "file" {
        // these are in-memory files that don't have a file path
        return roots.enumerate().next().map(|(index, _)| index);
    }

    let resolved_path = ResolvedPath::try_from(uri).ok()?;
    let file_path = resolved_path.as_path();

    roots
        .enumerate()
        .filter_map(|(index, root)| {
            let root_path = root.as_ref()?.as_ref();
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

    #[test]
    fn test_display_path_below() {
        assert_eq!(
            display_path_below(Path::new("/repo/packages/a"), Path::new("/repo")),
            "packages/a"
        );
        assert_eq!(display_path_below(Path::new("/other/a"), Path::new("/repo")), "/other/a");
    }

    /// Components are compared whole, with or without the case of the letters.
    #[test]
    fn test_starts_with_components() {
        let file = Path::new("/Repo/Packages/a/index.ts");

        for ignore_case in [false, true] {
            assert!(starts_with_components(file, Path::new("/Repo/Packages"), ignore_case));
            assert!(!starts_with_components(file, Path::new("/Repo/Pack"), ignore_case));
            assert!(!starts_with_components(file, Path::new("/Repo/Packages/b"), ignore_case));
            assert!(!starts_with_components(Path::new("/Repo"), Path::new("/Repo/a"), ignore_case));
        }

        assert!(!starts_with_components(file, Path::new("/repo/packages"), false));
        assert!(starts_with_components(file, Path::new("/repo/packages"), true));
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
