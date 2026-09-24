//! What a printed line would open.
//!
//! Wrapping moves words to line starts and joins source lines;
//! a line that then reads as a block start would leave the paragraph.
//! These predicates ask the parser (`lexical::line_start`) about the line the printer is about to produce.
//! The dialect line shapes (AGENTS.md "Dialects") live here too.

use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_markdown_parser::{Constructs, ast::Inline, lexical};

use crate::{context::Raw, options::ProseWrap};

use super::{HTML_WHITESPACE, MarkdownFormatter, inline::print_code_span, is_split_whitespace};

/// HTML whitespace: `\t\n\f\r` and space.
/// Dialect line shapes (see AGENTS.md "Dialects"):
/// a source line starting with one of these keeps both its line boundaries and is never re-wrapped,
/// whatever `proseWrap` says.
/// `alert`: the line is the first of a blockquote's first paragraph (`[!NOTE]`).
pub fn line_shape<'a>(
    children: &'a [Inline<'a>],
    j: usize,
    first_line: bool,
    alert: bool,
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    let Some(inline) = children.get(j) else { return false };
    if matches!(inline, Inline::SoftBreak(_) | Inline::HardBreak(_)) {
        return false;
    }
    // The whole source line from this node on (a liquid tag, a tag, text: whatever starts it)
    let line = printed_line_at(children, j, f);
    let raw: &str = &line.text;
    // The shapes, an alert, and a line that is not a block only because of what follows on it
    // (`[label]: dest text`, ``` ```a `b` ```, `$$x $y$`, `</span> text` on a first line):
    // a wrap inside it would leave the block behind.
    // (A first line that opens a block as a whole is escaped instead, or printed after its definition;
    // a continuation line's start is guarded where it stays, after a kept break.)
    is_line_shape_start(raw)
        || (alert && raw.starts_with("[!"))
        || is_unfinished_block_shape(raw, matches!(inline, Inline::Text(_)), first_line)
        || (first_line && !line_opens_block(raw, false) && line_prefix_opens_block(&line, false, f))
}

/// A source line as it prints: a code span at its start in its printed form
/// (the fence is recomputed, so ```` ```x``` ```` prints as `` `x` ``,
/// and next to stray backticks `` `x` `` as ```` ```x```` ````), the rest as in the source.
///
/// A code span is the only inline whose printed form can open a block where its source does not (or the reverse),
/// so this is the one substitution the source-based guards need.
/// Should a second one ever be needed, run the guards over the collected `Parts` instead
/// (the printed text, as `pairing::consistent` already does) rather than splicing again here.
pub struct PrintedLine<'a> {
    /// Source offset of the line's first byte.
    base: u32,
    pub text: Cow<'a, str>,
    /// The leading `printed` bytes of `text` stand for `source` bytes of the source line.
    printed: usize,
    source: usize,
}

impl<'a> PrintedLine<'a> {
    /// `printed` (possibly empty) in place of the first `source` bytes of the source line, then `tail`.
    fn spliced(base: u32, printed: &str, tail: Cow<'a, str>, source: usize) -> Self {
        if printed.is_empty() {
            Self { base, text: tail, printed: 0, source: 0 }
        } else {
            Self {
                base,
                text: Cow::Owned(format!("{printed}{tail}")),
                printed: printed.len(),
                source,
            }
        }
    }

    /// The source offset of byte `at` of `text`
    /// (inside the replaced span, its start: the span is opaque as a whole).
    fn offset(&self, at: usize) -> u32 {
        let at = if at < self.printed { 0 } else { at - self.printed + self.source };
        self.base + u32::try_from(at).unwrap_or(u32::MAX)
    }
}

/// The source line starting at offset `start` (as `source_line_at`),
/// its leading bytes up to `source_end` replaced by `printed` when given.
pub fn line_from<'a>(
    children: &[Inline<'_>],
    start: u32,
    replace: Option<(&'a str, u32)>,
    f: &MarkdownFormatter<'_, 'a>,
) -> PrintedLine<'a> {
    let (base, line) = source_line_at(children, start, f);
    let (printed, source) = replace.map_or(("", 0), |(printed, source_end)| {
        (printed, (source_end.saturating_sub(base) as usize).min(line.len()))
    });
    PrintedLine::spliced(base, printed, Cow::Borrowed(&line[source..]), source)
}

/// The source line from node `j` on, as it prints ([`line_from`]).
pub fn printed_line_at<'a>(
    children: &'a [Inline<'a>],
    j: usize,
    f: &MarkdownFormatter<'_, 'a>,
) -> PrintedLine<'a> {
    let node = &children[j];
    let replace = match node {
        Inline::CodeSpan(code) => {
            let printed = print_code_span(code, f);
            Some((printed.split('\n').next().unwrap_or(printed), code.span.end))
        }
        _ => None,
    };
    line_from(children, node.span().start, replace, f)
}

