use std::{fmt, num::ParseIntError, str::FromStr};

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
    pub const fn as_bytes(self) -> &'static [u8] {
        match self {
            LineEnding::Lf => b"\n",
            LineEnding::Crlf => b"\r\n",
            LineEnding::Cr => b"\r",
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
        Self(100)
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
#[expect(clippy::enum_variant_names)]
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

pub trait IndentWidthProvider {
    fn indent_width(&self) -> IndentWidth;
}
