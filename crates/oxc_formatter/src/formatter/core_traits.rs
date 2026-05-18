use crate::options::{IndentStyle, IndentWidth, LineEnding, LineWidth};

use super::printer::PrinterOptions;

/// Language-agnostic formatting context trait.
///
/// Each language formatter defines its own context type implementing this trait.
/// The context provides access to language-specific options and source text.
///
/// Note: The allocator is stored separately in `FormatState`, not in the context,
/// because the `'ast` arena lifetime is specific to oxc's memory model and should not
/// leak into the language-agnostic trait interface.
pub trait FormatContext {
    /// The language-specific format options type.
    type Options: FormatOptions;

    /// Returns the formatting options.
    fn options(&self) -> &Self::Options;

    /// Returns the source text being formatted as a raw string slice.
    fn source_code(&self) -> &str;
}

/// Language-agnostic format options trait.
///
/// Provides the subset of options needed by the core IR printer.
/// Language-specific options (e.g., JS quote style, semicolons) are added by each language's
/// concrete options type.
pub trait FormatOptions {
    /// The indent style (tabs or spaces).
    fn indent_style(&self) -> IndentStyle;

    /// The indent width in characters.
    fn indent_width(&self) -> IndentWidth;

    /// The maximum line width.
    fn line_width(&self) -> LineWidth;

    /// The line ending style.
    fn line_ending(&self) -> LineEnding;

    /// Convert to printer options.
    fn as_print_options(&self) -> PrinterOptions;
}
