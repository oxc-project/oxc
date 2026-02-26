use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::Comment;
use oxc_jsdoc::JSDoc;

use crate::options::JsdocOptions;

/// Compute the visual column offset of a position in source text by scanning
/// backwards to the start of the line and measuring whitespace width.
/// Tabs are expanded to `tab_width` columns.
pub fn source_indent_column(source_text: &str, position: u32, tab_width: u8) -> usize {
    let bytes = source_text.as_bytes();
    let pos = position as usize;
    // Scan backwards to find start of line
    let line_start =
        bytes[..pos].iter().rposition(|&b| b == b'\n' || b == b'\r').map_or(0, |i| i + 1);

    // Measure visual width of whitespace prefix
    let mut col = 0;
    for &b in &bytes[line_start..pos] {
        match b {
            b'\t' => col = (col / tab_width as usize + 1) * tab_width as usize,
            b' ' => col += 1,
            _ => break,
        }
    }
    col
}

/// Attempt to format a JSDoc comment. Returns `Some(formatted)` if the comment
/// was successfully reformatted, or `None` if it should be left unchanged.
///
/// The full comment text, including the `/**` and `*/` delimiters, is obtained
/// from `comment.span.source_text(source_text)`.
pub fn format_jsdoc_comment<'a>(
    comment: &Comment,
    source_text: &str,
    allocator: &'a Allocator,
    options: &JsdocOptions,
    print_width: u16,
    indent_column: usize,
) -> Option<&'a str> {
    let content = comment.span.source_text(source_text);

    // Need at least `/** */` (5 chars) to have inner content
    if content.len() < 5 {
        return None;
    }

    // Extract the inner content between `/**` and `*/`
    let inner = &content[3..content.len() - 2];

    let jsdoc = JSDoc::new(inner, comment.span);

    let description = jsdoc.comment().parsed_preserving_indent();
    let description = description.trim().to_string();
    let tags = jsdoc.tags();

    // Empty JSDoc (no description, no tags) — remove it
    if description.is_empty() && tags.is_empty() {
        return Some("");
    }

    // Available width for content inside the comment (after ` * ` prefix)
    let prefix_width = indent_column + 3; // ` * `
    let available_width = if print_width as usize > prefix_width {
        print_width as usize - prefix_width
    } else {
        40 // fallback minimum
    };

    let mut sb = StringBuilder::with_capacity_in(content.len(), allocator);

    // Collect formatted lines (without the ` * ` prefix)
    let mut lines: Vec<String> = Vec::new();

    // Format the description
    if !description.is_empty() {
        let desc = if options.capitalize_descriptions {
            capitalize_first(&description)
        } else {
            description
        };
        wrap_text(&desc, available_width, &mut lines);
    }

    // Format tags
    let mut prev_kind: Option<&str> = None;
    for tag in tags {
        let kind = normalize_tag_kind(tag.kind.parsed());

        // Add blank line separator: between description and first tag,
        // or when tag group changes (e.g. @param group → @returns)
        let needs_separator = if !lines.is_empty() && lines.last().is_some_and(|l| !l.is_empty()) {
            prev_kind.is_none_or(|prev| tag_group(prev) != tag_group(kind))
        } else {
            false
        };
        if needs_separator {
            lines.push(String::new());
        }

        // Tags that use type_name_comment pattern
        match kind {
            "param" | "property" | "typedef" | "template" => {
                let (type_part, name_part, comment_part) = tag.type_name_comment();
                let mut tag_line = format_tag_with_type_name_comment(
                    kind,
                    type_part
                        .as_ref()
                        .map(oxc_jsdoc::parser::jsdoc_parts::JSDocTagTypePart::parsed),
                    name_part
                        .as_ref()
                        .map(oxc_jsdoc::parser::jsdoc_parts::JSDocTagTypeNamePart::parsed),
                    &comment_part.parsed(),
                    options,
                );
                // Wrap if needed, but keep the tag on the first line
                wrap_tag_line(&mut tag_line, available_width, &mut lines);
            }
            "returns" | "yields" | "throws" | "type" | "satisfies" | "default" | "remarks" => {
                let (type_part, comment_part) = tag.type_comment();
                let mut tag_line = format_tag_with_type_comment(
                    kind,
                    type_part
                        .as_ref()
                        .map(oxc_jsdoc::parser::jsdoc_parts::JSDocTagTypePart::parsed),
                    &comment_part.parsed(),
                    options,
                );
                wrap_tag_line(&mut tag_line, available_width, &mut lines);
            }
            "example" => {
                // Preserve @example content verbatim with indentation
                let comment_text = tag.comment().parsed_preserving_indent();
                lines.push(format!("@{kind}"));
                if !comment_text.is_empty() {
                    for line in comment_text.lines() {
                        // Skip empty leading/trailing lines from the raw content
                        if line.trim().is_empty()
                            && lines.last().is_some_and(|l| l == &format!("@{kind}"))
                        {
                            continue;
                        }
                        lines.push(line.to_string());
                    }
                    // Remove trailing empty lines from example
                    while lines.last().is_some_and(String::is_empty) {
                        lines.pop();
                    }
                }
            }
            _ => {
                // Generic tag: @kind comment
                let comment_text = tag.comment().parsed();
                let mut tag_line = if comment_text.is_empty() {
                    format!("@{kind}")
                } else {
                    let comment_text = normalize_comment_text(&comment_text);
                    // Only capitalize tags that contain descriptive text,
                    // not identifier-like tags (@name, @category, @see, etc.)
                    let should_capitalize = options.capitalize_descriptions
                        && !matches!(
                            kind,
                            "name"
                                | "category"
                                | "see"
                                | "since"
                                | "version"
                                | "author"
                                | "module"
                                | "namespace"
                                | "memberof"
                                | "requires"
                                | "license"
                                | "borrows"
                                | "extends"
                                | "augments"
                                | "implements"
                                | "mixes"
                                | "override"
                                | "access"
                        );
                    let desc = if should_capitalize {
                        capitalize_first(&comment_text)
                    } else {
                        comment_text
                    };
                    format!("@{kind} {desc}")
                };
                wrap_tag_line(&mut tag_line, available_width, &mut lines);
            }
        }

        prev_kind = Some(kind);
    }

    // Try single-line form
    if options.single_line_when_possible && can_be_single_line(&lines, available_width) {
        let single = &lines[0];
        sb.push_str("/** ");
        sb.push_str(single);
        sb.push_str(" */");
        return Some(sb.into_str());
    }

    // Multi-line form
    sb.push_str("/**");
    for line in &lines {
        sb.push('\n');
        // Indent to match surrounding code
        push_indent(&mut sb, indent_column);
        if line.is_empty() {
            sb.push_str(" *");
        } else {
            sb.push_str(" * ");
            sb.push_str(line);
        }
    }
    sb.push('\n');
    push_indent(&mut sb, indent_column);
    sb.push_str(" */");

    let result = sb.into_str();

    // Only return formatted if it actually changed something
    if result == content {
        return None;
    }

    Some(result)
}

