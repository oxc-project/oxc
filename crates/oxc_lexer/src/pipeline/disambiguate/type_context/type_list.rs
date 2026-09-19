//! Would TypeScript accept `<...>` as type arguments in an expression?
//!
//! In an expression, TypeScript reads `f<T>(x)` as a call with type arguments,
//! but `a < b > c` as comparisons.
//! It decides by trying to parse type arguments, and keeps the result only if both checks here pass:
//!
//! - [`type_list_legal`]: Do the tokens between `<` and `>` form a list of types?
//!   `a<b + 1, c>` doesn't, because `+` can't appear in a type.
//! - [`gt_follower`]: Can the token after the `>` follow type arguments?
//!   A `(` can, as in `f<T>(x)`. An identifier can't, as in `a < b > c`.
//!   When the token alone doesn't settle it, the answer is [`Follow::Ctx`], and the caller decides.

use crate::token::tk;

use crate::pipeline::{
    bitmap::{bm_get, bm_next1},
    bytes::{is_digit, is_id_start},
    find::{find_line_terminator, unicode_ws_len},
    scan::scan_block_comment,
    tables::{OP_KIND_BASE, Tables},
};

use crate::pipeline::disambiguate::common::{
    kind_at, lt_in_range, type_prefix_kind, word_is_any, word_len,
};

use super::bytes::skip_ws_fwd;

