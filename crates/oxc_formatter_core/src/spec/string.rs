use std::borrow::Cow;

/// Normalizes a string-literal body for the chosen `preferred_quote` byte (`b'"'` / `b'\''`):
///
/// - Escapes any unescaped occurrence of `preferred_quote`
/// - Unescapes the alternate quote when `quotes_will_change` is `true`
///   - i.e. the outer delimiter is switching so previously-escaped chars no longer need escaping
/// - Normalizes line endings (`\r\n` and lone `\r`) to `\n`
///
/// Returns `Cow::Borrowed` when no rewrite was needed (fast path).
///
/// `raw_content` should be the body between the quotes (e.g. for `'foo'` pass `foo`).
/// The quote is a raw byte rather than a named type:
/// this layer owns no quote-style vocabulary, each formatter passes its own option's byte.
pub fn normalize_string(
    raw_content: &str,
    preferred_quote: u8,
    quotes_will_change: bool,
) -> Cow<'_, str> {
    let alternate_quote = if preferred_quote == b'"' { b'\'' } else { b'"' };
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
