//! Phrasing content.
//!
//! A paragraph is ONE `fill`: every whitespace that may break is a separator,
//! everything else (words, markers, code spans, hard breaks) is content.
//! [`Parts`] collects that shape; the node printers push into it.

use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_allocator::{Allocator, ArenaVec, StringBuilder};
use oxc_formatter_core::{
    Buffer, Format,
    builders::{hard_line_break, literal_line_break, mark_as_root, soft_line_break_or_space, text},
    write,
};
use oxc_markdown_parser::{
    ast::{CodeSpan, HardBreakKind, Inline},
    unicode,
};

use crate::{context::MarkdownFormatContext, options::ProseWrap};

use super::{
    MarkdownFormatter, backticks, block, format_with, join_pieces, link, text as words, with_depth,
};

/// A piece of fill content.
#[derive(Clone, Copy)]
pub enum Atom<'a> {
    Str(&'a str),
    /// A forced break inside content (backslash hard break, HTML comment lines).
    HardLine,
    /// A line break that keeps trailing whitespace and resumes at the current indention
    /// (two-space hard break, inline HTML lines).
    LiteralLine,
}

impl<'a> Format<'a, MarkdownFormatContext<'a>> for Atom<'a> {
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        match self {
            Atom::Str(s) => write!(f, text(s)),
            Atom::HardLine => write!(f, hard_line_break()),
            Atom::LiteralLine => write!(f, mark_as_root(&literal_line_break())),
        }
    }
}

/// A fill separator.
/// TODO: `softline`, for the `""` whitespace between CJ characters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sep {
    Line,
    HardLine,
}

impl<'a> Format<'a, MarkdownFormatContext<'a>> for Sep {
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        match self {
            Sep::Line => write!(f, soft_line_break_or_space()),
            Sep::HardLine => write!(f, hard_line_break()),
        }
    }
}

#[derive(Clone, Copy)]
enum Item<'a> {
    Atom(Atom<'a>),
    Sep(Sep),
}

/// The alternating content / separator list of one fill, flat (no per-word allocation);
/// runs of atoms are grouped when the fill is written.
pub struct Parts<'a> {
    items: ArenaVec<'a, Item<'a>>,
}

impl<'a> Parts<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { items: ArenaVec::new_in(&allocator) }
    }

    pub fn push_str(&mut self, s: &'a str) {
        if !s.is_empty() {
            self.push_atom(Atom::Str(s));
        }
    }

    pub fn push_atom(&mut self, atom: Atom<'a>) {
        self.items.push(Item::Atom(atom));
    }

    /// Adjacent separators collapse to the strongest (a text edge next to a soft break).
    pub fn push_sep(&mut self, sep: Sep) {
        match self.items.last_mut() {
            Some(Item::Sep(existing)) => *existing = (*existing).max(sep),
            _ => self.items.push(Item::Sep(sep)),
        }
    }

    /// `value` split into lines, `separator` between them.
    pub fn push_lines(&mut self, value: &'a str, separator: Atom<'a>) {
        for (i, line) in value.split('\n').enumerate() {
            if i > 0 {
                self.push_atom(separator);
            }
            self.push_str(line);
        }
    }

    /// The content on one line (table cells): every separator is a space.
    pub fn into_str(self, allocator: &'a Allocator) -> &'a str {
        if let [Item::Atom(Atom::Str(s))] = self.items.as_slice() {
            return s;
        }
        let mut out = StringBuilder::new_in(allocator);
        for item in &self.items {
            match item {
                Item::Atom(Atom::Str(s)) => out.push_str(s),
                _ => out.push(' '),
            }
        }
        out.into_str()
    }

    pub fn write_fill(self, f: &mut MarkdownFormatter<'_, 'a>) {
        let mut fill = f.fill();
        let mut sep = Sep::Line;
        let mut run_start = 0;
        for (i, item) in self.items.iter().enumerate() {
            if let Item::Sep(next) = item {
                // A fill alternates content / separator;
                // a separator with no content before it gets an empty entry.
                let run = &self.items[run_start..i];
                fill.entry(&sep, &format_with(|f| write_atoms(run, f)));
                sep = *next;
                run_start = i + 1;
            }
        }
        if !self.items.is_empty() {
            let run = &self.items[run_start..];
            fill.entry(&sep, &format_with(|f| write_atoms(run, f)));
        }
        fill.finish();
    }
}

