/// Byte length (2 or 3) of the multi-byte ECMAScript WhiteSpace /
/// LineTerminator at `p`, or 0. The non-ASCII set: U+0085, U+00A0, U+1680,
/// U+2000..=U+200B, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
#[inline]
pub unsafe fn unicode_ws_len(src: *const u8, p: usize) -> usize {
    let c1 = *src.add(p + 1);
    match *src.add(p) {
        0xC2 => usize::from(c1 == 0xA0 || c1 == 0x85) * 2,
        0xE1 => usize::from(c1 == 0x9A && *src.add(p + 2) == 0x80) * 3,
        0xE2 => {
            let c2 = *src.add(p + 2);
            let is_ws = (c1 == 0x80
                && ((0x80..=0x8B).contains(&c2) || c2 == 0xA8 || c2 == 0xA9 || c2 == 0xAF))
                || (c1 == 0x81 && c2 == 0x9F);
            usize::from(is_ws) * 3
        }
        0xE3 => usize::from(c1 == 0x80 && *src.add(p + 2) == 0x80) * 3,
        0xEF => usize::from(c1 == 0xBB && *src.add(p + 2) == 0xBF) * 3,
        _ => 0,
    }
}
