//! x86-64 SSE2 SIMD implementation for batch search.
//!
//! SSE2 is guaranteed to be available on all 64-bit x86 CPUs (since 2003).
//! This module uses compile-time platform detection, no runtime checks needed.

#![expect(clippy::inline_always)]
#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::lexer::search::SafeByteMatchTable;

/// Find the first byte in a 32-byte batch that matches the given table using SSE2.
///
/// Processes 32 bytes as 2x16 using SSE2 registers. For each byte, performs a
/// table lookup to check if it matches.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn find_first_match_in_batch(
    batch: *const u8,
    table: &SafeByteMatchTable,
) -> Option<usize> {
    // For now, use scalar lookup with the table.
    // The structure is set up for future optimization with known target bytes.
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe { find_first_match_scalar(batch, table) }
}

/// Scalar implementation that iterates through the batch.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn find_first_match_scalar(batch: *const u8, table: &SafeByteMatchTable) -> Option<usize> {
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

/// Find the first byte matching any of the target bytes using SSE2.
///
/// This is the optimized SIMD path for when we know the exact bytes to search for.
/// Used for string/template literal scanning where target bytes are known at compile time.
///
/// # Arguments
///
/// * `batch` - Pointer to 32 bytes to search
/// * `targets` - Slice of target bytes to search for (should be small, typically 3-5 bytes)
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn find_first_match_in_batch_simd(batch: *const u8, targets: &[u8]) -> Option<usize> {
    // SAFETY: All operations below are safe because:
    // - batch points to at least 32 valid bytes (caller guarantees)
    // - SSE2 intrinsics are available on all x86_64 platforms
    unsafe {
        // Load 32 bytes as 2x16-byte SSE2 registers
        let lo = _mm_loadu_si128(batch as *const __m128i);
        let hi = _mm_loadu_si128(batch.add(16) as *const __m128i);

        // Compare against each target byte and accumulate matches
        let mut mask_lo = 0i32;
        let mut mask_hi = 0i32;

        for &target in targets {
            let needle = _mm_set1_epi8(target as i8);
            mask_lo |= _mm_movemask_epi8(_mm_cmpeq_epi8(lo, needle));
            mask_hi |= _mm_movemask_epi8(_mm_cmpeq_epi8(hi, needle));
        }

        // Find first matching byte
        if mask_lo != 0 {
            Some((mask_lo as u32).trailing_zeros() as usize)
        } else if mask_hi != 0 {
            Some(16 + (mask_hi as u32).trailing_zeros() as usize)
        } else {
            None
        }
    }
}

/// Find the first byte that does NOT match the given "continue" characters.
///
/// This is the inverse search - finds the first byte that breaks out of a pattern.
/// Used for identifier scanning where we want to find the first non-identifier character.
///
/// # Arguments
///
/// * `batch` - Pointer to 32 bytes to search
/// * `continue_chars` - Characters that should continue (NOT trigger a match)
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn find_first_non_match_in_batch_simd(
    batch: *const u8,
    continue_chars: &[u8],
) -> Option<usize> {
    // SAFETY: All operations below are safe because:
    // - batch points to at least 32 valid bytes (caller guarantees)
    // - SSE2 intrinsics are available on all x86_64 platforms
    unsafe {
        // Load 32 bytes as 2x16-byte SSE2 registers
        let lo = _mm_loadu_si128(batch as *const __m128i);
        let hi = _mm_loadu_si128(batch.add(16) as *const __m128i);

        // Find bytes that match ANY continue character (these should NOT stop)
        let mut is_continue_lo = _mm_setzero_si128();
        let mut is_continue_hi = _mm_setzero_si128();

        for &ch in continue_chars {
            let needle = _mm_set1_epi8(ch as i8);
            is_continue_lo = _mm_or_si128(is_continue_lo, _mm_cmpeq_epi8(lo, needle));
            is_continue_hi = _mm_or_si128(is_continue_hi, _mm_cmpeq_epi8(hi, needle));
        }

        // Invert: find bytes that are NOT continue characters (these SHOULD stop)
        // Note: _mm_cmpeq_epi8 returns 0xFF for match, 0x00 for no match
        // We want to find positions where is_continue is 0x00 (not a continue char)
        let all_ones = _mm_set1_epi8(-1); // 0xFF in all bytes
        let not_continue_lo = _mm_andnot_si128(is_continue_lo, all_ones);
        let not_continue_hi = _mm_andnot_si128(is_continue_hi, all_ones);

        let mask_lo = _mm_movemask_epi8(not_continue_lo);
        let mask_hi = _mm_movemask_epi8(not_continue_hi);

        if mask_lo != 0 {
            Some((mask_lo as u32).trailing_zeros() as usize)
        } else if mask_hi != 0 {
            Some(16 + (mask_hi as u32).trailing_zeros() as usize)
        } else {
            None
        }
    }
}

