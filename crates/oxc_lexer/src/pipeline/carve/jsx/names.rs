use std::ptr;

use crate::pipeline::{
    bitmap::bm_next0,
    bytes::{is_word, is_ws},
    find::{find_line_terminator, unicode_ws_len},
    scan::scan_block_comment,
};

#[inline]
pub(super) unsafe fn jsx_names_equal_fast(
    src: *const u8,
    word: *const u64,
    n: usize,
    a: usize,
    b: usize,
    lim_b: usize,
) -> bool {
    let e = bm_next0(word, a, n);
    let len = e - a;
    let ce = *src.add(e);
    // The fast path compares one plain word; a comment or Unicode whitespace after it
    // (`<A/*c*/.B>`, `<A\u{a0}.B>`) may hide a member name, so those take the trivia-aware
    // path (a non-ASCII byte here is whitespace: `misc_pre` cleared it from `word`).
    if len <= 8
        && !matches!(ce, b'.' | b':' | b'-' | b'/')
        && ce < 0x80
        && !(is_ws(ce) && !is_word(*src.add(e + 1)) && jsx_name_continues_after(src, n, e))
    {
        let x = ptr::read_unaligned(src.add(a) as *const u64);
        let y = ptr::read_unaligned(src.add(b) as *const u64);
        let mask = if len == 8 { !0u64 } else { (1u64 << (len * 8)) - 1 };
        if (x ^ y) & mask != 0 {
            return false;
        }
        return b + len >= lim_b || !is_jsx_name_byte_at(src, b + len);
    }
    jsx_names_equal(src, n, a, b, lim_b)
}

/// Is the byte at `i` part of a JSX name? A non-ASCII lead byte is, unless it starts Unicode
/// whitespace (`</a\u{a0}>` closes `a`).
#[inline]
unsafe fn is_jsx_name_byte_at(src: *const u8, i: usize) -> bool {
    let c = *src.add(i);
    is_jsx_name_byte(c) && (c < 0x80 || unicode_ws_len(src, i) == 0)
}

unsafe fn jsx_name_continues_after(src: *const u8, lim: usize, i: usize) -> bool {
    let t = jsx_skip_trivia(src, lim, i);
    t < lim && matches!(*src.add(t), b'.' | b':')
}

/// [`jsx_skip_trivia`] with the hot shapes inline: nothing to skip (`<div`), or plain spaces
/// before a name byte (`<T extends`). Anything else (a comment, other whitespace, a
/// non-ASCII lead) takes the full skip. Kept inline: this runs once per JSX element.
#[inline(always)]
pub(super) unsafe fn jsx_skip_trivia_fast(src: *const u8, n: usize, mut i: usize) -> usize {
    while i < n && *src.add(i) == b' ' {
        i += 1;
    }
    let c = *src.add(i);
    if i < n && (is_ws(c) || c == b'/' || c >= 0x80) { jsx_skip_trivia(src, n, i) } else { i }
}

pub(super) unsafe fn jsx_skip_trivia(src: *const u8, n: usize, mut i: usize) -> usize {
    loop {
        if i >= n {
            return n;
        }
        let c = *src.add(i);
        if is_ws(c) {
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < n {
            match *src.add(i + 1) {
                b'*' => {
                    let e = scan_block_comment(src, n, i + 2).0;
                    if e >= n {
                        return n;
                    }
                    i = e + 1;
                    continue;
                }
                b'/' => {
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                _ => {}
            }
        }
        if c >= 0x80 {
            let w = unicode_ws_len(src, i);
            if w != 0 {
                i += w;
                continue;
            }
        }
        return i;
    }
}

unsafe fn jsx_names_equal(src: *const u8, n: usize, a: usize, b: usize, lim_b: usize) -> bool {
    let mut i = a;
    let mut j = b;
    let mut after_sep = false;
    loop {
        i = jsx_name_next(src, i, n, after_sep);
        j = jsx_name_next(src, j, lim_b, after_sep);
        let x = if i < n { *src.add(i) } else { 0 };
        let y = if j < lim_b { *src.add(j) } else { 0 };
        let xn = i < n && is_jsx_name_byte_at(src, i);
        let yn = j < lim_b && is_jsx_name_byte_at(src, j);
        if !xn && !yn {
            return true;
        }
        if x != y {
            return false;
        }
        after_sep = x == b'.' || x == b':';
        i += 1;
        j += 1;
    }
}

unsafe fn jsx_name_next(src: *const u8, i: usize, lim: usize, after_sep: bool) -> usize {
    if i >= lim {
        return lim;
    }
    let c = *src.add(i);
    let trivia = is_ws(c) || c == b'/' || (c >= 0x80 && unicode_ws_len(src, i) != 0);
    if !trivia {
        return i;
    }
    let t = jsx_skip_trivia(src, lim, i);
    if after_sep || (t < lim && matches!(*src.add(t), b'.' | b':')) { t } else { i }
}

pub(super) unsafe fn jsx_name_end(src: *const u8, n: usize, mut i: usize) -> usize {
    while i < n && is_jsx_name_byte(*src.add(i)) {
        i += 1;
    }
    i
}

#[inline(always)]
fn is_jsx_name_byte(c: u8) -> bool {
    is_word(c) || matches!(c, b'.' | b':' | b'-')
}
