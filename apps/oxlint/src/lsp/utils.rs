use std::path::{Component, Path, PathBuf};

use tower_lsp_server::ls_types::Range;

/// Normalize a path by removing `.` and resolving `..` components,
/// without touching the filesystem.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.as_ref().components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {
                // Skip current directory component
            }
            Component::Normal(c) => {
                result.push(c);
            }
            Component::RootDir | Component::Prefix(_) => {
                result.push(component.as_os_str());
            }
        }
    }

    result
}

/// Returns `true` if LSP ranges `a` and `b` overlap or touch (share a boundary point).
///
/// This uses non-strict comparisons (`<=`/`>=`), so adjacent ranges where the end of one
/// equals the start of the other are also considered overlapping. This is intentional for
/// code-action filtering in the LSP server (a cursor at a boundary position should match
/// actions on either side), and is used conservatively in `fix_all_text_edit` to avoid
/// applying edits that share a boundary (which is rare in practice and safe to defer).
pub(super) fn range_overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && a.end >= b.start
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::lsp::utils::normalize_path;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/root/directory/./.oxlintrc.json")),
            Path::new("/root/directory/.oxlintrc.json")
        );
    }
}
