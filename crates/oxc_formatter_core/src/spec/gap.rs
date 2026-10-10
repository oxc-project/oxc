//! Vertical-spacing classification of an inter-token source gap.
//!
//! Shared by the formatter family whose line terminators are CR, LF, or CRLF (no LS/PS).
//! Languages whose gap semantics differ do NOT parameterize this function.
//! They own their helper instead. (js, json)
//!
//! Some consumers that normalize line endings to `\n` before parsing (css, yaml)
//! simply never exercise the CR branches;
//! the CR handling is kept correct here for consumers that classify raw source slices (graphql).

/// Vertical spacing implied by an inter-token source gap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gap {
    /// Same line (no line terminator).
    None,
    /// One or more line breaks, but no blank line.
    Line,
    /// At least one blank line.
    Blank,
}

/// Classifies the gap `slice` between two source positions.
///
/// A blank line is a line strictly inside the gap consisting solely of
/// whitespace. Tokens in the gap make their line non-blank, so newline
/// counting alone would over-report blank lines (e.g. a yaml `-` indicator or
/// a graphql insignificant comma sitting on its own line).
/// Recognizes `\n`, lone `\r`, and `\r\n` line terminators.
pub fn classify_gap(slice: &[u8]) -> Gap {
    let mut newline_count = 0;
    let mut line_has_content = false;
    let mut blank = false;
    let mut i = 0;
    while i < slice.len() {
        match slice[i] {
            b'\r' | b'\n' => {
                // A line strictly between two terminators with no content is blank.
                if newline_count > 0 && !line_has_content {
                    blank = true;
                }
                newline_count += 1;
                line_has_content = false;
                // Collapse `\r\n` into one break.
                if slice[i] == b'\r' && slice.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
            }
            b' ' | b'\t' => {}
            _ => line_has_content = true,
        }
        i += 1;
    }
    if blank {
        Gap::Blank
    } else if newline_count > 0 {
        Gap::Line
    } else {
        Gap::None
    }
}

#[cfg(test)]
mod tests {
    use super::{Gap, classify_gap};

    // CR / CRLF cases matter for consumers that classify raw source slices
    // (`.gitattributes` keeps CR out of fixture files, so they are pinned here).
    #[test]
    fn classify_gap_counts_line_terminators() {
        assert_eq!(classify_gap(b" \t "), Gap::None);
        assert_eq!(classify_gap(b"a"), Gap::None);
        assert_eq!(classify_gap(b"\n"), Gap::Line);
        assert_eq!(classify_gap(b"\n  \n"), Gap::Blank);
        // CRLF must collapse to one break, never two (otherwise blank lines are invented).
        assert_eq!(classify_gap(b"\r\n"), Gap::Line);
        assert_eq!(classify_gap(b"\r\n\r\n"), Gap::Blank);
        // Lone CR is a line terminator.
        assert_eq!(classify_gap(b"\r"), Gap::Line);
        assert_eq!(classify_gap(b"\r\r"), Gap::Blank);
        // Mixed endings.
        assert_eq!(classify_gap(b"\n\r\n"), Gap::Blank);
    }

    #[test]
    fn classify_gap_treats_tokens_as_content() {
        // A token on its own line (a yaml `-` indicator, the graphql `&` between
        // two `implements` comments, an insignificant comma) is not a blank line.
        assert_eq!(classify_gap(b"\n-\n"), Gap::Line);
        assert_eq!(classify_gap(b"\n,\n"), Gap::Line);
        assert_eq!(classify_gap(b",\n\n"), Gap::Blank);
        // Content on the tail of the first or last line is not "inside" the gap.
        assert_eq!(classify_gap(b"-\n  "), Gap::Line);
        assert_eq!(classify_gap(b",\n  "), Gap::Line);
    }
}
