/// Compute the width of a string matching JavaScript's `.length` (UTF-16 code units).
/// This intentionally does NOT use `unicode_width::UnicodeWidthStr` because Prettier
/// measures `printWidth` in JS string length (UTF-16 code units), not terminal columns.
///
/// Fast path: for ASCII-only strings (99%+ of JSDoc content), `len()` equals UTF-16 count.
#[inline]
pub fn str_width(s: &str) -> usize {
    if s.is_ascii() { s.len() } else { s.encode_utf16().count() }
}

/// Lookup table of pre-allocated indent strings (0–12 spaces).
/// Avoids `" ".repeat(n)` heap allocations for common indent widths.
const INDENTS: [&str; 13] = [
    "",
    " ",
    "  ",
    "   ",
    "    ",
    "     ",
    "      ",
    "       ",
    "        ",
    "         ",
    "          ",
    "           ",
    "            ",
];

/// Get an indent string of `n` spaces. Uses a static lookup for n <= 12,
/// falls back to heap allocation for larger values.
pub fn indent_str(n: usize) -> std::borrow::Cow<'static, str> {
    if let Some(&s) = INDENTS.get(n) {
        std::borrow::Cow::Borrowed(s)
    } else {
        std::borrow::Cow::Owned(" ".repeat(n))
    }
}

/// Check if a line looks like a table separator row (e.g. `| --- | --- | --- |`).
fn is_table_separator(line: &str) -> bool {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    if inner.is_empty() {
        return false;
    }
    inner.split('|').all(|cell| {
        let cell = cell.trim();
        !cell.is_empty()
            && cell.chars().all(|c| c == '-' || c == ':' || c == ' ')
            && cell.contains('-')
    })
}

/// Parse a table row into cells.
fn parse_table_cells(line: &str) -> Vec<&str> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(str::trim).collect()
}

/// Format a block of consecutive table lines.
/// If the table has a valid separator row, format with column padding.
/// Otherwise, output as-is.
///
pub fn format_table_block(table_lines: &[&str]) -> Vec<String> {
    let to_owned = || table_lines.iter().map(|l| String::from(*l)).collect();

    // Find separator row
    let Some(separator_idx) = table_lines.iter().position(|l| is_table_separator(l)) else {
        return to_owned();
    };

    // Parse all data rows into cells
    let all_cells: Vec<Vec<&str>> = table_lines
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != separator_idx)
        .map(|(_, l)| parse_table_cells(l))
        .collect();

    if all_cells.is_empty() {
        return to_owned();
    }

    // Determine number of columns
    let num_cols = all_cells.iter().map(std::vec::Vec::len).max().unwrap_or(0);
    if num_cols == 0 {
        return to_owned();
    }

    // Compute max width per column
    let mut col_widths = vec![3usize; num_cols]; // minimum 3 for "---"
    for row in &all_cells {
        for (j, cell) in row.iter().enumerate() {
            if j < num_cols {
                col_widths[j] = col_widths[j].max(str_width(cell));
            }
        }
    }

    // Total width per row: "| " + (cell_width + " | ") * num_cols
    let row_capacity: usize = 2 + col_widths.iter().map(|&w| w + 3).sum::<usize>();

    // Format each row, reusing already-parsed cells from all_cells
    let mut lines = Vec::with_capacity(table_lines.len());
    let mut data_row_idx = 0;
    for (idx, _) in table_lines.iter().enumerate() {
        let mut row = String::with_capacity(row_capacity);
        if idx == separator_idx {
            let sep_cells = parse_table_cells(table_lines[separator_idx]);
            for (j, &w) in col_widths.iter().enumerate() {
                row.push_str(if j == 0 { "| " } else { " | " });
                let cell = sep_cells.get(j).copied().unwrap_or("---");
                let left_align = cell.starts_with(':');
                let right_align = cell.ends_with(':');
                if left_align {
                    row.push(':');
                }
                let dashes = w - usize::from(left_align) - usize::from(right_align);
                for _ in 0..dashes {
                    row.push('-');
                }
                if right_align {
                    row.push(':');
                }
            }
            row.push_str(" |");
        } else {
            let cells = &all_cells[data_row_idx];
            data_row_idx += 1;
            for (j, &width) in col_widths.iter().enumerate() {
                row.push_str(if j == 0 { "| " } else { " | " });
                let cell = cells.get(j).copied().unwrap_or("");
                row.push_str(cell);
                // Pad with spaces to column width
                for _ in str_width(cell)..width {
                    row.push(' ');
                }
            }
            row.push_str(" |");
        }
        lines.push(row);
    }
    lines
}

