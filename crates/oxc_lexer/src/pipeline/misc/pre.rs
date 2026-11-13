use oxc_simd::{BASELINE, Cpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::Bitmap, bytes::is_id_start, lanes::Lanes, scan::scan_ident_esc, source::Source,
};

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn misc_pre(
    src: Source,
    nb: usize,
    st: Bitmap,
    word: Bitmap,
    misc: Bitmap,
    kind: *mut u8,
    vutf8: bool,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    if !misc.any(nb, cpu) {
        return 0;
    }

    if vutf8 {
        misc_pre_impl::<true>(src, st, word, misc, kind, lanes, cpu)
    } else {
        misc_pre_impl::<false>(src, st, word, misc, kind, lanes, cpu)
    }
}

/// `VUTF8` (`LexOptions::validate_utf8`) adds UTF-8 well-formedness
/// validation to the non-ASCII walk. Monomorphized so the default `false`
/// copy pays nothing for it.
///
/// `#[inline(always)]` so it gets compiled as part of each of [`misc_pre`]'s entry points.
#[cpu_multiversion]
#[inline(always)]
unsafe fn misc_pre_impl<const VUTF8: bool>(
    src: Source,
    st: Bitmap,
    word: Bitmap,
    misc: Bitmap,
    kind: *mut u8,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let mut nesc = 0usize;
    let nw = (src.len() + 63) >> 6;
    // VUTF8 only: continuation bits spilling into the next word, and the
    // once-per-file latch for the UTF-8 diagnostic (parser parity; also
    // bounds the diag Vec on binary input).
    let mut skip_lo: u64 = 0;
    let mut utf8_bad = false;
    let mut w = 0usize;
    loop {
        // Skip empty words in a tight loop, as the walk below keeps the word index on the stack
        let from = w;
        while w < nw && misc.block(w, cpu) == 0 {
            w += 1;
        }
        if w == nw {
            break;
        }
        if VUTF8 && w != from {
            // A skipped word consumed the carried continuation bits
            skip_lo = 0;
        }
        let blk = w;
        w += 1;
        let mut m = if VUTF8 { misc.block(blk, cpu) & !skip_lo } else { misc.block(blk, cpu) };
        if VUTF8 {
            skip_lo = 0;
        }
        while m != 0 {
            let bit = m.trailing_zeros() as usize;
            m &= m - 1;
            let p = (blk << 6) + bit;
            // Never taken: classify clears every bit past the end, but the compiler cannot know.
            if p >= src.len() {
                break;
            }
            let c = src[p];
            // Multi-byte Unicode whitespace was bulk-classified as an
            // identifier char; re-mark it as a whitespace token boundary.
            // Continuation bytes and non-ws leads fall through cheaply.
            if c >= 0x80 {
                if VUTF8 {
                    let (ok, cont) = utf8_seq_check(src, p, cpu);
                    if ok {
                        // Consume the verified continuation bits so they are
                        // not re-visited; a continuation still visible to the
                        // walk had no valid lead - that is the stray-
                        // continuation check.
                        let cm: u128 = (((1u128 << cont) - 1) << 1) << (p & 63);
                        m &= !(cm as u64);
                        skip_lo |= (cm >> 64) as u64;
                    } else if !utf8_bad {
                        utf8_bad = true;
                        // Span = the maximal invalid subpart; one diag per
                        // file, context-free (fires inside strings too).
                        lanes.push_diag(p as u32, (1 + cont) as u32, DiagCode::InvalidUtf8);
                        continue;
                    } else {
                        continue;
                    }
                }
                let len = src.unicode_ws_len_at(p, cpu);
                if len != 0 {
                    *kind.add(p) = tk!(Whitespace);
                    word.clear_range(p, p + len - 1, cpu);
                    st.set(p, cpu);
                    st.clear_range(p + 1, p + len - 1, cpu);
                    if p + len < src.len() {
                        st.set(p + len, cpu);
                    }
                    lanes.out.irregular_whitespaces.push(Span::new(p as u32, (p + len) as u32));
                } else if (0xC2..=0xF4).contains(&c) {
                    // Non-whitespace lead: record the position only. The
                    // identifier-char check is deferred to drain, where leads
                    // inside literal tokens drop before ever paying for it.
                    lanes.unicode_leads.push(p as u32);
                }
                continue;
            }
            // Vertical tab or form feed, the only misc bytes below 0x20.
            if c < 0x20 {
                lanes.out.irregular_whitespaces.push(Span::new(p as u32, p as u32 + 1));
                continue;
            }
            if !st.get(p, cpu) {
                continue;
            }
            if c == b'#' {
                if p == 0 && src.len() > 1 && src[1] == b'!' {
                    continue;
                }
                // Accept an id-start or a leading unicode escape, so a
                // private name whose first char is itself an escape forms
                // one token instead of `#` + escape.
                let c1 = src[p + 1];
                if !(is_id_start(c1, cpu) || (c1 == b'\\' && src[p + 2] == b'u')) {
                    continue;
                }
                let e0 = word.next0(p + 1, src.len(), cpu);
                let e =
                    if src[e0] == b'\\' && src[e0 + 1] == b'u' && !holds_unicode_ws(src, p + 1, e0)
                    {
                        *kind.add(p) = tk!(PrivateIdentEscaped);
                        nesc += 1;
                        scan_ident_esc(src, e0, cpu)
                    } else {
                        *kind.add(p) = tk!(PrivateIdent);
                        e0
                    };
                st.clear_range(p + 1, e - 1, cpu);
            } else {
                if src[p + 1] != b'u' {
                    continue;
                }
                *kind.add(p) = tk!(IdentEscaped);
                nesc += 1;
                let e = scan_ident_esc(src, p, cpu);
                st.clear_range(p + 1, e - 1, cpu);
            }
        }
    }
    nesc
}

