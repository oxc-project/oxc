//! Bracket matching by scanning raw source bytes.
//!
//! `common` matches brackets by walking back token by token ([`match_delim_back`], [`angle_match_back`]).
//! Over long distances that is slow, so the scans here read source bytes instead.
//! The token-start bitmap hides bytes inside literals and comments,
//! and the operator bitmap hides the angle brackets of JSX tags.
//!
//! Backward:
//! - [`gt_run_closes_type_args`] goes from a run of `>`s back to the `<` which the run would close.
//! - [`enclosing_opener`] goes back to the nearest unclosed `(`, `[`, `{` or `<`.
//!
//! Forward:
//! - [`angle_close_fwd`] and [`paren_close_fwd`] go from an opener to its closer.
//! - [`lt_run_opens_type_args`] and [`arrow_after_paren_group`] check for the shape of an arrow function
//!   after a `<<` or a `(`, as in `Array<<T>(x: T) => T>` or `(x): T => x`.
//!
//! Every scan has a length limit, so pathological input can't make it slow.
//!
//! [`match_delim_back`]: super::super::common::match_delim_back
//! [`angle_match_back`]: super::super::common::angle_match_back

use crate::tables::{is_id_start, is_ws};

use super::super::super::{TMPL_MIDDLE, TMPL_TAIL, bitmap::bm_get};

use super::super::common::kind_at;

pub(super) const ENCLOSING_SCAN_CAP: usize = 1 << 16;

/// Cap for scanning `>` runs: matching `<` is within 100 bytes
/// Hitting the cap returns `None` (fuse), so it can only widen the residual, never split a shift.
pub(super) const GT_SCAN_CAP: usize = 1 << 16;

/// Bytes `gt_run_closes_type_args` reacts to. Everything else is skipped without touching a bitmap.
static GT_SCAN_DELIM: [bool; 256] = {
    let mut t = [false; 256];
    t[b'<' as usize] = true;
    t[b'>' as usize] = true;
    t[b'(' as usize] = true;
    t[b')' as usize] = true;
    t[b'[' as usize] = true;
    t[b']' as usize] = true;
    t[b'{' as usize] = true;
    t[b'}' as usize] = true;
    t[b';' as usize] = true;
    t
};

pub(super) enum Encl {
    Open(usize, u8),
    Top,
    Capped,
}

/// The `<` opening the outermost of the `run` nested type-argument lists that
/// the run of `>` bytes at `gt` would close, or `None` when the region is not
/// delimiter-balanced.
///
/// Reads source bytes backward rather than walking tokens: the matching `<`
/// is a hundred bytes away, but the token walk that finds it has to cross
/// every `(`/`[`/`{` in between and match each one back to its opener, which
/// measured ~5,000 cycles a site on `ts_zod.ts` (+1.08 cyc/B). Angles are
/// gated on `opch & st` so literal interiors and JSX tag punctuation are
/// invisible; the bracket counters are gated on `st` alone.
///
/// The counters are what separate the two readings. A type-argument list is
/// always delimiter-balanced, so an unmatched `(`/`[`/`{`, or a `;` outside
/// any of them, proves the region is not one — that is what rejects
/// `(a << 3) | (a >>> 29)`, `o[(y = e) >> 2]` and `x >>= 8`. A `<<` whose
/// second byte is no longer a token start is a fused shift; one that
/// `lt_run_split` already split counts as two openers.
pub(super) unsafe fn gt_run_closes_type_args(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    gt: usize,
    run: usize,
) -> Option<usize> {
    let mut depth = run as i32;
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let lo = gt.saturating_sub(GT_SCAN_CAP);
    let mut i = gt;
    while i > lo {
        i -= 1;
        let c = *src.add(i);
        if !GT_SCAN_DELIM[c as usize] {
            continue;
        }
        if !bm_get(st, i) {
            continue;
        }
        match c {
            b'>' => {
                if !bm_get(opch, i) {
                    continue;
                }
                if i > 0 && *src.add(i - 1) == b'=' {
                    continue;
                }
                let nx = *src.add(i + 1);
                if (nx == b'>' || nx == b'=') && !bm_get(st, i + 1) {
                    return None;
                }
                depth += 1;
            }
            b'<' => {
                if !bm_get(opch, i) {
                    continue;
                }
                let nx = *src.add(i + 1);
                if nx == b'=' || (nx == b'<' && !bm_get(st, i + 1)) {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    return (par == 0 && brk == 0 && brc == 0).then_some(i);
                }
            }
            b')' => par += 1,
            b'(' => {
                par -= 1;
                if par < 0 {
                    return None;
                }
            }
            b']' => brk += 1,
            b'[' => {
                brk -= 1;
                if brk < 0 {
                    return None;
                }
            }
            b'}' => {
                // A substitution-closing `}` is the start of the next
                // template segment, and its `${` was swallowed by the
                // preceding one — counting it would leave every
                // `Array<Map<A, `p${s}q`>>` looking brace-unbalanced.
                let kk = kind_at(kind, i);
                if kk == TMPL_MIDDLE || kk == TMPL_TAIL {
                    continue;
                }
                brc += 1;
            }
            b'{' => {
                brc -= 1;
                if brc < 0 {
                    return None;
                }
            }
            _ => {
                if par == 0 && brk == 0 && brc == 0 {
                    return None;
                }
            }
        }
    }
    None
}

