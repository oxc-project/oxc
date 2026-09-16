//! Moving backwards through the token stream.
//!
//! These are the primitives which the backward walks in `disambiguate` are built from.
//! They move around the token stream without deciding what any construct means.
//! Such decisions are left to [`constructs`] and [`operand`].
//!
//! Stepping and inspecting:
//! - [`bm_prev_sig`] steps to the previous token, skipping whitespace and comments.
//! - [`kind_at`] reads a token's kind, treating keywords as plain identifiers,
//!   and escaped identifiers as unescaped ones.
//! - [`ident_is`] and [`word_is_any`] check whether an identifier is a particular word,
//!   like `let`, or any word in a list.
//! - [`prop_name`] checks whether an identifier directly follows a `.`, as in `x.return`,
//!   which makes it a property name rather than a keyword.
//! - [`lt_in_range`] asks whether a line break separates two positions, which ASI depends on.
//!
//! Jumping over bracketed groups:
//! - [`match_delim_back`] goes from a `)`, `]` or `}` back to its opener.
//!   Past [`BRACE_MATCH_CAP`] steps it switches to a table of bracket pairs, built once per lex,
//!   so the answer stays exact.
//! - [`angle_match_back`] goes from a `>` back to a `<` which could open type arguments.
//! - [`chain_head`] goes from the last name in `a.b.c` back to `a`.
//!
//! [`constructs`]: super::constructs
//! [`operand`]: super::operand

use std::cell::{Cell, RefCell};

use crate::{
    opmap::OP_KIND_BASE,
    tables::is_word,
    token::{KW_KIND_BASE, KW_KIND_MAX, tk},
};

use super::super::super::bitmap::{bm_next1, bm_prev1};

const ANGLE_MATCH_CAP: u32 = 4096;

/// Distance cap (in token starts) for the backward delimiter matches below;
/// past it we fall back to the safe legacy "`}` means regex" answer. Only
/// pathological input gets near it.
const BRACE_MATCH_CAP: u32 = 1024;

pub enum AngleMatch {
    Found(usize),
    NotType,
    Unknown,
}

struct DelimMemo {
    generation: u64,
    upto: usize,
    pairs: Vec<(u32, u32)>,
    open: [Vec<u32>; 3],
}

thread_local! {
    static MEMO_GEN: Cell<u64> = const { Cell::new(0) };
    static DELIM_MEMO: RefCell<DelimMemo> = const {
        RefCell::new(DelimMemo {
            generation: 0,
            upto: 0,
            pairs: Vec::new(),
            open: [Vec::new(), Vec::new(), Vec::new()],
        })
    };
}

pub fn memo_new_lex() {
    MEMO_GEN.with(|g| g.set(g.get().wrapping_add(1)));
}

pub unsafe fn angle_match_back(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    gt: usize,
) -> AngleMatch {
    let mut depth: i32 = 1;
    let mut tmpl: i32 = 0;
    let mut steps: u32 = 0;
    let mut q = bm_prev_sig(st, kind, gt);
    while q >= 0 {
        steps += 1;
        if steps > ANGLE_MATCH_CAP {
            return AngleMatch::Unknown;
        }
        let w = q as usize;
        let kk = kind_at(kind, w);
        if tmpl > 0 {
            if kk == tk!(TemplateTail) {
                tmpl += 1;
            } else if kk == tk!(TemplateHead) {
                tmpl -= 1;
            }
            q = bm_prev_sig(st, kind, w);
            continue;
        }
        if kk == tk!(TemplateTail) {
            tmpl = 1;
            q = bm_prev_sig(st, kind, w);
            continue;
        }
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b'>' => {
                    if !(w > 0 && *src.add(w - 1) == b'=') {
                        depth += 1;
                    }
                }
                b'<' => {
                    if *src.add(w + 1) == b'=' {
                        return AngleMatch::NotType;
                    }
                    depth -= 1;
                    if depth == 0 {
                        return AngleMatch::Found(w);
                    }
                }
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, w, open, c) {
                        Some(op) => {
                            q = bm_prev_sig(st, kind, op);
                            continue;
                        }
                        None => return AngleMatch::Unknown,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return AngleMatch::NotType,
                b'.' | b',' | b'|' | b'&' | b'?' | b':' | b'=' | b'+' | b'-' => {}
                _ => return AngleMatch::NotType,
            }
        } else if !matches!(
            kk,
            tk!(Ident)
                | tk!(IdentEscaped)
                | tk!(Number)
                | tk!(BigInt)
                | tk!(String)
                | tk!(TemplateNoSub)
        ) {
            if kk == tk!(TemplateHead) || kk == tk!(TemplateMiddle) {
                return AngleMatch::Unknown;
            }
            return AngleMatch::NotType;
        }
        q = bm_prev_sig(st, kind, w);
    }
    AngleMatch::NotType
}

