use crate::tables::{is_word, is_ws};

use super::super::super::{
    bitmap::bm_next0,
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
    if len <= 8
        && !matches!(ce, b'.' | b':' | b'-')
        && !(is_ws(ce) && !is_word(*src.add(e + 1)) && jsx_name_continues_after(src, n, e))
    {
        let x = core::ptr::read_unaligned(src.add(a) as *const u64);
        let y = core::ptr::read_unaligned(src.add(b) as *const u64);
        let mask = if len == 8 { !0u64 } else { (1u64 << (len * 8)) - 1 };
        if (x ^ y) & mask != 0 {
            return false;
        }
        return b + len >= lim_b || !is_jsx_name_byte(*src.add(b + len));
    }
    jsx_names_equal(src, n, a, b, lim_b)
}

unsafe fn jsx_name_continues_after(src: *const u8, lim: usize, i: usize) -> bool {
    let t = jsx_skip_trivia(src, lim, i);
    t < lim && matches!(*src.add(t), b'.' | b':')
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
        let xn = is_jsx_name_byte(x);
        let yn = is_jsx_name_byte(y);
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
    if is_jsx_name_byte(c) || !(is_ws(c) || c == b'/') {
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
