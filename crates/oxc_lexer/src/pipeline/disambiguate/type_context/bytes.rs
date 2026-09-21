//! Bracket matching by scanning raw source bytes.
//!
//! `common` matches brackets backwards over a bitmap of bracket tokens ([`match_delim_back`]).
//! The scans here go forwards, reading raw source bytes.
//! The token-start bitmap hides bytes inside literals and comments,
//! and the operator bitmap hides the angle brackets of JSX tags.
//!
//! Forward:
//! - [`angle_close_fwd`] and [`paren_close_fwd`] go from an opener to its closer.
//! - [`lt_run_opens_type_args`] checks for the shape of an arrow function after a `<<`,
//!   as in `Array<<T>(x: T) => T>`.
//!
//! Every scan has a length limit, so pathological input can't make it slow.
//!
//! [`match_delim_back`]: crate::pipeline::disambiguate::common::Tokens::match_delim_back

use crate::{
    pipeline::bytes::{is_id_start, is_ws},
    token::{OP_KIND_BASE, tk},
};

use crate::pipeline::disambiguate::common::{bits, kind_at, text};

/// Byte cap on the forward scans. Hitting it answers `None` (fuse), so it can only widen a fused
/// run, never split a shift.
pub(super) const GT_SCAN_CAP: usize = 1 << 16;

/// Bytes the forward angle match reacts to. Everything else is skipped without touching a bitmap.
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
pub(in crate::pipeline::disambiguate) fn lt_run_opens_type_args(
    src: &[u8],
    st: &[u64],
    opch: &[u64],
    kind: &[u8],
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
    let hc = src[head];
    if !(is_id_start(hc) || (hc == b'\\' && src[head + 1] == b'u')) {
        return false;
    }
    let Some(gt) = angle_close_fwd(src, st, opch, kind, lt + 2, lim, 1) else {
        return false;
    };
    let lp = skip_ws_fwd(src, gt + 1, lim);
    if lp >= lim || src[lp] != b'(' {
        return false;
    }
    let Some(rp) = paren_close_fwd(src, st, lp, lim) else {
        return false;
    };
    let ar = skip_ws_fwd(src, rp + 1, lim);
    if ar + 1 >= lim || src[ar] != b'=' || src[ar + 1] != b'>' {
        return false;
    }
    // The outer list must close too, or this was a comparison against a
    // generic arrow function and the parser would have backtracked as well.
    angle_close_fwd(src, st, opch, kind, ar + 2, lim, 1).is_some()
}

#[inline]
pub(super) fn skip_ws_fwd(src: &[u8], mut i: usize, lim: usize) -> usize {
    while i < lim && is_ws(src[i]) {
        i += 1;
    }
    i
}

/// Forward angle match: from `i` at `depth`, the `>` that brings it to 0, or
/// `None` on an unmatched closer, a `;` outside every bracket, or the cap.
/// Angles count only where `opch & st` is set, the bracket counters where `st` is, and the close
/// must leave every bracket balanced.
fn angle_close_fwd(
    src: &[u8],
    st: &[u64],
    opch: &[u64],
    kind: &[u8],
    i: usize,
    lim: usize,
    depth: i32,
) -> Option<usize> {
    angle_close_fwd_capped(src, st, opch, kind, i, lim, depth).0
}

pub(in crate::pipeline::disambiguate) fn angle_close_fwd_capped(
    src: &[u8],
    st: &[u64],
    opch: &[u64],
    kind: &[u8],
    mut i: usize,
    lim: usize,
    mut depth: i32,
) -> (Option<usize>, bool) {
    let mut parens: i32 = 0;
    let mut brackets: i32 = 0;
    let mut braces: i32 = 0;
    while i < lim {
        if bits::get(st, i) {
            let j = skip_raw_literal(src, kind, lim, i);
            if j != i {
                i = j;
                continue;
            }
            // Asked from the JSX carve, a template head may be carved while its tail is still
            // raw text (its substitution `}` a punctuator): cross the literal whole.
            if kind_at(kind, i) == tk!(TemplateHead) {
                let (close, end) = raw_template_end(src, lim, i);
                if close < lim && kind_at(kind, close) >= OP_KIND_BASE {
                    if end > lim {
                        return (None, true);
                    }
                    i = end;
                    continue;
                }
            }
        }
        let c = src[i];
        if GT_SCAN_DELIM[c as usize] && bits::get(st, i) {
            let op = bits::get(opch, i);
            match c {
                b'<' => {
                    if op && src[i + 1] != b'=' {
                        depth += 1;
                    }
                }
                b'>' => {
                    if op && !(i > 0 && src[i - 1] == b'=') {
                        depth -= 1;
                        if depth == 0 {
                            return (
                                (parens == 0 && brackets == 0 && braces == 0).then_some(i),
                                false,
                            );
                        }
                    }
                }
                b'(' => parens += 1,
                b')' => {
                    parens -= 1;
                    if parens < 0 {
                        return (None, false);
                    }
                }
                b'[' => brackets += 1,
                b']' => {
                    brackets -= 1;
                    if brackets < 0 {
                        return (None, false);
                    }
                }
                b'{' => braces += 1,
                b'}' => {
                    let kk = kind_at(kind, i);
                    if kk != tk!(TemplateMiddle) && kk != tk!(TemplateTail) {
                        braces -= 1;
                        if braces < 0 {
                            return (None, false);
                        }
                    }
                }
                _ => {
                    if parens == 0 && brackets == 0 && braces == 0 {
                        return (None, false); // `;`
                    }
                }
            }
        }
        i += 1;
    }
    (None, true)
}

