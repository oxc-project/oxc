//! Bracket matching by scanning raw source bytes.
//!
//! `common` matches brackets backwards over a bitmap of the source's bracket bytes ([`match_delim_back`]).
//! The scans here go forwards, reading raw source bytes.
//! The token-start bitmap hides bytes inside literals and comments,
//! and the operator bitmap hides the angle brackets of JSX tags.
//!
//! Forward:
//! - [`list_closer`] and [`group_closer`] go from an opener to its closer.
//! - [`lt_run_split`] checks for the shape of an arrow function after a `<<`,
//!   as in `Array<<T>(x: T) => T>`.
//!
//! A scan reads at most [`FORWARD_SCAN_CAP`] bytes. Past that the answer is still exact: one pass
//! from the opener ([`resolve_lists`], [`resolve_groups`]) resolves it and records the closer of
//! every opener of the same kind it crosses in [`Closers`], so the openers inside it, and the
//! `<`s of a file whose lists never close, cost a lookup each.
//!
//! [`match_delim_back`]: crate::pipeline::disambiguate::common::Tokens::match_delim_back

use crate::{
    pipeline::bytes::{
        block_comment_end, is_id_start, is_ws, line_terminator_after, unicode_ws_len_at,
    },
    token::{OP_KIND_BASE, matches_tk, tk},
};

use crate::pipeline::disambiguate::{
    FORWARD_SCAN_CAP,
    common::{Closers, Tokens, bits, kind_at},
};

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
/// `<` therefore has to open a type-parameter list belonging to one:
///
/// ```text
/// Name < < TypeParams > ( Params ) => Type >
/// ```
///
/// So the whole shape is checked, not a prefix of it. That is what
/// separates `Array<<T>(x: T) => T>` from `a << b >> c` (no `(` after the
/// first `>`) and from `a << b > (c)` (no `=>` after the parameters). Every
/// reject path returns false, i.e. today's fused `<<`, so a wrong answer can
/// never split a real shift.
#[inline(never)]
pub(crate) fn lt_run_split(tokens: &Tokens, lt: usize) -> bool {
    let Tokens { src, st, n, .. } = *tokens;
    // A TypeParameter starts with an identifier (or the `const` modifier),
    // which is what makes `<<=` cost two bytes to reject.
    let head = skip_ws_fwd(src, lt + 2, n);
    if head >= n {
        return false;
    }
    let hc = src[head];
    if !(is_id_start(hc) || (hc == b'\\' && src[head + 1] == b'u')) {
        return false;
    }
    // Already fused by coalesce: a shift.
    if !bits::get(st, lt + 1) {
        return false;
    }
    let Some(gt) = list_closer(tokens, lt + 1) else {
        return false;
    };
    let lp = skip_ws_fwd(src, gt + 1, n);
    if lp >= n || src[lp] != b'(' {
        return false;
    }
    let Some(rp) = group_closer(tokens, lp) else {
        return false;
    };
    let ar = skip_ws_fwd(src, rp + 1, n);
    if ar + 1 >= n || src[ar] != b'=' || src[ar + 1] != b'>' {
        return false;
    }
    // The outer list must close too, or this was a comparison against a
    // generic arrow function and the parser would have backtracked as well.
    list_closer(tokens, lt).is_some()
}

pub(crate) fn arrow_after_params(tokens: &Tokens, lp: usize) -> bool {
    let Tokens { src, n, .. } = *tokens;
    let Some(rp) = group_closer(tokens, lp) else {
        return false;
    };
    let ar = skip_trivia_fwd(src, n, rp + 1);
    ar < n && src[ar] == b'=' && src[ar + 1] == b'>'
}

#[inline]
pub(super) fn skip_ws_fwd(src: &[u8], mut i: usize, lim: usize) -> usize {
    while i < lim && is_ws(src[i]) {
        i += 1;
    }
    i
}

