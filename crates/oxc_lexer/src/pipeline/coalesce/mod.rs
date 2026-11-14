use std::mem::MaybeUninit;

use oxc_simd::{Cpu, cpu_multiversion, prefetch};

use crate::{
    error::DiagCode,
    token::{TokenKind, tk},
};

use crate::pipeline::{
    bitmap::Bitmaps,
    bytes::{is_digit, is_word, is_ws},
    lanes::Lanes,
    lex::Lex,
    operators::{is_op_char, opmap_longest, opmap_pack2, opmap_pack3},
    scan::scan_number,
};

mod keywords;

pub use keywords::KWB;
use keywords::{kw_flush, kw_match_word};

/// [`glue_number`] computes the kind as `NUM + is_bigint` - keep them adjacent.
const _: () = assert!(tk!(BigInt) == tk!(Number) + 1);

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn coalesce(
    lex: Lex,
    kw_candidates: *mut u32,
    events: *mut u32,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, word, keyword_initial, op_char, digit, dot, .. } = *bitmaps;
    let is_ts = options.ts;
    lanes.disambiguate.restart(cpu);
    // Events are collected branchlessly and handled per batch. Unset masks are never read
    let mut rarem = [const { MaybeUninit::<u64>::uninit() }; KWB];
    let mut e = 0usize;
    let mut prev_empty = false;
    let block_count = src.block_count();
    let mut opprev: u64 = 0;
    let mut dtprev: u64 = 0;
    let mut cursor: usize = 0;
    let mut k = 0usize;
    for w in 0..block_count {
        let op = op_char.block(w, cpu);
        let opnext = if w + 1 < block_count { op_char.block(w + 1, cpu) } else { 0 };
        let dg = digit.block(w, cpu);
        let dgnext = if w + 1 < block_count { digit.block(w + 1, cpu) } else { 0 };
        let dt = dot.block(w, cpu);
        let stw = token_start.block(w, cpu);
        let wd = word.block(w, cpu);
        let wnext = if w + 1 < block_count { word.block(w + 1, cpu) } else { 0 };
        // The whole keywords stage lives inside this walk: the candidate
        // filter (token-start keyword-initial word char, next char also
        // word, not right after a member `.`, length <= 10 by the word-local
        // run filter) computes from values already in registers, candidates
        // extract into kw_candidates, and the batch verifies every KWB words while
        // this window's src/kind lines are still warm. The only token start
        // minted behind the filter is glue_number's invalid adjacency, which
        // resolves its keyword kind inline with the same inputs, so the two
        // writes agree even when both fire.
        let dm = dt & !((dt << 1) | (dtprev >> 63));
        let dcarry = ((dtprev >> 63) & !(dtprev >> 62)) & 1;
        let mut kwc = stw
            & wd
            & keyword_initial.block(w, cpu)
            & ((wd >> 1) | (wnext << 63))
            & !((dm << 1) | dcarry);
        let r2 = wd & (wd >> 1);
        let r4 = r2 & (r2 >> 2);
        let r8 = r4 & (r4 >> 4);
        kwc &= !(r8 & (r2 >> 8) & (wd >> 10));
        dtprev = dt;
        let multi = op & !((op << 1) | (opprev >> 63)) & ((op >> 1) | (opnext << 63)) & stw;
        let dote = stw & dt & ((dg >> 1) | (dgnext << 63));
        let mut dige = stw & dg;
        if dige != 0 {
            // Runs ending at a word char or dot need gluing, found by carrying back reversed
            let bad = !dg & (dg << 1) & (wd | dt);
            let rdg = dg.reverse_bits();
            let (sum, out) = rdg.overflowing_add((bad >> 1).reverse_bits());
            let mut glue = ((sum & !rdg).reverse_bits() << 1) | u64::from(out);
            if dg >> 63 != 0 {
                let tail = (!dgnext).trailing_zeros();
                let dtnext = if w + 1 < block_count { dot.block(w + 1, cpu) } else { 0 };
                if tail == 64 || ((wnext | dtnext) >> tail) & 1 != 0 {
                    glue |= 1u64 << (64 - (!dg).leading_zeros());
                }
            }
            dige &= glue;
        }
        let ev = multi | dige | dote;
        let rare = dige | dote;
        opprev = op;
        // Only skip within runs of empty blocks, so an isolated empty block never mispredicts
        let empty = (kwc | ev) == 0;
        if !(empty && prev_empty) {
            let base = (w << 6) as u32;
            let cnt = kwc.count_ones() as usize;
            prefetch(src.as_ptr().add(base as usize), cpu);
            prefetch(kinds.as_ptr().add(base as usize) as *const u8, cpu);
            for j in 0..8 {
                *kw_candidates.add(k + j) = base + kwc.trailing_zeros();
                kwc &= kwc.wrapping_sub(1);
            }
            if cnt > 8 {
                let mut kk = k + 8;
                while kwc != 0 {
                    *kw_candidates.add(kk) = base + kwc.trailing_zeros();
                    kk += 1;
                    kwc &= kwc - 1;
                }
            }
            k += cnt;
            rarem[w & (KWB - 1)].write(rare);
            let cnt = ev.count_ones() as usize;
            let mut evm = ev;
            for j in 0..2 {
                *events.add(e + j) = base + evm.trailing_zeros();
                evm &= evm.wrapping_sub(1);
            }
            if cnt > 2 {
                let mut ee = e + 2;
                while evm != 0 {
                    *events.add(ee) = base + evm.trailing_zeros();
                    ee += 1;
                    evm &= evm - 1;
                }
            }
            e += cnt;
        }
        prev_empty = empty;
        if w & (KWB - 1) == KWB - 1 {
            if e != 0 {
                cursor = coalesce_events(lex, lanes, events, e, &rarem, cursor, cpu);
                e = 0;
            }
            kw_flush(is_ts, src, word, kinds, kw_candidates, k, cpu);
            k = 0;
        }
    }
    if e != 0 {
        coalesce_events(lex, lanes, events, e, &rarem, cursor, cpu);
    }
    kw_flush(is_ts, src, word, kinds, kw_candidates, k, cpu);
}

