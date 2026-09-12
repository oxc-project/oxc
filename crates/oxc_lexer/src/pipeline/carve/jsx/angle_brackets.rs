use crate::{
    error::diag_code,
    lanes::Lanes,
    tables::{Tables, is_id_start, is_word, is_ws},
};

use super::super::super::{
    bitmap::bm_get,
    disambiguate::{jsx_site_is_expression, ts_type_region_open, type_parameter_list_head},
    find::{find_line_terminator, unicode_ws_len},
    scan::scan_block_comment,
};

const FN_TYPE_SCAN_CAP: usize = 1 << 16;
const FN_TYPE_TMPL_DEPTH: u32 = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AngleVerdict {
    TypeParams,
    Jsx,
    Ambiguous { gt: usize, lp: usize },
}

#[inline]
pub(super) unsafe fn jsx_over_type_params(
    t: &Tables,
    src: *const u8,
    srcs: &[u8],
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    word: *const u64,
    n: usize,
    lt: usize,
    tpos: usize,
    ts: bool,
    lanes: &mut Lanes,
) -> bool {
    if !ts {
        return true;
    }
    match ts_angle_verdict(srcs, n, tpos, word) {
        AngleVerdict::TypeParams => false,
        AngleVerdict::Jsx => true,
        AngleVerdict::Ambiguous { gt, lp } => {
            jsx_ambiguous_site(t, src, st, opch, kind, n, lt, gt, lp, lanes)
        }
    }
}

/// `.tsx` disambiguation: at an operand-position `<IDENT...`, is this a TS
/// type-parameter list rather than a JSX element? In `.tsx` a bare `<T>` is
/// JSX (a generic arrow must be written `<T,>`), so the signals are a
/// trailing `,`, a default `=`, or an `extends` constraint. Bounded forward
/// peek; the source pad makes the look-aheads safe past `n`.
///
/// Identifier runs come off the `word` bitmap, not `is_word` on the raw
/// byte. `is_word` accepts every byte >= 0x80, so non-ASCII whitespace in
/// the head glued into the first identifier and hid the signal; `misc_pre`
/// has already cleared `word` across Unicode whitespace by the time
/// `carve_jsx` runs, so the corrected run is free to read and no byte has to
/// be re-scanned for a leading 0x80.
#[inline]
unsafe fn ts_angle_verdict(src: &[u8], n: usize, t: usize, word: *const u64) -> AngleVerdict {
    let mut p = t;
    // optional `const` type-parameter modifier: `<const T,>`
    if n - p >= 6 && &src[p..p + 5] == b"const" && !is_word(src[p + 5]) {
        let mut qq = p + 5;
        while qq < n && is_ws(src[qq]) {
            qq += 1;
        }
        if qq < n && is_id_start(src[qq]) {
            p = qq; // `const` was a modifier; advance to the real param
        }
    }
    while p < n && bm_get(word, p) {
        p += 1; // first type-parameter identifier
    }
    while p < n {
        let w = head_ws_len(src, p);
        if w == 0 {
            break;
        }
        p += w;
    }
    if p >= n {
        return AngleVerdict::Jsx;
    }
    let c = src[p];
    if c == b',' || c == b'=' {
        return AngleVerdict::TypeParams; // `<T,>`  `<T,U>`  `<T = D>`
    }
    if c == b'>' {
        // `<T>(` and `<T> (` are the same type-parameter list, so the `(`
        // has to be found across trivia. Bounded: a JSX element open walks
        // its own indent run and stops at the first non-space, and running
        // out of budget yields `Jsx`, which is what this arm answered
        // before, so the bound can only leave a rare shape unresolved.
        let mut q = p + 1;
        while q < n && is_ws(src[q]) {
            q += 1;
        }
        let v = if q < n && src[q] == b'(' {
            AngleVerdict::Ambiguous { gt: p, lp: q }
        } else {
            AngleVerdict::Jsx
        };
        return v;
    }
    // `extends` is also a legal JSX attribute name; it signals a generic only
    // as a full word not followed by `=` (attr value) or `>` (boolean attr).
    if n - p >= 7 && &src[p..p + 7] == b"extends" && !bm_get(word, p + 7) {
        let mut qq = p + 7;
        while qq < n && is_ws(src[qq]) {
            qq += 1;
        }
        let d = if qq < n { src[qq] } else { 0 };
        if d == b'/' && !matches!(if qq + 1 < n { src[qq + 1] } else { 0 }, b'*' | b'/') {
            return AngleVerdict::Jsx;
        }
        let v = if d == b'=' || d == b'>' { AngleVerdict::Jsx } else { AngleVerdict::TypeParams };
        return v;
    }
    AngleVerdict::Jsx
}