fn skip_trivia_fwd(src: &[u8], n: usize, mut i: usize) -> usize {
    while i < n {
        let c = src[i];
        if is_ws(c) {
            i += 1;
        } else if c == b'/' && src[i + 1] == b'/' {
            i = line_terminator_after(src, n, i + 2);
        } else if c == b'/' && src[i + 1] == b'*' {
            let e = block_comment_end(src, n, i + 2);
            if e >= n {
                return n;
            }
            i = e + 1;
        } else if c >= 0x80 && unicode_ws_len_at(src, i) != 0 {
            i += unicode_ws_len_at(src, i);
        } else {
            break;
        }
    }
    i
}

fn list_closer(tokens: &Tokens, lt: usize) -> Option<usize> {
    match tokens.closers.get(lt) {
        Some(r) => r.closer(),
        None => scan_list_closer(tokens, lt),
    }
}

pub(super) fn scan_list_closer(tokens: &Tokens, lt: usize) -> Option<usize> {
    let lim = (lt + FORWARD_SCAN_CAP).min(tokens.n);
    match angle_close_fwd_capped(tokens, lt + 1, lim, 1) {
        (Some(gt), _) => Some(gt),
        (None, true) if lim < tokens.n => resolve_lists(tokens, lt),
        (None, _) => None,
    }
}

fn group_closer(tokens: &Tokens, lp: usize) -> Option<usize> {
    if let Some(r) = tokens.closers.get(lp) {
        return r.closer();
    }
    let lim = (lp + FORWARD_SCAN_CAP).min(tokens.n);
    match paren_close_fwd_capped(tokens, lp + 1, lim) {
        (Some(rp), _) => Some(rp),
        (None, true) if lim < tokens.n => resolve_groups(tokens, lp),
        (None, _) => None,
    }
}

#[inline]
fn past_raw(src: &[u8], kind: &[u8], lim: usize, i: usize) -> Option<usize> {
    let j = skip_raw_literal(src, kind, lim, i);
    if j != i {
        return (j < lim).then_some(j);
    }
    // Asked from the JSX carve, a template head may be carved while its tail is still
    // raw text (its substitution `}` a punctuator): cross the literal whole.
    if kind_at(kind, i) == tk!(TemplateHead) {
        let (close, end) = raw_template_end(src, lim, i);
        if close < lim && kind_at(kind, close) >= OP_KIND_BASE {
            return (end <= lim).then_some(end);
        }
    }
    Some(i)
}

