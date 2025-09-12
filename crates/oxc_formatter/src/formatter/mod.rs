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
#[cfg(debug_assertions)]
pub mod printed_tokens;
pub mod printer;
pub mod separated;
mod source_text;
mod state;
mod syntax_element_key;
mod syntax_node;
mod syntax_token;
mod syntax_trivia_piece_comments;
mod text_len;
mod text_range;
mod text_size;
pub mod token;
mod token_text;
pub mod trivia;
mod verbatim;

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub use buffer::{Buffer, BufferExtensions, VecBuffer};
pub use format_element::FormatElement;
pub use group_id::GroupId;
use oxc_allocator::{Address, GetAddress};
use oxc_ast::{AstKind, ast::Program};
use rustc_hash::FxHashMap;

pub use self::comments::Comments;
use self::printer::Printer;
pub use self::{
    arguments::{Argument, Arguments},
    context::FormatContext,
    diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError},
    formatter::Formatter,
    source_text::SourceText,
    state::{FormatState, FormatStateSnapshot},
    syntax_node::SyntaxNode,
    syntax_token::SyntaxToken,
    syntax_trivia_piece_comments::SyntaxTriviaPieceComments,
    text_len::TextLen,
    text_range::TextRange,
    text_size::TextSize,
    token_text::TokenText,
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

impl Formatted<'_> {
    pub fn print(&self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();

        let printed = Printer::new(print_options).print(&self.document)?;

        // let printed = match self.context.source_map() {
        // Some(source_map) => source_map.map_printed(printed),
        // None => printed,
        // };

        Ok(printed)
    }

    pub fn print_with_indent(&self, indent: u16) -> PrintResult<Printed> {
        todo!()
        // let print_options = self.context.options().as_print_options();
        // let printed = Printer::new(print_options).print_with_indent(&self.document, indent)?;

        // let printed = match self.context.source_map() {
        // Some(source_map) => source_map.map_printed(printed),
        // None => printed,
        // };

        // Ok(printed)
    }
}
pub type PrintResult<T> = Result<T, PrintError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Printed {
    code: String,
    range: Option<TextRange>,
    verbatim_ranges: Vec<TextRange>,
}

impl Printed {
    pub fn new(code: String, range: Option<TextRange>, verbatim_source: Vec<TextRange>) -> Self {
        Self { code, range, verbatim_ranges: verbatim_source }
    }

    /// Construct an empty formatter result
    pub fn new_empty() -> Self {
        Self { code: String::new(), range: None, verbatim_ranges: Vec::new() }
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

    /// The text in the formatted code that has been formatted as verbatim.
    pub fn verbatim(&self) -> impl Iterator<Item = (TextRange, &str)> {
        panic!();
        std::iter::empty()
        // self.verbatim_ranges.iter().map(|range| (*range, &self.code[*range]))
    }

    /// Ranges of the formatted code that have been formatted as verbatim.
    pub fn verbatim_ranges(&self) -> &[TextRange] {
        &self.verbatim_ranges
    }

    /// Takes the ranges of nodes that have been formatted as verbatim, replacing them with an empty list.
    pub fn take_verbatim_ranges(&mut self) -> Vec<TextRange> {
        std::mem::take(&mut self.verbatim_ranges)
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
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>) -> FormatResult<()> {
///         write!(f, [
///             hard_line_break(),
///             dynamic_text(&self.0, TextSize::from(0)),
///             hard_line_break(),
///         ])
///     }
/// }
///
/// # fn main() -> FormatResult<()> {
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
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;

    /// Formats the object using the given formatter with additional options.
    /// # Errors
    fn fmt_with_options(&self, options: T, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        unreachable!("Please implement it first.")
    }
}

impl<'ast, T> Format<'ast> for &T
where
    T: ?Sized + Format<'ast>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<'ast, T> Format<'ast> for &mut T
where
    T: ?Sized + Format<'ast>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<'ast, T> Format<'ast> for Option<T>
where
    T: Format<'ast>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        match self {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

impl Format<'_> for () {
    #[inline]
    fn fmt(&self, _: &mut Formatter) -> FormatResult<()> {
        // Intentionally left empty
        Ok(())
    }
}

impl Format<'_> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        crate::write!(f, builders::text(self))
    }
}

/// Default implementation for formatting a token
pub struct FormatToken<C> {
    context: PhantomData<C>,
}

impl<C> Default for FormatToken<C> {
    fn default() -> Self {
        Self { context: PhantomData }
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
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [format_args!(text("Hello World"))])?;
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
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [text("Hello World")])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
#[inline(always)]
pub fn write<'ast>(output: &mut dyn Buffer<'ast>, args: Arguments<'_, 'ast>) -> FormatResult<()> {
    Formatter::new(output).write_fmt(args)
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
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [&format_args!(text("test"))])?;
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
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [text("test")])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub fn format<'ast>(
    program: &'ast Program<'ast>,
    context: FormatContext<'ast>,
    arguments: Arguments<'_, 'ast>,
) -> FormatResult<Formatted<'ast>> {
    let mut state = FormatState::new(program, context);
    let mut buffer = VecBuffer::with_capacity(arguments.items().len(), &mut state);

    buffer.write_fmt(arguments)?;

    let mut document = Document::from(buffer.into_vec());
    document.propagate_expand();

    Ok(Formatted::new(document, state.into_context()))
}
