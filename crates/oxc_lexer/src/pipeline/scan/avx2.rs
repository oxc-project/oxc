use oxc_simd::{Avx2, Vec256, cpu_version};

use crate::pipeline::{bytes::is_ls_ps_tail, source::Source};

use super::common::{first_license_at, is_license_at};

#[cpu_version(Avx2)]
pub fn scan_block_comment_avx2(src: Source, mut i: usize, cpu: Avx2) -> (usize, bool, Option<u32>) {
    let n = src.len();
    if i > n {
        return (n, false, None);
    }
    let mut saw_nl = false;
    let mut license_at = None;
    while i + 32 <= n {
        // SAFETY: the guard keeps i <= n, so i + 33 cannot wrap, and src holds PAD bytes past n.
        let (v, vn) = unsafe {
            let p = src.as_ptr().add(i);
            (Vec256::load_unaligned(p, cpu), Vec256::load_unaligned(p.add(1), cpu))
        };
        let term = (v.eq_u8(b'*') & vn.eq_u8(b'/')).movemask();
        let nl = if saw_nl { 0 } else { (v.eq_u8(b'\n') | v.eq_u8(b'\r')).movemask() };
        let at = if license_at.is_none() { v.eq_u8(b'@').movemask() } else { 0 };
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
#[cpu_version(Avx2)]
pub fn scan_line_comment_avx2(src: Source, mut i: usize, cpu: Avx2) -> (usize, Option<u32>) {
    let n = src.len();
    if i > n {
        return (n, None);
    }
    let mut license_at = None;
    while i + 32 <= n {
        // SAFETY: the guard keeps i <= n, so i + 32 cannot wrap, and the 32 bytes at i are in src.
        let v = unsafe { Vec256::load_unaligned(src.as_ptr().add(i), cpu) };
        let mut term = (v.eq_u8(b'\n') | v.eq_u8(b'\r') | v.eq_u8(0xE2)).movemask();
        let at = v.eq_u8(b'@').movemask();
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