fn write_atoms<'a>(items: &[Item<'a>], f: &mut MarkdownFormatter<'_, 'a>) {
    for item in items {
        if let Item::Atom(atom) = item {
            atom.fmt(f);
        }
    }
}

/// Where the inline children hang; a few rules depend on the immediate parent.
#[derive(Clone, Copy, Default)]
pub struct InlineParent {
    /// The children are a paragraph's (its first / last text gets trimmed).
    pub paragraph: bool,
    /// The parent is an `Emphasis` / `Strong` (a leading `*` / `_` on the first word is escaped).
    pub delimiter: bool,
    /// `Some(has_word_neighbor)` when the parent is a `Strong` (nested emphasis reads it).
    pub strong_neighbor: Option<bool>,
    /// The paragraph is the first block of a blockquote
    /// (`[!NOTE]` on its first line is an alert marker).
    pub first_in_container: bool,
}

/// Paragraph, heading or table-cell content as one fill.
pub fn write_inlines<'a>(
    children: &'a [Inline<'a>],
    parent: InlineParent,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    let mut parts = Parts::new(f.allocator());
    let raw = parent.paragraph && wiki_link_risk(children, &mut false, f);
    f.context().raw_text().set(raw);
    collect_inlines(children, parent, &mut parts, f);
    f.context().raw_text().set(false);
    parts.write_fill(f);
}

/// Dialect line shapes (see AGENTS.md "Dialects"):
/// a source line starting with one of these keeps both its line boundaries and is never re-wrapped,
/// whatever `proseWrap` says.
/// `alert`: the line is the first of a blockquote's first paragraph (`[!NOTE]`).
fn line_shape<'a>(
    inline: Option<&'a Inline<'a>>,
    alert: bool,
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    let raw = match inline {
        Some(Inline::Text(t)) => f.context().slice(t.span).trim_start_matches(HTML_WHITESPACE),
        Some(Inline::HtmlInline(h)) => f.context().slice(h.span),
        _ => return false,
    };
    // `:::` that is not a directive opener (a stray closer, `:::)`), a VitePress `<<<` snippet import,
    // a component tag (`<Badge />`, `<my-element>`: uppercase or hyphenated names, which are not HTML)
    raw.starts_with(":::")
        || raw.starts_with("<<<")
        || (alert && raw.starts_with("[!"))
        || is_component_tag(raw)
}

fn is_component_tag(raw: &str) -> bool {
    let Some(rest) = raw.strip_prefix('<') else { return false };
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let name_len = rest.bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'-').count();
    let name = &rest[..name_len];
    name.starts_with(|c: char| c.is_ascii_uppercase())
        || (name.contains('-') && name.starts_with(|c: char| c.is_ascii_alphabetic()))
}

/// Prettier's `riskyParagraphPositions`:
/// a text with `[[` followed (in source order, itself included) by a text with `]]` or a wiki link.
/// Wrapping such a paragraph could merge `[[foo\n[[wiki link]]` into one link,
/// so its text is printed as written.
fn wiki_link_risk<'a>(
    children: &'a [Inline<'a>],
    can_open: &mut bool,
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    children.iter().any(|child| match child {
        Inline::Text(t) => {
            let raw = f.context().slice(t.span);
            *can_open |= raw.contains("[[");
            *can_open && raw.contains("]]")
        }
        Inline::WikiLink(_) => *can_open,
        Inline::Emphasis(e) => wiki_link_risk(&e.children, can_open, f),
        Inline::Strong(s) => wiki_link_risk(&s.children, can_open, f),
        Inline::Strikethrough(s) => wiki_link_risk(&s.children, can_open, f),
        Inline::Link(l) => wiki_link_risk(&l.children, can_open, f),
        _ => false,
    })
}