/// If a comment, string or template carve has not reached yet starts at the token start `i`, the
/// position just past it; else `i`. Forward scans from before the carve cursor cross raw text;
/// carved literals have cleared interiors and are never re-entered.
#[inline]
pub(super) fn skip_raw_literal(src: &[u8], kind: &[u8], n: usize, i: usize) -> usize {
    if kind[i] < OP_KIND_BASE {
        return i;
    }
    let c = src[i];
    match c {
        b'/' => match src[i + 1] {
            b'/' => text::line_terminator_after(src, n, i + 2),
            b'*' => {
                let e = text::block_comment_end(src, n, i + 2);
                if e < n { e + 1 } else { n }
            }
            _ => i,
        },
        b'"' | b'\'' => {
            let mut j = i + 1;
            while j < n {
                let d = src[j];
                if d == b'\\' {
                    j += 2;
                    continue;
                }
                if d == c || d == b'\n' || d == b'\r' {
                    return j + 1;
                }
                j += 1;
            }
            n
        }
        b'`' => {
            let mut j = i + 1;
            while j < n {
                let d = src[j];
                if d == b'\\' {
                    j += 2;
                    continue;
                }
                if d == b'`' {
                    return j + 1;
                }
                j += 1;
            }
            n
        }
        _ => i,
    }
}

/// For a template head whose backtick is at `i`: the `}` closing its first substitution and the
/// end (exclusive) of the whole literal, over raw bytes. `lim` when the `}` is not found before
/// `lim`, `lim + 1` when the literal does not end before it. Cold: a template inside a
/// speculated type-argument list.
pub(super) fn raw_template_end(src: &[u8], lim: usize, i: usize) -> (usize, usize) {
    let mut j = i + 1;
    loop {
        if j >= lim {
            return (lim, lim + 1);
        }
        match src[j] {
            b'\\' => j += 2,
            b'`' => return (lim, j + 1),
            b'$' if src[j + 1] == b'{' => {
                j += 2;
                break;
            }
            _ => j += 1,
        }
    }
    let close = raw_substitution_close(src, lim, j, 0);
    if close >= lim {
        return (lim, lim + 1);
    }
    (close, raw_template_rest(src, lim, close + 1, 0))
}

/// From just after `${`: the index of the matching `}`, or `lim`.
fn raw_substitution_close(src: &[u8], lim: usize, mut j: usize, depth: u32) -> usize {
    let mut braces = 0u32;
    while j < lim {
        let c = src[j];
        match c {
            b'{' => braces += 1,
            b'}' => {
                if braces == 0 {
                    return j;
                }
                braces -= 1;
            }
            b'`' => {
                if depth > 8 {
                    return lim;
                }
                j = raw_template_rest(src, lim, j + 1, depth + 1);
                continue;
            }
            b'"' | b'\'' => {
                j += 1;
                while j < lim {
                    let d = src[j];
                    if d == b'\\' {
                        j += 2;
                        continue;
                    }
                    j += 1;
                    if d == c || d == b'\n' || d == b'\r' {
                        break;
                    }
                }
                continue;
            }
            b'/' => match src[j + 1] {
                b'/' => {
                    j = text::line_terminator_after(src, lim, j + 2);
                    continue;
                }
                b'*' => {
                    let e = text::block_comment_end(src, lim, j + 2);
                    if e >= lim {
                        return lim;
                    }
                    j = e + 1;
                    continue;
                }
                _ => {}
            },
            _ => {}
        }
        j += 1;
    }
    lim
}

/// Template text from `j` (after a substitution's `}` or the opening backtick): the index after
/// the closing backtick, or `lim + 1`.
fn raw_template_rest(src: &[u8], lim: usize, mut j: usize, depth: u32) -> usize {
    while j < lim {
        match src[j] {
            b'\\' => j += 2,
            b'`' => return j + 1,
            b'$' if src[j + 1] == b'{' => {
                let close = raw_substitution_close(src, lim, j + 2, depth);
                if close >= lim {
                    return lim + 1;
                }
                j = close + 1;
            }
            _ => j += 1,
        }
    }
    lim + 1
}

/// The `)` matching the `(` at `i`, or `None` past `lim`.
fn paren_close_fwd(src: &[u8], st: &[u64], mut i: usize, lim: usize) -> Option<usize> {
    let mut d: i32 = 0;
    while i < lim {
        if bits::get(st, i) {
            match src[i] {
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