pub unsafe fn chain_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> Option<usize> {
    let mut head = from;
    while prop_name(src, head) {
        let d = bm_prev_sig(st, kind, head);
        if d < 0 {
            return None;
        }
        let o = bm_prev_sig(st, kind, d as usize);
        if o < 0 {
            return None;
        }
        let op = o as usize;
        if kind_at(kind, op) == tk!(Ident) {
            head = op;
            continue;
        }
        if kind_at(kind, op) >= OP_KIND_BASE && *src.add(op) == b')' {
            let lp = match_delim_back(src, st, kind, op, b'(', b')')?;
            let im = bm_prev_sig(st, kind, lp);
            if im >= 0
                && kind_at(kind, im as usize) == tk!(Ident)
                && ident_is(src, im as usize, b"import")
            {
                return Some(im as usize);
            }
        }
        return None;
    }
    Some(head)
}

/// Match the close punctuator at `from` back to its opener, counting only
/// punctuator delimiters - template-closing `}`s and cleared literal
/// interiors are invisible. Past the cap the answer comes from a per-file
/// closer-to-opener table built once, so it is exact; None if unbalanced.
#[inline]
pub unsafe fn match_delim_back(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut steps: u32 = 0;
    let mut q = bm_prev1(st, from);
    while q >= 0 {
        steps += 1;
        if steps > BRACE_MATCH_CAP {
            return delim_memo_opener(src, st, kind, from);
        }
        let pos = q as usize;
        if *kind.add(pos) >= OP_KIND_BASE {
            let c = *src.add(pos);
            if c == close {
                depth += 1;
            } else if c == open {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
            }
        }
        q = bm_prev1(st, pos);
    }
    None
}

