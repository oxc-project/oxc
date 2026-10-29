use crate::pipeline::bytes;

/// Byte length (2 or 3) of the multi-byte ECMAScript WhiteSpace /
/// LineTerminator at `p`, or 0. The non-ASCII set: U+0085, U+00A0, U+1680,
/// U+2000..=U+200B, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
#[inline]
pub unsafe fn unicode_ws_len(src: *const u8, p: usize) -> usize {
    bytes::unicode_ws_len(*src.add(p), *src.add(p + 1), *src.add(p + 2))
}
