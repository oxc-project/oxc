#[inline(always)]
pub(super) const fn is_ws(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0c || c == 0x0b
}

#[inline(always)]
pub(super) const fn is_word(c: u8) -> bool {
    is_id_start(c) || is_digit(c)
}

#[inline(always)]
pub(super) const fn is_id_start(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' || c == b'$' || c >= 0x80
}

#[inline(always)]
pub(super) const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[inline(always)]
/// Byte length (2 or 3) of the multi-byte WhiteSpace or LineTerminator starting at c0, or 0.
pub(super) const fn unicode_ws_len(c0: u8, c1: u8, c2: u8) -> usize {
    match c0 {
        0xC2 => (c1 == 0xA0 || c1 == 0x85) as usize * 2,
        0xE1 => (c1 == 0x9A && c2 == 0x80) as usize * 3,
        0xE2 => {
            let ws = (c1 == 0x80
                && ((c2 >= 0x80 && c2 <= 0x8B) || c2 == 0xA8 || c2 == 0xA9 || c2 == 0xAF))
                || (c1 == 0x81 && c2 == 0x9F);
            ws as usize * 3
        }
        0xE3 => (c1 == 0x80 && c2 == 0x80) as usize * 3,
        0xEF => (c1 == 0xBB && c2 == 0xBF) as usize * 3,
        _ => 0,
    }
}

#[inline]
pub(super) fn unicode_ws_len_at(src: &[u8], p: usize) -> usize {
    unicode_ws_len(src[p], src[p + 1], src[p + 2])
}

/// The first line terminator (LF, CR, LS or PS) at or after i, or n.
pub(super) fn line_terminator_after(src: &[u8], n: usize, mut i: usize) -> usize {
    while i < n {
        match src[i] {
            b'\n' | b'\r' => return i,
            0xE2 if src[i + 1] == 0x80 && matches!(src[i + 2], 0xA8 | 0xA9) => return i,
            _ => i += 1,
        }
    }
    n
}

#[inline]
/// Does src[a..b] hold a line terminator (LF, CR, or the 3-byte LS or PS)?
pub(super) fn line_break_in(src: &[u8], a: usize, b: usize) -> bool {
    line_terminator_after(src, b, a) < b
}

/// The slash of the first */ at or after i, or n when the comment is unterminated.
pub(super) fn block_comment_end(src: &[u8], n: usize, mut i: usize) -> usize {
    while i + 1 < n {
        if src[i] == b'*' && src[i + 1] == b'/' {
            return i + 1;
        }
        i += 1;
    }
    n
}

#[inline(always)]
pub(super) fn hex_val(c: u8) -> u32 {
    if c >= b'0' && c <= b'9' {
        return (c - b'0') as u32;
    }
    let l = c | 0x20;
    if l >= b'a' && l <= b'f' {
        return (l - b'a' + 10) as u32;
    }
    255
}
