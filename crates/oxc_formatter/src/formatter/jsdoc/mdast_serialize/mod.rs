mod collect;
mod detect;
mod nodes;
mod preprocess;

use std::borrow::Cow;

use markdown::{Constructs, ParseOptions, mdast::Node, to_mdast};

use oxc_formatter_core::FormatSession;

use crate::JsFormatOptions;

use super::line_buffer::LineBuffer;
use super::wrap::{wrap_plain_paragraphs, wrap_plain_paragraphs_balance};

use collect::{emphasis_marker, strong_marker};
use detect::needs_mdast_parsing;
use nodes::serialize_children;
use preprocess::{
    convert_star_list_markers, normalize_legacy_ordered_list_markers, protect_jsdoc_links,
    restore_in_string,
};

/// Format a markdown description using mdast parsing.
///
/// Parses the text into a markdown AST, then serializes it back to formatted
/// text with proper indentation, wrapping, and emphasis normalization.
/// This replaces the manual normalize+wrap pipeline with an approach matching
/// the upstream prettier-plugin-jsdoc's use of `fromMarkdown` + `stringify`.
pub fn format_description_mdast(
    text: &str,
    max_width: usize,
    tag_string_length: usize,
    capitalize: bool,
    format_options: &JsFormatOptions,
    session: &FormatSession<'_>,
) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    let jsdoc_opts = format_options.jsdoc.as_ref();
    let description_with_dot = jsdoc_opts.is_some_and(|o| o.description_with_dot);
    let prefer_code_fences = jsdoc_opts.is_some_and(|o| o.prefer_code_fences);
    let line_wrapping_style =
        jsdoc_opts.map_or(crate::LineWrappingStyle::default(), |o| o.line_wrapping_style);

    // Fast path: if text has no markdown constructs requiring AST parsing,
    // use lightweight wrap_plain_paragraphs() directly.
    // Skip fast path when tag_string_length > 0 (first-line offset needs mdast threading).
    if tag_string_length == 0 && !needs_mdast_parsing(text) {
        // Balance mode: try to preserve original line breaks per paragraph
        let result = if matches!(line_wrapping_style, crate::LineWrappingStyle::Balance) {
            wrap_plain_paragraphs_balance(text, max_width)
        } else {
            wrap_plain_paragraphs(text, max_width)
        };
        if !capitalize && !description_with_dot {
            return result;
        }
        // Capitalize the first word of each paragraph (after blank lines),
        // matching the mdast path's per-paragraph capitalization.
        // Also apply trailing dot if enabled.
        let mut out = String::with_capacity(result.len() + 1);
        let mut iter = result.split('\n').peekable();
        let mut at_paragraph_start = true;
        let mut first = true;
        while let Some(line) = iter.next() {
            if !first {
                out.push('\n');
            }
            first = false;
            // A line is "last in paragraph" if it's the final line overall or
            // the next line is empty (paragraph boundary).
            let is_last_in_para = iter.peek().is_none_or(|next| next.is_empty());
            if line.is_empty() {
                at_paragraph_start = true;
            } else if at_paragraph_start {
                let line = if capitalize {
                    super::normalize::capitalize_first(line)
                } else {
                    Cow::Borrowed(line)
                };
                if description_with_dot && is_last_in_para {
                    out.push_str(&super::normalize::append_trailing_dot(&line));
                } else {
                    out.push_str(&line);
                }
                at_paragraph_start = false;
                continue;
            } else if description_with_dot && is_last_in_para {
                out.push_str(&super::normalize::append_trailing_dot(line));
                continue;
            }
            out.push_str(line);
        }
        return out;
    }

    let text = normalize_legacy_ordered_list_markers(text);
    let text = convert_star_list_markers(&text);

    // Protect JSDoc inline tags from markdown parsing (GFM autolink would mangle URLs)
    let (protected, placeholders) = protect_jsdoc_links(&text);

    let Ok(root) = to_mdast(&protected, &parse_options()) else {
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
        tag_string_length,
        capitalize,
        description_with_dot,
        prefer_code_fences,
        line_wrapping_style,
        source: &protected,
        format_options,
        session,
    };
    serialize_children(&root, 0, opts.tag_string_length, &opts, &mut lines);

    let output = lines.into_string();
    restore_in_string(&output, &placeholders).into_owned()
}

/// Keep GFM constructs that affect inline parsing, but let
/// pipe-prefixed table-like blocks be handled by the serializer using the raw
/// paragraph text instead of the markdown crate's table node.
fn parse_options() -> ParseOptions {
    ParseOptions {
        constructs: Constructs {
            gfm_autolink_literal: false,
            gfm_footnote_definition: true,
            gfm_label_start_footnote: true,
            gfm_strikethrough: true,
            gfm_table: false,
            gfm_task_list_item: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    }
}

/// Normalize the emphasis markers of `text` in place, as the serializer writes them
/// (see `emphasis_marker` / `strong_marker`), leaving everything else as written.
/// For descriptions written out without going through `format_description_mdast`.
pub fn normalize_emphasis_markers(text: &str) -> Cow<'_, str> {
    // Only `*x*` and `__x__` can change
    if !text.contains("__") && !text.contains('*') {
        return Cow::Borrowed(text);
    }
    let (protected, placeholders) = protect_jsdoc_links(text);
    let Ok(root) = to_mdast(&protected, &parse_options()) else {
        return Cow::Borrowed(text);
    };
    // Markers are replaced by ASCII ones of the same length, so the offsets stay valid
    let mut normalized: Option<String> = None;
    let mut stack = vec![&root];
    while let Some(node) = stack.pop() {
        stack.extend(node.children().into_iter().flatten());
        let marker = match node {
            Node::Emphasis(_) => emphasis_marker(node.position(), &protected),
            Node::Strong(_) => strong_marker(node.position(), &protected),
            _ => continue,
        };
        let Some(position) = node.position() else { continue };
        let open = position.start.offset..position.start.offset + marker.len();
        let close = position.end.offset - marker.len()..position.end.offset;
        if protected[open.clone()] == *marker && protected[close.clone()] == *marker {
            continue;
        }
        let out = normalized.get_or_insert_with(|| protected.to_string());
        out.replace_range(open, marker);
        out.replace_range(close, marker);
    }
    let Some(normalized) = normalized else {
        return Cow::Borrowed(text);
    };
    match restore_in_string(&normalized, &placeholders) {
        Cow::Borrowed(_) => Cow::Owned(normalized),
        Cow::Owned(restored) => Cow::Owned(restored),
    }
}

pub(super) struct SerializeOptions<'a> {
    pub(super) max_width: usize,
    pub(super) tag_string_length: usize,
    pub(super) capitalize: bool,
    pub(super) description_with_dot: bool,
    pub(super) prefer_code_fences: bool,
    pub(super) line_wrapping_style: crate::LineWrappingStyle,
    pub(super) source: &'a str,
    pub(super) format_options: &'a JsFormatOptions,
    pub(super) session: &'a FormatSession<'a>,
}
