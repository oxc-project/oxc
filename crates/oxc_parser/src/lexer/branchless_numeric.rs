//! Branchless and optimized numeric parsing functions.

use super::simd_search::ascii_hex_digit_branchless;

/// Branchless function to check if a byte is a decimal digit.
#[inline]
pub const fn is_decimal_digit_branchless(byte: u8) -> bool {
    // Use wrapping subtraction and comparison to avoid branches
    byte.wrapping_sub(b'0') <= 9
}

/// Branchless function to check if a byte is a binary digit.
#[inline]
pub const fn is_binary_digit_branchless(byte: u8) -> bool {
    matches!(byte, b'0' | b'1')
}

/// Branchless function to check if a byte is an octal digit.
#[inline]
pub const fn is_octal_digit_branchless(byte: u8) -> bool {
    // Use wrapping subtraction and comparison
    byte.wrapping_sub(b'0') <= 7
}

/// Branchless function to check if a byte is a hexadecimal digit.
#[inline]
pub const fn is_hex_digit_branchless(byte: u8) -> bool {
    ascii_hex_digit_branchless(byte) < 16
}

/// Branchless conversion of decimal digit to value.
/// Returns 10 for non-decimal digits.
#[inline]
pub const fn decimal_digit_value_branchless(byte: u8) -> u8 {
    let value = byte.wrapping_sub(b'0');
    if value <= 9 { value } else { 10 }
}

/// Branchless conversion of binary digit to value.
/// Returns 2 for non-binary digits.
#[inline]
pub const fn binary_digit_value_branchless(byte: u8) -> u8 {
    match byte {
        b'0' => 0,
        b'1' => 1,
        _ => 2,
    }
}

/// Branchless conversion of octal digit to value.
/// Returns 8 for non-octal digits.
#[inline]
pub const fn octal_digit_value_branchless(byte: u8) -> u8 {
    let value = byte.wrapping_sub(b'0');
    if value <= 7 { value } else { 8 }
}

/// Fast scan for consecutive decimal digits.
/// Returns the number of consecutive decimal digits found.
pub fn scan_decimal_digits(bytes: &[u8]) -> usize {
    let mut count = 0;
    for &byte in bytes {
        if is_decimal_digit_branchless(byte) {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Fast scan for consecutive binary digits.
/// Returns the number of consecutive binary digits found.
pub fn scan_binary_digits(bytes: &[u8]) -> usize {
    let mut count = 0;
    for &byte in bytes {
        if is_binary_digit_branchless(byte) {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Fast scan for consecutive octal digits.
/// Returns the number of consecutive octal digits found.
pub fn scan_octal_digits(bytes: &[u8]) -> usize {
    let mut count = 0;
    for &byte in bytes {
        if is_octal_digit_branchless(byte) {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Fast scan for consecutive hexadecimal digits.
/// Returns the number of consecutive hexadecimal digits found.
pub fn scan_hex_digits(bytes: &[u8]) -> usize {
    let mut count = 0;
    for &byte in bytes {
        if is_hex_digit_branchless(byte) {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Branchless check if byte is underscore (numeric separator).
#[inline]
pub const fn is_numeric_separator(byte: u8) -> bool {
    byte == b'_'
}

/// Fast scan for numeric digits with underscore separators.
/// Returns (digit_count, total_bytes_consumed).
pub fn scan_digits_with_separators<F>(bytes: &[u8], is_valid_digit: F) -> (usize, usize)
where
    F: Fn(u8) -> bool,
{
    let mut digit_count = 0;
    let mut pos = 0;
    let mut last_was_underscore = false;

    for &byte in bytes {
        if is_valid_digit(byte) {
            digit_count += 1;
            pos += 1;
            last_was_underscore = false;
        } else if is_numeric_separator(byte) && !last_was_underscore && digit_count > 0 {
            pos += 1;
            last_was_underscore = true;
        } else {
            break;
        }
    }

    // Don't include trailing underscore
    if last_was_underscore && pos > 0 {
        pos -= 1;
    }

    (digit_count, pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branchless_digit_checks() {
        // Test decimal digits
        assert!(is_decimal_digit_branchless(b'0'));
        assert!(is_decimal_digit_branchless(b'9'));
        assert!(!is_decimal_digit_branchless(b'a'));
        assert!(!is_decimal_digit_branchless(b'/'));
        assert!(!is_decimal_digit_branchless(b':'));

        // Test binary digits
        assert!(is_binary_digit_branchless(b'0'));
        assert!(is_binary_digit_branchless(b'1'));
        assert!(!is_binary_digit_branchless(b'2'));
        assert!(!is_binary_digit_branchless(b'a'));

        // Test octal digits
        assert!(is_octal_digit_branchless(b'0'));
        assert!(is_octal_digit_branchless(b'7'));
        assert!(!is_octal_digit_branchless(b'8'));
        assert!(!is_octal_digit_branchless(b'9'));

        // Test hex digits
        assert!(is_hex_digit_branchless(b'0'));
        assert!(is_hex_digit_branchless(b'9'));
        assert!(is_hex_digit_branchless(b'a'));
        assert!(is_hex_digit_branchless(b'F'));
        assert!(!is_hex_digit_branchless(b'g'));
        assert!(!is_hex_digit_branchless(b'G'));
    }

    #[test]
    fn test_digit_value_conversion() {
        assert_eq!(decimal_digit_value_branchless(b'0'), 0);
        assert_eq!(decimal_digit_value_branchless(b'9'), 9);
        assert_eq!(decimal_digit_value_branchless(b'a'), 10);

        assert_eq!(binary_digit_value_branchless(b'0'), 0);
        assert_eq!(binary_digit_value_branchless(b'1'), 1);
        assert_eq!(binary_digit_value_branchless(b'2'), 2);

        assert_eq!(octal_digit_value_branchless(b'0'), 0);
        assert_eq!(octal_digit_value_branchless(b'7'), 7);
        assert_eq!(octal_digit_value_branchless(b'8'), 8);
    }

    #[test]
    fn test_digit_scanning() {
        let input = b"123abc";
        assert_eq!(scan_decimal_digits(input), 3);

        let input = b"101abc";
        assert_eq!(scan_binary_digits(input), 3);

        let input = b"567abc";
        assert_eq!(scan_octal_digits(input), 3);

        let input = b"abcdef123";
        assert_eq!(scan_hex_digits(input), 6);
    }

    #[test]
    fn test_digits_with_separators() {
        let input = b"123_456_789";
        let (digits, total) = scan_digits_with_separators(input, is_decimal_digit_branchless);
        assert_eq!(digits, 9);
        assert_eq!(total, 11);

        let input = b"101_010";
        let (digits, total) = scan_digits_with_separators(input, is_binary_digit_branchless);
        assert_eq!(digits, 6);
        assert_eq!(total, 7);

        let input = b"123_"; // Trailing underscore should be excluded
        let (digits, total) = scan_digits_with_separators(input, is_decimal_digit_branchless);
        assert_eq!(digits, 3);
        assert_eq!(total, 3);
    }
}
