//! Text: filled words, or lines as written where whitespace is content.

use oxc_allocator::StringBuilder;
use oxc_formatter_core::{
    FormatOptions,
    builders::{align, dedent_to_root, exact_line_breaks, soft_line_break_or_space, text},
    text_width, write,
};
use oxc_htmlish_parser::ast::NodeId;

use super::{
    HtmlishFormatter,
    embed::write_embedded_content,
    format_with,
    tag::{closing_tag_suffix, opening_tag_prefix},
};
use crate::tree::Tree;

pub fn write_text<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    let parent = &t[t.parent(id)];
    let value = t.text(id);
    let prefix = opening_tag_prefix(t, id);
    let suffix = closing_tag_suffix(t, id);

    if parent.pre_like {
        // Opening the content, it starts on its own line from the root (`line_before` in `element.rs`)
        let starts_line = parent.first_child == Some(id)
            && !(t[id].has_leading_spaces && t[id].leading_sensitive);
        // A JS expression substituted into it indents from its line (`apps/oxfmt/DIVERGENCES.md#pre-like-template-expression`)
        write!(f, prefix);
        write_pre_like_text(value, starts_line, f);
        write!(f, suffix);
        return;
    }
    if parent.script_like {
        write!(f, prefix);
        if !write_embedded_content(t, t.parent(id), value, f) {
            write_dedented_lines(value, f);
        }
        write!(f, suffix);
        return;
    }

    let mut words = words(value, t[id].has_cdata).peekable();
    let mut fill = f.fill();
    let mut first = true;
    while let Some(word) = words.next() {
        let last = words.peek().is_none();
        fill.entry(
            &soft_line_break_or_space(),
            &format_with(|f| {
                if first {
                    write!(f, prefix);
                }
                write!(f, text(word));
                if last {
                    write!(f, suffix);
                }
            }),
        );
        first = false;
    }
    fill.finish();
}

/// Writes `value`'s lines, trimmed and dedented (Prettier's `htmlTrimPreserveIndentation` + `dedentString`),
/// joined by line breaks: every blank line is kept, and prints without indentation.
pub fn write_dedented_lines<'a>(value: &'a str, f: &mut HtmlishFormatter<'_, 'a>) {
    let mut breaks = 0;
    for line in dedent(trim_preserving_indentation(value)) {
        if line.is_empty() {
            breaks += 1;
            continue;
        }
        if breaks > 0 {
            write!(f, exact_line_breaks(breaks));
        }
        write!(f, text(line));
        breaks = 1;
    }
}

/// Whitespace-separated words; with `cdata`, a CDATA section is part of one word, whatever it holds
/// (DIVERGENCES.md#cdata-as-written).
fn words(value: &str, cdata: bool) -> impl Iterator<Item = &str> {
    let bytes = value.as_bytes();
    let mut pos = 0;
    std::iter::from_fn(move || {
        pos += bytes[pos..].iter().take_while(|b| b.is_ascii_whitespace()).count();
        if pos == bytes.len() {
            return None;
        }
        let start = pos;
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
            pos = if cdata && bytes[pos..].starts_with(b"<![CDATA[") {
                memchr::memmem::find(&bytes[pos..], b"]]>").map_or(bytes.len(), |end| pos + end + 3)
            } else {
                pos + 1
            };
        }
        Some(&value[start..pos])
    })
}

/// Prettier's `htmlTrimPreserveIndentation`: trailing whitespace and leading blank lines go.
fn trim_preserving_indentation(value: &str) -> &str {
    let value = value.trim_ascii_end();
    let mut start = 0;
    while let Some(newline) = value[start..].find('\n') {
        if value[start..start + newline].trim_ascii().is_empty() {
            start += newline + 1;
        } else {
            break;
        }
    }
    &value[start..]
}

/// [`dedent`] as one text; borrowed when no line is indented.
pub fn dedent_string<'a>(value: &'a str, f: &HtmlishFormatter<'_, 'a>) -> &'a str {
    let min = common_indentation(value);
    if min == 0 {
        return value;
    }
    let mut out = StringBuilder::new_in(f.allocator());
    for (i, line) in dedent_by(value, min).enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(line);
    }
    out.into_str()
}

/// Prettier's `dedentString`: the lines with their common indentation removed.
fn dedent(value: &str) -> impl Iterator<Item = &str> {
    dedent_by(value, common_indentation(value))
}

/// The lines of `value` with `min` bytes of indentation removed.
fn dedent_by(value: &str, min: usize) -> impl Iterator<Item = &str> {
    value.split('\n').map(move |line| line.get(min..).unwrap_or(""))
}

/// The indentation every non-blank line of `value` shares.
fn common_indentation(value: &str) -> usize {
    value
        .split('\n')
        .filter(|line| !line.trim_ascii_start().is_empty())
        .map(|line| line.len() - line.trim_ascii_start().len())
        .min()
        .unwrap_or(0)
}

/// `pre`-like text as written.
/// A parent's placeholder for a JS expression on a line that starts at the root (after a line break in the text,
/// or at its start when the text opens the content on its own line) aligns with the whitespace that line starts with,
/// so that the expression's own lines indent from its `${`, as anywhere else.
fn write_pre_like_text<'a>(value: &'a str, starts_line: bool, f: &mut HtmlishFormatter<'_, 'a>) {
    let prefix = f.context().embedded_in().and_then(|e| e.placeholder).map(|(prefix, _)| prefix);
    let indent_width = f.options().indent_width();
    let mut written = 0;
    while let Some(start) = prefix.and_then(|p| value[written..].find(p)).map(|i| written + i) {
        let line_end = value[start..].find('\n').map_or(value.len(), |i| start + i);
        let line_start = value[..start].rfind('\n').map(|i| i + 1).or(starts_line.then_some(0));
        if let Some(line_start) = line_start {
            let line = &value[line_start..start];
            let indentation = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
            let width = u8::try_from(text_width(indentation, indent_width)).unwrap_or(u8::MAX);
            write!(
                f,
                [
                    text(&value[written..start]),
                    dedent_to_root(&align(width, &text(&value[start..line_end])))
                ]
            );
        } else {
            write!(f, text(&value[written..line_end]));
        }
        written = line_end;
    }
    write!(f, text(&value[written..]));
}