/// Find the last space in the text that's not inside a `{@link ...}` construct.
/// Returns `None` if no such space exists. This prevents the overflow
/// post-processing from splitting `{@link EncryptedDbChange}` into
/// `{@link` / `EncryptedDbChange}`.
fn find_last_breakable_space(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut last_space = None;
    let mut i = 0;
    while i < len {
        if bytes[i] == b'{' && i + 1 < len && bytes[i + 1] == b'@' {
            // Skip the entire {@link ...} construct
            let mut depth = 1;
            i += 2;
            while i < len && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            // Skip trailing non-whitespace (punctuation attached to the link)
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        } else {
            if bytes[i] == b' ' {
                last_space = Some(i);
            }
            i += 1;
        }
    }
    last_space
}

/// Tokenize text into words, treating `{@link ...}` and markdown `[text](url)` as atomic tokens.
pub fn tokenize_words(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Skip leading whitespace
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    while i < len {
        // Check for `{@link`, `{@linkcode`, `{@linkplain`, `{@tutorial`
        if bytes[i] == b'{' && i + 1 < len && bytes[i + 1] == b'@' {
            let start = i;
            // Find matching closing `}`
            let mut depth = 1;
            i += 2;
            while i < len && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            // Include trailing non-whitespace text as part of the {@link} token.
            // This prevents wrapping from splitting `{@link collect}ed` into
            // `{@link collect} ed` — the suffix must stay attached.
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(&text[start..i]);
            // Skip whitespace after token
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            continue;
        }

        // Regular word: advance to next whitespace
        let start = i;
        while i < len && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i > start {
            tokens.push(&text[start..i]);
        }
        // Skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
    }

    tokens
}

/// Compute the width of a token for wrapping purposes.
/// Uses the full character width of the token, including `{@link ...}` syntax,
/// since the output retains the full syntax and must fit within printWidth.
fn link_rendered_width(token: &str) -> usize {
    str_width(token)
}

/// Compute the syntax overhead of an inline tag token like `{@link Foo}`.
///
/// The rendered output of `{@link Foo}` is just "Foo", so the `{@link }` wrapper
/// adds characters that don't contribute to the visual width. Upstream Prettier's
/// JSDoc plugin is more lenient about line breaks when the overflow is caused by
/// inline tag syntax — it effectively allows the line to exceed `printWidth` by
/// the amount of the tag wrapper overhead.
///
/// Returns 0 for non-inline-tag tokens.
fn inline_tag_overhead(token: &str) -> usize {
    let bytes = token.as_bytes();
    // Must start with `{@`
    if bytes.len() < 3 || bytes[0] != b'{' || bytes[1] != b'@' {
        return 0;
    }
    // Find the space after the tag name (e.g., `{@link ` or `{@linkcode `)
    // The overhead is the tag prefix length + 1 for the closing `}`
    if let Some(space_pos) = token[2..].find(' ') {
        // overhead = "{@tagname " (space_pos + 3) + "}" (1)
        // But only if the token actually ends with `}` (possibly with trailing punctuation)
        let core = token.trim_end_matches(|c: char| c != '}');
        if core.ends_with('}') {
            return space_pos + 3 + 1; // "{@tagname " + "}"
        }
    }
    0
}