/// Byte length of the whitespace at `p` — ASCII, or the multi-byte
/// ECMAScript whitespace `misc_pre` marked as a token boundary — else 0.
#[inline(always)]
unsafe fn head_ws_len(src: &[u8], p: usize) -> usize {
    let c = src[p];
    if is_ws(c) {
        return 1;
    }
    if c >= 0x80 {
        return unicode_ws_len(src.as_ptr(), p);
    }
    0
}

#[inline(never)]
unsafe fn jsx_ambiguous_site(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    gt: usize,
    lp: usize,
    lanes: &mut Lanes,
) -> bool {
    if ts_type_region_open(t, src, st, opch, kind, n, lt) {
        return false;
    }
    if type_parameter_list_head(t, src, st, opch, kind, n, lt) {
        return false;
    }
    if generic_fn_type_after(src, n, lp) {
        if jsx_site_is_expression(t, src, st, opch, kind, n, lt) {
            lanes.push_diag(lt as u32, (gt + 1 - lt) as u32, diag_code::UNTERMINATED_JSX_ELEMENT);
        }
        return false;
    }
    true
}

unsafe fn generic_fn_type_after(src: *const u8, n: usize, lp: usize) -> bool {
    let lim = (lp + FN_TYPE_SCAN_CAP).min(n);
    let mut depth: i32 = 0;
    let mut i = lp;
    loop {
        if i >= lim {
            return false;
        }
        let c = *src.add(i);
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            b'"' | b'\'' => {
                let Some(e) = skip_quoted_type(src, lim, i, c) else {
                    return false;
                };
                i = e;
                continue;
            }
            b'`' => {
                let Some(e) = skip_template_type(src, lim, i, 0) else {
                    return false;
                };
                i = e;
                continue;
            }
            b'/' => match *src.add(i + 1) {
                b'*' => {
                    let e = scan_block_comment(src, n, i + 2).0;
                    if e >= n {
                        return false;
                    }
                    i = e;
                }
                b'/' => {
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                _ => return false,
            },
            _ => {}
        }
        i += 1;
    }
    loop {
        if i >= lim {
            return false;
        }
        let c = *src.add(i);
        if is_ws(c) {
            i += 1;
            continue;
        }
        if c == b'/' {
            match *src.add(i + 1) {
                b'*' => {
                    let e = scan_block_comment(src, n, i + 2).0;
                    if e >= n {
                        return false;
                    }
                    i = e + 1;
                    continue;
                }
                b'/' => {
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                _ => return false,
            }
        }
        if c >= 0x80 {
            let w = unicode_ws_len(src, i);
            if w != 0 {
                i += w;
                continue;
            }
        }
        return c == b'=' && *src.add(i + 1) == b'>';
    }
}

unsafe fn skip_template_type(src: *const u8, lim: usize, i: usize, depth: u32) -> Option<usize> {
    if depth > FN_TYPE_TMPL_DEPTH {
        return None;
    }
    let mut j = i + 1;
    let mut subs = [0u32; 8];
    let mut nsub = 0usize;
    loop {
        if j >= lim {
            return None;
        }
        let d = *src.add(j);
        if nsub == 0 {
            match d {
                b'\\' => j += 2,
                b'`' => return Some(j + 1),
                b'$' if *src.add(j + 1) == b'{' => {
                    if nsub == subs.len() {
                        return None;
                    }
                    subs[nsub] = 0;
                    nsub += 1;
                    j += 2;
                }
                _ => j += 1,
            }
        } else {
            match d {
                b'{' => {
                    subs[nsub - 1] += 1;
                    j += 1;
                }
                b'}' => {
                    if subs[nsub - 1] == 0 {
                        nsub -= 1;
                    } else {
                        subs[nsub - 1] -= 1;
                    }
                    j += 1;
                }
                b'"' | b'\'' => j = skip_quoted_type(src, lim, j, d)?,
                b'`' => j = skip_template_type(src, lim, j, depth + 1)?,
                _ => j += 1,
            }
        }
    }
}

unsafe fn skip_quoted_type(src: *const u8, lim: usize, i: usize, q: u8) -> Option<usize> {
    let mut j = i + 1;
    loop {
        if j >= lim {
            return None;
        }
        let d = *src.add(j);
        if d == b'\\' {
            j += 2;
            continue;
        }
        if d == q {
            return Some(j + 1);
        }
        if d == b'\n' || d == b'\r' {
            return None;
        }
        j += 1;
    }
}
