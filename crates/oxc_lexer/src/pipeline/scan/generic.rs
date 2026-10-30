use crate::pipeline::chunk::{eqm, load64};

use super::common::lic_verify_at;

/// Scan a `/*` block comment to its `*/`, noting whether the body holds a
/// LineTerminator (LF, CR, or LS/PS), and tracking the first `@license` /
/// `@preserve` position for comment metadata. A 0xE2 that isn't LS/PS
/// (typographic punctuation in prose) is no line break; the LS/PS confirm can
/// read the pad, which never matches 0x80.
pub unsafe fn scan_block_comment_generic(
    src: *const u8,
    n: usize,
    mut i: usize,
) -> (usize, bool, i64) {
    let mut saw_nl = false;
    let mut lic_q: i64 = -1;
    while i + 8 <= n {
        let x = load64(src, i);
        let y = load64(src, i + 1);
        let term = eqm(x, b'*') & eqm(y, b'/');
        let nl = if saw_nl { 0 } else { eqm(x, b'\n') | eqm(x, b'\r') | eqm(x, 0xE2) };
        let at = eqm(x, b'@');
        if term != 0 {
            let j = (term.trailing_zeros() >> 3) as usize;
            let bodymask: u64 = if j > 0 { (1u64 << (j * 8)) - 1 } else { 0 };
            saw_nl |= any_nl8(src, i, nl & bodymask);
            if lic_q < 0 {
                let am = at & bodymask;
                if am != 0 {
                    lic_q = lic_first8(src, i, am);
                }
            }
            return (i + j + 1, saw_nl, lic_q);
        }
        saw_nl |= any_nl8(src, i, nl);
        if lic_q < 0 && at != 0 {
            lic_q = lic_first8(src, i, at);
        }
        i += 8;
    }
    while i + 1 < n {
        let c = *src.add(i);
        if c == b'*' && *src.add(i + 1) == b'/' {
            return (i + 1, saw_nl, lic_q);
        }
        if c == b'\n'
            || c == b'\r'
            || (c == 0xE2
                && *src.add(i + 1) == 0x80
                && (*src.add(i + 2) == 0xA8 || *src.add(i + 2) == 0xA9))
        {
            saw_nl = true;
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, saw_nl, lic_q)
}

/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment
/// metadata. A 0xE2 that isn't LS/PS (typographic punctuation in prose) is
/// cleared from the mask and the scan continues; the LS/PS confirm can read
/// the pad, which never matches 0x80.
pub unsafe fn scan_line_comment_generic(src: *const u8, n: usize, mut i: usize) -> (usize, i64) {
    let mut lic_q: i64 = -1;
    while i + 8 <= n {
        let x = load64(src, i);
        let mut term = eqm(x, b'\n') | eqm(x, b'\r') | eqm(x, 0xE2);
        let at = eqm(x, b'@');
        while term != 0 {
            let t = (term.trailing_zeros() >> 3) as usize;
            let c = *src.add(i + t);
            if c == 0xE2
                && !(*src.add(i + t + 1) == 0x80
                    && (*src.add(i + t + 2) == 0xA8 || *src.add(i + t + 2) == 0xA9))
            {
                term &= term - 1;
                continue;
            }
            if lic_q < 0 {
                let am = at & if t > 0 { (1u64 << (t * 8)) - 1 } else { 0 };
                if am != 0 {
                    lic_q = lic_first8(src, i, am);
                }
            }
            return (i + t, lic_q);
        }
        if lic_q < 0 && at != 0 {
            lic_q = lic_first8(src, i, at);
        }
        i += 8;
    }
    while i < n {
        let c = *src.add(i);
        if c == b'\n' || c == b'\r' {
            return (i, lic_q);
        }
        if c == 0xE2
            && *src.add(i + 1) == 0x80
            && (*src.add(i + 2) == 0xA8 || *src.add(i + 2) == 0xA9)
        {
            return (i, lic_q);
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, lic_q)
}

#[inline(always)]
unsafe fn lic_first8(src: *const u8, base: usize, mut am: u64) -> i64 {
    while am != 0 {
        let q = base + (am.trailing_zeros() >> 3) as usize;
        am &= am - 1;
        if lic_verify_at(src, q) {
            return q as i64;
        }
    }
    -1
}

/// Is any LF, CR or 0xE2 marked in `nm` a LineTerminator? A 0xE2 is one only
/// as the lead byte of LS/PS.
#[inline(always)]
unsafe fn any_nl8(src: *const u8, base: usize, mut nm: u64) -> bool {
    while nm != 0 {
        let q = base + (nm.trailing_zeros() >> 3) as usize;
        nm &= nm - 1;
        if *src.add(q) != 0xE2
            || (*src.add(q + 1) == 0x80 && (*src.add(q + 2) == 0xA8 || *src.add(q + 2) == 0xA9))
        {
            return true;
        }
    }
    false
}