/// Handle a batch's events in order. Rereading op_char is safe: events only clear it below cursor.
#[cpu_multiversion]
#[inline(never)]
unsafe fn coalesce_events(
    lex: Lex,
    lanes: &mut Lanes,
    events: *const u32,
    count: usize,
    rarem: &[MaybeUninit<u64>; KWB],
    mut cursor: usize,
    cpu: impl Cpu,
) -> usize {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, op_char, .. } = *bitmaps;
    let is_ts = options.ts;
    let block_count = src.block_count();
    for i in 0..count {
        let p = *events.add(i) as usize;
        if p < cursor {
            continue;
        }
        let (w, bit) = (p >> 6, p & 63);
        let rare = rarem[w & (KWB - 1)].assume_init();
        let has_next = w + 1 < block_count;
        if (rare >> bit) & 1 != 0 {
            cursor = glue_number(lex, p, lanes, cpu);
        } else {
            let op = op_char.block(w, cpu);
            let opnext = if has_next { op_char.block(w + 1, cpu) } else { 0 };
            let y2 = (op >> bit) | ((opnext << (63 - bit)) << 1);
            let run = (!y2).trailing_zeros() as usize;
            let q = src.u32_at(p);
            let b0 = q as u8;
            let b1 = (q >> 8) as u8;
            // In TS, `>` may close nested type args including `Foo<T>= 1`, so fusing can diverge.
            // Cheap inline byte checks handle this
            if b0 == b'>'
                && is_ts
                && (b1 == b'>' || (b1 == b'=' && p > 0 && !is_ws(src[p - 1], cpu)))
            {
                let kw_final = (w & !(KWB - 1)) << 6;
                let g = lanes.disambiguate.gt_run_split(lex, kw_final, p, run, cpu);
                if g != 0 {
                    // Only `>`s stay split; the rest still munches, or `>>&&` would emit two `&`s.
                    cursor = munch_walk(lex, p + g, lanes, cpu);
                    continue;
                }
            }
            // A << may open two lists; longer runs always reach munch_walk.
            if b1 == b'<' && is_ts && (b0 == b'<' || (q >> 16) as u8 == b'<') {
                cursor = munch_walk(lex, p, lanes, cpu);
                continue;
            }
            if run == 2 {
                let key = q & 0xFFFF;
                let pack = opmap_pack2(key, cpu);
                let mut ok = pack as u16 == q as u16;
                let kk = (pack >> 24) as u8;
                ok &= !((kk == tk!(OptionalChain)) && is_digit((q >> 16) as u8, cpu));
                let hm: u8 = 0u8.wrapping_sub(ok as u8);
                kinds.set(p, (kinds.get(p) & !hm) | (kk & hm));
                let clr = (ok as u64) << ((p + 1) & 63);
                *token_start.as_ptr().add((p + 1) >> 6) &= !clr;
                cursor = p + 1 + ok as usize;
                continue;
            }
            if run > 3 {
                cursor = munch_walk(lex, p, lanes, cpu);
                continue;
            }
            let b2 = (q >> 16) as u8;
            let key3 = q & 0xFF_FFFF;
            let p3 = opmap_pack3(key3, cpu);
            let ok3 = ((p3 ^ q) & 0xFF_FFFF) == 0;
            let key2a = q & 0xFFFF;
            let pa = opmap_pack2(key2a, cpu);
            let ka = (pa >> 24) as u8;
            let mut ok2a = pa as u16 == q as u16;
            ok2a &= !((ka == tk!(OptionalChain)) && is_digit(b2, cpu));
            let key2b = key3 >> 8;
            let pb = opmap_pack2(key2b, cpu);
            let kb = (pb >> 24) as u8;
            let mut ok2b = pb as u16 == key2b as u16;
            ok2b &= !((kb == tk!(OptionalChain)) && is_digit((q >> 24) as u8, cpu));
            let c1 = ok3 | ok2a; // token at `p` is 2+ bytes
            let sel2b = !c1 & ok2b; // 2-byte token starts at `p + 1`
            let c2 = ok3 | sel2b; // `p + 2` is not a token start
            let m0: u8 = 0u8.wrapping_sub(c1 as u8);
            let k0v: u8 = if ok3 { (p3 >> 24) as u8 } else { ka };
            kinds.set(p, (kinds.get(p) & !m0) | (k0v & m0));
            let m1: u8 = 0u8.wrapping_sub(sel2b as u8);
            kinds.set(p + 1, (kinds.get(p + 1) & !m1) | (kb & m1));
            let clr1 = (c1 as u64) << ((p + 1) & 63);
            *token_start.as_ptr().add((p + 1) >> 6) &= !clr1;
            let clr2 = (c2 as u64) << ((p + 2) & 63);
            *token_start.as_ptr().add((p + 2) >> 6) &= !clr2;
            // `run == 3` in this arm
            cursor = p + 2 + c2 as usize;
        }
    }
    cursor
}

