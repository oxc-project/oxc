//! Text: filled words, or lines as written where whitespace is content.

use oxc_formatter_core::{
    builders::{exact_line_breaks, soft_line_break_or_space, text},
    write,
};
use oxc_htmlish_parser::ast::NodeId;

use super::{
    HtmlishFormatter, format_with,
    tag::{closing_tag_suffix, opening_tag_prefix},
};
use crate::tree::Tree;

pub fn write_text<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    let parent = &t[t.parent(id)];
    let value = t.text(id);
    let prefix = opening_tag_prefix(t, id);
    let suffix = closing_tag_suffix(t, id);

    if parent.pre_like {
        write!(f, [prefix, text(value), suffix]);
        return;
    }
    if parent.script_like {
        write!(f, prefix);
        write_dedented_lines(value, f);
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

/// Prettier's `dedentString`: the lines with their common indentation removed.
fn dedent(value: &str) -> impl Iterator<Item = &str> {
    let indentation = |line: &str| line.len() - line.trim_ascii_start().len();
    let min = value
        .split('\n')
        .filter(|line| !line.trim_ascii_start().is_empty())
        .map(indentation)
        .min()
        .unwrap_or(0);
    value.split('\n').map(move |line| line.get(min..).unwrap_or(""))
}
