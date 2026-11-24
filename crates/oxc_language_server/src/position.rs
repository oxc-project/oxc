use tower_lsp_server::gen_lsp_types::Position;

/// Convert a UTF-8 byte offset to an LSP position.
///
/// Returns the same position as [`LineIndex::position`], but scans `source_text`
/// only up to `offset`. Prefer using [`LineIndex`] to convert many offsets.
///
/// # Panics
///
/// Panics if `offset` is out of bounds, does not point to a UTF-8 character
/// boundary in `source_text`, or the computed LSP character offset does not fit
/// in `u32`.
pub fn offset_to_position(source_text: &str, offset: u32) -> Position {
    let offset_usize = usize::try_from(offset).expect("offset must fit in usize");
    assert!(offset_usize <= source_text.len(), "offset out of bounds");
    assert!(source_text.is_char_boundary(offset_usize), "offset is not a char boundary");

    let inside_crlf = is_inside_crlf(source_text.as_bytes(), offset_usize);
    let offset = offset - u32::from(inside_crlf);
    let offset_usize = offset_usize - usize::from(inside_crlf);
    LineIndex::new(&source_text[..offset_usize]).position(offset)
}

/// Whether `offset` is after the CR and before the LF of a CRLF, which LSP cannot express.
/// Such an offset converts as the offset of its CR, as `vscode-languageserver-textdocument`
/// and Oxc's diagnostic renderer do.
fn is_inside_crlf(bytes: &[u8], offset: usize) -> bool {
    offset > 0 && bytes.get(offset - 1) == Some(&b'\r') && bytes.get(offset) == Some(&b'\n')
}

/// The line starts of a text, to convert many of its UTF-8 byte offsets to LSP
/// positions without rescanning the text for each (see [`LineIndex::position`]).
///
/// To convert a single offset, [`offset_to_position`] avoids indexing the entire text.
pub struct LineIndex<'a> {
    text: &'a str,
    /// The byte offset of each line's start; the first is 0.
    line_starts: Vec<u32>,
}

