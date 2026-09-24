//! Phrasing content: the node printers and the queries about a node's neighbors they need.
//!
//! A paragraph is ONE `fill` ([`Parts`]); `collect_inlines` pushes the nodes into it,
//! `collect_phrasing` wraps that with the pre-passes and the pairing check.

use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_formatter_core::arena_cow_str;
use oxc_markdown_parser::{
    Span,
    ast::{CodeSpan, Emphasis, HardBreakKind, Inline, LinkKind, Strong},
    escapes_next, unicode,
};

use crate::{
    context::{MarkdownFormatContext, Raw},
    options::ProseWrap,
    print::{Mark, pairing},
};

use super::{
    HTML_WHITESPACE, MarkdownFormatter, backticks, escape, is_split_whitespace, join_pieces,
    line_shape::{
        is_line_shape_start, line_from, line_opens_block, line_or_prefix_opens_block, line_shape,
        printed_line_at, printed_line_opens_block, source_line_at,
    },
    link,
    parts::{Atom, Parts, Sep},
    text as words, with_depth,
};

/// Where the inline children hang; a few rules depend on the immediate parent.
#[derive(Clone, Copy, Default)]
pub struct InlineParent {
    /// The children are a paragraph's (its first / last text gets trimmed).
    pub paragraph: bool,
    /// The parent is an `Emphasis` / `Strong`: its marker character as printed (`*` / `_`),
    /// the neighbor of the content's edge words.
    pub delimiter: Option<u8>,
    /// `Some(has_word_neighbor)` when the parent is a `Strong` (nested emphasis reads it).
    pub strong_neighbor: Option<bool>,
    /// The paragraph is the first block of a blockquote
    /// (`[!NOTE]` on its first line is an alert marker).
    pub first_in_container: bool,
    /// The paragraph is the document's first block and its `---` / `+++` first line would be read as
    /// front matter on the next parse (`block::front_matter_risk`), so it is escaped.
    pub first_in_document: bool,
    /// The paragraph follows a task list checkbox on its line: its first line is not a line start.
    pub after_checkbox: bool,
}

/// Paragraph or heading content as one fill.
pub fn write_inlines<'a>(
    children: &'a [Inline<'a>],
    parent: InlineParent,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    collect_phrasing(children, parent, f).write_fill(f);
}

/// One phrasing content (a paragraph, a heading, a table cell) collected.
pub fn collect_phrasing<'a>(
    children: &'a [Inline<'a>],
    parent: InlineParent,
    f: &mut MarkdownFormatter<'_, 'a>,
) -> Parts<'a> {
    let raw = {
        let mut opaque = f.context().opaque_spans().borrow_mut();
        opaque.clear();
        collect_opaque_spans(children, &mut opaque);
        code_span_literal_runs(
            children,
            &opaque,
            &mut f.context().code_span_literal_runs().borrow_mut(),
            f,
        );
        parent.paragraph && wiki_link_risk(children, &opaque, f)
    };
    let collect = |initial: Raw, f: &mut MarkdownFormatter<'_, 'a>| {
        let mut parts = Parts::new(f.allocator());
        f.context().raw_text().set(initial);
        collect_inlines(children, parent, &mut parts, f);
        f.context().raw_text().set(Raw::No);
        parts
    };
    let consistent = |parts: &Parts<'a>, f: &MarkdownFormatter<'_, 'a>| {
        !parts.marks.iter().any(|m| matches!(m, Mark::Delimiter { .. }))
            || pairing::consistent(parts.flat(f.allocator()), &parts.marks)
    };
    let initial = if raw { Raw::Text } else { Raw::No };
    let mut parts = collect(initial, f);
    // Normalized markers must pair as the source's did; otherwise the markers are printed as written,
    // and if joined lines still pair differently, the text as written too
    if !consistent(&parts, f) {
        f.context().literal_markers().set(true);
        parts = collect(initial, f);
        if !consistent(&parts, f) {
            parts = collect(Raw::Text, f);
        }
        f.context().literal_markers().set(false);
    }
    parts
}