/// The source line starting at offset `start` (nodes after it on the line included),
/// up to the content's end, without the leading whitespace the printer drops;
/// with the offset it then starts at.
pub fn source_line_at<'a>(
    children: &[Inline<'_>],
    start: u32,
    f: &MarkdownFormatter<'_, 'a>,
) -> (u32, &'a str) {
    let end = children.last().map_or(start, |last| last.span().end);
    let raw = f.context().source_text().slice_range(start, end.max(start));
    let line = raw.trim_start_matches(HTML_WHITESPACE);
    let base = start + u32::try_from(raw.len() - line.len()).unwrap_or(0);
    (base, line.split('\n').next().unwrap_or(line))
}

/// Whether the line, or any prefix wrapping may cut it to, opens a block.
pub fn line_or_prefix_opens_block(
    line: &PrintedLine<'_>,
    in_paragraph: bool,
    f: &MarkdownFormatter<'_, '_>,
) -> bool {
    line_opens_block(&line.text, in_paragraph) || line_prefix_opens_block(line, in_paragraph, f)
}

/// Whether a proper prefix of the line, cut at a whitespace wrapping may break
/// (outside opaque nodes, a code span being one atom, and not right after a leading liquid tag,
/// which `push_whitespace` glues to its next word), opens a block.
/// Only `always` wraps, so only it has prefixes.
pub fn line_prefix_opens_block(
    line: &PrintedLine<'_>,
    in_paragraph: bool,
    f: &MarkdownFormatter<'_, '_>,
) -> bool {
    let text: &str = &line.text;
    if f.options().prose_wrap != ProseWrap::Always
        || !text.as_bytes().first().is_some_and(|&b| may_open_block(b))
    {
        return false;
    }
    // A prefix that is exactly a leading liquid tag: the whitespace after it never breaks
    let glued_tag = |prefix: &str| {
        let tag = prefix.trim_end_matches(is_split_whitespace);
        (tag.ends_with("%}") || tag.ends_with("}}"))
            && lexical::line_start(&Constructs::markdown(), tag, in_paragraph)
                == Some(lexical::LineStart::Liquid)
    };
    let opaque = f.context().opaque_spans().borrow();
    text.match_indices(is_split_whitespace).any(|(at, _)| {
        let offset = line.offset(at);
        !opaque.iter().any(|(span, _)| span.contains(offset))
            && !glued_tag(&text[..at])
            && line_opens_block(&text[..at], in_paragraph)
    })
}

/// A paragraph whose first line, printed at a block start,
/// opens a block that no escape can prevent (a tag: `<Badge />`, `</span>`).
/// Such a paragraph is the rest of one a definition was split from;
/// printed right after the definition, as in the source, its line stays a continuation.
pub fn first_line_opens_block(children: &[Inline<'_>], f: &MarkdownFormatter<'_, '_>) -> bool {
    match children.first() {
        None | Some(Inline::Text(_)) => false,
        Some(first) => line_opens_block(source_line_at(children, first.span().start, f).1, false),
    }
}

/// After a kept line break before node `i`:
/// whether a line the printer may produce from there opens a block.
/// Under `preserve` (and on a raw line) that is the source line.
/// Otherwise the source lines up to the next kept break are joined,
/// and the printed line is that text (`never`) or any prefix of it ending at a word boundary (`always`);
/// each is a candidate, which keeps the decision the same on every pass
/// (`| - | - | a` may wrap to a delimiter row `| - | - |`, `:-` may join to `:- ]`).
pub fn printed_line_opens_block<'a>(
    children: &'a [Inline<'a>],
    i: usize,
    break_kept: &dyn Fn(usize, &MarkdownFormatter<'_, 'a>) -> bool,
    f: &MarkdownFormatter<'_, 'a>,
) -> bool {
    // The source line runs to the line end
    // (through a backslash hard break: `---\` is no thematic break)
    let line = printed_line_at(children, i, f);
    if !line.text.as_bytes().first().is_some_and(|&b| may_open_block(b)) {
        return false;
    }
    let prose_wrap = f.options().prose_wrap;
    if prose_wrap == ProseWrap::Preserve || f.context().raw_text().get() != Raw::No {
        return line_opens_block(&line.text, true);
    }
    let start = children[i].span().start;
    let end = children
        .iter()
        .enumerate()
        .skip(i + 1)
        .find_map(|(j, c)| match c {
            Inline::HardBreak(_) => Some(c.span().start),
            Inline::SoftBreak(_) if break_kept(j, f) => Some(c.span().start),
            _ => None,
        })
        .unwrap_or_else(|| children[children.len() - 1].span().end);
    // A verbatim node keeps its line breaks: the line ends at its first one
    let end = children[i..]
        .iter()
        .filter(|c| {
            matches!(
                c,
                Inline::HtmlInline(_) | Inline::Liquid(_) | Inline::MathSpan(_) | Inline::Image(_)
            )
        })
        .take_while(|c| c.span().start < end)
        .find_map(|c| {
            let newline = f.context().slice(c.span()).find('\n')?;
            Some(c.span().start + u32::try_from(newline).unwrap_or(0))
        })
        .map_or(end, |newline| newline.min(end));
    // A leading code span prints joined too (`print_code_span`);
    // the rest is the source, lines joined.
    let (head, rest_start) = match &children[i] {
        Inline::CodeSpan(code) => (print_code_span(code, f), code.span.end.min(end)),
        _ => ("", start),
    };
    let segment = f.context().source_text().slice_range(rest_start, end.max(rest_start));
    let trimmed =
        if head.is_empty() { segment.trim_start_matches(HTML_WHITESPACE) } else { segment };
    let line = PrintedLine::spliced(
        start + u32::try_from(segment.len() - trimmed.len()).unwrap_or(0),
        head,
        trimmed.cow_replace('\n', " "),
        (rest_start - start) as usize,
    );
    if prose_wrap == ProseWrap::Always {
        line_or_prefix_opens_block(&line, true, f)
    } else {
        line_opens_block(&line.text, true)
    }
}

