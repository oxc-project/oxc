use std::borrow::Cow;

use markdown::{Constructs, ParseOptions, mdast::Node, to_mdast};

use oxc_allocator::Allocator;

use crate::FormatOptions;

use super::embedded::{format_embedded_js, is_js_ts_lang};
use super::line_buffer::LineBuffer;
use super::wrap::{
    format_table_block, wrap_paragraph, wrap_plain_paragraphs, wrap_plain_paragraphs_balance,
};

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
            // Strikethrough, images, backslash escapes, HTML tags
            b'~' | b'\\' | b'<' => return true,
            // Emphasis/strong (*) and underscore (_): only trigger when they
            // could be markdown emphasis (adjacent to non-space), not when
            // used as arithmetic (`2 * 3`) or separators.
            // Also trigger for `* ` at line starts which could be list markers.
            b'*' | b'_' => {
                let next = if i + 1 < len { bytes[i + 1] } else { b' ' };
                let prev = if i > 0 { bytes[i - 1] } else { b' ' };
                // Emphasis: `*word` or `word*` (adjacent to non-space on at
                // least one side). `2 * 3` (spaces on both sides) is arithmetic.
                if !next.is_ascii_whitespace() || !prev.is_ascii_whitespace() {
                    return true;
                }
                // `* ` at line start: could be an unordered list marker
                if bytes[i] == b'*' && next == b' ' && (i == 0 || prev == b'\n') {
                    return true;
                }
            }
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
                // Check if this line starts a new block context (after blank line
                // or at text start). Lines that are paragraph continuations
                // (preceded by non-empty line) should not trigger block detection
                // for ambiguous markers like `-`, `+`, digits, `|`.
                let is_block_start = i == 0
                    || (i >= 2 && bytes[i - 1] == b'\n' && bytes[i - 2] == b'\n')
                    || (i >= 3
                        && bytes[i - 1] == b'\n'
                        && bytes[i - 2] == b' '
                        && bytes[i - 3] == b'\n');

                // Count leading spaces
                let mut spaces = 0;
                while i + spaces < len && bytes[i + spaces] == b' ' {
                    spaces += 1;
                }
                // 4+ leading spaces = indented code block (only at block start)
                if spaces >= 4 && is_block_start {
                    return true;
                }
                // Check trigger character after whitespace
                if i + spaces < len {
                    match bytes[i + spaces] {
                        // Headings and blockquotes are unambiguous — always trigger
                        b'#' | b'>' => return true,
                        // Digits: ordered lists (1. foo) or legacy markers (1- foo)
                        b'0'..=b'9' => {
                            // Only trigger if digits are followed by `. `, `) `, or `- `
                            // to avoid false positives from prose like "...and\n1. They"
                            let mut j = i + spaces;
                            while j < len && bytes[j].is_ascii_digit() {
                                j += 1;
                            }
                            if j < len && j + 1 < len && bytes[j + 1] == b' ' {
                                match bytes[j] {
                                    b'.' | b')' if is_block_start => return true,
                                    b'-' => return true, // legacy marker always
                                    _ => {}
                                }
                            }
                        }
                        b'|' => {
                            // Table detection: require pipe at start AND end of line
                            // (i.e., `| cell | cell |` pattern). Bare `|word|` in
                            // prose should not trigger.
                            let line_start = i + spaces;
                            if bytes[line_start] == b'|' {
                                // Find end of line
                                let mut line_end = line_start + 1;
                                while line_end < len && bytes[line_end] != b'\n' {
                                    line_end += 1;
                                }
                                // Check if line ends with `|` (after trimming spaces)
                                let mut end = line_end;
                                while end > line_start + 1 && bytes[end - 1].is_ascii_whitespace() {
                                    end -= 1;
                                }
                                if end > line_start + 1 && bytes[end - 1] == b'|' {
                                    return true;
                                }
                            }
                        }
                        // Unordered list markers: only at block start to avoid
                        // false positives from wrapped text like "min\n+ spacing"
                        b'-' | b'+' | b'*'
                            if is_block_start
                                && i + spaces + 1 < len
                                && bytes[i + spaces + 1] == b' ' =>
                        {
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
    tag_string_length: usize,
    capitalize: bool,
    format_options: Option<&FormatOptions>,
    allocator: Option<&Allocator>,
) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    let jsdoc_opts = format_options.and_then(|opts| opts.jsdoc.as_ref());
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
            let is_last_in_para =
                iter.peek().is_none_or(|next| next.is_empty());
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
    let text = escape_false_list_markers(&text);

    // Protect JSDoc inline tags from markdown parsing (GFM autolink would mangle URLs)
    let (protected, placeholders) = protect_jsdoc_links(&text);

    // Parse into mdast. Keep GFM constructs that affect inline parsing, but let
    // pipe-prefixed table-like blocks be handled by the serializer using the raw
    // paragraph text instead of the markdown crate's table node.
    let parse_opts = ParseOptions {
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
        tag_string_length,
        capitalize,
        description_with_dot,
        prefer_code_fences,
        line_wrapping_style,
        source: &protected,
        format_options,
        allocator,
    };
    serialize_children(&root, 0, opts.tag_string_length, &opts, &mut lines);

    let output = lines.into_string();
    restore_in_string(&output, &placeholders).into_owned()
}

struct SerializeOptions<'a> {
    max_width: usize,
    tag_string_length: usize,
    capitalize: bool,
    description_with_dot: bool,
    prefer_code_fences: bool,
    line_wrapping_style: crate::LineWrappingStyle,
    source: &'a str,
    format_options: Option<&'a FormatOptions>,
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
                // Only treat as a list marker if dash is followed by whitespace or pipe
                // (matching upstream regex `^(\d+)[-][\s|]+`)
                let after_dash = trimmed.as_bytes().get(dash_pos + 1).copied();
                if matches!(after_dash, Some(b' ' | b'\t' | b'|')) {
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
        }

        result.push_str(line);
    }

    if changed { Cow::Owned(result) } else { Cow::Borrowed(text) }
}

