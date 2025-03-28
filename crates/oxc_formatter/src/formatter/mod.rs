//! Infrastructure for code formatting
//!
//! This module defines [FormatElement], an IR to format code documents and provides a mean to print
//! such a document to a string. Objects that know how to format themselves implement the [Format] trait.
//!
//! ## Formatting Traits
//!
//! * [Format]: Implemented by objects that can be formatted.
//! * [FormatRule]: Rule that knows how to format an object of another type. Necessary in the situation where
//!     it's necessary to implement [Format] on an object from another crate. This module defines the
//!     [FormatRefWithRule] and [FormatOwnedWithRule] structs to pass an item with its corresponding rule.
//! * [FormatWithRule] implemented by objects that know how to format another type. Useful for implementing
//!     some reusable formatting logic inside of this module if the type itself doesn't implement [Format]
//!
//! ## Formatting Macros
//!
//! This crate defines two macros to construct the IR. These are inspired by Rust's `fmt` macros
//! * [`format!`]: Formats a formattable object
//! * [`format_args!`]: Concatenates a sequence of Format objects.
//! * [`write!`]: Writes a sequence of formattable objects into an output buffer.

#![deny(rustdoc::broken_intra_doc_links)]

mod arguments;
mod buffer;
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

pub use self::arguments::{Argument, Arguments};
pub use self::formatter::Formatter;
use self::group_id::UniqueGroupIdBuilder;
use self::prelude::TagKind;
pub use self::state::{FormatState, FormatStateSnapshot};
pub use self::syntax_trivia_piece_comments::SyntaxTriviaPieceComments;
use std::fmt;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
// use self::builders::syntax_token_cow_slice;
pub use self::comments::{
    CommentKind, CommentPlacement, CommentStyle, Comments, DecoratedComment, SourceComment,
};
pub use self::context::FormatContext;
pub use self::diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError};
use self::format_element::document::Document;
#[cfg(debug_assertions)]
// use self::printed_tokens::PrintedTokens;
use self::printer::{Printer, PrinterOptions};
pub use self::syntax_element_key::SyntaxElementKey;
pub use self::syntax_node::SyntaxNode;
pub use self::syntax_token::SyntaxToken;
pub use self::text_len::TextLen;
pub use self::text_range::TextRange;
pub use self::text_size::TextSize;
pub use self::token_text::TokenText;
// use self::trivia::{format_skipped_token_trivia, format_trimmed_token};
pub use buffer::{
    Buffer, BufferExtensions, BufferSnapshot, Inspect, PreambleBuffer, RemoveSoftLinesBuffer,
    VecBuffer,
};
pub use builders::BestFitting;
pub use format_element::{FormatElement, LINE_TERMINATORS, normalize_newlines};
pub use group_id::GroupId;
use oxc_ast::ast::Program;
// use std::marker::PhantomData;
use std::num::ParseIntError;
use std::str::FromStr;
use token::string::Quote;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub enum IndentStyle {
    /// Tab
    #[default]
    Tab,
    /// Space
    Space,
}

impl IndentStyle {
    pub const DEFAULT_SPACES: u8 = 2;

    /// Returns `true` if this is an [IndentStyle::Tab].
    pub const fn is_tab(&self) -> bool {
        matches!(self, IndentStyle::Tab)
    }

    /// Returns `true` if this is an [IndentStyle::Space].
    pub const fn is_space(&self) -> bool {
        matches!(self, IndentStyle::Space)
    }
}

impl FromStr for IndentStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tab" => Ok(Self::Tab),
            "space" => Ok(Self::Space),
            // TODO: replace this error with a diagnostic
            _ => Err("Unsupported value for this option"),
        }
    }
}

