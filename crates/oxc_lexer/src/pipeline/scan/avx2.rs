use oxc_simd::{Avx2, Vec256, cpu_version};

use crate::pipeline::{bytes::is_ls_ps_tail, source::Source};

use super::common::{first_license_at, window_bits};

#[cpu_version(Avx2)]
pub fn scan_block_comment_avx2(src: Source, mut i: usize, cpu: Avx2) -> (usize, bool, Option<u32>) {
    let n = src.len();
    let mut saw_nl = false;
    let mut license_at = None;
    while i < n {
        let v = Vec256::read_unaligned(src.window(i), cpu);
        let vn = Vec256::read_unaligned(src.window(i + 1), cpu);
        let term = (v.eq_u8(b'*') & vn.eq_u8(b'/')).movemask();
        let nl = if saw_nl { 0 } else { (v.eq_u8(b'\n') | v.eq_u8(b'\r')).movemask() };
        let at = if license_at.is_none() { v.eq_u8(b'@').movemask() } else { 0 };
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
#[cpu_version(Avx2)]
pub fn scan_line_comment_avx2(src: Source, mut i: usize, cpu: Avx2) -> (usize, Option<u32>) {
    let n = src.len();
    let mut license_at = None;
    while i < n {
        let v = Vec256::read_unaligned(src.window(i), cpu);
        let mut term = (v.eq_u8(b'\n') | v.eq_u8(b'\r') | v.eq_u8(0xE2)).movemask();
        let at = v.eq_u8(b'@').movemask();
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