const FOLLOW_SPLIT_WORDS: &[&[u8]] =
    &[b"in", b"instanceof", b"as", b"satisfies", b"extends", b"implements"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Follow {
    Split,
    Fuse,
    Ctx,
}

pub(super) unsafe fn gt_follower(src: *const u8, n: usize, mut i: usize) -> Follow {
    let mut broke = false;
    loop {
        if i >= n {
            return Follow::Split;
        }
        let c = *src.add(i);
        match c {
            b' ' | b'\t' | 0x0b | 0x0c => {
                i += 1;
                continue;
            }
            b'\n' | b'\r' => {
                broke = true;
                i += 1;
                continue;
            }
            b'/' => {
                let d = *src.add(i + 1);
                if d == b'/' {
                    broke = true;
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                if d != b'*' {
                    return Follow::Split;
                }
                let e = scan_block_comment(src, n, i + 2).0;
                if e >= n {
                    return Follow::Split;
                }
                if lt_in_range(src, i + 2, e) {
                    broke = true;
                }
                i = e + 1;
                continue;
            }
            _ => {}
        }
        if c >= 0x80 {
            if c == 0xe2
                && *src.add(i + 1) == 0x80
                && (*src.add(i + 2) == 0xa8 || *src.add(i + 2) == 0xa9)
            {
                broke = true;
                i += 3;
                continue;
            }
            let wl = unicode_ws_len(src, i);
            if wl != 0 {
                i += wl;
                continue;
            }
            return if broke { Follow::Split } else { Follow::Fuse };
        }
        let nx = *src.add(i + 1);
        if broke {
            return match c {
                b'<' if nx != b'<' && nx != b'=' => Follow::Ctx,
                b'+' | b'-' if nx != b'=' && nx != c => Follow::Ctx,
                b'>' => Follow::Ctx,
                _ => Follow::Split,
            };
        }
        return match c {
            b'(' | b'`' | b'=' | b')' | b']' | b'}' | b',' | b';' | b':' | b'?' | b'|' | b'&'
            | b'*' | b'%' | b'^' => Follow::Split,
            b'!' => {
                if nx == b'=' {
                    Follow::Split
                } else {
                    Follow::Fuse
                }
            }
            b'.' => {
                if is_digit(nx) {
                    Follow::Fuse
                } else {
                    Follow::Split
                }
            }
            b'{' | b'[' | b'>' => Follow::Ctx,
            b'+' | b'-' => {
                if nx == b'=' {
                    Follow::Split
                } else {
                    Follow::Fuse
                }
            }
            b'<' => {
                if nx == b'<' || nx == b'=' {
                    Follow::Split
                } else {
                    Follow::Fuse
                }
            }
            b'~' | b'@' | b'#' | b'"' | b'\'' => Follow::Fuse,
            _ => {
                if is_digit(c) {
                    Follow::Fuse
                } else if is_id_start(c) {
                    if word_is_any(src, i, FOLLOW_SPLIT_WORDS) {
                        Follow::Split
                    } else {
                        Follow::Fuse
                    }
                } else {
                    Follow::Split
                }
            }
        };
    }
}

pub(super) unsafe fn type_list_legal(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lo: usize,
    hi: usize,
) -> bool {
    let mut start = true;
    let mut brc: i32 = 0;
    let mut brk: i32 = 0;
    let mut angle_bits: u64 = 0;
    let mut angle_depth: u32 = 0;
    let mut paren_ok = false;
    let mut cond_ok = false;
    let mut par: i32 = 0;
    let mut this_head = false;
    let mut skip = usize::MAX;
    let mut w = bm_next1(st, lo, hi);
    while w < hi {
        let k = kind_at(kind, w);
        if w == skip || k == tk!(Whitespace) || k == tk!(LineComment) || k == tk!(BlockComment) {
            w = bm_next1(st, w + 1, hi);
            continue;
        }
        let c = *src.add(w);
        let mut ok_paren = false;
        let was_this = this_head;
        this_head = false;
        if k == tk!(Ident) || k == tk!(IdentEscaped) {
            let kk = t.kwts.lookup(src.add(w), word_len(src, w)) as u8;
            this_head = kk == tk!(KwThis);
            if !start && brc == 0 && !matches!(kk, tk!(KwExtends) | tk!(KwIs) | tk!(KwIn)) {
                return false;
            }
            if type_illegal_kind(kk) {
                return false;
            }
            if kk == tk!(KwExtends) {
                cond_ok = true;
            }
            start = type_prefix_kind(kk);
        } else if k == tk!(Number)
            || k == tk!(BigInt)
            || k == tk!(String)
            || k == tk!(TemplateNoSub)
            || k == tk!(TemplateTail)
        {
            if !start && brc == 0 && k != tk!(TemplateTail) {
                return false;
            }
            start = false;
        } else if k == tk!(TemplateHead) || k == tk!(TemplateMiddle) {
            if k == tk!(TemplateHead) && !start && brc == 0 {
                return false;
            }
            start = true;
        } else if k >= OP_KIND_BASE {
            match c {
                b'(' => {
                    if !start && brc == 0 && !paren_ok {
                        return false;
                    }
                    par += 1;
                    start = true;
                }
                b')' => {
                    par -= 1;
                    start = false;
                }
                b']' => {
                    brk -= 1;
                    start = false;
                }
                b'[' => {
                    brk += 1;
                    start = true;
                }
                b'{' => {
                    if !start && brc == 0 {
                        return false;
                    }
                    brc += 1;
                    start = true;
                }
                b'}' => {
                    brc -= 1;
                    start = false;
                }
                b'<' => {
                    let nx = *src.add(w + 1);
                    if was_this || nx == b'=' || (nx == b'<' && !bm_get(st, w + 1)) {
                        return false;
                    }
                    angle_bits = (angle_bits << 1) | u64::from(start);
                    angle_depth += 1;
                    start = true;
                }
                b'>' => {
                    let nx = *src.add(w + 1);
                    if (nx == b'=' || nx == b'>') && !bm_get(st, w + 1) {
                        return false;
                    }
                    if angle_depth > 0 {
                        ok_paren = angle_bits & 1 != 0;
                        angle_bits >>= 1;
                        angle_depth -= 1;
                    }
                    start = false;
                }
                b'=' => {
                    if *src.add(w + 1) != b'>' {
                        return false;
                    }
                    if bm_get(st, w + 1) {
                        skip = w + 1;
                    }
                    start = true;
                }
                b':' | b'.' => start = true,
                b',' => {
                    if angle_depth == 0 && brc == 0 && brk == 0 && par == 0 {
                        cond_ok = false;
                    }
                    start = true;
                }
                b'|' | b'&' => {
                    if *src.add(w + 1) == c {
                        return false;
                    }
                    start = true;
                }
                b'?' => {
                    let nx = *src.add(w + 1);
                    if nx == b'.' || nx == b'?' {
                        return false;
                    }
                    if brc == 0 && brk == 0 {
                        let optional = par > 0
                            && matches!(*src.add(skip_ws_fwd(src, w + 1, hi)), b':' | b',' | b')');
                        if !optional {
                            if !cond_ok {
                                return false;
                            }
                            cond_ok = false;
                        }
                    }
                    start = true;
                }
                b';' => {
                    if brc == 0 {
                        return false;
                    }
                    start = true;
                }
                b'-' => {
                    if !start && brc == 0 {
                        return false;
                    }
                    start = true;
                }
                b'+' => {
                    if brc == 0 {
                        return false;
                    }
                    start = true;
                }
                _ => return false,
            }
        } else {
            return false;
        }
        paren_ok = ok_paren;
        w = bm_next1(st, w + 1, hi);
    }
    true
}

#[inline(always)]
fn type_illegal_kind(k: u8) -> bool {
    matches!(
        k,
        tk!(KwAwait)
            | tk!(KwYield)
            | tk!(KwDelete)
            | tk!(KwFunction)
            | tk!(KwClass)
            | tk!(KwInstanceof)
            | tk!(KwSuper)
            | tk!(KwSwitch)
            | tk!(KwCase)
            | tk!(KwReturn)
            | tk!(KwThrow)
            | tk!(KwVar)
            | tk!(KwLet)
            | tk!(KwConst)
            | tk!(KwIf)
            | tk!(KwElse)
            | tk!(KwFor)
            | tk!(KwWhile)
            | tk!(KwDo)
            | tk!(KwBreak)
            | tk!(KwContinue)
            | tk!(KwWith)
            | tk!(KwTry)
            | tk!(KwCatch)
            | tk!(KwFinally)
            | tk!(KwDebugger)
            | tk!(KwDefault)
            | tk!(KwExport)
            | tk!(KwEnum)
    )
}
