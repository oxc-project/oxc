use super::find;

use crate::pipeline::bytes;

pub const OPENER: [u8; 6] = [b'"', b'\'', b'`', b'/', b'<', b'>'];
pub const OPENER8: [u8; 8] = [b'"', b'\'', b'`', b'/', b'{', b'}', b'<', b'>'];
pub const OPENER_JSX5: [u8; 5] = [b'"', b'\'', b'`', b'/', b'<'];
pub const OPENER_JSX7: [u8; 7] = [b'"', b'\'', b'`', b'/', b'{', b'}', b'<'];
pub const JSX_TAG: [u8; 6] = [b'"', b'\'', b'{', b'/', b'>', b'<'];
pub const JSX_TEXT: [u8; 4] = [b'<', b'{', b'>', b'}'];
pub const TMPL: [u8; 3] = [b'`', b'\\', b'$'];

/// Regex-body scan. LF, CR and the 0xE2 lead are watched so line terminators in the body can be diagnosed.
pub const REGEX: [u8; 7] = [b'/', b'\\', b'[', b']', b'\n', b'\r', 0xE2];

/// First ECMAScript LineTerminator at/after `i`: LF, CR, or the 3-byte LS/PS
/// (U+2028/U+2029). Stops at LS/PS too, so the hashbang scan cannot run
/// through one. A 0xE2 that isn't LS/PS keeps scanning; the 2-byte confirm
/// past a trailing 0xE2 reads the pad, which can never match 0x80.
#[inline]
pub unsafe fn find_line_terminator(src: *const u8, n: usize, mut i: usize) -> usize {
    loop {
        let p = find(src, n, i, [b'\n', b'\r', 0xE2]);
        if p >= n {
            return n;
        }
        if *src.add(p) != 0xE2 {
            return p;
        }
        if *src.add(p + 1) == 0x80 && (*src.add(p + 2) == 0xA8 || *src.add(p + 2) == 0xA9) {
            return p;
        }
        i = p + 1;
    }
}

/// Byte length (2 or 3) of the multi-byte ECMAScript WhiteSpace /
/// LineTerminator at `p`, or 0. The non-ASCII set: U+0085, U+00A0, U+1680,
/// U+2000..=U+200B, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
#[inline]
pub unsafe fn unicode_ws_len(src: *const u8, p: usize) -> usize {
    bytes::unicode_ws_len(*src.add(p), *src.add(p + 1), *src.add(p + 2))
}
