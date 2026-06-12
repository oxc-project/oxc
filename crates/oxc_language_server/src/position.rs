use tower_lsp_server::ls_types::Position;

/// Convert a UTF-8 byte offset to an LSP position.
///
/// LSP positions use zero-based line numbers and UTF-16 character offsets. The
/// protocol only treats CR, LF, and CRLF as line breaks, so Unicode line and
/// paragraph separators must remain ordinary characters here.
///
/// # Panics
///
/// Panics if `offset` is out of bounds, does not point to a UTF-8 character
/// boundary in `source_text`, or the computed LSP character offset does not fit
/// in `u32`.
pub fn offset_to_position(source_text: &str, offset: u32) -> Position {
    let offset = usize::try_from(offset).expect("offset must fit in usize");
    assert!(offset <= source_text.len(), "offset out of bounds");
    assert!(source_text.is_char_boundary(offset), "offset is not a char boundary");

    let bytes = source_text.as_bytes();
    let mut line = 0;
    let mut line_start = 0;
    let mut i = 0;

    while i < offset {
        match bytes[i] {
            b'\r' => {
                i += if i + 1 < offset && bytes[i + 1] == b'\n' { 2 } else { 1 };
                line += 1;
                line_start = i;
            }
            b'\n' => {
                i += 1;
                line += 1;
                line_start = i;
            }
            _ => {
                let ch = source_text[i..].chars().next().expect("valid char boundary");
                i += ch.len_utf8();
            }
        }
    }

    let character = u32::try_from(source_text[line_start..offset].encode_utf16().count())
        .expect("LSP character offset must fit in u32");
    Position::new(line, character)
}

#[cfg(test)]
mod tests {
    use super::offset_to_position;

    fn assert_position(source_text: &str, offset: usize, expected: (u32, u32)) {
        let position = offset_to_position(
            source_text,
            u32::try_from(offset).expect("test offset must fit in u32"),
        );
        assert_eq!((position.line, position.character), expected);
    }

    #[test]
    fn uses_lsp_line_breaks() {
        let source = "a\u{2028}b\nc\u{2029}d";

        assert_position(source, source.find('b').unwrap(), (0, 2));
        assert_position(source, source.find('c').unwrap(), (1, 0));
        assert_position(source, source.find('d').unwrap(), (1, 2));
    }

    #[test]
    fn treats_crlf_as_one_line_break() {
        let source = "a\r\nb";

        assert_position(source, source.find('b').unwrap(), (1, 0));
    }

    #[test]
    fn reports_utf16_character_offsets() {
        let source = "😀a";

        assert_position(source, source.find('a').unwrap(), (0, 2));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn panics_for_out_of_bounds_offset() {
        offset_to_position("foo", 100);
    }
}