/// Forward angle match: from `i` at `depth`, the `>` that brings it to 0, or
/// `None` on an unmatched closer, a `;` outside every bracket, or the cap.
/// The flag says the scan ran out at `lim` with the match still open, so the caller falls back to
/// [`resolve_lists`]; on the other `None`s the region cannot be a list at all.
/// Angles count only where `opch & st` is set, the bracket counters where `st` is, and the close
/// must leave every bracket balanced.
fn angle_close_fwd_capped(
    tokens: &Tokens,
    mut i: usize,
    lim: usize,
    mut depth: i32,
) -> (Option<usize>, bool) {
    let Tokens { src, st, opch, kind, .. } = *tokens;
    let mut parens: i32 = 0;
    let mut brackets: i32 = 0;
    let mut braces: i32 = 0;
    while i < lim {
        if bits::get(st, i) {
            match past_raw(src, kind, lim, i) {
                Some(j) if j != i => {
                    i = j;
                    continue;
                }
                Some(_) => {}
                None => return (None, true),
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
                    // A substitution-closing `}` is the start of the next
                    // template segment, and its `${` was swallowed by the
                    // preceding one - counting it would leave every
                    // `Array<Map<A, `p${s}q`>>` looking brace-unbalanced.
                    let kk = kind_at(kind, i);
                    if !matches_tk!(kk, TemplateMiddle | TemplateTail) {
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

/// The `)` matching the `(` at `i`, or `None`. The flag says the scan ran out at `lim` with the
/// match still open, so the caller falls back to [`resolve_groups`].
fn paren_close_fwd_capped(tokens: &Tokens, mut i: usize, lim: usize) -> (Option<usize>, bool) {
    let Tokens { src, st, kind, .. } = *tokens;
    let mut depth: i32 = 1;
    while i < lim {
        if bits::get(st, i) {
            match past_raw(src, kind, lim, i) {
                Some(j) if j != i => {
                    i = j;
                    continue;
                }
                Some(_) => {}
                None => return (None, true),
            }
            match src[i] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return (Some(i), false);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    (None, true)
}

struct Open {
    pos: u32,
    parens: i32,
    brackets: i32,
    braces: i32,
    /// A run of ruled-out <s, kept for the depth of the lists around them.
    dead: u32,
}

/// The rules of [`angle_close_fwd_capped`], run from every `<` at once and memoized.
#[inline(never)]
fn resolve_lists(tokens: &Tokens, start: usize) -> Option<usize> {
    let Tokens { src, st, opch, kind, n, closers, .. } = *tokens;
    let mut stack: Vec<Open> = Vec::with_capacity(16);
    stack.push(Open { pos: start as u32, parens: 0, brackets: 0, braces: 0, dead: 0 });
    let (mut parens, mut brackets, mut braces) = (0i32, 0i32, 0i32);
    let mut i = start + 1;
    while i < n {
        if bits::get(st, i) {
            match past_raw(src, kind, n, i) {
                Some(j) if j != i => {
                    i = j;
                    continue;
                }
                Some(_) => {}
                None => break,
            }
        }
        let c = src[i];
        if GT_SCAN_DELIM[c as usize] && bits::get(st, i) {
            let op = bits::get(opch, i);
            match c {
                b'<' => {
                    if op && src[i + 1] != b'=' {
                        stack.push(Open { pos: i as u32, parens, brackets, braces, dead: 0 });
                    }
                }
                b'>' => {
                    if op && !(i > 0 && src[i - 1] == b'=') {
                        // Never empty: start is popped last.
                        let top = stack.last_mut().unwrap();
                        if top.dead > 0 {
                            top.dead -= 1;
                            if top.dead == 0 {
                                stack.pop();
                            }
                        } else {
                            let e = stack.pop().unwrap();
                            let balanced =
                                e.parens == parens && e.brackets == brackets && e.braces == braces;
                            let closer = balanced.then_some(i);
                            closers.set(e.pos as usize, closer);
                            if e.pos as usize == start {
                                return closer;
                            }
                        }
                    }
                }
                b'(' => parens += 1,
                b')' => {
                    parens -= 1;
                    kill(&mut stack, closers, |e| e.parens > parens);
                }
                b'[' => brackets += 1,
                b']' => {
                    brackets -= 1;
                    kill(&mut stack, closers, |e| e.brackets > brackets);
                }
                b'{' => braces += 1,
                b'}' => {
                    let kk = kind_at(kind, i);
                    if !matches_tk!(kk, TemplateMiddle | TemplateTail) {
                        braces -= 1;
                        kill(&mut stack, closers, |e| e.braces > braces);
                    }
                }
                _ => kill(&mut stack, closers, |e| {
                    e.parens == parens && e.brackets == brackets && e.braces == braces
                }),
            }
            if stack[0].dead > 0 {
                // start died, and everything above it went with it.
                return None;
            }
        }
        i += 1;
    }
    for e in &stack {
        if e.dead == 0 {
            closers.set(e.pos as usize, None);
        }
    }
    None
}

/// Live openers' bracket counts only grow upward, so nothing below the first survivor is doomed.
fn kill(stack: &mut Vec<Open>, closers: &Closers, doomed: impl Fn(&Open) -> bool) {
    let mut dead = 0u32;
    while let Some(top) = stack.last() {
        if top.dead > 0 {
            dead += top.dead;
        } else if doomed(top) {
            closers.set(top.pos as usize, None);
            dead += 1;
        } else {
            break;
        }
        stack.pop();
    }
    if dead > 0 {
        stack.push(Open { pos: 0, parens: 0, brackets: 0, braces: 0, dead });
    }
}

#[inline(never)]
fn resolve_groups(tokens: &Tokens, start: usize) -> Option<usize> {
    let Tokens { src, st, kind, n, closers, .. } = *tokens;
    let mut open: Vec<u32> = Vec::with_capacity(16);
    open.push(start as u32);
    let mut i = start + 1;
    while i < n {
        if bits::get(st, i) {
            match past_raw(src, kind, n, i) {
                Some(j) if j != i => {
                    i = j;
                    continue;
                }
                Some(_) => {}
                None => break,
            }
            match src[i] {
                b'(' => open.push(i as u32),
                b')' => {
                    // Never empty: start is popped last.
                    let lp = open.pop().unwrap() as usize;
                    closers.set(lp, Some(i));
                    if lp == start {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    for lp in open {
        closers.set(lp as usize, None);
    }
    None
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
            b'/' => line_terminator_after(src, n, i + 2),
            b'*' => {
                let e = block_comment_end(src, n, i + 2);
                if e < n { e + 1 } else { n }
            }
            _ => i,
        },
        b'"' | b'\'' => past_raw_string(src, n, i, c),
        b'`' => raw_template_end(src, n, i).1.min(n),
        _ => i,
    }
}

fn past_raw_string(src: &[u8], lim: usize, i: usize, quote: u8) -> usize {
    let mut j = i + 1;
    while j < lim {
        let d = src[j];
        if d == b'\\' {
            j += 2;
            continue;
        }
        if d == quote || d == b'\n' || d == b'\r' {
            return j + 1;
        }
        j += 1;
    }
    lim
}

/// For a template head whose backtick is at `i`: the `}` closing its first substitution and the
/// end (exclusive) of the whole literal, over raw bytes. `lim` when the `}` is not found before
/// `lim`, `lim + 1` when the literal does not end before it. Cold: a template inside a
/// speculated type-argument list.
pub(super) fn raw_template_end(src: &[u8], lim: usize, i: usize) -> (usize, usize) {
    let mut first_close = lim;
    let mut subs: Vec<u32> = Vec::new();
    let mut in_text = true;
    let mut j = i + 1;
    while j < lim {
        let c = src[j];
        if in_text {
            match c {
                b'\\' => j += 2,
                b'`' => {
                    if subs.is_empty() {
                        return (first_close, j + 1);
                    }
                    in_text = false;
                    j += 1;
                }
                b'$' if src[j + 1] == b'{' => {
                    subs.push(0);
                    in_text = false;
                    j += 2;
                }
                _ => j += 1,
            }
        } else {
            match c {
                b'{' => {
                    if let Some(d) = subs.last_mut() {
                        *d += 1;
                    }
                    j += 1;
                }
                b'}' => {
                    match subs.last_mut() {
                        Some(d) if *d > 0 => *d -= 1,
                        _ => {
                            subs.pop();
                            in_text = true;
                            if subs.is_empty() && first_close == lim {
                                first_close = j;
                            }
                        }
                    }
                    j += 1;
                }
                b'`' => {
                    in_text = true;
                    j += 1;
                }
                b'"' | b'\'' => j = past_raw_string(src, lim, j, c),
                b'/' => match src[j + 1] {
                    b'/' => j = line_terminator_after(src, lim, j + 2),
                    b'*' => {
                        let e = block_comment_end(src, lim, j + 2);
                        if e >= lim {
                            return (lim, lim + 1);
                        }
                        j = e + 1;
                    }
                    _ => j += 1,
                },
                _ => j += 1,
            }
        }
    }
    (lim, lim + 1)
}