pub fn collect_inlines<'a>(
    children: &'a [Inline<'a>],
    parent: InlineParent,
    parts: &mut Parts<'a>,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    let last = children.len().wrapping_sub(1);
    // Raw text is switched per source line of a top-level paragraph:
    // every line when wiki links are at risk, else the lines starting with a dialect shape.
    // Nested content (emphasis, links) inherits the line's setting through the context.
    let raw_paragraph = parent.paragraph && f.context().raw_text().get();
    let raw_line = |j: usize, f: &MarkdownFormatter<'_, 'a>| {
        raw_paragraph
            || (parent.paragraph
                && line_shape(children.get(j), j == 0 && parent.first_in_container, f))
    };
    if parent.paragraph {
        f.context().raw_text().set(raw_line(0, f));
    }
    for (i, child) in children.iter().enumerate() {
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
                if f.context().raw_text().get() {
                    parts.push_lines(raw, Atom::HardLine);
                    continue;
                }
                let cx = words::TextContext {
                    first_of_delimiter: parent.delimiter && i == 0,
                    after_soft_break: matches!(
                        children.get(i.wrapping_sub(1)),
                        Some(Inline::SoftBreak(_))
                    ),
                    next_word: next_word_of(children, i, f),
                    ..words::TextContext::default()
                };
                words::push_text(raw, cx, parts, f);
            }
            Inline::SoftBreak(_) => {
                // The break stays when either side of it is raw
                let raw_cell = f.context().raw_text();
                let mut keep = raw_cell.get();
                if parent.paragraph {
                    let next = raw_line(i + 1, f);
                    keep |= next;
                    raw_cell.set(next);
                }
                if keep {
                    parts.push_atom(Atom::HardLine);
                } else {
                    let cx = words::TextContext {
                        next_word: next_word_of(children, i, f),
                        ..words::TextContext::default()
                    };
                    words::push_whitespace(words::Ws::Newline, &cx, parts, f);
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
                if parent.paragraph {
                    f.context().raw_text().set(raw_line(i + 1, f));
                }
            }
            Inline::Emphasis(e) => {
                // Around an autolink (`<…>` or a bare URL) the marker is kept as written
                let style: &'static str = if matches!(
                    e.children.first(),
                    Some(Inline::Autolink(_) | Inline::AutolinkLiteral(_))
                ) {
                    if e.marker == b'_' { "_" } else { "*" }
                } else {
                    // `1*2*3` is emphasis but `1_2_3` is not: a word right next to the marker forces `*`
                    let word_neighbor = has_word_neighbor(children, i, f)
                        || parent.strong_neighbor == Some(true)
                        || f.context().emphasis_depth().get() > 0;
                    if word_neighbor { "*" } else { "_" }
                };
                parts.push_str(style);
                let inner = InlineParent { delimiter: true, ..InlineParent::default() };
                with_depth(f, MarkdownFormatContext::emphasis_depth, |f| {
                    with_depth(f, MarkdownFormatContext::delimiter_depth, |f| {
                        collect_inlines(&e.children, inner, parts, f);
                    });
                });
                parts.push_str(style);
            }
            Inline::Strong(s) => {
                parts.push_str("**");
                let inner = InlineParent {
                    delimiter: true,
                    strong_neighbor: Some(has_word_neighbor(children, i, f)),
                    ..InlineParent::default()
                };
                with_depth(f, MarkdownFormatContext::delimiter_depth, |f| {
                    collect_inlines(&s.children, inner, parts, f);
                });
                parts.push_str("**");
            }
            Inline::Strikethrough(s) => {
                parts.push_str("~~");
                collect_inlines(&s.children, InlineParent::default(), parts, f);
                parts.push_str("~~");
            }
            Inline::CodeSpan(c) => parts.push_str(print_code_span(c, f)),
            Inline::Link(l) => link::collect_link(l, parts, f),
            Inline::Image(img) => link::collect_image(img, parts, f),
            Inline::HtmlInline(h) => {
                // Verbatim, continuation indentation included (mdast's `html` value is the source slice)
                let value = join_pieces(&h.pieces, f);
                let separator =
                    if block::is_html_comment(value) { Atom::HardLine } else { Atom::LiteralLine };
                parts.push_lines(value, separator);
            }
            // Printed as written.
            // Math spans too: remark-math trims them, but the match may be accidental.
            Inline::Autolink(_)
            | Inline::AutolinkLiteral(_)
            | Inline::FootnoteReference(_)
            | Inline::MathSpan(_)
            | Inline::MdxExpression(_)
            | Inline::MdxJsx(_) => parts.push_str(f.context().slice(child.span())),
            Inline::WikiLink(w) => {
                let raw = f.context().slice(w.span);
                let inner = &raw[2..raw.len() - 2];
                parts.push_str("[[");
                if f.options().prose_wrap == ProseWrap::Preserve || !inner.contains(['\t', '\n']) {
                    parts.push_str(inner);
                } else {
                    parts.push_str(collapse_tabs_and_newlines(inner, f));
                }
                parts.push_str("]]");
            }
            Inline::Liquid(l) => parts.push_lines(join_pieces(&l.pieces, f), Atom::HardLine),
        }
    }
}

