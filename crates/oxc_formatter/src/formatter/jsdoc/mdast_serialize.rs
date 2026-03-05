use std::borrow::Cow;

use cow_utils::CowUtils;
use markdown::{Constructs, ParseOptions, mdast::Node, to_mdast};

use oxc_allocator::Allocator;

use crate::{ExternalCallbacks, FormatOptions};

use super::line_buffer::LineBuffer;
use super::wrap::{format_table_block, wrap_paragraph, wrap_plain_paragraphs};

/// Placeholder prefix for protecting `{@link ...}` tokens from markdown parsing.
/// Uses a format that `tokenize_words` won't split (no spaces, looks like a word).
const PLACEHOLDER_PREFIX: &str = "\x00JDLNK";

/// Check if the text contains markdown constructs that require full AST parsing.
/// Returns `false` only for pure plain-text paragraphs that `wrap_plain_paragraphs()`
/// can handle directly (no lists, tables, code fences, headings, blockquotes, or
/// inline markdown like emphasis/links).
fn needs_mdast_parsing(text: &str) -> bool {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        match bytes[i] {
            // Emphasis, strikethrough, strong, list marker (*), images,
            // backslash escapes, HTML tags
            b'_' | b'~' | b'*' | b'\\' | b'<' => return true,
            // `[` — only trigger for markdown link/reference patterns, not bare
            // brackets from JavaScript code (e.g. `const [`, `][]`).
            b'[' => {
                // Footnote reference: `[^note]`
                if i + 1 < len && bytes[i + 1] == b'^' {
                    return true;
                }
                // Scan for closing `]` followed by `(` or `[` on the same line
                // to detect `[text](url)` or `[text][ref]` patterns.
                let mut j = i + 1;
                let mut has_content = false;
                while j < len && bytes[j] != b'\n' {
                    if bytes[j] == b']' {
                        if has_content
                            && j + 1 < len
                            && (bytes[j + 1] == b'(' || bytes[j + 1] == b'[')
                        {
                            return true;
                        }
                        break;
                    }
                    if !bytes[j].is_ascii_whitespace() {
                        has_content = true;
                    }
                    j += 1;
                }
            }
            // At line start (after optional leading spaces): detect block-level constructs.
            // We skip leading spaces to catch indented lists/code blocks.
            b' ' | b'#' | b'>' | b'-' | b'0'..=b'9' | b'|' | b'+'
                if i == 0 || bytes[i - 1] == b'\n' =>
            {
                // Count leading spaces
                let mut spaces = 0;
                while i + spaces < len && bytes[i + spaces] == b' ' {
                    spaces += 1;
                }
                // 4+ leading spaces = indented code block
                if spaces >= 4 {
                    return true;
                }
                // Check trigger character after whitespace
                if i + spaces < len {
                    match bytes[i + spaces] {
                        b'#' | b'>' | b'-' | b'0'..=b'9' | b'|' => return true,
                        b'+' if i + spaces + 1 < len && bytes[i + spaces + 1] == b' ' => {
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            // Code fences
            b'`' if i + 2 < len && bytes[i + 1] == b'`' && bytes[i + 2] == b'`' => {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Format a markdown description using mdast parsing.
///
/// Parses the text into a markdown AST, then serializes it back to formatted
/// text with proper indentation, wrapping, and emphasis normalization.
/// This replaces the manual normalize+wrap pipeline with an approach matching
/// the upstream prettier-plugin-jsdoc's use of `fromMarkdown` + `stringify`.
pub fn format_description_mdast(
    text: &str,
    max_width: usize,
    capitalize: bool,
    format_options: Option<&FormatOptions>,
    external_callbacks: Option<&ExternalCallbacks>,
    allocator: Option<&Allocator>,
) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    // Fast path: if text has no markdown constructs requiring AST parsing,
    // use lightweight wrap_plain_paragraphs() directly.
    if !needs_mdast_parsing(text) {
        let result = wrap_plain_paragraphs(text, max_width);
        if !capitalize {
            return result;
        }
        // Capitalize the first word of each paragraph (after blank lines),
        // matching the mdast path's per-paragraph capitalization.
        let mut out = String::with_capacity(result.len());
        let mut at_paragraph_start = true;
        for (i, line) in result.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if line.is_empty() {
                at_paragraph_start = true;
            } else if at_paragraph_start {
                out.push_str(&super::normalize::capitalize_first(line));
                at_paragraph_start = false;
                continue;
            }
            out.push_str(line);
        }
        return out;
    }

    let text = normalize_legacy_ordered_list_markers(text);

    // Protect JSDoc inline tags from markdown parsing (GFM autolink would mangle URLs)
    let (protected, placeholders) = protect_jsdoc_links(&text);

    // Parse into mdast. Keep GFM constructs that affect inline parsing, but let
    // pipe-prefixed table-like blocks be handled by the serializer using the raw
    // paragraph text instead of the markdown crate's table node.
    let parse_opts = ParseOptions {
        constructs: Constructs {
            gfm_autolink_literal: true,
            gfm_footnote_definition: true,
            gfm_label_start_footnote: true,
            gfm_strikethrough: true,
            gfm_table: false,
            gfm_task_list_item: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    };
    let Ok(root) = to_mdast(&protected, &parse_opts) else {
        // If parsing fails, fall back to returning the text as-is
        let mut out = String::new();
        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(line.trim());
        }
        return out;
    };

    let mut lines = LineBuffer::new();
    let opts = SerializeOptions {
        max_width,
        capitalize,
        placeholders: &placeholders,
        source: &protected,
        format_options,
        external_callbacks,
        allocator,
    };
    serialize_children(&root, 0, &opts, &mut lines);

    lines.into_string()
}

struct SerializeOptions<'a> {
    max_width: usize,
    capitalize: bool,
    placeholders: &'a [&'a str],
    source: &'a str,
    format_options: Option<&'a FormatOptions>,
    external_callbacks: Option<&'a ExternalCallbacks>,
    allocator: Option<&'a Allocator>,
}

// ──────────────────────────────────────────────────
// JSDoc link protection
// ──────────────────────────────────────────────────

/// Normalize the legacy `1- foo` list-marker style used by some existing JSDoc
/// fixtures into standard ordered-list syntax so markdown parsing can treat them
/// as list items.
fn normalize_legacy_ordered_list_markers(text: &str) -> Cow<'_, str> {
    // Fast path: the pattern is `<digit(s)>- ` at line start. Check for the
    // minimal signature (a digit followed somewhere by `-`) to skip the
    // per-line scan for the vast majority of descriptions.
    let bytes = text.as_bytes();
    let has_digit_dash = bytes.windows(2).any(|w| w[0].is_ascii_digit() && w[1] == b'-');
    if !has_digit_dash {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut changed = false;

    for line in text.lines() {
        if !result.is_empty() {
            result.push('\n');
        }

        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();

        if let Some(first) = trimmed.chars().next()
            && first.is_ascii_digit()
            && let Some(dash_pos) = trimmed.find('-')
            && dash_pos < 5
        {
            let number = &trimmed[..dash_pos];
            if number.chars().all(|c| c.is_ascii_digit()) {
                let rest = trimmed[dash_pos + 1..].trim_start();
                if !rest.is_empty() {
                    result.push_str(&line[..leading]);
                    result.push_str(number);
                    result.push_str(". ");
                    result.push_str(rest);
                    changed = true;
                    continue;
                }
            }
        }

        result.push_str(line);
    }

    if changed { Cow::Owned(result) } else { Cow::Borrowed(text) }
}

/// Replace `{@link ...}`, `{@linkcode ...}`, `{@linkplain ...}`, `{@tutorial ...}`
/// with numbered placeholders so the markdown parser (especially GFM autolink) doesn't
/// mangle URLs inside them.
fn protect_jsdoc_links(text: &str) -> (Cow<'_, str>, Vec<&str>) {
    // Fast path: if no `{@` in the text, nothing to protect
    if !text.contains("{@") {
        return (Cow::Borrowed(text), Vec::new());
    }

    let mut result = String::with_capacity(text.len());
    let mut placeholders = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' && i + 1 < len && bytes[i + 1] == b'@' {
            let start = i;
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
            // Include trailing punctuation that is part of the token
            while i < len && matches!(bytes[i], b'.' | b',' | b';' | b':' | b'!' | b'?') {
                i += 1;
            }
            let token = &text[start..i];
            let idx = placeholders.len();
            placeholders.push(token);
            // Use a placeholder that looks like a single word (no spaces)
            // so tokenize_words treats it atomically
            result.push_str(PLACEHOLDER_PREFIX);
            result.push_str(itoa::Buffer::new().format(idx));
        } else {
            let ch = text[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    (Cow::Owned(result), placeholders)
}

/// Restore all placeholder tokens in a string back to their original `{@link ...}` form.
fn restore_in_string<'a>(s: &'a str, placeholders: &[&str]) -> Cow<'a, str> {
    if placeholders.is_empty() || !s.contains(PLACEHOLDER_PREFIX) {
        return Cow::Borrowed(s);
    }

    Cow::Owned(replace_placeholders(s, placeholders))
}

/// Single-pass scan that replaces all `PLACEHOLDER_PREFIX<digits>` occurrences
/// with their original strings from `placeholders`.
fn replace_placeholders(s: &str, placeholders: &[&str]) -> String {
    let prefix = PLACEHOLDER_PREFIX;
    let prefix_len = prefix.len();
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if i + prefix_len <= len && &s[i..i + prefix_len] == prefix {
            // Found prefix, parse the index digits that follow
            let digit_start = i + prefix_len;
            let mut digit_end = digit_start;
            while digit_end < len && bytes[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if digit_end > digit_start
                && let Ok(idx) = s[digit_start..digit_end].parse::<usize>()
                && let Some(original) = placeholders.get(idx)
            {
                result.push_str(original);
                i = digit_end;
                continue;
            }
            // Not a valid placeholder, copy the prefix character and advance
            let ch = s[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        } else {
            let ch = s[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    result
}

// ──────────────────────────────────────────────────
// Node serialization
// ──────────────────────────────────────────────────

/// Serialize children of a parent node, inserting blank lines between block-level nodes.
fn serialize_children(
    node: &Node,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    let Some(children) = node.children() else {
        return;
    };

    for (i, child) in children.iter().enumerate() {
        // Add blank line between block-level siblings (except first)
        if i > 0 && is_block_node(child) && !lines.last_is_empty() {
            lines.push_empty();
        }

        serialize_node(child, indent, opts, lines);
    }
}

fn is_block_node(node: &Node) -> bool {
    matches!(
        node,
        Node::Paragraph(_)
            | Node::Heading(_)
            | Node::List(_)
            | Node::Code(_)
            | Node::Blockquote(_)
            | Node::ThematicBreak(_)
            | Node::Definition(_)
            | Node::Html(_)
    )
}

fn serialize_node(node: &Node, indent: usize, opts: &SerializeOptions<'_>, lines: &mut LineBuffer) {
    match node {
        Node::Root(_) => {
            serialize_children(node, indent, opts, lines);
        }
        Node::Paragraph(para) => {
            serialize_paragraph(para, indent, opts, lines);
        }
        Node::Heading(heading) => {
            let text = collect_inline_text(node);
            // Restore placeholders in heading text before emitting
            let text = restore_in_string(&text, opts.placeholders);
            let prefix = "#".repeat(heading.depth as usize);
            if !lines.is_empty() && !lines.last_is_empty() {
                lines.push_empty();
            }
            {
                let s = lines.begin_line();
                s.push_str(&prefix);
                s.push(' ');
                s.push_str(&text);
            }
        }
        Node::List(list) => {
            serialize_list(list, indent, opts, lines);
        }
        // ListItems are handled by serialize_list; thematic breaks are dropped (matches upstream)
        Node::ListItem(_) | Node::ThematicBreak(_) => {}
        Node::Code(code) => {
            serialize_code(code, opts, lines);
        }
        Node::Blockquote(bq) => {
            serialize_blockquote(bq, opts, lines);
        }
        Node::Definition(def) => {
            let label = def.label.as_deref().unwrap_or(&def.identifier);
            {
                let s = lines.begin_line();
                s.push('[');
                s.push_str(label);
                s.push_str("]: ");
                s.push_str(&def.url);
            }
        }
        Node::Html(html) => {
            for line in html.value.lines() {
                lines.push(line);
            }
        }
        // Inline nodes are normally collected by collect_inline_text,
        // but handle them here for any edge case where they appear at block level
        _ => {
            let text = collect_inline_text(node);
            if !text.is_empty() {
                lines.push(text);
            }
        }
    }
}

// ──────────────────────────────────────────────────
// Paragraph serialization
// ──────────────────────────────────────────────────

/// Serialize a paragraph node. Handles Break nodes (hard line breaks from `\` at EOL)
/// by splitting the paragraph into segments that are wrapped independently.
fn serialize_paragraph(
    para: &markdown::mdast::Paragraph,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    if serialize_pipe_prefixed_paragraph(para, indent, opts, lines) {
        return;
    }

    // Check if the paragraph contains Break nodes (hard line breaks from `\` at EOL)
    let has_breaks = para.children.iter().any(|c| matches!(c, Node::Break(_)));

    if has_breaks {
        // Split into segments at Break nodes, each segment on its own line
        let indent_str = super::wrap::indent_str(indent);
        let mut current_segment = String::new();

        for child in &para.children {
            if matches!(child, Node::Break(_)) {
                // Emit current segment with trailing backslash
                let text = restore_in_string(current_segment.trim(), opts.placeholders);
                if indent > 0 {
                    {
                        let s = lines.begin_line();
                        s.push_str(&indent_str);
                        s.push_str(&text);
                        s.push('\\');
                    }
                } else if opts.capitalize && lines.is_empty() {
                    let text = super::normalize::capitalize_first(&text);
                    {
                        let s = lines.begin_line();
                        s.push_str(&text);
                        s.push('\\');
                    }
                } else {
                    {
                        let s = lines.begin_line();
                        s.push_str(&text);
                        s.push('\\');
                    }
                }
                current_segment.clear();
            } else {
                collect_inline_recursive(child, &mut current_segment);
            }
        }

        // Emit final segment (no trailing backslash)
        if !current_segment.trim().is_empty() {
            let text = restore_in_string(current_segment.trim(), opts.placeholders);
            if indent > 0 {
                {
                    let s = lines.begin_line();
                    s.push_str(&indent_str);
                    s.push_str(&text);
                }
            } else {
                lines.push(text);
            }
        }
        return;
    }

    // Normal paragraph: collect all inline text, restore placeholders, then wrap
    let inline_text = collect_inline_text_from_children(&para.children);
    let inline_text = restore_in_string(&inline_text, opts.placeholders);
    let effective_width = opts.max_width.saturating_sub(indent);
    let ind = super::wrap::indent_str(indent);

    let mut para_buf = LineBuffer::new();
    wrap_paragraph(&inline_text, effective_width, 0, &mut para_buf);
    let para_str = para_buf.into_string();

    for (i, line) in para_str.split('\n').enumerate() {
        if indent > 0 {
            if line.is_empty() {
                lines.push_empty();
            } else {
                let s = lines.begin_line();
                s.push_str(&ind);
                s.push_str(line);
            }
        } else if opts.capitalize && i == 0 {
            lines.push(super::normalize::capitalize_first(line));
        } else {
            lines.push(line);
        }
    }
}

fn serialize_pipe_prefixed_paragraph(
    para: &markdown::mdast::Paragraph,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) -> bool {
    let Some(position) = para.position.as_ref() else {
        return false;
    };

    let raw = &opts.source[position.start.offset..position.end.offset];
    let raw_lines: Vec<&str> = raw.lines().collect();
    if raw_lines.is_empty() || !raw_lines.iter().any(|line| line.trim_start().starts_with('|')) {
        return false;
    }

    let ind = super::wrap::indent_str(indent);
    let mut index = 0;
    let mut emitted_segment = false;

    while index < raw_lines.len() {
        while index < raw_lines.len() && raw_lines[index].trim().is_empty() {
            index += 1;
        }
        if index >= raw_lines.len() {
            break;
        }

        if emitted_segment && !lines.last_is_empty() {
            lines.push_empty();
        }

        if raw_lines[index].trim_start().starts_with('|') {
            let start = index;
            while index < raw_lines.len() && raw_lines[index].trim_start().starts_with('|') {
                index += 1;
            }

            let block_lines = format_table_block(&raw_lines[start..index]);

            for line in block_lines {
                if indent > 0 && !line.is_empty() {
                    {
                        let s = lines.begin_line();
                        s.push_str(&ind);
                        s.push_str(&line);
                    }
                } else {
                    lines.push(line);
                }
            }
        } else {
            let start = index;
            while index < raw_lines.len() && !raw_lines[index].trim_start().starts_with('|') {
                index += 1;
            }

            let text_parts: Vec<&str> = raw_lines[start..index]
                .iter()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect();
            if text_parts.is_empty() {
                continue;
            }

            let joined = text_parts.join(" ");
            let text = restore_in_string(&joined, opts.placeholders);
            let effective_width = opts.max_width.saturating_sub(indent);
            let mut para_buf = LineBuffer::new();
            wrap_paragraph(&text, effective_width, 0, &mut para_buf);
            let para_str = para_buf.into_string();

            for (i, line) in para_str.split('\n').enumerate() {
                if indent > 0 {
                    let s = lines.begin_line();
                    s.push_str(&ind);
                    s.push_str(line);
                } else if opts.capitalize && i == 0 {
                    lines.push(super::normalize::capitalize_first(line));
                } else {
                    lines.push(line);
                }
            }
        }

        emitted_segment = true;
    }

    true
}

// ──────────────────────────────────────────────────
// List serialization
// ──────────────────────────────────────────────────

fn serialize_list(
    list: &markdown::mdast::List,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    let ind = super::wrap::indent_str(indent);
    let mut counter = list.start.unwrap_or(1);

    for child in &list.children {
        let Node::ListItem(item) = child else {
            continue;
        };

        // Build marker
        let (marker, marker_width) = if list.ordered {
            let mut buf = itoa::Buffer::new();
            let num_str = buf.format(counter);
            let mut m = String::with_capacity(num_str.len() + 2);
            m.push_str(num_str);
            m.push_str(". ");
            let width = m.len();
            counter += 1;
            (Cow::Owned(m), width)
        } else {
            (Cow::Borrowed("- "), 2)
        };

        // The upstream plugin does NOT add blank lines between list items.
        // Blank lines appear only within an item's children (between paragraphs).

        // Serialize each child of the ListItem
        let mut first_child = true;
        for item_child in &item.children {
            if first_child {
                // First child: prepend the marker
                let child_str = serialize_node_for_list_item(item_child, marker_width, true, opts);

                for (line_idx, line) in child_str.split('\n').enumerate() {
                    if line_idx == 0 {
                        let text = if opts.capitalize {
                            super::normalize::capitalize_first(line)
                        } else {
                            std::borrow::Cow::Borrowed(line)
                        };
                        {
                            let s = lines.begin_line();
                            s.push_str(&ind);
                            s.push_str(&marker);
                            s.push_str(&text);
                        }
                    } else if line.is_empty() {
                        lines.push_empty();
                    } else {
                        {
                            let s = lines.begin_line();
                            s.push_str(&ind);
                            s.push_str(line);
                        }
                    }
                }
                first_child = false;
            } else {
                // Subsequent children: indented by marker width, with blank line separation
                if is_block_node(item_child) && !lines.last_is_empty() {
                    lines.push_empty();
                }

                if matches!(item_child, Node::Definition(_)) {
                    serialize_node(item_child, 0, opts, lines);
                }
                // Nested lists align to the parent item's content block. At the
                // first nesting level that is the parent marker width; at deeper
                // levels we keep the child list under the already-indented content
                // column, which requires one additional marker-width step.
                else if matches!(item_child, Node::List(_)) {
                    let nested_indent = if indent == 0 {
                        indent + marker_width
                    } else {
                        indent + marker_width + marker_width
                    };
                    serialize_node(item_child, nested_indent, opts, lines);
                } else {
                    let child_str =
                        serialize_node_for_list_item(item_child, marker_width, false, opts);
                    let child_ind = super::wrap::indent_str(indent + marker_width);
                    for line in child_str.split('\n') {
                        if line.is_empty() {
                            lines.push_empty();
                        } else {
                            {
                                let s = lines.begin_line();
                                s.push_str(&child_ind);
                                s.push_str(line);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Serialize a node that is a child of a list item.
/// For paragraphs, we collect text, restore placeholders, then wrap.
///
/// `is_first_child`: If true, the first child of the list item. The first line wraps
/// at `max_width` and the marker is prepended by the caller. Continuation lines get
/// `marker_width` indent from wrap_paragraph.
///
/// If false, a subsequent child. The paragraph still wraps at `max_width`; the caller
/// prepends `marker_width` spaces afterward so continuation blocks stay aligned to the
/// list item's content column.
fn serialize_node_for_list_item(
    node: &Node,
    marker_width: usize,
    is_first_child: bool,
    opts: &SerializeOptions<'_>,
) -> String {
    if let Node::Paragraph(para) = node {
        let inline_text = collect_inline_text_from_children(&para.children);
        let inline_text = restore_in_string(&inline_text, opts.placeholders);
        let mut buf = LineBuffer::new();
        if is_first_child {
            wrap_paragraph(&inline_text, opts.max_width, marker_width, &mut buf);
        } else {
            wrap_paragraph(&inline_text, opts.max_width, 0, &mut buf);
        }
        buf.into_string()
    } else {
        let mut buf = LineBuffer::new();
        serialize_node(node, 0, opts, &mut buf);
        buf.into_string()
    }
}

// ──────────────────────────────────────────────────
// Code block serialization
// ──────────────────────────────────────────────────

fn serialize_code(
    code: &markdown::mdast::Code,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    // Blank line before code block
    if !lines.is_empty() && !lines.last_is_empty() {
        lines.push_empty();
    }

    let code_width = opts.max_width.saturating_sub(4);
    let formatted_value = format_code_value(&code.value, code.lang.as_deref(), code_width, opts);

    if let Some(lang) = &code.lang
        && !lang.is_empty()
    {
        // Fenced code block with language
        {
            let s = lines.begin_line();
            s.push_str("```");
            s.push_str(lang);
        }
        for line in formatted_value.lines() {
            lines.push(line);
        }
        lines.push("```");
    } else {
        // No language: indented code block (4-space prefix)
        for line in formatted_value.lines() {
            if line.is_empty() {
                lines.push_empty();
            } else {
                {
                    let s = lines.begin_line();
                    s.push_str("    ");
                    s.push_str(line);
                }
            }
        }
    }
}

/// Try to format the code value using the appropriate formatter.
/// Returns the formatted code if successful, or the original code as-is.
fn format_code_value<'a>(
    code: &'a str,
    lang: Option<&str>,
    width: usize,
    opts: &SerializeOptions<'_>,
) -> Cow<'a, str> {
    let (Some(format_options), Some(allocator)) = (opts.format_options, opts.allocator) else {
        return Cow::Borrowed(code);
    };

    if let Some(lang) = lang {
        // Case-insensitive matching (matches upstream's `mdAst.lang.toLowerCase()`)
        let lang_lower = lang.cow_to_ascii_lowercase();
        let lang = lang_lower.as_ref();

        // JS/TS: native formatter
        if super::serialize::is_js_ts_lang(lang)
            && let Some(formatted) =
                super::serialize::format_embedded_js(code, width, format_options, allocator)
        {
            return Cow::Owned(formatted);
        }
        // CSS/HTML/GraphQL/MD/YAML: external formatter
        if let Some(ext_lang) = super::serialize::fenced_lang_to_external_language(lang)
            && let Some(cbs) = opts.external_callbacks
            && let Some(formatted) =
                super::serialize::format_external_language(code, ext_lang, width, cbs)
        {
            return Cow::Owned(formatted);
        }
        // Unknown language: fall back to JS (matches upstream default "babel" parser)
        if let Some(formatted) =
            super::serialize::format_embedded_js(code, width, format_options, allocator)
        {
            return Cow::Owned(formatted);
        }
        Cow::Borrowed(code)
    } else {
        // No language: try as JS (matches upstream default "babel" parser)
        if let Some(formatted) =
            super::serialize::format_embedded_js(code, width, format_options, allocator)
        {
            Cow::Owned(formatted)
        } else {
            Cow::Borrowed(code)
        }
    }
}

// ──────────────────────────────────────────────────
// Blockquote serialization
// ──────────────────────────────────────────────────

fn serialize_blockquote(
    bq: &markdown::mdast::Blockquote,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    // Serialize each child of the blockquote separately.
    // Between block-level children, emit a bare blank line (no `>` prefix)
    // to match the upstream plugin's behavior of separating blockquote
    // sections with blank comment lines.
    for (i, child) in bq.children.iter().enumerate() {
        if i > 0 {
            // Blank line between blockquote sections (no `>` prefix)
            lines.push_empty();
        }
        let mut inner_buf = LineBuffer::new();
        serialize_node(child, 0, opts, &mut inner_buf);
        for line in inner_buf.into_string().split('\n') {
            if line.is_empty() {
                lines.push(">");
            } else {
                {
                    let s = lines.begin_line();
                    s.push_str("> ");
                    s.push_str(line);
                }
            }
        }
    }
}

// ──────────────────────────────────────────────────
// Inline text collection
// ──────────────────────────────────────────────────

/// Collect inline content from a node into a single text string.
/// This handles emphasis, strong, code, links, etc.
/// Note: the returned text may contain placeholder tokens which must be
/// restored before being used in output.
fn collect_inline_text(node: &Node) -> String {
    let mut result = String::new();
    collect_inline_recursive(node, &mut result);
    result
}

/// Collect inline text from a slice of child nodes directly, avoiding
/// the need to clone a parent node just to iterate its children.
fn collect_inline_text_from_children(children: &[Node]) -> String {
    let mut result = String::new();
    for child in children {
        collect_inline_recursive(child, &mut result);
    }
    result
}

fn collect_inline_recursive(node: &Node, out: &mut String) {
    match node {
        Node::Text(text) => {
            out.push_str(&text.value);
        }
        Node::Emphasis(emp) => {
            out.push('_');
            for child in &emp.children {
                collect_inline_recursive(child, out);
            }
            out.push('_');
        }
        Node::Strong(strong) => {
            out.push_str("**");
            for child in &strong.children {
                collect_inline_recursive(child, out);
            }
            out.push_str("**");
        }
        Node::InlineCode(code) => {
            out.push('`');
            out.push_str(&code.value);
            out.push('`');
        }
        Node::Link(link) => {
            // Check if this is a GFM autolink (bare URL converted to link).
            // If the link text equals the URL, emit just the URL.
            let link_text = {
                let mut t = String::new();
                for child in &link.children {
                    collect_inline_recursive(child, &mut t);
                }
                t
            };
            if link_text == link.url && link.title.is_none() {
                out.push_str(&link.url);
            } else {
                out.push('[');
                out.push_str(&link_text);
                out.push_str("](");
                out.push_str(&link.url);
                if let Some(title) = &link.title {
                    out.push_str(" \"");
                    out.push_str(title);
                    out.push('"');
                }
                out.push(')');
            }
        }
        Node::LinkReference(link_ref) => {
            out.push('[');
            for child in &link_ref.children {
                collect_inline_recursive(child, out);
            }
            out.push(']');
            let label = link_ref.label.as_deref().unwrap_or(&link_ref.identifier);
            match link_ref.reference_kind {
                markdown::mdast::ReferenceKind::Full | markdown::mdast::ReferenceKind::Shortcut => {
                    out.push('[');
                    out.push_str(label);
                    out.push(']');
                }
                markdown::mdast::ReferenceKind::Collapsed => {
                    out.push_str("[]");
                }
            }
        }
        Node::Image(image) => {
            out.push_str("![");
            out.push_str(&image.alt);
            out.push_str("](");
            out.push_str(&image.url);
            if let Some(title) = &image.title {
                out.push_str(" \"");
                out.push_str(title);
                out.push('"');
            }
            out.push(')');
        }
        Node::ImageReference(img_ref) => {
            out.push_str("![");
            out.push_str(&img_ref.alt);
            out.push(']');
            let label = img_ref.label.as_deref().unwrap_or(&img_ref.identifier);
            match img_ref.reference_kind {
                markdown::mdast::ReferenceKind::Full | markdown::mdast::ReferenceKind::Shortcut => {
                    out.push('[');
                    out.push_str(label);
                    out.push(']');
                }
                markdown::mdast::ReferenceKind::Collapsed => {
                    out.push_str("[]");
                }
            }
        }
        Node::Break(_) => {
            // Break nodes in inline context: just a space (actual line breaking
            // is handled by serialize_paragraph for paragraph-level breaks)
            out.push(' ');
        }
        Node::Delete(del) => {
            out.push_str("~~");
            for child in &del.children {
                collect_inline_recursive(child, out);
            }
            out.push_str("~~");
        }
        Node::FootnoteReference(fn_ref) => {
            out.push_str("[^");
            out.push_str(&fn_ref.identifier);
            out.push(']');
        }
        Node::Html(html) => {
            out.push_str(&html.value);
        }
        // For parent nodes, recurse into children
        Node::Paragraph(para) => {
            for child in &para.children {
                collect_inline_recursive(child, out);
            }
        }
        Node::Heading(h) => {
            for child in &h.children {
                collect_inline_recursive(child, out);
            }
        }
        Node::TableCell(cell) => {
            for child in &cell.children {
                collect_inline_recursive(child, out);
            }
        }
        _ => {
            // Fallback: use the node's to_string
            out.push_str(&node.to_string());
        }
    }
}
