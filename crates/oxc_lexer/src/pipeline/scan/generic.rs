use oxc_simd::{BASELINE, cpu_version};

use crate::pipeline::{
    bytes::is_ls_ps_tail,
    chunk::{eqm, load64},
};

use super::common::is_license_at;

#[cpu_version(Generic)]
pub unsafe fn scan_block_comment_generic(
    src: *const u8,
    n: usize,
    mut i: usize,
) -> (usize, bool, Option<u32>) {
    let cpu = BASELINE;

    let mut saw_nl = false;
    let mut license_at = None;
    while i + 8 <= n {
        let x = load64(src, i);
        let y = load64(src, i + 1);
        let term = eqm(x, b'*') & eqm(y, b'/');
        let nl = eqm(x, b'\n') | eqm(x, b'\r');
        let at = eqm(x, b'@');
        if term != 0 {
            let j = (term.trailing_zeros() >> 3) as usize;
            let bodymask: u64 = if j > 0 { (1u64 << (j * 8)) - 1 } else { 0 };
            saw_nl |= (nl & bodymask) != 0;
            if license_at.is_none() {
                let am = at & bodymask;
                if am != 0 {
                    license_at = first_license_at8(src, i, am);
                }
            }
            return (i + j + 1, saw_nl, license_at);
        }
        saw_nl |= nl != 0;
        if license_at.is_none() && at != 0 {
            license_at = first_license_at8(src, i, at);
        }
        i += 8;
    }
    // The last byte can still be a newline. Reading the byte after it reads the PAD
    while i < n {
        let c = *src.add(i);
        if c == b'*' && *src.add(i + 1) == b'/' {
            return (i + 1, saw_nl, license_at);
        }
        if c == b'\n' || c == b'\r' {
            saw_nl = true;
        }
        if c == b'@' && license_at.is_none() && is_license_at(src, i, cpu) {
            license_at = Some(i as u32);
        }
        i += 1;
    }
    (n, saw_nl, license_at)
}

/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment
/// metadata. A 0xE2 that isn't LS/PS (typographic punctuation in prose) is
/// cleared from the mask and the scan continues; the LS/PS confirm can read
/// the pad, which never matches 0x80.
#[cpu_version(Generic)]
pub unsafe fn scan_line_comment_generic(
    src: *const u8,
    n: usize,
    mut i: usize,
) -> (usize, Option<u32>) {
    let cpu = BASELINE;

    let mut license_at = None;
    while i + 8 <= n {
        let x = load64(src, i);
        let mut term = eqm(x, b'\n') | eqm(x, b'\r') | eqm(x, 0xE2);
        let at = eqm(x, b'@');
        while term != 0 {
            let t = (term.trailing_zeros() >> 3) as usize;
            let c = *src.add(i + t);
            if c == 0xE2 && !is_ls_ps_tail(*src.add(i + t + 1), *src.add(i + t + 2), cpu) {
                term &= term - 1;
                continue;
            }
            if license_at.is_none() {
                let am = at & if t > 0 { (1u64 << (t * 8)) - 1 } else { 0 };
                if am != 0 {
                    license_at = first_license_at8(src, i, am);
                }
            }
            return (i + t, license_at);
        }
        if license_at.is_none() && at != 0 {
            license_at = first_license_at8(src, i, at);
        }
        i += 8;
    }
    while i < n {
        let c = *src.add(i);
        if c == b'\n' || c == b'\r' {
            return (i, license_at);
        }
        if c == 0xE2 && is_ls_ps_tail(*src.add(i + 1), *src.add(i + 2), cpu) {
            return (i, license_at);
        }
        if c == b'@' && license_at.is_none() && is_license_at(src, i, cpu) {
            license_at = Some(i as u32);
        }
        i += 1;
    }
    (n, license_at)
}

#[cpu_version(Generic)]
#[inline(always)]
unsafe fn first_license_at8(src: *const u8, base: usize, mut am: u64) -> Option<u32> {
    let cpu = BASELINE;

    while am != 0 {
        let q = base + (am.trailing_zeros() >> 3) as usize;
        am &= am - 1;
        if is_license_at(src, q, cpu) {
            return Some(q as u32);
        }
    }
    None
}
