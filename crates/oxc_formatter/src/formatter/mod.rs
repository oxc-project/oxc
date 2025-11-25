//! Infrastructure for code formatting
//!
//! This module defines [FormatElement], an IR to format code documents and provides a mean to print
//! such a document to a string. Objects that know how to format themselves implement the [Format] trait.
//!
//! ## Formatting Traits
//!
//! * [Format]: Implemented by objects that can be formatted.
//!
//! ## Formatting Macros
//!
//! This crate defines two macros to construct the IR. These are inspired by Rust's `fmt` macros
//! * [`format!`]: Formats a formattable object
//! * [`format_args!`]: Concatenates a sequence of Format objects.
//! * [`write!`]: Writes a sequence of formattable objects into an output buffer.

// FIXME
#![allow(rustdoc::broken_intra_doc_links)]

mod arguments;
pub mod buffer;
mod builders;
pub mod comments;
mod context;
pub mod diagnostics;
pub mod format_element;
mod format_extensions;
pub mod formatter;
pub mod group_id;
pub mod macros;
pub mod prelude;
pub mod printer;
pub mod separated;
mod source_text;
mod state;
mod text_range;
pub mod token;
pub mod trivia;

use std::fmt::Debug;

pub use buffer::{Buffer, BufferExtensions, VecBuffer};
pub use format_element::FormatElement;
pub use group_id::GroupId;

pub use self::comments::Comments;
use self::printer::Printer;
pub use self::{
    arguments::{Argument, Arguments},
    context::FormatContext,
    diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError},
    formatter::Formatter,
    source_text::SourceText,
    state::FormatState,
    text_range::TextRange,
};
use self::{format_element::document::Document, group_id::UniqueGroupIdBuilder, prelude::TagKind};

#[derive(Debug, Clone)]
pub struct Formatted<'a> {
    document: Document<'a>,
    context: FormatContext<'a>,
}

impl<'a> Formatted<'a> {
    pub fn new(document: Document<'a>, context: FormatContext<'a>) -> Self {
        Self { document, context }
    }

    /// Returns the context used during formatting.
    pub fn context(&self) -> &FormatContext<'a> {
        &self.context
    }

    /// Returns the formatted document.
    pub fn document(&self) -> &Document<'a> {
        &self.document
    }

    /// Consumes `self` and returns the formatted document.
    pub fn into_document(self) -> Document<'a> {
        self.document
    }
}

impl<'a> Formatted<'a> {
    pub fn apply_transform(&mut self, transform: impl FnOnce(&Document<'a>) -> Document<'a>) {
        self.document = transform(&self.document);
    }
}

impl Formatted<'_> {
    pub fn print(&self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();

        let printed = Printer::new(print_options).print(&self.document)?;

        Ok(printed)
    }

    pub fn print_with_indent(&self, indent: u16) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let printed = Printer::new(print_options).print_with_indent(&self.document, indent)?;

        Ok(printed)
    }
}
pub type PrintResult<T> = Result<T, PrintError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Printed {
    code: String,
    range: Option<TextRange>,
}

impl Printed {
    pub fn new(code: String, range: Option<TextRange>) -> Self {
        Self { code, range }
    }

    /// Construct an empty formatter result
    pub fn new_empty() -> Self {
        Self { code: String::new(), range: None }
    }

    /// Range of the input source file covered by this formatted code,
    /// or None if the entire file is covered in this instance
    pub fn range(&self) -> Option<TextRange> {
        self.range
    }

    /// Access the resulting code, borrowing the result
    pub fn as_code(&self) -> &str {
        &self.code
    }

    /// Access the resulting code, consuming the result
    pub fn into_code(self) -> String {
        self.code
    }
}

// Public return type of the formatter
pub type FormatResult<F> = Result<F, FormatError>;

/// Formatting trait for types that can create a formatted representation. The `biome_formatter` equivalent
/// to [std::fmt::Display].
///
/// ## Example
/// Implementing `Format` for a custom struct
///
/// ```
/// use biome_formatter::{format, write, IndentStyle, LineWidth};
/// use biome_formatter::prelude::*;
/// use biome_rowan::TextSize;
///
/// struct Paragraph(String);
///
/// impl Format<SimpleFormatContext> for Paragraph {
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>)  {
///         write!(f, [
///             hard_line_break(),
///             text(&self.0, TextSize::from(0)),
///             hard_line_break(),
///         ])
///     }
/// }
///
/// # fn main()  {
/// let paragraph = Paragraph(String::from("test"));
/// let formatted = format!(SimpleFormatContext::default(), [paragraph])?;
///
/// assert_eq!("test\n", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub trait Format<'ast, T = ()> {
    /// Formats the object using the given formatter.
    /// # Errors
    fn fmt(&self, f: &mut Formatter<'_, 'ast>);

    /// Formats the object using the given formatter with additional options.
    /// # Errors
    fn fmt_with_options(&self, _options: T, _f: &mut Formatter<'_, 'ast>) {
        unreachable!("Please implement it first.")
    }
}

impl<'ast, T> Format<'ast> for &T
where
    T: ?Sized + Format<'ast>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) {
        Format::fmt(&**self, f);
    }
}

impl<'ast, T> Format<'ast> for &mut T
where
    T: ?Sized + Format<'ast>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) {
        Format::fmt(&**self, f);
    }
}

impl<'ast, T> Format<'ast> for Option<T>
where
    T: Format<'ast>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) {
        if let Some(value) = self {
            value.fmt(f);
        }
    }
}

impl Format<'_> for () {
    #[inline]
    fn fmt(&self, _: &mut Formatter) {
        // Intentionally left empty
    }
}

impl Format<'_> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut Formatter) {
        crate::write!(f, builders::token(self));
    }
}

/// The `write` function takes a target buffer and an `Arguments` struct that can be precompiled with the `format_args!` macro.
///
/// The arguments will be formatted in-order into the output buffer provided.
///
/// # Examples
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{VecBuffer, format_args, FormatState, write, Formatted};
///
/// # fn main()  {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [format_args!(token("Hello World"))])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// Please note that using [`write!`] might be preferable. Example:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{VecBuffer, format_args, FormatState, write, Formatted};
///
/// # fn main()  {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [token("Hello World")])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
#[inline(always)]
pub fn write<'ast>(output: &mut dyn Buffer<'ast>, args: Arguments<'_, 'ast>) {
    Formatter::new(output).write_fmt(args);
}

/// The `format` function takes an [`Arguments`] struct and returns the resulting formatting IR.
///
/// The [`Arguments`] instance can be created with the [`format_args!`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format, format_args};
///
/// # fn main()  {
/// let formatted = format!(SimpleFormatContext::default(), [&format_args!(token("test"))])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// Please note that using [`format!`] might be preferable. Example:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format};
///
/// # fn main()  {
/// let formatted = format!(SimpleFormatContext::default(), [token("test")])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub fn format<'ast>(
    context: FormatContext<'ast>,
    arguments: Arguments<'_, 'ast>,
) -> Formatted<'ast> {
    // Pre-allocate buffer at 40% of source length (source_len * 2 / 5).
    // Analysis of 4,891 VSCode files shows FormatElement buffer length is typically 19% of source (median),
    // with 95th percentile at 30-38% across all file sizes. This 0.4x multiplier avoids
    // reallocation for 95%+ of files.
    let capacity = (context.source_text().len() * 2) / 5;

    let mut state = FormatState::new(context);
    let mut buffer = VecBuffer::with_capacity(capacity, &mut state);

    buffer.write_fmt(arguments);

    let document = Document::from(buffer.into_vec());
    document.propagate_expand();

    Formatted::new(document, state.into_context())
}
