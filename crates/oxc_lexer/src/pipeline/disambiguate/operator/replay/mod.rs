//! Is `yield` or `await` a keyword at this position?
//!
//! Outside modules, `yield` and `await` are keywords in some places, and ordinary identifiers elsewhere.
//! That decides whether a `/` after one starts a regex (`yield /re/`) or is division (`yield / 2`).
//!
//! - `yield` is a keyword inside a generator, and in strict mode code.
//! - `await` is a keyword inside an async function.
//! - Both are reserved inside a class's `static` block.
//!
//! That depends on every enclosing function, which is hard to see by walking backwards.
//! So [`replay_is_keyword`] replays the tokens forwards from the start of the file,
//! keeping a stack of the functions, classes and object literals it's inside.
//! A `"use strict"` directive or a class body makes the code inside strict.
//!
//! This only runs when `yield` or `await` comes directly before a `/`, or before a `<` in a JSX file.
//! That is rare, so replaying from the start of the file is affordable.

use crate::token::tk;

use crate::pipeline::{
    bitmap::bm_next1,
    tables::{OP_KIND_BASE, Tables},
};

use crate::pipeline::disambiguate::common::{
    AngleMatch, angle_match_back, bm_prev_sig, ident_is, lt_in_range, match_delim_back,
    operand_position, prop_name, return_type_signature_paren, tail_before,
};

#[cfg(test)]
mod tests;

const MAX_SCOPES: usize = 512;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Pop {
    Brace = 0,
    Paren = 1,
    Concise = 2,
}

#[derive(Clone, Copy)]
struct Scope {
    is_gen: bool,
    asyn: bool,
    strict: bool,
    reserved: bool,
    is_class: bool,
    is_obj: bool,
    pop: Pop,
    par: i32,
    brk: i32,
    brc: i32,
    tdep: i32,
    qdebt: u32,
}

impl Scope {
    fn child(&self) -> Scope {
        Scope { is_class: false, is_obj: false, pop: Pop::Brace, qdebt: 0, ..*self }
    }
}