impl Display for IndentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndentStyle::Tab => std::write!(f, "Tab"),
            IndentStyle::Space => std::write!(f, "Space"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub enum LineEnding {
    ///  Line Feed only (\n), common on Linux and macOS as well as inside git repos
    #[default]
    Lf,

    /// Carriage Return + Line Feed characters (\r\n), common on Windows
    Crlf,

    /// Carriage Return character only (\r), used very rarely
    Cr,
}

impl LineEnding {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    /// Returns `true` if this is a [LineEnding::Lf].
    pub const fn is_line_feed(&self) -> bool {
        matches!(self, LineEnding::Lf)
    }

    /// Returns `true` if this is a [LineEnding::Crlf].
    pub const fn is_carriage_return_line_feed(&self) -> bool {
        matches!(self, LineEnding::Crlf)
    }

    /// Returns `true` if this is a [LineEnding::Cr].
    pub const fn is_carriage_return(&self) -> bool {
        matches!(self, LineEnding::Cr)
    }
}

impl FromStr for LineEnding {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lf" => Ok(Self::Lf),
            "crlf" => Ok(Self::Crlf),
            "cr" => Ok(Self::Cr),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for LineEnding"),
        }
    }
}

impl std::fmt::Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineEnding::Lf => std::write!(f, "LF"),
            LineEnding::Crlf => std::write!(f, "CRLF"),
            LineEnding::Cr => std::write!(f, "CR"),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct IndentWidth(u8);

impl IndentWidth {
    pub const MIN: u8 = 0;

    pub const MAX: u8 = 24;

    /// Return the numeric value for this [IndentWidth]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for IndentWidth {
    fn default() -> Self {
        Self(2)
    }
}

impl FromStr for IndentWidth {
    type Err = ParseFormatNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = u8::from_str(s).map_err(ParseFormatNumberError::ParseError)?;
        let value = Self::try_from(value).map_err(ParseFormatNumberError::TryFromU8Error)?;
        Ok(value)
    }
}

impl TryFrom<u8> for IndentWidth {
    type Error = IndentWidthFromIntError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(IndentWidthFromIntError(value))
        }
    }
}

// impl biome_console::fmt::Display for IndentWidth {
// fn fmt(&self, fmt: &mut biome_console::fmt::Formatter) -> std::io::Result<()> {
// fmt.write_markup(markup! {{self.value()}})
// }
// }

impl Display for IndentWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        f.write_str(&std::format!("{value}"))
    }
}

impl Debug for IndentWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Validated value for the `line_width` formatter options
///
/// The allowed range of values is 1..=320
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LineWidth(u16);

impl LineWidth {
    /// Minimum allowed value for a valid [LineWidth]
    pub const MIN: u16 = 1;
    /// Maximum allowed value for a valid [LineWidth]
    pub const MAX: u16 = 320;

    /// Return the numeric value for this [LineWidth]
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl Default for LineWidth {
    fn default() -> Self {
        Self(80)
    }
}

impl Display for LineWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        f.write_str(&std::format!("{value}"))
    }
}

impl Debug for LineWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Error type returned when parsing a [LineWidth] or [IndentWidth] from a string fails
pub enum ParseFormatNumberError {
    /// The string could not be parsed to a number
    ParseError(ParseIntError),
    /// The `u16` value of the string is not a valid [LineWidth]
    TryFromU16Error(LineWidthFromIntError),
    /// The `u8 value of the string is not a valid [IndentWidth]
    TryFromU8Error(IndentWidthFromIntError),
}

impl From<IndentWidthFromIntError> for ParseFormatNumberError {
    fn from(value: IndentWidthFromIntError) -> Self {
        Self::TryFromU8Error(value)
    }
}

impl From<LineWidthFromIntError> for ParseFormatNumberError {
    fn from(value: LineWidthFromIntError) -> Self {
        Self::TryFromU16Error(value)
    }
}

impl From<ParseIntError> for ParseFormatNumberError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(value)
    }
}

impl Debug for ParseFormatNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for ParseFormatNumberError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseFormatNumberError::ParseError(err) => std::fmt::Display::fmt(err, fmt),
            ParseFormatNumberError::TryFromU16Error(err) => std::fmt::Display::fmt(err, fmt),
            ParseFormatNumberError::TryFromU8Error(err) => std::fmt::Display::fmt(err, fmt),
        }
    }
}

impl TryFrom<u16> for LineWidth {
    type Error = LineWidthFromIntError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(LineWidthFromIntError(value))
        }
    }
}

impl FromStr for LineWidth {
    type Err = ParseFormatNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = u16::from_str(s).map_err(ParseFormatNumberError::ParseError)?;
        let value = Self::try_from(value).map_err(ParseFormatNumberError::TryFromU16Error)?;
        Ok(value)
    }
}

