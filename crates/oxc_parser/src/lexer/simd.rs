//! Lexer methods using portable-SIMD
//! See:
//!   * <https://github.com/rust-lang/portable-simd/blob/master/beginners-guide.md>
//!   * <https://rapidjson.org/md_doc_internals.html#SkipwhitespaceWithSIMD>
//!   * <https://lemire.me/blog/2017/01/20/how-quickly-can-you-remove-spaces-from-a-string>

use std::simd::{Simd, SimdPartialEq, ToBitMask};

use lazy_static::lazy_static;

const ELEMENTS: usize = 16;
type SimdVec = Simd<u8, ELEMENTS>;

lazy_static! {
    static ref STAR: SimdVec = SimdVec::splat(b'*');
    static ref SLASH: SimdVec = SimdVec::splat(b'/');
    static ref LF: SimdVec = SimdVec::splat(b'\n');
    static ref CR: SimdVec = SimdVec::splat(b'\r');
    static ref LSPS: SimdVec = SimdVec::splat(226);
}

#[derive(Debug)]
pub struct MultiLineComment<'a> {
    /// Total offset
    pub offset: usize,

    /// Found multiline comment end '*/'?
    pub found: bool,

    /// Found newline inside the comment?
    pub newline: bool,

    /// Does the previous chunk has a '*' at the end?
    /// For checking against the first '/' on the current chunk.
    previous_star_at_end: bool,

    /// Remaining char bytes from the lexer
    remaining: &'a [u8],
}

impl<'a> MultiLineComment<'a> {
    pub const fn new(remaining: &'a [u8]) -> Self {
        Self { offset: 0, found: false, newline: false, previous_star_at_end: false, remaining }
    }

    pub fn simd(mut self, remaining: &[u8]) -> Self {
        let (chunks, remainder) = remaining.as_chunks::<ELEMENTS>();

        for chunk in chunks {
            self.check(chunk, chunk.len());
            if self.found {
                return self;
            }
        }

        if !remainder.is_empty() {
            // Align the last chunk for avoiding the use of a scalar version
            let mut chunk = [0; ELEMENTS];
            let len = remainder.len();
            chunk[..len].copy_from_slice(remainder);
            self.check(&chunk, len);
        }

        self
    }

    /// Check and compute state for a single chunk
    /// `chunk_len` can be < ELEMENTS for the last chunk
    fn check(&mut self, chunk: &[u8], chunk_len: usize) {
        let s = SimdVec::from_slice(chunk);

        let any_star = s.simd_eq(*STAR);
        let any_slash = s.simd_eq(*SLASH);
        let star_mask = any_star.to_bitmask();
        let slash_mask = any_slash.to_bitmask();

        // Get the offset of '/' if '*' is immediately followed by '/'
        let star_slash_mask = (star_mask << 1) & slash_mask;
        let star_slash_pos = star_slash_mask.trailing_zeros();

        let offset_total = if star_slash_mask > 0 {
            self.found = true;
            star_slash_pos as usize + 1
        } else if self.previous_star_at_end && slash_mask & 1 > 0 {
            // at boundary
            self.found = true;
            1
        } else {
            // Is '*' at the end?
            self.previous_star_at_end = star_mask & 1 << (ELEMENTS - 1) > 0;
            chunk_len
        };

        // Look for '\n' and '\r'
        if !self.newline {
            let any_newline = s.simd_eq(*LF) | s.simd_eq(*CR);
            let newline_mask = any_newline.to_bitmask();
            self.newline = newline_mask.trailing_zeros() < star_slash_pos;
            // Look for LS '\u{2028}' [226, 128, 168] and PS '\u{2029}' [226, 128, 169]
            if !self.newline {
                let lspf_mask = s.simd_eq(*LSPS).to_bitmask();
                if lspf_mask > 0 {
                    let offset_by = lspf_mask.trailing_zeros();
                    if offset_by < star_slash_pos {
                        let second = self.offset + offset_by as usize + 1;
                        // Using scalar version `.get` instead of simd
                        // to avoid checking on the next chunk
                        // because this may be on the chunk boundary
                        if self.remaining.get(second) == Some(&128) {
                            let third = self.remaining.get(second + 1);
                            if matches!(third, Some(&168 | &169)) {
                                self.newline = true;
                            }
                        }
                    }
                }
            }
        }

        self.offset += offset_total;
    }
}