#[inline(never)]
unsafe fn delim_memo_opener(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> Option<usize> {
    let generation = MEMO_GEN.with(Cell::get);
    DELIM_MEMO.with(|cell| {
        let mut m = cell.borrow_mut();
        if m.generation != generation {
            m.generation = generation;
            m.upto = 0;
            m.pairs.clear();
            for stack in &mut m.open {
                stack.clear();
            }
        }
        let mut w = bm_next1(st, m.upto, from + 1);
        while w <= from {
            if *kind.add(w) >= OP_KIND_BASE {
                let c = *src.add(w);
                match c {
                    b'(' | b'[' | b'{' => m.open[delim_slot(c)].push(w as u32),
                    b')' | b']' | b'}' => {
                        if let Some(o) = m.open[delim_slot(c)].pop() {
                            m.pairs.push((w as u32, o));
                        }
                    }
                    _ => {}
                }
            }
            w = bm_next1(st, w + 1, from + 1);
        }
        m.upto = from + 1;
        match m.pairs.binary_search_by_key(&(from as u32), |pr| pr.0) {
            Ok(i) => Some(m.pairs[i].1 as usize),
            Err(_) => None,
        }
    })
}

fn delim_slot(c: u8) -> usize {
    match c {
        b'(' | b')' => 0,
        b'[' | b']' => 1,
        _ => 2,
    }
}

pub unsafe fn word_is_any(src: *const u8, w: usize, words: &[&[u8]]) -> bool {
    let len = word_len(src, w);
    let first = *src.add(w);
    let mut i = 0;
    while i < words.len() {
        let kw = words[i];
        if kw.len() == len && kw[0] == first && ident_is(src, w, kw) {
            return true;
        }
        i += 1;
    }
    false
}

#[inline(always)]
pub unsafe fn word_len(src: *const u8, w: usize) -> usize {
    let mut e = w + 1;
    while is_word(*src.add(e)) {
        e += 1;
    }
    e - w
}

/// Does the identifier at `pos` equal exactly `kw`? The following-byte check
/// rejects longer identifiers (the source pad makes it safe at EOF).
#[inline]
pub unsafe fn ident_is(src: *const u8, pos: usize, kw: &[u8]) -> bool {
    let mut i = 0;
    while i < kw.len() {
        if *src.add(pos + i) != kw[i] {
            return false;
        }
        i += 1;
    }
    let after = pos + kw.len();
    !is_word(*src.add(after)) || trivia_at(src, after).is_some()
}

#[inline]
pub unsafe fn trivia_at(src: *const u8, i: usize) -> Option<(bool, usize)> {
    let b = *src.add(i);
    if b < 0x80 {
        return None;
    }
    let b1 = *src.add(i + 1);
    let b2 = *src.add(i + 2);
    match b {
        0xc2 if b1 == 0xa0 || b1 == 0x85 => Some((false, 2)),
        0xe1 if b1 == 0x9a && b2 == 0x80 => Some((false, 3)),
        0xe2 if b1 == 0x80 && ((0x80..=0x8b).contains(&b2) || b2 == 0xaf) => Some((false, 3)),
        0xe2 if b1 == 0x80 && (b2 == 0xa8 || b2 == 0xa9) => Some((true, 3)),
        0xe2 if b1 == 0x81 && b2 == 0x9f => Some((false, 3)),
        0xe3 if b1 == 0x80 && b2 == 0x80 => Some((false, 3)),
        0xef if b1 == 0xbb && b2 == 0xbf => Some((false, 3)),
        _ => None,
    }
}

/// Previous significant token start before `pos` (skipping trivia), or -1 at
/// start of input.
#[inline]
pub unsafe fn bm_prev_sig(st: *const u64, kind: *const u8, pos: usize) -> i64 {
    let mut q = bm_prev1(st, pos);
    while q >= 0 {
        let k = *kind.add(q as usize);
        if k == tk!(Whitespace)
            || k == tk!(LineComment)
            || k == tk!(BlockComment)
            || k == tk!(Hashbang)
        {
            q = bm_prev1(st, q as usize);
            continue;
        }
        break;
    }
    q
}

#[inline(always)]
pub unsafe fn kind_at(kind: *const u8, w: usize) -> u8 {
    let k = *kind.add(w);
    if k >= KW_KIND_BASE && k <= KW_KIND_MAX {
        return tk!(Ident);
    }
    if k == tk!(IdentEscaped) || k == tk!(PrivateIdentEscaped) {
        return k & !(tk!(IdentEscaped) ^ tk!(Ident));
    }
    k
}

#[inline]
pub unsafe fn prop_name(src: *const u8, pos: usize) -> bool {
    pos > 0 && *src.add(pos - 1) == b'.' && (pos < 2 || *src.add(pos - 2) != b'.')
}

pub unsafe fn lt_in_range(src: *const u8, a: usize, b: usize) -> bool {
    let mut i = a;
    while i < b {
        let c = *src.add(i);
        if c == b'\n' || c == b'\r' {
            return true;
        }
        if c == 0xe2
            && *src.add(i + 1) == 0x80
            && (*src.add(i + 2) == 0xa8 || *src.add(i + 2) == 0xa9)
        {
            return true;
        }
        i += 1;
    }
    false
}
