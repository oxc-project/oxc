use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Quote {
    Double,
    Single,
}

impl Quote {
    pub fn as_char(self) -> char {
        match self {
            Quote::Double => '"',
            Quote::Single => '\'',
        }
    }

    pub fn as_byte(self) -> u8 {
        self.as_char() as u8
    }

    /// Given the current quote, it returns the other one
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Quote::Double => Quote::Single,
            Quote::Single => Quote::Double,
        }
    }
}

/// This function is responsible of:
///
/// - escaping `preferred_quote`
/// - unescape alternate quotes of `preferred_quote` if `quotes_will_change`
/// - normalize the new lines by replacing `\r\n` with `\n` and '\r' with '\n'.
///
/// The function allocates a new string only if at least one change is performed.
///
/// In the following example `"` is escaped and the newline is normalized.
///
/// ```
/// use biome_formatter::token::string::{normalize_string, Quote};
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
    let capacity = if quotes_will_change {
        // If quotes change, we might need to escape some quotes, so preallocate enough space for the `\` character.
        raw_content.len() + raw_content.bytes().filter(|&b| b == preferred_quote).count()
    } else {
        // If quotes are not changing, then no `\` will be added, so the original length is sufficient.
        raw_content.len()
    };
    let mut reduced_string = String::with_capacity(capacity);
    let mut copy_start = 0;
    let mut bytes = raw_content.bytes().enumerate();
    while let Some((byte_index, byte)) = bytes.next() {
        match byte {
            // If the next character is escaped
            b'\\' => {
                if let Some((escaped_index, escaped)) = bytes.next() {
                    if escaped == b'\r' {
                        reduced_string.push_str(&raw_content[copy_start..escaped_index]);
                        copy_start = escaped_index + 1;
                        if !matches!(bytes.next(), Some((_, b'\n'))) {
                            reduced_string.push('\n');
                        }
                    } else if quotes_will_change && escaped == alternate_quote {
                        // Unescape alternate quotes if quotes are changing
                        reduced_string.push_str(&raw_content[copy_start..byte_index]);
                        copy_start = escaped_index;
                    }
                }
            }
            b'\r' => {
                reduced_string.push_str(&raw_content[copy_start..byte_index]);
                copy_start = byte_index + 1;
                if !matches!(bytes.next(), Some((_, b'\n'))) {
                    reduced_string.push('\n');
                }
            }
            _ => {
                // If we encounter a preferred quote and it's not escaped, we have to replace it with
                // an escaped version.
                // This is done because of how the enclosed strings can change.
                // Check `computed_preferred_quote` for more details.
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
        debug_assert!(
            reduced_string.len() <= capacity,
            "Something went wrong with the capacity calculation"
        );
        Cow::Owned(reduced_string)
    }
}
