use std::mem::MaybeUninit;

use oxc_simd::{Cpu, cpu_multiversion, prefetch};

use crate::{
    error::DiagCode,
    token::{TokenKind, tk},
};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    bytes::{is_digit, is_word, is_ws},
    disambiguate::DisambiguateState,
    kinds::Kinds,
    lanes::Lanes,
    lex::{Lex, LexState},
    operators::{is_op_char, opmap_longest, opmap_pack2, opmap_pack3},
    scan::scan_number,
};

mod keywords;

pub use keywords::KWB;
use keywords::{kw_flush, kw_match_word};

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn coalesce(
    lex: Lex,
    kw_candidates: *mut u32,
    events: *mut u32,
    state: &mut LexState,
    cpu: impl Cpu,
) {
    let Lex { src, bitmaps, kinds, options } = lex;
    let block_count = src.block_count();
    let Bitmaps { token_start, word, keyword_initial, op_char, digit, dot, .. } =
        bitmaps.first(block_count + 1);
    let is_ts = options.ts;
    state.disambiguate.restart(cpu);
    // Events are collected branchlessly and handled per batch. Unset masks are never read
    let mut rare_masks = [const { MaybeUninit::<u64>::uninit() }; KWB];
    let mut event_count = 0usize;
    let mut prev_empty = false;
    let mut op_prev: u64 = 0;
    let mut dot_prev: u64 = 0;
    let mut cursor: usize = 0;
    let mut candidate_count = 0usize;
    for w in 0..block_count {
        let has_next = w + 1 < block_count;
        let masks = BlockMasks {
            op: op_char.block(w, cpu),
            op_next: if has_next { op_char.block(w + 1, cpu) } else { 0 },
            digit: digit.block(w, cpu),
            digit_next: if has_next { digit.block(w + 1, cpu) } else { 0 },
            dot: dot.block(w, cpu),
            token_start: token_start.block(w, cpu),
            word: word.block(w, cpu),
            word_next: if has_next { word.block(w + 1, cpu) } else { 0 },
        };
        // The whole keywords stage lives inside this walk: the candidate
        // filter (token-start keyword-initial word char, next char also
        // word, not right after a member `.`, length <= 10 by the word-local
        // run filter) computes from values already in registers, candidates
        // extract into kw_candidates, and the batch verifies every KWB words while
        // this window's src/kind lines are still warm. The only token start
        // minted behind the filter is glue_number's invalid adjacency, which
        // resolves its keyword kind inline with the same inputs, so the two
        // writes agree even when both fire.
        let candidates = masks.keyword_candidates(keyword_initial.block(w, cpu), dot_prev);
        dot_prev = masks.dot;
        let op_runs = masks.operator_run_starts(op_prev);
        let dot_digits = masks.dot_digit_starts();
        let mut digit_runs = masks.token_start & masks.digit;
        if digit_runs != 0 {
            // Runs ending at a word char or dot need gluing, found by carrying back reversed
            let glue_after = !masks.digit & (masks.digit << 1) & (masks.word | masks.dot);
            let digits_rev = masks.digit.reverse_bits();
            let (sum, out) = digits_rev.overflowing_add((glue_after >> 1).reverse_bits());
            let mut glue = ((sum & !digits_rev).reverse_bits() << 1) | u64::from(out);
            if masks.digit >> 63 != 0 {
                let tail = (!masks.digit_next).trailing_zeros();
                let dot_next = if has_next { dot.block(w + 1, cpu) } else { 0 };
                if tail == 64 || ((masks.word_next | dot_next) >> tail) & 1 != 0 {
                    glue |= 1u64 << (64 - (!masks.digit).leading_zeros());
                }
            }
            digit_runs &= glue;
        }
        let event_bits = op_runs | digit_runs | dot_digits;
        let rare = digit_runs | dot_digits;
        op_prev = masks.op;
        // Only skip within runs of empty blocks, so an isolated empty block never mispredicts
        let empty = (candidates | event_bits) == 0;
        if !(empty && prev_empty) {
            let base = (w << 6) as u32;
            prefetch(src.as_ptr().add(base as usize), cpu);
            prefetch(kinds.as_ptr().add(base as usize) as *const u8, cpu);
            candidate_count = push_positions::<8>(kw_candidates, candidate_count, base, candidates);
            rare_masks[w & (KWB - 1)].write(rare);
            event_count = push_positions::<2>(events, event_count, base, event_bits);
        }
        prev_empty = empty;
        if w & (KWB - 1) == KWB - 1 {
            if event_count != 0 {
                cursor = coalesce_events(lex, state, events, event_count, &rare_masks, cursor, cpu);
                event_count = 0;
            }
            kw_flush(is_ts, src, word, kinds, kw_candidates, candidate_count, cpu);
            candidate_count = 0;
        }
    }
    if event_count != 0 {
        coalesce_events(lex, state, events, event_count, &rare_masks, cursor, cpu);
    }
    kw_flush(is_ts, src, word, kinds, kw_candidates, candidate_count, cpu);
}

