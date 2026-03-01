use cow_utils::CowUtils;
use markdown::{Constructs, ParseOptions, mdast::Node, to_mdast};

use super::wrap::{format_table_block, wrap_paragraph};

/// Placeholder prefix for protecting `{@link ...}` tokens from markdown parsing.
/// Uses a format that `tokenize_words` won't split (no spaces, looks like a word).
const PLACEHOLDER_PREFIX: &str = "\x00JDLNK";

/// Format a markdown description using mdast parsing.
///
/// Parses the text into a markdown AST, then serializes it back to formatted
/// text with proper indentation, wrapping, and emphasis normalization.
/// This replaces the manual normalize+wrap pipeline with an approach matching
/// the upstream prettier-plugin-jsdoc's use of `fromMarkdown` + `stringify`.
pub fn format_description_mdast(text: &str, max_width: usize, capitalize: bool) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
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
        return text.lines().map(|l| l.trim().to_string()).collect();
    };

    let mut lines = Vec::new();
    let opts =
        SerializeOptions { max_width, capitalize, placeholders: &placeholders, source: &protected };
    serialize_children(&root, 0, &opts, &mut lines);

    // Restore any remaining placeholders in output lines
    restore_placeholders(&mut lines, &placeholders);

    // Remove trailing blank lines
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }

    lines
}

struct SerializeOptions<'a> {
    max_width: usize,
    capitalize: bool,
    placeholders: &'a [String],
    source: &'a str,
}

// ──────────────────────────────────────────────────
// JSDoc link protection
// ──────────────────────────────────────────────────

/// Normalize the legacy `1- foo` list-marker style used by some existing JSDoc
/// fixtures into standard ordered-list syntax so markdown parsing can treat them
/// as list items.
fn normalize_legacy_ordered_list_markers(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

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
                    continue;
                }
            }
        }

        result.push_str(line);
    }

    result
}

