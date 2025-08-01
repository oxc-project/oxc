//! Branchless utility functions for character classification and common operations.
//!
//! This module provides branchless functions for common lexer operations
//! like character classification to avoid branch mispredictions and improve performance.

/// Branchless function to check if a byte is ASCII whitespace.
/// Uses bit manipulation to avoid branches.
#[inline]
pub const fn is_ascii_whitespace_branchless(byte: u8) -> bool {
    // ASCII whitespace: ' ' (32), '\t' (9), '\n' (10), '\r' (13)
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

/// Branchless function to check if a byte is an ASCII identifier start character.
/// Uses bit manipulation for maximum performance.
#[inline]
pub const fn is_ascii_id_start_branchless(byte: u8) -> bool {
    // ASCII identifier start: a-z, A-Z, _, $
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => true,
        _ => false,
    }
}

/// Branchless function to check if a byte is an ASCII identifier continuation character.
/// Uses bit manipulation for maximum performance.
#[inline]
pub const fn is_ascii_id_continue_branchless(byte: u8) -> bool {
    // ASCII identifier continue: a-z, A-Z, 0-9, _, $
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' => true,
        _ => false,
    }
}

/// Branchless function to convert ASCII hex digit to value.
/// Returns 16 for non-hex digits.
#[inline]
pub const fn ascii_hex_digit_branchless(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 16, // Invalid hex digit
    }
}

/// Scan for consecutive ASCII whitespace characters.
/// Returns the number of whitespace bytes consumed.
pub fn skip_ascii_whitespace(bytes: &[u8]) -> usize {
    let mut pos = 0;
    for &byte in bytes {
        if is_ascii_whitespace_branchless(byte) {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

/// Scan for ASCII identifier characters.
/// Returns the length of the valid ASCII identifier.
pub fn scan_ascii_identifier(bytes: &[u8], start_char: bool) -> usize {
    if bytes.is_empty() {
        return 0;
    }

    let mut pos = 0;

    // Check first character if it's a start character
    if start_char {
        if !is_ascii_id_start_branchless(bytes[0]) {
            return 0;
        }
        pos = 1;
    }

    // Scan remaining characters
    for &byte in &bytes[pos..] {
        if is_ascii_id_continue_branchless(byte) {
            pos += 1;
        } else {
            break;
        }
    }

    pos
}

/// Scan string content until quote or escape character.
/// Returns the number of bytes until the first quote or backslash.
pub fn scan_string_content(bytes: &[u8], quote: u8) -> usize {
    let mut pos = 0;
    for &byte in bytes {
        if byte == quote || byte == b'\\' {
            break;
        }
        pos += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branchless_whitespace() {
        assert!(is_ascii_whitespace_branchless(b' '));
        assert!(is_ascii_whitespace_branchless(b'\t'));
        assert!(is_ascii_whitespace_branchless(b'\n'));
        assert!(is_ascii_whitespace_branchless(b'\r'));
        assert!(!is_ascii_whitespace_branchless(b'a'));
        assert!(!is_ascii_whitespace_branchless(b'0'));
    }

    #[test]
    fn test_branchless_identifier() {
        // Test identifier start
        assert!(is_ascii_id_start_branchless(b'a'));
        assert!(is_ascii_id_start_branchless(b'Z'));
        assert!(is_ascii_id_start_branchless(b'_'));
        assert!(is_ascii_id_start_branchless(b'$'));
        assert!(!is_ascii_id_start_branchless(b'0'));
        assert!(!is_ascii_id_start_branchless(b' '));

        // Test identifier continue
        assert!(is_ascii_id_continue_branchless(b'a'));
        assert!(is_ascii_id_continue_branchless(b'Z'));
        assert!(is_ascii_id_continue_branchless(b'0'));
        assert!(is_ascii_id_continue_branchless(b'_'));
        assert!(is_ascii_id_continue_branchless(b'$'));
        assert!(!is_ascii_id_continue_branchless(b' '));
        assert!(!is_ascii_id_continue_branchless(b'.'));
    }

    #[test]
    fn test_hex_digit_branchless() {
        assert_eq!(ascii_hex_digit_branchless(b'0'), 0);
        assert_eq!(ascii_hex_digit_branchless(b'9'), 9);
        assert_eq!(ascii_hex_digit_branchless(b'a'), 10);
        assert_eq!(ascii_hex_digit_branchless(b'f'), 15);
        assert_eq!(ascii_hex_digit_branchless(b'A'), 10);
        assert_eq!(ascii_hex_digit_branchless(b'F'), 15);
        assert_eq!(ascii_hex_digit_branchless(b'g'), 16);
        assert_eq!(ascii_hex_digit_branchless(b' '), 16);
    }

    #[test]
    fn test_whitespace_scanning() {
        let input = b"   \t\n  hello";
        let result = skip_ascii_whitespace(input);
        assert_eq!(result, 7);

        let input = b"hello";
        let result = skip_ascii_whitespace(input);
        assert_eq!(result, 0);

        let input = b"     ";
        let result = skip_ascii_whitespace(input);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_identifier_scanning() {
        let input = b"hello123_$ world";
        let result = scan_ascii_identifier(input, true);
        assert_eq!(result, 10);

        let input = b"123hello";
        let result = scan_ascii_identifier(input, true);
        assert_eq!(result, 0);

        let input = b"_private";
        let result = scan_ascii_identifier(input, true);
        assert_eq!(result, 8);
    }

    #[test]
    fn test_string_scanning() {
        let input = b"hello world\"";
        let result = scan_string_content(input, b'"');
        assert_eq!(result, 11);

        let input = b"hello\\nworld";
        let result = scan_string_content(input, b'"');
        assert_eq!(result, 5);

        let input = b"simple";
        let result = scan_string_content(input, b'"');
        assert_eq!(result, 6);
    }
}