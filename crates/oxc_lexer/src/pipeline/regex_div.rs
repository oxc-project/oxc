use crate::opmap::{OP_KIND_BASE, OP_QDOT};
use crate::tables::{Tables, is_digit, is_glue_join, is_word, is_ws};

use super::bitmap::{bm_next0, bm_next1, bm_prev1};
use super::find::scan_number;
use super::{
    BCOM, BIGINT, HASHBANG, IDENT, IDENT_ESC, JEND, JSX_LT, LCOM, NUM, PRIV_IDENT, PRIV_IDENT_ESC,
    REGEX, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS,
};

#[inline]
unsafe fn glue_anchor(src: *const u8, st: *const u64, qi: usize) -> usize {
    let mut a = qi;
    loop {
        let q = bm_prev1(st, a);
        if q < 0 {
            break;
        }
        if !is_glue_join(*src.add(q as usize)) {
            break;
        }
        a = q as usize;
    }
    a
}
#[inline]
pub(super) unsafe fn prop_name(src: *const u8, pos: usize) -> bool {
    pos > 0 && *src.add(pos - 1) == b'.' && (pos < 2 || *src.add(pos - 2) != b'.')
}
enum RunEnd {
    Seg(usize, usize, bool),
    Blank(bool),
}

unsafe fn word_run_end(src: *const u8, s: usize, e: usize) -> RunEnd {
    let mut i = s;
    let mut seg: Option<(usize, usize)> = None;
    let mut nl_after = false;
    let mut any_nl = false;
    while i < e {
        let b = *src.add(i);
        let mut trivia: Option<(bool, usize)> = None;
        if b >= 0x80 {
            let b1 = *src.add(i + 1);
            let b2 = *src.add(i + 2);
            trivia = match b {
                0xc2 if b1 == 0xa0 => Some((false, 2)),
                0xe1 if b1 == 0x9a && b2 == 0x80 => Some((false, 3)),
                0xe2 if b1 == 0x80 && ((0x80..=0x8a).contains(&b2) || b2 == 0xaf) => {
                    Some((false, 3))
                }
                0xe2 if b1 == 0x80 && (b2 == 0xa8 || b2 == 0xa9) => Some((true, 3)),
                0xe2 if b1 == 0x81 && b2 == 0x9f => Some((false, 3)),
                0xe3 if b1 == 0x80 && b2 == 0x80 => Some((false, 3)),
                0xef if b1 == 0xbb && b2 == 0xbf => Some((false, 3)),
                _ => None,
            };
        }
        if let Some((nl, w)) = trivia {
            if nl {
                nl_after = true;
                any_nl = true;
            }
            i += w;
        } else {
            match &mut seg {
                Some((_, se)) if *se == i => *se = i + 1,
                _ => {
                    seg = Some((i, i + 1));
                    nl_after = false;
                }
            }
            i += 1;
        }
    }
    match seg {
        Some((ss, se)) => RunEnd::Seg(ss, se, nl_after),
        None => RunEnd::Blank(any_nl),
    }
}
unsafe fn prev_regex_sim(
    t: &Tables,
    src: *const u8,
    n: usize,
    a: usize,
    p: usize,
    seed_tail: bool,
) -> bool {
    let mut lastk: i32 = -1;
    let mut ls = 0usize;
    let mut le = 0usize;
    let mut prevk: i32 = -1;
    let mut pls = 0usize;
    let mut ple = 0usize;
    let mut pos = a;
    while pos < p {
        let c = *src.add(pos);
        let e: usize;
        let nk: i32;
        if is_digit(c) || (c == b'.' && pos + 1 < n && is_digit(*src.add(pos + 1))) {
            e = scan_number(src, n, pos);
            nk = NUM as i32;
        } else if is_word(c) {
            let mut w = pos;
            while w < n && is_word(*src.add(w)) {
                w += 1;
            }
            e = w;
            nk = IDENT as i32;
        } else if c == b'.' || c == b'+' || c == b'-' || c == b'?' {
            let b1 = *src.add(pos + 1);
            let b2 = *src.add(pos + 2);
            let b3 = *src.add(pos + 3);
            let mut opl: usize = 1;
            for l in (2..=4u32).rev() {
                if pos + l as usize > n {
                    continue;
                }
                let kk = t.op.opmap_lookup(c, b1, b2, b3, l);
                if kk == 0 {
                    continue;
                }
                if kk == OP_QDOT as u32 && pos + 2 < n && is_digit(*src.add(pos + 2)) {
                    continue;
                }
                opl = l as usize;
                break;
            }
            e = pos + opl;
            nk = 1000;
        } else {
            break;
        }
        prevk = lastk;
        pls = ls;
        ple = le;
        ls = pos;
        le = e;
        lastk = nk;
        pos = e;
    }
    if lastk == -1 {
        return true;
    }
    if lastk == NUM as i32 {
        return false;
    }
    if lastk == IDENT as i32 {
        if prop_name(src, ls) {
            return false;
        }
        return match word_run_end(src, ls, le) {
            RunEnd::Seg(ss, se, _) => t.is_regex_keyword(src.add(ss), se - ss),
            RunEnd::Blank(_) => !seed_tail,
        };
    }
    if le - ls == 2
        && *src.add(ls + 1) == *src.add(ls)
        && (*src.add(ls) == b'+' || *src.add(ls) == b'-')
    {
        let tail = if prevk == NUM as i32 {
            true
        } else if prevk == IDENT as i32 {
            match word_run_end(src, pls, ple) {
                RunEnd::Seg(ss, se, false) => {
                    prop_name(src, pls)
                        || prop_name(src, ss)
                        || !t.is_regex_keyword(src.add(ss), se - ss)
                }
                RunEnd::Seg(_, _, true) => false,
                RunEnd::Blank(true) => false,
                RunEnd::Blank(false) => seed_tail,
            }
        } else if prevk == -1 {
            seed_tail
        } else {
            false
        };
        return !tail;
    }
    !(*src.add(le - 1) == b')' || *src.add(le - 1) == b']')
}
unsafe fn anchor_seed_tail(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    anchor: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, anchor);
    if q < 0 {
        return false;
    }
    let mut sp = q as usize;
    {
        let te = bm_next1(st, sp + 1, n);
        if lt_in_range(src, te, anchor) {
            return false;
        }
    }
    for _ in 0..8 {
        let kk = *kind.add(sp);
        if kk >= OP_KIND_BASE {
            let ch = *src.add(sp);
            return ch == b')' || ch == b']';
        }
        if kk == IDENT
            || kk == IDENT_ESC
            || kk == PRIV_IDENT
            || kk == PRIV_IDENT_ESC
            || kk == NUM
            || kk == BIGINT
        {
            let e = bm_next1(st, sp + 1, n);
            match word_run_end(src, sp, e) {
                RunEnd::Seg(_, _, true) => return false,
                RunEnd::Seg(ss, se, false) => {
                    if kk == NUM || kk == BIGINT {
                        return true;
                    }
                    if prop_name(src, sp) || prop_name(src, ss) {
                        return true;
                    }
                    return !t.is_regex_keyword(src.add(ss), se - ss);
                }
                RunEnd::Blank(true) => return false,
                RunEnd::Blank(false) => {
                    let q2 = bm_prev_sig(st, kind, sp);
                    if q2 < 0 {
                        return false;
                    }
                    let t2 = bm_next1(st, q2 as usize + 1, n);
                    if lt_in_range(src, t2, sp) {
                        return false;
                    }
                    sp = q2 as usize;
                    continue;
                }
            }
        }
        return matches!(kk, STR | REGEX | TMPL_NOSUB | TMPL_TAIL | JEND);
    }
    false
}
/// Distance cap (in token starts) for the backward delimiter matches below;
/// past it we fall back to the safe legacy "`}` means regex" answer. Only
/// pathological input gets near it.
const BRACE_MATCH_CAP: u32 = 1024;
/// Previous significant token start before `pos` (skipping trivia), or -1 at
/// start of input.
#[inline]
pub(super) unsafe fn bm_prev_sig(st: *const u64, kind: *const u8, pos: usize) -> i64 {
    let mut q = bm_prev1(st, pos);
    while q >= 0 {
        let k = *kind.add(q as usize);
        if k == WS || k == LCOM || k == BCOM || k == HASHBANG {
            q = bm_prev1(st, q as usize);
            continue;
        }
        break;
    }
    q
}
/// Punctuators after which only an expression can begin: a `{` here is an
/// object literal, a `function` a function expression. `>` is excluded
/// because `=>` ends in it (an arrow's `{}` is a block body); `=` is safe
/// because every operator ending in `=` lands on the `=`.
#[inline(always)]
fn is_operand_punct(ch: u8) -> bool {
    matches!(
        ch,
        b'(' | b'['
            | b','
            | b'='
            | b'?'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'&'
            | b'|'
            | b'^'
            | b'!'
            | b'~'
            | b'<'
    )
}
pub(super) unsafe fn lt_in_range(src: *const u8, a: usize, b: usize) -> bool {
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
/// Is the token at `pos` in operand (expression-only) position? Strict
/// whitelist; anything else returns false. Deliberately not `prev_is_regex`:
/// a regex may follow `;`/`:`/`else`, but a `{` there is a block, so reusing
/// it would be unsound. Complement of acorn's `braceIsBlock`.
#[inline]
pub(super) unsafe fn operand_position(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
    ts: bool,
    depth: u32,
) -> bool {
    operand_position_at(t, src, st, kind, n, pos, ts, depth, 0)
}
unsafe fn operand_position_at(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
    ts: bool,
    depth: u32,
    hops: u32,
) -> bool {
    if hops > 8 || depth > 8 {
        return false;
    }
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false; // start of input => statement position
    }
    let p = q as usize;
    if *kind.add(p) < OP_KIND_BASE {
        let k = *kind.add(p);
        if k == TMPL_HEAD || k == TMPL_MIDDLE {
            return true;
        }
        if k == IDENT && !prop_name(src, p) {
            if ident_is(src, p, b"new")
                || ident_is(src, p, b"typeof")
                || ident_is(src, p, b"void")
                || ident_is(src, p, b"delete")
                || ident_is(src, p, b"in")
                || ident_is(src, p, b"instanceof")
                || ident_is(src, p, b"case")
            {
                return true;
            }
            if ident_is(src, p, b"return")
                || ident_is(src, p, b"throw")
                || ident_is(src, p, b"yield")
                || ident_is(src, p, b"await")
            {
                let e = bm_next1(st, p + 1, n);
                return !lt_in_range(src, e, pos);
            }
            if ident_is(src, p, b"async") && ident_is(src, pos, b"function") {
                let e = bm_next1(st, p + 1, n);
                if !lt_in_range(src, e, pos) {
                    return operand_position_at(t, src, st, kind, n, p, ts, depth, hops + 1);
                }
                return false;
            }
        }
        if let Some(at) = decorator_start(src, st, kind, p) {
            return operand_position_at(t, src, st, kind, n, at, ts, depth, hops + 1);
        }
        return false;
    }
    let ch = *src.add(p);
    if is_operand_punct(ch) {
        return true;
    }
    if ch == b':' {
        return colon_marks_value(t, src, st, kind, n, p, ts, depth);
    }
    if ch == b')' {
        if let Some(at) = decorator_start(src, st, kind, p) {
            return operand_position_at(t, src, st, kind, n, at, ts, depth, hops + 1);
        }
    }
    false
}
unsafe fn colon_marks_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    ts: bool,
    depth: u32,
) -> bool {
    if depth > 8 {
        return false;
    }
    let mut debt: u32 = 0;
    let mut steps: u32 = 0;
    let mut q = bm_prev_sig(st, kind, colon);
    while q >= 0 {
        steps += 1;
        if steps > BRACE_MATCH_CAP {
            return false;
        }
        let w = q as usize;
        if *kind.add(w) >= OP_KIND_BASE {
            match *src.add(w) {
                c @ (b')' | b']' | b'}') => {
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
                        None => return false,
                    }
                }
                b'{' => {
                    return brace_opens_value(t, src, st, kind, n, w, ts, depth + 1);
                }
                b'(' | b'[' | b';' => return false,
                b':' => debt += 1,
                b'?' => {
                    if *src.add(w + 1) != b'?'
                        && *src.add(w + 1) != b'.'
                        && (w == 0 || *src.add(w - 1) != b'?')
                    {
                        if debt == 0 {
                            return true;
                        }
                        debt -= 1;
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}
unsafe fn decorator_start(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    p: usize,
) -> Option<usize> {
    let mut q = p as i64;
    let mut steps: u32 = 0;
    while q >= 0 {
        steps += 1;
        if steps > 32 {
            return None;
        }
        let w = q as usize;
        let kk = *kind.add(w);
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            if c == b'@' {
                return Some(w);
            }
            if c == b')' {
                let lp = match_delim_back(src, st, kind, w, b'(', b')')?;
                q = bm_prev_sig(st, kind, lp);
                continue;
            }
            if c != b'.' {
                return None;
            }
        } else if kk != IDENT {
            return None;
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}
/// Does the identifier at `pos` equal exactly `kw`? The following-byte check
/// rejects longer identifiers (the source pad makes it safe at EOF).
#[inline]
pub(super) unsafe fn ident_is(src: *const u8, pos: usize, kw: &[u8]) -> bool {
    let mut i = 0;
    while i < kw.len() {
        if *src.add(pos + i) != kw[i] {
            return false;
        }
        i += 1;
    }
    !is_word(*src.add(pos + kw.len()))
}
enum AngleMatch {
    Found(usize),
    NotType,
    Unknown,
}
unsafe fn angle_match_back(
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
        if steps > 128 {
            return AngleMatch::Unknown;
        }
        let w = q as usize;
        let kk = *kind.add(w);
        if tmpl > 0 {
            if kk == TMPL_TAIL {
                tmpl += 1;
            } else if kk == TMPL_HEAD {
                tmpl -= 1;
            }
            q = bm_prev_sig(st, kind, w);
            continue;
        }
        if kk == TMPL_TAIL {
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
        } else if !matches!(kk, IDENT | IDENT_ESC | NUM | BIGINT | STR | TMPL_NOSUB) {
            if kk == TMPL_HEAD || kk == TMPL_MIDDLE {
                return AngleMatch::Unknown;
            }
            return AngleMatch::NotType;
        }
        q = bm_prev_sig(st, kind, w);
    }
    AngleMatch::NotType
}
unsafe fn chain_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> Option<usize> {
    let mut head = from;
    let mut steps: u32 = 0;
    while prop_name(src, head) {
        steps += 1;
        if steps > 32 {
            return None;
        }
        let d = bm_prev_sig(st, kind, head);
        if d < 0 {
            return None;
        }
        let o = bm_prev_sig(st, kind, d as usize);
        if o < 0 || *kind.add(o as usize) != IDENT {
            return None;
        }
        head = o as usize;
    }
    Some(head)
}
/// Match the close punctuator at `from` back to its opener, counting only
/// punctuator delimiters — template-closing `}`s and cleared literal
/// interiors are invisible. None if unbalanced or past the cap.
#[inline]
pub(super) unsafe fn match_delim_back(
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
            return None;
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
#[inline]
unsafe fn brace_opens_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
) -> bool {
    if depth > 8 {
        return false;
    }
    if operand_position(t, src, st, kind, n, brace, ts, depth) {
        return true;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false; // start of input => block
    }
    let p = q as usize;
    if *kind.add(p) >= OP_KIND_BASE {
        let ch = *src.add(p);
        if ch == b'>' {
            if p > 0 && *src.add(p - 1) == b'=' {
                return false;
            }
            if !ts {
                return true;
            }
            return ts_gt_brace_value(t, src, st, kind, n, p, depth);
        }
        if ch == b')' {
            if let Some(lp) = match_delim_back(src, st, kind, p, b'(', b')') {
                let w = bm_prev_sig(st, kind, lp);
                if w >= 0 {
                    let wp = w as usize;
                    if *kind.add(wp) < OP_KIND_BASE
                        && ident_is(src, wp, b"function")
                        && operand_position(t, src, st, kind, n, wp, ts, depth)
                    {
                        return true;
                    }
                }
            }
        }
    }
    if ts
        && *kind.add(p) == IDENT
        && !prop_name(src, p)
        && (ident_is(src, p, b"as") || ident_is(src, p, b"satisfies"))
        && tail_or_brace_before(t, src, st, kind, n, p)
    {
        return true;
    }
    class_brace_is_value(t, src, st, kind, n, brace, ts, depth)
}
const CLASS_WALK_CAP: u32 = 64;
unsafe fn class_brace_is_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
) -> bool {
    class_walk_from(t, src, st, kind, n, bm_prev_sig(st, kind, brace), ts, depth)
}
unsafe fn class_walk_from(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    start: i64,
    ts: bool,
    depth: u32,
) -> bool {
    let mut q = start;
    let mut steps: u32 = 0;
    let mut crossed_obj = false;
    let mut pending_comma = false;
    while q >= 0 {
        steps += 1;
        if steps > CLASS_WALK_CAP {
            return false;
        }
        let w = q as usize;
        let kk = *kind.add(w);
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            if c == b')' || c == b']' {
                let open = if c == b')' { b'(' } else { b'[' };
                match match_delim_back(src, st, kind, w, open, c) {
                    Some(op) => {
                        q = bm_prev_sig(st, kind, op);
                        continue;
                    }
                    None => return false,
                }
            }
            if c == b'}' {
                if crossed_obj {
                    return false;
                }
                let Some(ob) = match_delim_back(src, st, kind, w, b'{', b'}') else {
                    return false;
                };
                let nx = bm_prev_sig(st, kind, ob);
                if nx < 0 {
                    return false;
                }
                let np = nx as usize;
                if *kind.add(np) != IDENT || prop_name(src, np) || !ident_is(src, np, b"extends") {
                    return false;
                }
                crossed_obj = true;
                q = nx;
                continue;
            }
            if ts && c == b',' {
                pending_comma = true;
                q = bm_prev_sig(st, kind, w);
                continue;
            }
            if c == b':' {
                return colon_return_type_value(t, src, st, kind, n, w, ts, depth)
                    || colon_marks_value(t, src, st, kind, n, w, ts, depth);
            }
            if ts && c == b'>' && !(w > 0 && *src.add(w - 1) == b'=') {
                match angle_match_back(src, st, kind, w) {
                    AngleMatch::Found(lt) => {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                    _ => return false,
                }
            }
            if c != b'.' && c != b'?' {
                return false;
            }
        } else if kk == IDENT {
            if !prop_name(src, w) {
                if ident_is(src, w, b"class") {
                    if pending_comma {
                        return false;
                    }
                    return operand_position(t, src, st, kind, n, w, ts, depth);
                }
                if ts && ident_is(src, w, b"implements") {
                    pending_comma = false;
                }
            }
        } else if !matches!(kk, NUM | BIGINT | STR | REGEX | TMPL_NOSUB) {
            return false;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}
unsafe fn of_is_forof_keyword(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    qi: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, qi);
    if q < 0 {
        return false;
    }
    let tp = q as usize;
    let tk = *kind.add(tp);
    if tk == IDENT {
        let e = bm_next1(st, tp + 1, n);
        if !prop_name(src, tp) && t.is_regex_keyword(src.add(tp), e - tp) {
            return false;
        }
    } else if tk >= OP_KIND_BASE {
        let c = *src.add(tp);
        if c != b']' && c != b'}' && c != b')' {
            return false;
        }
    } else {
        return false;
    }
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let mut steps: u32 = 0;
    let mut w = q;
    while w >= 0 {
        steps += 1;
        if steps > BRACE_MATCH_CAP {
            return false;
        }
        let p = w as usize;
        let kk = *kind.add(p);
        if kk >= OP_KIND_BASE {
            match *src.add(p) {
                b')' => par += 1,
                b']' => brk += 1,
                b'}' => brc += 1,
                b'(' => {
                    if par == 0 && brk == 0 && brc == 0 {
                        let h = bm_prev_sig(st, kind, p);
                        if h < 0 || *kind.add(h as usize) != IDENT {
                            return false;
                        }
                        let mut hp = h as usize;
                        if ident_is(src, hp, b"await") {
                            let h2 = bm_prev_sig(st, kind, hp);
                            if h2 < 0 || *kind.add(h2 as usize) != IDENT {
                                return false;
                            }
                            hp = h2 as usize;
                        }
                        return !prop_name(src, hp) && ident_is(src, hp, b"for");
                    }
                    par -= 1;
                }
                b'[' => {
                    if brk == 0 && par == 0 && brc == 0 {
                        return false;
                    }
                    brk -= 1;
                }
                b'{' => {
                    if brc == 0 && par == 0 && brk == 0 {
                        return false;
                    }
                    brc -= 1;
                }
                _ => {}
            }
        } else if kk == IDENT
            && par == 0
            && brk == 0
            && brc == 0
            && !prop_name(src, p)
            && ident_is(src, p, b"of")
        {
            let b = bm_prev_sig(st, kind, p);
            if b >= 0 {
                let bp = b as usize;
                let bk = *kind.add(bp);
                let tail = if bk == IDENT {
                    let be = bm_next1(st, bp + 1, n);
                    prop_name(src, bp) || !t.is_regex_keyword(src.add(bp), be - bp)
                } else {
                    bk >= OP_KIND_BASE && matches!(*src.add(bp), b']' | b'}' | b')')
                };
                if tail {
                    return false;
                }
            }
        }
        w = bm_prev_sig(st, kind, p);
    }
    false
}
unsafe fn colon_return_type_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    ts: bool,
    depth: u32,
) -> bool {
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 || *kind.add(q as usize) < OP_KIND_BASE || *src.add(q as usize) != b')' {
        return false;
    }
    let Some(lp) = match_delim_back(src, st, kind, q as usize, b'(', b')') else {
        return false;
    };
    let w = bm_prev_sig(st, kind, lp);
    if w < 0 {
        return false;
    }
    let wp = w as usize;
    *kind.add(wp) < OP_KIND_BASE
        && !prop_name(src, wp)
        && ident_is(src, wp, b"function")
        && operand_position(t, src, st, kind, n, wp, ts, depth)
}
pub(super) unsafe fn tail_before(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
) -> bool {
    let s = bm_prev_sig(st, kind, pos);
    if s < 0 {
        return false;
    }
    let sp = s as usize;
    let sk = *kind.add(sp);
    if sk >= OP_KIND_BASE {
        return matches!(*src.add(sp), b')' | b']');
    }
    if sk == IDENT {
        let e = bm_next1(st, sp + 1, n);
        return prop_name(src, sp) || !t.is_regex_keyword(src.add(sp), e - sp);
    }
    matches!(sk, NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL | REGEX | PRIV_IDENT)
}
unsafe fn tail_or_brace_before(
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
    let s = bm_prev_sig(st, kind, pos);
    s >= 0 && *kind.add(s as usize) >= OP_KIND_BASE && *src.add(s as usize) == b'}'
}
unsafe fn as_gated_type_ref(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 || *kind.add(b as usize) != IDENT {
        return false;
    }
    let Some(head) = chain_head(src, st, kind, b as usize) else {
        return false;
    };
    let a = bm_prev_sig(st, kind, head);
    if a < 0 {
        return false;
    }
    let ap = a as usize;
    if *kind.add(ap) != IDENT
        || prop_name(src, ap)
        || !(ident_is(src, ap, b"as") || ident_is(src, ap, b"satisfies"))
    {
        return false;
    }
    tail_or_brace_before(t, src, st, kind, n, ap)
}
unsafe fn ts_gt_brace_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    gt: usize,
    depth: u32,
) -> bool {
    let lt = match angle_match_back(src, st, kind, gt) {
        AngleMatch::Found(lt) => lt,
        AngleMatch::NotType => return true,
        AngleMatch::Unknown => return false,
    };
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 {
        return true;
    }
    let bp = b as usize;
    if *kind.add(bp) != IDENT {
        return true;
    }
    let Some(head) = chain_head(src, st, kind, bp) else {
        return false;
    };
    let tq = bm_prev_sig(st, kind, head);
    if tq < 0 {
        return true;
    }
    let tp = tq as usize;
    let tk = *kind.add(tp);
    if tk == IDENT && !prop_name(src, tp) {
        if ident_is(src, tp, b"class") {
            return operand_position(t, src, st, kind, n, tp, true, depth);
        }
        if ident_is(src, tp, b"interface") {
            return false;
        }
        if ident_is(src, tp, b"extends") || ident_is(src, tp, b"implements") {
            return class_walk_from(t, src, st, kind, n, tq, true, depth);
        }
        let e = bm_next1(st, tp + 1, n);
        if t.is_regex_keyword(src.add(tp), e - tp) {
            return true;
        }
        return false;
    }
    if tk >= OP_KIND_BASE && *src.add(tp) == b':' {
        return colon_return_type_value(t, src, st, kind, n, tp, true, depth);
    }
    true
}
unsafe fn paren_close_is_regex(src: *const u8, st: *const u64, kind: *const u8, qi: usize) -> bool {
    let Some(lp) = match_delim_back(src, st, kind, qi, b'(', b')') else {
        return false;
    };
    let q = bm_prev_sig(st, kind, lp);
    if q < 0 {
        return false;
    }
    let mut w = q as usize;
    if *kind.add(w) != IDENT {
        return false;
    }
    if ident_is(src, w, b"await") {
        let q2 = bm_prev_sig(st, kind, w);
        if q2 < 0 || *kind.add(q2 as usize) != IDENT {
            return false;
        }
        w = q2 as usize;
        return !prop_name(src, w) && ident_is(src, w, b"for");
    }
    !prop_name(src, w)
        && (ident_is(src, w, b"if")
            || ident_is(src, w, b"while")
            || ident_is(src, w, b"for")
            || ident_is(src, w, b"with"))
}
/// A `/` right after `}`: regex if the `}` closed a block, division if it
/// closed a value. Cold; any uncertainty falls back to regex (the legacy
/// answer), which can never introduce a stream-swallowing regex.
#[inline]
unsafe fn brace_close_is_regex(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    qi: usize,
    ts: bool,
) -> bool {
    match match_delim_back(src, st, kind, qi, b'{', b'}') {
        Some(brace) => !brace_opens_value(t, src, st, kind, n, brace, ts, 0),
        None => true, // unbalanced / cap => fallback regex
    }
}

pub(super) unsafe fn prev_is_regex(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    word: *const u64,
    digit: *const u64,
    n: usize,
    p: usize,
    ts: bool,
    module: bool,
) -> bool {
    let mut q = bm_prev1(st, p);
    while q >= 0 {
        let qi = q as usize;
        let k = *kind.add(qi);
        if k == WS || k == LCOM || k == BCOM || k == HASHBANG {
            q = bm_prev1(st, qi);
            continue;
        }
        if k == STR
            || k == REGEX
            || k == TMPL_NOSUB
            || k == TMPL_TAIL
            || k == PRIV_IDENT
            || k == IDENT_ESC
            || k == PRIV_IDENT_ESC
            || k == JEND
            || k == JSX_LT
        {
            return false;
        }
        if k == TMPL_HEAD || k == TMPL_MIDDLE {
            return true;
        }
        if k == NUM {
            let we = bm_next0(word, qi, n);
            let de = bm_next0(digit, qi, n);
            if de >= we {
                return false;
            }
            let a = glue_anchor(src, st, qi);
            return prev_regex_sim(t, src, n, a, p, anchor_seed_tail(t, src, st, kind, n, a));
        }
        if k == IDENT {
            if prop_name(src, qi) {
                return false;
            }
            let e = bm_next1(st, qi + 1, n);
            let (ws, we) = match word_run_end(src, qi, e) {
                RunEnd::Seg(ss, se, _) => (ss, se),
                RunEnd::Blank(_) => {
                    q = bm_prev1(st, qi);
                    continue;
                }
            };
            if t.is_regex_keyword(src.add(ws), we - ws) {
                if !module && we - ws == 5 {
                    if ident_is(src, ws, b"yield") {
                        return super::replay::replay_is_keyword(
                            t, src, st, kind, n, qi, ts, false,
                        );
                    }
                    if ident_is(src, ws, b"await") {
                        return super::replay::replay_is_keyword(t, src, st, kind, n, qi, ts, true);
                    }
                }
                return true;
            }
            if we - ws == 2 && *src.add(ws) == b'o' && *src.add(ws + 1) == b'f' {
                return of_is_forof_keyword(t, src, st, kind, n, qi);
            }
            return false;
        }
        if k >= OP_KIND_BASE {
            let ch = *src.add(qi);
            // TS postfix non-null `!`: `x! / 2` is division, not a regex —
            // look through the `!`, unless a newline sits before it (ASI
            // makes it a prefix `!/re/`).
            if ts && ch == b'!' {
                let mut j = qi;
                while j > 0
                    && is_ws(*src.add(j - 1))
                    && *src.add(j - 1) != b'\n'
                    && *src.add(j - 1) != b'\r'
                {
                    j -= 1;
                }
                if j > 0 && (*src.add(j - 1) == b'\n' || *src.add(j - 1) == b'\r') {
                    return true; // prefix `!`
                }
                q = bm_prev1(st, qi); // postfix: keep walking
                continue;
            }
            if ch == b'.' || ch == b'+' || ch == b'-' {
                let a = glue_anchor(src, st, qi);
                return prev_regex_sim(t, src, n, a, p, anchor_seed_tail(t, src, st, kind, n, a));
            }
            // `}` closed either a block (regex follows) or a value (division).
            if ch == b'}' {
                return brace_close_is_regex(t, src, st, kind, n, qi, ts);
            }
            if ch == b')' {
                return paren_close_is_regex(src, st, kind, qi);
            }
            if ts && ch == b'>' && !(qi > 0 && *src.add(qi - 1) == b'=') {
                if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, qi) {
                    if as_gated_type_ref(t, src, st, kind, n, lt) {
                        return false;
                    }
                }
                return true;
            }
            return ch != b']';
        }
        return true;
    }
    true
}
#[cfg(test)]
mod tests {
    use crate::options::default_options;
    use crate::token::token_kind as k;
    use crate::{Lexer, PAD};

    fn kinds_of(code: &str, ts: bool, jsx: bool) -> Vec<u8> {
        let mut buf = code.as_bytes().to_vec();
        let n = buf.len();
        buf.resize(n + PAD, 0);
        let mut opts = default_options();
        opts.ts = ts;
        opts.jsx = jsx;
        let mut lx = Lexer::new();
        let count = lx.lex(&buf, n, opts);
        lx.kinds[..count - 1].iter().copied().filter(|&kk| !crate::is_trivia(kk)).collect()
    }

    #[track_caller]
    fn regex(code: &str, ts: bool) {
        let ks = kinds_of(code, ts, false);
        assert!(ks.contains(&k::REGEXP), "expected regex in {code:?}: kinds {ks:?}");
    }

    #[track_caller]
    fn division(code: &str, ts: bool) {
        let ks = kinds_of(code, ts, false);
        assert!(!ks.contains(&k::REGEXP), "expected division in {code:?}: kinds {ks:?}");
        assert!(ks.contains(&k::SLASH), "expected a `/` in {code:?}: kinds {ks:?}");
    }

    #[track_caller]
    fn assert_regex(code: &str) {
        regex(code, false);
    }

    #[track_caller]
    fn assert_division(code: &str) {
        division(code, false);
    }

    #[test]
    fn debugger_precedes_regex() {
        regex("debugger\n/re/.test(x);", false);
        division("x.debugger / 2;", false);
    }

    #[test]
    fn for_of_head_precedes_regex() {
        regex("for (x of /re/) ;", false);
        regex("for ([a, b] of /re/) ;", false);
        regex("for ({a} of /re/) ;", false);
        regex("for (const x of /re/) ;", false);
        regex("for await (x of /re/) ;", false);
        regex("for ([a, of] of /re/) ;", false);
        division("var of = 1; of / 2;", false);
        division("instance/of/g;", false);
        division("for (of / 2;;) ;", false);
        division("for (of of of / 2) ;", false);
        division("f(x, of / 2);", false);
    }

    #[test]
    fn unicode_ident_postfix_is_division() {
        division("\u{53d8}\u{91cf}++ / b;", false);
        division("\u{53d8}\u{91cf} ++ / b;", false);
        regex("a\u{2028}++/re/.lastIndex;", false);
        regex("a\u{2029}++/re/.lastIndex;", false);
        division("a\u{00a0}++ / b;", false);
        division("a\u{200a}++ / b;", false);
        division("a\u{feff}++ / b;", false);
        division("a\u{200d}b++ / 2;", false);
    }

    #[test]
    fn operand_keywords_before_value_braces() {
        division("x = typeof {} / 2;", false);
        division("x = void {} / 2;", false);
        division("f(new class {} / 2);", false);
        division("return function(){} / 2;", false);
        division("throw {} / 2;", false);
        division("if (k in {} / 2) ;", false);
        division("switch (x) { case class {} / 2: break; }", false);
        regex("return\nclass C {} /re/.test(x);", false);
        regex("export default class C {} /re/.test(x);", false);
    }

    #[test]
    fn object_literal_heritage() {
        division("(class C extends {valueOf(){}} {} / 2);", false);
        division("(class extends {a:1}.constructor {} / 2);", false);
        regex("x = class {}\n{} /re/.test(s);", false);
        regex("x = class C extends {a:1} {}\n{} /re/.test(s);", false);
    }

    #[test]
    fn ts_implements_heritage() {
        division("(class C implements I, J {} / 2);", true);
        division("(class C extends B implements I, J {} / 2);", true);
        regex("class C implements I, J {} /re/.test(x);", true);
        regex("x = class A {}, y\n{} /re/.test(s);", true);
    }

    #[test]
    fn decorated_class_expression() {
        division("x = @dec class {} / 2;", true);
        division("x = @ns.dec() class {} / 2;", true);
        division("f(@a @b(1) class {} / 2);", true);
        regex("@dec class C {} /re/.test(x);", true);
    }

    #[test]
    fn ts_angle_before_brace() {
        division("(class C<T> {} / 2);", true);
        division("(class C extends B<T> {} / 2);", true);
        division("(class C<T extends {a: 1}> {} / 2);", true);
        division("(class C<T = X> {} / 2);", true);
        division("x = f < T > {} / re / g;", true);
        division("x = a < b + c > {} / 2;", true);
        division("(a > {} / 2);", true);
        regex("class C<T> {} /re/.test(x);", true);
        regex("interface I<T> {} /re/.test(x);", true);
        regex("declare class C<T> {} /re/.test(x);", true);
        regex("class C extends B<T> {}\n/re/.test(x);", true);
    }

    #[test]
    fn brace_tail_before_as() {
        division("let v = {a: 1} as T / y;", true);
        division("let v = {} as A<B> / y;", true);
        division("let v = {} satisfies T / y;", true);
        division("let v = ({} as T) / y;", true);
    }

    #[test]
    fn template_literal_types_cross() {
        division("(class C<T extends `a${X}`> {} / 2);", true);
        division("(class C<T extends `a${`b${Y}`}`> {} / 2);", true);
        division("let v = x as A<`a${B}`> / y;", true);
        regex("class C<T extends `a${X}`> {} /re/.test(x);", true);
    }

    #[test]
    fn hidden_trivia_segments() {
        division("a\u{00a0}++ / b;", false);
        division("a\u{200a}++ / b;", false);
        division("a\u{feff}++ / b;", false);
        division("x = a \u{00a0}++ / b;", false);
        regex("return\u{00a0}++/re/.lastIndex;", false);
        division("x.\u{00a0}return++ / 2;", false);
        regex("a\u{2028}++/re/.lastIndex;", false);
        regex("f(a,\u{00a0}++/re/.lastIndex);", false);
    }

    #[test]
    fn ts_as_type_brace() {
        division("let v = x as {} / y;", true);
        division("let v = x as {a: 1} / y;", true);
        division("let v = f() as {} / y;", true);
        division("let v = x satisfies {} / y;", true);
        regex("f();\nas: {} /re/.test(s);", true);
    }

    #[test]
    fn ts_as_angle_form() {
        division("let v = x as A<B> / y;", true);
        division("let v = x as a.b.C<D> / y;", true);
        division("let v = f() as A<B<C>> / y;", true);
        division("let v = x satisfies A<B> / y;", true);
        regex("let q = a > /re/.source.length;", true);
        regex("let q = (x < y) > /re/.source;", true);
        regex("g(x => /re/.test(x));", true);
    }

    #[test]
    fn ts_return_type_function_expr() {
        division("x = function(): T {} / 2;", true);
        division("x = function(): a.b.T {} / 2;", true);
        division("x = function(): Map<A, B> {} / 2;", true);
        division("x = async function(): T {} / 2;", true);
        regex("function f(): T {} /re/.test(x);", true);
        regex("function f(): Map<A, B> {} /re/.test(x);", true);
        regex("L: {} /re/.test(x);", true);
        regex("switch (x) { case f(y): {} /re/.test(s); }", true);
    }

    #[test]
    fn async_function_expression_value() {
        division("x = async function(){} / 2;", false);
        regex("async function f(){} /re/.test(x);", false);
    }

    #[test]
    fn slash_dense_chains() {
        let count = |code: &str, want_re: usize, want_slash: usize| {
            let ks = kinds_of(code, false, false);
            let re = ks.iter().filter(|&&kk| kk == k::REGEXP).count();
            let sl = ks.iter().filter(|&&kk| kk == k::SLASH).count();
            assert_eq!(
                (re, sl),
                (want_re, want_slash),
                "{code:?}: kinds {ks:?} (regexps, slashes)"
            );
        };
        count("x = /a/g / /b/g;", 2, 1);
        count("x = /a/ / /b/;", 2, 1);
        count("x = a / /b/ / c;", 1, 2);
        count("x = /a/ / b / c;", 1, 2);
        count("x = /a/.lastIndex / 2;", 1, 1);
        count("x /= /re/.source.length;", 1, 0);
        count("x = /a/ instanceof /b/ ? 1 : 2;", 2, 0);
        count("x = /a/ in /b/ ? 1 : 2;", 2, 0);
        count("f(/a/, /b/, a / b);", 2, 1);
        count("x = [/a/, /b/][i] / 2;", 2, 1);
        count("x = `${/a/.source}` / 2;", 1, 1);
    }

    #[test]
    fn colon_member_and_ternary_values() {
        division("({a: function(){} / 2, b: 1});", false);
        division("({a: {} / 2});", false);
        division("({a: {b: {} / 2}});", false);
        division("({a: class {} / 2});", false);
        division("x = c ? y : {} / 2;", false);
        division("x = c ? d ? a : b : {} / 2;", false);
        regex("L: {} /re/.test(x);", false);
        regex("{ L: function f(){} /re/ }", false);
        regex("switch (x) { case a: function f(){} /re/ }", false);
        regex("switch (x) { case f(y), z: {} /re/.test(s); }", false);
        regex("c ? a : b\nL: {} /re/.test(s);", false);
        regex("if (c) {} L: {} /re/.test(s);", false);
    }

    #[test]
    fn postfix_incdec_then_slash_is_division() {
        assert_division("a++ / b;");
        assert_division("a-- / b;");
        assert_division("a ++ / b;");
        assert_division("x[i]++ / n;");
        assert_division("f(x)++ / n;");
        assert_division("a.return++ / 2;");
        assert_division("obj.#f++ / 2;");
        assert_division("a = b++/c/g;");
        assert_division("a++\n/ b / c;");
    }

    #[test]
    fn prefix_incdec_then_slash_is_regex() {
        assert_regex("++/re/.lastIndex;");
        assert_regex("x = ++/re/.lastIndex;");
        assert_regex("(a, ++/re/.lastIndex);");
        assert_regex("f(++/re/.lastIndex);");
        assert_regex("return++/re/.lastIndex;");
        assert_regex("a + ++/re/.lastIndex;");
        assert_regex("a ** ++/re/.lastIndex;");
    }

    #[test]
    fn line_terminator_forces_prefix() {
        assert_regex("a\n++/re/.lastIndex;");
        assert_regex("a\r\n++/re/.lastIndex;");
        assert_regex("a\u{2028}++/re/.lastIndex;");
        assert_regex("a\u{2029}++/re/.lastIndex;");
        assert_regex("a /* x\ny */ ++/re/.lastIndex;");
        assert_division("a /* xy */ ++ / b;");
    }

    #[test]
    fn incdec_runs_keep_maximal_munch() {
        assert_regex("a+++/re/;");
        assert_regex("a---/re/;");
        assert_division("a++/b/;");
    }

    #[test]
    fn default_and_extends_precede_regex() {
        assert_regex("export default /^x$/;");
        assert_regex("export default /re/.source;");
        assert_regex("class C extends /re/.constructor {}");
        assert_division("x.default / 2;");
        assert_division("x.extends / 2;");
    }

    #[test]
    fn statement_head_paren_then_regex() {
        assert_regex("if (x) /re/.test(y);");
        assert_regex("if (f(x)) /re/.test(y);");
        assert_regex("while (x) /re/.exec(y);");
        assert_regex("for (;;) /re/.test(x);");
        assert_regex("with (o) /re/.test(x);");
        assert_regex("for await (x of y) /re/.test(x);");
        assert_regex("do x; while (y) /re/.test(z);");
        assert_regex("if (a) while (b) /re/.test(c);");
    }

    #[test]
    fn value_paren_then_slash_stays_division() {
        assert_division("f(x) / 2;");
        assert_division("(a + b) / 2;");
        assert_division("x.if(a) / 2;");
        assert_division("x?.while(a) / 2;");
        assert_division("if (a) (b) / c / d;");
        assert_division("await (x) / 2;");
    }

    #[test]
    fn class_expression_brace_then_slash_is_division() {
        assert_division("(class {} / 2);");
        assert_division("(class C {} / 2);");
        assert_division("(class extends B {} / 2);");
        assert_division("(class C extends B {} / 2);");
        assert_division("(class C extends f(B) {} / 2);");
        assert_division("(class C extends a.b[0] {} / 2);");
        assert_division("x = class {} / 2;");
        assert_division("f(class {m(){}} / 2);");
        assert_division("`${class {} / 2}`;");
    }

    #[test]
    fn class_declaration_brace_then_slash_is_regex() {
        assert_regex("class C {} /re/.test(x);");
        assert_regex("class C extends B {}\n/re/.test(x);");
        assert_regex("{ class C {} } /re/.test(x);");
    }

    #[test]
    fn block_braces_keep_the_regex_answer() {
        assert_regex("{} /re/.test(x);");
        assert_regex(";{} /re/.test(x);");
        assert_regex("L: {} /re/.test(x);");
        assert_regex("if(a){}else{} /'/.test(s);\nconst t='x';");
        assert_regex("x = () => {}\n/re/.test(s);");
    }

    #[test]
    fn value_braces_keep_the_division_answer() {
        assert_division("f({} / 2);");
        assert_division("x = function(){} / 2;");
    }

    #[test]
    fn plain_contexts_unchanged() {
        assert_division("a / b;");
        assert_division("1n / 2;");
        assert_regex("a + /re/g;");
        assert_regex("x = /re/;");
        assert_regex("f(/re/);");
    }

    #[test]
    fn ts_postfix_bang_unchanged() {
        let ks = kinds_of("x! / 2;", true, false);
        assert!(!ks.contains(&k::REGEXP), "x! / 2 must stay division: {ks:?}");
    }

    #[test]
    fn bare_gt_object_rhs_is_division() {
        assert_division("x = f < T > {} / re / g;");
        assert_division("x = a > {} / 2;");
        assert_division("x = a >> {} / 2;");
        assert_division("x = a >>> {} / 2;");
        assert_division("x = a-- > {} / 2;");
        assert_division("x = a >\n{} / 2;");
    }

    #[test]
    fn arrow_block_bodies_still_regex() {
        assert_regex("x = y => {}\n/re/.test(s);");
        assert_regex("x = async () => {}\n/re/.test(s);");
    }

    #[test]
    fn ts_angle_close_resolved() {
        let ks = kinds_of("class C<T> {} /re/.test(x);", true, false);
        assert!(ks.contains(&k::REGEXP), "TS class decl with type params: {ks:?}");
        let ks = kinds_of("x = f < T > {} / re / g;", true, false);
        assert!(!ks.contains(&k::REGEXP), "TS relational re-read must divide: {ks:?}");
    }

    #[test]
    fn unicode_ident_tail_resolved() {
        assert_division("\u{53d8}\u{91cf}++ / b;");
        assert_regex("a\u{2028}++/re/.lastIndex;");
    }

    #[test]
    fn of_trade_resolved() {
        assert_regex("for (x of /re/) ;");
        assert_division("var of = 1; of / 2;");
        assert_division("instance/of/g;");
    }

    #[test]
    fn jsx_operand_positions() {
        let jsx = |code: &str| kinds_of(code, false, true);
        let ks = jsx("export default <App/>;");
        assert!(ks.contains(&k::JSX_LT), "export default <App/> must open JSX: {ks:?}");
        let ks = jsx("if (x) <App/>;");
        assert!(ks.contains(&k::JSX_LT), "if (x) <App/> must open JSX: {ks:?}");
        let ks = jsx("x = a < b;");
        assert!(!ks.contains(&k::JSX_LT), "a < b is a comparison: {ks:?}");
        let ks = jsx("f(x) < y;");
        assert!(!ks.contains(&k::JSX_LT), "f(x) < y is a comparison: {ks:?}");
        let ks = jsx("a++ < b;");
        assert!(!ks.contains(&k::JSX_LT), "a++ < b is a comparison: {ks:?}");
        let ks = jsx("x = a > {} < b;");
        assert!(!ks.contains(&k::JSX_LT), "a > {{}} < b is a comparison chain: {ks:?}");
    }

    #[test]
    fn jsx_replay_oracle() {
        let jsx = |code: &str| kinds_of(code, false, true);
        let ks = jsx("function* items(d) { for (const x of d) yield <li id={x}/>; }");
        assert!(ks.contains(&k::JSX_LT), "yielded JSX element must frame: {ks:?}");
        let ks = jsx("var await = 1, max = 10;\nif (await < max) done();");
        assert!(!ks.contains(&k::JSX_LT), "await < max is a comparison: {ks:?}");
        assert!(ks.contains(&k::LT), "expected a plain `<`: {ks:?}");
        let ks = jsx("async function f() { return await <Spinner/>; }");
        assert!(ks.contains(&k::JSX_LT), "awaited JSX element must frame: {ks:?}");
        let ks =
            jsx("var await = 1, g = 2;\nvar el = <a b={async () => await 1} c={await /2/g}/>;");
        assert!(!ks.contains(&k::REGEXP), "container leak, expected division: {ks:?}");
    }
}
