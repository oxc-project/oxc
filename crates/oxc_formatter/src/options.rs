use std::{fmt, num::ParseIntError, str::FromStr};

pub use crate::formatter::{
    Buffer, Format, FormatResult, SyntaxTriviaPieceComments, token::string::Quote,
};
use crate::{
    formatter::{
        formatter::Formatter,
        prelude::{if_group_breaks, text},
        printer::PrinterOptions,
    },
    write,
};

// TODO: rename these to align with prettier
#[derive(Debug, Default, Clone)]
pub struct FormatOptions {
    /// The indent style.
    pub indent_style: IndentStyle,

    /// The indent width.
    pub indent_width: IndentWidth,

    /// The type of line ending.
    pub line_ending: LineEnding,

    /// What's the max width of a line. Defaults to 80.
    pub line_width: LineWidth,

    /// The style for quotes. Defaults to double.
    pub quote_style: QuoteStyle,

    /// The style for JSX quotes. Defaults to double.
    pub jsx_quote_style: QuoteStyle,

    /// When properties in objects are quoted. Defaults to as-needed.
    pub quote_properties: QuoteProperties,

    /// Print trailing commas wherever possible in multi-line comma-separated syntactic structures. Defaults to "all".
    pub trailing_commas: TrailingCommas,

    /// Whether the formatter prints semicolons for all statements, class members, and type members or only when necessary because of [ASI](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-automatic-semicolon-insertion).
    pub semicolons: Semicolons,

    /// Whether to add non-necessary parentheses to arrow functions. Defaults to "always".
    pub arrow_parentheses: ArrowParentheses,

    /// Whether to insert spaces around brackets in object literals. Defaults to true.
    pub bracket_spacing: BracketSpacing,

    /// Whether to hug the closing bracket of multiline HTML/JSX tags to the end of the last line, rather than being alone on the following line. Defaults to false.
    pub bracket_same_line: BracketSameLine,

    /// Attribute position style. By default auto.
    pub attribute_position: AttributePosition,

    /// Whether to expand object and array literals to multiple lines. Defaults to "auto".
    pub expand: Expand,

    /// Controls the position of operators in binary expressions.
    /// Accepted values are:
    /// - `"start"`: Places the operator at the beginning of the next line.
    /// - `"end"`: Places the operator at the end of the current line (default).
    pub experimental_operator_position: OperatorPosition,

    // TODO: `FormatOptions`? Split out as `TransformOptions`?
    /// Sort import statements. By default disabled.
    pub experimental_sort_imports: Option<SortImports>,
}

impl FormatOptions {
    pub fn new() -> Self {
        Self {
            indent_style: IndentStyle::default(),
            indent_width: IndentWidth::default(),
            line_ending: LineEnding::default(),
            line_width: LineWidth::default(),
            quote_style: QuoteStyle::default(),
            jsx_quote_style: QuoteStyle::default(),
            quote_properties: QuoteProperties::default(),
            trailing_commas: TrailingCommas::default(),
            semicolons: Semicolons::default(),
            arrow_parentheses: ArrowParentheses::default(),
            bracket_spacing: BracketSpacing::default(),
            bracket_same_line: BracketSameLine::default(),
            attribute_position: AttributePosition::default(),
            expand: Expand::default(),
            experimental_operator_position: OperatorPosition::default(),
            experimental_sort_imports: None,
        }
    }

    pub fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::from(self)
    }
}

