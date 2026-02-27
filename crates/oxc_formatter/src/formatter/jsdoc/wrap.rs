/// Check if a line represents structured content that should not be wrapped.
/// This includes markdown lists, code fences, tables, headings, and blockquotes.
fn is_structured_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    // Markdown unordered list items
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        // Markdown ordered list items
        || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
            && trimmed.contains(". ")
            && trimmed.find(". ").unwrap_or(trimmed.len()) < 5
        // Code fences
        || trimmed.starts_with("```")
        // Tables
        || trimmed.starts_with('|')
        // Headings
        || trimmed.starts_with('#')
        // Blockquotes
        || trimmed.starts_with('>')
}

/// Wrap a single paragraph of plain text to the given max width.
fn wrap_single_paragraph(text: &str, max_width: usize, lines: &mut Vec<String>) {
    if text.is_empty() {
        return;
    }

    // If the text already fits on one line, just add it
    if text.len() <= max_width {
        lines.push(text.to_string());
        return;
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return;
    }

    let mut current_line = String::with_capacity(max_width);

    for word in words {
        if current_line.is_empty() {
            // First word on the line always goes on the line
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Word doesn't fit, start a new line
            lines.push(std::mem::take(&mut current_line));
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }
}

/// Wrap text into lines, preserving structured content (lists, code blocks, tables, etc.)
/// and wrapping plain paragraphs to the given max width.
pub fn wrap_text(text: &str, max_width: usize, lines: &mut Vec<String>) {
    if text.is_empty() {
        return;
    }

    let input_lines: Vec<&str> = text.lines().collect();
    let mut in_code_fence = false;
    let mut paragraph = String::new();

    for line in &input_lines {
        let trimmed = line.trim();

        // Track code fence state
        if trimmed.starts_with("```") {
            // Flush any pending paragraph
            if !paragraph.is_empty() {
                wrap_single_paragraph(paragraph.trim(), max_width, lines);
                paragraph.clear();
            }
            in_code_fence = !in_code_fence;
            lines.push(trimmed.to_string());
            continue;
        }

        // Inside code fence: pass through verbatim
        if in_code_fence {
            lines.push((*line).to_string());
            continue;
        }

        // Empty line: paragraph break
        if trimmed.is_empty() {
            if !paragraph.is_empty() {
                wrap_single_paragraph(paragraph.trim(), max_width, lines);
                paragraph.clear();
            }
            lines.push(String::new());
            continue;
        }

        // Structured content: don't wrap
        if is_structured_line(line) {
            if !paragraph.is_empty() {
                wrap_single_paragraph(paragraph.trim(), max_width, lines);
                paragraph.clear();
            }
            lines.push(trimmed.to_string());
            continue;
        }

        // Regular text: accumulate for paragraph wrapping
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(trimmed);
    }

    // Flush any remaining paragraph
    if !paragraph.is_empty() {
        wrap_single_paragraph(paragraph.trim(), max_width, lines);
    }
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
    fn test_wrap_preserves_code_fence() {
        let mut lines = Vec::new();
        wrap_text("Some text\n```\ncode here\n  indented\n```\nMore text", 80, &mut lines);
        assert_eq!(lines, vec!["Some text", "```", "code here", "  indented", "```", "More text"]);
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
}
