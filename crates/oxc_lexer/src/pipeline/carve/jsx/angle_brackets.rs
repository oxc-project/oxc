use crate::{error::DiagCode, lanes::Lanes};

use crate::pipeline::{
    bitmap::bm_get,
    bytes::{is_id_start, is_word, is_ws},
    disambiguate::{
        Tokens, Walks, jsx_site_is_expression, ts_type_region_open, type_parameter_list_head,
    },
    find::{find_line_terminator, unicode_ws_len},
    scan::scan_block_comment,
    tables::Tables,
    token_view,
};

use super::names::jsx_skip_trivia_fast;

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
            let tokens = token_view(
                t,
                src,
                st,
                opch,
                word,
                kind,
                n,
                ts,
                0,
                lanes.module,
                &lanes.disambiguate.brackets,
            );
            let (jsx, unterminated) =
                jsx_ambiguous_site(&tokens, &mut lanes.disambiguate.walks, src, n, lt, lp);
            if unterminated {
                lanes.push_diag(lt as u32, (gt + 1 - lt) as u32, DiagCode::UnterminatedJsxElement);
            }
            jsx
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
    // optional `const` type-parameter modifier: `<const T,>`. Like any modifier it must stay
    // on the line of its parameter; a comment between them is fine.
    if n - p >= 6 && &src[p..p + 5] == b"const" && !is_word(src[p + 5]) {
        let qq = jsx_skip_trivia_fast(src.as_ptr(), n, p + 5);
        if qq < n && is_id_start(src[qq]) && !line_break_in(src, p + 5, qq) {
            p = qq; // `const` was a modifier; advance to the real param
        }
    }
    while p < n && bm_get(word, p) {
        p += 1; // first type-parameter identifier
    }
    // The signal may sit behind whitespace (Unicode too) or a comment: `<T /*c*/ extends U>`.
    p = jsx_skip_trivia_fast(src.as_ptr(), n, p);
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
        let qq = jsx_skip_trivia_fast(src.as_ptr(), n, p + 7);
        let d = if qq < n { src[qq] } else { 0 };
        if d == b'/' && !matches!(if qq + 1 < n { src[qq + 1] } else { 0 }, b'*' | b'/') {
            return AngleVerdict::Jsx;
        }
        let v = if d == b'=' || d == b'>' { AngleVerdict::Jsx } else { AngleVerdict::TypeParams };
        return v;
    }
    AngleVerdict::Jsx
}

/// Does `src[a..b]` hold a LineTerminator (LF, CR, or the 3-byte LS/PS)?
#[inline]
fn line_break_in(src: &[u8], a: usize, b: usize) -> bool {
    let mut i = a;
    while i < b {
        match src[i] {
            b'\n' | b'\r' => return true,
            0xE2 if src[i + 1] == 0x80 && matches!(src[i + 2], 0xA8 | 0xA9) => return true,
            _ => i += 1,
        }
    }
    false
}

/// Is the ambiguous `<T>(` at `lt` JSX (true) or a type-parameter list (false)? The second
/// answer says whether it is an unterminated JSX element to report: a generic arrow shape at a
/// site where an operand may start.
#[inline(never)]
unsafe fn jsx_ambiguous_site(
    tokens: &Tokens,
    walks: &mut Walks,
    src: *const u8,
    n: usize,
    lt: usize,
    lp: usize,
) -> (bool, bool) {
    if ts_type_region_open(tokens, walks, lt) || type_parameter_list_head(tokens, walks, lt) {
        return (false, false);
    }
    if generic_fn_type_after(src, n, lp) {
        return (false, jsx_site_is_expression(tokens, walks, lt));
    }
    (true, false)
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