/// Find the first byte NOT in the ASCII identifier continue set (a-z, A-Z, 0-9, _, $).
///
/// Optimized for identifier scanning using range comparisons.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn find_first_non_id_continue(batch: *const u8) -> Option<usize> {
    // SAFETY: All operations below are safe because:
    // - batch points to at least 32 valid bytes (caller guarantees)
    // - SSE2 intrinsics are available on all x86_64 platforms
    unsafe {
        let lo = _mm_loadu_si128(batch as *const __m128i);
        let hi = _mm_loadu_si128(batch.add(16) as *const __m128i);

        // Check if byte is identifier continue character:
        // - 'a'-'z' (0x61-0x7A)
        // - 'A'-'Z' (0x41-0x5A)
        // - '0'-'9' (0x30-0x39)
        // - '_' (0x5F)
        // - '$' (0x24)

        let is_id_lo = is_ascii_id_continue_sse2(lo);
        let is_id_hi = is_ascii_id_continue_sse2(hi);

        // Find bytes that are NOT identifier continue (these should stop the search)
        let all_ones = _mm_set1_epi8(-1);
        let not_id_lo = _mm_andnot_si128(is_id_lo, all_ones);
        let not_id_hi = _mm_andnot_si128(is_id_hi, all_ones);

        let mask_lo = _mm_movemask_epi8(not_id_lo);
        let mask_hi = _mm_movemask_epi8(not_id_hi);

        if mask_lo != 0 {
            Some((mask_lo as u32).trailing_zeros() as usize)
        } else if mask_hi != 0 {
            Some(16 + (mask_hi as u32).trailing_zeros() as usize)
        } else {
            None
        }
    }
}

/// Check if bytes are ASCII identifier continue characters using SSE2.
/// Returns a mask where 0xFF = is identifier, 0x00 = is not.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn is_ascii_id_continue_sse2(v: __m128i) -> __m128i {
    // SAFETY: SSE2 intrinsics are available on all x86_64 platforms
    unsafe {
        // For SSE2, we use a simpler approach: check each specific character
        // that's an identifier continue and OR the results.
        // This avoids the complexity of range checks with signed comparison.

        // Check '_' and '$' (exact matches)
        let underscore = _mm_set1_epi8(b'_' as i8);
        let dollar = _mm_set1_epi8(b'$' as i8);
        let is_underscore = _mm_cmpeq_epi8(v, underscore);
        let is_dollar = _mm_cmpeq_epi8(v, dollar);

        // For alphanumeric, we use range checks via subtraction and comparison.
        // SSE2 only has signed comparison, so we need to be careful.
        // The trick: for unsigned comparison a <= x <= b, we check if (x - a) <= (b - a)
        // using unsigned saturation.

        // Check 'a'-'z': subtract 'a', compare with range width 25
        let a_lower = _mm_set1_epi8(b'a' as i8);
        let range_lower = _mm_set1_epi8(25); // 'z' - 'a' = 25
        let sub_lower = _mm_sub_epi8(v, a_lower);
        // For unsigned <= comparison: use saturating subtraction
        // if sub_lower <= range_lower, then (sub_lower - range_lower - 1) saturates to 0
        // We can also check if the high bit is not set after subtraction
        let in_lower = _mm_cmpeq_epi8(
            _mm_and_si128(sub_lower, _mm_cmpgt_epi8(sub_lower, range_lower)),
            _mm_setzero_si128(),
        );

        // Check 'A'-'Z': same approach
        let a_upper = _mm_set1_epi8(b'A' as i8);
        let range_upper = _mm_set1_epi8(25);
        let sub_upper = _mm_sub_epi8(v, a_upper);
        let in_upper = _mm_cmpeq_epi8(
            _mm_and_si128(sub_upper, _mm_cmpgt_epi8(sub_upper, range_upper)),
            _mm_setzero_si128(),
        );

        // Check '0'-'9': same approach with range 9
        let zero = _mm_set1_epi8(b'0' as i8);
        let range_digit = _mm_set1_epi8(9);
        let sub_digit = _mm_sub_epi8(v, zero);
        let in_digit = _mm_cmpeq_epi8(
            _mm_and_si128(sub_digit, _mm_cmpgt_epi8(sub_digit, range_digit)),
            _mm_setzero_si128(),
        );

        // OR all conditions together
        _mm_or_si128(
            _mm_or_si128(_mm_or_si128(in_lower, in_upper), in_digit),
            _mm_or_si128(is_underscore, is_dollar),
        )
    }
}

