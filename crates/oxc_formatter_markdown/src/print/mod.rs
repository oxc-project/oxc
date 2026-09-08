use std::{borrow::Cow, cell::Cell};

use oxc_allocator::StringBuilder;
use oxc_formatter_core::{
    Buffer, Format, Formatter,
    builders::{FormatWith, Line, empty_line, exact_line_breaks, hard_line_break, text, token},
    write,
};
use oxc_markdown_parser::{Segment, Span, ast::Root};

use crate::context::MarkdownFormatContext;

mod block;
mod blockquote;
mod code;
mod inline;
mod link;
mod list;
mod table;
mod text;

pub type MarkdownFormatter<'buf, 'a> = Formatter<'buf, 'a, MarkdownFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `MarkdownFormatContext`.
///
/// Hardcoded to `MarkdownFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, MarkdownFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        write!(f, token(self));
    }
}

/// Wraps a re-entrant Markdown closure in a [`FormatWith`].
/// The closure's context is pinned to [`MarkdownFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut MarkdownFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

pub fn write_root<'a>(root: &'a Root<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    block::write_blocks(&root.children, block::Parent::Root, f);
}

/// Runs `body` with one of the context's depth counters incremented.
/// The pairing is structural, so an early return inside `body` cannot leave the counter off.
pub fn with_depth<'a>(
    f: &mut MarkdownFormatter<'_, 'a>,
    counter: for<'c> fn(&'c MarkdownFormatContext<'a>) -> &'c Cell<u32>,
    body: impl FnOnce(&mut MarkdownFormatter<'_, 'a>),
) {
    let cell = counter(f.context());
    cell.set(cell.get() + 1);
    body(f);
    let cell = counter(f.context());
    cell.set(cell.get() - 1);
}

/// Emits the source slice as-is (`prettier-ignore`, constructs not printed yet).
///
/// The source is normalized to `\n` before parsing, so the slice is safe for the IR;
/// a multi-line `text` prints its embedded newlines literally,
/// so inside a blockquote the continuation lines keep the `>` markers they carry in the source (as Prettier's slices do).
/// The trailing line ending belongs to the sibling separator, not to the block.
pub fn write_verbatim(span: Span, f: &mut MarkdownFormatter<'_, '_>) {
    write!(f, text(f.context().slice(span).trim_end_matches('\n')));
}

/// Emits `lines` one per output line, `separator` between them.
///
/// `hard_line_break` collapses consecutive breaks, so callers pass `exact_line_breaks(1)` (re-indents)
/// or `literal_line_break` (root-anchored, keeps trailing whitespace).
/// An empty line emits no text, so its indention stays pending and never prints.
pub fn write_lines<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    separator: Line,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    for (i, line) in lines.into_iter().enumerate() {
        if i > 0 {
            write!(f, separator);
        }
        if !line.is_empty() {
            write!(f, text(line));
        }
    }
}

/// [`write_lines`] with re-indenting breaks.
/// Trailing spaces and tabs go, as Prettier's printer trims them before each hard line break.
pub fn write_indented_lines<'a>(value: &'a str, f: &mut MarkdownFormatter<'_, 'a>) {
    write_lines(
        value.split('\n').map(|line| line.trim_end_matches([' ', '\t'])),
        exact_line_breaks(1),
        f,
    );
}

/// The separator between two sibling blocks: a blank line, or just a line break.
pub fn write_gap(blank: bool, f: &mut MarkdownFormatter<'_, '_>) {
    if blank {
        write!(f, empty_line());
    } else {
        write!(f, hard_line_break());
    }
}

/// The logical text of a multi-line construct (`Segment::join`), in the arena when it has to be built.
pub fn join_pieces<'a>(pieces: &[Segment], f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    let source = f.context().source_text().as_str();
    match pieces {
        [single] if single.padding == 0 => single.span.slice(source),
        _ => {
            let len = pieces
                .iter()
                .map(|p| (p.span.end - p.span.start) as usize + usize::from(p.padding) + 1)
                .sum();
            let mut out = StringBuilder::with_capacity_in(len, f.allocator());
            for (i, piece) in pieces.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                for _ in 0..piece.padding {
                    out.push(' ');
                }
                out.push_str(piece.span.slice(source));
            }
            out.into_str()
        }
    }
}

/// 0-based column of `offset`, in bytes:
/// the only prefixes that lead a line are container markers and ASCII whitespace,
/// where bytes == characters.
pub fn column_of(source: &str, offset: u32) -> usize {
    source.as_bytes()[..offset as usize].iter().rev().take_while(|&&b| b != b'\n').count()
}

/// The longest run of `ch` in `text`.
pub fn max_run(text: &str, ch: u8) -> usize {
    let mut max = 0;
    let mut run = 0;
    for &b in text.as_bytes() {
        if b == ch {
            run += 1;
            max = max.max(run);
        } else {
            run = 0;
        }
    }
    max
}

/// `n` backticks; borrowed from a static run for every realistic length.
pub fn backticks(n: usize) -> Cow<'static, str> {
    const RUN: &str = "````````````````````````````````";
    if n <= RUN.len() { Cow::Borrowed(&RUN[..n]) } else { Cow::Owned("`".repeat(n)) }
}