pub(super) unsafe fn replay_is_keyword(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    site: usize,
    ts: bool,
    is_await: bool,
) -> bool {
    let mut scopes: Vec<Scope> = Vec::with_capacity(16);
    scopes.push(Scope {
        is_gen: false,
        asyn: false,
        strict: false,
        reserved: false,
        is_class: false,
        is_obj: false,
        pop: Pop::Brace,
        par: 0,
        brk: 0,
        brc: 0,
        tdep: 0,
        qdebt: 0,
    });
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let mut tdepth: i32 = 0;
    let mut pending_class: Vec<(i32, i32, i32)> = Vec::new();
    let mut prologue: u8 = 1;
    let mut prev_end: usize = 0;

    let mut i = 0usize;
    loop {
        loop {
            i = bm_next1(st, i, n);
            if i >= site {
                asi_pop_concise(
                    t,
                    &mut scopes,
                    src,
                    st,
                    kind,
                    n,
                    prev_end,
                    site,
                    par,
                    brk,
                    brc,
                    tdepth,
                    ts,
                );
                let top = *scopes.last().unwrap();
                return if is_await {
                    top.asyn || top.reserved
                } else {
                    top.is_gen || top.strict || top.reserved
                };
            }
            let k = *kind.add(i);
            if k == tk!(Whitespace)
                || k == tk!(LineComment)
                || k == tk!(BlockComment)
                || k == tk!(Hashbang)
            {
                i += 1;
                continue;
            }
            break;
        }
        let pos = i;
        let k = *kind.add(pos);
        i = pos + 1;
        asi_pop_concise(t, &mut scopes, src, st, kind, n, prev_end, pos, par, brk, brc, tdepth, ts);
        prev_end = bm_next1(st, pos + 1, n);

        if prologue != 0 {
            if k == tk!(String) {
                let e = bm_next1(st, pos + 1, n).min(site);
                let mut j = e;
                let confirmed;
                loop {
                    j = bm_next1(st, j, n);
                    if j >= site {
                        confirmed = true;
                        break;
                    }
                    let jk = *kind.add(j);
                    if jk == tk!(Whitespace) || jk == tk!(LineComment) || jk == tk!(BlockComment) {
                        j += 1;
                        continue;
                    }
                    confirmed = (jk >= OP_KIND_BASE
                        && (*src.add(j) == b';' || *src.add(j) == b'}'))
                        || (lt_in_range(src, e, j) && !continues_expression(src, kind, j, ts));
                    break;
                }
                if confirmed {
                    if e - pos == 12
                        && (ident_is(src, pos + 1, b"use strict"))
                        && (*src.add(pos) == b'"' || *src.add(pos) == b'\'')
                        && *src.add(e - 1) == *src.add(pos)
                    {
                        scopes.last_mut().unwrap().strict = true;
                    }
                    prologue = 2;
                } else {
                    prologue = 0;
                }
                continue;
            }
            if k >= OP_KIND_BASE && *src.add(pos) == b';' {
                prologue = if prologue == 2 { 1 } else { 0 };
            } else {
                prologue = 0;
            }
        }

        if k < OP_KIND_BASE {
            if k == tk!(Ident) && !prop_name(src, pos) && ident_is(src, pos, b"class") {
                pending_class.push((par, brk, brc));
            } else if k == tk!(TemplateHead) {
                tdepth += 1;
            } else if k == tk!(TemplateMiddle) || k == tk!(TemplateTail) {
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if top.pop == Pop::Concise && top.tdep == tdepth {
                        scopes.pop();
                        continue;
                    }
                    break;
                }
                if k == tk!(TemplateTail) {
                    tdepth -= 1;
                }
            }
            continue;
        }
        match *src.add(pos) {
            b'(' => {
                par += 1;
                let enc = *scopes.last().unwrap();
                if let Some((g, a)) = header_kind(src, st, kind, n, pos, enc.is_class, enc.is_obj) {
                    if scopes.len() >= MAX_SCOPES {
                        return true;
                    }
                    let mut s = enc.child();
                    s.is_gen = g;
                    s.asyn = a;
                    s.reserved = false;
                    s.pop = Pop::Paren;
                    s.par = par;
                    s.brk = brk;
                    s.brc = brc;
                    s.tdep = tdepth;
                    scopes.push(s);
                }
            }
            b')' => {
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if (top.pop == Pop::Concise && par - 1 < top.par)
                        || (top.pop == Pop::Paren && top.par == par)
                    {
                        scopes.pop();
                        continue;
                    }
                    break;
                }
                par -= 1;
            }
            b'[' => brk += 1,
            b']' => {
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if top.pop == Pop::Concise && brk - 1 < top.brk {
                        scopes.pop();
                        continue;
                    }
                    break;
                }
                brk -= 1;
            }
            b'{' => {
                brc += 1;
                if scopes.len() >= MAX_SCOPES {
                    return true;
                }
                let enc = *scopes.last().unwrap();
                let q = bm_prev_sig(st, kind, pos);
                let mut s = enc.child();
                s.par = par;
                s.brk = brk;
                s.brc = brc;
                s.tdep = tdepth;
                let mut opened_body = false;
                if q >= 0 {
                    let p = q as usize;
                    let pk = *kind.add(p);
                    if pk >= OP_KIND_BASE {
                        let pb = *src.add(p);
                        if pb == b'>' && p > 0 && *src.add(p - 1) == b'=' {
                            s.is_gen = false;
                            s.asyn = arrow_is_async(src, st, kind, n, p);
                            s.reserved = false;
                            opened_body = true;
                        } else if pb == b')'
                            && let Some(lp) = match_delim_back(src, st, kind, p, b'(', b')')
                            && let Some((g, a)) =
                                header_kind(src, st, kind, n, lp, enc.is_class, enc.is_obj)
                        {
                            s.is_gen = g;
                            s.asyn = a;
                            s.reserved = false;
                            opened_body = true;
                        }
                    }
                    if !opened_body
                        && ts
                        && let Some(lp) = return_type_signature_paren(src, st, kind, pos)
                        && let Some((g, a)) =
                            header_kind(src, st, kind, n, lp, enc.is_class, enc.is_obj)
                    {
                        s.is_gen = g;
                        s.asyn = a;
                        s.reserved = false;
                        opened_body = true;
                    }
                    if !opened_body
                        && pk == tk!(Ident)
                        && enc.is_class
                        && !prop_name(src, p)
                        && ident_is(src, p, b"static")
                    {
                        s.reserved = true;
                        s.strict = true;
                        opened_body = true;
                    }
                    if !opened_body
                        && !pending_class.is_empty()
                        && pk == tk!(Ident)
                        && !prop_name(src, p)
                        && ident_is(src, p, b"extends")
                    {
                        s.is_obj = true;
                        opened_body = true;
                    }
                }
                if !opened_body && !pending_class.is_empty() {
                    pending_class.pop();
                    s.strict = true;
                    s.is_class = true;
                    s.reserved = false;
                    opened_body = true;
                }
                if !opened_body && operand_position(t, src, st, kind, n, pos, ts, 0) {
                    s.is_obj = true;
                    opened_body = true;
                }
                let is_scope_start = opened_body && !s.is_obj && !s.is_class;
                scopes.push(s);
                if is_scope_start {
                    prologue = 1;
                }
            }
            b'}' => {
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if top.pop == Pop::Concise && brc - 1 < top.brc {
                        scopes.pop();
                        continue;
                    }
                    break;
                }
                if scopes.len() <= 1 {
                    return true;
                }
                scopes.pop();
                brc -= 1;
            }
            b'>' if pos > 0 && *src.add(pos - 1) == b'=' => {
                let mut j = pos + 1;
                let mut concise = true;
                loop {
                    j = bm_next1(st, j, n);
                    if j >= site.min(n) {
                        break;
                    }
                    let jk = *kind.add(j);
                    if jk == tk!(Whitespace) || jk == tk!(LineComment) || jk == tk!(BlockComment) {
                        j += 1;
                        continue;
                    }
                    concise = !(jk >= OP_KIND_BASE && *src.add(j) == b'{');
                    break;
                }
                if concise {
                    if scopes.len() >= MAX_SCOPES {
                        return true;
                    }
                    let enc = *scopes.last().unwrap();
                    let mut s = enc.child();
                    s.is_gen = false;
                    s.asyn = arrow_is_async(src, st, kind, n, pos);
                    s.reserved = false;
                    s.pop = Pop::Concise;
                    s.par = par;
                    s.brk = brk;
                    s.brc = brc;
                    s.tdep = tdepth;
                    scopes.push(s);
                }
            }
            b'?' => {
                if *src.add(pos + 1) != b'?'
                    && *src.add(pos + 1) != b'.'
                    && (pos == 0 || *src.add(pos - 1) != b'?')
                    && let Some(top) = scopes.last_mut()
                    && top.pop == Pop::Concise
                    && top.par == par
                    && top.brk == brk
                    && top.brc == brc
                    && top.tdep == tdepth
                {
                    top.qdebt += 1;
                }
            }
            b',' | b';' | b':' => {
                let is_colon = *src.add(pos) == b':';
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if top.pop == Pop::Concise && top.par == par && top.brk == brk && top.brc == brc
                    {
                        if is_colon && top.qdebt > 0 {
                            scopes.last_mut().unwrap().qdebt -= 1;
                            break;
                        }
                        scopes.pop();
                        continue;
                    }
                    break;
                }
                while let Some(&(cp, ck, cb)) = pending_class.last() {
                    if cp == par && ck == brk && cb == brc {
                        pending_class.pop();
                        continue;
                    }
                    break;
                }
            }
            _ => {}
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "cold walker plumbing")]
unsafe fn asi_pop_concise(
    t: &Tables,
    scopes: &mut Vec<Scope>,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    prev_end: usize,
    pos: usize,
    par: i32,
    brk: i32,
    brc: i32,
    tdepth: i32,
    ts: bool,
) {
    if scopes.last().is_none_or(|s| s.pop != Pop::Concise) {
        return;
    }
    if !lt_in_range(src, prev_end, pos)
        || continues_expression(src, kind, pos, ts)
        || !asi_tail_before(t, src, st, kind, n, pos)
    {
        return;
    }
    while scopes.len() > 1 {
        let top = *scopes.last().unwrap();
        if top.pop == Pop::Concise
            && top.par == par
            && top.brk == brk
            && top.brc == brc
            && top.tdep == tdepth
        {
            scopes.pop();
            continue;
        }
        break;
    }
}

