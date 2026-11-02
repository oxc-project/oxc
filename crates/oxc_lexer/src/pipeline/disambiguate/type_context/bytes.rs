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
//! [`match_delim_back`]: crate::pipeline::disambiguate::common::Tokens::match_delim_back

use crate::{
    cpu::{Cpu, apply, cpu_versions},
    token::{OP_KIND_BASE, matches_tk, tk},
};

use crate::pipeline::{
    bytes::{
        block_comment_end, is_id_start, is_ws, line_break_in, line_terminator_after,
        unicode_ws_len_at,
    },
    disambiguate::{
        FORWARD_SCAN_CAP,
        common::{Closers, Open, Tokens, bits},
    },
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
#[apply(cpu_versions)]
#[inline(never)]
pub fn lt_run_split(tokens: &Tokens, lt: usize, cpu: impl Cpu) -> bool {
    let Tokens { src, st, n, .. } = *tokens;
    // A TypeParameter starts with an identifier (or the `const` modifier),
    // which is what makes `<<=` cost two bytes to reject.
    let head = skip_trivia_fwd(src, n, lt + 2, cpu);
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
    let Some(gt) = list_closer(tokens, lt + 1, cpu) else {
        return false;
    };
    let lp = skip_trivia_fwd(src, n, gt + 1, cpu);
    if lp >= n || src[lp] != b'(' || !arrow_after_params(tokens, lp, cpu) {
        return false;
    }
    // The outer list must close too, or this was a comparison against a
    // generic arrow function and the parser would have backtracked as well.
    list_closer(tokens, lt, cpu).is_some()
}

#[apply(cpu_versions)]
pub(super) fn arrow_after_params(tokens: &Tokens, lp: usize, cpu: impl Cpu) -> bool {
    let Tokens { src, n, .. } = *tokens;
    let Some(rp) = group_closer(tokens, lp, cpu) else {
        return false;
    };
    let ar = skip_trivia_fwd(src, n, rp + 1, cpu);
    ar < n && src[ar] == b'=' && src[ar + 1] == b'>'
}

#[apply(cpu_versions)]
pub(super) fn skip_trivia_fwd(src: &[u8], n: usize, i: usize, cpu: impl Cpu) -> usize {
    skip_trivia_nl(src, n, i, cpu).0
}

/// The first non-trivia byte at or after i (or n), and whether a line terminator came first.
#[inline(always)]
pub(super) fn skip_trivia_nl(src: &[u8], n: usize, mut i: usize, cpu: impl Cpu) -> (usize, bool) {
    let mut broke = false;
    if i < n && !is_ws(src[i]) && src[i] != b'/' && src[i] < 0x80 {
        return (i, false);
    }
    while i < n {
        let c = src[i];
        if is_ws(c) {
            broke |= c == b'\n' || c == b'\r';
            i += 1;
        } else if c == b'/' && src[i + 1] == b'/' {
            i = line_terminator_after(src, n, i + 2, cpu);
        } else if c == b'/' && src[i + 1] == b'*' {
            let e = block_comment_end(src, n, i + 2, cpu);
            if e >= n {
                return (n, broke);
            }
            broke |= line_break_in(src, i + 2, e, cpu);
            i = e + 1;
        } else if c >= 0x80 && unicode_ws_len_at(src, i, cpu) != 0 {
            broke |= line_break_in(src, i, i + 3, cpu);
            i += unicode_ws_len_at(src, i, cpu);
        } else {
            break;
        }
    }
    (i, broke)
}

#[apply(cpu_versions)]
pub(super) fn list_closer(tokens: &Tokens, lt: usize, cpu: impl Cpu) -> Option<usize> {
    match tokens.closers.get(lt, cpu) {
        Some(r) => r.closer(cpu),
        None => resolve_lists(tokens, lt, cpu),
    }
}

#[apply(cpu_versions)]
fn group_closer(tokens: &Tokens, lp: usize, cpu: impl Cpu) -> Option<usize> {
    match tokens.closers.get(lp, cpu) {
        Some(r) => r.closer(cpu),
        None => resolve_groups(tokens, lp, cpu),
    }
}

#[apply(cpu_versions)]
#[inline]
fn past_raw(tokens: &Tokens, lim: usize, i: usize, cpu: impl Cpu) -> Option<usize> {
    let (src, kind) = (tokens.src, tokens.kind);
    let j = skip_raw_literal(src, kind, lim, i, cpu);
    if j != i {
        return (j < lim).then_some(j);
    }
    // Asked from the JSX carve, a template head may be carved while its tail is still
    // raw text (its substitution `}` a punctuator): cross the literal whole.
    if tokens.base_kind(i) == tk!(TemplateHead) {
        let (close, end) = raw_template_end(src, lim, i, cpu);
        if close < lim && tokens.base_kind(close) >= OP_KIND_BASE {
            return (end <= lim).then_some(end);
        }
    }
    Some(i)
}

/// The > closing the < at start, if the brackets inside balance and no ; on its level ends it.
#[apply(cpu_versions)]
#[inline(never)]
fn resolve_lists(tokens: &Tokens, start: usize, cpu: impl Cpu) -> Option<usize> {
    let Tokens { src, st, opch, n, closers, .. } = *tokens;
    let budget = start + FORWARD_SCAN_CAP;
    let mut stack = closers.lists.borrow_mut();
    stack.clear();
    stack.push(Open { pos: start as u32, parens: 0, brackets: 0, braces: 0, dead: 0 });
    let (mut parens, mut brackets, mut braces) = (0i32, 0i32, 0i32);
    let mut i = start + 1;
    while i < n {
        if bits::get(st, i) {
            match past_raw(tokens, n, i, cpu) {
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
                            if i > budget {
                                closers.set(e.pos as usize, closer, cpu);
                            }
                            if e.pos as usize == start {
                                return closer;
                            }
                        }
                    }
                }
                b'(' => parens += 1,
                b')' => {
                    parens -= 1;
                    kill(&mut stack, closers, i > budget, |e| e.parens > parens, cpu);
                }
                b'[' => brackets += 1,
                b']' => {
                    brackets -= 1;
                    kill(&mut stack, closers, i > budget, |e| e.brackets > brackets, cpu);
                }
                b'{' => braces += 1,
                b'}' => {
                    if !matches_tk!(tokens.base_kind(i), TemplateMiddle | TemplateTail) {
                        braces -= 1;
                        kill(&mut stack, closers, i > budget, |e| e.braces > braces, cpu);
                    }
                }
                _ => kill(
                    &mut stack,
                    closers,
                    i > budget,
                    |e| e.parens == parens && e.brackets == brackets && e.braces == braces,
                    cpu,
                ),
            }
            if stack[0].dead > 0 {
                // start died, and everything above it went with it.
                return None;
            }
        }
        i += 1;
    }
    if n > budget {
        for e in stack.iter().filter(|e| e.dead == 0) {
            closers.set(e.pos as usize, None, cpu);
        }
    }
    None
}

