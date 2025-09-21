use oxc_allocator::Allocator;
use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_formatter::{FormatOptions, Formatter, get_supported_source_type};
use oxc_parser::{ParseOptions, Parser};
use tower_lsp_server::{
    UriExt,
    lsp_types::{Position, Range, TextEdit, Uri},
};

pub struct ServerFormatter;

impl ServerFormatter {
    pub fn new() -> Self {
        Self {}
    }

    #[expect(clippy::unused_self)]
    pub fn run_single(&self, uri: &Uri, content: Option<String>) -> Option<Vec<TextEdit>> {
        let path = uri.to_file_path()?;
        let source_type = get_supported_source_type(&path)?;
        let source_text = if let Some(content) = content {
            content
        } else {
            std::fs::read_to_string(&path).ok()?
        };

        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: false,
                // Enable all syntax features
                allow_v8_intrinsics: true,
                allow_return_outside_function: true,
                // `oxc_formatter` expects this to be false
                preserve_parens: false,
            })
            .parse();

        if !ret.errors.is_empty() {
            return None;
        }

        let options = FormatOptions::default();
        let code = Formatter::new(&allocator, options).build(&ret.program);

        // nothing has changed
        if code == source_text {
            return Some(vec![]);
        }

        let (start, end, replacement) = compute_minimal_text_edit(&source_text, &code);
        let rope = Rope::from(source_text.as_str());
        let (start_line, start_character) = get_line_column(&rope, start, &source_text);
        let (end_line, end_character) = get_line_column(&rope, end, &source_text);

        Some(vec![TextEdit::new(
            Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            replacement.to_string(),
        )])
    }
}

/// Returns the minimal text edit (start, end, replacement) to transform `source_text` into `formatted_text`
#[expect(clippy::cast_possible_truncation)]
fn compute_minimal_text_edit<'a>(
    source_text: &str,
    formatted_text: &'a str,
) -> (u32, u32, &'a str) {
    debug_assert!(source_text != formatted_text);

    // Find common prefix (byte offset)
    let mut prefix_byte = 0;
    for (a, b) in source_text.chars().zip(formatted_text.chars()) {
        if a == b {
            prefix_byte += a.len_utf8();
        } else {
            break;
        }
    }

    // Find common suffix (byte offset from end)
    let mut suffix_byte = 0;
    let src_bytes = source_text.as_bytes();
    let fmt_bytes = formatted_text.as_bytes();
    let src_len = src_bytes.len();
    let fmt_len = fmt_bytes.len();

    while suffix_byte < src_len - prefix_byte
        && suffix_byte < fmt_len - prefix_byte
        && src_bytes[src_len - 1 - suffix_byte] == fmt_bytes[fmt_len - 1 - suffix_byte]
    {
        suffix_byte += 1;
    }

    let start = prefix_byte as u32;
    let end = (src_len - suffix_byte) as u32;
    let replacement_start = prefix_byte;
    let replacement_end = fmt_len - suffix_byte;
    let replacement = &formatted_text[replacement_start..replacement_end];

    (start, end, replacement)
}

#[cfg(test)]
mod tests {
    use super::compute_minimal_text_edit;
    use crate::formatter::{options::FormatOptions, tester::Tester};

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_no_change() {
        let src = "abc";
        let formatted = "abc";
        compute_minimal_text_edit(src, formatted);
    }

    #[test]
    fn test_single_char_change() {
        let src = "abc";
        let formatted = "axc";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Only 'b' replaced by 'x'
        assert_eq!((start, end, replacement), (1, 2, "x"));
    }

    #[test]
    fn test_insert_char() {
        let src = "abc";
        let formatted = "abxc";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Insert 'x' after 'b'
        assert_eq!((start, end, replacement), (2, 2, "x"));
    }

    #[test]
    fn test_delete_char() {
        let src = "abc";
        let formatted = "ac";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Delete 'b'
        assert_eq!((start, end, replacement), (1, 2, ""));
    }

    #[test]
    fn test_replace_multiple_chars() {
        let src = "abcdef";
        let formatted = "abXYef";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Replace "cd" with "XY"
        assert_eq!((start, end, replacement), (2, 4, "XY"));
    }

    #[test]
    fn test_replace_multiple_chars_between_similars_complex() {
        let src = "aYabYb";
        let formatted = "aXabXb";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        assert_eq!((start, end, replacement), (1, 5, "XabX"));
    }

    #[test]
    fn test_unicode() {
        let src = "a😀b";
        let formatted = "a😃b";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Replace 😀 with 😃
        assert_eq!((start, end, replacement), (1, 5, "😃"));
    }

    #[test]
    fn test_append() {
        let src = "a".repeat(100);
        let mut formatted = src.clone();
        formatted.push('b'); // Add a character at the end

        let (start, end, replacement) = compute_minimal_text_edit(&src, &formatted);
        assert_eq!((start, end, replacement), (100, 100, "b"));
    }

    #[test]
    fn test_prepend() {
        let src = "a".repeat(100);
        let mut formatted = String::from("b");
        formatted.push_str(&src); // Add a character at the start

        let (start, end, replacement) = compute_minimal_text_edit(&src, &formatted);
        assert_eq!((start, end, replacement), (0, 0, "b"));
    }

    #[test]
    #[cfg(not(windows))] // FIXME: snapshot differs on Windows. Possible newline issue?
    fn test_formatter() {
        Tester::new("fixtures/formatter/basic", Some(FormatOptions { experimental: true }))
            .format_and_snapshot_single_file("basic.ts");
    }
}
