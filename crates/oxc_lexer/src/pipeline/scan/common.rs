use core::slice;

use crate::tables::{hex_val, is_digit, is_word};

use super::super::find::{find_regex, find_tmpl, find4};

#[inline]
pub unsafe fn scan_quoted(
    src: *const u8,
    n: usize,
    mut i: usize,
    q: u8,
    saw_nl: &mut bool,
) -> usize {
    loop {
        // LF/CR are watched too: raw line terminators in strings are diagnosed.
        let p = find4(src, n, i, q, b'\\', b'\n', b'\r');
        if p >= n {
            return n;
        }
        let ch = *src.add(p);
        if ch == b'\\' {
            // Escape, including line continuations. `\<CR><LF>` is one
            // LineTerminatorSequence: skip all three bytes, or the next find
            // lands on the LF and flags a legal continuation.
            let crlf = *src.add(p + 1) == b'\r' && *src.add(p + 2) == b'\n';
            i = p + 2 + usize::from(crlf);
            continue;
        }
        if ch == b'\n' || ch == b'\r' {
            *saw_nl = true;
            i = p + 1; // keep scanning so the token end is unchanged
            continue;
        }
        return p;
    }
}

#[inline]
pub unsafe fn scan_regex(src: *const u8, n: usize, mut i: usize, nl_at: &mut usize) -> usize {
    let mut incl = false;
    loop {
        let p = find_regex(src, n, i);
        if p >= n {
            return n;
        }
        let c = *src.add(p);
        if c == b'\\' {
            // An escaped line terminator is still invalid in a regex (the
            // grammar requires a NonTerminator); record it here or the p+2
            // skip would sail past it. The p+1<n guard keeps garbage pad
            // bytes after a `\` at EOF out of the diagnostics.
            if p + 1 < n {
                let c1 = *src.add(p + 1);
                if (c1 == b'\n' || c1 == b'\r') && *nl_at == usize::MAX {
                    *nl_at = p + 1;
                } else if c1 == 0xE2
                    && *nl_at == usize::MAX
                    && p + 3 < n
                    && *src.add(p + 2) == 0x80
                    && (*src.add(p + 3) == 0xA8 || *src.add(p + 3) == 0xA9)
                {
                    // Escaped LS/PS: span ends past the whole char; the p+2
                    // skip lands mid-sequence (unwatched 0x80) and sails on.
                    *nl_at = p + 3;
                }
            }
            i = p + 2;
            continue;
        }
        if c == b'\n' || c == b'\r' {
            // Raw line terminator: invalid anywhere in the body, `[...]`
            // included. Record the first; keep scanning so the token end
            // (and every downstream span) is unchanged.
            if *nl_at == usize::MAX {
                *nl_at = p;
            }
            i = p + 1;
            continue;
        }
        if c == 0xE2 {
            // LS/PS are line terminators too; other E2-led chars (em dash,
            // arrows) fall through untouched.
            if *nl_at == usize::MAX
                && p + 2 < n
                && *src.add(p + 1) == 0x80
                && (*src.add(p + 2) == 0xA8 || *src.add(p + 2) == 0xA9)
            {
                *nl_at = p + 2; // span ends past the whole char
            }
            i = p + 1;
            continue;
        }
        if incl {
            if c == b']' {
                incl = false;
            }
            i = p + 1;
            continue;
        }
        if c == b'[' {
            incl = true;
            i = p + 1;
            continue;
        }
        if c == b'/' {
            return p;
        }
        i = p + 1;
    }
}

#[inline]
pub unsafe fn scan_tmpl_text(src: *const u8, n: usize, mut i: usize, term: &mut i32) -> usize {
    loop {
        let p = find_tmpl(src, n, i);
        if p >= n {
            *term = 0;
            return n;
        }
        let c = *src.add(p);
        if c == b'\\' {
            i = p + 2;
            continue;
        }
        if c == b'`' {
            *term = 1;
            return p + 1;
        }
        if *src.add(p + 1) == b'{' {
            *term = 2;
            return p + 2;
        }
        i = p + 1;
    }
}

#[inline]
pub unsafe fn scan_ident_esc(src: *const u8, n: usize, p: usize) -> usize {
    let mut i = p;
    while i < n {
        if *src.add(i) == b'\\' && i + 1 < n && *src.add(i + 1) == b'u' {
            i += 2;
            if i < n && *src.add(i) == b'{' {
                i += 1;
                while i < n && hex_val(*src.add(i)) != 255 {
                    i += 1;
                }
                if i < n && *src.add(i) == b'}' {
                    i += 1;
                }
            } else {
                let mut k = 0;
                while k < 4 && i < n && hex_val(*src.add(i)) != 255 {
                    i += 1;
                    k += 1;
                }
            }
            while i < n && is_word(*src.add(i)) {
                i += 1;
            }
            continue;
        }
        break;
    }
    i
}

#[inline]
pub unsafe fn scan_number(src: *const u8, n: usize, pos: usize) -> usize {
    if *src.add(pos) == b'0' && pos + 1 < n {
        let c1 = *src.add(pos + 1);
        let c = c1 | 0x20;
        if c == b'x' || c == b'o' || c == b'b' {
            let radix = if c == b'x' {
                16
            } else if c == b'o' {
                8
            } else {
                2
            };
            let mut i = pos + 2;
            while i < n {
                let d = *src.add(i);
                if d == b'_' {
                    i += 1;
                    continue;
                }
                if hex_val(d) < radix {
                    i += 1;
                } else {
                    break;
                }
            }
            if i < n && *src.add(i) == b'n' {
                return i + 1;
            }
            return i;
        }
        if is_digit(c1) {
            let mut i = pos + 1;
            let mut octal = true;
            while i < n {
                let d = *src.add(i);
                if is_digit(d) {
                    octal &= d < b'8';
                } else if d != b'_' {
                    break;
                }
                i += 1;
            }
            if octal {
                return i;
            }
            return scan_fraction_exponent(src, n, i).0;
        }
    }
    let mut i = pos;
    while i < n && (is_digit(*src.add(i)) || *src.add(i) == b'_') {
        i += 1;
    }
    let (i, is_float) = scan_fraction_exponent(src, n, i);
    if !is_float && i < n && *src.add(i) == b'n' {
        return i + 1;
    }
    i
}

#[inline]
unsafe fn scan_fraction_exponent(src: *const u8, n: usize, mut i: usize) -> (usize, bool) {
    let mut is_float = false;
    if i < n && *src.add(i) == b'.' {
        is_float = true;
        i += 1;
        while i < n && (is_digit(*src.add(i)) || *src.add(i) == b'_') {
            i += 1;
        }
    }
    if i < n && (*src.add(i) | 0x20) == b'e' {
        is_float = true;
        i += 1;
        if i < n && (*src.add(i) == b'+' || *src.add(i) == b'-') {
            i += 1;
        }
        while i < n && (is_digit(*src.add(i)) || *src.add(i) == b'_') {
            i += 1;
        }
    }
    (i, is_float)
}

#[inline(always)]
pub(super) unsafe fn lic_verify_at(src: *const u8, q: usize) -> bool {
    let c1 = *src.add(q + 1);
    // spellchecker:disable-next-line
    (c1 == b'l' && slice::from_raw_parts(src.add(q + 2), 6) == b"icense")
        || (c1 == b'p' && slice::from_raw_parts(src.add(q + 2), 7) == b"reserve")
}
