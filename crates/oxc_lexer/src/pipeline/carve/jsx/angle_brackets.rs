use crate::{error::DiagCode, lanes::Lanes};

use crate::pipeline::{
    bitmap::bm_get,
    bytes::{is_id_start, is_word, line_break_in},
    disambiguate::{
        Tokens, Walks, arrow_after_params, jsx_site_is_expression, ts_type_region_open,
        type_parameter_list_head,
    },
    tables::Tables,
    token_view,
};

use super::names::jsx_skip_trivia_fast;

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
                &lanes.disambiguate.closers,
            );
            let (jsx, unterminated) =
                jsx_ambiguous_site(&tokens, &mut lanes.disambiguate.walks, lt, lp);
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
        // A modifier must be on the same line as its parameter, so `const` followed by a line break
        // can only be a JSX tag name, e.g. `<const\nextends U>` is JSX.
        if line_break_in(src, p + 5, qq) {
            return AngleVerdict::Jsx;
        }
        if qq < n && is_id_start(src[qq]) {
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
        let q = jsx_skip_trivia_fast(src.as_ptr(), n, p + 1);
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

/// Is the ambiguous `<T>(` at `lt` JSX (true) or a type-parameter list (false)? The second
/// answer says whether it is an unterminated JSX element to report: a generic arrow shape at a
/// site where an operand may start.
#[inline(never)]
fn jsx_ambiguous_site(tokens: &Tokens, walks: &mut Walks, lt: usize, lp: usize) -> (bool, bool) {
    if ts_type_region_open(tokens, walks, lt) || type_parameter_list_head(tokens, walks, lt) {
        return (false, false);
    }
    if arrow_after_params(tokens, lp) {
        return (false, jsx_site_is_expression(tokens, walks, lt));
    }
    (true, false)
}
