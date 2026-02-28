/// Result of parsing a list item prefix.
struct ListItemPrefix {
    /// Width for continuation indentation
    indent_width: usize,
    /// The normalized prefix text (e.g., "1. ", "- ")
    normalized_prefix: String,
    /// The rest of the line after the prefix (trimmed)
    rest_start: usize,
}

/// Check if a line starts a list item and return parsing info.
/// Returns Some for list items, None for non-list lines.
fn parse_list_item(trimmed: &str) -> Option<ListItemPrefix> {
    // Markdown unordered list items: "- ", "* ", "+ "
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return Some(ListItemPrefix {
            indent_width: 2,
            normalized_prefix: "- ".to_string(),
            rest_start: 2,
        });
    }
    // Markdown ordered list items: "1. ", "2. ", "1- ", etc.
    if let Some(first) = trimmed.chars().next()
        && first.is_ascii_digit()
    {
        // Try "N. " format first
        if let Some(dot_space_pos) = trimmed.find(". ")
            && dot_space_pos < 5
        {
            let number = &trimmed[..dot_space_pos];
            let prefix = format!("{number}. ");
            let rest_start = dot_space_pos + 2;
            return Some(ListItemPrefix {
                indent_width: prefix.len(),
                normalized_prefix: prefix,
                rest_start,
            });
        }
        // Try "N- " format (non-standard, convert to "N. ")
        if let Some(dash_pos) = trimmed.find('-')
            && dash_pos < 5
        {
            let number = &trimmed[..dash_pos];
            if number.chars().all(|c| c.is_ascii_digit()) {
                let prefix = format!("{number}. ");
                // Skip past dash and any extra spaces
                let after_dash = &trimmed[dash_pos + 1..];
                let spaces = after_dash.len() - after_dash.trim_start().len();
                let rest_start = dash_pos + 1 + spaces;
                return Some(ListItemPrefix {
                    indent_width: prefix.len(),
                    normalized_prefix: prefix,
                    rest_start,
                });
            }
        }
    }
    None
}

/// Simple check for list item (returns just the indent width for compatibility).
fn list_item_indent(trimmed: &str) -> Option<usize> {
    parse_list_item(trimmed).map(|p| p.indent_width)
}

/// Check if a line is a table line (starts with |)
fn is_table_line(trimmed: &str) -> bool {
    trimmed.starts_with('|')
}

/// Check if a line is a heading (starts with #)
fn is_heading_line(trimmed: &str) -> bool {
    trimmed.starts_with('#')
}

/// Check if a line is a blockquote (starts with >)
fn is_blockquote_line(trimmed: &str) -> bool {
    trimmed.starts_with('>')
}

/// Check if a line starts a code fence
fn is_code_fence(trimmed: &str) -> bool {
    trimmed.starts_with("```")
}

/// Check if a line is an indented code block (4+ spaces of indentation).
/// Only counts as indented code if it's not a list continuation.
fn is_indented_code(line: &str) -> bool {
    let leading_spaces = line.len() - line.trim_start().len();
    leading_spaces >= 4
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
fn parse_table_cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|cell| cell.trim().to_string()).collect()
}

