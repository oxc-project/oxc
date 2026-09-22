pub fn line_terminator_after(src: &[u8], n: usize, mut i: usize) -> usize {
    while i < n {
        match src[i] {
            b'\n' | b'\r' => return i,
            0xE2 if src[i + 1] == 0x80 && matches!(src[i + 2], 0xA8 | 0xA9) => return i,
            _ => i += 1,
        }
    }
    n
}

pub fn block_comment_end(src: &[u8], n: usize, mut i: usize) -> usize {
    while i + 1 < n {
        if src[i] == b'*' && src[i + 1] == b'/' {
            return i + 1;
        }
        i += 1;
    }
    n
}

/// 2 or 3 for a non-ASCII WhiteSpace or LineTerminator at p, else 0.
#[inline]
pub fn unicode_ws_len(src: &[u8], p: usize) -> usize {
    let c1 = src[p + 1];
    match src[p] {
        0xC2 => usize::from(c1 == 0xA0 || c1 == 0x85) * 2,
        0xE1 => usize::from(c1 == 0x9A && src[p + 2] == 0x80) * 3,
        0xE2 => {
            let c2 = src[p + 2];
            let is_ws = (c1 == 0x80
                && ((0x80..=0x8B).contains(&c2) || c2 == 0xA8 || c2 == 0xA9 || c2 == 0xAF))
                || (c1 == 0x81 && c2 == 0x9F);
            usize::from(is_ws) * 3
        }
        0xE3 => usize::from(c1 == 0x80 && src[p + 2] == 0x80) * 3,
        0xEF => usize::from(c1 == 0xBB && src[p + 2] == 0xBF) * 3,
        _ => 0,
    }
}
