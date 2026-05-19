use std::cell::Cell;

use oxc_span::{GetSpan, Span};

use super::{JsFormatContext, prelude::*, separated::FormatSeparatedIter};
use crate::{TrailingSeparator, write};

// Re-export the language-agnostic builders that live in `oxc_formatter_core`,
// so that legacy `crate::formatter::builders::*` call-sites keep working
// without having to be updated.
pub use oxc_formatter_core::builders::*;

/// Utility for formatting some content with an inline lambda function.
#[derive(Copy, Clone)]
pub struct FormatWith<T> {
    formatter: T,
}

impl<'ast, C, T> Format<'ast, C> for FormatWith<T>
where
    T: Fn(&mut Formatter<'_, 'ast, C>),
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        (self.formatter)(f);
    }
}

impl<T> std::fmt::Debug for FormatWith<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormatWith").field(&"{{formatter}}").finish()
    }
}

/// Creates an object implementing `Format` that calls the passed closure to perform the formatting.
pub const fn format_with<'ast, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut JsFormatter<'_, 'ast>),
{
    FormatWith { formatter }
}

/// Creates an inline `Format` object that can only be formatted once.
pub const fn format_once<'ast, T>(formatter: T) -> FormatOnce<T>
where
    T: FnOnce(&mut JsFormatter<'_, 'ast>),
{
    FormatOnce { formatter: Cell::new(Some(formatter)) }
}

pub struct FormatOnce<T> {
    formatter: Cell<Option<T>>,
}

impl<'ast, C, T> Format<'ast, C> for FormatOnce<T>
where
    T: FnOnce(&mut Formatter<'_, 'ast, C>),
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        let formatter = self.formatter.take().expect("Tried to format a `format_once` at least twice. This is not allowed. You may want to use `format_with` or `format.memoized` instead.");

        (formatter)(f);
    }
}

impl<T> std::fmt::Debug for FormatOnce<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormatOnce").field(&"{{formatter}}").finish()
    }
}

/// Extension trait that adds JS-specific helper methods on [`JoinBuilder`].
///
/// [`JoinBuilder`] itself lives in `oxc_formatter_core` and is generic over the
/// format context, so adding inherent methods here would violate the orphan
/// rule. The trait is blanket-implemented for the JS-bound specialization.
pub trait JoinBuilderJsExt<'ast, Separator> {
    fn entries_with_trailing_separator<F, I>(
        &mut self,
        entries: I,
        separator: &'static str,
        trailing_separator: TrailingSeparator,
    ) -> &mut Self
    where
        F: Format<'ast, JsFormatContext<'ast>> + GetSpan,
        I: IntoIterator<Item = F>;
}

impl<'ast, Separator> JoinBuilderJsExt<'ast, Separator>
    for JoinBuilder<'_, '_, 'ast, Separator, JsFormatContext<'ast>>
where
    Separator: Format<'ast, JsFormatContext<'ast>>,
{
    fn entries_with_trailing_separator<F, I>(
        &mut self,
        entries: I,
        separator: &'static str,
        trailing_separator: TrailingSeparator,
    ) -> &mut Self
    where
        F: Format<'ast, JsFormatContext<'ast>> + GetSpan,
        I: IntoIterator<Item = F>,
    {
        let iter = FormatSeparatedIter::new(entries.into_iter(), separator)
            .with_trailing_separator(trailing_separator);

        for entry in iter {
            self.entry(&entry);
        }

        self
    }
}

/// Builder to join together nodes that ensures that nodes separated by empty lines continue
/// to be separated by empty lines in the formatted output.
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct JoinNodesBuilder<'fmt, 'buf, 'ast, Separator> {
    /// The separator to insert between nodes. Either a soft or hard line break
    separator: Separator,
    fmt: &'fmt mut JsFormatter<'buf, 'ast>,
    has_elements: bool,
}

impl<'fmt, 'buf, 'ast, Separator> JoinNodesBuilder<'fmt, 'buf, 'ast, Separator>
where
    Separator: Format<'ast, JsFormatContext<'ast>>,
{
    pub(super) fn new(separator: Separator, fmt: &'fmt mut JsFormatter<'buf, 'ast>) -> Self {
        Self { separator, fmt, has_elements: false }
    }

    /// Returns a reference to the formatter.
    pub fn fmt(&self) -> &JsFormatter<'buf, 'ast> {
        self.fmt
    }

    /// Returns a mutable reference to the formatter.
    pub fn fmt_mut(&mut self) -> &mut JsFormatter<'buf, 'ast> {
        self.fmt
    }

    /// Adds a new node with the specified formatted content to the output, respecting any new lines
    /// that appear before the node in the input source.
    pub fn entry(&mut self, span: Span, content: &dyn Format<'ast, JsFormatContext<'ast>>) {
        self.separator_no_entry(span);
        self.has_elements = true;
        write!(self.fmt, content);
    }

    /// Writes an entry without adding a separating line break or empty line.
    pub fn entry_no_separator(&mut self, content: &dyn Format<'ast, JsFormatContext<'ast>>) {
        self.has_elements = true;
        write!(self.fmt, content);
    }

    /// Writes a separator for before `span`, without adding an entry.
    pub fn separator_no_entry(&mut self, span: Span) {
        if self.has_elements {
            if self.has_lines_before(span) {
                write!(self.fmt, empty_line());
            } else {
                self.separator.fmt(self.fmt);
            }
        }
    }

    /// Adds an iterator of entries to the output. Each entry is a `(node, content)` tuple.
    pub fn entries<'a, F, I>(&mut self, entries: I) -> &mut Self
    where
        F: Format<'ast, JsFormatContext<'ast>> + GetSpan + 'a,
        I: IntoIterator<Item = F>,
    {
        for content in entries {
            self.entry(content.span(), &content);
        }
        self
    }

    pub fn entries_with_trailing_separator<'a, F, I>(
        &mut self,
        entries: I,
        separator: &'static str,
        trailing_separator: TrailingSeparator,
    ) -> &mut Self
    where
        F: Format<'ast, JsFormatContext<'ast>> + GetSpan + 'a,
        I: IntoIterator<Item = F>,
    {
        let iter = FormatSeparatedIter::new(entries.into_iter(), separator)
            .with_trailing_separator(trailing_separator);

        for content in iter {
            self.entry(content.span(), &content);
        }
        self
    }

    /// Get the number of line breaks between two consecutive SyntaxNodes in the tree
    pub fn has_lines_before(&self, span: Span) -> bool {
        self.fmt.source_text().get_lines_before(span, self.fmt.comments()) > 1
    }
}