impl<'a> LineIndex<'a> {
    /// Indexes the lines of `text`.
    ///
    /// # Panics
    ///
    /// Panics if `text` is longer than `u32::MAX` bytes.
    pub fn new(text: &'a str) -> Self {
        let bytes = text.as_bytes();
        let mut line_starts = vec![0];
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\r' if bytes.get(i + 1) == Some(&b'\n') => i += 2,
                b'\r' | b'\n' => i += 1,
                _ => {
                    i += 1;
                    continue;
                }
            }
            line_starts.push(u32::try_from(i).expect("text must fit in u32"));
        }
        Self { text, line_starts }
    }

    /// The indexed text.
    pub fn text(&self) -> &'a str {
        self.text
    }

    /// Converts a UTF-8 byte offset of the text to an LSP position.
    ///
    /// LSP positions use zero-based line numbers and UTF-16 character offsets.
    /// The protocol only treats CR, LF, and CRLF as line breaks, so Unicode line
    /// and paragraph separators remain ordinary characters here. An offset
    /// between the CR and LF of a CRLF converts as if the offset of its CR had
    /// been requested.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is out of bounds, does not point to a UTF-8 character
    /// boundary in the text, or the computed LSP character offset does not fit
    /// in `u32`.
    pub fn position(&self, offset: u32) -> Position {
        let offset_usize = usize::try_from(offset).expect("offset must fit in usize");
        assert!(offset_usize <= self.text.len(), "offset out of bounds");
        assert!(self.text.is_char_boundary(offset_usize), "offset is not a char boundary");
        let inside_crlf = is_inside_crlf(self.text.as_bytes(), offset_usize);
        let offset = offset - u32::from(inside_crlf);
        let offset_usize = offset_usize - usize::from(inside_crlf);

        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let line_start = self.line_starts[line] as usize;
        let line_text = &self.text[line_start..offset_usize];
        let character =
            if line_text.is_ascii() { line_text.len() } else { line_text.encode_utf16().count() };

        Position::new(
            u32::try_from(line).expect("line must fit in u32"),
            u32::try_from(character).expect("LSP character offset must fit in u32"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{LineIndex, offset_to_position};

    /// Checks both `offset_to_position`, which indexes the text only up to `offset`, and a
    /// `LineIndex` of the whole text, which must also look past lines after `offset`.
    fn assert_position(source_text: &str, offset: usize, expected: (u32, u32)) {
        let offset = u32::try_from(offset).expect("test offset must fit in u32");

        let position = offset_to_position(source_text, offset);
        assert_eq!((position.line, position.character), expected, "offset_to_position");

        let position = LineIndex::new(source_text).position(offset);
        assert_eq!((position.line, position.character), expected, "LineIndex::position");
    }

    #[test]
    fn test_line_break() {
        for nl in ["\n", "\r", "\r\n"] {
            let source = format!("a{nl}b");
            assert_position(&source, source.find('b').unwrap(), (1, 0));
        }

        for nl in ["\u{2028}", "\u{2029}"] {
            let source = format!("a{nl}b");
            assert_position(&source, source.find('b').unwrap(), (0, 2));
        }
    }

    #[test]
    fn offset_between_cr_and_lf_is_end_of_line() {
        let source = "a\r\nb";
        assert_position(source, 1, (0, 1));
        assert_position(source, 2, (0, 1));
        assert_position(source, 3, (1, 0));

        // Followed by more lines, so a whole-text index has line starts after the offset.
        let source = "a\r\nb\r\nc";
        assert_position(source, 5, (1, 1));
        assert_position(source, 6, (2, 0));

        // A lone CR before the CRLF is its own line break.
        assert_position("\r\r\n", 2, (1, 0));

        // The column of the CR counts UTF-16 units, like any other column.
        assert_position("😀\r\n", 5, (0, 2));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn panics_for_out_of_bounds_offset() {
        offset_to_position("foo", 100);
    }

    #[test]
    fn empty_file() {
        assert_position("", 0, (0, 0));
    }

    #[test]
    fn first_line_start() {
        assert_position("foo\nbar\n", 0, (0, 0));
    }

    #[test]
    fn first_line_middle() {
        assert_position("blahblahblah\noops\n", 5, (0, 5));
    }

    #[test]
    fn later_line_start() {
        assert_position("foo\nbar\nblahblahblah", 8, (2, 0));
    }

    #[test]
    fn later_line_middle() {
        assert_position("foo\nbar\nblahblahblah", 12, (2, 4));
    }

    #[test]
    fn after_2_byte_unicode() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_position("£abc", 4, (0, 3));
    }

    #[test]
    fn after_3_byte_unicode() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_position("अabc", 5, (0, 3));
    }

    #[test]
    fn after_4_byte_unicode() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_position("🍄abc", 6, (0, 4));
    }

    #[test]
    fn after_2_byte_unicode_on_previous_line() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_position("£\nabc", 4, (1, 1));
    }

    #[test]
    fn after_3_byte_unicode_on_previous_line() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_position("अ\nabc", 5, (1, 1));
    }

    #[test]
    fn after_4_byte_unicode_on_previous_line() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_position("🍄\nabc", 6, (1, 1));
    }

    #[test]
    fn single_line() {
        let source = "foo.bar!;";
        assert_position(source, 0, (0, 0));
        assert_position(source, 4, (0, 4));
        assert_position(source, 9, (0, 9));
    }

    #[test]
    fn multi_line() {
        let source = "console.log(\n  foo.bar!\n);";
        assert_position(source, 0, (0, 0));
        assert_position(source, 12, (0, 12));
        assert_position(source, 13, (1, 0));
        assert_position(source, 23, (1, 10));
        assert_position(source, 24, (2, 0));
        assert_position(source, 26, (2, 2));
    }

    #[test]
    fn multi_byte() {
        let source = "let foo = \n  '👍';";
        assert_position(source, 10, (0, 10));
        assert_position(source, 11, (1, 0));
        assert_position(source, 14, (1, 3));
        assert_position(source, 18, (1, 5));
        assert_position(source, 19, (1, 6));
    }

    #[test]
    fn unicode_line_and_paragraph_separators_are_not_lsp_line_breaks() {
        let source = "a\u{2028}b\nc\u{2029}d";
        assert_position(source, source.find('b').unwrap(), (0, 2));
        assert_position(source, source.find('c').unwrap(), (1, 0));
        assert_position(source, source.find('d').unwrap(), (1, 2));
    }

    #[cfg(test)]
    fn utf16_len(s: &str) -> usize {
        s.encode_utf16().count()
    }

    #[test]
    fn line_index_line_starts() {
        for (text, expected) in [
            ("", &[0][..]),
            ("a\nb", &[0, 2]),
            ("a\rb", &[0, 2]),
            ("a\r\nb", &[0, 3]),
            ("\r\r\n", &[0, 1, 3]),
            ("\n\n", &[0, 1, 2]),
            ("a\u{2028}b\u{2029}c", &[0]),
            ("line\n", &[0, 5]),
            ("a\r", &[0, 2]),
            ("a\r\n", &[0, 3]),
            ("é\n😀\r\nx", &[0, 3, 9]),
        ] {
            assert_eq!(LineIndex::new(text).line_starts, expected, "{text:?}");
        }
    }
}
