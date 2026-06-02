use std::borrow::Cow;

use markdown::mdast::Node;

use super::super::embedded::{format_embedded_js, is_js_ts_lang};
use super::super::line_buffer::LineBuffer;
use super::super::wrap::{format_table_block, wrap_paragraph};
use super::SerializeOptions;
use super::collect::{
    collect_inline_recursive, collect_inline_text, collect_inline_text_from_children,
};

/// Serialize children of a parent node, inserting blank lines between block-level nodes.
///
/// Consecutive Paragraph and inline-like Html nodes are merged into a single
/// paragraph so that HTML tag references like `<option>` or `<div>` that the
/// markdown parser extracted as separate Html block nodes don't create spurious
/// blank lines in the middle of what should be a single paragraph.
pub(super) fn serialize_children(
    node: &Node,
    indent: usize,
    first_para_offset: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    let Some(children) = node.children() else {
        return;
    };

    let mut i = 0;
    while i < children.len() {
        let child = &children[i];

        // Try to merge a run of Paragraph + inline-Html nodes into one paragraph.
        if matches!(child, Node::Paragraph(_) | Node::Html(_)) {
            let run_start = i;
            let mut run_end = i + 1;
            let mut has_html = matches!(child, Node::Html(_));

            // Extend the run while next nodes are Paragraphs or inline-like Html
            while run_end < children.len() {
                match &children[run_end] {
                    Node::Paragraph(_) => {
                        run_end += 1;
                    }
                    Node::Html(html) if is_inline_html(&html.value) => {
                        has_html = true;
                        run_end += 1;
                    }
                    _ => break,
                }
            }

            // If we found a mixed run (has Html nodes mixed with Paragraphs),
            // merge them into a single paragraph text.
            if has_html && run_end - run_start > 1 {
                // Add blank line before the merged paragraph if needed
                if run_start > 0 && !lines.last_is_empty() {
                    lines.push_empty();
                }

                let mut merged_text = String::new();
                for child in children.iter().take(run_end).skip(run_start) {
                    match child {
                        Node::Paragraph(para) => {
                            let text = collect_inline_text_from_children(&para.children);
                            if !merged_text.is_empty() && !merged_text.ends_with(' ') {
                                merged_text.push(' ');
                            }
                            merged_text.push_str(text.trim());
                        }
                        Node::Html(html) => {
                            if !merged_text.is_empty() && !merged_text.ends_with(' ') {
                                merged_text.push(' ');
                            }
                            merged_text.push_str(html.value.trim());
                        }
                        _ => {}
                    }
                }

                // Wrap the merged text as a single paragraph
                let offset = if run_start == 0 { first_para_offset } else { 0 };
                let effective_width = opts.max_width.saturating_sub(indent);
                let ind = super::super::wrap::indent_str(indent);
                let mut para_buf = LineBuffer::new();
                wrap_paragraph(&merged_text, effective_width, offset, 0, &mut para_buf);
                let para_str = para_buf.into_string();

                let mut para_iter = para_str.split('\n').peekable();
                let mut li = 0usize;
                while let Some(line) = para_iter.next() {
                    let is_last = para_iter.peek().is_none();
                    if indent > 0 {
                        if line.is_empty() {
                            lines.push_empty();
                        } else if opts.description_with_dot && is_last {
                            let dotted = super::super::normalize::append_trailing_dot(line);
                            let s = lines.begin_line();
                            s.push_str(&ind);
                            s.push_str(&dotted);
                        } else {
                            let s = lines.begin_line();
                            s.push_str(&ind);
                            s.push_str(line);
                        }
                    } else if opts.capitalize && li == 0 {
                        let cap = super::super::normalize::capitalize_first(line);
                        if opts.description_with_dot && is_last {
                            lines.push(super::super::normalize::append_trailing_dot(&cap));
                        } else {
                            lines.push(cap);
                        }
                    } else if opts.description_with_dot && is_last {
                        lines.push(super::super::normalize::append_trailing_dot(line));
                    } else {
                        lines.push(line);
                    }
                    li += 1;
                }

                i = run_end;
                continue;
            }
        }

        // Normal path: add blank line between block-level siblings (except first)
        if i > 0 && is_block_node(child) && !lines.last_is_empty() {
            lines.push_empty();
        }

        // Only the first paragraph gets the tag-string-length offset
        let offset = if i == 0 { first_para_offset } else { 0 };
        serialize_node(child, indent, offset, opts, lines);
        i += 1;
    }
}