/// Error type returned when converting a u16 to a [LineWidth] fails
#[derive(Clone, Copy, Debug)]
pub struct IndentWidthFromIntError(pub u8);

impl std::fmt::Display for IndentWidthFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "The indent width should be between {} and {}", LineWidth::MIN, LineWidth::MAX,)
    }
}

/// Error type returned when converting a u16 to a [LineWidth] fails
#[derive(Clone, Copy, Debug)]
pub struct LineWidthFromIntError(pub u16);

impl std::fmt::Display for LineWidthFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "The line width should be between {} and {}", LineWidth::MIN, LineWidth::MAX,)
    }
}

impl From<LineWidth> for u16 {
    fn from(value: LineWidth) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum QuoteStyle {
    #[default]
    Double,
    Single,
}

impl QuoteStyle {
    pub fn from_byte(byte: u8) -> Option<QuoteStyle> {
        match byte {
            b'"' => Some(QuoteStyle::Double),
            b'\'' => Some(QuoteStyle::Single),
            _ => None,
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            QuoteStyle::Double => '"',
            QuoteStyle::Single => '\'',
        }
    }

    pub fn as_byte(&self) -> u8 {
        self.as_char() as u8
    }

    /// Returns the quote in HTML entity
    pub fn as_html_entity(&self) -> &str {
        match self {
            QuoteStyle::Double => "&quot;",
            QuoteStyle::Single => "&apos;",
        }
    }

    /// Given the current quote, it returns the other one
    pub fn other(&self) -> Self {
        match self {
            QuoteStyle::Double => QuoteStyle::Single,
            QuoteStyle::Single => QuoteStyle::Double,
        }
    }

    pub const fn is_double(&self) -> bool {
        matches!(self, Self::Double)
    }
}

impl FromStr for QuoteStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "double" => Ok(Self::Double),
            "single" => Ok(Self::Single),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for QuoteStyle"),
        }
    }
}

impl std::fmt::Display for QuoteStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteStyle::Double => std::write!(f, "Double Quotes"),
            QuoteStyle::Single => std::write!(f, "Single Quotes"),
        }
    }
}

impl From<QuoteStyle> for Quote {
    fn from(quote: QuoteStyle) -> Self {
        match quote {
            QuoteStyle::Double => Self::Double,
            QuoteStyle::Single => Self::Single,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BracketSpacing(bool);

impl BracketSpacing {
    /// Return the boolean value for this [BracketSpacing]
    pub fn value(&self) -> bool {
        self.0
    }
}

impl Default for BracketSpacing {
    fn default() -> Self {
        Self(true)
    }
}

impl From<bool> for BracketSpacing {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for BracketSpacing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{:?}", self.value())
    }
}

// impl biome_console::fmt::Display for BracketSpacing {
// fn fmt(&self, fmt: &mut biome_console::fmt::Formatter) -> std::io::Result<()> {
// fmt.write_str(&self.0.to_string())
// }
// }

impl FromStr for BracketSpacing {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = bool::from_str(s);

        match value {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(
                "Value not supported for BracketSpacing. Supported values are 'true' and 'false'.",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AttributePosition {
    #[default]
    Auto,
    Multiline,
}

impl std::fmt::Display for AttributePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributePosition::Auto => std::write!(f, "Auto"),
            AttributePosition::Multiline => std::write!(f, "Multiline"),
        }
    }
}

impl FromStr for AttributePosition {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "multiline" => Ok(Self::Multiline),
            "auto" => Ok(Self::Auto),
            _ => Err(
                "Value not supported for attribute_position. Supported values are 'auto' and 'multiline'.",
            ),
        }
    }
}

/// Put the `>` of a multi-line HTML or JSX element at the end of the last line instead of being alone on the next line (does not apply to self closing elements).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BracketSameLine(bool);

impl BracketSameLine {
    /// Return the boolean value for this [BracketSameLine]
    pub fn value(&self) -> bool {
        self.0
    }
}

impl From<bool> for BracketSameLine {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for BracketSameLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}", self.value())
    }
}