impl fmt::Display for FormatOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Indent style: {}", self.indent_style)?;
        writeln!(f, "Indent width: {}", self.indent_width.value())?;
        writeln!(f, "Line ending: {}", self.line_ending)?;
        writeln!(f, "Line width: {}", self.line_width.value())?;
        writeln!(f, "Quote style: {}", self.quote_style)?;
        writeln!(f, "JSX quote style: {}", self.jsx_quote_style)?;
        writeln!(f, "Quote properties: {}", self.quote_properties)?;
        writeln!(f, "Trailing commas: {}", self.trailing_commas)?;
        writeln!(f, "Semicolons: {}", self.semicolons)?;
        writeln!(f, "Arrow parentheses: {}", self.arrow_parentheses)?;
        writeln!(f, "Bracket spacing: {}", self.bracket_spacing.value())?;
        writeln!(f, "Bracket same line: {}", self.bracket_same_line.value())?;
        writeln!(f, "Attribute Position: {}", self.attribute_position)?;
        writeln!(f, "Expand lists: {}", self.expand)?;
        writeln!(f, "Experimental operator position: {}", self.experimental_operator_position)?;
        writeln!(f, "Experimental sort imports: {:?}", self.experimental_sort_imports)
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub enum IndentStyle {
    /// Tab
    Tab,
    /// Space
    #[default]
    Space,
}

impl IndentStyle {
    pub const DEFAULT_SPACES: u8 = 2;

    /// Returns `true` if this is an [IndentStyle::Tab].
    pub const fn is_tab(self) -> bool {
        matches!(self, IndentStyle::Tab)
    }

    /// Returns `true` if this is an [IndentStyle::Space].
    pub const fn is_space(self) -> bool {
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

impl fmt::Display for IndentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IndentStyle::Tab => "Tab",
            IndentStyle::Space => "Space",
        };
        f.write_str(s)
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
    pub const fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    /// Returns `true` if this is a [LineEnding::Lf].
    pub const fn is_line_feed(self) -> bool {
        matches!(self, LineEnding::Lf)
    }

    /// Returns `true` if this is a [LineEnding::Crlf].
    pub const fn is_carriage_return_line_feed(self) -> bool {
        matches!(self, LineEnding::Crlf)
    }

    /// Returns `true` if this is a [LineEnding::Cr].
    pub const fn is_carriage_return(self) -> bool {
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
            _ => Err("Value not supported for LineEnding"),
        }
    }
}

impl fmt::Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LineEnding::Lf => "LF",
            LineEnding::Crlf => "CRLF",
            LineEnding::Cr => "CR",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct IndentWidth(u8);

impl IndentWidth {
    pub const MAX: u8 = 24;
    pub const MIN: u8 = 0;

    /// Return the numeric value for this [IndentWidth]
    pub fn value(self) -> u8 {
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

impl fmt::Display for IndentWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        f.write_str(&std::format!("{value}"))
    }
}

impl fmt::Debug for IndentWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Validated value for the `line_width` formatter options
///
/// The allowed range of values is 1..=320
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LineWidth(u16);

impl LineWidth {
    /// Maximum allowed value for a valid [LineWidth]
    pub const MAX: u16 = 320;
    /// Minimum allowed value for a valid [LineWidth]
    pub const MIN: u16 = 1;

    /// Return the numeric value for this [LineWidth]
    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for LineWidth {
    fn default() -> Self {
        Self(80)
    }
}

impl fmt::Display for LineWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        f.write_str(&std::format!("{value}"))
    }
}

impl fmt::Debug for LineWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(self, f)
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

impl fmt::Debug for ParseFormatNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ParseFormatNumberError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseFormatNumberError::ParseError(err) => fmt::Display::fmt(err, fmt),
            ParseFormatNumberError::TryFromU16Error(err) => fmt::Display::fmt(err, fmt),
            ParseFormatNumberError::TryFromU8Error(err) => fmt::Display::fmt(err, fmt),
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

impl fmt::Display for IndentWidthFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "The indent width should be between {} and {}", LineWidth::MIN, LineWidth::MAX,)
    }
}

/// Error type returned when converting a u16 to a [LineWidth] fails
#[derive(Clone, Copy, Debug)]
pub struct LineWidthFromIntError(pub u16);

impl fmt::Display for LineWidthFromIntError {
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

    pub fn as_char(self) -> char {
        match self {
            QuoteStyle::Double => '"',
            QuoteStyle::Single => '\'',
        }
    }