/// Wrap a single paragraph of plain text to the given max width with optional indent for
/// continuation lines.
///
/// `first_line_offset` reduces the first line's capacity (e.g., when the paragraph starts
/// mid-line after a tag prefix like `@param {type} name - `).
///
/// Uses word-by-word greedy approach with `tokenize_words` to preserve atomic tokens
/// like `{@link ...}`. After building all lines, applies a post-processing step to match
/// the prettier-plugin-jsdoc `breakDescriptionToLines` behavior: when the last
/// continuation line has exactly `effective_max` characters, the plugin's `\n`-prefix
/// causes one more wrap iteration, splitting the last word to the next line.
pub fn wrap_paragraph(
    text: &str,
    max_width: usize,
    first_line_offset: usize,
    continuation_indent: usize,
    lines: &mut super::line_buffer::LineBuffer,
) {
    if text.is_empty() {
        return;
    }

    let words = tokenize_words(text);
    if words.is_empty() {
        return;
    }

    let indent_s = indent_str(continuation_indent);
    let first_line_max = max_width.saturating_sub(first_line_offset);
    let effective_max = max_width.saturating_sub(continuation_indent);
    let mut current_line = String::with_capacity(max_width);
    let mut current_width: usize = 0;
    let mut is_first_line = true;
    // Count inline tags on the current line. Each tag adds ~8 chars of syntax
    // overhead that doesn't contribute to visual width in rendered docs.
    let mut current_line_tag_count: usize = 0;

    for word in &words {
        let word_width = link_rendered_width(word);
        let is_inline_tag = inline_tag_overhead(word) > 0;
        let capacity = if is_first_line { first_line_max } else { effective_max };

        if current_line.is_empty() {
            current_line.push_str(word);
            current_width = word_width;
            current_line_tag_count = usize::from(is_inline_tag);
        } else if current_width + 1 + word_width <= capacity || {
            // Tolerance: allow ~1 char of overflow per inline tag on the line
            // (including the current word if it's a tag). Upstream Prettier's
            // JSDoc plugin is more lenient about line width when the line
            // contains `{@link ...}` tokens whose syntax overhead inflates
            // the raw character count beyond the visual width.
            let tag_count = current_line_tag_count + usize::from(is_inline_tag);
            tag_count > 0 && current_width + 1 + word_width <= capacity + tag_count
        } {
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
            current_line_tag_count += usize::from(is_inline_tag);
        } else {
            // Word doesn't fit, push current line and start new one
            if is_first_line {
                lines.push(&current_line);
                current_line.clear();
                is_first_line = false;
            } else {
                let s = lines.begin_line();
                s.push_str(&indent_s);
                s.push_str(&current_line);
                current_line.clear();
            }
            current_line.push_str(word);
            current_width = word_width;
            current_line_tag_count = usize::from(is_inline_tag);
        }
    }

    if !current_line.is_empty() {
        // Post-processing: match plugin's `>=` boundary behavior for continuation lines.
        // The plugin prepends `\n` to continuation text, which causes `str.length >= maxWidth`
        // to trigger an extra wrap when remaining content is exactly `maxWidth` characters.
        if !is_first_line
            && current_width == effective_max
            && let Some(last_space) = find_last_breakable_space(&current_line)
        {
            let overflow_start = last_space + 1;
            {
                let s = lines.begin_line();
                s.push_str(&indent_s);
                s.push_str(&current_line[..last_space]);
            }
            {
                let s = lines.begin_line();
                s.push_str(&indent_s);
                s.push_str(&current_line[overflow_start..]);
            }
            return;
        }

        if is_first_line {
            lines.push(&current_line);
        } else {
            let s = lines.begin_line();
            s.push_str(&indent_s);
            s.push_str(&current_line);
        }
    }
}

/// Wrap plain text with paragraph breaks into lines.
/// For text with no markdown constructs — just paragraphs separated by blank lines.
pub fn wrap_plain_paragraphs(text: &str, max_width: usize) -> String {
    let mut lines = super::line_buffer::LineBuffer::new();
    let mut paragraph = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !paragraph.is_empty() {
                wrap_paragraph(paragraph.trim(), max_width, 0, 0, &mut lines);
                paragraph.clear();
            }
            if !lines.last_is_empty() {
                lines.push_empty();
            }
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(trimmed);
        }
    }
    if !paragraph.is_empty() {
        wrap_paragraph(paragraph.trim(), max_width, 0, 0, &mut lines);
    }
    lines.into_string()
}

