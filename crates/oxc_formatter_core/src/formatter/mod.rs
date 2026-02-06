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
pub mod builders;
mod context;
pub mod diagnostics;
pub mod format_element;
pub mod format_extensions;
pub mod formatter;
pub mod group_id;
pub mod macros;
pub mod prelude;
pub mod printer;
pub mod separated;
mod state;
mod text_range;
pub mod token;

use std::fmt::Debug;

use self::printer::AsPrinterOptions;

pub use buffer::{Buffer, BufferExtensions, VecBuffer};
pub use format_element::FormatElement;
pub use group_id::GroupId;

use self::printer::Printer;
pub use self::{
    arguments::{Argument, Arguments},
    context::{
        FormatContext, FormatContextExt, SimpleFormatContext, SimpleFormatOptions, SourceTextExt,
    },
    diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError},
    formatter::Formatter,
    state::FormatState,
    text_range::TextRange,
};
use self::{format_element::document::Document, group_id::UniqueGroupIdBuilder, prelude::TagKind};

#[derive(Debug)]
pub struct Formatted<'a, Ctx>
where
    Ctx: FormatContext<'a>,
{
    document: Document<'a>,
    context: Ctx,
}

impl<'a, Ctx> Formatted<'a, Ctx>
where
    Ctx: FormatContext<'a>,
{
    pub fn new(document: Document<'a>, context: Ctx) -> Self {
        Self { document, context }
    }

    /// Returns the context used during formatting.
    pub fn context(&self) -> &Ctx {
        &self.context
    }

    /// Returns the formatted document.
    pub fn document(&self) -> &Document<'a> {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document<'a> {
        &mut self.document
    }

    /// Consumes `self` and returns the formatted document.
    pub fn into_document(self) -> Document<'a> {
        self.document
    }
}

impl<'a, Ctx> Formatted<'a, Ctx>
where
    Ctx: FormatContext<'a>,
{
    pub fn print(self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let (elements, sorted_tailwind_classes) =
            self.document.into_elements_and_tailwind_classes();
        let printed = Printer::new(print_options, &sorted_tailwind_classes).print(elements)?;
        Ok(printed)
    }

    pub fn print_with_indent(self, indent: u16) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let (elements, sorted_tailwind_classes) =
            self.document.into_elements_and_tailwind_classes();
        let printed = Printer::new(print_options, &sorted_tailwind_classes)
            .print_with_indent(elements, indent)?;
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
pub trait Format<'ast, Ctx, T = ()>
where
    Ctx: FormatContext<'ast>,
{
    /// Formats the object using the given formatter.
    /// # Errors
    fn fmt(&self, f: &mut Formatter<'_, 'ast, Ctx>);

    /// Formats the object using the given formatter with additional options.
    /// # Errors
    fn fmt_with_options(&self, _options: T, _f: &mut Formatter<'_, 'ast, Ctx>) {
        unreachable!("Please implement it first.")
    }
}

impl<'ast, Ctx, T> Format<'ast, Ctx> for &T
where
    Ctx: FormatContext<'ast>,
    T: ?Sized + Format<'ast, Ctx>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, Ctx>) {
        Format::fmt(&**self, f);
    }
}

impl<'ast, Ctx, T> Format<'ast, Ctx> for &mut T
where
    Ctx: FormatContext<'ast>,
    T: ?Sized + Format<'ast, Ctx>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, Ctx>) {
        Format::fmt(&**self, f);
    }
}

impl<'ast, Ctx, T> Format<'ast, Ctx> for Option<T>
where
    Ctx: FormatContext<'ast>,
    T: Format<'ast, Ctx>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'ast, Ctx>) {
        if let Some(value) = self {
            value.fmt(f);
        }
    }
}

impl<'ast, Ctx> Format<'ast, Ctx> for ()
where
    Ctx: FormatContext<'ast>,
{
    #[inline]
    fn fmt(&self, _: &mut Formatter<'_, 'ast, Ctx>) {
        // Intentionally left empty
    }
}

impl<'ast, Ctx> Format<'ast, Ctx> for str
where
    Ctx: FormatContext<'ast>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, Ctx>) {
        let text = f.context().allocator().alloc_str(self);
        Format::fmt(&builders::text(text), f);
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
pub fn write<'ast, Ctx>(output: &mut dyn Buffer<'ast, Ctx>, args: Arguments<'_, 'ast, Ctx>)
where
    Ctx: FormatContext<'ast>,
{
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
pub fn format<'ast, Ctx>(context: Ctx, arguments: Arguments<'_, 'ast, Ctx>) -> Formatted<'ast, Ctx>
where
    Ctx: FormatContext<'ast>,
{
    let mut state = FormatState::new(context);
    let mut buffer = VecBuffer::new(&mut state);

    buffer.write_fmt(arguments);

    let elements = buffer.into_vec();
    let document = Document::new(elements, Vec::new());

    document.propagate_expand();

    Formatted::new(document, state.into_context())
}