unsafe fn continues_expression(src: *const u8, kind: *const u8, pos: usize, ts: bool) -> bool {
    let k = *kind.add(pos);
    if k >= OP_KIND_BASE {
        let c = *src.add(pos);
        if (c == b'+' || c == b'-') && *src.add(pos + 1) == c {
            return false;
        }
        return matches!(
            c,
            b'+' | b'-'
                | b'*'
                | b'/'
                | b'%'
                | b'&'
                | b'|'
                | b'^'
                | b'<'
                | b'>'
                | b'='
                | b'?'
                | b'.'
                | b','
                | b'('
                | b'['
                | b':'
                | b')'
                | b']'
                | b'}'
        );
    }
    if k == tk!(Ident) {
        return ident_is(src, pos, b"in")
            || ident_is(src, pos, b"instanceof")
            || (ts && (ident_is(src, pos, b"as") || ident_is(src, pos, b"satisfies")));
    }
    matches!(k, tk!(TemplateHead) | tk!(TemplateNoSub))
}

unsafe fn asi_tail_before(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
) -> bool {
    if tail_before(t, src, st, kind, n, pos) {
        return true;
    }
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false;
    }
    let p = q as usize;
    if *kind.add(p) < OP_KIND_BASE {
        return false;
    }
    let c = *src.add(p);
    if c == b'}' {
        return true;
    }
    if (c == b'+' || c == b'-') && p > 0 && *src.add(p - 1) == c && (p < 2 || *src.add(p - 2) != c)
    {
        let f = p - 1;
        if tail_before(t, src, st, kind, n, f) {
            return true;
        }
        let b = bm_prev_sig(st, kind, f);
        return b >= 0 && *kind.add(b as usize) >= OP_KIND_BASE && *src.add(b as usize) == b'}';
    }
    false
}