#[cpu_multiversion]
unsafe fn glue_number(lex: Lex, mut p: usize, lanes: &mut Lanes, cpu: impl Cpu) -> usize {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, word, op_char, .. } = *bitmaps;
    let is_ts = options.ts;
    loop {
        let number_end = scan_number(src, p, cpu);
        kinds.set(p, tk!(Number) + (src[number_end - 1] == b'n') as u8);
        if number_end > p + 1 {
            token_start.clear_range(p + 1, number_end - 1, cpu);
            op_char.clear_range(p, number_end - 1, cpu);
        }
        if number_end >= src.len() {
            return src.len();
        }
        token_start.set(number_end, cpu);
        let c = src[number_end];
        // A word char abutting the number end (`1.5n`, `3in`, `0b12`, `1π`)
        // is a spec-invalid adjacency. Never true on valid input, so the
        // whole arm is cold.
        if is_word(c, cpu) {
            if c < 0x80 {
                // A surviving `n` is a misplaced bigint suffix; scan_number
                // consumes legal ones. Token spans are unchanged either way.
                let code = if c == b'n' {
                    DiagCode::InvalidBigint
                } else {
                    DiagCode::InvalidNumericLiteral
                };
                lanes.push_num_end_diag(src.bytes(), number_end, code);
                if is_digit(c, cpu) {
                    // Digit invalid for the radix (`0b12`) or after a bigint
                    // suffix (`1n2`): keep gluing from it.
                    p = number_end;
                    continue;
                }
                // This token start postdates the kwc mask built in the word
                // prelude, so resolve any keyword kind inline (`3in` is `3`
                // + KW_IN). Same inputs as kw_verify_batch: length from the
                // word bitmap, no match right after a member `.` (a number
                // ending in `.` is that dot-run's first dot).
                if src[number_end - 1] != b'.' {
                    let kk = kw_match_word(is_ts, src, word, number_end, cpu);
                    if kk != TokenKind::Ident {
                        kinds.set(number_end, kk as u8);
                    }
                }
            } else {
                // Cold decode; flags only a real Unicode IdentifierStart.
                lanes.push_num_end_diag_unicode(src.bytes(), number_end);
            }
            return number_end; // token start already set at number_end
        }
        if number_end + 1 < src.len() && is_op_char(c, cpu) && op_char.get(number_end + 1, cpu) {
            // A numeric literal type ends its type argument list here (`Map<A, 1.5>>`); without this the number's own munch
            // reaches the `>` run before `coalesce` ever raises it as an event.
            if c == b'>' && is_ts && matches!(src[number_end + 1], b'>' | b'=') {
                let end = op_char.next0(number_end, src.len(), cpu);
                // Keyword kinds are final only below this window's batch.
                let kw_final = ((number_end >> 6) & !(KWB - 1)) << 6;
                let g = lanes.disambiguate.gt_run_split(
                    lex,
                    kw_final,
                    number_end,
                    end - number_end,
                    cpu,
                );
                if g != 0 {
                    // The split `>`s stay single tokens, but whatever borders on them is still an operator run and has to be
                    // munched, or a following `&&` / `??` / `**` is emitted a byte at a time.
                    return munch_walk(lex, number_end + g, lanes, cpu);
                }
            }
            let q = munch_walk(lex, number_end, lanes, cpu);
            if q < src.len()
                && src[q] == b'.'
                && is_digit(src[q + 1], cpu)
                && token_start.get(q, cpu)
            {
                p = q;
                continue;
            }
            return q;
        }
        return number_end;
    }
}

