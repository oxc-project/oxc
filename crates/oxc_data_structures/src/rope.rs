//! Rope

pub use ropey::Rope;

/// Get UTF16 line and column from UTF8 offset and source text.
#[expect(clippy::cast_possible_truncation)]
pub fn get_line_column(rope: &Rope, offset: u32, source_text: &str) -> (u32, u32) {
    let offset = offset as usize;
    // Get line number and byte offset of start of line
    let line_index = rope.byte_to_line(offset);
    let line_offset = rope.line_to_byte(line_index);
    // Get column number
    let column_index = source_text[line_offset..offset].encode_utf16().count();
    (line_index as u32, column_index as u32)
}

#[cfg(test)]
mod test {
    use ropey::Rope;

    fn test_line_column(offset: u32, source_text: &str) -> (u32, u32) {
        let rope = Rope::from_str(source_text);
        super::get_line_column(&rope, offset, source_text)
    }

    #[test]
    fn empty_file() {
        assert_eq!(test_line_column(0, ""), (0, 0));
    }

    #[test]
    fn first_line_start() {
        assert_eq!(test_line_column(0, "foo\nbar\n"), (0, 0));
    }

    #[test]
    fn first_line_middle() {
        assert_eq!(test_line_column(5, "blahblahblah\noops\n"), (0, 5));
    }

    #[test]
    fn later_line_start() {
        assert_eq!(test_line_column(8, "foo\nbar\nblahblahblah"), (2, 0));
    }

    #[test]
    fn later_line_middle() {
        assert_eq!(test_line_column(12, "foo\nbar\nblahblahblah"), (2, 4));
    }

    #[test]
    fn after_2_byte_unicode() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_eq!(test_line_column(4, "£abc"), (0, 3));
    }

    #[test]
    fn after_3_byte_unicode() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_eq!(test_line_column(5, "अabc"), (0, 3));
    }

    #[test]
    fn after_4_byte_unicode() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_eq!(test_line_column(6, "🍄abc"), (0, 4));
    }

    #[test]
    fn after_2_byte_unicode_on_previous_line() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_eq!(test_line_column(4, "£\nabc"), (1, 1));
    }

    #[test]
    fn after_3_byte_unicode_on_previous_line() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_eq!(test_line_column(5, "अ\nabc"), (1, 1));
    }

    #[test]
    fn after_4_byte_unicode_on_previous_line() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_eq!(test_line_column(6, "🍄\nabc"), (1, 1));
    }

    #[cfg(test)]
    fn utf16_len(s: &str) -> usize {
        s.encode_utf16().count()
    }
}
