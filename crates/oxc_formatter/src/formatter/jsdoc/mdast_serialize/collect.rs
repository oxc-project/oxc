use markdown::{mdast::Node, unist::Position};

// The collected text must parse back to the same nodes and stay inside the JSDoc block (no `*/`).

/// Collect inline content from a node into a single text string.
/// This handles emphasis, strong, code, links, etc.
/// `source` is the text the node positions point into.
/// Note: the returned text may contain placeholder tokens which must be
/// restored before being used in output.
pub(super) fn collect_inline_text(node: &Node, source: &str) -> String {
    let mut result = String::new();
    collect_inline_recursive(node, source, &mut result);
    result
}

/// Collect inline text from a slice of child nodes directly, avoiding
/// the need to clone a parent node just to iterate its children.
pub(super) fn collect_inline_text_from_children(children: &[Node], source: &str) -> String {
    let mut result = String::new();
    for child in children {
        collect_inline_recursive(child, source, &mut result);
    }
    result
}

pub(super) fn collect_inline_recursive(node: &Node, source: &str, out: &mut String) {
    collect_inline_impl(node, source, out, false);
}

/// Write the text as it appears in `source`.
/// The parsed value drops escapes (`\*`) and decodes character references (`&amp;`):
/// written back, they would become syntax or change meaning.
/// Continuation lines drop their leading whitespace and blockquote markers, as the parser does.
fn push_raw_text(text: &markdown::mdast::Text, source: &str, out: &mut String) {
    let Some(position) = &text.position else {
        out.push_str(&text.value);
        return;
    };
    // markdown-rs starts a text that opens with an escape after its `\` (`\*a` is at `*a`).
    // A `\` right before a text belongs to no other node (they all close with another character),
    // so it is that escape.
    let mut start = position.start.offset;
    if source[..start].ends_with('\\') {
        start -= 1;
    }
    let mut lines = source[start..position.end.offset].split('\n');
    out.push_str(lines.next().unwrap());
    for line in lines {
        out.push('\n');
        out.push_str(line.trim_start_matches([' ', '\t', '>']));
    }
}

/// The emphasis marker for the emphasis at `position`:
/// `_` (upstream's style) unless inside a word, where only `*` delimits (`a*b*c`).
pub(super) fn emphasis_marker(position: Option<&Position>, source: &str) -> &'static str {
    let Some(position) = position else { return "_" };
    let before = source[..position.start.offset].chars().next_back();
    let after = source[position.end.offset..].chars().next();
    if before.is_some_and(char::is_alphanumeric) || after.is_some_and(char::is_alphanumeric) {
        "*"
    } else {
        "_"
    }
}

/// The strong marker for the strong at `position`:
/// `**` (upstream's style) unless followed by `/`, which would close the comment
/// (a strong before `/` can only be written `__`).
pub(super) fn strong_marker(position: Option<&Position>, source: &str) -> &'static str {
    if position.is_some_and(|p| source[p.end.offset..].starts_with('/')) { "__" } else { "**" }
}

fn collect_inline_impl(node: &Node, source: &str, out: &mut String, inside_link: bool) {
    match node {
        Node::Text(text) => push_raw_text(text, source, out),
        Node::Emphasis(emp) => {
            let marker = emphasis_marker(emp.position.as_ref(), source);
            out.push_str(marker);
            for child in &emp.children {
                collect_inline_impl(child, source, out, inside_link);
            }
            out.push_str(marker);
        }
        Node::Strong(strong) => {
            let marker = strong_marker(strong.position.as_ref(), source);
            out.push_str(marker);
            for child in &strong.children {
                collect_inline_impl(child, source, out, inside_link);
            }
            out.push_str(marker);
        }
        Node::InlineCode(code) => {
            let value = &code.value;
            let delimiter_len = min_not_present_backtick_run(value);
            let needs_padding = value.starts_with('`')
                || value.ends_with('`')
                || (value.starts_with([' ', '\n'])
                    && value.ends_with([' ', '\n'])
                    && value.bytes().any(|b| b != b' ' && b != b'\n'));
            for _ in 0..delimiter_len {
                out.push('`');
            }
            if needs_padding {
                out.push(' ');
            }
            out.push_str(value);
            if needs_padding {
                out.push(' ');
            }
            for _ in 0..delimiter_len {
                out.push('`');
            }
        }
        Node::Link(link) if inside_link => {
            // Nested link (e.g. GFM autolink inside explicit link text) —
            // just emit the text content to avoid double-encoding.
            for child in &link.children {
                collect_inline_impl(child, source, out, true);
            }
        }
        Node::Link(link) => {
            let link_text = {
                let mut t = String::new();
                for child in &link.children {
                    collect_inline_impl(child, source, &mut t, true);
                }
                t
            };
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
        Node::LinkReference(link_ref) => {
            out.push('[');
            for child in &link_ref.children {
                collect_inline_impl(child, source, out, inside_link);
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
                collect_inline_impl(child, source, out, inside_link);
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
                collect_inline_impl(child, source, out, inside_link);
            }
        }
        Node::Heading(h) => {
            for child in &h.children {
                collect_inline_impl(child, source, out, inside_link);
            }
        }
        Node::TableCell(cell) => {
            for child in &cell.children {
                collect_inline_impl(child, source, out, inside_link);
            }
        }
        _ => {
            // Fallback: use the node's to_string
            out.push_str(&node.to_string());
        }
    }
}

/// Returns the smallest `n >= 1` such that a maximal run of exactly `n`
/// backticks does not appear in `text`. Matches Prettier's
/// `getMinNotPresentContinuousCount`. Used to pick an inline code span
/// delimiter that won't collide with backticks in the content.
fn min_not_present_backtick_run(text: &str) -> usize {
    let mut present: Vec<bool> = Vec::new();
    let mut current = 0usize;
    for byte in text.bytes().chain(std::iter::once(b' ')) {
        if byte == b'`' {
            current += 1;
        } else if current > 0 {
            if present.len() <= current {
                present.resize(current + 1, false);
            }
            present[current] = true;
            current = 0;
        }
    }
    for (i, is_present) in present.iter().enumerate().skip(1) {
        if !is_present {
            return i;
        }
    }
    present.len().max(1)
}