impl FromStr for BracketSameLine {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match bool::from_str(s) {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(
                "Value not supported for BracketSameLine. Supported values are 'true' and 'false'.",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Expand {
    /// Objects are expanded when the first property has a leading newline. Arrays are always
    /// expanded if they are shorter than the line width.
    #[default]
    Auto,
    /// Objects and arrays are always expanded.
    Always,
    /// Objects and arrays are never expanded, if they are shorter than the line width.
    Never,
}

impl FromStr for Expand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(std::format!("unknown expand literal: {s}")),
        }
    }
}

impl fmt::Display for Expand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expand::Auto => std::write!(f, "Auto"),
            Expand::Always => std::write!(f, "Always"),
            Expand::Never => std::write!(f, "Never"),
        }
    }
}

/// Options customizing how the source code should be formatted.
///
/// **Note**: This trait should **only** contain the essential abstractions required for the printing phase.
/// For example, do not add a `fn bracket_spacing(&self) -> BracketSpacing` method here,
/// as the [BracketSpacing] option is not needed during the printing phase
/// and enforcing its implementation for all structs using this trait is unnecessary.
pub trait FormatOptions {
    /// The indent style.
    fn indent_style(&self) -> IndentStyle;

    /// The indent width.
    fn indent_width(&self) -> IndentWidth;

    /// What's the max width of a line. Defaults to 80.
    fn line_width(&self) -> LineWidth;

    /// The type of line ending.
    fn line_ending(&self) -> LineEnding;

    /// Derives the print options from the these format options
    fn as_print_options(&self) -> PrinterOptions;
}

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct SimpleFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
}

impl FormatOptions for SimpleFormatOptions {
    fn indent_style(&self) -> IndentStyle {
        self.indent_style
    }

    fn indent_width(&self) -> IndentWidth {
        self.indent_width
    }

    fn line_width(&self) -> LineWidth {
        self.line_width
    }

    fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::default()
            .with_indent_style(self.indent_style)
            .with_indent_width(self.indent_width)
            .with_print_width(self.line_width.into())
            .with_line_ending(self.line_ending)
    }
}

impl Display for SimpleFormatOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Lightweight sourcemap marker between source and output tokens
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SourceMarker {
    /// Position of the marker in the original source
    pub source: TextSize,
    /// Position of the marker in the output code
    pub dest: TextSize,
}

#[derive(Debug, Clone)]
pub struct Formatted<'a> {
    document: Document,
    context: FormatContext<'a>,
}

impl<'a> Formatted<'a> {
    pub fn new(document: Document, context: FormatContext<'a>) -> Self {
        Self { document, context }
    }

    /// Returns the context used during formatting.
    pub fn context(&self) -> &FormatContext<'a> {
        &self.context
    }

    /// Returns the formatted document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Consumes `self` and returns the formatted document.
    pub fn into_document(self) -> Document {
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
pub trait Format<'ast> {
    // Formats the object using the given formatter.
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
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

pub trait FormatRule<T> {
    fn fmt(&self, item: &T, f: &mut Formatter) -> FormatResult<()>;
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

pub trait FormatWithRule<'ast>: Format<'ast> {
    type Item;

    /// Returns the associated item
    fn item(&self) -> &Self::Item;
}

/// Formats the referenced `item` with the specified rule.
#[derive(Debug, Copy, Clone)]
pub struct FormatRefWithRule<'b, T, R>
where
    R: FormatRule<T>,
{
    item: &'b T,
    rule: R,
}

impl<'b, T, R> FormatRefWithRule<'b, T, R>
where
    R: FormatRule<T>,
{
    pub fn new(item: &'b T, rule: R) -> Self {
        Self { item, rule }
    }
}

// impl<T, R, O> FormatRefWithRule<'_, T, R>
// where
// R: FormatRuleWithOptions<T, Options = O>,
// {
// pub fn with_options(mut self, options: O) -> Self {
// self.rule = self.rule.with_options(options);
// self
// }
// }

impl<T, R> FormatWithRule<'_> for FormatRefWithRule<'_, T, R>
where
    R: FormatRule<T>,
{
    type Item = T;

    fn item(&self) -> &Self::Item {
        self.item
    }
}

impl<T, R> Format<'_> for FormatRefWithRule<'_, T, R>
where
    R: FormatRule<T>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        self.rule.fmt(self.item, f)
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
    let mut f = Formatter::new(output);

    f.write_fmt(args)
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
