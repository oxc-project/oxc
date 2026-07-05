use core::arch::x86_64::*;

use crate::opmap::KwSet;

use super::IDENT;

pub(super) const KWB: usize = 64;
/// Resolve a batch of keyword candidates (positions collected by
/// `coalesce`): exact match against the perfect-hash tables, patching
/// `kind` from IDENT to the keyword kind on hit. `TS_KEY` selects the
/// active set's hash key — `(c0, c1, len)` for JS, `(c0, c1, last, len)`
/// for TS — monomorphized so the JS copy carries none of the wider key.
/// Kept out of line: inlining would double both variants into each of
/// coalesce's flush sites, and one call per KWB words is free.
#[inline(never)]
pub(super) unsafe fn kw_verify_batch<const TS_KEY: bool>(
    kw: &KwSet,
    src: *const u8,
    word: *const u64,
    kind: *mut u8,
    pos: *const u32,
    k: usize,
) {
    let wb = word as *const u8;
    for ix in 0..k {
        let p = *pos.add(ix) as usize;
        let x = core::ptr::read_unaligned(wb.add(p >> 3) as *const u64) >> (p & 7);
        let len = (!x).trailing_zeros() as usize;
        if len > 8 {
            let kk = kw.lookup(src.add(p), len);
            if kk != 0 {
                *kind.add(p) = kk as u8;
            }
            continue;
        }
        let w8 = core::ptr::read_unaligned(src.add(p) as *const u64);
        let z = _bzhi_u64(w8, (len << 3) as u32);
        let key = if TS_KEY {
            // Last char comes off the bzhi'd word: bits above len*8 are
            // already zero, so the shift leaves exactly that byte.
            (w8 as u32 & 0xFFFF) | (((z >> ((len << 3) - 8)) as u32) << 16) | ((len as u32) << 24)
        } else {
            (w8 as u32 & 0xFFFF) | ((len as u32) << 16)
        };
        let h = (key.wrapping_mul(kw.kw_hash_mul) >> kw.kw_hash_shift) as usize;
        let hm: u8 = 0u8.wrapping_sub((z == kw.kwh_pat[h]) as u8);
        // Candidates are code-level identifier starts (carve cleared every
        // literal interior and JSX start from the masks), so the incumbent
        // kind is IDENT: select over it instead of a read-modify-write.
        *kind.add(p) = (IDENT & !hm) | (kw.kwh_kind[h] & hm);
    }
}
