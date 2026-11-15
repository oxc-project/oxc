use oxc_simd::{Neon, Vec128, cpu_version};

use crate::pipeline::{bytes::is_ls_ps_tail, chunk::mm32, source::Source};

use super::common::{first_license_at, is_license_at};

#[cpu_version(Neon)]
pub fn scan_block_comment_neon(src: Source, mut i: usize, cpu: Neon) -> (usize, bool, Option<u32>) {
    let n = src.len();
    if i > n {
        return (n, false, None);
    }
    let mut saw_nl = false;
    let mut license_at = None;
    while i + 32 <= n {
        // SAFETY: the guard keeps i <= n, so i + 33 cannot wrap, and src holds PAD bytes past n.
        let (v0, v1, w0, w1) = unsafe {
            let p = src.as_ptr().add(i);
            (
                Vec128::load_unaligned(p, cpu),
                Vec128::load_unaligned(p.add(16), cpu),
                Vec128::load_unaligned(p.add(1), cpu),
                Vec128::load_unaligned(p.add(17), cpu),
            )
        };
        let term = mm32(v0.eq_u8(b'*') & w0.eq_u8(b'/'), v1.eq_u8(b'*') & w1.eq_u8(b'/'));
        let nl = if saw_nl {
            0
        } else {
            mm32(v0.eq_u8(b'\n') | v0.eq_u8(b'\r'), v1.eq_u8(b'\n') | v1.eq_u8(b'\r'))
        };
        let at = if license_at.is_none() { mm32(v0.eq_u8(b'@'), v1.eq_u8(b'@')) } else { 0 };
        if term != 0 {
            let j = term.trailing_zeros() as usize;
            let bodymask: u32 = if j > 0 { (1u32 << j) - 1 } else { 0 };
            saw_nl |= (nl & bodymask) != 0;
            if license_at.is_none() {
                let am = at & bodymask;
                if am != 0 {
                    license_at = first_license_at(src, i, am, cpu);
                }
            }
            return (i + j + 1, saw_nl, license_at);
        }
        saw_nl |= nl != 0;
        if license_at.is_none() && at != 0 {
            license_at = first_license_at(src, i, at, cpu);
        }
        i += 32;
    }
    // The last byte can still be a newline. Reading the byte after it reads the PAD
    while i < n {
        let c = src[i];
        if c == b'*' && src[i + 1] == b'/' {
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
#[cpu_version(Neon)]
pub fn scan_line_comment_neon(src: Source, mut i: usize, cpu: Neon) -> (usize, Option<u32>) {
    let n = src.len();
    if i > n {
        return (n, None);
    }
    let mut license_at = None;
    while i + 32 <= n {
        // SAFETY: the guard keeps i <= n, so i + 32 cannot wrap, and the 32 bytes at i are in src.
        let (v0, v1) = unsafe {
            let p = src.as_ptr().add(i);
            (Vec128::load_unaligned(p, cpu), Vec128::load_unaligned(p.add(16), cpu))
        };
        let term0 = v0.eq_u8(b'\n') | v0.eq_u8(b'\r') | v0.eq_u8(0xE2);
        let term1 = v1.eq_u8(b'\n') | v1.eq_u8(b'\r') | v1.eq_u8(0xE2);
        let mut term = mm32(term0, term1);
        let at = mm32(v0.eq_u8(b'@'), v1.eq_u8(b'@'));
        while term != 0 {
            let t = term.trailing_zeros() as usize;
            let c = src[i + t];
            if c == 0xE2 && !is_ls_ps_tail(src[i + t + 1], src[i + t + 2], cpu) {
                term &= term - 1; // not LS/PS: clear and keep scanning
                continue;
            }
            if license_at.is_none() {
                let am = at & if t > 0 { (1u32 << t) - 1 } else { 0 };
                if am != 0 {
                    license_at = first_license_at(src, i, am, cpu);
                }
            }
            return (i + t, license_at);
        }
        if license_at.is_none() && at != 0 {
            license_at = first_license_at(src, i, at, cpu);
        }
        i += 32;
    }
    while i < n {
        let c = src[i];
        if c == b'\n' || c == b'\r' {
            return (i, license_at);
        }
        if c == 0xE2 && is_ls_ps_tail(src[i + 1], src[i + 2], cpu) {
            return (i, license_at);
        }
        if c == b'@' && license_at.is_none() && is_license_at(src, i, cpu) {
            license_at = Some(i as u32);
        }
        i += 1;
    }
    (n, license_at)
}
