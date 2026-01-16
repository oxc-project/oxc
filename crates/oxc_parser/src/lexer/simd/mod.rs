//! SIMD-accelerated batch search for lexer.
//!
//! Provides platform-specific SIMD implementations for finding the first byte
//! matching a lookup table in a 32-byte batch. Falls back to scalar
//! implementation on unsupported platforms.
//!
//! # Platform Support
//!
//! | Platform | SIMD Used | Detection |
//! |----------|-----------|-----------|
//! | x86-64   | SSE2      | Compile-time (guaranteed on all x86-64) |
//! | ARM64    | NEON      | Compile-time (guaranteed on all ARM64) |
//! | Other    | Scalar    | Compile-time fallback |

// The specialized SIMD functions may or may not be used depending on optimization choices
#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

mod scalar;

use crate::lexer::search::SafeByteMatchTable;

/// Find the first byte in a batch that matches the given table.
///
/// Returns the index of the first matching byte, or `None` if no match is found.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[expect(clippy::inline_always)]
#[inline(always)]
pub unsafe fn find_first_match_in_batch(
    batch: *const u8,
    table: &SafeByteMatchTable,
) -> Option<usize> {
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        {
            x86_64::find_first_match_in_batch(batch, table)
        }

        #[cfg(target_arch = "aarch64")]
        {
            aarch64::find_first_match_in_batch(batch, table)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            scalar::find_first_match_in_batch(batch, table)
        }
    }
}

/// Find the first byte matching any of the target bytes using SIMD.
///
/// This is the optimized SIMD path for when we know the exact bytes to search for.
/// Used for string/template literal scanning where target bytes are known at compile time.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[expect(clippy::inline_always)]
#[inline(always)]
pub unsafe fn find_first_match_in_batch_simd(batch: *const u8, targets: &[u8]) -> Option<usize> {
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        {
            x86_64::find_first_match_in_batch_simd(batch, targets)
        }

        #[cfg(target_arch = "aarch64")]
        {
            aarch64::find_first_match_in_batch_simd(batch, targets)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            scalar::find_first_match_in_batch_simd(batch, targets)
        }
    }
}

/// Find the first byte that does NOT match the given "continue" characters.
///
/// This is the inverse search - finds the first byte that breaks out of a pattern.
/// Used for identifier scanning where we want to find the first non-identifier character.
///
/// # Safety
///
/// `batch` must point to at least 32 valid bytes.
#[expect(clippy::inline_always)]
#[inline(always)]
pub unsafe fn find_first_non_match_in_batch_simd(
    batch: *const u8,
    continue_chars: &[u8],
) -> Option<usize> {
    // SAFETY: Caller guarantees batch points to at least 32 valid bytes.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        {
            x86_64::find_first_non_match_in_batch_simd(batch, continue_chars)
        }

        #[cfg(target_arch = "aarch64")]
        {
            aarch64::find_first_non_match_in_batch_simd(batch, continue_chars)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            scalar::find_first_non_match_in_batch_simd(batch, continue_chars)
        }
    }
}