pub(super) unsafe fn enclosing_opener(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
    stop_semi: bool,
    cap: usize,
) -> Encl {
    let lo = from.saturating_sub(cap);
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let mut ang: i32 = 0;
    let mut i = from + 1;
    while i > lo {
        i -= 1;
        let c = *src.add(i);
        if !GT_SCAN_DELIM[c as usize] || !bm_get(st, i) {
            continue;
        }
        match c {
            b'>' => {
                if bm_get(opch, i) && !(i > 0 && *src.add(i - 1) == b'=') {
                    ang += 1;
                }
            }
            b'<' => {
                if !bm_get(opch, i) {
                    continue;
                }
                let nx = *src.add(i + 1);
                if nx == b'=' || (nx == b'<' && !bm_get(st, i + 1)) {
                    continue;
                }
                if nx == b'<' || (i > 0 && *src.add(i - 1) == b'<' && bm_get(st, i - 1)) {
                    let first = if nx == b'<' { i } else { i - 1 };
                    if !lt_run_opens_type_args(src, st, opch, kind, n, first) {
                        continue;
                    }
                }
                if ang == 0 {
                    return Encl::Open(i, b'<');
                }
                ang -= 1;
            }
            b')' => par += 1,
            b'(' => {
                if par == 0 {
                    return Encl::Open(i, b'(');
                }
                par -= 1;
            }
            b']' => brk += 1,
            b'[' => {
                if brk == 0 {
                    return Encl::Open(i, b'[');
                }
                brk -= 1;
            }
            b'}' => {
                let kk = kind_at(kind, i);
                if kk == TMPL_MIDDLE || kk == TMPL_TAIL {
                    continue;
                }
                brc += 1;
            }
            b'{' => {
                if brc == 0 {
                    return Encl::Open(i, b'{');
                }
                brc -= 1;
            }
            _ => {
                if stop_semi && par == 0 && brk == 0 && brc == 0 && ang == 0 {
                    return Encl::Top;
                }
            }
        }
    }
    if lo == 0 { Encl::Top } else { Encl::Capped }
}

/// True when the `<<` at `lt` is two type-argument/type-parameter openers
/// rather than shift-left.
///
/// Only one TypeScript production puts two `<` next to each other: a
/// type-argument list whose first argument is a function type. The second
/// `<` therefore has to open a type-parameter list belonging to one —
///
/// ```text
/// Name < < TypeParams > ( Params ) => Type >
/// ```
///
/// — so the whole shape is checked, not a prefix of it. That is what
/// separates `Array<<T>(x: T) => T>` from `a << b >> c` (no `(` after the
/// first `>`) and from `a << b > (c)` (no `=>` after the parameters). Every
/// reject path returns false, i.e. today's fused `<<`, so a wrong answer can
/// never split a real shift.
pub(super) unsafe fn lt_run_opens_type_args(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let lim = (lt + GT_SCAN_CAP).min(n);
    // A TypeParameter starts with an identifier (or the `const` modifier),
    // which is what makes `<<=` cost two bytes to reject.
    let head = skip_ws_fwd(src, lt + 2, lim);
    if head >= lim {
        return false;
    }
    let hc = *src.add(head);
    if !(is_id_start(hc) || (hc == b'\\' && *src.add(head + 1) == b'u')) {
        return false;
    }
    let Some(gt) = angle_close_fwd(src, st, opch, kind, lt + 2, lim, 1) else {
        return false;
    };
    let lp = skip_ws_fwd(src, gt + 1, lim);
    if lp >= lim || *src.add(lp) != b'(' {
        return false;
    }
    let Some(rp) = paren_close_fwd(src, st, lp, lim) else {
        return false;
    };
    let ar = skip_ws_fwd(src, rp + 1, lim);
    if ar + 1 >= lim || *src.add(ar) != b'=' || *src.add(ar + 1) != b'>' {
        return false;
    }
    // The outer list must close too, or this was a comparison against a
    // generic arrow function and the parser would have backtracked as well.
    angle_close_fwd(src, st, opch, kind, ar + 2, lim, 1).is_some()
}