/// One block's bitmap words, and the next block's where a run carries over.
#[derive(Clone, Copy)]
struct BlockMasks {
    op: u64,
    op_next: u64,
    digit: u64,
    digit_next: u64,
    dot: u64,
    token_start: u64,
    word: u64,
    word_next: u64,
}

impl BlockMasks {
    /// Token starts that can begin a keyword: a keyword initial, 2 to 10 word chars, not a member.
    #[inline(always)]
    fn keyword_candidates(self, keyword_initial: u64, dot_prev: u64) -> u64 {
        let Self { token_start, word, word_next, dot, .. } = self;
        // The first dot of each dot run, and one ending the previous block.
        let run_dot = dot & !((dot << 1) | (dot_prev >> 63));
        let carried_dot = ((dot_prev >> 63) & !(dot_prev >> 62)) & 1;
        let mut candidates = token_start
            & word
            & keyword_initial
            & ((word >> 1) | (word_next << 63))
            & !((run_dot << 1) | carried_dot);
        let r2 = word & (word >> 1);
        let r4 = r2 & (r2 >> 2);
        let r8 = r4 & (r4 >> 4);
        // No keyword is longer than ten chars.
        candidates &= !(r8 & (r2 >> 8) & (word >> 10));
        candidates
    }

    /// Token starts that begin a run of two or more operator chars.
    #[inline(always)]
    fn operator_run_starts(self, op_prev: u64) -> u64 {
        let Self { token_start, op, op_next, .. } = self;
        op & !((op << 1) | (op_prev >> 63)) & ((op >> 1) | (op_next << 63)) & token_start
    }

    /// Token-start dots followed by a digit, the numbers like .5.
    #[inline(always)]
    fn dot_digit_starts(self) -> u64 {
        self.token_start & self.dot & ((self.digit >> 1) | (self.digit_next << 63))
    }
}

/// Write mask's bit positions to out[len..], always filling N slots; returns the new len.
#[inline(always)]
unsafe fn push_positions<const N: usize>(
    out: *mut u32,
    len: usize,
    base: u32,
    mut mask: u64,
) -> usize {
    let count = mask.count_ones() as usize;
    for j in 0..N {
        *out.add(len + j) = base + mask.trailing_zeros();
        mask &= mask.wrapping_sub(1);
    }
    if count > N {
        let mut at = len + N;
        while mask != 0 {
            *out.add(at) = base + mask.trailing_zeros();
            at += 1;
            mask &= mask - 1;
        }
    }
    len + count
}

/// Handle a batch's events in order. Rereading op_char is safe: events only clear it below cursor.
#[cpu_multiversion]
#[inline(never)]
unsafe fn coalesce_events(
    lex: Lex,
    state: &mut LexState,
    events: *const u32,
    count: usize,
    rare_masks: &[MaybeUninit<u64>; KWB],
    mut cursor: usize,
    cpu: impl Cpu,
) -> usize {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, op_char, .. } = *bitmaps;
    let LexState { lanes, disambiguate, .. } = state;
    let is_ts = options.ts;
    let block_count = src.block_count();
    for i in 0..count {
        let p = *events.add(i) as usize;
        if p < cursor {
            continue;
        }
        let (w, bit) = (p >> 6, p & 63);
        let rare = rare_masks[w & (KWB - 1)].assume_init();
        let has_next = w + 1 < block_count;
        if (rare >> bit) & 1 != 0 {
            cursor = glue_number(lex, p, lanes, disambiguate, cpu);
        } else {
            let op = *op_char.as_ptr().add(w);
            let op_next = if has_next { *op_char.as_ptr().add(w + 1) } else { 0 };
            let y2 = (op >> bit) | ((op_next << (63 - bit)) << 1);
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
                let g = disambiguate.gt_run_split(lex, kw_final, p, run, cpu);
                if g != 0 {
                    // Only `>`s stay split; the rest still munches, or `>>&&` would emit two `&`s.
                    cursor = munch_walk(lex, p + g, disambiguate, cpu);
                    continue;
                }
            }
            // A << may open two lists; longer runs always reach munch_walk.
            if b1 == b'<' && is_ts && (b0 == b'<' || (q >> 16) as u8 == b'<') {
                cursor = munch_walk(lex, p, disambiguate, cpu);
                continue;
            }
            if run == 2 {
                cursor = fuse_pair(kinds, token_start, p, q, cpu);
                continue;
            }
            if run > 3 {
                cursor = munch_walk(lex, p, disambiguate, cpu);
                continue;
            }
            cursor = fuse_triple(kinds, token_start, p, q, cpu);
        }
    }
    cursor
}

