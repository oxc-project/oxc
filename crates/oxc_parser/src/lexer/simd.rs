//! Lexer methods using portable-SIMD
//! See:
//!   * <https://github.com/rust-lang/portable-simd/blob/master/beginners-guide.md>
//!   * <https://rapidjson.org/md_doc_internals.html#SkipwhitespaceWithSIMD>
//!   * <https://lemire.me/blog/2017/01/20/how-quickly-can-you-remove-spaces-from-a-string>

use std::simd::{Simd, SimdPartialEq, ToBitMask};

const ELEMENTS: usize = 16;
type SimdVec = Simd<u8, ELEMENTS>;

pub struct SkipWhitespace {
    /// Total offset
    pub offset: usize,

    /// Found multiline comment end '*/'?
    pub found: bool,

    /// Found newline inside the comment?
    pub newline: bool,

    lf: SimdVec,
    cr: SimdVec,
    space: SimdVec,
    tab: SimdVec,
}

impl SkipWhitespace {
    pub fn new(newline: bool) -> Self {
        Self {
            offset: 0,
            found: false,
            newline,
            lf: SimdVec::splat(b'\n'),
            cr: SimdVec::splat(b'\r'),
            space: SimdVec::splat(b' '),
            tab: SimdVec::splat(b'\t'),
        }
    }

    pub fn simd(mut self, bytes: &[u8]) -> Self {
        let (chunks, remainder) = bytes.as_chunks::<ELEMENTS>();

        for chunk in chunks {
            self.check_chunk(chunk);
            if self.found {
                return self;
            }
        }

        if !remainder.is_empty() {
            // Align the last chunk for avoiding the use of a scalar version
            let mut chunk = [0; ELEMENTS];
            let len = remainder.len();
            chunk[..len].copy_from_slice(remainder);
            self.check_chunk(&chunk);
        }

        self
    }

    fn check_chunk(&mut self, chunk: &[u8]) {
        let s = SimdVec::from_slice(chunk);

        let any_newline = s.simd_eq(self.lf) | s.simd_eq(self.cr);
        let any_white = s.simd_eq(self.space) | s.simd_eq(self.tab) | any_newline;

        let advance_by = (!any_white.to_bitmask()).trailing_zeros();

        // If the advanced offset contains a newline
        if !self.newline
            && advance_by > 0
            && any_newline.to_bitmask() & (1u16.checked_shl(advance_by).map_or(u16::MAX, |c| c - 1))
                > 0
        {
            self.newline = true;
        }

        if (advance_by as usize) < ELEMENTS {
            self.found = true;
        }

        self.offset += advance_by as usize;
    }
}

pub struct SkipMultilineComment<'a> {
    /// Total offset
    pub offset: usize,

    /// Found multiline comment end '*/'?
    pub found: bool,

    /// Found newline inside the comment?
    pub newline: bool,

    /// Remaining char bytes from the lexer
    remaining: &'a [u8],

    star: SimdVec,
    slash: SimdVec,
    lf: SimdVec,
    cr: SimdVec,
    lsps: SimdVec,
}

impl<'a> SkipMultilineComment<'a> {
    pub fn new(newline: bool, remaining: &'a [u8]) -> Self {
        Self {
            offset: 0,
            found: false,
            newline,
            remaining,
            star: SimdVec::splat(b'*'),
            slash: SimdVec::splat(b'/'),
            lf: SimdVec::splat(b'\n'),
            cr: SimdVec::splat(b'\r'),
            lsps: SimdVec::splat(226),
        }
    }

    pub fn simd(mut self) -> Self {
        let (chunks, remainder) = self.remaining.as_chunks::<ELEMENTS>();

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

        let any_star = s.simd_eq(self.star);
        let any_slash = s.simd_eq(self.slash);
        let star_mask = any_star.to_bitmask();
        let slash_mask = any_slash.to_bitmask();

        // Get the offset of '/' if '*' is immediately followed by '/'
        let star_slash_mask = (star_mask << 1) & slash_mask;
        let star_slash_pos = star_slash_mask.trailing_zeros();

        let chunk_offset = if star_slash_mask > 0 {
            self.found = true;
            star_slash_pos as usize + 1
        } else {
            // Is '*' at the end?
            if star_mask & 1 << (ELEMENTS - 1) > 0
                && self.remaining.get(self.offset + ELEMENTS) == Some(&b'/')
            {
                self.found = true;
                ELEMENTS + 1
            } else {
                chunk_len
            }
        };

        // Look for '\n' and '\r'
        if !self.newline {
            let any_newline = s.simd_eq(self.lf) | s.simd_eq(self.cr);
            let newline_mask = any_newline.to_bitmask();
            self.newline = (newline_mask.trailing_zeros() as usize) < chunk_offset;
            // Look for LS '\u{2028}' [226, 128, 168] and PS '\u{2029}' [226, 128, 169]
            if !self.newline {
                let lsps_mask = s.simd_eq(self.lsps).to_bitmask();
                if lsps_mask > 0 {
                    let offset_by = lsps_mask.trailing_zeros() as usize;
                    if offset_by < chunk_offset {
                        let second = self.offset + offset_by + 1;
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

        self.offset += chunk_offset;
    }
}

pub struct SkipSinglineComment<'a> {
    /// Total offset
    pub offset: usize,

    /// Found multiline comment end '*/'?
    pub found: bool,

    /// Remaining char bytes from the lexer
    remaining: &'a [u8],

    lf: SimdVec,
    cr: SimdVec,
    lsps: SimdVec,
}

impl<'a> SkipSinglineComment<'a> {
    pub fn new(remaining: &'a [u8]) -> Self {
        Self {
            offset: 0,
            found: false,
            remaining,
            lf: SimdVec::splat(b'\n'),
            cr: SimdVec::splat(b'\r'),
            lsps: SimdVec::splat(226),
        }
    }

    pub fn simd(mut self) -> Self {
        let (chunks, remainder) = self.remaining.as_chunks::<ELEMENTS>();

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

        let any_newline = s.simd_eq(self.lf) | s.simd_eq(self.cr);
        let newline_mask = any_newline.to_bitmask();

        let advance_by = newline_mask.trailing_zeros() as usize;

        let chunk_offset = if advance_by < ELEMENTS {
            self.found = true;
            advance_by
        } else {
            chunk_len
        };

        // Look for LS '\u{2028}' [226, 128, 168] and PS '\u{2029}' [226, 128, 169]
        // if !self.found {
        // let lspf_mask = s.simd_eq(self.lsps).to_bitmask();
        // if lspf_mask > 0 {
        // let offset_by = lspf_mask.trailing_zeros() as usize;
        // if offset_by < chunk_offset {
        // let second = self.offset + offset_by + 1;
        // // Using scalar version `.get` instead of simd
        // // to avoid checking on the next chunk
        // // because this may be on the chunk boundary
        // if self.remaining.get(second) == Some(&128) {
        // let third = self.remaining.get(second + 1);
        // if matches!(third, Some(&168 | &169)) {
        // self.found = true;
        // chunk_offset = offset_by;
        // }
        // }
        // }
        // }
        // }

        self.offset += chunk_offset;
    }
}