pub(super) unsafe fn arrow_after_paren_group(
    src: *const u8,
    st: *const u64,
    lp: usize,
    n: usize,
) -> bool {
    let lim = (lp + GT_SCAN_CAP).min(n);
    let Some(rp) = paren_close_fwd(src, st, lp, lim) else {
        return false;
    };
    let mut i = skip_ws_fwd(src, rp + 1, lim);
    if i + 1 < lim && *src.add(i) == b'=' && *src.add(i + 1) == b'>' {
        return true;
    }
    if i >= lim || *src.add(i) != b':' {
        return false;
    }
    let mut depth: i32 = 0;
    i += 1;
    while i < lim {
        if bm_get(st, i) {
            match *src.add(i) {
                b'(' | b'[' | b'{' | b'<' => depth += 1,
                b')' | b']' | b'}' | b'>' => {
                    if *src.add(i) == b'>' && i > 0 && *src.add(i - 1) == b'=' {
                        if depth == 0 {
                            return true;
                        }
                    } else {
                        depth -= 1;
                        if depth < 0 {
                            return false;
                        }
                    }
                }
                b';' => return false,
                b',' if depth == 0 => return false,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

#[inline]
pub(super) unsafe fn skip_ws_fwd(src: *const u8, mut i: usize, lim: usize) -> usize {
    while i < lim && is_ws(*src.add(i)) {
        i += 1;
    }
    i
}

/// Forward angle match: from `i` at `depth`, the `>` that brings it to 0, or
/// `None` on an unmatched closer, a `;` outside every bracket, or the cap.
/// Same gating as [`gt_run_closes_type_args`] — `opch & st` for angles, `st`
/// for the bracket counters — and the same balance requirement at the close.
unsafe fn angle_close_fwd(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    i: usize,
    lim: usize,
    depth: i32,
) -> Option<usize> {
    angle_close_fwd_capped(src, st, opch, kind, i, lim, depth).0
}

pub(super) unsafe fn angle_close_fwd_capped(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    mut i: usize,
    lim: usize,
    mut depth: i32,
) -> (Option<usize>, bool) {
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    while i < lim {
        let c = *src.add(i);
        if GT_SCAN_DELIM[c as usize] && bm_get(st, i) {
            let op = bm_get(opch, i);
            match c {
                b'<' => {
                    if op && *src.add(i + 1) != b'=' {
                        depth += 1;
                    }
                }
                b'>' => {
                    if op && !(i > 0 && *src.add(i - 1) == b'=') {
                        depth -= 1;
                        if depth == 0 {
                            return ((par == 0 && brk == 0 && brc == 0).then_some(i), false);
                        }
                    }
                }
                b'(' => par += 1,
                b')' => {
                    par -= 1;
                    if par < 0 {
                        return (None, false);
                    }
                }
                b'[' => brk += 1,
                b']' => {
                    brk -= 1;
                    if brk < 0 {
                        return (None, false);
                    }
                }
                b'{' => brc += 1,
                b'}' => {
                    let kk = kind_at(kind, i);
                    if kk != TMPL_MIDDLE && kk != TMPL_TAIL {
                        brc -= 1;
                        if brc < 0 {
                            return (None, false);
                        }
                    }
                }
                _ => {
                    if par == 0 && brk == 0 && brc == 0 {
                        return (None, false); // `;`
                    }
                }
            }
        }
        i += 1;
    }
    (None, true)
}

/// The `)` matching the `(` at `i`, or `None` past `lim`.
unsafe fn paren_close_fwd(
    src: *const u8,
    st: *const u64,
    mut i: usize,
    lim: usize,
) -> Option<usize> {
    let mut d: i32 = 0;
    while i < lim {
        if bm_get(st, i) {
            match *src.add(i) {
                b'(' => d += 1,
                b')' => {
                    d -= 1;
                    if d == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}