/// Find the first whitespace byte (' ', '\t', '\r', '\n') in batch.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn find_first_whitespace(batch: *const u8) -> Option<usize> {
    static WHITESPACE: [u8; 4] = [b' ', b'\t', b'\r', b'\n'];
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe { find_first_match_in_batch_simd(batch, &WHITESPACE) }
}

/// Find the first non-whitespace byte (anything except ' ', '\t', '\r', '\n') in batch.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn find_first_non_whitespace(batch: *const u8) -> Option<usize> {
    static WHITESPACE: [u8; 4] = [b' ', b'\t', b'\r', b'\n'];
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe { find_first_non_match_in_batch_simd(batch, &WHITESPACE) }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    #[test]
    fn test_find_first_match_simd() {
        let targets = [b'"', b'\\', b'\r', b'\n'];

        // Test: match at beginning
        let batch: [u8; 32] = [b'"'; 32];
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(0));

        // Test: match in middle (first half)
        let mut batch = [b'x'; 32];
        batch[10] = b'"';
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(10));

        // Test: match in middle (second half)
        let mut batch = [b'x'; 32];
        batch[20] = b'\\';
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(20));

        // Test: match at end
        let mut batch = [b'x'; 32];
        batch[31] = b'\n';
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(31));

        // Test: no match
        let batch = [b'x'; 32];
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, None);

        // Test: multiple matches, returns first
        let mut batch = [b'x'; 32];
        batch[5] = b'"';
        batch[15] = b'\\';
        batch[25] = b'\n';
        let result = unsafe { find_first_match_in_batch_simd(batch.as_ptr(), &targets) };
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_find_first_non_whitespace() {
        // Test: all whitespace
        let batch = [b' '; 32];
        let result = unsafe { find_first_non_whitespace(batch.as_ptr()) };
        assert_eq!(result, None);

        // Test: non-whitespace at start
        let mut batch = [b' '; 32];
        batch[0] = b'a';
        let result = unsafe { find_first_non_whitespace(batch.as_ptr()) };
        assert_eq!(result, Some(0));

        // Test: non-whitespace in middle
        let mut batch = [b' '; 32];
        batch[15] = b'x';
        let result = unsafe { find_first_non_whitespace(batch.as_ptr()) };
        assert_eq!(result, Some(15));

        // Test: mixed whitespace types
        let mut batch = [0u8; 32];
        for i in 0..32 {
            batch[i] = match i % 4 {
                0 => b' ',
                1 => b'\t',
                2 => b'\r',
                3 => b'\n',
                _ => unreachable!(),
            };
        }
        let result = unsafe { find_first_non_whitespace(batch.as_ptr()) };
        assert_eq!(result, None);

        // Add a non-whitespace
        batch[17] = b'!';
        let result = unsafe { find_first_non_whitespace(batch.as_ptr()) };
        assert_eq!(result, Some(17));
    }

    #[test]
    fn test_find_first_non_id_continue() {
        // All identifier chars
        let batch: [u8; 32] = *b"abcdefghijklmnopqrstuvwxyz_$0123";
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
