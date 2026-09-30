//! Single home for the markdown list-marker "alphabet" shared across the JSDoc formatter:
//! - the wrap guard (`wrap.rs`)
//! - mdast routing (`detect.rs`)
//! - blank-line decisions (`line_buffer.rs`)
//!
//! One home for the alphabet, so per-call-site drift is caught here
//! (see the cross-consistency test at the bottom).
//!
//! Scope: list markers, plus the wrap-time block-marker-word test.
//! Code fences, headings, blockquotes, and indented code are still classified at their use sites.

/// Kind of list marker at the start of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ListMarker {
    /// `- `, `* `, `+ `
    Unordered,
    /// `1. `, `1) `.
    /// CommonMark lets an ordered list interrupt a paragraph only when it starts at 1
    /// (the spec's own guard against wrapped prose like `... in 1986. What a year`)
    Ordered { starts_at_one: bool },
    /// Legacy JSDoc `1- ` style (normalized to `1. ` by the mdast preprocess).
    /// NOTE: only `<digits>- ` (space separator).
    /// The normalization rewrite in `preprocess.rs` deliberately accepts a broader separator set
    /// (upstream regex `^(\d+)[-][\s|]+`), that rule stays with the rewriter.
    LegacyOrdered,
}

/// Detect a list marker at the start of `line`, ignoring leading spaces/tabs.
/// `line` may extend past the line end — only the marker prefix is read
/// (a leading `\n` is never skipped, so an empty line yields `None`).
pub(super) fn list_marker(line: &str) -> Option<ListMarker> {
    let bytes = line.trim_start_matches([' ', '\t']).as_bytes();
    match bytes.first()? {
        b'-' | b'*' | b'+' if bytes.get(1) == Some(&b' ') => Some(ListMarker::Unordered),
        first @ b'0'..=b'9' => {
            let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
            if bytes.get(digits + 1) != Some(&b' ') {
                return None;
            }
            match bytes[digits] {
                b'.' | b')' => {
                    Some(ListMarker::Ordered { starts_at_one: digits == 1 && *first == b'1' })
                }
                b'-' => Some(ListMarker::LegacyOrdered),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if a whitespace-delimited word would be parsed as
/// a markdown block-construct marker when placed at the start of a line:
///
/// - `-` / `+` / `*`: unordered list markers
/// - `>`-prefixed words: blockquote (`>` needs no space, so even `>=` starts one)
/// - `#`…`######`: ATX headings
/// - `1.` / `1)`: ordered list markers (any digit count), `1-`: legacy JSDoc ordered marker
///
/// Wrapping must never move such a word to a line start:
/// the next format pass re-parses the comment as markdown
/// and the word would change meaning (prose → list item), breaking idempotency.
/// Same alphabet as Prettier's markdown printer (`shouldPreventBreak`, regex `/^>|^(?:[*+-]|#{1,6}|\d+[).])$/`)
/// In particular any digit count is guarded, not just `1.`, and `>` matches as a prefix.
pub(super) fn is_block_marker_token(word: &str) -> bool {
    match word.as_bytes() {
        [b'-' | b'+' | b'*'] | [b'>', ..] => true,
        // ATX headings: `#`…`######`
        hashes @ [b'#', ..] => hashes.len() <= 6 && hashes.iter().all(|&b| b == b'#'),
        // Ordered list markers: digits followed by `.`, `)`, or legacy `-`
        [digits @ .., b'.' | b')' | b'-'] => {
            !digits.is_empty() && digits.iter().all(u8::is_ascii_digit)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_marker() {
        assert_eq!(list_marker("- item"), Some(ListMarker::Unordered));
        assert_eq!(list_marker("* item"), Some(ListMarker::Unordered));
        assert_eq!(list_marker("+ item"), Some(ListMarker::Unordered));
        assert_eq!(list_marker("  - indented"), Some(ListMarker::Unordered));
        assert_eq!(list_marker("1. item"), Some(ListMarker::Ordered { starts_at_one: true }));
        assert_eq!(list_marker("1) item"), Some(ListMarker::Ordered { starts_at_one: true }));
        assert_eq!(list_marker("2. item"), Some(ListMarker::Ordered { starts_at_one: false }));
        assert_eq!(list_marker("12) item"), Some(ListMarker::Ordered { starts_at_one: false }));
        assert_eq!(list_marker("1- legacy"), Some(ListMarker::LegacyOrdered));
        assert_eq!(list_marker("prose"), None);
        assert_eq!(list_marker("-dash"), None);
        assert_eq!(list_marker("1.5 value"), None);
        assert_eq!(list_marker("2016-2020"), None);
        // A leading `\n` is not skipped: an empty line is not a marker
        assert_eq!(list_marker("\n- item"), None);
        assert_eq!(list_marker(""), None);
    }

    #[test]
    fn line_and_word_alphabets_agree() {
        // Every marker `list_marker` accepts must also be guarded as a bare word
        // by `is_block_marker_token`, or wrapping could place at a line start
        // a word that routing then reads as a list
        for line in ["- x", "+ x", "* x", "1. x", "12) x", "1- x"] {
            assert!(list_marker(line).is_some());
            let word = line.split(' ').next().unwrap();
            assert!(is_block_marker_token(word), "{word} must be wrap-guarded");
        }
    }

    #[test]
    fn test_is_block_marker_token() {
        assert!(is_block_marker_token("-"));
        assert!(is_block_marker_token("+"));
        assert!(is_block_marker_token("*"));
        assert!(is_block_marker_token(">"));
        assert!(is_block_marker_token(">=")); // `>=` at line start would open a blockquote
        assert!(is_block_marker_token("#"));
        assert!(is_block_marker_token("###"));
        assert!(is_block_marker_token("1."));
        assert!(is_block_marker_token("12)"));
        assert!(is_block_marker_token("5.")); // any digit count, matching Prettier's `\d+[).]`
        assert!(is_block_marker_token("3-")); // legacy JSDoc ordered marker
        assert!(!is_block_marker_token("word"));
        assert!(!is_block_marker_token("--"));
        assert!(!is_block_marker_token("#hash"));
        assert!(!is_block_marker_token("1.5"));
        assert!(!is_block_marker_token("2016-2020"));
    }
}
