use markdown::mdast::Node;

/// Collect inline content from a node into a single text string.
/// This handles emphasis, strong, code, links, etc.
/// Note: the returned text may contain placeholder tokens which must be
/// restored before being used in output.
pub(super) fn collect_inline_text(node: &Node) -> String {
    let mut result = String::new();
    collect_inline_recursive(node, &mut result);
    result
}

/// Collect inline text from a slice of child nodes directly, avoiding
/// the need to clone a parent node just to iterate its children.
pub(super) fn collect_inline_text_from_children(children: &[Node]) -> String {
    let mut result = String::new();
    for child in children {
        collect_inline_recursive(child, &mut result);
    }
    result
}

pub(super) fn collect_inline_recursive(node: &Node, out: &mut String) {
    collect_inline_impl(node, out, false);
}

fn collect_inline_impl(node: &Node, out: &mut String, inside_link: bool) {
    match node {
        Node::Text(text) => {
            out.push_str(&text.value);
        }
        Node::Emphasis(emp) => {
            out.push('_');
            for child in &emp.children {
                collect_inline_impl(child, out, inside_link);
            }
            out.push('_');
        }
        Node::Strong(strong) => {
            out.push_str("**");
            for child in &strong.children {
                collect_inline_impl(child, out, inside_link);
            }
            out.push_str("**");
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
                collect_inline_impl(child, out, true);
            }
        }
        Node::Link(link) => {
            let link_text = {
                let mut t = String::new();
                for child in &link.children {
                    collect_inline_impl(child, &mut t, true);
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
                collect_inline_impl(child, out, inside_link);
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
                collect_inline_impl(child, out, inside_link);
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
                collect_inline_impl(child, out, inside_link);
            }
        }
        Node::Heading(h) => {
            for child in &h.children {
                collect_inline_impl(child, out, inside_link);
            }
        }
        Node::TableCell(cell) => {
            for child in &cell.children {
                collect_inline_impl(child, out, inside_link);
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