/// Live openers' bracket counts only grow upward, so nothing below the first survivor is doomed.
#[apply(cpu_versions)]
fn kill(
    stack: &mut Vec<Open>,
    closers: &Closers,
    record: bool,
    doomed: impl Fn(&Open) -> bool,
    cpu: impl Cpu,
) {
    let mut dead = 0u32;
    while let Some(top) = stack.last() {
        if top.dead > 0 {
            dead += top.dead;
        } else if doomed(top) {
            if record {
                closers.set(top.pos as usize, None, cpu);
            }
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

#[apply(cpu_versions)]
#[inline(never)]
fn resolve_groups(tokens: &Tokens, start: usize, cpu: impl Cpu) -> Option<usize> {
    let Tokens { src, st, n, closers, .. } = *tokens;
    let budget = start + FORWARD_SCAN_CAP;
    let mut open = closers.groups.borrow_mut();
    open.clear();
    open.push(start as u32);
    let mut i = start + 1;
    while i < n {
        if bits::get(st, i) {
            match past_raw(tokens, n, i, cpu) {
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
                    if i > budget {
                        closers.set(lp, Some(i), cpu);
                    }
                    if lp == start {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    if n > budget {
        for &lp in open.iter() {
            closers.set(lp as usize, None, cpu);
        }
    }
    None
}

/// If a comment, string or template carve has not reached yet starts at the token start `i`, the
/// position just past it; else `i`. Forward scans from before the carve cursor cross raw text;
/// carved literals have cleared interiors and are never re-entered.
#[apply(cpu_versions)]
#[inline]
pub(super) fn skip_raw_literal(
    src: &[u8],
    kind: &[u8],
    n: usize,
    i: usize,
    cpu: impl Cpu,
) -> usize {
    if kind[i] < OP_KIND_BASE {
        return i;
    }
    let c = src[i];
    match c {
        b'/' => match src[i + 1] {
            b'/' => line_terminator_after(src, n, i + 2, cpu),
            b'*' => {
                let e = block_comment_end(src, n, i + 2, cpu);
                if e < n { e + 1 } else { n }
            }
            _ => i,
        },
        b'"' | b'\'' => past_raw_string(src, n, i, c, cpu),
        b'`' => raw_template_end(src, n, i, cpu).1.min(n),
        _ => i,
    }
}

#[apply(cpu_versions)]
fn past_raw_string(src: &[u8], lim: usize, i: usize, quote: u8, _cpu: impl Cpu) -> usize {
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

/// For a template head whose backtick is at i: the brace closing its last substitution and the
/// end (exclusive) of the whole literal, over raw bytes. `lim` when the `}` is not found before
/// `lim`, `lim + 1` when the literal does not end before it. Cold: a template inside a
/// speculated type-argument list.
#[apply(cpu_versions)]
pub(super) fn raw_template_end(src: &[u8], lim: usize, i: usize, cpu: impl Cpu) -> (usize, usize) {
    let mut last_close = lim;
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
                        return (last_close, j + 1);
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
                            if subs.is_empty() {
                                last_close = j;
                            }
                        }
                    }
                    j += 1;
                }
                b'`' => {
                    in_text = true;
                    j += 1;
                }
                b'"' | b'\'' => j = past_raw_string(src, lim, j, c, cpu),
                b'/' => match src[j + 1] {
                    b'/' => j = line_terminator_after(src, lim, j + 2, cpu),
                    b'*' => {
                        let e = block_comment_end(src, lim, j + 2, cpu);
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