/// The line, printed at column 0, would open a block:
/// as a paragraph continuation (`in_paragraph`, where setext underlines and table delimiter rows also count)
/// or as a paragraph's first line.
/// A bare `:::` run only closes, and a `|` row is inert without a table.
pub fn line_opens_block(line: &str, in_paragraph: bool) -> bool {
    match lexical::line_start(&Constructs::markdown(), line, in_paragraph) {
        None | Some(lexical::LineStart::TableRow | lexical::LineStart::DirectiveCloser) => false,
        Some(_) => true,
    }
}

/// A line that is not a block only because of what follows on it:
/// a fence or math opener whose info has a backtick or a `$` (they interrupt a paragraph, so any line counts),
/// a liquid tag not closed on the line (a later line may close it; one closed here with text after it is paragraph text),
/// and on a paragraph's first line a definition (`[label]: dest text`; one cannot interrupt).
/// `is_text`: the line starts with text (a code span or math span is one atom and never wraps inside).
fn is_unfinished_block_shape(raw: &str, is_text: bool, first_line: bool) -> bool {
    (is_text
        && ((first_line
            && raw.starts_with('[')
            && raw.find(']').is_some_and(|i| raw[i + 1..].starts_with(':')))
            || raw.starts_with("```")
            || raw.starts_with("~~~")
            || raw.starts_with("$$")))
        || ((raw.starts_with("{%") || raw.starts_with("{{"))
            && lexical::line_start(&Constructs::markdown(), raw, false)
                == Some(lexical::LineStart::Liquid))
}

/// Every block start (and line shape) begins with one of these bytes.
pub fn may_open_block(first: u8) -> bool {
    matches!(
        first,
        b'>' | b'=' | b'-' | b'*' | b'_' | b'+' | b'0'
            ..=b'9' | b'#' | b'`' | b'~' | b'<' | b'$' | b'{' | b':' | b'[' | b'|'
    )
}

/// Dialect line shapes (see AGENTS.md "Dialects") that a line may start with:
/// a `:::` run that is not a directive opener (a bare closer, `:::)`), a VitePress `<<<` snippet import,
/// a component tag (`<Badge />`, `<my-element>`: uppercase or hyphenated names, which are not HTML).
pub fn is_line_shape_start(line: &str) -> bool {
    (line.starts_with(":::")
        && lexical::line_start(&Constructs::markdown(), line, false)
            != Some(lexical::LineStart::DirectiveOpener))
        || line.starts_with("<<<")
        || is_component_tag(line)
}

fn is_component_tag(raw: &str) -> bool {
    let Some(rest) = raw.strip_prefix('<') else { return false };
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let name_len = rest.bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'-').count();
    let name = &rest[..name_len];
    name.starts_with(|c: char| c.is_ascii_uppercase())
        || (name.contains('-') && name.starts_with(|c: char| c.is_ascii_alphabetic()))
}