/// Normalize tag aliases to their canonical form.
fn normalize_tag_kind(kind: &str) -> &str {
    match kind {
        "return" => "returns",
        "arg" => "param",
        "yield" => "yields",
        "prop" => "property",
        _ => kind,
    }
}

/// Group tags into categories for blank line separation.
/// Tags in the same group are not separated by blank lines.
fn tag_group(kind: &str) -> u8 {
    match kind {
        // Parameter-like tags
        "param" | "property" | "this" | "template" | "typedef" => 0,
        // Return-like tags
        "returns" | "yields" => 1,
        // Error tags
        "throws" => 2,
        // Example tags
        "example" => 3,
        // Everything else gets its own group based on kind
        _ => 4,
    }
}

/// Format a tag with `{type} name comment` pattern (e.g., @param).
fn format_tag_with_type_name_comment(
    kind: &str,
    type_str: Option<&str>,
    name: Option<&str>,
    comment: &str,
    options: &JsdocOptions,
) -> String {
    let mut result = format!("@{kind}");

    if let Some(t) = type_str {
        let normalized = normalize_type(t);
        result.push(' ');
        result.push('{');
        result.push_str(&normalized);
        result.push('}');
    }

    if let Some(n) = name {
        result.push(' ');
        result.push_str(n);
    }

    if !comment.is_empty() {
        // Normalize multiline comment text into a single line
        let comment = normalize_comment_text(comment);
        // Preserve the original dash prefix style
        let (prefix, stripped) = if let Some(rest) = comment.strip_prefix("- ") {
            (" - ", rest)
        } else {
            (" ", comment.as_str())
        };
        let desc = if options.capitalize_descriptions {
            capitalize_first(stripped)
        } else {
            stripped.to_string()
        };
        result.push_str(prefix);
        result.push_str(&desc);
    }

    result
}