/// For every code span, the backtick run lengths (bit `n - 1` for a run of `n`) that occur
/// before it in the content as literal text
/// (outside code spans, HTML, autolinks, math, liquid, whose backticks never delimit):
/// a printed fence of such a length would pair with the run and move the span.
fn code_span_literal_runs<'a>(
    children: &'a [Inline<'a>],
    opaque: &[(Span, bool)],
    out: &mut Vec<(u32, u64)>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    out.clear();
    let (Some(first), Some(last)) = (children.first(), children.last()) else { return };
    let source = f.context().source_text();
    if !source.bytes_contain(first.span().start, last.span().end, b'`') {
        return;
    }
    let mut mask = 0;
    let mut at = first.span().start;
    for &(span, is_code) in opaque {
        if span.start > at {
            mask |= run_mask(source.slice_range(at, span.start).as_bytes(), b'`', true);
        }
        if is_code {
            out.push((span.start, mask));
        }
        at = at.max(span.end);
    }
}

/// Spans whose backticks are not code span delimiters, in source order;
/// `true` marks code spans.
fn collect_opaque_spans<'a>(children: &'a [Inline<'a>], out: &mut Vec<(Span, bool)>) {
    for child in children {
        match child {
            Inline::CodeSpan(c) => out.push((c.span, true)),
            Inline::HtmlInline(_)
            | Inline::Autolink(_)
            | Inline::AutolinkLiteral(_)
            | Inline::MathSpan(_)
            | Inline::Liquid(_)
            | Inline::WikiLink(_) => out.push((child.span(), false)),
            Inline::Emphasis(e) => collect_opaque_spans(&e.children, out),
            Inline::Strong(s) => collect_opaque_spans(&s.children, out),
            Inline::Strikethrough(s) => collect_opaque_spans(&s.children, out),
            // Link text is inline content; a reference label or a title is not
            Inline::Link(l) => {
                collect_opaque_spans(&l.children, out);
                collect_link_kind_spans(&l.kind, out);
            }
            Inline::Image(i) => {
                collect_opaque_spans(&i.children, out);
                collect_link_kind_spans(&i.kind, out);
            }
            _ => {}
        }
    }
}

fn collect_link_kind_spans(kind: &LinkKind<'_>, out: &mut Vec<(Span, bool)>) {
    let pieces = match kind {
        LinkKind::Reference { label, .. } => label,
        LinkKind::Inline { title: Some(title), .. } => title,
        LinkKind::Inline { title: None, .. } => return,
    };
    out.extend(pieces.iter().map(|piece| (piece.span, false)));
}

/// The emphasis marker as printed: `_` unless a word touches it (`1*2*3` is emphasis, `1_2_3` is not),
/// the source marker around an autolink, on a raw line, and when a literal run of the target
/// character before it could pair with it.
fn emphasis_style<'a>(
    e: &'a Emphasis<'a>,
    children: &'a [Inline<'a>],
    i: usize,
    parent: InlineParent,
    f: &MarkdownFormatter<'_, 'a>,
) -> &'static str {
    let source = if e.marker == b'_' { "_" } else { "*" };
    if markers_as_written(f)
        || matches!(e.children.first(), Some(Inline::Autolink(_) | Inline::AutolinkLiteral(_)))
    {
        return source;
    }
    let word_neighbor = has_word_neighbor(children, i, f)
        || parent.strong_neighbor == Some(true)
        || f.context().emphasis_depth().get() > 0;
    if word_neighbor { "*" } else { "_" }
}

/// `**`; `__` stays as written.
fn strong_style<'a>(s: &'a Strong<'a>, f: &MarkdownFormatter<'_, 'a>) -> &'static str {
    if s.marker == b'_' && markers_as_written(f) { "__" } else { "**" }
}

/// On a raw line, or when normalized markers would pair differently (`pairing`).
fn markers_as_written(f: &MarkdownFormatter<'_, '_>) -> bool {
    f.context().raw_text().get() == Raw::Line || f.context().literal_markers().get()
}