/// Format a block of consecutive table lines.
/// If the table has a valid separator row, format with column padding.
/// Otherwise, output as-is.
pub fn format_table_block(table_lines: &[String], lines: &mut Vec<String>) {
    // Find separator row
    let separator_idx = table_lines.iter().position(|l| is_table_separator(l));

    if separator_idx.is_none() {
        // No separator row: output as-is
        for line in table_lines {
            lines.push(line.clone());
        }
        return;
    }

    // Parse all rows into cells
    let all_cells: Vec<Vec<String>> = table_lines
        .iter()
        .filter(|l| !is_table_separator(l))
        .map(|l| parse_table_cells(l))
        .collect();

    if all_cells.is_empty() {
        for line in table_lines {
            lines.push(line.clone());
        }
        return;
    }

    // Determine number of columns
    let num_cols = all_cells.iter().map(std::vec::Vec::len).max().unwrap_or(0);
    if num_cols == 0 {
        for line in table_lines {
            lines.push(line.clone());
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

    // Format each row
    let separator_idx = separator_idx.unwrap();
    for (idx, table_line) in table_lines.iter().enumerate() {
        if idx == separator_idx {
            // Format separator row
            let sep_cells: Vec<String> = col_widths.iter().map(|&w| "-".repeat(w)).collect();
            let formatted = format!("| {} |", sep_cells.join(" | "));
            lines.push(formatted);
        } else {
            // Format data row
            let cells = parse_table_cells(table_line);
            let padded_cells: Vec<String> = (0..num_cols)
                .map(|j| {
                    let cell = cells.get(j).map_or("", String::as_str);
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

    let indent_str = " ".repeat(continuation_indent);
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
                lines.push(format!("{indent_str}{}", std::mem::take(&mut current_line)));
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
            lines.push(format!("{indent_str}{current_line}"));
            lines.push(format!("{indent_str}{overflow}"));
            return;
        }

        if is_first_line {
            lines.push(current_line);
        } else {
            lines.push(format!("{indent_str}{current_line}"));
        }
    }
}

// ──────────────────────────────────────────────────
// Nested list handling
// ──────────────────────────────────────────────────

/// A node in a nested list tree.
struct ListItemNode {
    text: String,
    marker_type: ListMarkerType,
    children: Vec<ListItemNode>,
}

#[derive(Clone, Copy, PartialEq)]
enum ListMarkerType {
    Ordered,
    Unordered,
}

/// Check if the description text is a pure nested list (all non-blank lines
/// are list items, with at least one top-level and one indented item).
/// Returns false for mixed content (paragraphs, code blocks, headers + lists).
pub fn has_nested_lists(text: &str) -> bool {
    let mut has_top_level_list = false;
    let mut has_indented_list = false;
    for line in text.lines() {
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // If any non-blank line is NOT a list item, this is mixed content
        if parse_list_item(trimmed).is_none() {
            return false;
        }
        if leading == 0 {
            has_top_level_list = true;
        } else {
            has_indented_list = true;
        }
    }
    has_top_level_list && has_indented_list
}

struct ParsedListLine {
    indent: usize,
    text: String,
    marker_type: ListMarkerType,
}

/// Build a tree of nested list items from flat (depth, text, marker_type) tuples.
fn build_list_tree(
    items: &[(usize, &str, ListMarkerType)],
    target_depth: usize,
) -> Vec<ListItemNode> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < items.len() {
        let (depth, text, marker_type) = items[i];
        if depth < target_depth {
            break;
        }
        if depth == target_depth {
            // Collect children: all items after this one at deeper depth,
            // up until the next item at the same or lesser depth
            let mut end = i + 1;
            while end < items.len() && items[end].0 > target_depth {
                end += 1;
            }
            let children = if end > i + 1 {
                build_list_tree(&items[i + 1..end], target_depth + 1)
            } else {
                Vec::new()
            };
            result.push(ListItemNode { text: text.to_string(), marker_type, children });
            i = end;
        } else {
            i += 1;
        }
    }
    result
}

/// Parse flat input lines into a tree of nested list items.
/// Uses leading whitespace to determine nesting depth.
fn parse_nested_list(text: &str) -> Vec<ListItemNode> {
    let mut parsed = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let leading = line.len() - line.trim_start().len();
        if let Some(p) = parse_list_item(trimmed) {
            let rest = trimmed[p.rest_start..].trim().to_string();
            let marker_type = if p.normalized_prefix.contains('.') {
                ListMarkerType::Ordered
            } else {
                ListMarkerType::Unordered
            };
            parsed.push(ParsedListLine { indent: leading, text: rest, marker_type });
        }
    }

    if parsed.is_empty() {
        return Vec::new();
    }

    // Determine indent levels: collect unique indents and sort them
    let mut indent_levels: Vec<usize> = parsed.iter().map(|p| p.indent).collect();
    indent_levels.sort_unstable();
    indent_levels.dedup();

    // Assign depth based on indent level position
    let depth_of =
        |indent: usize| -> usize { indent_levels.iter().position(|&i| i == indent).unwrap_or(0) };

    let items: Vec<(usize, &str, ListMarkerType)> =
        parsed.iter().map(|p| (depth_of(p.indent), p.text.as_str(), p.marker_type)).collect();

    build_list_tree(&items, 0)
}

/// Serialize a nested list tree into output lines with proper indentation.
/// Uses remark-style indentation:
/// - Level 0: indent 0
/// - Level 1: indent = parent marker width (3 for ordered)
/// - Level 2+: indent = parent indent + parent marker width + parent marker width
///   (accounts for content column + extra indent for readability)
fn serialize_nested_list(
    nodes: &[ListItemNode],
    depth: usize,
    indent: usize,
    capitalize: bool,
    lines: &mut Vec<String>,
) {
    if nodes.is_empty() {
        return;
    }

    let indent_str = " ".repeat(indent);
    let mut counter = 0u32;

    for node in nodes {
        let has_children = !node.children.is_empty();

        // Build marker
        let marker = if node.marker_type == ListMarkerType::Ordered {
            counter += 1;
            format!("{counter}. ")
        } else {
            "- ".to_string()
        };

        // Format text
        let text = if capitalize {
            super::normalize::capitalize_first(&node.text)
        } else {
            node.text.clone()
        };

        lines.push(format!("{indent_str}{marker}{text}"));

        if has_children {
            // Add blank line before sub-list (remark spread item behavior)
            lines.push(String::new());

            // Calculate child indent
            let marker_width = marker.len();
            let child_indent = if depth == 0 {
                // Level 0 → Level 1: just marker width
                indent + marker_width
            } else {
                // Level N → Level N+1: content column + marker width
                indent + marker_width + marker_width
            };

            serialize_nested_list(&node.children, depth + 1, child_indent, capitalize, lines);
        }
    }
}

/// Format a nested list description.
pub fn format_nested_list(text: &str, capitalize: bool, lines: &mut Vec<String>) {
    let tree = parse_nested_list(text);
    if tree.is_empty() {
        return;
    }
    serialize_nested_list(&tree, 0, 0, capitalize, lines);
}

/// Wrap text into lines, preserving structured content (lists, code blocks, tables, etc.)
/// and wrapping plain paragraphs to the given max width.
///
/// List items are wrapped with continuation indent:
/// ```text
/// - Long list item text that wraps
///   to the next line with 2-space indent
/// ```
pub fn wrap_text(text: &str, max_width: usize, lines: &mut Vec<String>) {
    if text.is_empty() {
        return;
    }

    let input_lines: Vec<&str> = text.lines().collect();
    let mut in_code_fence = false;
    let mut code_fence_has_language = false;
    let mut paragraph = String::new();
    // Track current list item state
    let mut current_list_indent: Option<usize> = None;
    let mut list_marker = String::new();
    let mut list_text = String::new();
    let mut in_list = false;
    let mut numbered_list_counter: u32 = 0;
    let mut just_finished_table = false;
    let mut just_finished_code_fence = false;
    let mut prev_was_indented_code = false;

    let flush_paragraph = |paragraph: &mut String, lines: &mut Vec<String>| {
        if !paragraph.is_empty() {
            wrap_paragraph(paragraph.trim(), max_width, 0, lines);
            paragraph.clear();
        }
    };

    let flush_list_item = |list_marker: &mut String,
                           list_text: &mut String,
                           current_list_indent: &mut Option<usize>,
                           lines: &mut Vec<String>| {
        if !list_text.is_empty() {
            let indent = current_list_indent.unwrap_or(0);
            // Wrap the content without the marker, then prepend the marker
            // to the first line. This matches the plugin's behavior where
            // the paragraph content is wrapped at commentContentPrintWidth,
            // and the list marker is prepended afterward (allowing the first
            // line to exceed the nominal width by the marker width).
            let mut item_lines = Vec::new();
            wrap_paragraph(list_text.trim(), max_width, indent, &mut item_lines);
            for (idx, line) in item_lines.into_iter().enumerate() {
                if idx == 0 {
                    lines.push(format!("{list_marker}{line}"));
                } else {
                    lines.push(line);
                }
            }
            list_text.clear();
            list_marker.clear();
        }
        *current_list_indent = None;
    };

    let mut i = 0;
    while i < input_lines.len() {
        let line = input_lines[i];
        let trimmed = line.trim();

        // Track code fence state
        if is_code_fence(trimmed) {
            flush_paragraph(&mut paragraph, lines);
            flush_list_item(&mut list_marker, &mut list_text, &mut current_list_indent, lines);
            in_list = false;
            // Don't reset numbered_list_counter here: numbered lists can span
            // across code fences (e.g. "1. step one\n```js\ncode\n```\n2. step two").
            if in_code_fence {
                in_code_fence = false;
                if code_fence_has_language {
                    lines.push(trimmed.to_string());
                }
                code_fence_has_language = false;
                just_finished_code_fence = true;
            } else {
                in_code_fence = true;
                // Check if fenced code has a language tag (```js, ```python, etc.)
                code_fence_has_language = trimmed.len() > 3 && !trimmed[3..].trim().is_empty();
                // Add blank line before code fence if needed
                if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
                    lines.push(String::new());
                }
                if code_fence_has_language {
                    // Keep fenced code with language tag as-is
                    lines.push(trimmed.to_string());
                }
            }
            i += 1;
            continue;
        }

        // Inside code fence: pass through verbatim
        if in_code_fence {
            if code_fence_has_language {
                lines.push(line.to_string());
            } else {
                // Convert to indented code block: add 4-space prefix
                let content = line.trim();
                if content.is_empty() {
                    lines.push(String::new());
                } else {
                    lines.push(format!("    {content}"));
                }
            }
            i += 1;
            continue;
        }

        // Empty line handling
        if trimmed.is_empty() {
            just_finished_code_fence = false;
            just_finished_table = false;
            if in_list {
                // Find next non-blank line
                let next_non_blank = {
                    let mut j = i + 1;
                    while j < input_lines.len() && input_lines[j].trim().is_empty() {
                        j += 1;
                    }
                    input_lines.get(j)
                };

                // Check if next content is a list item
                let next_is_list = next_non_blank.is_some_and(|next| {
                    let next_trimmed = next.trim();
                    !next_trimmed.is_empty() && list_item_indent(next_trimmed).is_some()
                });

                // Check if next content is an indented continuation paragraph
                // (indented by at least the list item's content indent)
                let next_is_continuation = !next_is_list
                    && current_list_indent.is_some()
                    && next_non_blank.is_some_and(|next| {
                        let leading = next.len() - next.trim_start().len();
                        let indent = current_list_indent.unwrap();
                        leading >= indent && !next.trim().is_empty()
                    });

                if next_is_list {
                    flush_list_item(
                        &mut list_marker,
                        &mut list_text,
                        &mut current_list_indent,
                        lines,
                    );
                    i += 1;
                    continue;
                }

                if next_is_continuation {
                    // Flush current list item, add blank separator.
                    // Save the list indent for the continuation paragraph.
                    let saved_indent = current_list_indent.unwrap_or(0);
                    flush_list_item(
                        &mut list_marker,
                        &mut list_text,
                        &mut current_list_indent,
                        lines,
                    );
                    lines.push(String::new());
                    // Exit list mode; instead accumulate the continuation paragraph
                    // and wrap ALL lines (including first) with the saved indent.
                    in_list = false;
                    numbered_list_counter = 0;

                    // Collect the continuation paragraph text
                    let mut cont_paragraph = String::new();
                    i += 1; // skip blank line
                    while i < input_lines.len() {
                        let cont_line = input_lines[i];
                        let cont_trimmed = cont_line.trim();
                        if cont_trimmed.is_empty() {
                            break;
                        }
                        if !cont_paragraph.is_empty() {
                            cont_paragraph.push(' ');
                        }
                        cont_paragraph.push_str(cont_trimmed);
                        i += 1;
                    }
                    if !cont_paragraph.is_empty() {
                        let indent_str = " ".repeat(saved_indent);
                        // Wrap at max_width (not max_width - indent) to match plugin
                        // behavior: the paragraph is wrapped first, then indentation
                        // is prepended. This allows indented lines to exceed the
                        // nominal width, matching the plugin's markdown AST approach.
                        let mut para_lines = Vec::new();
                        wrap_paragraph(&cont_paragraph, max_width, 0, &mut para_lines);
                        for pl in para_lines {
                            lines.push(format!("{indent_str}{pl}"));
                        }
                    }
                    continue;
                }

                flush_list_item(&mut list_marker, &mut list_text, &mut current_list_indent, lines);
                in_list = false;
                // Don't reset numbered_list_counter: the list may continue
                // after blank lines and code fences.
            }
            flush_paragraph(&mut paragraph, lines);
            // Allow consecutive blank lines between indented code blocks
            // (when the next non-blank line is also indented code)
            let next_is_indented_code = prev_was_indented_code && {
                let mut j = i + 1;
                while j < input_lines.len() && input_lines[j].trim().is_empty() {
                    j += 1;
                }
                j < input_lines.len() && is_indented_code(input_lines[j])
            };
            if next_is_indented_code || !lines.last().is_some_and(String::is_empty) {
                lines.push(String::new());
            }
            i += 1;
            continue;
        }

        // Check for list items
        if let Some(parsed) = parse_list_item(trimmed) {
            flush_paragraph(&mut paragraph, lines);
            // Add blank line after code fence/table before list items
            if just_finished_code_fence || just_finished_table {
                just_finished_code_fence = false;
                just_finished_table = false;
                if !lines.last().is_some_and(String::is_empty) {
                    lines.push(String::new());
                }
            }
            if in_list {
                flush_list_item(&mut list_marker, &mut list_text, &mut current_list_indent, lines);
            }
            in_list = true;
            let rest = &trimmed[parsed.rest_start..].trim_start();

            if parsed.normalized_prefix.contains('.') {
                numbered_list_counter += 1;
                list_marker = format!("{numbered_list_counter}. ");
                current_list_indent = Some(list_marker.len());
            } else {
                numbered_list_counter = 0;
                current_list_indent = Some(parsed.indent_width);
                list_marker = parsed.normalized_prefix;
            }
            list_text = rest.to_string();
            i += 1;
            continue;
        }

        // Non-list text following a list item is continuation
        if in_list && current_list_indent.is_some() {
            list_text.push(' ');
            list_text.push_str(trimmed);
            i += 1;
            continue;
        }

        // Check for indented code blocks (4+ spaces)
        if is_indented_code(line) && !in_list {
            flush_paragraph(&mut paragraph, lines);
            lines.push(line.to_string());
            prev_was_indented_code = true;
            i += 1;
            continue;
        }

        // Reset indented code tracking when we see non-empty, non-code content
        prev_was_indented_code = false;

        // Table lines: collect consecutive table lines as a block
        if is_table_line(trimmed) {
            flush_paragraph(&mut paragraph, lines);
            numbered_list_counter = 0;
            let mut table_lines = vec![trimmed.to_string()];
            let mut j = i + 1;
            while j < input_lines.len() {
                let next = input_lines[j].trim();
                if is_table_line(next) {
                    table_lines.push(next.to_string());
                    j += 1;
                } else {
                    break;
                }
            }
            // Add blank line before table if needed
            if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
                lines.push(String::new());
            }
            format_table_block(&table_lines, lines);
            just_finished_table = true;
            i = j; // Skip past all consumed table lines
            continue;
        }

        // Heading lines
        if is_heading_line(trimmed) {
            flush_paragraph(&mut paragraph, lines);
            numbered_list_counter = 0;
            // Add blank line before heading if needed
            if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
                lines.push(String::new());
            }
            lines.push(trimmed.to_string());
            let next_is_content =
                input_lines.get(i + 1).is_some_and(|next| !next.trim().is_empty());
            if next_is_content {
                lines.push(String::new());
            }
            i += 1;
            continue;
        }

        // Blockquote lines
        if is_blockquote_line(trimmed) {
            flush_paragraph(&mut paragraph, lines);
            numbered_list_counter = 0;
            if trimmed == ">" {
                if !lines.last().is_some_and(String::is_empty) {
                    lines.push(String::new());
                }
            } else {
                lines.push(trimmed.to_string());
            }
            i += 1;
            continue;
        }

        // Check for backslash line continuation
        if trimmed.ends_with('\\') {
            flush_paragraph(&mut paragraph, lines);
            lines.push(trimmed.to_string());
            i += 1;
            continue;
        }

        // Add blank line after table/code-fence blocks before regular text
        if just_finished_table || just_finished_code_fence {
            just_finished_table = false;
            just_finished_code_fence = false;
            if !lines.last().is_some_and(String::is_empty) {
                lines.push(String::new());
            }
        }

        // Regular text: accumulate for paragraph wrapping
        // Paragraphs definitively end numbered lists.
        if numbered_list_counter > 0 && !in_list {
            numbered_list_counter = 0;
        }
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(trimmed);
        i += 1;
    }

    // Flush any remaining content
    flush_list_item(&mut list_marker, &mut list_text, &mut current_list_indent, lines);
    flush_paragraph(&mut paragraph, lines);
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
        // Fenced code without language tag is converted to indented code block
        // Blank lines are added before and after the code block
        assert_eq!(lines, vec!["Some text", "", "    code here", "    indented", "", "More text"]);
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
