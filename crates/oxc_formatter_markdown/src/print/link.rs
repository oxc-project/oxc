//! Links, images and definitions:
//! destinations and titles are re-derived from their cooked values,
//! reference forms other than `[text][label]` are re-emitted from the source.

use oxc_allocator::StringBuilder;
use oxc_formatter_core::{
    Buffer,
    builders::{group, indent, soft_line_break_or_space, text, token},
    write,
};
use oxc_markdown_parser::{
    Constructs, Segment,
    ast::{Definition, Image, Inline, Link, LinkKind, ReferenceKind},
    decode, lexical,
};

use crate::{context::MarkdownFormatContext, options::ProseWrap, print::Mark};

use super::{
    MarkdownFormatter, format_with,
    inline::{InlineParent, collect_inlines},
    join_pieces,
    parts::{Atom, Parts},
    text as words, with_depth,
};

pub fn collect_link<'a>(
    link: &'a Link<'a>,
    parts: &mut Parts<'a>,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    parts.push_str("[");
    match &link.kind {
        LinkKind::Inline { .. } | LinkKind::Reference { kind: ReferenceKind::Full, .. } => {
            // The text's delimiter runs pair on their own (bracket level).
            // A full reference's text wraps like any text: its `][label]` tail is short.
            // An inline link's does not: its `](url)` tail is unbreakable and usually what overflows,
            // so a break inside the text would only split it without fitting the line.
            let inner = InlineParent::default();
            let text_start = parts.len();
            if matches!(link.kind, LinkKind::Inline { .. }) {
                with_depth(f, MarkdownFormatContext::no_wrap_depth, |f| {
                    collect_inlines(&link.children, inner, parts, f);
                });
            } else {
                collect_inlines(&link.children, inner, parts, f);
            }
            let text_end = parts.len();
            push_target(&link.kind, parts, f);
            parts.mark(Mark::Group { start: text_start, end: text_end });
            parts.mark(Mark::Opaque { start: text_end, end: parts.len() });
        }
        LinkKind::Reference { kind, .. } => {
            // Kept as written, re-split into words for wrapping
            collect_raw_children(&link.children, parts, f);
            parts.push_str("]");
            if *kind == ReferenceKind::Collapsed {
                parts.push_str("[]");
            }
        }
    }
}

pub fn collect_image<'a>(
    image: &'a Image<'a>,
    parts: &mut Parts<'a>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    match &image.kind {
        LinkKind::Inline { .. } | LinkKind::Reference { kind: ReferenceKind::Full, .. } => {
            parts.push_str("![");
            parts.push_lines(join_pieces(&image.alt, f), Atom::VerbatimLine);
            push_target(&image.kind, parts, f);
        }
        LinkKind::Reference { kind, label } => {
            parts.push_str("!");
            parts.push_str(print_label(label, f));
            if *kind == ReferenceKind::Collapsed {
                parts.push_str("[]");
            }
        }
    }
}

/// What follows the text of an inline (`](url "title")`) or full-reference (`][label]`) link.
fn push_target<'a>(kind: &LinkKind<'a>, parts: &mut Parts<'a>, f: &MarkdownFormatter<'_, 'a>) {
    match kind {
        LinkKind::Inline { destination, title } => {
            let source = f.context().source_text().as_str();
            let url = decode::destination(source, destination);
            parts.push_str("](");
            parts.push_str(if url.is_empty() { "<>" } else { print_url(&url, false, f) });
            if let Some(title) = title {
                let cooked = decode::title(source, title);
                if !cooked.is_empty() {
                    parts.push_str(" ");
                    parts.push_lines(print_title(&cooked, f), Atom::VerbatimLine);
                }
            }
            parts.push_str(")");
        }
        LinkKind::Reference { label, .. } => {
            parts.push_str("]");
            parts.push_str(print_label(label, f));
        }
    }
}

/// Each child as its source text, split into words for wrapping only.
fn collect_raw_children<'a>(
    children: &'a [Inline<'a>],
    parts: &mut Parts<'a>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    let cx = words::TextContext { is_link: true, ..words::TextContext::default() };
    for child in children {
        match child {
            Inline::SoftBreak(_) => {
                words::push_whitespace(true, &cx, parts, f);
            }
            _ => words::push_text(f.context().slice(child.span()), cx, parts, f),
        }
    }
}