unsafe fn header_kind(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    lp: usize,
    in_class: bool,
    in_obj: bool,
) -> Option<(bool, bool)> {
    let w = bm_prev_sig(st, kind, lp);
    if w < 0 {
        return None;
    }
    let mut p = w as usize;
    if *kind.add(p) >= OP_KIND_BASE && *src.add(p) == b'>' && !(p > 0 && *src.add(p - 1) == b'=') {
        let AngleMatch::Found(lt) = angle_match_back(src, st, kind, p) else {
            return None;
        };
        let w = bm_prev_sig(st, kind, lt);
        if w < 0 {
            return None;
        }
        p = w as usize;
    }
    if *kind.add(p) >= OP_KIND_BASE && *src.add(p) == b'*' {
        let f = bm_prev_sig(st, kind, p);
        if f >= 0 && *kind.add(f as usize) == tk!(Ident) && ident_is(src, f as usize, b"function") {
            let a = bm_prev_sig(st, kind, f as usize);
            let asyn = a >= 0 && async_modifier(src, st, kind, n, a as usize, f as usize);
            return Some((true, asyn));
        }
        return None;
    }
    let named = if *kind.add(p) >= OP_KIND_BASE {
        if *src.add(p) == b']' {
            let Some(lb) = match_delim_back(src, st, kind, p, b'[', b']') else {
                return None;
            };
            p = lb;
            true
        } else {
            return None;
        }
    } else if *kind.add(p) == tk!(Ident) {
        if ident_is(src, p, b"function") && !prop_name(src, p) && !(in_class || in_obj) {
            let a = bm_prev_sig(st, kind, p);
            let asyn = a >= 0 && async_modifier(src, st, kind, n, a as usize, p);
            return Some((false, asyn));
        }
        let f = bm_prev_sig(st, kind, p);
        if f >= 0 {
            let fp = f as usize;
            if *kind.add(fp) == tk!(Ident) && !prop_name(src, fp) && ident_is(src, fp, b"function")
            {
                let a = bm_prev_sig(st, kind, fp);
                let asyn = a >= 0 && async_modifier(src, st, kind, n, a as usize, fp);
                return Some((false, asyn));
            }
            if *kind.add(fp) >= OP_KIND_BASE && *src.add(fp) == b'*' {
                let g = bm_prev_sig(st, kind, fp);
                if g >= 0
                    && *kind.add(g as usize) == tk!(Ident)
                    && ident_is(src, g as usize, b"function")
                {
                    let a = bm_prev_sig(st, kind, g as usize);
                    let asyn = a >= 0 && async_modifier(src, st, kind, n, a as usize, g as usize);
                    return Some((true, asyn));
                }
            }
        }
        true
    } else {
        matches!(
            *kind.add(p),
            tk!(String)
                | tk!(Number)
                | tk!(PrivateIdent)
                | tk!(IdentEscaped)
                | tk!(PrivateIdentEscaped)
        )
    };
    if !named || !(in_class || in_obj) {
        return None;
    }
    let mut is_gen = false;
    let mut asyn = false;
    let mut cur = p;
    for _ in 0..4 {
        let m = bm_prev_sig(st, kind, cur);
        if m < 0 {
            break;
        }
        let mp = m as usize;
        if *kind.add(mp) >= OP_KIND_BASE {
            if *src.add(mp) == b'*' && !is_gen {
                is_gen = true;
                cur = mp;
                continue;
            }
            break;
        }
        if *kind.add(mp) == tk!(Ident) && !prop_name(src, mp) {
            if !asyn && async_modifier(src, st, kind, n, mp, cur) {
                asyn = true;
                cur = mp;
                continue;
            }
            if ident_is(src, mp, b"get") || ident_is(src, mp, b"set") {
                cur = mp;
                continue;
            }
            if ident_is(src, mp, b"static") {
                cur = mp;
                continue;
            }
        }
        break;
    }
    if is_gen || asyn {
        let cpq = bm_prev_sig(st, kind, cur);
        let mut method_pos = false;
        if cpq >= 0 {
            let cp = cpq as usize;
            if *kind.add(cp) >= OP_KIND_BASE {
                let c = *src.add(cp);
                method_pos = c == b'{' || c == b',' || c == b';' || (c == b'}' && in_class);
            }
            if !method_pos && in_class {
                let ce = bm_next1(st, cp + 1, n);
                method_pos = lt_in_range(src, ce, cur);
            }
        }
        if !method_pos {
            if *kind.add(p) == tk!(Ident) && !prop_name(src, p) && ident_is(src, p, b"function") {
                let a = bm_prev_sig(st, kind, p);
                let asy = a >= 0 && async_modifier(src, st, kind, n, a as usize, p);
                return Some((false, asy));
            }
            return None;
        }
    }
    Some((is_gen, asyn))
}

