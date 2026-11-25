use std::str;

use oxc_simd::{Cpu, cpu_multiversion, cpu_version};

#[cpu_multiversion]
#[inline(always)]
pub(super) const fn is_ws(c: u8, _cpu: impl Cpu) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0c || c == 0x0b
}

#[cpu_multiversion]
#[inline(always)]
pub(super) const fn is_word(c: u8, cpu: impl Cpu) -> bool {
    is_id_start(c, cpu) || is_digit(c, cpu)
}

#[cpu_multiversion]
#[inline(always)]
pub(super) const fn is_id_start(c: u8, _cpu: impl Cpu) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' || c == b'$' || c >= 0x80
}

#[cpu_multiversion]
#[inline(always)]
pub(super) const fn is_digit(c: u8, _cpu: impl Cpu) -> bool {
    c >= b'0' && c <= b'9'
}

/// Whether the two bytes after an 0xE2 lead make LS (U+2028) or PS (U+2029).
#[cpu_multiversion]
#[inline(always)]
pub(super) const fn is_ls_ps_tail(c1: u8, c2: u8, _cpu: impl Cpu) -> bool {
    c1 == 0x80 && (c2 == 0xA8 || c2 == 0xA9)
}

/// Byte length (2 or 3) of the multi-byte WhiteSpace or LineTerminator starting at c0, or 0.
#[cpu_multiversion]
#[inline(always)]
pub(super) const fn unicode_ws_len(c0: u8, c1: u8, c2: u8, _cpu: impl Cpu) -> usize {
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

/// Byte length (2 or 3) of the multi-byte ECMAScript WhiteSpace / LineTerminator at `p`, or 0.
///
/// The non-ASCII set:
/// U+0085, U+00A0, U+1680, U+2000..=U+200B, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
#[cpu_multiversion]
#[inline]
pub(super) fn unicode_ws_len_at(src: &[u8], p: usize, cpu: impl Cpu) -> usize {
    unicode_ws_len(src[p], src[p + 1], src[p + 2], cpu)
}

/// The first line terminator (LF, CR, LS or PS) at or after i, or n.
#[cpu_multiversion]
pub(super) fn line_terminator_after(src: &[u8], n: usize, mut i: usize, cpu: impl Cpu) -> usize {
    while i < n {
        match src[i] {
            b'\n' | b'\r' => return i,
            0xE2 if is_ls_ps_tail(src[i + 1], src[i + 2], cpu) => return i,
            _ => i += 1,
        }
    }
    n
}

/// Does src[a..b] hold a line terminator (LF, CR, or the 3-byte LS or PS)?
#[cpu_multiversion]
#[inline(always)]
pub(super) fn line_break_in(src: &[u8], a: usize, b: usize, cpu: impl Cpu) -> bool {
    line_terminator_after(src, b, a, cpu) < b
}

/// The slash of the first */ at or after i, or n when the comment is unterminated.
#[cpu_multiversion]
pub(super) fn block_comment_end(src: &[u8], n: usize, mut i: usize, _cpu: impl Cpu) -> usize {
    while i + 1 < n {
        if src[i] == b'*' && src[i + 1] == b'/' {
            return i + 1;
        }
        i += 1;
    }
    n
}

/// The char at byte i, or None if the bytes there are not valid UTF-8.
#[cpu_version(Generic)]
pub(super) fn decode_char_at(s: &[u8], i: usize) -> Option<char> {
    let end = (i + 4).min(s.len());
    let slice = s.get(i..end)?;
    match str::from_utf8(slice) {
        Ok(t) => t.chars().next(),
        // a multi-byte char cut at end still has a decodable valid prefix
        Err(e) if e.valid_up_to() > 0 => {
            str::from_utf8(&slice[..e.valid_up_to()]).ok()?.chars().next()
        }
        Err(_) => None,
    }
}

#[cpu_multiversion]
#[inline(always)]
pub(super) fn hex_digit(c: u8, _cpu: impl Cpu) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}