/// Under `proseWrap: preserve`, whether any soft break of this paragraph survives printing
/// (one before a block-start-looking word is joined away; one next to a dialect shape line stays).
pub fn keeps_a_line_break<'a>(children: &'a [Inline<'a>], f: &MarkdownFormatter<'_, 'a>) -> bool {
    let last = children.len().wrapping_sub(1);
    children.iter().enumerate().any(|(i, child)| {
        matches!(child, Inline::SoftBreak(_))
            && i != 0
            && i != last
            && (line_shape(children, i + 1, false, false, f)
                || !words::prevents_break(
                    true,
                    next_word_of(children, i, None, f),
                    ProseWrap::Preserve,
                ))
    })
}

/// Prettier's `riskyParagraphPositions`: a `[[` in text, followed by `]]` (text or a wiki link's).
/// Wrapping such a paragraph could merge `[[foo\n[[wiki link]]` into one link,
/// so its text is printed as written.
/// Checked on the source, where a bracket pair split across nodes (`[` + `[link](u)`) still counts;
/// a `[[` inside an opaque span (a wiki link's own, a code span, HTML) does not.
fn wiki_link_risk<'a>(
    children: &'a [Inline<'a>],
    opaque: &[(Span, bool)],
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    let (Some(first), Some(last)) = (children.first(), children.last()) else { return false };
    let start = first.span().start;
    let raw = f.context().source_text().slice_range(start, last.span().end);
    let Some(last_close) = raw.rfind("]]") else { return false };
    raw[..last_close].match_indices("[[").any(|(at, _)| {
        let abs = start + u32::try_from(at).unwrap_or(u32::MAX);
        !opaque.iter().any(|(span, _)| span.contains(abs))
    })
}

