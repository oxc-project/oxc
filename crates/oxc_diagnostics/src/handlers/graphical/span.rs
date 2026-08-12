//! The span model used while drawing a snippet.
//!
//! A [`FancySpan`] is one of the diagnostic's labels paired with the [`Style`]
//! it will be drawn in. Label text stays borrowed and is split into display
//! lines only when it is drawn.

use owo_colors::Style;

use oxc_span::Span;

/// How a label is being drawn on the current output line.
///
/// A label can be a single trailing line, or a block that spans several output
/// lines (the first line drawn differently from the rest).
#[derive(Copy, Clone, PartialEq, Debug)]
pub(super) enum LabelRenderMode {
    /// we're rendering a single line label (or not rendering in any special way)
    SingleLine,
    /// we're rendering a multiline label
    BlockFirst,
    /// we're rendering the rest of a multiline label
    BlockRest,
}

#[derive(Debug, Clone)]
pub(super) struct FancySpan<'a> {
    label: Option<&'a str>,
    span: Span,
    pub(super) style: Style,
}

impl PartialEq for FancySpan<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.span == other.span
    }
}

impl<'a> FancySpan<'a> {
    pub(super) fn new(label: Option<&'a str>, span: Span, style: Style) -> Self {
        FancySpan { label, span, style }
    }

    pub(super) fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub(super) fn label(&self) -> Option<&str> {
        self.label
    }

    pub(super) fn offset(&self) -> usize {
        self.span.start as usize
    }

    pub(super) fn len(&self) -> usize {
        self.span.size() as usize
    }
}
