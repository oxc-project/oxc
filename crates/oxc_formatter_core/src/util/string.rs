//! Language-agnostic string-literal quote normalization, shared by every
//! language the formatter targets.

use std::{borrow::Cow, fmt, str::FromStr};

/// Which ASCII quote character delimits a string literal.
///
/// JS / TS / JSON / JSON5 all distinguish exactly two quote styles, so this
/// enum lives in `oxc_formatter_core` and is re-aliased per-language (e.g.
/// `oxc_formatter::QuoteStyle = Quote`).
#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Quote {
    #[default]
    Double,
    Single,
}

impl Quote {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'"' => Some(Self::Double),
            b'\'' => Some(Self::Single),
            _ => None,
        }
    }

    pub const fn as_char(self) -> char {
        match self {
            Self::Double => '"',
            Self::Single => '\'',
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Double => "\"",
            Self::Single => "'",
        }
    }

    pub const fn as_byte(self) -> u8 {
        self.as_char() as u8
    }

    /// Returns the quote in HTML-entity form (`&quot;` / `&apos;`).
    pub const fn as_html_entity(self) -> &'static str {
        match self {
            Self::Double => "&quot;",
            Self::Single => "&apos;",
        }
    }

    /// Returns the opposite quote.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Double => Self::Single,
            Self::Single => Self::Double,
        }
    }

    pub const fn is_double(self) -> bool {
        matches!(self, Self::Double)
    }
}

impl FromStr for Quote {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "double" => Ok(Self::Double),
            "single" => Ok(Self::Single),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for Quote"),
        }
    }
}

impl fmt::Display for Quote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Double => "Double",
            Self::Single => "Single",
        })
    }
}

/// Normalizes a string-literal body for the chosen `preferred_quote`:
///
/// - Escapes any unescaped occurrence of `preferred_quote`
/// - Unescapes the alternate quote (`preferred_quote.other()`) when
///   `quotes_will_change` is `true` — i.e. the outer delimiter is switching
///   so previously-escaped chars no longer need escaping
/// - Normalizes line endings (`\r\n` and lone `\r`) to `\n`
///
/// Returns `Cow::Borrowed` when no rewrite was needed (fast path).
///
/// `raw_content` should be the body *between* the quotes (e.g. for `'foo'`
/// pass `foo`).
///
/// ```text
/// use oxc_formatter_core::util::string::{Quote, normalize_string};
/// assert_eq!(
///     normalize_string(" \"He\\llo\\tworld\" \\' \\' \r\n ", Quote::Double, true),
///     " \\\"He\\llo\\tworld\\\" ' ' \n ",
/// );
/// ```
pub fn normalize_string(
    raw_content: &str,
    preferred_quote: Quote,
    quotes_will_change: bool,
) -> Cow<'_, str> {
    let alternate_quote = preferred_quote.other().as_byte();
    let preferred_quote = preferred_quote.as_byte();
    let mut reduced_string = String::new();
    let mut copy_start = 0;
    let mut bytes = raw_content.bytes().enumerate().peekable();
    while let Some((byte_index, byte)) = bytes.next() {
        match byte {
            // If the next character is escaped
            b'\\' => {
                if let Some(&(escaped_index, escaped)) = bytes.peek() {
                    if escaped == b'\r' {
                        bytes.next(); // consume the \r
                        // Copy up to (not including) the \r
                        reduced_string.push_str(&raw_content[copy_start..escaped_index]);
                        if bytes.next_if(|(_, b)| *b == b'\n').is_some() {
                            // \\\r\n -> keep \\ and \n, skip \r
                            // The \n will be included when we copy from copy_start
                        } else {
                            // \\\r -> convert \r to \n
                            reduced_string.push('\n');
                        }
                        copy_start = escaped_index + 1;
                    } else if quotes_will_change && escaped == alternate_quote {
                        bytes.next(); // consume the escaped character
                        // Unescape alternate quotes if quotes are changing
                        reduced_string.push_str(&raw_content[copy_start..byte_index]);
                        copy_start = escaped_index;
                    } else {
                        bytes.next(); // consume the escaped character
                    }
                }
            }
            // Normalize \r\n and \r to \n
            b'\r' => {
                reduced_string.push_str(&raw_content[copy_start..byte_index]);
                if bytes.next_if(|(_, b)| *b == b'\n').is_some() {
                    // \r\n -> skip \r, the \n will be included when we copy from copy_start
                } else {
                    // Single \r -> convert to \n
                    reduced_string.push('\n');
                }
                copy_start = byte_index + 1;
            }
            _ => {
                // Escape unescaped preferred-quote occurrences.
                if byte == preferred_quote {
                    reduced_string.push_str(&raw_content[copy_start..byte_index]);
                    reduced_string.push('\\');
                    copy_start = byte_index;
                }
            }
        }
    }

    if copy_start == 0 && reduced_string.is_empty() {
        Cow::Borrowed(raw_content)
    } else {
        // Copy the remaining characters
        reduced_string.push_str(&raw_content[copy_start..]);
        Cow::Owned(reduced_string)
    }
}