pub fn collect_inlines<'a>(
    children: &'a [Inline<'a>],
    parent: InlineParent,
    parts: &mut Parts<'a>,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    let last = children.len().wrapping_sub(1);
    // Raw text is switched per source line:
    // every line when wiki links are at risk, else the lines starting with a dialect shape.
    // Nested content (emphasis, links) inherits the line's setting through the context
    // and switches it at its own line breaks; only a paragraph's first line is a first line.
    let raw_paragraph = f.context().raw_text().get() != Raw::No;
    let raw_for = |shape: bool| {
        if shape {
            Raw::Line
        } else if raw_paragraph {
            Raw::Text
        } else {
            Raw::No
        }
    };
    let raw_line = |j: usize, f: &MarkdownFormatter<'_, 'a>| -> Raw {
        let first_line = parent.paragraph && j == 0;
        raw_for(line_shape(children, j, first_line, first_line && parent.first_in_container, f))
    };
    if parent.paragraph {
        f.context().raw_text().set(raw_line(0, f));
    }
    // The soft break at `j` stays a line break: either side of it is raw
    let break_kept = |j: usize, f: &MarkdownFormatter<'_, 'a>| -> bool {
        f.context().raw_text().get() != Raw::No || raw_line(j + 1, f) != Raw::No
    };
    // Prettier's sentence is a run of texts joined by soft breaks; its CJ spacing style is one statistic.
    let mut sentence_cj_spaces: Option<Option<bool>> = None;
    // The previous sibling was a line break that stays in the output
    let mut last_break_kept = false;
    for (i, child) in children.iter().enumerate() {
        if !matches!(child, Inline::Text(_) | Inline::SoftBreak(_)) {
            sentence_cj_spaces = None;
        }
        let after_kept_break = last_break_kept;
        if !matches!(child, Inline::SoftBreak(_) | Inline::HardBreak(_)) {
            last_break_kept = false;
        }
        // After a kept line break, a line that would open a block at column 0 stays paragraph text
        // behind four spaces (indented code cannot interrupt a paragraph)
        if after_kept_break
            && matches!(
                child,
                Inline::Text(_) | Inline::HtmlInline(_) | Inline::Liquid(_) | Inline::MathSpan(_)
            )
            && printed_line_opens_block(children, i, &break_kept, f)
        {
            parts.push_str("    ");
        }
        match child {
            Inline::Text(t) => {
                let mut raw = f.context().slice(t.span);
                // CommonMark keeps a paragraph's edge `\f`; HTML rendering drops it
                if parent.paragraph {
                    if i == 0 {
                        raw = raw.trim_start_matches(HTML_WHITESPACE);
                    }
                    if i == last {
                        raw = raw.trim_end_matches(HTML_WHITESPACE);
                    }
                }
                if f.context().raw_text().get() != Raw::No {
                    // A line ending with `\` would be a hard break
                    let before_break = matches!(children.get(i + 1), Some(Inline::SoftBreak(_)));
                    let mut lines = raw.split('\n').peekable();
                    while let Some(line) = lines.next() {
                        parts.push_str(line);
                        let more = lines.peek().is_some();
                        if (more || before_break) && escape::ends_with_unescaped_backslash(line) {
                            parts.push_str("\\");
                        }
                        if more {
                            parts.push_sep(Sep::HardLine);
                        }
                    }
                    continue;
                }
                // Edge characters matter only inside emphasis (escaping)
                let (edge_prev, edge_next) = if f.context().delimiter_depth().get() > 0 {
                    (
                        edge_char(children, i, parent.delimiter, true, f),
                        edge_char(children, i, parent.delimiter, false, f),
                    )
                } else {
                    (None, None)
                };
                let cj_spaces = *sentence_cj_spaces
                    .get_or_insert_with(|| sentence_cj_spaces_at(children, i, f));
                // A paragraph's first line that opens a block
                // (the rest of a paragraph a definition was split from) is escaped: `\- x`, `1\. x`
                if i == 0
                    && parent.paragraph
                    && !parent.after_checkbox
                    && (line_opens_block(source_line_at(children, t.span.start, f).1, false)
                        || (parent.first_in_document
                            && (raw.starts_with("---") || raw.starts_with("+++"))))
                {
                    let digits = raw.bytes().take_while(u8::is_ascii_digit).count();
                    if digits > 0 && matches!(raw.as_bytes().get(digits), Some(b'.' | b')')) {
                        parts.push_str(&raw[..digits]);
                        raw = &raw[digits..];
                    }
                    parts.push_str("\\");
                }
                let autolink_stretch = autolink_stretch(children, i, raw);
                let cx = words::TextContext {
                    autolink_stretch,
                    first_of_delimiter: parent.delimiter.filter(|_| i == 0),
                    last_of_delimiter: parent.delimiter.filter(|_| i == last),
                    edge_prev,
                    edge_next,
                    after_soft_break: matches!(
                        children.get(i.wrapping_sub(1)),
                        Some(Inline::SoftBreak(_))
                    ),
                    after_liquid: follows_liquid(children, i),
                    before_soft_break: matches!(children.get(i + 1), Some(Inline::SoftBreak(_))),
                    next_word: next_word_of(children, i, parent.delimiter, f),
                    glued_last_word: glued_last_word(children, i, raw, f),
                    cj_spaces,
                    ..words::TextContext::default()
                };
                words::push_text(raw, cx, parts, f);
            }
            // A soft break at a paragraph edge (a task checkbox followed by a newline) is stripped
            Inline::SoftBreak(_) if parent.paragraph && (i == 0 || i == last) => {}
            Inline::SoftBreak(_) => {
                let next_raw = raw_line(i + 1, f);
                let keep = f.context().raw_text().get() != Raw::No || next_raw != Raw::No;
                f.context().raw_text().set(next_raw);
                last_break_kept = keep;
                if keep {
                    // A separator, not content: the next line is measured on its own
                    parts.push_sep(Sep::HardLine);
                } else {
                    let cj_spaces = *sentence_cj_spaces
                        .get_or_insert_with(|| sentence_cj_spaces_at(children, i, f));
                    let cx = words::TextContext {
                        next_word: next_word_of(children, i, parent.delimiter, f),
                        after_liquid: follows_liquid(children, i),
                        // Only the CJK rules look back (a trailing `\` was escaped by the text);
                        // a word glued to an autolink literal is part of it, the break after it stays a space
                        prev_word: if cj_spaces.is_some() && !glued_to_autolink(children, i, f) {
                            prev_word_of(children, i, f)
                        } else {
                            None
                        },
                        cj_spaces,
                        ..words::TextContext::default()
                    };
                    words::push_whitespace(true, &cx, parts, f);
                }
            }
            Inline::HardBreak(b) => {
                match b.kind {
                    HardBreakKind::Spaces => {
                        parts.push_str("  ");
                        parts.push_atom(Atom::LiteralLine);
                    }
                    HardBreakKind::Backslash => {
                        parts.push_str("\\");
                        parts.push_atom(Atom::HardLine);
                    }
                }
                f.context().raw_text().set(raw_line(i + 1, f));
                last_break_kept = true;
            }
            Inline::Emphasis(e) => {
                let style = emphasis_style(e, children, i, parent, f);
                let open = parts.len();
                parts.push_str(style);
                let inner = InlineParent {
                    delimiter: Some(style.as_bytes()[0]),
                    ..InlineParent::default()
                };
                with_depth(f, MarkdownFormatContext::emphasis_depth, |f| {
                    with_depth(f, MarkdownFormatContext::delimiter_depth, |f| {
                        collect_inlines(&e.children, inner, parts, f);
                    });
                });
                mark_delimiter(open, parts, false);
                parts.push_str(style);
            }
            Inline::Strong(s) => {
                let style = strong_style(s, f);
                let open = parts.len();
                parts.push_str(style);
                let inner = InlineParent {
                    delimiter: Some(style.as_bytes()[0]),
                    strong_neighbor: Some(has_word_neighbor(children, i, f)),
                    ..InlineParent::default()
                };
                with_depth(f, MarkdownFormatContext::delimiter_depth, |f| {
                    collect_inlines(&s.children, inner, parts, f);
                });
                mark_delimiter(open, parts, true);
                parts.push_str(style);
            }
            Inline::Strikethrough(s) => {
                let style = &"~~"[..usize::from(s.tildes)];
                let open = parts.len();
                parts.push_str(style);
                let inner = InlineParent::default();
                collect_inlines(&s.children, inner, parts, f);
                mark_delimiter(open, parts, false);
                parts.push_str(style);
            }
            Inline::Link(l) => {
                link::collect_link(l, parts, f);
                // A title keeps its newlines (a label's are collapsed): as after an opaque node
                if let LinkKind::Inline { title: Some(title), .. } = &l.kind
                    && let (Some(first), Some(last)) = (title.first(), title.last())
                    && let Some(risky) =
                        tail_line_risky(children, first.span.start, last.span.end, None, f)
                {
                    f.context().raw_text().set(raw_for(risky));
                }
            }
            // Opaque to delimiter pairing (an image's alt text too: it is printed as written)
            _ => {
                let start = parts.len();
                let raw = f.context().slice(child.span());
                // A code span joins its lines: no newline in its printed text
                let mut code_span_joined = false;
                // The last line of a multi-line code span as printed (its fence is recomputed)
                let mut code_span_tail = None;
                match child {
                    Inline::CodeSpan(c) => {
                        let printed = print_code_span(c, f);
                        code_span_joined = !printed.contains('\n');
                        code_span_tail = printed.rsplit('\n').next();
                        parts.push_lines(printed, Atom::VerbatimLine);
                    }
                    Inline::Image(img) => link::collect_image(img, parts, f),
                    Inline::HtmlInline(h) => {
                        // Verbatim, continuation indentation included (mdast's `html` value is the source slice)
                        parts.push_lines(join_pieces(&h.pieces, f), Atom::VerbatimLine);
                    }
                    Inline::WikiLink(w) => {
                        let raw = f.context().slice(w.span);
                        let inner = &raw[2..raw.len() - 2];
                        parts.push_str("[[");
                        if f.options().prose_wrap == ProseWrap::Preserve
                            || !inner.contains(['\t', '\n'])
                        {
                            parts.push_str(inner);
                        } else {
                            parts.push_str(collapse_tabs_and_newlines(inner, f));
                        }
                        parts.push_str("]]");
                    }
                    Inline::Liquid(l) => {
                        parts.push_lines(join_pieces(&l.pieces, f), Atom::VerbatimLine);
                    }
                    // Printed as written: remark-math trims math spans, but the match may be accidental
                    Inline::MathSpan(m) => {
                        parts.push_lines(join_pieces(&m.pieces, f), Atom::VerbatimLine);
                    }
                    // Printed as written; single-line by construction.
                    _ => parts.push_str(raw),
                }
                parts.mark(Mark::Opaque { start, end: parts.len() });
                // A newline inside the node starts a source line with the node's tail;
                // what follows on that line may be cut by wrapping (`$$\n$$ text`: `$$` alone opens math),
                // so such a line stays as written.
                if !code_span_joined
                    && let Some(risky) = tail_line_risky(
                        children,
                        child.span().start,
                        child.span().end,
                        code_span_tail,
                        f,
                    )
                {
                    f.context().raw_text().set(raw_for(risky));
                }
            }
        }
    }
}

