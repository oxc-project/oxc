//! SIMD-optimized byte searching for performance-critical lexer operations.
//!
//! This module provides SIMD-accelerated functions for common lexer operations
//! like whitespace scanning, identifier recognition, and string processing.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD batch size - use 16 bytes for SSE2, 32 bytes for AVX2
const SIMD_BATCH_SIZE: usize = 32;

/// Branchless function to check if a byte is ASCII whitespace.
/// Uses bit manipulation to avoid branches.
#[inline]
pub const fn is_ascii_whitespace_branchless(byte: u8) -> bool {
    // ASCII whitespace: ' ' (32), '\t' (9), '\n' (10), '\r' (13)
    // Use lookup table approach with bit operations
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

/// SIMD-accelerated whitespace skipping.
/// Returns the number of whitespace bytes consumed.
#[cfg(target_arch = "x86_64")]
pub fn skip_ascii_whitespace_simd(bytes: &[u8]) -> usize {
    if !is_x86_feature_detected!("avx2") {
        return skip_ascii_whitespace_fallback(bytes);
    }

    let mut pos = 0;
    let len = bytes.len();

    // Process 32-byte chunks with AVX2
    while pos + 32 <= len {
        unsafe {
            let chunk = _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i);

            // Create masks for each whitespace character
            let spaces = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b' ' as i8));
            let tabs = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b'\t' as i8));
            let newlines = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b'\n' as i8));
            let returns = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b'\r' as i8));

            // Combine all whitespace masks
            let whitespace =
                _mm256_or_si256(_mm256_or_si256(spaces, tabs), _mm256_or_si256(newlines, returns));

            // Get bitmask of matches
            let mask = _mm256_movemask_epi8(whitespace) as u32;

            if mask == 0xFFFFFFFF {
                // All 32 bytes are whitespace, continue
                pos += 32;
            } else if mask == 0 {
                // No whitespace found, we're done
                break;
            } else {
                // Some whitespace found - count leading whitespace bytes
                let trailing_zeros = mask.trailing_zeros() as usize;
                pos += trailing_zeros;
                break;
            }
        }
    }

    // Handle remaining bytes with scalar code
    pos + skip_ascii_whitespace_fallback(&bytes[pos..])
}

/// Fallback implementation for whitespace skipping without SIMD.
pub fn skip_ascii_whitespace_fallback(bytes: &[u8]) -> usize {
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

/// SIMD-accelerated ASCII identifier scanning.
/// Returns the length of the valid ASCII identifier.
#[cfg(target_arch = "x86_64")]
pub fn scan_ascii_identifier_simd(bytes: &[u8], start_char: bool) -> usize {
    if !is_x86_feature_detected!("avx2") {
        return scan_ascii_identifier_fallback(bytes, start_char);
    }

    if bytes.is_empty() {
        return 0;
    }

    let mut pos = 0;

    // Check first character if it's a start character
    if start_char && !is_ascii_id_start_branchless(bytes[0]) {
        return 0;
    }

    if start_char {
        pos = 1;
    }

    // Process remaining bytes in 32-byte chunks
    while pos + 32 <= bytes.len() {
        unsafe {
            let chunk = _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i);

            // Check for valid identifier continuation characters
            // Create ranges for a-z, A-Z, 0-9
            let lower_a = _mm256_set1_epi8(b'a' as i8);
            let lower_z = _mm256_set1_epi8(b'z' as i8);
            let upper_a = _mm256_set1_epi8(b'A' as i8);
            let upper_z = _mm256_set1_epi8(b'Z' as i8);
            let digit_0 = _mm256_set1_epi8(b'0' as i8);
            let digit_9 = _mm256_set1_epi8(b'9' as i8);
            let underscore = _mm256_set1_epi8(b'_' as i8);
            let dollar = _mm256_set1_epi8(b'$' as i8);

            // Check ranges
            let is_lower = _mm256_and_si256(
                _mm256_cmpgt_epi8(chunk, _mm256_sub_epi8(lower_a, _mm256_set1_epi8(1))),
                _mm256_cmpgt_epi8(_mm256_add_epi8(lower_z, _mm256_set1_epi8(1)), chunk),
            );
            let is_upper = _mm256_and_si256(
                _mm256_cmpgt_epi8(chunk, _mm256_sub_epi8(upper_a, _mm256_set1_epi8(1))),
                _mm256_cmpgt_epi8(_mm256_add_epi8(upper_z, _mm256_set1_epi8(1)), chunk),
            );
            let is_digit = _mm256_and_si256(
                _mm256_cmpgt_epi8(chunk, _mm256_sub_epi8(digit_0, _mm256_set1_epi8(1))),
                _mm256_cmpgt_epi8(_mm256_add_epi8(digit_9, _mm256_set1_epi8(1)), chunk),
            );
            let is_underscore = _mm256_cmpeq_epi8(chunk, underscore);
            let is_dollar = _mm256_cmpeq_epi8(chunk, dollar);

            // Combine all valid identifier character masks
            let valid = _mm256_or_si256(
                _mm256_or_si256(_mm256_or_si256(is_lower, is_upper), is_digit),
                _mm256_or_si256(is_underscore, is_dollar),
            );

            let mask = _mm256_movemask_epi8(valid) as u32;

            if mask == 0xFFFFFFFF {
                // All 32 bytes are valid identifier characters
                pos += 32;
            } else {
                // Some invalid character found - count valid leading bytes
                let trailing_zeros = mask.trailing_zeros() as usize;
                pos += trailing_zeros;
                break;
            }
        }
    }

    // Handle remaining bytes with scalar code
    pos + scan_ascii_identifier_fallback(&bytes[pos..], false)
}

