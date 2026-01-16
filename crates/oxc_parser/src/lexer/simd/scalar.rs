//! Scalar fallback implementation for batch search.
//!
//! Used on platforms without SIMD support (WASM without SIMD, 32-bit, exotic architectures).

#![expect(clippy::inline_always)]
#![allow(dead_code)]

use crate::lexer::search::SafeByteMatchTable;

/// Find the first byte in a 32-byte batch that matches the given table.
///
/// Simple scalar implementation that iterates through the batch byte by byte.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[inline(always)]
pub unsafe fn find_first_match_in_batch(
    batch: *const u8,
    table: &SafeByteMatchTable,
) -> Option<usize> {
    for i in 0..32 {
        // SAFETY: Caller guarantees batch points to at least 32 valid bytes,
        // and i is always < 32.
        let byte = unsafe { *batch.add(i) };
        if table.matches(byte) {
            return Some(i);
        }
    }
    None
}

/// Find the first byte matching any of the target bytes (scalar version).
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[inline(always)]
pub unsafe fn find_first_match_in_batch_simd(batch: *const u8, targets: &[u8]) -> Option<usize> {
    for i in 0..32 {
        // SAFETY: Caller guarantees batch points to at least 32 valid bytes,
        // and i is always < 32.
        let byte = unsafe { *batch.add(i) };
        if targets.contains(&byte) {
            return Some(i);
        }
    }
    None
}

/// Find the first byte that does NOT match the given "continue" characters (scalar version).
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[inline(always)]
pub unsafe fn find_first_non_match_in_batch_simd(
    batch: *const u8,
    continue_chars: &[u8],
) -> Option<usize> {
    for i in 0..32 {
        // SAFETY: Caller guarantees batch points to at least 32 valid bytes,
        // and i is always < 32.
        let byte = unsafe { *batch.add(i) };
        if !continue_chars.contains(&byte) {
            return Some(i);
        }
    }
    None
}

/// Find the first byte NOT in the ASCII identifier continue set (scalar version).
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[inline(always)]
pub unsafe fn find_first_non_id_continue(batch: *const u8) -> Option<usize> {
    for i in 0..32 {
        // SAFETY: Caller guarantees batch points to at least 32 valid bytes,
        // and i is always < 32.
        let byte = unsafe { *batch.add(i) };
        if !is_ascii_id_continue(byte) {
            return Some(i);
        }
    }
    None
}

#[inline(always)]
fn is_ascii_id_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Find the first whitespace byte (scalar version).
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[inline(always)]
pub unsafe fn find_first_whitespace(batch: *const u8) -> Option<usize> {
    static WHITESPACE: [u8; 4] = [b' ', b'\t', b'\r', b'\n'];
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe { find_first_match_in_batch_simd(batch, &WHITESPACE) }
}

/// Find the first non-whitespace byte (scalar version).
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[inline(always)]
pub unsafe fn find_first_non_whitespace(batch: *const u8) -> Option<usize> {
    static WHITESPACE: [u8; 4] = [b' ', b'\t', b'\r', b'\n'];
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe { find_first_non_match_in_batch_simd(batch, &WHITESPACE) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_first_match_simd() {
        let targets = [b'"', b'\\', b'\r', b'\n'];

        // Test: match at beginning
        let batch: [u8; 32] = [b'"'; 32];
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(0));

        // Test: match in middle
        let mut batch = [b'x'; 32];
        batch[10] = b'"';
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(10));

        // Test: no match
        let batch = [b'x'; 32];
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_first_non_id_continue() {
        // All identifier chars
        let batch: [u8; 32] = *b"abcdefghijklmnopqrstuvwxyz012345";
        let result = unsafe { find_first_non_id_continue(batch.as_ptr()) };
        assert_eq!(result, None);

        // Non-id char at start
        let batch: [u8; 32] = *b"!bcdefghijklmnopqrstuvwxyz012345";
        let result = unsafe { find_first_non_id_continue(batch.as_ptr()) };
        assert_eq!(result, Some(0));

        // Non-id char in middle
        let mut batch: [u8; 32] = *b"abcdefghijklmnopqrstuvwxyz012345";
        batch[15] = b'!';
        let result = unsafe { find_first_non_id_continue(batch.as_ptr()) };
        assert_eq!(result, Some(15));
    }
}