/// After a newline inside `start..end` (a node printed with its line breaks),
/// whether the source line it starts is a shape or may open a block; `None` without a newline.
/// `tail`: the printed form of that line up to `end`, when it differs from the source.
fn tail_line_risky<'a>(
    children: &[Inline<'_>],
    start: u32,
    end: u32,
    tail: Option<&'a str>,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<bool> {
    let newline = f.context().source_text().slice_range(start, end).rfind('\n')?;
    let line_start = start + u32::try_from(newline).unwrap_or(0) + 1;
    let line = line_from(children, line_start, tail.map(|tail| (tail, end)), f);
    Some(is_line_shape_start(&line.text) || line_or_prefix_opens_block(&line, true, f))
}

/// Records a delimiter node whose opening marker was pushed at `open` and whose closing marker
/// is about to be pushed.
fn mark_delimiter(open: u32, parts: &mut Parts<'_>, strong: bool) {
    parts.mark(Mark::Delimiter { open, close: parts.len(), strong });
}

/// Bit `n - 1` set: a run of exactly `n` `ch`s occurs in `bytes` (runs past 64 count as 64).
/// `escapes`: `bytes` is markdown text, where a backslash-escaped `ch` is not part of a run.
fn run_mask(bytes: &[u8], ch: u8, escapes: bool) -> u64 {
    let mut present: u64 = 0;
    let mut run = 0usize;
    let mut i = 0;
    loop {
        let b = bytes.get(i).copied();
        if b == Some(ch) {
            run += 1;
            i += 1;
            continue;
        }
        if run > 0 {
            present |= 1u64 << (run.min(64) - 1);
            run = 0;
        }
        match b {
            None => return present,
            Some(b'\\') if escapes && escapes_next(bytes, i) => i += 2,
            Some(_) => i += 1,
        }
    }
}

