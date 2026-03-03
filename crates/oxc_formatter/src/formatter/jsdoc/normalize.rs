use std::borrow::Cow;

/// Normalize markdown emphasis markers:
/// - `__text__` → `**text**` (double underscore bold → asterisk bold)
/// - `*text*` → `_text_` (single asterisk italic → underscore italic)
///
/// Matches prettier-plugin-jsdoc which normalizes emphasis through remark.
/// Bold uses `**`, italic uses `_`.
pub fn normalize_markdown_emphasis(text: &str) -> Cow<'_, str> {
    if !text.contains("__") && !text.contains('*') {
        return Cow::Borrowed(text);
    }

    // Work with bytes directly — all significant chars (_, *, `, whitespace)
    // are ASCII single-byte, so byte-level mutation is safe and uses ~3x less
    // memory than Vec<char>.
    let mut bytes: Vec<u8> = text.as_bytes().to_vec();
    let len = bytes.len();
    let mut i = 0;
    let mut in_code = false;

    // First pass: convert `__` → `**`
    while i < len {
        if bytes[i] == b'`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if in_code {
            i += 1;
            continue;
        }
        if bytes[i] == b'_' && i + 1 < len && bytes[i + 1] == b'_' {
            bytes[i] = b'*';
            bytes[i + 1] = b'*';
            i += 2;
            continue;
        }
        i += 1;
    }

    // Second pass: convert single `*text*` → `_text_`
    // Skip `**` (bold) and content inside backticks.
    in_code = false;
    i = 0;
    while i < len {
        if bytes[i] == b'`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if in_code {
            i += 1;
            continue;
        }

        // Skip `**` (bold markers)
        if bytes[i] == b'*' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            continue;
        }

        // Single `*` — check if it's an opening emphasis marker:
        // Must be followed by a non-whitespace character
        if bytes[i] == b'*' && i + 1 < len && !bytes[i + 1].is_ascii_whitespace() {
            // Look for matching closing `*`
            let opener = i;
            let mut j = opener + 1;
            while j < len {
                if bytes[j] == b'`' {
                    // Skip inline code spans
                    j += 1;
                    while j < len && bytes[j] != b'`' {
                        j += 1;
                    }
                    if j < len {
                        j += 1;
                    }
                    continue;
                }
                // Skip `**` inside emphasis
                if bytes[j] == b'*' && j + 1 < len && bytes[j + 1] == b'*' {
                    j += 2;
                    continue;
                }
                // Found closing `*`: must be preceded by non-whitespace
                if bytes[j] == b'*' && j > opener + 1 && !bytes[j - 1].is_ascii_whitespace() {
                    bytes[opener] = b'_';
                    bytes[j] = b'_';
                    i = j + 1;
                    break;
                }
                j += 1;
            }
            if i <= opener {
                i = opener + 1;
            }
            continue;
        }

        i += 1;
    }

    // We only replaced ASCII bytes (_, *) with other ASCII bytes (*, _),
    // so UTF-8 validity is preserved.
    Cow::Owned(String::from_utf8(bytes).unwrap())
}

/// Capitalize the first ASCII lowercase letter of a string.
/// Skips if the string starts with a backtick (inline code) or a URL.
/// Recurses for `"- "` prefix: `"- hello"` → `"- Hello"` (matches upstream's `capitalizer()`).
pub fn capitalize_first(s: &str) -> Cow<'_, str> {
    if s.is_empty() || s.starts_with('`') || s.starts_with("http://") || s.starts_with("https://") {
        return Cow::Borrowed(s);
    }

    // Handle dash-prefix: "- text" → "- Text"
    if let Some(rest) = s.strip_prefix("- ") {
        let capitalized = capitalize_first(rest);
        let mut result = String::with_capacity(2 + capitalized.len());
        result.push_str("- ");
        result.push_str(&capitalized);
        return Cow::Owned(result);
    }

    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {
            let mut result = String::with_capacity(s.len());
            result.push(c.to_ascii_uppercase());
            result.push_str(chars.as_str());
            Cow::Owned(result)
        }
        _ => Cow::Borrowed(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("Hello"), "Hello");
        assert_eq!(capitalize_first("`code`"), "`code`");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("123"), "123");
        assert_eq!(capitalize_first("a"), "A");
        // Dash prefix handling (matches upstream's capitalizer)
        assert_eq!(capitalize_first("- hello"), "- Hello");
        assert_eq!(capitalize_first("- Hello"), "- Hello");
        assert_eq!(capitalize_first("- `code`"), "- `code`");
    }
}
