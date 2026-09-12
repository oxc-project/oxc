use crate::{opmap::OP_KIND_BASE, tables::Tables};

use super::super::super::{
    BCOM, HASHBANG, IDENT, IDENT_ESC, LCOM, NUM, PRIV_IDENT, PRIV_IDENT_ESC, STR, TMPL_HEAD,
    TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS, bitmap::bm_next1,
};

use super::super::{
    AngleMatch, angle_match_back, bm_prev_sig, ident_is, lt_in_range, match_delim_back,
    operand_position, prop_name, return_type_signature_paren, tail_before,
};

#[cfg(test)]
mod tests;

const POP_BRACE: u8 = 0;
const POP_PAREN: u8 = 1;
const POP_CONCISE: u8 = 2;

const MAX_SCOPES: usize = 512;

#[derive(Clone, Copy)]
struct Scope {
    is_gen: bool,
    asyn: bool,
    strict: bool,
    reserved: bool,
    is_class: bool,
    is_obj: bool,
    pop: u8,
    par: i32,
    brk: i32,
    brc: i32,
    tdep: i32,
    qdebt: u32,
}

impl Scope {
    fn child(&self) -> Scope {
        Scope { is_class: false, is_obj: false, pop: POP_BRACE, qdebt: 0, ..*self }
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
        pop: POP_BRACE,
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
            if k == WS || k == LCOM || k == BCOM || k == HASHBANG {
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
            if k == STR {
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
                    if jk == WS || jk == LCOM || jk == BCOM {
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
            if k == IDENT && !prop_name(src, pos) && ident_is(src, pos, b"class") {
                pending_class.push((par, brk, brc));
            } else if k == TMPL_HEAD {
                tdepth += 1;
            } else if k == TMPL_MIDDLE || k == TMPL_TAIL {
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if top.pop == POP_CONCISE && top.tdep == tdepth {
                        scopes.pop();
                        continue;
                    }
                    break;
                }
                if k == TMPL_TAIL {
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
                    s.pop = POP_PAREN;
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
                    if (top.pop == POP_CONCISE && par - 1 < top.par)
                        || (top.pop == POP_PAREN && top.par == par)
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
                    if top.pop == POP_CONCISE && brk - 1 < top.brk {
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
                        } else if pb == b')' {
                            if let Some(lp) = match_delim_back(src, st, kind, p, b'(', b')') {
                                if let Some((g, a)) =
                                    header_kind(src, st, kind, n, lp, enc.is_class, enc.is_obj)
                                {
                                    s.is_gen = g;
                                    s.asyn = a;
                                    s.reserved = false;
                                    opened_body = true;
                                }
                            }
                        }
                    }
                    if !opened_body && ts {
                        if let Some(lp) = return_type_signature_paren(src, st, kind, pos) {
                            if let Some((g, a)) =
                                header_kind(src, st, kind, n, lp, enc.is_class, enc.is_obj)
                            {
                                s.is_gen = g;
                                s.asyn = a;
                                s.reserved = false;
                                opened_body = true;
                            }
                        }
                    }
                    if !opened_body
                        && pk == IDENT
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
                        && pk == IDENT
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
                    if top.pop == POP_CONCISE && brc - 1 < top.brc {
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
                    if jk == WS || jk == LCOM || jk == BCOM {
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
                    s.pop = POP_CONCISE;
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
                {
                    if let Some(top) = scopes.last_mut() {
                        if top.pop == POP_CONCISE
                            && top.par == par
                            && top.brk == brk
                            && top.brc == brc
                            && top.tdep == tdepth
                        {
                            top.qdebt += 1;
                        }
                    }
                }
            }
            b',' | b';' | b':' => {
                let is_colon = *src.add(pos) == b':';
                while scopes.len() > 1 {
                    let top = *scopes.last().unwrap();
                    if top.pop == POP_CONCISE && top.par == par && top.brk == brk && top.brc == brc
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
    if scopes.last().is_none_or(|s| s.pop != POP_CONCISE) {
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
        if top.pop == POP_CONCISE
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
    if k == IDENT {
        return ident_is(src, pos, b"in")
            || ident_is(src, pos, b"instanceof")
            || (ts && (ident_is(src, pos, b"as") || ident_is(src, pos, b"satisfies")));
    }
    matches!(k, TMPL_HEAD | TMPL_NOSUB)
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
        if f >= 0 && *kind.add(f as usize) == IDENT && ident_is(src, f as usize, b"function") {
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
    } else if *kind.add(p) == IDENT {
        if ident_is(src, p, b"function") && !prop_name(src, p) && !(in_class || in_obj) {
            let a = bm_prev_sig(st, kind, p);
            let asyn = a >= 0 && async_modifier(src, st, kind, n, a as usize, p);
            return Some((false, asyn));
        }
        let f = bm_prev_sig(st, kind, p);
        if f >= 0 {
            let fp = f as usize;
            if *kind.add(fp) == IDENT && !prop_name(src, fp) && ident_is(src, fp, b"function") {
                let a = bm_prev_sig(st, kind, fp);
                let asyn = a >= 0 && async_modifier(src, st, kind, n, a as usize, fp);
                return Some((false, asyn));
            }
            if *kind.add(fp) >= OP_KIND_BASE && *src.add(fp) == b'*' {
                let g = bm_prev_sig(st, kind, fp);
                if g >= 0
                    && *kind.add(g as usize) == IDENT
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
        matches!(*kind.add(p), STR | NUM | PRIV_IDENT | IDENT_ESC | PRIV_IDENT_ESC)
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
        if *kind.add(mp) == IDENT && !prop_name(src, mp) {
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
            if *kind.add(p) == IDENT && !prop_name(src, p) && ident_is(src, p, b"function") {
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
    if *kind.add(hp) == IDENT {
        let a = bm_prev_sig(st, kind, hp);
        if a >= 0 && async_modifier(src, st, kind, n, a as usize, hp) {
            return true;
        }
    } else if *kind.add(hp) >= OP_KIND_BASE && *src.add(hp) == b')' {
        if let Some(lp) = match_delim_back(src, st, kind, hp, b'(', b')') {
            let a = bm_prev_sig(st, kind, lp);
            if a >= 0 && async_modifier(src, st, kind, n, a as usize, lp) {
                return true;
            }
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
    if *kind.add(pos) != IDENT || prop_name(src, pos) || !ident_is(src, pos, b"async") {
        return false;
    }
    let e = bm_next1(st, pos + 1, n);
    !lt_in_range(src, e, next)
}
