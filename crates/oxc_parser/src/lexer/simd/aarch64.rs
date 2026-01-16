//! ARM64 NEON SIMD implementation for batch search.
//!
//! NEON is guaranteed to be available on all ARM64 (aarch64) CPUs.
//! This module uses compile-time platform detection, no runtime checks needed.

#![expect(clippy::inline_always)]
#![allow(dead_code)]

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::lexer::search::SafeByteMatchTable;

/// Find the first byte in a 32-byte batch that matches the given table using NEON.
///
/// Processes 32 bytes as 2x16 using NEON registers. For each byte, performs a
/// table lookup to check if it matches.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "aarch64")]
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
#[cfg(target_arch = "aarch64")]
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

/// Find the first byte matching any of the target bytes using NEON.
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
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn find_first_match_in_batch_simd(batch: *const u8, targets: &[u8]) -> Option<usize> {
    // SAFETY: All operations below are safe because:
    // - batch points to at least 32 valid bytes (caller guarantees)
    // - NEON intrinsics are available on all aarch64 platforms
    unsafe {
        // Load 32 bytes as 2x16-byte NEON registers
        let lo = vld1q_u8(batch);
        let hi = vld1q_u8(batch.add(16));

        // Compare against each target byte and accumulate matches
        let mut mask_lo = vdupq_n_u8(0);
        let mut mask_hi = vdupq_n_u8(0);

        for &target in targets {
            let needle = vdupq_n_u8(target);
            mask_lo = vorrq_u8(mask_lo, vceqq_u8(lo, needle));
            mask_hi = vorrq_u8(mask_hi, vceqq_u8(hi, needle));
        }

        // Find first matching byte
        // NEON doesn't have a direct movemask equivalent, so we use a different approach
        find_first_nonzero_byte_neon(mask_lo, mask_hi)
    }
}

/// Find the index of the first non-zero byte in the combined mask vectors.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn find_first_nonzero_byte_neon(lo: uint8x16_t, hi: uint8x16_t) -> Option<usize> {
    // SAFETY: NEON intrinsics are available on all aarch64 platforms
    unsafe {
        // Check if lo has any matches (any byte is 0xFF)
        let lo_max = vmaxvq_u8(lo);
        if lo_max != 0 {
            // Find first match in lo
            return Some(find_first_nonzero_in_vector(lo));
        }

        // Check hi
        let hi_max = vmaxvq_u8(hi);
        if hi_max != 0 {
            return Some(16 + find_first_nonzero_in_vector(hi));
        }

        None
    }
}

/// Find the index of the first non-zero byte in a single 16-byte vector.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn find_first_nonzero_in_vector(v: uint8x16_t) -> usize {
    // SAFETY: NEON intrinsics are available on all aarch64 platforms
    unsafe {
        // Store to array and find first non-zero
        // This is a simple approach; more optimized versions exist
        let mut arr = [0u8; 16];
        vst1q_u8(arr.as_mut_ptr(), v);
        for (i, &byte) in arr.iter().enumerate() {
            if byte != 0 {
                return i;
            }
        }
        16 // Should not happen if called correctly
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
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn find_first_non_match_in_batch_simd(
    batch: *const u8,
    continue_chars: &[u8],
) -> Option<usize> {
    // SAFETY: All operations below are safe because:
    // - batch points to at least 32 valid bytes (caller guarantees)
    // - NEON intrinsics are available on all aarch64 platforms
    unsafe {
        // Load 32 bytes as 2x16-byte NEON registers
        let lo = vld1q_u8(batch);
        let hi = vld1q_u8(batch.add(16));

        // Find bytes that match ANY continue character (these should NOT stop)
        let mut is_continue_lo = vdupq_n_u8(0);
        let mut is_continue_hi = vdupq_n_u8(0);

        for &ch in continue_chars {
            let needle = vdupq_n_u8(ch);
            is_continue_lo = vorrq_u8(is_continue_lo, vceqq_u8(lo, needle));
            is_continue_hi = vorrq_u8(is_continue_hi, vceqq_u8(hi, needle));
        }

        // Invert: find bytes that are NOT continue characters (these SHOULD stop)
        let all_ones = vdupq_n_u8(0xFF);
        let not_continue_lo = veorq_u8(is_continue_lo, all_ones); // XOR with all 1s = NOT
        let not_continue_hi = veorq_u8(is_continue_hi, all_ones);

        find_first_nonzero_byte_neon(not_continue_lo, not_continue_hi)
    }
}

/// Find the first byte NOT in the ASCII identifier continue set (a-z, A-Z, 0-9, _, $).
///
/// Optimized for identifier scanning using range comparisons.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn find_first_non_id_continue(batch: *const u8) -> Option<usize> {
    // SAFETY: All operations below are safe because:
    // - batch points to at least 32 valid bytes (caller guarantees)
    // - NEON intrinsics are available on all aarch64 platforms
    unsafe {
        let lo = vld1q_u8(batch);
        let hi = vld1q_u8(batch.add(16));

        // Check if byte is identifier continue character using range checks
        let is_id_lo = is_ascii_id_continue_neon(lo);
        let is_id_hi = is_ascii_id_continue_neon(hi);

        // Find bytes that are NOT identifier continue
        let all_ones = vdupq_n_u8(0xFF);
        let not_id_lo = veorq_u8(is_id_lo, all_ones);
        let not_id_hi = veorq_u8(is_id_hi, all_ones);

        find_first_nonzero_byte_neon(not_id_lo, not_id_hi)
    }
}

/// Check if bytes are ASCII identifier continue characters using NEON.
/// Returns a mask where 0xFF = is identifier, 0x00 = is not.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn is_ascii_id_continue_neon(v: uint8x16_t) -> uint8x16_t {
    // SAFETY: NEON intrinsics are available on all aarch64 platforms
    unsafe {
        // Check for: a-z, A-Z, 0-9, _, $

        // 'a'-'z': 0x61-0x7A
        let a_lower = vdupq_n_u8(b'a');
        let z_lower = vdupq_n_u8(b'z');
        let in_lower = vandq_u8(vcgeq_u8(v, a_lower), vcleq_u8(v, z_lower));

        // 'A'-'Z': 0x41-0x5A
        let a_upper = vdupq_n_u8(b'A');
        let z_upper = vdupq_n_u8(b'Z');
        let in_upper = vandq_u8(vcgeq_u8(v, a_upper), vcleq_u8(v, z_upper));

        // '0'-'9': 0x30-0x39
        let zero = vdupq_n_u8(b'0');
        let nine = vdupq_n_u8(b'9');
        let in_digit = vandq_u8(vcgeq_u8(v, zero), vcleq_u8(v, nine));

        // '_' and '$'
        let underscore = vdupq_n_u8(b'_');
        let dollar = vdupq_n_u8(b'$');
        let is_underscore = vceqq_u8(v, underscore);
        let is_dollar = vceqq_u8(v, dollar);

        // OR all conditions
        vorrq_u8(
            vorrq_u8(vorrq_u8(in_lower, in_upper), in_digit),
            vorrq_u8(is_underscore, is_dollar),
        )
    }
}

/// Find the first whitespace byte (' ', '\t', '\r', '\n') in batch.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[cfg(target_arch = "aarch64")]
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
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn find_first_non_whitespace(batch: *const u8) -> Option<usize> {
    static WHITESPACE: [u8; 4] = [b' ', b'\t', b'\r', b'\n'];
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe { find_first_non_match_in_batch_simd(batch, &WHITESPACE) }
}

#[cfg(all(test, target_arch = "aarch64"))]
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
    }
}
