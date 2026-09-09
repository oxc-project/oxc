use crate::{error::diag_code, lanes::Lanes, tables::is_id_start};

use super::super::{
    IDENT_ESC, PRIV_IDENT, PRIV_IDENT_ESC, WS,
    bitmap::{bm_clear_range, bm_get, bm_next0, bm_set},
    classify::unicode_ws_len,
    find::scan_ident_esc,
};

/// `VUTF8` (`LexOptions::validate_utf8`) adds UTF-8 well-formedness
/// validation to the non-ASCII walk. Monomorphized so the default `false`
/// copy pays nothing for it.
pub unsafe fn misc_pre<const VUTF8: bool>(
    src: *const u8,
    n: usize,
    st: *mut u64,
    word: *mut u64,
    misc: *const u64,
    kind: *mut u8,
    lanes: &mut Lanes,
) -> usize {
    let mut nesc = 0usize;
    let nw = (n + 63) >> 6;
    // VUTF8 only: continuation bits spilling into the next word, and the
    // once-per-file latch for the UTF-8 diagnostic (parser parity; also
    // bounds the diag Vec on binary input).
    let mut skip_lo: u64 = 0;
    let mut utf8_bad = false;
    for w in 0..nw {
        let mut m = if VUTF8 { *misc.add(w) & !skip_lo } else { *misc.add(w) };
        if VUTF8 {
            skip_lo = 0;
        }
        while m != 0 {
            let bit = m.trailing_zeros() as usize;
            m &= m - 1;
            let p = (w << 6) + bit;
            let c = *src.add(p);
            // Multi-byte Unicode whitespace was bulk-classified as an
            // identifier char; re-mark it as a whitespace token boundary.
            // Continuation bytes and non-ws leads fall through cheaply.
            if c >= 0x80 {
                if VUTF8 {
                    let (ok, cont) = utf8_seq_check(src, p, n);
                    if ok {
                        // Consume the verified continuation bits so they are
                        // not re-visited; a continuation still visible to the
                        // walk had no valid lead — that is the stray-
                        // continuation check.
                        let cm: u128 = (((1u128 << cont) - 1) << 1) << (p & 63);
                        m &= !(cm as u64);
                        skip_lo |= (cm >> 64) as u64;
                    } else if !utf8_bad {
                        utf8_bad = true;
                        // Span = the maximal invalid subpart; one diag per
                        // file, context-free (fires inside strings too).
                        lanes.push_diag(p as u32, (1 + cont) as u32, diag_code::INVALID_UTF8);
                        continue;
                    } else {
                        continue;
                    }
                }
                let len = unicode_ws_len(src, p);
                if len != 0 {
                    *kind.add(p) = WS;
                    bm_clear_range(word, p, p + len - 1);
                    bm_set(st, p);
                    bm_clear_range(st, p + 1, p + len - 1);
                    if p + len < n {
                        bm_set(st, p + len);
                    }
                } else if (0xC2..=0xF4).contains(&c) {
                    // Non-whitespace lead: record the position only. The
                    // identifier-char check is deferred to drain, where leads
                    // inside literal tokens drop before ever paying for it.
                    lanes.unicode_leads.push(p as u32);
                }
                continue;
            }
            if !bm_get(st, p) {
                continue;
            }
            if c == b'#' {
                if p == 0 && n > 1 && *src.add(1) == b'!' {
                    continue;
                }
                // Accept an id-start or a leading unicode escape, so a
                // private name whose first char is itself an escape forms
                // one token instead of `#` + escape.
                let c1 = *src.add(p + 1);
                if !(is_id_start(c1) || (c1 == b'\\' && *src.add(p + 2) == b'u')) {
                    continue;
                }
                let e0 = bm_next0(word, p + 1, n);
                let e = if *src.add(e0) == b'\\' && *src.add(e0 + 1) == b'u' {
                    *kind.add(p) = PRIV_IDENT_ESC;
                    scan_ident_esc(src, n, e0)
                } else {
                    *kind.add(p) = PRIV_IDENT;
                    e0
                };
                bm_clear_range(st, p + 1, e - 1);
            } else {
                if *src.add(p + 1) != b'u' {
                    continue;
                }
                *kind.add(p) = IDENT_ESC;
                nesc += 1;
                let e = scan_ident_esc(src, n, p);
                bm_clear_range(st, p + 1, e - 1);
            }
        }
    }
    nesc
}

/// Validate the UTF-8 sequence led by the byte at `p` (>= 0x80). Returns
/// `(valid, cont)` where `cont` is the count of range-valid continuation
/// bytes after the lead. Second-byte ranges are per-lead, so overlongs,
/// surrogates, and out-of-range sequences report the same maximal subpart
/// as `std::str::from_utf8`'s `error_len`.
#[inline]
unsafe fn utf8_seq_check(src: *const u8, p: usize, n: usize) -> (bool, usize) {
    let (need, lo, hi): (usize, u8, u8) = match *src.add(p) {
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
    if p + 1 < n && *src.add(p + 1) >= lo && *src.add(p + 1) <= hi {
        cont = 1;
        while cont < need && p + 1 + cont < n {
            let b = *src.add(p + 1 + cont);
            if !(0x80..=0xBF).contains(&b) {
                break;
            }
            cont += 1;
        }
    }
    (cont == need, cont)
}
