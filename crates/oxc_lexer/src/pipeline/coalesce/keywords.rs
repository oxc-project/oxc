//! Keyword recognition for `coalesce`.
//!
//! `coalesce` collects the positions of words which may be keywords (candidates) in `kwpos`,
//! and every [`KWB`] words of the bitmaps calls [`kw_flush`] to check them with [`kw_match`].
//! A candidate which is a keyword has its kind changed from `Ident` to the keyword's [`TokenKind`].

use std::ptr;

use crate::token::TokenKind;

use crate::pipeline::keywords::kw_match;

pub const KWB: usize = 64;

/// Dispatch to the key-monomorphized verify without duplicating `coalesce`
/// itself: one predictable branch per flush, not per candidate.
#[inline(always)]
pub(super) unsafe fn kw_flush(
    ts: bool,
    src: *const u8,
    word: *const u64,
    kind: *mut u8,
    kwpos: *const u32,
    k: usize,
) {
    if ts {
        kw_verify_batch::<true>(src, word, kind, kwpos, k);
    } else {
        kw_verify_batch::<false>(src, word, kind, kwpos, k);
    }
}

/// Resolve a batch of keyword candidates (positions collected by
/// `coalesce`): exact match against the perfect-hash tables, patching
/// `kind` from IDENT to the keyword kind on hit. `IS_TS` selects the
/// active set's hash key - `(c0, c1, len)` for JS, `(c0, c1, last, len)`
/// for TS - monomorphized so the JS copy carries none of the wider key.
/// Kept out of line: inlining would double both variants into each of
/// coalesce's flush sites, and one call per KWB words is free.
#[inline(never)]
unsafe fn kw_verify_batch<const IS_TS: bool>(
    src: *const u8,
    word: *const u64,
    kind: *mut u8,
    pos: *const u32,
    k: usize,
) {
    for ix in 0..k {
        let p = *pos.add(ix) as usize;
        let len = word_len(word, p);
        // Candidates are code-level identifier starts (carve cleared every
        // literal interior and JSX start from the masks), so the incumbent
        // kind is IDENT: select over it instead of a read-modify-write.
        *kind.add(p) = kw_match::<IS_TS>(src.add(p), len) as u8;
    }
}

/// Check if the word starting at `pos` is a keyword, outside of a batch.
///
/// For `glue_number`, which makes a token start of a word directly after a number (`3in`).
/// That happens after `coalesce` has collected the candidates for the window, so the word is not in a batch.
///
/// Returns the keyword's [`TokenKind`] as a `u8`, or `Ident` if it's not a keyword.
///
/// # SAFETY
///
/// * `word` must be the word bitmap for `src`, and valid for reading 8 bytes from byte `pos / 8`.
/// * The byte at `pos` must be a word char.
/// * `src` must be valid for reading 16 bytes from `pos`.
#[inline]
pub(super) unsafe fn kw_match_word(
    ts: bool,
    src: *const u8,
    word: *const u64,
    pos: usize,
) -> TokenKind {
    let len = word_len(word, pos);
    if ts { kw_match::<true>(src.add(pos), len) } else { kw_match::<false>(src.add(pos), len) }
}

/// Get the length of the word starting at `pos` from the word bitmap, capped at 16 (`kw_match`'s limit).
///
/// Keyword candidates can be longer than 10 bytes, because `coalesce`'s length filter only sees one word
/// of the bitmap, so a word which crosses into the next one escapes it. A word of 16 bytes or more
/// is not a keyword, and with length 16, it doesn't match one either.
///
/// Returns 0 if the byte at `pos` is not a word char.
///
/// # SAFETY
///
/// `word` must be valid for reading 8 bytes from byte `pos / 8`.
#[inline(always)]
unsafe fn word_len(word: *const u64, pos: usize) -> usize {
    let x = ptr::read_unaligned((word as *const u8).add(pos >> 3) as *const u64) >> (pos & 7);
    (!(x as u16)).trailing_zeros() as usize
}

#[cfg(test)]
mod tests {
    use crate::token::tk;

    use super::*;

    /// Check [`kw_flush`] gets each word's length from the word bitmap and sets its kind,
    /// including for words longer than 16 bytes, and words crossing a 64-byte boundary.
    #[test]
    fn test_kw_flush() {
        let words: [(usize, &str, TokenKind, bool); 6] = [
            (3, "class", TokenKind::KwClass, false),
            (20, "interface", TokenKind::KwInterface, true),
            (40, "instanceof", TokenKind::KwInstanceof, false),
            (58, "instanceofabcdefghijklmnopqrstuvwxyz", TokenKind::Ident, false),
            (100, "abcdefghijklmnopq", TokenKind::Ident, false),
            (125, "typeof", TokenKind::KwTypeof, false),
        ];

        let mut src = [b' '; 256];
        let mut kinds = [tk!(Whitespace); 256];
        let mut word_bitmap = [0u64; 5];
        let mut positions = vec![];
        for &(pos, text, _, _) in &words {
            src[pos..pos + text.len()].copy_from_slice(text.as_bytes());
            for i in pos..pos + text.len() {
                kinds[i] = tk!(Ident);
                word_bitmap[i >> 6] |= 1 << (i & 63);
            }
            positions.push(pos as u32);
        }

        for is_ts in [false, true] {
            let mut kinds = kinds; // Clone
            // SAFETY: `src` and `kinds` cover every word plus 16 bytes,
            // and `word_bitmap` has a spare `u64` after the last word
            unsafe {
                kw_flush(
                    is_ts,
                    src.as_ptr(),
                    word_bitmap.as_ptr(),
                    kinds.as_mut_ptr(),
                    positions.as_ptr(),
                    positions.len(),
                );
            }

            for &(pos, text, mut kind, is_ts_only_keyword) in &words {
                if is_ts_only_keyword && !is_ts {
                    kind = TokenKind::Ident;
                }
                assert_eq!(kinds[pos], kind as u8, "`{text}` (is_ts = {is_ts})");
            }
        }
    }
}
