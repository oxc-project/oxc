use std::cell::{Cell, RefCell};

use oxc_formatter_core::{FormatContext, SourceText};
use oxc_markdown_parser::Span;

use crate::options::MarkdownFormatOptions;

/// Formatting context for Markdown.
pub struct MarkdownFormatContext<'a> {
    options: MarkdownFormatOptions,
    source_text: SourceText<'a>,
    /// Every logical blank line, in source order (`ParserReturn::blanks`).
    /// Blank runs between siblings carry meaning (loose lists, HTML verbatim boundaries),
    /// so the printer consults this table instead of rescanning the source.
    blanks: &'a [Span],
    /// Enclosing lists, outermost first.
    /// Prettier reads these off `path.ancestors`; this printer has no path, so containers push as they descend.
    lists: RefCell<Vec<ListFrame>>,
    /// Enclosing nodes whose content never wraps (ATX heading, link, table cell, wiki link).
    no_wrap_depth: Cell<u32>,
    /// Enclosing `Emphasis` nodes (marker choice for nested emphasis).
    emphasis_depth: Cell<u32>,
    /// Enclosing `Emphasis` or `Strong` nodes (delimiter-run escaping applies inside).
    delimiter_depth: Cell<u32>,
    /// Inside a paragraph where wrapping could form a wiki link (`[[` ... `]]` across texts):
    /// text is printed as written, newlines included.
    raw_text: Cell<bool>,
}

/// What a list's descendants need to know about it.
#[derive(Clone, Copy, Debug)]
pub struct ListFrame {
    /// The list prints the alternate marker (`*` / `)`), and its thematic breaks `---` instead of `***`.
    pub alternate_marker: bool,
    /// Prettier's `isAligned`; an unaligned ancestor makes every descendant list unaligned.
    pub aligned: bool,
}

impl<'a> MarkdownFormatContext<'a> {
    pub fn new(options: MarkdownFormatOptions, source_code: &'a str, blanks: &'a [Span]) -> Self {
        Self {
            options,
            source_text: SourceText::new(source_code),
            blanks,
            lists: RefCell::new(Vec::new()),
            no_wrap_depth: Cell::new(0),
            emphasis_depth: Cell::new(0),
            delimiter_depth: Cell::new(0),
            raw_text: Cell::new(false),
        }
    }

    /// Returns the source text with the arena lifetime (vs the trait's borrow-elided `&str`).
    /// Slices taken via this method carry the `'a` lifetime,
    /// so they don't have to be re-allocated for `text(...)`.
    pub fn source_text(&self) -> SourceText<'a> {
        self.source_text
    }

    pub fn slice(&self, span: Span) -> &'a str {
        self.source_text.slice_range(span.start, span.end)
    }

    /// Whether a logical blank line lies between two sibling spans.
    ///
    /// Prettier compares `previous.position.end.line + 1 < node.position.start.line`;
    /// the parser's blank table answers the same question without line numbers,
    /// and already ignores lines inside fenced blocks.
    pub fn has_blank_between(&self, prev_end: u32, next_start: u32) -> bool {
        // A blank that starts where the previous node ends is on that node's last line
        // (a blockquote's bare `>` line), not a line between the two.
        let i = self.blanks.partition_point(|blank| blank.start <= prev_end);
        self.blanks.get(i).is_some_and(|blank| blank.start < next_start)
    }

    pub fn lists(&self) -> &RefCell<Vec<ListFrame>> {
        &self.lists
    }

    pub fn no_wrap_depth(&self) -> &Cell<u32> {
        &self.no_wrap_depth
    }

    pub fn emphasis_depth(&self) -> &Cell<u32> {
        &self.emphasis_depth
    }

    pub fn delimiter_depth(&self) -> &Cell<u32> {
        &self.delimiter_depth
    }

    pub fn raw_text(&self) -> &Cell<bool> {
        &self.raw_text
    }
}

impl FormatContext for MarkdownFormatContext<'_> {
    type Options = MarkdownFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