/// `[label]: url "title"`, breaking after the colon under `proseWrap: always`.
pub fn write_definition<'a>(definition: &'a Definition<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    let source = f.context().source_text().as_str();
    let label = print_label(&definition.label, f);
    let url = decode::destination(source, &definition.destination);
    let url: &'a str = if url.is_empty() { "<>" } else { print_url(&url, true, f) };
    // An empty title (`""`) is dropped
    let title: Option<&'a str> = definition
        .title
        .as_ref()
        .map(|title| decode::title(source, title))
        .filter(|cooked| !cooked.is_empty())
        .map(|cooked| print_title(&cooked, f));
    // A destination wrapped to its own line must not open a block there (`:::x`, `<div>`)
    let always = f.options().prose_wrap == ProseWrap::Always
        && lexical::line_start(&Constructs::markdown(), url, false).is_none();
    let line_or_space = format_with(move |f| {
        if always {
            write!(f, soft_line_break_or_space());
        } else {
            write!(f, token(" "));
        }
    });
    write!(
        f,
        group(&format_with(|f| {
            write!(f, [text(label), token(":")]);
            write!(
                f,
                indent(&format_with(|f| {
                    write!(f, [line_or_space, text(url)]);
                    if let Some(title) = title {
                        // A multi-line title prints as is; its newlines do not break the group
                        write!(f, [line_or_space, text(title).without_expand_parent()]);
                    }
                }))
            );
        }))
    );
}

/// `[label]`: the cooked label with whitespace collapsed and `\`,
/// `[`, `]` escaped (escaping the raw text would double its escapes on every run).
fn print_label<'a>(pieces: &[Segment], f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    let raw = join_pieces(pieces, f);
    let joined = decode::decode(raw, true);
    // Decoding frees at least the byte each re-added escape costs, so the raw length bounds the output
    let mut out = StringBuilder::with_capacity_in(raw.len() + 2, f.allocator());
    out.push('[');
    let mut in_ws = false;
    for c in joined.chars() {
        if super::is_split_whitespace(c) {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
            continue;
        }
        in_ws = false;
        if matches!(c, '\\' | '[' | ']') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push(']');
    out.into_str()
}

/// The destination as Prettier `main` prints it (prettier/prettier#19482, #19849, #19891):
///
/// - `\` followed by ASCII punctuation, or last, is doubled (it would start an escape)
/// - `&` that would start a character reference becomes `\&`
/// - the whole thing is wrapped in `<>`, inner `<` / `>` escaped, when it holds whitespace,
///   a control character, a leading `<`, or parentheses;
///   definitions tolerate balanced parentheses (CommonMark: nested up to three deep)
fn print_url<'a>(
    url: &str,
    unwrap_balanced_parens: bool,
    f: &MarkdownFormatter<'_, 'a>,
) -> &'a str {
    let bytes = url.as_bytes();
    let needs_brackets = bytes.iter().any(|&b| b.is_ascii_control() || b == b' ')
        || url.starts_with('<')
        || if unwrap_balanced_parens {
            !has_balanced_parens(url)
        } else {
            url.contains(['(', ')'])
        };
    let mut out = StringBuilder::with_capacity_in(url.len() + 2, f.allocator());
    if needs_brackets {
        out.push('<');
    }
    for (i, c) in url.char_indices() {
        match c {
            '\\' if bytes.get(i + 1).is_none_or(u8::is_ascii_punctuation) => out.push_str("\\\\"),
            '&' if decode::entity(url, i).is_some() => out.push_str("\\&"),
            '<' | '>' if needs_brackets => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    if needs_brackets {
        out.push('>');
    }
    out.into_str()
}

/// Balanced parentheses nested at most three deep.
fn has_balanced_parens(url: &str) -> bool {
    let mut depth = 0i32;
    for c in url.chars() {
        match c {
            '(' => {
                depth += 1;
                if depth > 3 {
                    return false;
                }
            }
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

/// The title in the preferred quote (see `MarkdownFormatOptions::preferred_quote`),
/// or `(...)` when both quotes appear and no parenthesis does (prettier/prettier#19890).
/// Backslashes, the chosen quote and character-reference `&`s are escaped.
fn print_title<'a>(title: &str, f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    let parenthesized = title.contains('"') && title.contains('\'') && !title.contains(['(', ')']);
    let (open, close, quote) = if parenthesized {
        ('(', ')', None)
    } else {
        let quote = f.options().preferred_quote(title) as char;
        (quote, quote, Some(quote))
    };

    let mut out = StringBuilder::with_capacity_in(title.len() + 2, f.allocator());
    out.push(open);
    for (i, c) in title.char_indices() {
        match c {
            '\\' => out.push_str("\\\\"),
            '&' if decode::entity(title, i).is_some() => out.push_str("\\&"),
            _ if Some(c) == quote => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out.push(close);
    out.into_str()
}
