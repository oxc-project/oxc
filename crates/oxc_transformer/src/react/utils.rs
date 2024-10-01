use ropey::Rope;

/// Get line and column from offset and source text.
///
/// Line number starts at 1.
/// Column number is in UTF-16 characters, and starts at 1.
///
/// This matches Babel's output.
pub fn get_line_column(rope: &Rope, offset: u32, source_text: &str) -> (usize, usize) {
    let offset = offset as usize;
    // Get line number and byte offset of start of line
    let line_index = rope.byte_to_line(offset);
    let line_offset = rope.line_to_byte(line_index);
    // Get column number
    let column_index = source_text[line_offset..offset].encode_utf16().count();
    // line and column are zero-indexed, but we want 1-indexed
    (line_index + 1, column_index + 1)
}

#[cfg(test)]
mod test {
    use ropey::Rope;

    fn test_line_column(offset: u32, source_text: &str) -> (usize, usize) {
        let rope = Rope::from_str(source_text);
        super::get_line_column(&rope, offset, source_text)
    }

    #[test]
    fn empty_file() {
        assert_eq!(test_line_column(0, ""), (1, 1));
    }

    #[test]
    fn first_line_start() {
        assert_eq!(test_line_column(0, "foo\nbar\n"), (1, 1));
    }

    #[test]
    fn first_line_middle() {
        assert_eq!(test_line_column(5, "blahblahblah\noops\n"), (1, 6));
    }

    #[test]
    fn later_line_start() {
        assert_eq!(test_line_column(8, "foo\nbar\nblahblahblah"), (3, 1));
    }

    #[test]
    fn later_line_middle() {
        assert_eq!(test_line_column(12, "foo\nbar\nblahblahblah"), (3, 5));
    }

    #[test]
    fn after_2_byte_unicode() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_eq!(test_line_column(4, "£abc"), (1, 4));
    }

    #[test]
    fn after_3_byte_unicode() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_eq!(test_line_column(5, "अabc"), (1, 4));
    }

    #[test]
    fn after_4_byte_unicode() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_eq!(test_line_column(6, "🍄abc"), (1, 5));
    }

    #[test]
    fn after_2_byte_unicode_on_previous_line() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_eq!(test_line_column(4, "£\nabc"), (2, 2));
    }

    #[test]
    fn after_3_byte_unicode_on_previous_line() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_eq!(test_line_column(5, "अ\nabc"), (2, 2));
    }

    #[test]
    fn after_4_byte_unicode_on_previous_line() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_eq!(test_line_column(6, "🍄\nabc"), (2, 2));
    }

    #[cfg(test)]
    fn utf16_len(s: &str) -> usize {
        s.encode_utf16().count()
    }
}