/// Fallback implementation for ASCII identifier scanning without SIMD.
pub fn scan_ascii_identifier_fallback(bytes: &[u8], start_char: bool) -> usize {
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

/// SIMD-accelerated string scanning until quote or escape.
/// Returns the number of bytes until the first quote or backslash.
#[cfg(target_arch = "x86_64")]
pub fn scan_string_content_simd(bytes: &[u8], quote: u8) -> usize {
    if !is_x86_feature_detected!("avx2") {
        return scan_string_content_fallback(bytes, quote);
    }

    let mut pos = 0;

    while pos + 32 <= bytes.len() {
        unsafe {
            let chunk = _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i);

            // Look for quote character and backslash
            let quotes = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(quote as i8));
            let backslashes = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b'\\' as i8));

            // Combine masks
            let terminators = _mm256_or_si256(quotes, backslashes);

            let mask = _mm256_movemask_epi8(terminators) as u32;

            if mask == 0 {
                // No terminators found in this chunk
                pos += 32;
            } else {
                // Found terminator - return position of first one
                pos += mask.trailing_zeros() as usize;
                break;
            }
        }
    }

    // Handle remaining bytes
    pos + scan_string_content_fallback(&bytes[pos..], quote)
}

/// Fallback implementation for string content scanning without SIMD.
pub fn scan_string_content_fallback(bytes: &[u8], quote: u8) -> usize {
    let mut pos = 0;
    for &byte in bytes {
        if byte == quote || byte == b'\\' {
            break;
        }
        pos += 1;
    }
    pos
}

#[cfg(not(target_arch = "x86_64"))]
pub fn scan_ascii_identifier_simd(bytes: &[u8], start_char: bool) -> usize {
    scan_ascii_identifier_fallback(bytes, start_char)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn skip_ascii_whitespace_simd(bytes: &[u8]) -> usize {
    skip_ascii_whitespace_fallback(bytes)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn scan_string_content_simd(bytes: &[u8], quote: u8) -> usize {
    scan_string_content_fallback(bytes, quote)
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
        let result = skip_ascii_whitespace_fallback(input);
        assert_eq!(result, 6);

        let input = b"hello";
        let result = skip_ascii_whitespace_fallback(input);
        assert_eq!(result, 0);

        let input = b"     ";
        let result = skip_ascii_whitespace_fallback(input);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_identifier_scanning() {
        let input = b"hello123_$ world";
        let result = scan_ascii_identifier_fallback(input, true);
        assert_eq!(result, 10);

        let input = b"123hello";
        let result = scan_ascii_identifier_fallback(input, true);
        assert_eq!(result, 0);

        let input = b"_private";
        let result = scan_ascii_identifier_fallback(input, true);
        assert_eq!(result, 8);
    }

    #[test]
    fn test_string_scanning() {
        let input = b"hello world\"";
        let result = scan_string_content_fallback(input, b'"');
        assert_eq!(result, 11);

        let input = b"hello\\nworld";
        let result = scan_string_content_fallback(input, b'"');
        assert_eq!(result, 5);

        let input = b"simple";
        let result = scan_string_content_fallback(input, b'"');
        assert_eq!(result, 6);
    }
}