unsafe fn arrow_is_async(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    gt: usize,
) -> bool {
    let h = bm_prev_sig(st, kind, gt.saturating_sub(1));
    if h < 0 {
        return false;
    }
    let hp = h as usize;
    if *kind.add(hp) == tk!(Ident) {
        let a = bm_prev_sig(st, kind, hp);
        if a >= 0 && async_modifier(src, st, kind, n, a as usize, hp) {
            return true;
        }
    } else if *kind.add(hp) >= OP_KIND_BASE
        && *src.add(hp) == b')'
        && let Some(lp) = match_delim_back(src, st, kind, hp, b'(', b')')
    {
        let a = bm_prev_sig(st, kind, lp);
        if a >= 0 && async_modifier(src, st, kind, n, a as usize, lp) {
            return true;
        }
    }
    if let Some(lp) = return_type_signature_paren(src, st, kind, gt.saturating_sub(1)) {
        let a = bm_prev_sig(st, kind, lp);
        return a >= 0 && async_modifier(src, st, kind, n, a as usize, lp);
    }
    false
}

unsafe fn async_modifier(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
    next: usize,
) -> bool {
    if *kind.add(pos) != tk!(Ident) || prop_name(src, pos) || !ident_is(src, pos, b"async") {
        return false;
    }
    let e = bm_next1(st, pos + 1, n);
    !lt_in_range(src, e, next)
}
