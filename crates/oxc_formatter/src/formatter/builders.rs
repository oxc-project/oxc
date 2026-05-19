use std::cell::Cell;

use oxc_span::{GetSpan, Span};

use Tag::{EndEntry, EndFill, StartEntry, StartFill};

use super::{
    Buffer, JsFormatContext,
    format_element::{TextWidth, tag::Tag},
    prelude::*,
    separated::FormatSeparatedIter,
};
use crate::{TrailingSeparator, write};

// Re-export the language-agnostic builders that live in `oxc_formatter_core`,
// so that legacy `crate::formatter::builders::*` call-sites keep working
// without having to be updated.
pub use oxc_formatter_core::builders::*;

/// Creates a text from a dynamic string and a range of the input source
pub fn text(text: &str) -> Text<'_> {
    debug_assert_no_cr_line_break(text);
    Text { text, width: None }
}

/// Creates a text from a dynamic string that contains no whitespace characters
pub fn text_without_whitespace(text: &str) -> Text<'_> {
    debug_assert!(
        text.as_bytes().iter().all(|&b| !b.is_ascii_whitespace()),
        "The content '{text}' contains whitespace characters but text must not contain any whitespace characters."
    );
    Text { text, width: Some(TextWidth::from_non_whitespace_str(text)) }
}

#[derive(Eq, PartialEq)]
pub struct Text<'a> {
    text: &'a str,
    width: Option<TextWidth>,
}

impl<'a> Format<'a, JsFormatContext<'a>> for Text<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.write_element(FormatElement::Text {
            text: self.text,
            width: self
                .width
                .unwrap_or_else(|| TextWidth::from_text(self.text, f.options().indent_width)),
        });
    }
}

impl std::fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "Text({})", self.text)
    }
}

/// Debug assert that the given text contains no `\r` line terminator characters.
//
// `#[inline(always)]` because this is a no-op in release mode
#[inline(always)]
#[expect(clippy::inline_always)]
#[track_caller]
fn debug_assert_no_cr_line_break(text: &str) {
    debug_assert!(
        !text.contains('\r'),
        "The content `{text}` contains an unsupported `\\r` line terminator character but text must only use line feeds `\\n` as line separator. Use `\\n` instead of `\\r` and `\\r\\n` to insert a line break in strings."
    );
}

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

/// Builder to join together a sequence of content.
/// See [Formatter::join]
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct JoinBuilder<'fmt, 'buf, 'ast, Separator> {
    fmt: &'fmt mut JsFormatter<'buf, 'ast>,
    with: Option<Separator>,
    has_elements: bool,
}

impl<'fmt, 'buf, 'ast, Separator> JoinBuilder<'fmt, 'buf, 'ast, Separator>
where
    Separator: Format<'ast, JsFormatContext<'ast>>,
{
    /// Creates a new instance that joins the elements without a separator
    pub(super) fn new(fmt: &'fmt mut JsFormatter<'buf, 'ast>) -> Self {
        Self { fmt, has_elements: false, with: None }
    }

    /// Creates a new instance that prints the passed separator between every two entries.
    pub(super) fn with_separator(fmt: &'fmt mut JsFormatter<'buf, 'ast>, with: Separator) -> Self {
        Self { fmt, has_elements: false, with: Some(with) }
    }

    /// Adds a new entry to the join output.
    pub fn entry(&mut self, entry: &dyn Format<'ast, JsFormatContext<'ast>>) -> &mut Self {
        if let Some(with) = &self.with
            && self.has_elements
        {
            with.fmt(self.fmt);
        }
        self.has_elements = true;

        entry.fmt(self.fmt);

        self
    }

    /// Adds the contents of an iterator of entries to the join output.
    pub fn entries<F, I>(&mut self, entries: I) -> &mut Self
    where
        F: Format<'ast, JsFormatContext<'ast>>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.entry(&entry);
        }

        self
    }

    pub fn entries_with_trailing_separator<F, I>(
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

/// Builder to fill as many elements as possible on a single line.
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct FillBuilder<'fmt, 'buf, 'ast> {
    fmt: &'fmt mut JsFormatter<'buf, 'ast>,
    empty: bool,
}

impl<'fmt, 'buf, 'ast> FillBuilder<'fmt, 'buf, 'ast> {
    pub(crate) fn new(fmt: &'fmt mut JsFormatter<'buf, 'ast>) -> Self {
        fmt.write_element(FormatElement::Tag(StartFill));

        Self { fmt, empty: true }
    }

    /// Adds an iterator of entries to the fill output. Uses the passed `separator` to separate any two items.
    pub fn entries<F, I>(
        &mut self,
        separator: &dyn Format<'ast, JsFormatContext<'ast>>,
        entries: I,
    ) -> &mut Self
    where
        F: Format<'ast, JsFormatContext<'ast>>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.entry(separator, &entry);
        }

        self
    }

    /// Adds a new entry to the fill output. The `separator` isn't written if this is the first element in the list.
    pub fn entry(
        &mut self,
        separator: &dyn Format<'ast, JsFormatContext<'ast>>,
        entry: &dyn Format<'ast, JsFormatContext<'ast>>,
    ) -> &mut Self {
        if self.empty {
            self.empty = false;
        } else {
            self.fmt.write_element(FormatElement::Tag(StartEntry));
            separator.fmt(self.fmt);
            self.fmt.write_element(FormatElement::Tag(EndEntry));
        }

        self.fmt.write_element(FormatElement::Tag(StartEntry));
        entry.fmt(self.fmt);
        self.fmt.write_element(FormatElement::Tag(EndEntry));

        self
    }

    /// Finishes the output and returns any error encountered
    pub fn finish(&mut self) {
        self.fmt.write_element(FormatElement::Tag(EndFill));
    }
}
