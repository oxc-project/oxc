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