/// The smallest run length of `ch` absent from `text`.
fn min_absent_run(text: &str, ch: u8) -> usize {
    run_mask(text.as_bytes(), ch, false).trailing_ones() as usize + 1
}

/// Runs of tabs / newlines become one space (wiki link contents under wrapping).
fn collapse_tabs_and_newlines<'a>(s: &str, f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    let mut out = oxc_allocator::StringBuilder::with_capacity_in(s.len(), f.allocator());
    let mut in_run = false;
    for c in s.chars() {
        if c == '\t' || c == '\n' {
            if !in_run {
                out.push(' ');
                in_run = true;
            }
        } else {
            out.push(c);
            in_run = false;
        }
    }
    out.into_str()
}

/// The first word the next sibling starts, for the whitespace that ends this one
/// (any node wrapped to a line start may open a block: `<!--`, `<div>`, `$$`, `{% t %}`, `[[toc]]`).
/// `delimiter`: the enclosing emphasis marker, the edge character past the last sibling.
fn next_word_of<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    delimiter: Option<u8>,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<words::NextWord<'a>> {
    let next = children.get(i + 1)?;
    if matches!(next, Inline::SoftBreak(_) | Inline::HardBreak(_)) {
        return None;
    }
    // The word runs across nodes (`[^1]:` is a footnote reference and a text);
    // a code span's fence is recomputed, so the line is taken as it prints
    let line: &'a str = arena_cow_str(&printed_line_at(children, i + 1, f).text, f);
    let word = first_word(line)?;
    // A text's leading run after a soft break is escaped inside emphasis
    // (as `push_text` decides: a space before it, and after it the text's next character or the node edge)
    let in_sentence = matches!(next, Inline::Text(_));
    let escaped = in_sentence
        && f.context().delimiter_depth().get() > 0
        && !f.context().literal_markers().get()
        && escape::leading_run_escaped(
            word,
            Some(' '),
            edge_char(children, i + 1, delimiter, false, f),
        );
    Some(words::NextWord { word, line, escaped, in_sentence })
}