/// Fuse the operator pair at p into one token if it spells one; returns the cursor.
#[cpu_multiversion]
#[inline(always)]
unsafe fn fuse_pair(kinds: Kinds, token_start: Bitmap, p: usize, q: u32, cpu: impl Cpu) -> usize {
    let key = q & 0xFFFF;
    let pack = opmap_pack2(key, cpu);
    let mut ok = pack as u16 == q as u16;
    let kk = (pack >> 24) as u8;
    ok &= !((kk == tk!(OptionalChain)) && is_digit((q >> 16) as u8, cpu));
    let hm: u8 = 0u8.wrapping_sub(ok as u8);
    let k = kinds.as_ptr().add(p);
    *k = (*k & !hm) | (kk & hm);
    let clr = (ok as u64) << ((p + 1) & 63);
    *token_start.as_ptr().add((p + 1) >> 6) &= !clr;
    p + 1 + ok as usize
}

/// Fuse the 3-char operator run at p into one token, or a pair at p or p + 1; returns the cursor.
#[cpu_multiversion]
#[inline(always)]
unsafe fn fuse_triple(kinds: Kinds, token_start: Bitmap, p: usize, q: u32, cpu: impl Cpu) -> usize {
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
    let c1 = ok3 | ok2a; // the token at p is 2 or more bytes
    let sel2b = !c1 & ok2b; // a 2-byte token starts at p + 1
    let c2 = ok3 | sel2b; // p + 2 is not a token start
    let m0: u8 = 0u8.wrapping_sub(c1 as u8);
    let k0v: u8 = if ok3 { (p3 >> 24) as u8 } else { ka };
    let k = kinds.as_ptr().add(p);
    *k = (*k & !m0) | (k0v & m0);
    let m1: u8 = 0u8.wrapping_sub(sel2b as u8);
    *k.add(1) = (*k.add(1) & !m1) | (kb & m1);
    let clr1 = (c1 as u64) << ((p + 1) & 63);
    *token_start.as_ptr().add((p + 1) >> 6) &= !clr1;
    let clr2 = (c2 as u64) << ((p + 2) & 63);
    *token_start.as_ptr().add((p + 2) >> 6) &= !clr2;
    p + 2 + c2 as usize
}

#[cpu_multiversion]
unsafe fn glue_number(
    lex: Lex,
    mut p: usize,
    lanes: &mut Lanes,
    disambiguate: &mut DisambiguateState,
    cpu: impl Cpu,
) -> usize {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, word, op_char, .. } = *bitmaps;
    let is_ts = options.ts;
    loop {
        let number_end = scan_number(src, p, cpu);
        let bigint = src[number_end - 1] == b'n';
        kinds.set(p, if bigint { TokenKind::BigInt } else { TokenKind::Number });
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
                // This token start postdates the keyword candidates built in the word
                // prelude, so resolve any keyword kind inline (`3in` is `3`
                // + KW_IN). Same inputs as kw_verify_batch: length from the
                // word bitmap, no match right after a member `.` (a number
                // ending in `.` is that dot-run's first dot).
                if src[number_end - 1] != b'.' {
                    let kk = kw_match_word(is_ts, src, word, number_end, cpu);
                    if kk != TokenKind::Ident {
                        kinds.set(number_end, kk);
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
                let g = disambiguate.gt_run_split(lex, kw_final, number_end, end - number_end, cpu);
                if g != 0 {
                    // The split `>`s stay single tokens, but whatever borders on them is still an operator run and has to be
                    // munched, or a following `&&` / `??` / `**` is emitted a byte at a time.
                    return munch_walk(lex, number_end + g, disambiguate, cpu);
                }
            }
            let q = munch_walk(lex, number_end, disambiguate, cpu);
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
fn munch_walk(lex: Lex, mut pos: usize, disambiguate: &DisambiguateState, cpu: impl Cpu) -> usize {
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
            if is_ts && opk == TokenKind::LShift && disambiguate.lt_run_split(lex, pos, cpu) {
                pos += 2;
                continue;
            }
            kinds.set(pos, opk);
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