/// Drop the irregular whitespace candidates that carve put inside a literal, comment or JSX text.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
pub fn retain_code_whitespace(st: &[u64], kind: &[u8], spans: &mut Vec<Span>) {
    spans.retain(|span| {
        let p = span.start as usize;
        let mut w = p >> 6;
        let mut bits = st[w] & (u64::MAX >> (63 - (p & 63)));
        while bits == 0 {
            let Some(prev) = w.checked_sub(1) else { return false };
            w = prev;
            bits = st[w];
        }
        kind[(w << 6) + 63 - bits.leading_zeros() as usize] == tk!(Whitespace)
    });
}

/// Cold: whether src[a..b] holds a multi-byte WhiteSpace or LineTerminator.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
fn holds_unicode_ws(src: Source, a: usize, b: usize) -> bool {
    let cpu = BASELINE;
    (a..b).any(|i| src[i] >= 0x80 && src.unicode_ws_len_at(i, cpu) != 0)
}

/// Validate the UTF-8 sequence led by the byte at `p` (>= 0x80). Returns
/// `(valid, cont)` where `cont` is the count of range-valid continuation
/// bytes after the lead. Second-byte ranges are per-lead, so overlongs,
/// surrogates, and out-of-range sequences report the same maximal subpart
/// as `std::str::from_utf8`'s `error_len`.
#[cpu_multiversion]
#[inline]
fn utf8_seq_check(src: Source, p: usize, _cpu: impl Cpu) -> (bool, usize) {
    let (need, lo, hi): (usize, u8, u8) = match src[p] {
        0xC2..=0xDF => (1, 0x80, 0xBF),
        0xE0 => (2, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE..=0xEF => (2, 0x80, 0xBF),
        0xED => (2, 0x80, 0x9F),
        0xF0 => (3, 0x90, 0xBF),
        0xF1..=0xF3 => (3, 0x80, 0xBF),
        0xF4 => (3, 0x80, 0x8F),
        // 0x80..=0xBF stray continuation, 0xC0/0xC1 overlong leads, 0xF5..=0xFF
        _ => return (false, 0),
    };
    let mut cont = 0usize;
    if p + 1 < src.len() && src[p + 1] >= lo && src[p + 1] <= hi {
        cont = 1;
        while cont < need && p + 1 + cont < src.len() {
            let b = src[p + 1 + cont];
            if !(0x80..=0xBF).contains(&b) {
                break;
            }
            cont += 1;
        }
    }
    (cont == need, cont)
}