    pub fn as_byte(self) -> u8 {
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
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            QuoteStyle::Double => QuoteStyle::Single,
            QuoteStyle::Single => QuoteStyle::Double,
        }
    }

    pub const fn is_double(self) -> bool {
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

impl fmt::Display for QuoteStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            QuoteStyle::Double => "Double Quotes",
            QuoteStyle::Single => "Single Quotes",
        };
        f.write_str(s)
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

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct TabWidth(u8);

impl From<u8> for TabWidth {
    fn from(value: u8) -> Self {
        TabWidth(value)
    }
}

impl From<TabWidth> for u8 {
    fn from(width: TabWidth) -> Self {
        width.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum QuoteProperties {
    #[default]
    AsNeeded,
    Preserve,
}

impl FromStr for QuoteProperties {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "preserve" => Ok(Self::Preserve),
            _ => Err("Value not supported for QuoteProperties"),
        }
    }
}

impl fmt::Display for QuoteProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            QuoteProperties::AsNeeded => "As needed",
            QuoteProperties::Preserve => "Preserve",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Semicolons {
    #[default]
    Always,
    AsNeeded,
}

impl Semicolons {
    pub const fn is_as_needed(self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub const fn is_always(self) -> bool {
        matches!(self, Self::Always)
    }
}

impl FromStr for Semicolons {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "always" => Ok(Self::Always),
            _ => Err(
                "Value not supported for Semicolons. Supported values are 'as-needed' and 'always'.",
            ),
        }
    }
}

impl fmt::Display for Semicolons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Semicolons::AsNeeded => "As needed",
            Semicolons::Always => "Always",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ArrowParentheses {
    #[default]
    Always,
    AsNeeded,
}

impl ArrowParentheses {
    pub const fn is_as_needed(self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub const fn is_always(self) -> bool {
        matches!(self, Self::Always)
    }
}

impl FromStr for ArrowParentheses {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "always" => Ok(Self::Always),
            _ => Err(
                "Value not supported for Arrow parentheses. Supported values are 'as-needed' and 'always'.",
            ),
        }
    }
}

impl fmt::Display for ArrowParentheses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ArrowParentheses::AsNeeded => "As needed",
            ArrowParentheses::Always => "Always",
        };
        f.write_str(s)
    }
}

/// This enum is used within formatting functions to print or omit trailing commas.
#[derive(Debug, Copy, Clone)]
pub enum FormatTrailingCommas {
    /// Print trailing commas if the option is [TrailingCommas::All].
    All,
    /// Print trailing commas if the option is [TrailingCommas::All] or [TrailingCommas::Es5].
    ES5,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum TrailingSeparator {
    /// A trailing separator is allowed and preferred
    #[default]
    Allowed,

    /// A trailing separator is not allowed
    Disallowed,

    /// A trailing separator is mandatory for the syntax to be correct
    Mandatory,

    /// A trailing separator might be present, but the consumer
    /// decides to remove it
    Omit,
}

impl FormatTrailingCommas {
    /// This function returns corresponding [TrailingSeparator] for `format_separated` function.
    pub fn trailing_separator(self, options: &FormatOptions) -> TrailingSeparator {
        if options.trailing_commas.is_none() {
            return TrailingSeparator::Omit;
        }

        match self {
            FormatTrailingCommas::All => {
                if options.trailing_commas.is_all() {
                    TrailingSeparator::Allowed
                } else {
                    TrailingSeparator::Omit
                }
            }
            FormatTrailingCommas::ES5 => TrailingSeparator::Allowed,
        }
    }
}

impl Format<'_> for FormatTrailingCommas {
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        if f.options().trailing_commas.is_none() {
            return Ok(());
        }

        if matches!(self, FormatTrailingCommas::ES5) || f.options().trailing_commas.is_all() {
            write!(f, [if_group_breaks(&text(","))])?;
        }

        Ok(())
    }
}

/// Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
#[derive(Clone, Copy, Default, Debug, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Trailing commas wherever possible (including function parameters and calls).
    #[default]
    All,
    /// Trailing commas where valid in ES5 (objects, arrays, etc.). No trailing commas in type parameters in TypeScript.
    Es5,
    /// No trailing commas.
    None,
}

impl TrailingCommas {
    pub const fn is_es5(self) -> bool {
        matches!(self, TrailingCommas::Es5)
    }

    pub const fn is_all(self) -> bool {
        matches!(self, TrailingCommas::All)
    }

    pub const fn is_none(self) -> bool {
        matches!(self, TrailingCommas::None)
    }
}

impl FromStr for TrailingCommas {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "es5" => Ok(Self::Es5),
            "all" => Ok(Self::All),
            "none" => Ok(Self::None),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for TrailingCommas"),
        }
    }
}

impl fmt::Display for TrailingCommas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TrailingCommas::Es5 => "ES5",
            TrailingCommas::All => "All",
            TrailingCommas::None => "None",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AttributePosition {
    #[default]
    Auto,
    Multiline,
}

impl fmt::Display for AttributePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AttributePosition::Auto => "Auto",
            AttributePosition::Multiline => "Multiline",
        };
        f.write_str(s)
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BracketSpacing(bool);

impl BracketSpacing {
    /// Return the boolean value for this [BracketSpacing]
    pub fn value(self) -> bool {
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

impl fmt::Display for BracketSpacing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.value(), f)
    }
}

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

/// Put the `>` of a multi-line HTML or JSX element at the end of the last line instead of being alone on the next line (does not apply to self closing elements).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BracketSameLine(bool);

impl BracketSameLine {
    /// Return the boolean value for this [BracketSameLine]
    pub fn value(self) -> bool {
        self.0
    }
}

impl From<bool> for BracketSameLine {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl fmt::Display for BracketSameLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.value(), f)
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
        let s = match self {
            Expand::Auto => "Auto",
            Expand::Always => "Always",
            Expand::Never => "Never",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum OperatorPosition {
    /// When binary expressions wrap lines, print operators at the start of new lines.
    Start,

    // Default behavior; when binary expressions wrap lines, print operators at the end of previous lines.
    #[default]
    End,
}

impl OperatorPosition {
    pub const fn is_start(self) -> bool {
        matches!(self, Self::Start)
    }

    pub const fn is_end(self) -> bool {
        matches!(self, Self::End)
    }
}

impl FromStr for OperatorPosition {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            _ => Err("Value not supported for OperatorPosition"),
        }
    }
}

impl fmt::Display for OperatorPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            OperatorPosition::Start => "Start",
            OperatorPosition::End => "End",
        };
        f.write_str(s)
    }
}

// ---

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SortImports {
    /// Partition imports by newlines.
    /// Default is `false`.
    pub partition_by_newline: bool,
    /// Partition imports by comments.
    /// Default is `false`.
    pub partition_by_comment: bool,
    /// Sort side effects imports.
    /// Default is `false`.
    pub sort_side_effects: bool,
    /// Sort order (asc or desc).
    /// Default is ascending (asc).
    pub order: SortOrder,
    /// Ignore case when sorting.
    /// Default is `true`.
    pub ignore_case: bool,
}

impl Default for SortImports {
    fn default() -> Self {
        Self {
            partition_by_newline: false,
            partition_by_comment: false,
            sort_side_effects: false,
            order: SortOrder::default(),
            ignore_case: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SortOrder {
    /// Sort in ascending order (A-Z).
    #[default]
    Asc,
    /// Sort in descending order (Z-A).
    Desc,
}

impl SortOrder {
    pub const fn is_asc(self) -> bool {
        matches!(self, Self::Asc)
    }

    pub const fn is_desc(self) -> bool {
        matches!(self, Self::Desc)
    }
}

impl FromStr for SortOrder {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err("Value not supported for SortOrder. Supported values are 'asc' and 'desc'."),
        }
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };
        f.write_str(s)
    }
}
