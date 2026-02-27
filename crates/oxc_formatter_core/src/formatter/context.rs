use oxc_allocator::Allocator;
use oxc_span::Span;

use crate::{
    formatter::printer::{AsPrinterOptions, PrinterOptions},
    options::{IndentStyle, IndentWidth, IndentWidthProvider, LineEnding, LineWidth},
};

/// Formatting context for a formatting session.
pub trait FormatContext<'ast> {
    type Options: AsPrinterOptions + IndentWidthProvider;

    /// Returns the formatting options for this context.
    fn options(&self) -> &Self::Options;

    /// Returns the allocator used for this formatting session.
    fn allocator(&self) -> &'ast Allocator;
}

/// Optional extension trait for contexts that expose comment and source text helpers.
pub trait FormatContextExt<'ast>: FormatContext<'ast> {
    type Comments;
    type SourceText: Copy + SourceTextExt<Self::Comments>;

    fn comments(&self) -> &Self::Comments;
    fn comments_mut(&mut self) -> &mut Self::Comments;
    fn source_text(&self) -> Self::SourceText;
}

pub trait SourceTextExt<Comments> {
    fn get_lines_before(&self, span: Span, comments: &Comments) -> usize;
}

#[derive(Debug, Clone)]
pub struct SimpleFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_ending: LineEnding,
    pub line_width: LineWidth,
}

impl Default for SimpleFormatOptions {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::default(),
            indent_width: IndentWidth::default(),
            line_ending: LineEnding::default(),
            line_width: LineWidth::default(),
        }
    }
}

impl AsPrinterOptions for SimpleFormatOptions {
    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::default()
            .with_indent_style(self.indent_style)
            .with_indent_width(self.indent_width)
            .with_print_width(self.line_width.into())
            .with_line_ending(self.line_ending)
    }
}

impl IndentWidthProvider for SimpleFormatOptions {
    fn indent_width(&self) -> IndentWidth {
        self.indent_width
    }
}

#[derive(Clone)]
pub struct SimpleFormatContext<'ast> {
    allocator: &'ast Allocator,
    options: SimpleFormatOptions,
}

impl<'ast> SimpleFormatContext<'ast> {
    pub fn new(allocator: &'ast Allocator) -> Self {
        Self { allocator, options: SimpleFormatOptions::default() }
    }

    pub fn with_options(allocator: &'ast Allocator, options: SimpleFormatOptions) -> Self {
        Self { allocator, options }
    }
}

impl<'ast> FormatContext<'ast> for SimpleFormatContext<'ast> {
    type Options = SimpleFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn allocator(&self) -> &'ast Allocator {
        self.allocator
    }
}

impl Default for SimpleFormatContext<'static> {
    fn default() -> Self {
        let allocator = Box::leak(Box::new(Allocator::default()));
        Self::new(allocator)
    }
}