/// Cold path. Kept out of line so it does not slow the coalesce loop.
#[cpu_multiversion]
#[inline(never)]
unsafe fn munch_walk(lex: Lex, mut pos: usize, lanes: &Lanes, cpu: impl Cpu) -> usize {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, op_char, .. } = *bitmaps;
    let is_ts = options.ts;
    let end = op_char.next0(pos, src.len(), cpu);
    while end - pos >= 2 {
        token_start.set(pos, cpu);
        let rem = end - pos;
        let bytes = src.u32_at(pos).to_le_bytes();
        let (opk, opl) = opmap_longest(bytes, rem as u32, cpu);
        if let Some(opk) = opk {
            // Two list openers, not a shift: Array<<T>(x: T) => T>, a?.<<T>(x: T) => T>(b).
            if is_ts && opk == TokenKind::LShift && lanes.disambiguate.lt_run_split(lex, pos, cpu) {
                pos += 2;
                continue;
            }
            kinds.set(pos, opk as u8);
            let mut j = 1usize;
            while j < opl as usize {
                token_start.clear(pos + j, cpu);
                j += 1;
            }
            pos += opl as usize;
        } else {
            pos += 1;
        }
    }
    if end - pos == 1 {
        token_start.set(pos, cpu);
    }
    pos
}
