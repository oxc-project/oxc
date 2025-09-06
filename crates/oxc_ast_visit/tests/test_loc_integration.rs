#![cfg(all(test, feature = "serialize"))]

use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;

#[test]
fn test_loc_integration() {
    let source_text = "hello\nworld\n🤨";

    // Test UTF8 to UTF16 conversion table with line support
    let converter = Utf8ToUtf16::new_with_lines(source_text, true);

    // Test line/column conversion
    assert_eq!(converter.offset_to_line_column(0), Some((0, 0))); // start of 'hello'
    assert_eq!(converter.offset_to_line_column(5), Some((0, 5))); // end of 'hello'
    assert_eq!(converter.offset_to_line_column(6), Some((1, 0))); // start of 'world'
    assert_eq!(converter.offset_to_line_column(11), Some((1, 5))); // end of 'world'
    assert_eq!(converter.offset_to_line_column(12), Some((2, 0))); // unicode char
}

#[test]
fn test_different_line_endings() {
    // Test with \r\n line endings
    let source_crlf = "hello\r\nworld";
    let converter_crlf = Utf8ToUtf16::new_with_lines(source_crlf, true);
    assert_eq!(converter_crlf.offset_to_line_column(0), Some((0, 0)));
    assert_eq!(converter_crlf.offset_to_line_column(7), Some((1, 0))); // start of 'world'

    // Test with \r line endings
    let source_cr = "hello\rworld";
    let converter_cr = Utf8ToUtf16::new_with_lines(source_cr, true);
    assert_eq!(converter_cr.offset_to_line_column(0), Some((0, 0)));
    assert_eq!(converter_cr.offset_to_line_column(6), Some((1, 0))); // start of 'world'

    // Test with Unicode line separators
    let source_unicode = "hello\u{2028}world";
    let converter_unicode = Utf8ToUtf16::new_with_lines(source_unicode, true);
    assert_eq!(converter_unicode.offset_to_line_column(0), Some((0, 0)));
    assert_eq!(converter_unicode.offset_to_line_column(8), Some((1, 0))); // start of 'world'
}

#[test]
fn test_empty_and_single_line() {
    // Test empty source
    let empty_source = "";
    let converter_empty = Utf8ToUtf16::new_with_lines(empty_source, true);
    assert_eq!(converter_empty.offset_to_line_column(0), Some((0, 0)));

    // Test single line
    let single_line = "hello world";
    let converter_single = Utf8ToUtf16::new_with_lines(single_line, true);
    assert_eq!(converter_single.offset_to_line_column(0), Some((0, 0)));
    assert_eq!(converter_single.offset_to_line_column(5), Some((0, 5)));
    assert_eq!(converter_single.offset_to_line_column(11), Some((0, 11)));
}