fn first_word(line: &str) -> Option<&str> {
    line.split(is_split_whitespace).find(|w| !w.is_empty())
}

/// The character right before (`before`) or after the text at `i`, as it will be printed:
/// a space for a line break, the neighbor node's edge character, the enclosing emphasis marker;
/// `None` at the edge of a paragraph.
fn edge_char<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    delimiter: Option<u8>,
    before: bool,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<char> {
    let neighbor =
        if before { i.checked_sub(1).map(|j| &children[j]) } else { children.get(i + 1) };
    match neighbor {
        Some(Inline::SoftBreak(_) | Inline::HardBreak(_)) => Some(' '),
        // An autolink's extent depends on what follows it: escaping there changes the URL
        Some(Inline::Autolink(_) | Inline::AutolinkLiteral(_)) => None,
        Some(node) => {
            let raw = f.context().slice(node.span());
            if before { raw.chars().next_back() } else { raw.chars().next() }
        }
        // A paragraph edge is a boundary, whitespace to the flanking rules
        None => delimiter.map_or(Some(' '), |m| Some(char::from(m))),
    }
}

/// The last word of text `i` (its printed `raw`) extended by the nodes glued after it
/// on its source line (a backslash hard break's `\` included).
fn glued_last_word<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    raw: &str,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<&'a str> {
    // Nothing glues after whitespace, a line end, or a two-space hard break
    match children.get(i + 1) {
        None | Some(Inline::SoftBreak(_)) => return None,
        Some(Inline::HardBreak(b)) if b.kind == HardBreakKind::Spaces => return None,
        _ if raw.ends_with(is_split_whitespace) => return None,
        _ => {}
    }
    let last = raw.rsplit(is_split_whitespace).next()?;
    let start = children[i].span().end - u32::try_from(last.len()).unwrap_or(0);
    first_word(source_line_at(children, start, f).1).filter(|word| word.len() != last.len())
}

/// The last word of the text before node `i`.
fn prev_word_of<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<&'a str> {
    let Some(Inline::Text(t)) = children.get(i.checked_sub(1)?) else { return None };
    f.context().slice(t.span).rsplit(is_split_whitespace).find(|w| !w.is_empty())
}

