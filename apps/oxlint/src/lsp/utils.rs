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

    use tower_lsp_server::ls_types::{Position, Range};

    use crate::lsp::utils::{normalize_path, range_overlaps};

    fn range(sl: u32, sc: u32, el: u32, ec: u32) -> Range {
        Range::new(Position::new(sl, sc), Position::new(el, ec))
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/root/directory/./.oxlintrc.json")),
            Path::new("/root/directory/.oxlintrc.json")
        );
    }

    #[test]
    fn test_range_overlaps_with_equal_ranges() {
        let range = range(1, 2, 3, 4);
        // A range always overlaps itself.
        assert!(range_overlaps(range, range));
    }

    #[test]
    fn test_range_overlaps_when_touching_at_boundary() {
        let a = range(1, 0, 1, 5);
        let b = range(1, 5, 1, 10);
        // End/start boundary is inclusive, so touching at one position counts as overlap.
        assert!(range_overlaps(a, b));
        assert!(range_overlaps(b, a));
    }

    #[test]
    fn test_range_overlaps_when_one_contains_the_other() {
        let outer = range(2, 0, 6, 0);
        let inner = range(3, 4, 4, 8);
        // Full containment is a strict overlap in either argument order.
        assert!(range_overlaps(outer, inner));
        assert!(range_overlaps(inner, outer));
    }

    #[test]
    fn test_range_overlaps_when_partially_overlapping() {
        let a = range(10, 0, 12, 0);
        let b = range(11, 5, 14, 0);
        // Shared interior span means they overlap.
        assert!(range_overlaps(a, b));
        assert!(range_overlaps(b, a));
    }

    #[test]
    fn test_range_overlaps_when_disjoint() {
        let a = range(1, 0, 1, 4);
        let b = range(1, 5, 1, 10);
        // There is a strict gap between a.end and b.start, so no overlap.
        assert!(!range_overlaps(a, b));
        assert!(!range_overlaps(b, a));
    }

    #[test]
    fn test_range_overlaps_respects_line_before_character() {
        let a = range(1, 50, 1, 200);
        let b = range(2, 0, 2, 100);
        // Even though character columns could look overlapping, line 2 is strictly after line 1.
        // Position ordering is line-first, then character, so these ranges are disjoint.
        assert!(!range_overlaps(a, b));
        assert!(!range_overlaps(b, a));
    }
}