/// HTML whitespace: `\t\n\f\r` and space.
const HTML_WHITESPACE: [char; 5] = ['\t', '\n', '\u{c}', '\r', ' '];

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

/// The first word of the next sibling when it is a text, for the whitespace that ends this one.
fn next_word_of<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<words::NextWord<'a>> {
    let Some(Inline::Text(t)) = children.get(i + 1) else { return None };
    let raw = f.context().slice(t.span);
    let mut it = raw.split(words::is_split_whitespace).filter(|w| !w.is_empty());
    let word = it.next()?;
    Some(words::NextWord { word, alone_on_line: it.next().is_none() })
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

/// Backtick fence of the shortest length the content permits; a space pads content that
/// starts / ends with a backtick or is surrounded by spaces (CommonMark strips one).
/// Inside a table cell a `|` is `\|` in the source and stays so
/// (the raw slice already carries the escape Prettier re-adds to its decoded value).
fn print_code_span<'a>(code: &'a CodeSpan<'a>, f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    let joined = join_pieces(&code.pieces, f);
    let value: Cow<'_, str> = if f.options().prose_wrap == ProseWrap::Preserve {
        Cow::Borrowed(joined)
    } else {
        joined.cow_replace('\n', " ")
    };
    let fence = backticks(min_absent_run(&value, b'`'));
    let is_space_or_newline = |c: char| c == ' ' || c == '\n';
    let padding = value.starts_with('`')
        || value.ends_with('`')
        || (value.starts_with(is_space_or_newline)
            && value.ends_with(is_space_or_newline)
            && value.chars().any(|c| !is_space_or_newline(c)));
    let padding = if padding { " " } else { "" };
    f.allocator().alloc_concat_strs_array([&fence, padding, &value, padding, &fence])
}

/// The smallest run length of `ch` absent from `text`.
fn min_absent_run(text: &str, ch: u8) -> usize {
    // Bit `n - 1` set: a run of exactly `n` is present (runs past 64 are treated as present)
    let mut present: u64 = 0;
    let mut run = 0usize;
    for &b in text.as_bytes().iter().chain(std::iter::once(&0)) {
        if b == ch {
            run += 1;
        } else if run > 0 {
            present |= 1u64.checked_shl(u32::try_from(run - 1).unwrap_or(64)).unwrap_or(0);
            run = 0;
        }
    }
    present.trailing_ones() as usize + 1
}
