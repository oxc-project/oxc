use oxc_simd::{Neon, Vec128, cpu_version};

use crate::pipeline::{bytes::is_ls_ps_tail, chunk::mm32, source::Source};

use super::common::{first_license_at, window_bits};

#[cpu_version(Neon)]
pub fn scan_block_comment_neon(src: Source, mut i: usize, cpu: Neon) -> (usize, bool, Option<u32>) {
    let n = src.len();
    let mut saw_nl = false;
    let mut license_at = None;
    while i < n {
        let v0 = Vec128::read_unaligned(src.window(i), cpu);
        let v1 = Vec128::read_unaligned(src.window(i + 16), cpu);
        let w0 = Vec128::read_unaligned(src.window(i + 1), cpu);
        let w1 = Vec128::read_unaligned(src.window(i + 17), cpu);
        let term = mm32(v0.eq_u8(b'*') & w0.eq_u8(b'/'), v1.eq_u8(b'*') & w1.eq_u8(b'/'));
        let nl = if saw_nl {
            0
        } else {
            mm32(v0.eq_u8(b'\n') | v0.eq_u8(b'\r'), v1.eq_u8(b'\n') | v1.eq_u8(b'\r'))
        };
        let at = if license_at.is_none() { mm32(v0.eq_u8(b'@'), v1.eq_u8(b'@')) } else { 0 };
        if term != 0 {
            let j = term.trailing_zeros() as usize;
            // A star at or past n is in the padding, so no comment ends in this window.
            if i + j < n {
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
        }
        if nl != 0 {
            saw_nl |= (nl & window_bits(n - i)) != 0;
        }
        if license_at.is_none() && at != 0 {
            license_at = first_license_at(src, i, at & window_bits(n - i), cpu);
        }
        i += 32;
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
    let mut license_at = None;
    while i < n {
        let v0 = Vec128::read_unaligned(src.window(i), cpu);
        let v1 = Vec128::read_unaligned(src.window(i + 16), cpu);
        let term0 = v0.eq_u8(b'\n') | v0.eq_u8(b'\r') | v0.eq_u8(0xE2);
        let term1 = v1.eq_u8(b'\n') | v1.eq_u8(b'\r') | v1.eq_u8(0xE2);
        let mut term = mm32(term0, term1);
        let at = mm32(v0.eq_u8(b'@'), v1.eq_u8(b'@'));
        while term != 0 {
            let t = term.trailing_zeros() as usize;
            // A terminator at or past n is in the padding, so the comment runs to n.
            if i + t >= n {
                break;
            }
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
            license_at = first_license_at(src, i, at & window_bits(n - i), cpu);
        }
        i += 32;
    }
    (n, license_at)
}