/// Convert `* ` list markers at the start of lines to `- ` to prevent the markdown
/// parser from treating them as emphasis markers. In CommonMark, `* text` after a
/// paragraph is emphasis (italic), not a list item. Converting to `- ` makes the
/// markdown parser correctly recognize these as unordered list items.
fn convert_star_list_markers(text: &str) -> Cow<'_, str> {
    if !text.contains("* ") {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut changed = false;

    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let trimmed = line.trim_start();
        if let Some(after_star) = trimmed.strip_prefix("* ") {
            let leading = line.len() - trimmed.len();
            result.push_str(&line[..leading]);
            result.push_str("- ");
            result.push_str(after_star);
            changed = true;
        } else {
            result.push_str(line);
        }
    }

    if changed { Cow::Owned(result) } else { Cow::Borrowed(text) }
}

/// Escape `+ ` at the start of continuation lines (lines preceded by a non-empty line)
/// to prevent the markdown parser from treating them as unordered list markers.
/// This handles cases like `min\n+ spacing` in JSDoc where `+` is an arithmetic operator.
fn escape_false_list_markers(text: &str) -> Cow<'_, str> {
    // Fast path: no `+ ` in text
    if !text.contains("+ ") {
        return Cow::Borrowed(text);
    }

    let lines: Vec<&str> = text.lines().collect();
    let mut result = String::with_capacity(text.len());
    let mut changed = false;

    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }

        let trimmed = line.trim_start();
        // Only escape `+ ` when:
        // 1. Line starts with `+ ` (after indent)
        // 2. Previous line is non-empty (it's a continuation, not a new block)
        // 3. Previous line is NOT a list item (so we don't escape real list sequences)
        let prev_is_list_item = i > 0 && {
            let prev = lines[i - 1].trim_start();
            prev.starts_with("+ ")
                || prev.starts_with("- ")
                || prev.starts_with("* ")
                || prev.strip_prefix(|c: char| c.is_ascii_digit()).is_some_and(|r| {
                    r.trim_start_matches(|c: char| c.is_ascii_digit()).starts_with(". ")
                })
        };
        if trimmed.starts_with("+ ")
            && i > 0
            && !lines[i - 1].trim().is_empty()
            && !prev_is_list_item
        {
            let leading = line.len() - trimmed.len();
            result.push_str(&line[..leading]);
            result.push_str("\\+ ");
            result.push_str(&trimmed[2..]);
            changed = true;
        } else {
            result.push_str(line);
        }
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
            // so tokenize_words treats it atomically.
            // Pad to match original token length so wrapping width calculations
            // are correct (otherwise shorter placeholders cause lines to exceed
            // the wrap width after restoration).
            let mut itoa_buf = itoa::Buffer::new();
            let idx_str = itoa_buf.format(idx);
            let placeholder_len = PLACEHOLDER_PREFIX.len() + idx_str.len();
            result.push_str(PLACEHOLDER_PREFIX);
            result.push_str(idx_str);
            // Pad with \x01 to match original width
            for _ in placeholder_len..token.len() {
                result.push('\x01');
            }
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
        if i + prefix_len <= len
            && s.is_char_boundary(i + prefix_len)
            && &s[i..i + prefix_len] == prefix
        {
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
                // Skip \x01 padding characters after the index
                let mut pad_end = digit_end;
                while pad_end < len && bytes[pad_end] == 0x01 {
                    pad_end += 1;
                }
                i = pad_end;
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
///
/// Consecutive Paragraph and inline-like Html nodes are merged into a single
/// paragraph so that HTML tag references like `<option>` or `<div>` that the
/// markdown parser extracted as separate Html block nodes don't create spurious
/// blank lines in the middle of what should be a single paragraph.
fn serialize_children(
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
                let ind = super::wrap::indent_str(indent);
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
                            let dotted = super::normalize::append_trailing_dot(line);
                            let s = lines.begin_line();
                            s.push_str(&ind);
                            s.push_str(&dotted);
                        } else {
                            let s = lines.begin_line();
                            s.push_str(&ind);
                            s.push_str(line);
                        }
                    } else if opts.capitalize && li == 0 {
                        let cap = super::normalize::capitalize_first(line);
                        if opts.description_with_dot && is_last {
                            lines.push(super::normalize::append_trailing_dot(&cap));
                        } else {
                            lines.push(cap);
                        }
                    } else if opts.description_with_dot && is_last {
                        lines.push(super::normalize::append_trailing_dot(line));
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
/// tag names (e.g., "renders a <div> element") rather than actual HTML blocks.
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
        let indent_str = super::wrap::indent_str(indent);
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
                    let text = super::normalize::capitalize_first(text);
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
    let ind = super::wrap::indent_str(indent);

    // Balance mode: try to preserve original line breaks if all lines fit
    if matches!(opts.line_wrapping_style, crate::LineWrappingStyle::Balance)
        && let Some(position) = para.position.as_ref()
    {
        let raw = &opts.source[position.start.offset..position.end.offset];
        let original_lines: Vec<&str> =
            raw.lines().map(str::trim).filter(|l| !l.is_empty()).collect();

        if original_lines.len() > 1
            && original_lines.iter().all(|l| super::wrap::str_width(l) <= effective_width)
        {
            // Preserve original line breaks
            let total = original_lines.len();
            for (i, orig_line) in original_lines.iter().enumerate() {
                let is_first = i == 0;
                let is_last = i == total - 1;

                let line: String = {
                    let mut s = (*orig_line).to_owned();
                    if is_first && opts.capitalize {
                        s = super::normalize::capitalize_first(&s).into_owned();
                    }
                    if is_last && opts.description_with_dot {
                        s = super::normalize::append_trailing_dot(&s).into_owned();
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
                let dotted = super::normalize::append_trailing_dot(line);
                let s = lines.begin_line();
                s.push_str(&ind);
                s.push_str(&dotted);
            } else {
                let s = lines.begin_line();
                s.push_str(&ind);
                s.push_str(line);
            }
        } else if opts.capitalize && i == 0 {
            let cap = super::normalize::capitalize_first(line);
            if opts.description_with_dot && is_last {
                lines.push(super::normalize::append_trailing_dot(&cap));
            } else {
                lines.push(cap);
            }
        } else if opts.description_with_dot && is_last {
            lines.push(super::normalize::append_trailing_dot(line));
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
        // For fenced code with an explicit non-JS/TS lang, preserve verbatim
        if let Some(l) = lang
            && !is_js_ts_lang(l)
        {
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
            out.push('`');
            out.push_str(&code.value);
            out.push('`');
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