/// Replace `{@link ...}`, `{@linkcode ...}`, `{@linkplain ...}`, `{@tutorial ...}`
/// with numbered placeholders so the markdown parser (especially GFM autolink) doesn't
/// mangle URLs inside them.
fn protect_jsdoc_links(text: &str) -> (String, Vec<String>) {
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
            placeholders.push(token.to_string());
            // Use a placeholder that looks like a single word (no spaces)
            // so tokenize_words treats it atomically
            result.push_str(PLACEHOLDER_PREFIX);
            result.push_str(&idx.to_string());
        } else {
            let ch = text[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    (result, placeholders)
}

/// Restore all placeholder tokens in a string back to their original `{@link ...}` form.
fn restore_in_string(s: &str, placeholders: &[String]) -> String {
    if placeholders.is_empty() && !s.contains(PLACEHOLDER_PREFIX) {
        return s.to_string();
    }

    let mut result = s.to_string();

    for (idx, original) in placeholders.iter().enumerate() {
        let placeholder = format!("{PLACEHOLDER_PREFIX}{idx}");
        if result.contains(&placeholder) {
            result = result.cow_replace(&*placeholder, original.as_str()).into_owned();
        }
    }
    result
}

/// Restore placeholders in all output lines.
fn restore_placeholders(lines: &mut [String], placeholders: &[String]) {
    for line in lines.iter_mut() {
        if line.contains(PLACEHOLDER_PREFIX) {
            *line = restore_in_string(line, placeholders);
        }
    }
}

// ──────────────────────────────────────────────────
// Node serialization
// ──────────────────────────────────────────────────

/// Serialize children of a parent node, inserting blank lines between block-level nodes.
fn serialize_children(
    node: &Node,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut Vec<String>,
) {
    let Some(children) = node.children() else {
        return;
    };

    for (i, child) in children.iter().enumerate() {
        // Add blank line between block-level siblings (except first)
        if i > 0 && is_block_node(child) && !lines.last().is_some_and(String::is_empty) {
            lines.push(String::new());
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

fn serialize_node(
    node: &Node,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut Vec<String>,
) {
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
            if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
                lines.push(String::new());
            }
            lines.push(format!("{prefix} {text}"));
        }
        Node::List(list) => {
            serialize_list(list, indent, opts, lines);
        }
        // ListItems are handled by serialize_list; thematic breaks are dropped (matches upstream)
        Node::ListItem(_) | Node::ThematicBreak(_) => {}
        Node::Code(code) => {
            serialize_code(code, lines);
        }
        Node::Blockquote(bq) => {
            serialize_blockquote(bq, opts, lines);
        }
        Node::Definition(def) => {
            let label = def.label.as_deref().unwrap_or(&def.identifier);
            lines.push(format!("[{label}]: {}", def.url));
        }
        Node::Html(html) => {
            for line in html.value.lines() {
                lines.push(line.to_string());
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
    lines: &mut Vec<String>,
) {
    if serialize_pipe_prefixed_paragraph(para, indent, opts, lines) {
        return;
    }

    // Check if the paragraph contains Break nodes (hard line breaks from `\` at EOL)
    let has_breaks = para.children.iter().any(|c| matches!(c, Node::Break(_)));

    if has_breaks {
        // Split into segments at Break nodes, each segment on its own line
        let indent_str = " ".repeat(indent);
        let mut current_segment = String::new();

        for child in &para.children {
            if matches!(child, Node::Break(_)) {
                // Emit current segment with trailing backslash
                let text = current_segment.trim().to_string();
                let text = restore_in_string(&text, opts.placeholders);
                if indent > 0 {
                    lines.push(format!("{indent_str}{text}\\"));
                } else {
                    let text = if opts.capitalize && lines.is_empty() {
                        super::normalize::capitalize_first(&text)
                    } else {
                        text
                    };
                    lines.push(format!("{text}\\"));
                }
                current_segment.clear();
            } else {
                collect_inline_recursive(child, &mut current_segment);
            }
        }

        // Emit final segment (no trailing backslash)
        if !current_segment.trim().is_empty() {
            let text = current_segment.trim().to_string();
            let text = restore_in_string(&text, opts.placeholders);
            if indent > 0 {
                lines.push(format!("{indent_str}{text}"));
            } else {
                lines.push(text);
            }
        }
        return;
    }

    // Normal paragraph: collect all inline text, restore placeholders, then wrap
    let inline_text = collect_inline_text(&Node::Paragraph(para.clone()));
    let inline_text = restore_in_string(&inline_text, opts.placeholders);
    let effective_width = opts.max_width.saturating_sub(indent);
    let indent_str = " ".repeat(indent);

    let mut para_lines = Vec::new();
    wrap_paragraph(&inline_text, effective_width, 0, &mut para_lines);

    for (i, line) in para_lines.iter().enumerate() {
        if indent > 0 {
            lines.push(format!("{indent_str}{line}"));
        } else {
            let line = if opts.capitalize && i == 0 {
                super::normalize::capitalize_first(line)
            } else {
                line.clone()
            };
            lines.push(line);
        }
    }
}

fn serialize_pipe_prefixed_paragraph(
    para: &markdown::mdast::Paragraph,
    indent: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut Vec<String>,
) -> bool {
    let Some(position) = para.position.as_ref() else {
        return false;
    };

    let raw = &opts.source[position.start.offset..position.end.offset];
    let raw_lines: Vec<&str> = raw.lines().collect();
    if raw_lines.is_empty() || !raw_lines.iter().any(|line| line.trim_start().starts_with('|')) {
        return false;
    }

    let indent_str = " ".repeat(indent);
    let mut index = 0;
    let mut emitted_segment = false;

    while index < raw_lines.len() {
        while index < raw_lines.len() && raw_lines[index].trim().is_empty() {
            index += 1;
        }
        if index >= raw_lines.len() {
            break;
        }

        if emitted_segment && !lines.last().is_some_and(String::is_empty) {
            lines.push(String::new());
        }

        if raw_lines[index].trim_start().starts_with('|') {
            let start = index;
            while index < raw_lines.len() && raw_lines[index].trim_start().starts_with('|') {
                index += 1;
            }

            let pipe_lines: Vec<String> = raw_lines[start..index]
                .iter()
                .map(|line| restore_in_string(line.trim(), opts.placeholders))
                .collect();
            let mut block_lines = Vec::new();
            format_table_block(&pipe_lines, &mut block_lines);

            for line in block_lines {
                if indent > 0 && !line.is_empty() {
                    lines.push(format!("{indent_str}{line}"));
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

            let text = restore_in_string(&text_parts.join(" "), opts.placeholders);
            let effective_width = opts.max_width.saturating_sub(indent);
            let mut para_lines = Vec::new();
            wrap_paragraph(&text, effective_width, 0, &mut para_lines);

            for (i, line) in para_lines.iter().enumerate() {
                if indent > 0 {
                    lines.push(format!("{indent_str}{line}"));
                } else {
                    let line = if opts.capitalize && i == 0 {
                        super::normalize::capitalize_first(line)
                    } else {
                        line.clone()
                    };
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
    lines: &mut Vec<String>,
) {
    let indent_str = " ".repeat(indent);
    let mut counter = list.start.unwrap_or(1);

    for child in &list.children {
        let Node::ListItem(item) = child else {
            continue;
        };

        // Build marker
        let marker = if list.ordered {
            let m = format!("{counter}. ");
            counter += 1;
            m
        } else {
            "- ".to_string()
        };
        let marker_width = marker.len();

        // The upstream plugin does NOT add blank lines between list items.
        // Blank lines appear only within an item's children (between paragraphs).

        // Serialize each child of the ListItem
        let mut first_child = true;
        for item_child in &item.children {
            if first_child {
                // First child: prepend the marker
                let mut child_lines = Vec::new();
                serialize_node_for_list_item(
                    item_child,
                    marker_width,
                    true,
                    opts,
                    &mut child_lines,
                );

                for (line_idx, line) in child_lines.iter().enumerate() {
                    if line_idx == 0 {
                        let text = if opts.capitalize {
                            super::normalize::capitalize_first(line)
                        } else {
                            line.clone()
                        };
                        lines.push(format!("{indent_str}{marker}{text}"));
                    } else if line.is_empty() {
                        lines.push(String::new());
                    } else {
                        // wrap_paragraph already adds marker_width indent to
                        // continuation lines, so only prepend outer indent.
                        lines.push(format!("{indent_str}{line}"));
                    }
                }
                first_child = false;
            } else {
                // Subsequent children: indented by marker width, with blank line separation
                if is_block_node(item_child) && !lines.last().is_some_and(String::is_empty) {
                    lines.push(String::new());
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
                    let mut child_lines = Vec::new();
                    serialize_node_for_list_item(
                        item_child,
                        marker_width,
                        false,
                        opts,
                        &mut child_lines,
                    );
                    let child_indent = " ".repeat(indent + marker_width);
                    for line in &child_lines {
                        if line.is_empty() {
                            lines.push(String::new());
                        } else {
                            lines.push(format!("{child_indent}{line}"));
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
    lines: &mut Vec<String>,
) {
    match node {
        Node::Paragraph(para) => {
            let inline_text = collect_inline_text(&Node::Paragraph(para.clone()));
            let inline_text = restore_in_string(&inline_text, opts.placeholders);
            if is_first_child {
                // First line wraps at max_width; continuation at max_width - marker_width.
                // The caller prepends the marker to the first line.
                wrap_paragraph(&inline_text, opts.max_width, marker_width, lines);
            } else {
                // Wrap at max_width (full width), then let the caller add the
                // list-item indent so continuation blocks stay aligned under the
                // marker's content column.
                wrap_paragraph(&inline_text, opts.max_width, 0, lines);
            }
        }
        _ => {
            serialize_node(node, 0, opts, lines);
        }
    }
}

// ──────────────────────────────────────────────────
// Code block serialization
// ──────────────────────────────────────────────────

fn serialize_code(code: &markdown::mdast::Code, lines: &mut Vec<String>) {
    if let Some(lang) = &code.lang
        && !lang.is_empty()
    {
        // Fenced code block with language
        if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
            lines.push(String::new());
        }
        lines.push(format!("```{lang}"));
        for line in code.value.lines() {
            lines.push(line.to_string());
        }
        lines.push("```".to_string());
        return;
    }

    // No language: convert to indented code block (matches upstream behavior)
    if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
        lines.push(String::new());
    }
    for line in code.value.lines() {
        if line.is_empty() {
            lines.push(String::new());
        } else {
            lines.push(format!("    {line}"));
        }
    }
}

// ──────────────────────────────────────────────────
// Blockquote serialization
// ──────────────────────────────────────────────────

fn serialize_blockquote(
    bq: &markdown::mdast::Blockquote,
    opts: &SerializeOptions<'_>,
    lines: &mut Vec<String>,
) {
    // Serialize each child of the blockquote separately.
    // Between block-level children, emit a bare blank line (no `>` prefix)
    // to match the upstream plugin's behavior of separating blockquote
    // sections with blank comment lines.
    for (i, child) in bq.children.iter().enumerate() {
        if i > 0 {
            // Blank line between blockquote sections (no `>` prefix)
            lines.push(String::new());
        }
        let mut inner_lines = Vec::new();
        serialize_node(child, 0, opts, &mut inner_lines);
        for line in inner_lines {
            if line.is_empty() {
                lines.push(">".to_string());
            } else {
                lines.push(format!("> {line}"));
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