/// Balance mode variant of `wrap_plain_paragraphs`.
/// For each paragraph, if the original line breaks result in all lines fitting within
/// `max_width`, preserve the original breaks. Otherwise fall back to greedy wrapping.
pub fn wrap_plain_paragraphs_balance(text: &str, max_width: usize) -> String {
    let mut lines = super::line_buffer::LineBuffer::new();
    // Collect paragraphs with their original lines
    let mut para_lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !para_lines.is_empty() {
                flush_paragraph_balance(&para_lines, max_width, &mut lines);
                para_lines.clear();
            }
            if !lines.last_is_empty() {
                lines.push_empty();
            }
        } else {
            para_lines.push(trimmed);
        }
    }
    if !para_lines.is_empty() {
        flush_paragraph_balance(&para_lines, max_width, &mut lines);
    }
    lines.into_string()
}

fn flush_paragraph_balance(
    original_lines: &[&str],
    max_width: usize,
    lines: &mut super::line_buffer::LineBuffer,
) {
    // If multiple lines and all fit, preserve original breaks
    if original_lines.len() > 1 && original_lines.iter().all(|l| str_width(l) <= max_width) {
        for l in original_lines {
            lines.push(l);
        }
    } else {
        // Fall back to greedy wrapping
        let joined: String = original_lines.join(" ");
        wrap_paragraph(joined.trim(), max_width, 0, 0, lines);
    }
}

