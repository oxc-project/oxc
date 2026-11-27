use cow_utils::CowUtils;
use tower_lsp_server::gen_lsp_types::Uri;

use crate::{BuildContext, file_system::ResolvedPath, worker::WorkspaceWorker};

/// The nested workspace folders around the root of a worker.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct NestedContext {
    /// The roots strictly below the root, which are served by their own worker. Sorted by path.
    pub excluded_roots: Vec<Uri>,
    /// The outermost root strictly above the root, whose ignore files still apply to it.
    pub parent_root: Option<Uri>,
}

impl NestedContext {
    /// The context a tool is built with.
    pub(crate) fn as_build_context(&self) -> BuildContext<'_> {
        BuildContext {
            excluded_roots: &self.excluded_roots,
            parent_root: self.parent_root.as_ref(),
        }
    }
}

/// The [`NestedContext`] of each of `roots`, in the same order.
///
/// Paths are compared the way the worker manager routes a file: a file below several roots goes
/// to the one with the longest path. A root which is equal to another one is neither nested in it
/// nor around it, and counts once. Roots which are not `file://` URIs have no path, no context,
/// and are never nested.
pub(crate) fn nested_contexts(roots: &[Uri]) -> Vec<NestedContext> {
    let resolved = roots.iter().map(|root| ResolvedPath::try_from(root).ok()).collect::<Vec<_>>();
    let others = roots.iter().zip(&resolved).filter_map(|(uri, path)| Some((uri, path.as_ref()?)));

    resolved
        .iter()
        .map(|root| {
            let Some(root) = root else {
                return NestedContext::default();
            };

            let mut excluded: Vec<(&Uri, &ResolvedPath)> = vec![];
            let mut parent: Option<(&Uri, &ResolvedPath)> = None;
            for (other, other_path) in others.clone() {
                if other_path.as_path() == root.as_path() {
                    continue;
                }
                if other_path.starts_with(root) {
                    excluded.push((other, other_path));
                } else if root.starts_with(other_path) {
                    let is_outer = parent.is_none_or(|(_, parent_path)| {
                        other_path.as_path().as_os_str().len()
                            < parent_path.as_path().as_os_str().len()
                    });
                    if is_outer {
                        parent = Some((other, other_path));
                    }
                }
            }

            excluded.sort_by(|(_, left), (_, right)| left.as_path().cmp(right.as_path()));
            excluded.dedup_by(|(_, left), (_, right)| left.as_path() == right.as_path());

            NestedContext {
                excluded_roots: excluded.into_iter().map(|(uri, _)| uri.clone()).collect(),
                parent_root: parent.map(|(uri, _)| uri.clone()),
            }
        })
        .collect()
}

/// Set the [`NestedContext`] of each of `workers`, among their roots.
///
/// Returns, for each worker, whether its context changed. Call it before the workers are started,
/// so their first build already skips the roots served by another worker.
pub(crate) async fn set_nested_contexts(workers: &[&WorkspaceWorker]) -> Vec<bool> {
    let roots = workers.iter().map(|worker| worker.get_root_uri().clone()).collect::<Vec<_>>();
    let mut changed = Vec::with_capacity(workers.len());
    for (worker, context) in workers.iter().zip(nested_contexts(&roots)) {
        changed.push(worker.set_context(context).await);
    }
    changed
}

/// Whether the worker rooted at `root` validates the document `uri`, given the paths of the roots
/// nested in it: the files below its root, except the ones of a nested root, whose worker
/// validates them. A document which is not a file, like an untitled one, has no root.
pub(crate) fn owns(root: Option<&ResolvedPath>, excluded: &[ResolvedPath], uri: &Uri) -> bool {
    let Ok(path) = ResolvedPath::try_from(uri) else {
        return true;
    };
    root.is_none_or(|root| path.starts_with(root))
        && !excluded.iter().any(|excluded| path.starts_with(excluded))
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

    fn uris(uris: &[&str]) -> Vec<Uri> {
        uris.iter().map(|uri| uri.parse().unwrap()).collect()
    }

    fn context(excluded_roots: &[&str], parent_root: Option<&str>) -> NestedContext {
        NestedContext {
            excluded_roots: uris(excluded_roots),
            parent_root: parent_root.map(|uri| uri.parse().unwrap()),
        }
    }

    #[test]
    fn test_nested_contexts() {
        let roots = uris(&[
            "file:///repo",
            "file:///repo/packages/a",
            "file:///repo/packages/a/src",
            "file:///repo/packages/b",
            "file:///repo-2",
            "untitled:///Untitled-1",
        ]);

        assert_eq!(
            nested_contexts(&roots),
            vec![
                // a sibling with the same name prefix is not nested
                context(
                    &[
                        "file:///repo/packages/a",
                        "file:///repo/packages/a/src",
                        "file:///repo/packages/b"
                    ],
                    None
                ),
                // the parent is the outermost enclosing root, not the closest one
                context(&["file:///repo/packages/a/src"], Some("file:///repo")),
                context(&[], Some("file:///repo")),
                context(&[], Some("file:///repo")),
                context(&[], None),
                // a root which is not a file has no context
                context(&[], None),
            ]
        );
    }

    /// A root which is listed twice is neither nested in itself nor excluded twice, and the order
    /// the roots were given in does not matter.
    #[test]
    fn test_nested_contexts_ignore_equal_roots_and_the_order() {
        let expected = context(&["file:///repo/a", "file:///repo/b"], None);

        for roots in [
            uris(&["file:///repo", "file:///repo/", "file:///repo/b", "file:///repo/a"]),
            uris(&["file:///repo/b", "file:///repo/a", "file:///repo/a/", "file:///repo"]),
        ] {
            let contexts = nested_contexts(&roots);
            let index = roots.iter().position(|root| root.as_str() == "file:///repo").unwrap();
            assert_eq!(contexts[index], expected);
        }
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
