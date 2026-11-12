use oxc_simd::{Neon, Vec128, cpu_version};

use crate::pipeline::chunk::mm32;

use super::common::lic_verify_at;

// Unlike `find`, these scans do bit arithmetic on the mask after the test, so they build
// a `movemask`-style mask with `mm32` and share the AVX2 implementation's logic.

#[cpu_version(Neon)]
pub unsafe fn scan_block_comment_neon(
    src: *const u8,
    n: usize,
    mut i: usize,
    cpu: Neon,
) -> (usize, bool, i64) {
    let mut saw_nl = false;
    let mut lic_q: i64 = -1;
    while i + 32 <= n {
        let v0 = Vec128::load_unaligned(src.add(i), cpu);
        let v1 = Vec128::load_unaligned(src.add(i + 16), cpu);
        // Reads 1 byte past n at most, into the caller-guaranteed PAD.
        let w0 = Vec128::load_unaligned(src.add(i + 1), cpu);
        let w1 = Vec128::load_unaligned(src.add(i + 17), cpu);
        let term = mm32(v0.eq_u8(b'*') & w0.eq_u8(b'/'), v1.eq_u8(b'*') & w1.eq_u8(b'/'));
        let nl = if saw_nl { 0 } else { mm32(newline(v0), newline(v1)) };
        let at = if lic_q < 0 { mm32(v0.eq_u8(b'@'), v1.eq_u8(b'@')) } else { 0 };
        if term != 0 {
            let j = term.trailing_zeros() as usize;
            let bodymask: u32 = if j > 0 { (1u32 << j) - 1 } else { 0 };
            saw_nl |= (nl & bodymask) != 0;
            if lic_q < 0 {
                let am = at & bodymask;
                if am != 0 {
                    lic_q = lic_first(src, i, am, cpu);
                }
            }
            return (i + j + 1, saw_nl, lic_q);
        }
        saw_nl |= nl != 0;
        if lic_q < 0 && at != 0 {
            lic_q = lic_first(src, i, at, cpu);
        }
        i += 32;
    }
    while i + 1 < n {
        let c = *src.add(i);
        if c == b'*' && *src.add(i + 1) == b'/' {
            return (i + 1, saw_nl, lic_q);
        }
        if c == b'\n' || c == b'\r' {
            saw_nl = true;
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i, cpu) {
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
#[cpu_version(Neon)]
pub unsafe fn scan_line_comment_neon(
    src: *const u8,
    n: usize,
    mut i: usize,
    cpu: Neon,
) -> (usize, i64) {
    let mut lic_q: i64 = -1;
    while i + 32 <= n {
        let v0 = Vec128::load_unaligned(src.add(i), cpu);
        let v1 = Vec128::load_unaligned(src.add(i + 16), cpu);
        let mut term = mm32(newline(v0) | v0.eq_u8(0xE2), newline(v1) | v1.eq_u8(0xE2));
        let at = mm32(v0.eq_u8(b'@'), v1.eq_u8(b'@'));
        while term != 0 {
            let t = term.trailing_zeros() as usize;
            let c = *src.add(i + t);
            if c == 0xE2
                && !(*src.add(i + t + 1) == 0x80
                    && (*src.add(i + t + 2) == 0xA8 || *src.add(i + t + 2) == 0xA9))
            {
                term &= term - 1; // not LS/PS: clear and keep scanning
                continue;
            }
            if lic_q < 0 {
                let am = at & if t > 0 { (1u32 << t) - 1 } else { 0 };
                if am != 0 {
                    lic_q = lic_first(src, i, am, cpu);
                }
            }
            return (i + t, lic_q);
        }
        if lic_q < 0 && at != 0 {
            lic_q = lic_first(src, i, at, cpu);
        }
        i += 32;
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
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i, cpu) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, lic_q)
}

#[cpu_version(Neon)]
#[inline(always)]
fn newline(v: Vec128<Neon>) -> Vec128<Neon> {
    v.eq_u8(b'\n') | v.eq_u8(b'\r')
}

#[cpu_version(Neon)]
#[inline(always)]
unsafe fn lic_first(src: *const u8, base: usize, mut am: u32, cpu: Neon) -> i64 {
    while am != 0 {
        let q = base + am.trailing_zeros() as usize;
        am &= am - 1;
        if lic_verify_at(src, q, cpu) {
            return q as i64;
        }
    }
    -1
}