/// Check if an HTML node looks like an inline tag reference that the markdown
/// parser incorrectly extracted as a block-level element.
///
/// In JSDoc descriptions, `<div>`, `<table>`, etc. are usually mentioned as
/// tag names (e.g., "renders a `<div>` element") rather than actual HTML blocks.
/// The CommonMark parser treats these as HTML block starts, absorbing subsequent
/// text into the Html node. This function detects such cases so they can be
/// merged back into the surrounding paragraph.
///
/// Returns `true` when the Html node content looks like an inline tag reference
/// (possibly followed by absorbed paragraph text), NOT a genuine HTML block
/// with structured content.
fn is_inline_html(html: &str) -> bool {
    let trimmed = html.trim();
    if !trimmed.starts_with('<') {
        return false;
    }

    // Find the end of the first tag
    let Some(tag_end) = trimmed.find('>') else {
        return false;
    };

    // Extract the tag name (strip `<`, `/`, attributes)
    let tag_content = &trimmed[1..tag_end];
    let tag_name = tag_content
        .trim_start_matches('/')
        .split(|c: char| c.is_ascii_whitespace() || c == '/')
        .next()
        .unwrap_or("");

    if tag_name.is_empty() {
        return false;
    }

    // If the content after the first tag is just a closing tag or empty, it's
    // a simple inline reference like `<div>` or `<div></div>`
    let after_tag = trimmed[tag_end + 1..].trim();
    if after_tag.is_empty() {
        return true;
    }

    // If the content after the tag looks like plain text (no more HTML structure),
    // this is likely a tag-name mention that absorbed following paragraph text.
    // e.g., "<div>\nelement with the given props."
    //
    // Check: does the remaining content contain a matching closing tag with
    // structured content? If not, it's likely absorbed paragraph text.
    let closing_tag = format!("</{tag_name}>");
    if after_tag.contains(&closing_tag) {
        // Has a matching closing tag — could be genuine HTML block.
        // But in JSDoc context, even `<div>...</div>` is usually inline.
        // Be conservative: if there's a closing tag, check if it's the
        // entire content (e.g., `<div>content</div>`) or if there's text after.
        // For JSDoc purposes, treat it all as inline.
        return true;
    }

    // No closing tag — the parser absorbed following paragraph text after the
    // block-level tag. This is the exact bug scenario.
    true
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
    first_para_offset: usize,
    opts: &SerializeOptions<'_>,
    lines: &mut LineBuffer,
) {
    match node {
        Node::Root(_) => {
            serialize_children(node, indent, 0, opts, lines);
        }
        Node::Paragraph(para) => {
            serialize_paragraph(para, indent, first_para_offset, opts, lines);
        }
        Node::Heading(heading) => {
            let text = collect_inline_text(node);
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
    first_line_offset: usize,
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
        let indent_str = super::super::wrap::indent_str(indent);
        let mut current_segment = String::new();

        for child in &para.children {
            if matches!(child, Node::Break(_)) {
                // Emit current segment with trailing backslash
                let text = current_segment.trim();
                if indent > 0 {
                    {
                        let s = lines.begin_line();
                        s.push_str(&indent_str);
                        s.push_str(text);
                        s.push('\\');
                    }
                } else if opts.capitalize && lines.is_empty() {
                    let text = super::super::normalize::capitalize_first(text);
                    {
                        let s = lines.begin_line();
                        s.push_str(&text);
                        s.push('\\');
                    }
                } else {
                    {
                        let s = lines.begin_line();
                        s.push_str(text);
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
            let text = current_segment.trim();
            if indent > 0 {
                {
                    let s = lines.begin_line();
                    s.push_str(&indent_str);
                    s.push_str(text);
                }
            } else {
                lines.push(text);
            }
        }
        return;
    }

    // Normal paragraph: collect all inline text, then wrap (placeholders restored at the end)
    let inline_text = collect_inline_text_from_children(&para.children);
    let effective_width = opts.max_width.saturating_sub(indent);
    let ind = super::super::wrap::indent_str(indent);

    // Balance mode: try to preserve original line breaks if all lines fit
    if matches!(opts.line_wrapping_style, crate::LineWrappingStyle::Balance)
        && let Some(position) = para.position.as_ref()
    {
        let raw = &opts.source[position.start.offset..position.end.offset];
        let original_lines: Vec<&str> =
            raw.lines().map(str::trim).filter(|l| !l.is_empty()).collect();

        if original_lines.len() > 1
            && original_lines.iter().all(|l| super::super::wrap::str_width(l) <= effective_width)
        {
            // Preserve original line breaks
            let total = original_lines.len();
            for (i, orig_line) in original_lines.iter().enumerate() {
                let is_first = i == 0;
                let is_last = i == total - 1;

                let line: String = {
                    let mut s = (*orig_line).to_owned();
                    if is_first && opts.capitalize {
                        s = super::super::normalize::capitalize_first(&s).into_owned();
                    }
                    if is_last && opts.description_with_dot {
                        s = super::super::normalize::append_trailing_dot(&s).into_owned();
                    }
                    s
                };

                if indent > 0 {
                    let s = lines.begin_line();
                    s.push_str(&ind);
                    s.push_str(&line);
                } else {
                    lines.push(&line);
                }
            }
            return;
        }
        // Fall through to greedy wrapping
    }

    let mut para_buf = LineBuffer::new();
    wrap_paragraph(&inline_text, effective_width, first_line_offset, 0, &mut para_buf);
    let para_str = para_buf.into_string();

    let mut para_iter = para_str.split('\n').peekable();
    let mut i = 0usize;
    while let Some(line) = para_iter.next() {
        let is_last = para_iter.peek().is_none();
        if indent > 0 {
            if line.is_empty() {
                lines.push_empty();
            } else if opts.description_with_dot && is_last {
                let dotted = super::super::normalize::append_trailing_dot(line);
                let s = lines.begin_line();
                s.push_str(&ind);
                s.push_str(&dotted);
            } else {
                let s = lines.begin_line();
                s.push_str(&ind);
                s.push_str(line);
            }
        } else if opts.capitalize && i == 0 {
            let cap = super::super::normalize::capitalize_first(line);
            if opts.description_with_dot && is_last {
                lines.push(super::super::normalize::append_trailing_dot(&cap));
            } else {
                lines.push(cap);
            }
        } else if opts.description_with_dot && is_last {
            lines.push(super::super::normalize::append_trailing_dot(line));
        } else {
            lines.push(line);
        }
        i += 1;
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
    // A table row needs at least 2 pipes: |cell| — a single | in prose (e.g. `|splineCurve|`)
    // is not a table.
    if raw_lines.is_empty()
        || !raw_lines.iter().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2
        })
    {
        return false;
    }

    let ind = super::super::wrap::indent_str(indent);
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

        let is_table = {
            let trimmed = raw_lines[index].trim_start();
            trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2
        };
        if is_table {
            let start = index;
            while index < raw_lines.len() && {
                let t = raw_lines[index].trim_start();
                t.starts_with('|') && t.ends_with('|') && t.len() > 2
            } {
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
            while index < raw_lines.len()
                && !{
                    let t = raw_lines[index].trim_start();
                    t.starts_with('|') && t.ends_with('|') && t.len() > 2
                }
            {
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
            let effective_width = opts.max_width.saturating_sub(indent);
            let mut para_buf = LineBuffer::new();
            wrap_paragraph(&joined, effective_width, 0, 0, &mut para_buf);
            let para_str = para_buf.into_string();

            for (i, line) in para_str.split('\n').enumerate() {
                if indent > 0 {
                    let s = lines.begin_line();
                    s.push_str(&ind);
                    s.push_str(line);
                } else if opts.capitalize && i == 0 {
                    lines.push(super::super::normalize::capitalize_first(line));
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
    let ind = super::super::wrap::indent_str(indent);
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

        // Serialize each child of the ListItem
        let mut first_child = true;
        for item_child in &item.children {
            if first_child {
                // First child: prepend the marker
                let child_str = serialize_node_for_list_item(item_child, marker_width, true, opts);

                for (line_idx, line) in child_str.split('\n').enumerate() {
                    if line_idx == 0 {
                        let text = if opts.capitalize {
                            super::super::normalize::capitalize_first(line)
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
                // Subsequent children: indented by marker width, with blank line separation.
                // Always add a blank line before nested lists and other block-level nodes
                // when they follow the first child of the list item (matching upstream behavior).
                if is_block_node(item_child) && !lines.last_is_empty() {
                    lines.push_empty();
                }

                if matches!(item_child, Node::Definition(_)) {
                    serialize_node(item_child, 0, 0, opts, lines);
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
                    serialize_node(item_child, nested_indent, 0, opts, lines);
                } else {
                    let child_str =
                        serialize_node_for_list_item(item_child, marker_width, false, opts);
                    let child_ind = super::super::wrap::indent_str(indent + marker_width);
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
        let mut buf = LineBuffer::new();
        if is_first_child {
            wrap_paragraph(&inline_text, opts.max_width, 0, marker_width, &mut buf);
        } else {
            wrap_paragraph(&inline_text, opts.max_width, 0, 0, &mut buf);
        }
        buf.into_string()
    } else {
        let mut buf = LineBuffer::new();
        serialize_node(node, 0, 0, opts, &mut buf);
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

    let has_lang = code.lang.as_ref().is_some_and(|l| !l.is_empty());
    let use_fence = has_lang || opts.prefer_code_fences;

    if use_fence {
        // Fenced code block
        {
            let s = lines.begin_line();
            s.push_str("```");
            if let Some(lang) = &code.lang {
                s.push_str(lang);
            }
        }
        for line in formatted_value.lines() {
            lines.push(line);
        }
        lines.push("```");
    } else {
        // No language: indented code block (4-space prefix).
        // Strip common leading whitespace from the code value first — the markdown
        // parser may leave residual indent when the source had more than 4 spaces
        // (e.g. continuation indent + code indent). Re-adding exactly 4 spaces
        // normalizes the output.
        let min_indent = formatted_value
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        for line in formatted_value.lines() {
            if line.is_empty() {
                lines.push_empty();
            } else {
                {
                    let s = lines.begin_line();
                    s.push_str("    ");
                    if min_indent > 0 && line.len() >= min_indent {
                        s.push_str(&line[min_indent..]);
                    } else {
                        s.push_str(line);
                    }
                }
            }
        }
    }
}

/// Format code block content in JSDoc descriptions.
/// For JS/TS code (fenced with a JS/TS lang tag, fenced with no lang tag, or
/// indented code blocks), the code is formatted through Prettier's parser via
/// `format_embedded_js`. For non-JS/TS fenced code (css, html, etc.), the code
/// is preserved verbatim.
fn format_code_value<'a>(
    code: &'a str,
    lang: Option<&str>,
    width: usize,
    opts: &SerializeOptions<'_>,
) -> Cow<'a, str> {
    if let (Some(format_options), Some(allocator)) = (opts.format_options, opts.allocator) {
        // For fenced code with an explicit non-JS/TS lang, try external formatter (CSS, HTML, etc.)
        if let Some(l) = lang
            && !is_js_ts_lang(l)
        {
            if let Some(external) = opts.external_callbacks
                && let Some(Ok(formatted)) = external.format_embedded(l, code)
            {
                let mut result = formatted;
                // Trim trailing newline that Prettier adds
                if result.ends_with('\n') {
                    result.pop();
                }
                return Cow::Owned(result);
            }
            return Cow::Borrowed(code);
        }
        // Fenced JS/TS, fenced with no lang, or indented code: try formatting
        if let Some(formatted) = format_embedded_js(code, width, format_options, allocator) {
            return Cow::Owned(formatted);
        }
    }
    Cow::Borrowed(code)
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
        serialize_node(child, 0, 0, opts, &mut inner_buf);
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