/// Wrap text into lines, preserving structured content (lists, code blocks, tables, etc.)
/// and wrapping plain paragraphs to the given max width.
///
/// Delegates to `format_description_mdast` for full markdown-aware formatting.
pub fn wrap_text(
    text: &str,
    max_width: usize,
    tag_string_length: usize,
    capitalize: bool,
    format_options: Option<&crate::FormatOptions>,
    allocator: Option<&oxc_allocator::Allocator>,
) -> String {
    if text.is_empty() {
        return String::new();
    }
    super::mdast_serialize::format_description_mdast(
        text,
        max_width,
        tag_string_length,
        capitalize,
        format_options,
        allocator,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_simple_text() {
        let result = wrap_text("This is a short line", 80, 0, false, None, None);
        assert_eq!(result, "This is a short line");
    }

    #[test]
    fn test_wrap_long_text() {
        let result = wrap_text(
            "This is a long line that should be wrapped because it exceeds the maximum width",
            40,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "This is a long line that should be\n\
             wrapped because it exceeds the maximum\n\
             width"
        );
    }

    #[test]
    fn test_wrap_preserves_markdown_list() {
        let result =
            wrap_text("- item one\n- item two\n- item three", 80, 0, false, None, None);
        assert_eq!(result, "- item one\n- item two\n- item three");
    }

    #[test]
    fn test_wrap_list_item_with_continuation() {
        let result = wrap_text(
            "- This is a very long list item that should be wrapped to the next line with proper indent",
            40,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "- This is a very long list item that\n  should be wrapped to the next line\n  with proper indent"
        );
    }

    #[test]
    fn test_wrap_converts_code_fence_to_indented() {
        let result = wrap_text(
            "Some text\n```\ncode here\n  indented\n```\nMore text",
            80,
            0,
            false,
            None,
            None,
        );
        // Fenced code without language tag is converted to indented code block.
        assert_eq!(result, "Some text\n\n    code here\n      indented\n\nMore text");
    }

    #[test]
    fn test_wrap_preserves_code_fence_with_language() {
        let result = wrap_text(
            "Some text\n```js\nconst x = 1;\n```\nMore text",
            80,
            0,
            false,
            None,
            None,
        );
        assert_eq!(result, "Some text\n\n```js\nconst x = 1;\n```\n\nMore text");
    }

    #[test]
    fn test_wrap_empty_lines() {
        let result = wrap_text("Paragraph one\n\nParagraph two", 80, 0, false, None, None);
        assert_eq!(result, "Paragraph one\n\nParagraph two");
    }

    #[test]
    fn test_wrap_empty_text() {
        let result = wrap_text("", 80, 0, false, None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_numbered_spread_list_collapses_blank_lines() {
        // Upstream prettier-plugin-jsdoc removes blank lines between list items
        // even when the source has them (spread lists).
        let result =
            wrap_text("1. Thing 1\n\n2. Thing 2\n\n3. Thing 3", 80, 0, false, None, None);
        assert_eq!(result, "1. Thing 1\n2. Thing 2\n3. Thing 3");
    }

    #[test]
    fn test_list_item_wrapping_at_boundary() {
        let result = wrap_text(
            "- Consider caching this for the lifetime of the component, or possibly being able to share this cache between any `ScrollMap` view.",
            77,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "- Consider caching this for the lifetime of the component, or possibly being\n  able to share this cache between any `ScrollMap` view."
        );
    }

    #[test]
    fn test_list_multiline_input() {
        // Test that multi-line list items from JSDoc are joined correctly
        let result = wrap_text(
            "- Consider caching this for the lifetime of the component, or possibly being able to share this\ncache between any `ScrollMap` view.",
            77,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "- Consider caching this for the lifetime of the component, or possibly being\n  able to share this cache between any `ScrollMap` view."
        );
    }

    #[test]
    fn test_wrap_link_uses_full_width() {
        // {@link type} is 12 chars, not 4 — wrapping must account for the full syntax
        let result = wrap_text(
            "The `string` values within a renderer are always associated with the {@link type} of that renderer. To switch types, call {@link child} with a different `type` argument.",
            97,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "The `string` values within a renderer are always associated with the {@link type} of that\nrenderer. To switch types, call {@link child} with a different `type` argument."
        );
    }

    #[test]
    fn test_wrap_link_overflow_tolerance() {
        // Upstream Prettier allows lines with {@link} tags to slightly overflow printWidth.
        // Each inline tag contributes ~1 char of tolerance.

        // 1 tag, overflow by 1: should keep on one line
        let result = wrap_text(
            "Checks if an array is non-empty and narrows its type to {@link NonEmptyArray}.",
            77,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "Checks if an array is non-empty and narrows its type to {@link NonEmptyArray}."
        );

        // 2 tags, overflow by 2 (last word is regular): should keep on one line
        let result = wrap_text(
            "Creates an {@link Eq} for {@link Redacted} values based on an equality function for the underlying type.",
            77,
            0,
            false,
            None,
            None,
        );
        assert_eq!(
            result,
            "Creates an {@link Eq} for {@link Redacted} values based on an equality function\nfor the underlying type."
        );

        // No tags, overflow by 1: should NOT get tolerance
        let result = wrap_text(
            "This function returns a value similar to SomeLongClassName and does something when called today.",
            77,
            0,
            false,
            None,
            None,
        );
        assert!(
            result.contains('\n'),
            "Line with no {{@link}} tags should still wrap at width boundary"
        );
    }

    #[test]
    fn test_spread_list_collapses_blank_lines() {
        // Upstream prettier-plugin-jsdoc removes blank lines between list items
        // even when the source has them (spread lists).
        let result =
            wrap_text("- item one\n\n- item two\n\n- item three", 80, 0, false, None, None);
        assert_eq!(result, "- item one\n- item two\n- item three");
    }

    #[test]
    fn test_wrap_indented_code_block_in_description() {
        // 4-space indented code block should be preserved
        let input =
            "The options object:\n\n    const result = process(options);\n    console.log(result);";
        // Simulate @param {object} options: tag_str_len=22, indent_width=78
        let result = wrap_text(input, 78, 22, false, None, None);
        // The indented code block should be preserved with 4-space indent
        assert!(
            result.contains("    const result = process(options);"),
            "Code block lost 4-space indent. Got:\n{result}"
        );
        assert!(
            result.contains("    console.log(result);"),
            "Second code line lost 4-space indent. Got:\n{result}"
        );
    }
}