/// Bytes at the start of text `i` (`raw`) that may still extend the autolink literal before it,
/// its trailing punctuation stretch: up to the first whitespace or `<`; 0 after any other node.
fn autolink_stretch(children: &[Inline<'_>], i: usize, raw: &str) -> usize {
    if matches!(children.get(i.wrapping_sub(1)), Some(Inline::AutolinkLiteral(_))) {
        raw.find(|c: char| unicode::is_whitespace(c) || c == '<').unwrap_or(raw.len())
    } else {
        0
    }
}

/// The text before node `i` is one word in an autolink literal's stretch (`http://x.y2` + `.`):
/// what follows the break must not glue to it (`push_text` makes the same call for its first word).
fn glued_to_autolink<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    let Some(Inline::Text(t)) = children.get(i.wrapping_sub(1)) else { return false };
    let raw = f.context().slice(t.span);
    !raw.contains(is_split_whitespace) && autolink_stretch(children, i - 1, raw) > 0
}

fn follows_liquid(children: &[Inline<'_>], i: usize) -> bool {
    matches!(children.get(i.wrapping_sub(1)), Some(Inline::Liquid(_)))
}

/// `usesCJSpaces` of the sentence (run of texts and soft breaks) containing node `i`;
/// `None` without CJK text, which is the common case and skips the scan and the CJK rules.
fn sentence_cj_spaces_at<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<bool> {
    let in_sentence = |c: &Inline<'a>| matches!(c, Inline::Text(_) | Inline::SoftBreak(_));
    let start = (0..i).rev().take_while(|&j| in_sentence(&children[j])).last().unwrap_or(i);
    let end = (i..children.len()).take_while(|&j| in_sentence(&children[j])).last().unwrap_or(i);
    let texts = children[start..=end].iter().filter_map(|c| match c {
        Inline::Text(t) => Some(t),
        _ => None,
    });
    if !texts.clone().any(|t| t.contains_cjk) {
        return None;
    }
    Some(words::uses_cj_spaces(texts.map(|t| f.context().slice(t.span))))
}

/// A word character directly before or after node `i`.
fn has_word_neighbor<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    let is_word_char = |c: char| !unicode::is_whitespace(c) && !unicode::is_punctuation(c);
    let before = i > 0
        && matches!(&children[i - 1], Inline::Text(t)
            if f.context().slice(t.span).chars().next_back().is_some_and(is_word_char));
    let after = matches!(children.get(i + 1), Some(Inline::Text(t))
        if f.context().slice(t.span).chars().next().is_some_and(is_word_char));
    before || after
}

/// Backtick fence of the shortest length the content permits that no literal backtick run of the
/// paragraph shares (a fence equal to a stray run would pair with it and move the span:
/// Prettier's content-only rule, prettier/prettier#6035).
/// A space pads content that starts / ends with a backtick or is surrounded by spaces (CommonMark strips one).
/// Inside a table cell a `|` is `\|` in the source and stays so
/// (the raw slice already carries the escape Prettier re-adds to its decoded value).
pub fn print_code_span<'a>(code: &'a CodeSpan<'a>, f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    let joined = join_pieces(&code.pieces, f);
    // Raw text keeps its line breaks: joining `[[\`a\nb\`]]` onto one line would form a wiki link
    let value: Cow<'_, str> = if f.options().prose_wrap == ProseWrap::Preserve
        || f.context().raw_text().get() != Raw::No
    {
        Cow::Borrowed(joined)
    } else {
        joined.cow_replace('\n', " ")
    };
    let runs = f.context().code_span_literal_runs().borrow();
    let literal_runs =
        runs.binary_search_by_key(&code.span.start, |&(start, _)| start).map_or(0, |i| runs[i].1);
    let mut len = min_absent_run(&value, b'`');
    while len <= 64 && literal_runs & (1u64 << (len - 1)) != 0 {
        len += 1;
    }
    let fence = backticks(len);
    let is_space_or_newline = |c: char| c == ' ' || c == '\n';
    let padding = value.starts_with('`')
        || value.ends_with('`')
        || (value.starts_with(is_space_or_newline)
            && value.ends_with(is_space_or_newline)
            && value.chars().any(|c| !is_space_or_newline(c)));
    let padding = if padding { " " } else { "" };
    f.allocator().alloc_concat_strs_array([&fence, padding, &value, padding, &fence])
}
