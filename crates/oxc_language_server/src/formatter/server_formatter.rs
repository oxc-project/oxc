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
        let (start_line, start_character) = get_line_column(&rope, start, source_text.as_str());
        let (end_line, end_character) = get_line_column(&rope, end, source_text.as_str());

        Some(vec![TextEdit::new(
            Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            replacement.to_string(),
        )])
    }
}

/// Returns a Vec of (start, end, replacement) for all minimal edits.
/// Spans are in UTF-8 byte offsets.
fn compute_minimal_text_edit<'a>(
    source_text: &str,
    formatted_text: &'a str,
) -> (u32, u32, &'a str) {
    debug_assert!(source_text != formatted_text);

    let source_chars: Vec<(usize, char)> = source_text.char_indices().collect();
    let formatted_chars: Vec<(usize, char)> = formatted_text.char_indices().collect();

    let source_len = source_chars.len();
    let formatted_len = formatted_chars.len();

    // Find length of common prefix (in chars)
    let mut prefix = 0;
    while prefix < source_len
        && prefix < formatted_len
        && source_chars[prefix].1 == formatted_chars[prefix].1
    {
        prefix += 1;
    }

    // Find length of common suffix (in chars)
    let mut suffix = 0;
    while prefix + suffix < source_len
        && prefix + suffix < formatted_len
        && source_chars[source_len - 1 - suffix].1 == formatted_chars[formatted_len - 1 - suffix].1
    {
        suffix += 1;
    }

    // Compute byte indices for slicing
    let start = if prefix < source_len { source_chars[prefix].0 } else { source_text.len() };
    let end =
        if suffix < source_len { source_chars[source_len - suffix].0 } else { source_text.len() };

    let replacement_start =
        if prefix < formatted_len { formatted_chars[prefix].0 } else { formatted_text.len() };
    let replacement_end = if suffix < formatted_len {
        formatted_chars[formatted_len - suffix].0
    } else {
        formatted_text.len()
    };

    let replacement = &formatted_text[replacement_start..replacement_end];

    (start as u32, end as u32, replacement)
}

#[cfg(test)]
mod tests {
    use super::compute_minimal_text_edit;

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
}
