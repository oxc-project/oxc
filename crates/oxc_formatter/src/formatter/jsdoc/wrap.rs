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
/// Accepts `&[&str]` or `&[String]` via `AsRef<str>`.
pub fn format_table_block<S: AsRef<str>>(table_lines: &[S], lines: &mut Vec<String>) {
    // Find separator row
    let separator_idx = table_lines.iter().position(|l| is_table_separator(l.as_ref()));

    if separator_idx.is_none() {
        // No separator row: output as-is
        for line in table_lines {
            lines.push(line.as_ref().to_string());
        }
        return;
    }

    // Parse all rows into cells
    let all_cells: Vec<Vec<&str>> = table_lines
        .iter()
        .filter(|l| !is_table_separator(l.as_ref()))
        .map(|l| parse_table_cells(l.as_ref()))
        .collect();

    if all_cells.is_empty() {
        for line in table_lines {
            lines.push(line.as_ref().to_string());
        }
        return;
    }

    // Determine number of columns
    let num_cols = all_cells.iter().map(std::vec::Vec::len).max().unwrap_or(0);
    if num_cols == 0 {
        for line in table_lines {
            lines.push(line.as_ref().to_string());
        }
        return;
    }

    // Compute max width per column
    let mut col_widths = vec![3usize; num_cols]; // minimum 3 for "---"
    for row in &all_cells {
        for (j, cell) in row.iter().enumerate() {
            if j < num_cols {
                col_widths[j] = col_widths[j].max(cell.len());
            }
        }
    }

    // Format each row, reusing already-parsed cells from all_cells
    let separator_idx = separator_idx.unwrap();
    let mut data_row_idx = 0;
    for (idx, _table_line) in table_lines.iter().enumerate() {
        if idx == separator_idx {
            // Format separator row
            let sep_cells: Vec<String> = col_widths.iter().map(|&w| "-".repeat(w)).collect();
            let formatted = format!("| {} |", sep_cells.join(" | "));
            lines.push(formatted);
        } else {
            // Format data row using pre-parsed cells
            let cells = &all_cells[data_row_idx];
            data_row_idx += 1;
            let padded_cells: Vec<String> = (0..num_cols)
                .map(|j| {
                    let cell = cells.get(j).copied().unwrap_or("");
                    format!("{cell:<width$}", width = col_widths[j])
                })
                .collect();
            let formatted = format!("| {} |", padded_cells.join(" | "));
            lines.push(formatted);
        }
    }
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
            // Include trailing punctuation (`.`, `,`, `;`, `:`, `!`, `?`) as part of the token
            // This prevents wrapping from splitting `{@link Foo}.` into separate lines
            while i < len && matches!(bytes[i], b'.' | b',' | b';' | b':' | b'!' | b'?') {
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

/// Wrap a single paragraph of plain text to the given max width with optional indent for
/// continuation lines.
///
/// Uses word-by-word greedy approach with `tokenize_words` to preserve atomic tokens
/// like `{@link ...}`. After building all lines, applies a post-processing step to match
/// the prettier-plugin-jsdoc `breakDescriptionToLines` behavior: when the last
/// continuation line has exactly `effective_max` characters, the plugin's `\n`-prefix
/// causes one more wrap iteration, splitting the last word to the next line.
pub fn wrap_paragraph(
    text: &str,
    max_width: usize,
    continuation_indent: usize,
    lines: &mut Vec<String>,
) {
    if text.is_empty() {
        return;
    }

    let words = tokenize_words(text);
    if words.is_empty() {
        return;
    }

    let indent_s = indent_str(continuation_indent);
    let effective_max = max_width.saturating_sub(continuation_indent);
    let mut current_line = String::with_capacity(max_width);
    let mut is_first_line = true;

    for word in words {
        let capacity = if is_first_line { max_width } else { effective_max };

        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= capacity {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Word doesn't fit, push current line and start new one
            if is_first_line {
                lines.push(std::mem::take(&mut current_line));
                is_first_line = false;
            } else {
                let mut s = String::with_capacity(indent_s.len() + current_line.len());
                s.push_str(&indent_s);
                s.push_str(&current_line);
                current_line.clear();
                lines.push(s);
            }
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        // Post-processing: match plugin's `>=` boundary behavior for continuation lines.
        // The plugin prepends `\n` to continuation text, which causes `str.length >= maxWidth`
        // to trigger an extra wrap when remaining content is exactly `maxWidth` characters.
        if !is_first_line
            && current_line.len() == effective_max
            && let Some(last_space) = current_line.rfind(' ')
        {
            let overflow = current_line[last_space + 1..].to_string();
            current_line.truncate(last_space);
            let mut s = String::with_capacity(indent_s.len() + current_line.len());
            s.push_str(&indent_s);
            s.push_str(&current_line);
            lines.push(s);
            let mut s = String::with_capacity(indent_s.len() + overflow.len());
            s.push_str(&indent_s);
            s.push_str(&overflow);
            lines.push(s);
            return;
        }

        if is_first_line {
            lines.push(current_line);
        } else {
            let mut s = String::with_capacity(indent_s.len() + current_line.len());
            s.push_str(&indent_s);
            s.push_str(&current_line);
            lines.push(s);
        }
    }
}

/// Wrap plain text with paragraph breaks into lines.
/// For text with no markdown constructs — just paragraphs separated by blank lines.
pub fn wrap_plain_paragraphs(text: &str, max_width: usize, lines: &mut Vec<String>) {
    let mut paragraph = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !paragraph.is_empty() {
                wrap_paragraph(paragraph.trim(), max_width, 0, lines);
                paragraph.clear();
            }
            if !lines.last().is_some_and(String::is_empty) {
                lines.push(String::new());
            }
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(trimmed);
        }
    }
    if !paragraph.is_empty() {
        wrap_paragraph(paragraph.trim(), max_width, 0, lines);
    }
}

/// Wrap text into lines, preserving structured content (lists, code blocks, tables, etc.)
/// and wrapping plain paragraphs to the given max width.
///
/// Delegates to `format_description_mdast` for full markdown-aware formatting.
pub fn wrap_text(text: &str, max_width: usize, lines: &mut Vec<String>) {
    if text.is_empty() {
        return;
    }
    let result = super::mdast_serialize::format_description_mdast(text, max_width, false);
    lines.extend(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_simple_text() {
        let mut lines = Vec::new();
        wrap_text("This is a short line", 80, &mut lines);
        assert_eq!(lines, vec!["This is a short line"]);
    }

    #[test]
    fn test_wrap_long_text() {
        let mut lines = Vec::new();
        wrap_text(
            "This is a long line that should be wrapped because it exceeds the maximum width",
            40,
            &mut lines,
        );
        assert_eq!(
            lines,
            vec![
                "This is a long line that should be",
                "wrapped because it exceeds the maximum",
                "width",
            ]
        );
    }

    #[test]
    fn test_wrap_preserves_markdown_list() {
        let mut lines = Vec::new();
        wrap_text("- item one\n- item two\n- item three", 80, &mut lines);
        assert_eq!(lines, vec!["- item one", "- item two", "- item three"]);
    }

    #[test]
    fn test_wrap_list_item_with_continuation() {
        let mut lines = Vec::new();
        wrap_text(
            "- This is a very long list item that should be wrapped to the next line with proper indent",
            40,
            &mut lines,
        );
        assert_eq!(
            lines,
            vec![
                "- This is a very long list item that",
                "  should be wrapped to the next line",
                "  with proper indent",
            ]
        );
    }

    #[test]
    fn test_wrap_converts_code_fence_to_indented() {
        let mut lines = Vec::new();
        wrap_text("Some text\n```\ncode here\n  indented\n```\nMore text", 80, &mut lines);
        // Fenced code without language tag is converted to indented code block.
        // The MDAST path preserves original indentation within the code block.
        assert_eq!(
            lines,
            vec!["Some text", "", "    code here", "      indented", "", "More text"]
        );
    }

    #[test]
    fn test_wrap_preserves_code_fence_with_language() {
        let mut lines = Vec::new();
        wrap_text("Some text\n```js\nconst x = 1;\n```\nMore text", 80, &mut lines);
        // Blank lines are added before and after fenced code blocks
        assert_eq!(lines, vec!["Some text", "", "```js", "const x = 1;", "```", "", "More text"]);
    }

    #[test]
    fn test_wrap_empty_lines() {
        let mut lines = Vec::new();
        wrap_text("Paragraph one\n\nParagraph two", 80, &mut lines);
        assert_eq!(lines, vec!["Paragraph one", "", "Paragraph two"]);
    }

    #[test]
    fn test_wrap_empty_text() {
        let mut lines = Vec::new();
        wrap_text("", 80, &mut lines);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_numbered_list_removes_blank_lines() {
        let mut lines = Vec::new();
        wrap_text("1. Thing 1\n\n2. Thing 2\n\n3. Thing 3", 80, &mut lines);
        assert_eq!(lines, vec!["1. Thing 1", "2. Thing 2", "3. Thing 3"]);
    }

    #[test]
    fn test_list_item_wrapping_at_boundary() {
        let mut lines = Vec::new();
        wrap_text(
            "- Consider caching this for the lifetime of the component, or possibly being able to share this cache between any `ScrollMap` view.",
            77,
            &mut lines,
        );
        assert_eq!(
            lines,
            vec![
                "- Consider caching this for the lifetime of the component, or possibly being",
                "  able to share this cache between any `ScrollMap` view.",
            ]
        );
    }

    #[test]
    fn test_list_multiline_input() {
        // Test that multi-line list items from JSDoc are joined correctly
        let mut lines = Vec::new();
        wrap_text(
            "- Consider caching this for the lifetime of the component, or possibly being able to share this\ncache between any `ScrollMap` view.",
            77,
            &mut lines,
        );
        assert_eq!(
            lines,
            vec![
                "- Consider caching this for the lifetime of the component, or possibly being",
                "  able to share this cache between any `ScrollMap` view.",
            ]
        );
    }
}