/// Format a tag with `{type} comment` pattern (e.g., @returns).
fn format_tag_with_type_comment(
    kind: &str,
    type_str: Option<&str>,
    comment: &str,
    options: &JsdocOptions,
) -> String {
    let mut result = format!("@{kind}");

    if let Some(t) = type_str {
        let normalized = normalize_type(t);
        result.push(' ');
        result.push('{');
        result.push_str(&normalized);
        result.push('}');
    }

    if !comment.is_empty() {
        // Normalize multiline comment text into a single line
        let comment = normalize_comment_text(comment);
        let desc =
            if options.capitalize_descriptions { capitalize_first(&comment) } else { comment };
        result.push(' ');
        result.push_str(&desc);
    }

    result
}

/// Normalize multiline tag comment text into a single line.
/// Only joins plain continuation lines — preserves list items, paragraph breaks,
/// and other structured content.
fn normalize_comment_text(text: &str) -> String {
    // If it contains structured content (lists, paragraph breaks, separators),
    // don't normalize — return as-is with trimmed lines
    if has_structured_content(text) {
        return text.to_string();
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Check if text contains structured content that should not be collapsed.
fn has_structured_content(text: &str) -> bool {
    for line in text.lines() {
        let trimmed = line.trim();
        // Empty line = paragraph break
        if trimmed.is_empty() {
            return true;
        }
        // Markdown list items
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            return true;
        }
        // Numbered list items (e.g., "1. ", "2. ")
        if starts_with_numbered_list(trimmed) {
            return true;
        }
        // Visual separators (lines of repeated characters)
        if trimmed.len() >= 5 && trimmed.chars().all(|c| c == '=' || c == '-' || c == '*') {
            return true;
        }
    }
    false
}

/// Check if a line starts with a numbered list pattern like "1. ", "2. ".
fn starts_with_numbered_list(s: &str) -> bool {
    let mut chars = s.chars();
    if let Some(first) = chars.next()
        && first.is_ascii_digit()
    {
        for ch in chars {
            if ch == '.' {
                return true;
            }
            if !ch.is_ascii_digit() {
                return false;
            }
        }
    }
    false
}

/// Normalize whitespace in type expressions: `  string  |  number  ` → `string | number`.
/// For multi-line types, first strip JSDoc `*` line prefixes before normalizing.
fn normalize_type(t: &str) -> String {
    // First, strip JSDoc `*` prefixes from each line if multi-line
    let cleaned = if t.contains('\n') {
        t.lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed.strip_prefix("* ") {
                    rest.trim_end()
                } else if let Some(rest) = trimmed.strip_prefix('*') {
                    rest.trim_end()
                } else {
                    trimmed.trim_end()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        t.to_string()
    };

    // Collapse internal whitespace
    let mut result = String::with_capacity(cleaned.len());
    let mut prev_space = false;
    for ch in cleaned.trim().chars() {
        if ch.is_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }
    result
}

/// Capitalize the first letter of a string, preserving inline code blocks.
fn capitalize_first(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // Don't capitalize if starts with backtick (inline code)
    if s.starts_with('`') {
        return s.to_string();
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if first.is_ascii_lowercase() {
        let mut result = String::with_capacity(s.len());
        result.push(first.to_ascii_uppercase());
        result.push_str(chars.as_str());
        result
    } else {
        s.to_string()
    }
}

/// Wrap text at word boundaries to fit within `max_width`.
/// Joins consecutive plain text lines into paragraphs before wrapping,
/// but preserves list items, code fence blocks, and paragraph breaks.
fn wrap_text(text: &str, max_width: usize, lines: &mut Vec<String>) {
    let raw_lines: Vec<&str> = text.split('\n').collect();
    let mut i = 0;
    let mut in_code_fence = false;
    while i < raw_lines.len() {
        let line = raw_lines[i];
        let trimmed = line.trim();

        // Inside a code fence: preserve lines verbatim until closing fence
        if in_code_fence {
            lines.push(line.to_string());
            if trimmed.starts_with("```") {
                in_code_fence = false;
            }
            i += 1;
            continue;
        }

        if trimmed.is_empty() {
            lines.push(String::new());
            i += 1;
            continue;
        }

        // Code fence opening: start verbatim block
        if trimmed.starts_with("```") {
            lines.push(trimmed.to_string());
            in_code_fence = true;
            i += 1;
            continue;
        }

        // If this line starts structured content, output it directly
        if is_structured_line(trimmed) {
            lines.push(trimmed.to_string());
            i += 1;
            continue;
        }

        // Join consecutive plain text lines into a paragraph
        let mut paragraph = trimmed.to_string();
        while i + 1 < raw_lines.len() {
            let next = raw_lines[i + 1].trim();
            if next.is_empty() || is_structured_line(next) {
                break;
            }
            paragraph.push(' ');
            paragraph.push_str(next);
            i += 1;
        }
        wrap_single_paragraph(&paragraph, max_width, lines);
        i += 1;
    }
}

/// Check if a line is structured content that should not be joined with adjacent lines.
fn is_structured_line(trimmed: &str) -> bool {
    // Markdown list items
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return true;
    }
    // Numbered list items
    if starts_with_numbered_list(trimmed) {
        return true;
    }
    // Visual separators
    if trimmed.len() >= 5 && trimmed.chars().all(|c| c == '=' || c == '-' || c == '*') {
        return true;
    }
    // Code fence
    if trimmed.starts_with("```") {
        return true;
    }
    false
}

/// Wrap a single paragraph of text.
fn wrap_single_paragraph(text: &str, max_width: usize, lines: &mut Vec<String>) {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return;
    }

    let mut current_line = String::new();
    for word in words {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() > max_width {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
}

/// Wrap a tag line, keeping the tag on the first line and wrapping continuations.
/// Continuation lines are not indented — they appear as plain text after ` * `.
fn wrap_tag_line(tag_line: &mut String, max_width: usize, lines: &mut Vec<String>) {
    if tag_line.len() <= max_width {
        lines.push(std::mem::take(tag_line));
        return;
    }

    // Find a good break point within the tag line
    let words: Vec<&str> = tag_line.split_whitespace().collect();
    let mut current_line = String::new();
    let mut first = true;

    for word in &words {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() > max_width && !first {
            lines.push(current_line);
            // Indent continuation lines to align with text after the tag
            current_line = format!("  {word}");
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
        first = false;
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
}

/// Check if the content can fit on a single line.
fn can_be_single_line(lines: &[String], available_width: usize) -> bool {
    // Must be exactly one non-empty line
    if lines.len() != 1 {
        return false;
    }
    let line = &lines[0];

    // The line must fit: `/** ` + content + ` */` = content + 7
    if line.len() + 7 > available_width + 3 {
        // +3 because available_width already subtracts prefix
        return false;
    }

    // Don't single-line descriptions (only tags)
    if !line.starts_with('@') {
        return false;
    }

    true
}

/// Push indentation spaces.
fn push_indent(sb: &mut StringBuilder<'_>, indent_column: usize) {
    sb.push_ascii_byte_repeat(b' ', indent_column);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_indent_column() {
        assert_eq!(source_indent_column("/**", 0, 2), 0);
        assert_eq!(source_indent_column("    /**", 4, 2), 4);
        assert_eq!(source_indent_column("\t/**", 1, 2), 2);
        assert_eq!(source_indent_column("\t/**", 1, 4), 4);
        assert_eq!(source_indent_column("\t  /**", 3, 4), 6);
        assert_eq!(source_indent_column("x\n  /**", 4, 2), 2);
        assert_eq!(source_indent_column("x\r\n    /**", 7, 2), 4);
    }

    #[test]
    fn test_normalize_tag_kind() {
        assert_eq!(normalize_tag_kind("return"), "returns");
        assert_eq!(normalize_tag_kind("arg"), "param");
        assert_eq!(normalize_tag_kind("yield"), "yields");
        assert_eq!(normalize_tag_kind("prop"), "property");
        assert_eq!(normalize_tag_kind("param"), "param");
        assert_eq!(normalize_tag_kind("type"), "type");
        assert_eq!(normalize_tag_kind("custom"), "custom");
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("Hello"), "Hello");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("`code`"), "`code`");
        assert_eq!(capitalize_first("123"), "123");
        assert_eq!(capitalize_first("a"), "A");
    }

    #[test]
    fn test_normalize_type() {
        assert_eq!(normalize_type("string"), "string");
        assert_eq!(normalize_type("  string  "), "string");
        assert_eq!(normalize_type("string | number"), "string | number");
        assert_eq!(normalize_type("string  |  number"), "string | number");
        assert_eq!(normalize_type("Array< string >"), "Array< string >");
        assert_eq!(normalize_type(""), "");
    }
}
