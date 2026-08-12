use crate::error::diag_code;
use crate::lanes::Lanes;
use crate::opmap::{KwSet, OP_QDOT};
use crate::tables::{Tables, is_digit, is_op_char, is_word};

use super::NUM;
use super::bitmap::{bm_clear_range, bm_next0, bm_set1};
use super::find::scan_number;
use super::keywords::{KWB, kw_verify_batch};

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline(always)]
fn prefetch(p: *const u8) {
    unsafe { core::arch::x86_64::_mm_prefetch(p as *const i8, core::arch::x86_64::_MM_HINT_T0) }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline(always)]
fn prefetch(_p: *const u8) {}

unsafe fn munch_walk(
    t: &Tables,
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *const u64,
    kind: *mut u8,
    mut pos: usize,
) -> usize {
    let end = bm_next0(opch, pos, n);
    while end - pos >= 2 {
        bm_set1(st, pos);
        let rem = end - pos;
        let lmax: u32 = if rem < 4 { rem as u32 } else { 4 };
        let b0 = *src.add(pos);
        let b1 = *src.add(pos + 1);
        let b2 = *src.add(pos + 2);
        let b3 = *src.add(pos + 3);
        let mut opk: u32 = 0;
        let mut opl: u32 = 0;
        let mut l = lmax;
        while l >= 2 {
            let k = t.op.opmap_lookup(b0, b1, b2, b3, l);
            if k != 0 && !(k == OP_QDOT as u32 && pos + 2 < n && is_digit(*src.add(pos + 2))) {
                opk = k;
                opl = l;
                break;
            }
            l -= 1;
        }
        if opk != 0 {
            *kind.add(pos) = opk as u8;
            let mut j = 1usize;
            while j < opl as usize {
                *st.add((pos + j) >> 6) &= !(1u64 << ((pos + j) & 63));
                j += 1;
            }
            pos += opl as usize;
        } else {
            pos += 1;
        }
    }
    if end - pos == 1 {
        bm_set1(st, pos);
    }
    pos
}
unsafe fn glue_number(
    t: &Tables,
    kw: &KwSet,
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    kind: *mut u8,
    mut p: usize,
    lanes: &mut Lanes,
) -> usize {
    loop {
        let e2 = scan_number(src, n, p);
        *kind.add(p) = NUM + (*src.add(e2 - 1) == b'n') as u8;
        if e2 > p + 1 {
            bm_clear_range(st, p + 1, e2 - 1);
            bm_clear_range(opch, p, e2 - 1);
        }
        if e2 >= n {
            return n;
        }
        bm_set1(st, e2);
        let c = *src.add(e2);
        // A word char abutting the number end (`1.5n`, `3in`, `0b12`, `1π`)
        // is a spec-invalid adjacency. Never true on valid input, so the
        // whole arm is cold.
        if is_word(c) {
            let srcs = core::slice::from_raw_parts(src, n);
            if c < 0x80 {
                // A surviving `n` is a misplaced bigint suffix; scan_number
                // consumes legal ones. Token spans are unchanged either way.
                let code = if c == b'n' {
                    diag_code::INVALID_BIGINT
                } else {
                    diag_code::INVALID_NUMERIC_LITERAL
                };
                lanes.push_num_end_diag(srcs, e2, code);
                if is_digit(c) {
                    // Digit invalid for the radix (`0b12`) or after a bigint
                    // suffix (`1n2`): keep gluing from it.
                    p = e2;
                    continue;
                }
                // This token start postdates the kwc mask built in the word
                // prelude, so resolve any keyword kind inline (`3in` is `3`
                // + KW_IN). Same inputs as kw_verify_batch: length from the
                // word bitmap, no match right after a member `.` (a number
                // ending in `.` is that dot-run's first dot).
                if *src.add(e2 - 1) != b'.' {
                    let wb = word as *const u8;
                    let x = core::ptr::read_unaligned(wb.add(e2 >> 3) as *const u64) >> (e2 & 7);
                    let kk = kw.lookup(src.add(e2), (!x).trailing_zeros() as usize);
                    if kk != 0 {
                        *kind.add(e2) = kk as u8;
                    }
                }
            } else {
                // Cold decode; flags only a real Unicode IdentifierStart.
                lanes.push_num_end_diag_unicode(srcs, e2);
            }
            return e2; // token start already set at e2
        }
        if e2 + 1 < n && is_op_char(c) && (*opch.add((e2 + 1) >> 6) >> ((e2 + 1) & 63)) & 1 != 0 {
            let q = munch_walk(t, src, n, st, opch, kind, e2);
            if q < n
                && *src.add(q) == b'.'
                && is_digit(*src.add(q + 1))
                && (*st.add(q >> 6) >> (q & 63)) & 1 != 0
            {
                p = q;
                continue;
            }
            return q;
        }
        return e2;
    }
}
/// Dispatch to the key-monomorphized verify without duplicating `coalesce`
/// itself: one predictable branch per flush, not per candidate, keyed off
/// the set itself so the walk carries no extra mode scalar.
#[inline(always)]
unsafe fn kw_flush(
    kw: &KwSet,
    src: *const u8,
    word: *const u64,
    kind: *mut u8,
    kwpos: *const u32,
    k: usize,
) {
    if kw.ts_key {
        kw_verify_batch::<true>(kw, src, word, kind, kwpos, k);
    } else {
        kw_verify_batch::<false>(kw, src, word, kind, kwpos, k);
    }
}
pub(super) unsafe fn coalesce(
    t: &Tables,
    kw: &KwSet,
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    digit: *const u64,
    dot: *const u64,
    kwinit: *const u64,
    kind: *mut u8,
    kwpos: *mut u32,
    lanes: &mut Lanes,
) {
    let nw = (n + 63) >> 6;
    let mut opprev: u64 = 0;
    let mut dtprev: u64 = 0;
    let mut cursor: usize = 0;
    let mut k = 0usize;
    for w in 0..nw {
        let op = *opch.add(w);
        let opnext = if w + 1 < nw { *opch.add(w + 1) } else { 0 };
        let dg = *digit.add(w);
        let dgnext = if w + 1 < nw { *digit.add(w + 1) } else { 0 };
        let dt = *dot.add(w);
        let stw = *st.add(w);
        let wd = *word.add(w);
        let wnext = if w + 1 < nw { *word.add(w + 1) } else { 0 };
        // The whole keywords stage lives inside this walk: the candidate
        // filter (token-start keyword-initial word char, next char also
        // word, not right after a member `.`, length <= 10 by the word-local
        // run filter) computes from values already in registers, candidates
        // extract into `kwpos`, and the batch verifies every KWB words while
        // this window's src/kind lines are still warm. The only token start
        // minted behind the filter is glue_number's invalid adjacency, which
        // resolves its keyword kind inline with the same inputs, so the two
        // writes agree even when both fire.
        let dm = dt & !((dt << 1) | (dtprev >> 63));
        let dcarry = ((dtprev >> 63) & !(dtprev >> 62)) & 1;
        let mut kwc =
            stw & wd & *kwinit.add(w) & ((wd >> 1) | (wnext << 63)) & !((dm << 1) | dcarry);
        let r2 = wd & (wd >> 1);
        let r4 = r2 & (r2 >> 2);
        let r8 = r4 & (r4 >> 4);
        kwc &= !(r8 & (r2 >> 8) & (wd >> 10));
        dtprev = dt;
        if kwc != 0 {
            let base = (w << 6) as u32;
            let cnt = kwc.count_ones() as usize;
            prefetch(src.add(base as usize));
            prefetch(kind.add(base as usize) as *const u8);
            *kwpos.add(k) = base + kwc.trailing_zeros();
            kwc &= kwc.wrapping_sub(1);
            *kwpos.add(k + 1) = base + kwc.trailing_zeros();
            kwc &= kwc.wrapping_sub(1);
            *kwpos.add(k + 2) = base + kwc.trailing_zeros();
            kwc &= kwc.wrapping_sub(1);
            *kwpos.add(k + 3) = base + kwc.trailing_zeros();
            kwc &= kwc.wrapping_sub(1);
            if cnt > 4 {
                let mut kk = k + 4;
                while kwc != 0 {
                    *kwpos.add(kk) = base + kwc.trailing_zeros();
                    kk += 1;
                    kwc &= kwc - 1;
                }
            }
            k += cnt;
        }
        let multi = op & !((op << 1) | (opprev >> 63)) & ((op >> 1) | (opnext << 63)) & stw;
        let dige0 = stw & dg;
        let dote = stw & dt & ((dg >> 1) | (dgnext << 63));
        let mut dige = dige0;
        if dige != 0 {
            let dend = !dg & (dg << 1);
            let bad = dend & (wd | dt);
            let mut keep: u64 = if bad != 0 { !0u64 } else { 0 };
            if dg >> 63 != 0 {
                keep |= 1u64 << (64 - (!dg).leading_zeros());
            }
            dige &= keep;
        }
        let mut ev = multi | dige | dote;
        let rare = dige | dote;
        opprev = op;
        while ev != 0 {
            let bit = ev.trailing_zeros() as usize;
            ev &= ev - 1;
            let p = (w << 6) + bit;
            if p < cursor {
                continue;
            }
            if (rare >> bit) & 1 != 0 {
                if (dige >> bit) & 1 != 0 {
                    let x = (wd >> bit) | ((wnext << (63 - bit)) << 1);
                    let y = (dg >> bit) | ((dgnext << (63 - bit)) << 1);
                    let wl = (!x).trailing_zeros() as usize;
                    let dl = (!y).trailing_zeros() as usize;
                    if wl == dl && wl < 64 && *src.add(p + wl) != b'.' {
                        continue;
                    }
                }
                cursor = glue_number(t, kw, src, n, st, opch, word, kind, p, lanes);
            } else {
                let y2 = (op >> bit) | ((opnext << (63 - bit)) << 1);
                let run = (!y2).trailing_zeros() as usize;
                let q = core::ptr::read_unaligned(src.add(p) as *const u32);
                let b0 = q as u8;
                let b1 = (q >> 8) as u8;
                if run == 2 {
                    let key = (q & 0xFFFF) | (2u32 << 24);
                    let pack = t.op2_pack[(key.wrapping_mul(t.op.opmap_mul) >> 24) as usize];
                    let want = 2u32 | ((b0 as u32) << 8) | ((b1 as u32) << 16);
                    let mut ok = ((pack ^ want) & 0x00FF_FFFF) == 0;
                    let kk = (pack >> 24) as u8;
                    ok &= !((kk == OP_QDOT) && is_digit((q >> 16) as u8));
                    let hm: u8 = 0u8.wrapping_sub(ok as u8);
                    *kind.add(p) = (*kind.add(p) & !hm) | (kk & hm);
                    let clr = (ok as u64) << ((p + 1) & 63);
                    *st.add((p + 1) >> 6) &= !clr;
                    cursor = p + 1 + ok as usize;
                    continue;
                }
                if run > 3 {
                    cursor = munch_walk(t, src, n, st, opch, kind, p);
                    continue;
                }
                let b2 = (q >> 16) as u8;
                let key3 = (q & 0xFF_FFFF) | (3u32 << 24);
                let p3 = t.op3_pack[(key3.wrapping_mul(t.op.opmap_mul) >> 24) as usize];
                let want3 = 3u64 | ((b0 as u64) << 8) | ((b1 as u64) << 16) | ((b2 as u64) << 24);
                let ok3 = ((p3 ^ want3) & 0xFFFF_FFFF) == 0;
                let key2a = (q & 0xFFFF) | (2u32 << 24);
                let pa = t.op2_pack[(key2a.wrapping_mul(t.op.opmap_mul) >> 24) as usize];
                let wanta = 2u32 | ((b0 as u32) << 8) | ((b1 as u32) << 16);
                let ka = (pa >> 24) as u8;
                let mut ok2a = ((pa ^ wanta) & 0x00FF_FFFF) == 0;
                ok2a &= !((ka == OP_QDOT) && is_digit(b2));
                let key2b = ((q >> 8) & 0xFFFF) | (2u32 << 24);
                let pb = t.op2_pack[(key2b.wrapping_mul(t.op.opmap_mul) >> 24) as usize];
                let wantb = 2u32 | ((b1 as u32) << 8) | ((b2 as u32) << 16);
                let kb = (pb >> 24) as u8;
                let mut ok2b = ((pb ^ wantb) & 0x00FF_FFFF) == 0;
                ok2b &= !((kb == OP_QDOT) && is_digit((q >> 24) as u8));
                let sel3 = ok3;
                let sel2a = !ok3 && ok2a;
                let sel2b = !ok3 && !ok2a && ok2b;
                let m0: u8 = 0u8.wrapping_sub((sel3 || sel2a) as u8);
                let k0v: u8 = if sel3 { (p3 >> 32) as u8 } else { ka };
                *kind.add(p) = (*kind.add(p) & !m0) | (k0v & m0);
                let m1: u8 = 0u8.wrapping_sub(sel2b as u8);
                *kind.add(p + 1) = (*kind.add(p + 1) & !m1) | (kb & m1);
                let clr1 = ((sel3 || sel2a) as u64) << ((p + 1) & 63);
                *st.add((p + 1) >> 6) &= !clr1;
                let clr2 = ((sel3 || sel2b) as u64) << ((p + 2) & 63);
                *st.add((p + 2) >> 6) &= !clr2;
                let mut adv = if sel3 { 3 } else { run - 1 };
                adv = if sel2a { 2 } else { adv };
                adv = if sel2b { 3 } else { adv };
                cursor = p + adv;
            }
        }
        if w & (KWB - 1) == KWB - 1 {
            kw_flush(kw, src, word, kind, kwpos, k);
            k = 0;
        }
    }
    kw_flush(kw, src, word, kind, kwpos, k);
}
