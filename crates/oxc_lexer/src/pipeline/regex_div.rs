use core::cell::{Cell, RefCell};

use crate::opmap::{OP_KIND_BASE, OP_QDOT};
use crate::tables::{Tables, is_digit, is_glue_join, is_id_start, is_word, is_ws};
use crate::token::{KW_BASE, KW_MAX, TokenKind};

use super::bitmap::{bm_get, bm_next0, bm_next1, bm_prev1};
use super::find::{find_line_terminator, scan_block_comment, scan_number};
use super::{
    BCOM, BIGINT, HASHBANG, IDENT, IDENT_ESC, JEND, JSX_LT, JTEXT, LCOM, NUM, PRIV_IDENT,
    PRIV_IDENT_ESC, REGEX, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS,
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
#[inline(always)]
unsafe fn kind_at(kind: *const u8, w: usize) -> u8 {
    let k = *kind.add(w);
    if k >= KW_BASE && k <= KW_MAX {
        return IDENT;
    }
    if k == IDENT_ESC || k == PRIV_IDENT_ESC {
        return k & !(IDENT_ESC ^ IDENT);
    }
    k
}

enum RunEnd {
    Seg(usize, usize, bool),
    Blank(bool),
}

#[inline]
unsafe fn trivia_at(src: *const u8, i: usize) -> Option<(bool, usize)> {
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
unsafe fn word_run_end(src: *const u8, s: usize, e: usize) -> RunEnd {
    let mut i = s;
    let mut seg: Option<(usize, usize)> = None;
    let mut nl_after = false;
    let mut any_nl = false;
    while i < e {
        let trivia = trivia_at(src, i);
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
            if c >= 0x80 {
                match word_run_end(src, pos, w) {
                    RunEnd::Blank(_) => {
                        pos = w;
                        continue;
                    }
                    RunEnd::Seg(ss, _, _) if is_digit(*src.add(ss)) => {
                        e = scan_number(src, n, ss);
                        nk = NUM as i32;
                    }
                    RunEnd::Seg(..) => {
                        e = w;
                        nk = IDENT as i32;
                    }
                }
            } else {
                e = w;
                nk = IDENT as i32;
            }
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
    if hops > 64 || depth > 64 {
        return false;
    }
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false; // start of input => statement position
    }
    let p = q as usize;
    if kind_at(kind, p) < OP_KIND_BASE {
        let k = kind_at(kind, p);
        if k == TMPL_HEAD || k == TMPL_MIDDLE {
            return true;
        }
        if k == IDENT && !prop_name(src, p) {
            if ident_is(src, p, b"void") {
                return !(ts && void_is_type_position(t, src, st, kind, n, p, depth));
            }
            if ident_is(src, p, b"new")
                || ident_is(src, p, b"typeof")
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
            if ident_is(src, p, b"of") && of_is_forof_keyword(t, src, st, kind, n, p) {
                return true;
            }
            if ident_is(src, p, b"default") && *src.add(pos) == b'{' {
                let e = bm_prev_sig(st, kind, p);
                return e >= 0
                    && kind_at(kind, e as usize) == IDENT
                    && !prop_name(src, e as usize)
                    && ident_is(src, e as usize, b"export");
            }
        }
        if let Some(at) = decorator_start(src, st, kind, p) {
            return operand_position_at(t, src, st, kind, n, at, ts, depth, hops + 1);
        }
        return false;
    }
    let ch = *src.add(p);
    if (ch == b'+' || ch == b'-') && p > 0 && *src.add(p - 1) == ch {
        return !(incdec_is_postfix(src, st, kind, n, p - 1) && lt_in_range(src, p + 1, pos));
    }
    if (ch == b'+' || ch == b'-') && *src.add(p + 1) == ch {
        return !(incdec_is_postfix(src, st, kind, n, p) && lt_in_range(src, p + 2, pos));
    }
    if (ch == b'>' && p > 0 && *src.add(p - 1) == b'=') || (ch == b'=' && *src.add(p + 1) == b'>') {
        return *src.add(pos) != b'{';
    }
    if ch == b'>' {
        if *src.add(pos) == b'{' {
            return false;
        }
        return !(ts
            && lt_in_range(src, p + 1, pos)
            && type_annotation_asi(t, src, st, kind, n, p, depth + 1));
    }
    if ch == b'.' {
        return (*src.add(p + 1) == b'.' && *src.add(p + 2) == b'.')
            || (p >= 2 && *src.add(p - 1) == b'.' && *src.add(p - 2) == b'.');
    }
    if ch == b'{' {
        return jsx_container_or_object_brace(t, src, st, kind, n, p, ts, depth + 1, false);
    }
    if ts && ch == b'!' && *src.add(p + 1) != b'=' && bang_is_postfix(src, st, kind, n, p) {
        return false;
    }
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
unsafe fn jsx_container_or_object_brace(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
    strict: bool,
) -> bool {
    if depth > 64 {
        return false;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k == JTEXT || k == JEND {
        return true;
    }
    if k >= OP_KIND_BASE {
        match *src.add(w) {
            b'>' => {
                return !(w > 0 && *src.add(w - 1) == b'=')
                    && matches!(angle_match_back(src, st, kind, w), AngleMatch::NotType);
            }
            b'}' => {
                return match_delim_back(src, st, kind, w, b'{', b'}').is_some_and(|o| {
                    jsx_container_or_object_brace(t, src, st, kind, n, o, ts, depth + 1, true)
                });
            }
            _ => {}
        }
    }
    !strict && brace_opens_object_literal(t, src, st, kind, n, brace, ts, depth + 1)
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
    if depth > 64 {
        return false;
    }
    let mut debt: u32 = 0;
    let mut q = bm_prev_sig(st, kind, colon);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
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
                    return brace_opens_object_literal(t, src, st, kind, n, w, ts, depth + 1);
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
    while q >= 0 {
        let w = q as usize;
        let kk = kind_at(kind, w);
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
    let after = pos + kw.len();
    !is_word(*src.add(after)) || trivia_at(src, after).is_some()
}
const ANGLE_MATCH_CAP: u32 = 4096;
pub(super) enum AngleMatch {
    Found(usize),
    NotType,
    Unknown,
}
pub(super) unsafe fn angle_match_back(
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
        if kind_at(kind, op) == IDENT {
            head = op;
            continue;
        }
        if kind_at(kind, op) >= OP_KIND_BASE && *src.add(op) == b')' {
            let lp = match_delim_back(src, st, kind, op, b'(', b')')?;
            let im = bm_prev_sig(st, kind, lp);
            if im >= 0
                && kind_at(kind, im as usize) == IDENT
                && ident_is(src, im as usize, b"import")
            {
                return Some(im as usize);
            }
        }
        return None;
    }
    Some(head)
}
#[inline]
unsafe fn as_type_operand(src: *const u8, st: *const u64, kind: *const u8, pos: usize) -> bool {
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    *kind.add(w) == IDENT
        && !prop_name(src, w)
        && (ident_is(src, w, b"as") || ident_is(src, w, b"satisfies"))
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

pub(super) fn memo_new_lex() {
    MEMO_GEN.with(|g| g.set(g.get().wrapping_add(1)));
}

fn delim_slot(c: u8) -> usize {
    match c {
        b'(' | b')' => 0,
        b'[' | b']' => 1,
        _ => 2,
    }
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

/// Match the close punctuator at `from` back to its opener, counting only
/// punctuator delimiters — template-closing `}`s and cleared literal
/// interiors are invisible. Past the cap the answer comes from a per-file
/// closer-to-opener table built once, so it is exact; None if unbalanced.
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
#[inline]
unsafe fn brace_opens_object_literal(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
) -> bool {
    if depth > 64 {
        return false;
    }
    if operand_position(t, src, st, kind, n, brace, ts, depth) {
        return true;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let p = q as usize;
    if kind_at(kind, p) >= OP_KIND_BASE {
        let ch = *src.add(p);
        if ch == b'>' {
            if p > 0 && *src.add(p - 1) == b'=' {
                return false;
            }
            if !ts {
                return true;
            }
            return ts_gt_brace(t, src, st, kind, n, p, depth) == GtBrace::Value;
        }
        return false;
    }
    ts && kind_at(kind, p) == IDENT
        && !prop_name(src, p)
        && (ident_is(src, p, b"as") || ident_is(src, p, b"satisfies"))
        && tail_or_brace_before(t, src, st, kind, n, p)
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
    if depth > 64 {
        return false;
    }
    if brace_opens_object_literal(t, src, st, kind, n, brace, ts, depth) {
        return true;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false; // start of input => block
    }
    let p = q as usize;
    if kind_at(kind, p) >= OP_KIND_BASE {
        let ch = *src.add(p);
        if ts
            && ch == b'>'
            && !(p > 0 && *src.add(p - 1) == b'=')
            && ts_gt_brace(t, src, st, kind, n, p, depth) == GtBrace::Body
        {
            return true;
        }
        if ch == b')' {
            if let Some(lp) = match_delim_back(src, st, kind, p, b'(', b')') {
                if let Some(fk) = function_keyword_before_params(src, st, kind, lp) {
                    if operand_position(t, src, st, kind, n, fk, ts, depth) {
                        return true;
                    }
                }
            }
        }
    }
    if ts {
        if let Some(lp) = return_type_signature_paren(src, st, kind, brace) {
            if let Some(fk) = function_keyword_before_params(src, st, kind, lp) {
                return operand_position(t, src, st, kind, n, fk, ts, depth);
            }
        }
    }
    class_brace_is_value(t, src, st, kind, n, brace, ts, depth)
}
unsafe fn function_keyword_before_params(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lp: usize,
) -> Option<usize> {
    let mut q = bm_prev_sig(st, kind, lp);
    if q < 0 {
        return None;
    }
    let mut w = q as usize;
    if kind_at(kind, w) >= OP_KIND_BASE
        && *src.add(w) == b'>'
        && !(w > 0 && *src.add(w - 1) == b'=')
    {
        let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
            return None;
        };
        q = bm_prev_sig(st, kind, lt);
        if q < 0 {
            return None;
        }
        w = q as usize;
    }
    if kind_at(kind, w) < OP_KIND_BASE && !ident_is(src, w, b"function") {
        q = bm_prev_sig(st, kind, w);
        if q < 0 {
            return None;
        }
        w = q as usize;
    }
    if kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b'*' {
        q = bm_prev_sig(st, kind, w);
        if q < 0 {
            return None;
        }
        w = q as usize;
    }
    if kind_at(kind, w) < OP_KIND_BASE && !prop_name(src, w) && ident_is(src, w, b"function") {
        Some(w)
    } else {
        None
    }
}
unsafe fn bang_is_postfix(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    bang: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, bang);
    if q < 0 {
        return false;
    }
    let v = q as usize;
    let vk = kind_at(kind, v);
    let value = matches!(vk, IDENT | PRIV_IDENT | NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL)
        || (vk >= OP_KIND_BASE && matches!(*src.add(v), b')' | b']' | b'}'));
    value && !lt_in_range(src, bm_next1(st, v + 1, n), bang)
}
unsafe fn incdec_is_postfix(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    first: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, first);
    if q < 0 {
        return false;
    }
    let v = q as usize;
    let vk = kind_at(kind, v);
    let value = matches!(vk, IDENT | IDENT_ESC | NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL)
        || (vk >= OP_KIND_BASE && matches!(*src.add(v), b')' | b']'));
    value && !lt_in_range(src, bm_next1(st, v + 1, n), first)
}
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
    let mut crossed_obj = false;
    let mut pending_comma = false;
    let mut hopped = false;
    while q >= 0 {
        let w = q as usize;
        let kk = kind_at(kind, w);
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            if c == b')' || c == b']' {
                let open = if c == b')' { b'(' } else { b'[' };
                match match_delim_back(src, st, kind, w, open, c) {
                    Some(op) => {
                        hopped = true;
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
                if ts && type_literal_brace_opener(src, st, kind, ob) {
                    q = bm_prev_sig(st, kind, ob);
                    continue;
                }
                let nx = bm_prev_sig(st, kind, ob);
                if nx < 0 {
                    return false;
                }
                let np = nx as usize;
                if kind_at(kind, np) != IDENT
                    || prop_name(src, np)
                    || !ident_is(src, np, b"extends")
                {
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
                if hopped {
                    return false;
                }
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
                if word_is_any(src, w, CLASS_WALK_STOP_WORDS) {
                    return false;
                }
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
    if tk == IDENT || tk == IDENT_ESC {
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
    let mut w = q;
    while w >= 0 {
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
                let tail = if bk == IDENT || bk == IDENT_ESC {
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
    if q < 0 || kind_at(kind, q as usize) < OP_KIND_BASE || *src.add(q as usize) != b')' {
        return false;
    }
    let Some(lp) = match_delim_back(src, st, kind, q as usize, b'(', b')') else {
        return false;
    };
    function_keyword_before_params(src, st, kind, lp)
        .is_some_and(|fk| operand_position(t, src, st, kind, n, fk, ts, depth))
}
pub(super) unsafe fn tail_before(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
) -> bool {
    let mut s = bm_prev_sig(st, kind, pos);
    loop {
        if s < 0 {
            return false;
        }
        let w = s as usize;
        if kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b'!' && *src.add(w + 1) != b'=' {
            s = bm_prev_sig(st, kind, w);
            continue;
        }
        break;
    }
    if s < 0 {
        return false;
    }
    let sp = s as usize;
    let sk = kind_at(kind, sp);
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
    s >= 0 && kind_at(kind, s as usize) >= OP_KIND_BASE && *src.add(s as usize) == b'}'
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum GtBrace {
    Value,
    Body,
    No,
}
unsafe fn ts_gt_brace(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    gt: usize,
    depth: u32,
) -> GtBrace {
    let body = |yes: bool| if yes { GtBrace::Body } else { GtBrace::No };
    let lt = match angle_match_back(src, st, kind, gt) {
        AngleMatch::Found(lt) => lt,
        AngleMatch::NotType => return GtBrace::Value,
        AngleMatch::Unknown => return GtBrace::No,
    };
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 {
        return GtBrace::Value;
    }
    let bp = b as usize;
    if kind_at(kind, bp) != IDENT {
        return GtBrace::Value;
    }
    let Some(head) = chain_head(src, st, kind, bp) else {
        return GtBrace::No;
    };
    let tq = bm_prev_sig(st, kind, head);
    if tq < 0 {
        return GtBrace::Value;
    }
    let tp = tq as usize;
    let tk = kind_at(kind, tp);
    if tk == IDENT && !prop_name(src, tp) {
        if ident_is(src, tp, b"class") {
            return body(operand_position(t, src, st, kind, n, tp, true, depth));
        }
        if ident_is(src, tp, b"interface") {
            return GtBrace::No;
        }
        if ident_is(src, tp, b"extends") || ident_is(src, tp, b"implements") {
            return body(class_walk_from(t, src, st, kind, n, tq, true, depth));
        }
        let e = bm_next1(st, tp + 1, n);
        if t.is_regex_keyword(src.add(tp), e - tp) {
            return GtBrace::Value;
        }
        return GtBrace::No;
    }
    if tk >= OP_KIND_BASE && *src.add(tp) == b':' {
        return body(colon_return_type_value(t, src, st, kind, n, tp, true, depth));
    }
    if tk >= OP_KIND_BASE && *src.add(tp) == b',' && class_like_walk(src, st, kind, tp) {
        return body(class_walk_from(t, src, st, kind, n, tq, true, depth));
    }
    if tk >= OP_KIND_BASE
        && (matches!(*src.add(tp), b'|' | b'&')
            || (*src.add(tp) == b'>' && tp > 0 && *src.add(tp - 1) == b'=')
            || (*src.add(tp) == b'=' && *src.add(tp + 1) == b'>'))
    {
        return GtBrace::No;
    }
    GtBrace::Value
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
#[inline]
unsafe fn is_binder_keyword(src: *const u8, kind: *const u8, w: usize) -> bool {
    kind_at(kind, w) == IDENT
        && !prop_name(src, w)
        && (ident_is(src, w, b"let")
            || ident_is(src, w, b"const")
            || ident_is(src, w, b"var")
            || ident_is(src, w, b"using"))
}
#[inline]
unsafe fn binder_outside_paren_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    binder: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, binder);
    q < 0 || !(kind_at(kind, q as usize) >= OP_KIND_BASE && *src.add(q as usize) == b'(')
}
unsafe fn declarator_without_init(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    id: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, id);
    if q < 0 {
        return false;
    }
    let mut w = q as usize;
    if is_binder_keyword(src, kind, w) {
        return binder_outside_paren_head(src, st, kind, w);
    }
    if !(kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b',') {
        return false;
    }
    loop {
        if kind_at(kind, w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, w, open, c) {
                        Some(op) => {
                            let q2 = bm_prev_sig(st, kind, op);
                            if q2 < 0 {
                                return false;
                            }
                            w = q2 as usize;
                            continue;
                        }
                        None => return false,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return false,
                _ => {}
            }
        } else if is_binder_keyword(src, kind, w) {
            return binder_outside_paren_head(src, st, kind, w);
        }
        let q2 = bm_prev_sig(st, kind, w);
        if q2 < 0 {
            return false;
        }
        w = q2 as usize;
    }
}
unsafe fn type_alias_head(src: *const u8, st: *const u64, kind: *const u8, eq: usize) -> bool {
    let q = bm_prev_sig(st, kind, eq);
    if q < 0 {
        return false;
    }
    let mut w = q as usize;
    if kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b'>' {
        match angle_match_back(src, st, kind, w) {
            AngleMatch::Found(lt) => {
                let p = bm_prev_sig(st, kind, lt);
                if p < 0 {
                    return false;
                }
                w = p as usize;
            }
            _ => return false,
        }
    }
    if kind_at(kind, w) != IDENT || prop_name(src, w) {
        return false;
    }
    let p = bm_prev_sig(st, kind, w);
    p >= 0
        && kind_at(kind, p as usize) == IDENT
        && !prop_name(src, p as usize)
        && ident_is(src, p as usize, b"type")
}
unsafe fn signature_return_type(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    rparen: usize,
) -> bool {
    let Some(lp) = match_delim_back(src, st, kind, rparen, b'(', b')') else {
        return false;
    };
    function_keyword_before_params(src, st, kind, lp).is_some()
}
unsafe fn annotation_colon_is_declaration(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    depth: u32,
) -> bool {
    if colon_marks_value(t, src, st, kind, n, colon, true, depth) {
        return false;
    }
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let kk = kind_at(kind, w);
    if kk == IDENT {
        return declarator_without_init(src, st, kind, w);
    }
    if kk >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b')' {
            return signature_return_type(src, st, kind, w);
        }
        if c == b'!' && *src.add(w + 1) != b'=' {
            let q2 = bm_prev_sig(st, kind, w);
            return q2 >= 0
                && kind_at(kind, q2 as usize) == IDENT
                && declarator_without_init(src, st, kind, q2 as usize);
        }
        if c == b']' || c == b'}' {
            let open = if c == b']' { b'[' } else { b'{' };
            if let Some(op) = match_delim_back(src, st, kind, w, open, c) {
                let p = bm_prev_sig(st, kind, op);
                if p >= 0 && is_binder_keyword(src, kind, p as usize) {
                    return binder_outside_paren_head(src, st, kind, p as usize);
                }
            }
        }
    }
    false
}
unsafe fn extends_precedes_question(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    question: usize,
) -> bool {
    let mut q = bm_prev_sig(st, kind, question);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
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
                        None => return false,
                    }
                }
                b'(' | b'[' | b'{' | b';' | b':' | b'?' => return false,
                b'>' if !(w > 0 && *src.add(w - 1) == b'=') => {
                    if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                }
                _ => {}
            }
        } else if kind_at(kind, w) == IDENT && !prop_name(src, w) && ident_is(src, w, b"extends") {
            return true;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}
unsafe fn conditional_type_question(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    colon: usize,
) -> Option<usize> {
    let mut debt: u32 = 0;
    let mut q = bm_prev_sig(st, kind, colon);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
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
                        None => return None,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return None,
                b'>' if !(w > 0 && *src.add(w - 1) == b'=') => {
                    if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                }
                b':' => debt += 1,
                b'?' => {
                    if *src.add(w + 1) != b'?'
                        && *src.add(w + 1) != b'.'
                        && (w == 0 || *src.add(w - 1) != b'?')
                    {
                        if debt == 0 {
                            return if extends_precedes_question(src, st, kind, w) {
                                Some(w)
                            } else {
                                None
                            };
                        }
                        debt -= 1;
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}
unsafe fn type_annotation_asi(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
    depth: u32,
) -> bool {
    let mut q = from as i64;
    let mut prev_atom = false;
    let mut prev_params = false;
    while q >= 0 {
        let w = q as usize;
        let kk = *kind.add(w);
        let ends;
        let starts;
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
                    if prev_atom {
                        return false;
                    }
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, w, open, c) {
                        Some(op) => {
                            if c == b'}' && !type_literal_brace_opener(src, st, kind, op) {
                                return false;
                            }
                            prev_atom = c != b']';
                            prev_params = c == b')';
                            q = bm_prev_sig(st, kind, op);
                            continue;
                        }
                        None => return false,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return false,
                b':' => {
                    if annotation_colon_is_declaration(t, src, st, kind, n, w, depth) {
                        return true;
                    }
                    match conditional_type_question(src, st, kind, w) {
                        Some(qm) => {
                            prev_atom = false;
                            q = bm_prev_sig(st, kind, qm);
                            continue;
                        }
                        None => return false,
                    }
                }
                b'=' => {
                    if *src.add(w + 1) != b'>' {
                        if *src.add(w + 1) == b'=' {
                            return false;
                        }
                        if w > 0
                            && matches!(*src.add(w - 1), b'=' | b'!' | b'<' | b'>' | b'+' | b'-')
                        {
                            return false;
                        }
                        return type_alias_head(src, st, kind, w);
                    }
                    ends = false;
                    starts = false;
                }
                b'>' => {
                    if !(w > 0 && *src.add(w - 1) == b'=') {
                        if prev_atom && !prev_params {
                            return false;
                        }
                        match angle_match_back(src, st, kind, w) {
                            AngleMatch::Found(lt) => {
                                prev_atom = false;
                                prev_params = false;
                                q = bm_prev_sig(st, kind, lt);
                                continue;
                            }
                            _ => return false,
                        }
                    }
                    ends = false;
                    starts = false;
                }
                b'.' | b'|' | b'&' | b'?' | b'-' => {
                    ends = false;
                    starts = false;
                }
                _ => return false,
            }
        } else if kk == IDENT || kk == IDENT_ESC {
            let kw = t.kwts.lookup(src.add(w), word_len(src, w)) as u8;
            if matches!(kw, KW_EXTENDS | KW_IS | KW_IN | KW_AS) {
                ends = false;
                starts = false;
            } else if type_prefix_kind(kw) {
                ends = false;
                starts = true;
            } else {
                ends = true;
                starts = true;
            }
        } else if matches!(kk, NUM | BIGINT | STR | TMPL_NOSUB) {
            ends = true;
            starts = true;
        } else if kk == TMPL_HEAD {
            ends = false;
            starts = true;
        } else if kk == TMPL_MIDDLE {
            ends = false;
            starts = false;
        } else if kk == TMPL_TAIL {
            ends = true;
            starts = false;
        } else {
            return false;
        }
        if ends && prev_atom {
            return false;
        }
        prev_atom = starts;
        prev_params = false;
        q = bm_prev_sig(st, kind, w);
    }
    false
}
pub(super) unsafe fn return_type_signature_paren(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    body: usize,
) -> Option<usize> {
    let mut q = bm_prev_sig(st, kind, body);
    while q >= 0 {
        let w = q as usize;
        let k = kind_at(kind, w);
        if k >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    let op = match_delim_back(src, st, kind, w, open, c)?;
                    if c == b'}' && !type_literal_brace_opener(src, st, kind, op) {
                        return None;
                    }
                    q = bm_prev_sig(st, kind, op);
                    continue;
                }
                b'>' => {
                    if w > 0 && *src.add(w - 1) == b'=' {
                        q = bm_prev_sig(st, kind, w);
                        continue;
                    }
                    let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
                        return None;
                    };
                    q = bm_prev_sig(st, kind, lt);
                    continue;
                }
                b'=' => {
                    if *src.add(w + 1) != b'>' {
                        return None;
                    }
                }
                b':' => {
                    let p = bm_prev_sig(st, kind, w);
                    if p >= 0
                        && kind_at(kind, p as usize) >= OP_KIND_BASE
                        && *src.add(p as usize) == b')'
                    {
                        return match_delim_back(src, st, kind, p as usize, b'(', b')');
                    }
                    let qm = conditional_type_question(src, st, kind, w)?;
                    q = bm_prev_sig(st, kind, qm);
                    continue;
                }
                b'.' | b'|' | b'&' | b'?' | b',' => {}
                _ => return None,
            }
        } else if k == IDENT {
            if !prop_name(src, w) && word_is_any(src, w, RETURN_TYPE_STOP_WORDS) {
                return None;
            }
        } else if !matches!(
            k,
            NUM | BIGINT | STR | TMPL_NOSUB | TMPL_HEAD | TMPL_MIDDLE | TMPL_TAIL
        ) {
            return None;
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}
const RETURN_TYPE_STOP_WORDS: &[&[u8]] = &[
    b"return",
    b"throw",
    b"yield",
    b"await",
    b"delete",
    b"case",
    b"else",
    b"do",
    b"if",
    b"while",
    b"for",
    b"switch",
    b"catch",
    b"with",
    b"default",
    b"let",
    b"const",
    b"var",
    b"class",
    b"function",
    b"interface",
    b"enum",
    b"try",
    b"finally",
    b"export",
];
const TYPE_LITERAL_HEAD_WORDS: &[&[u8]] =
    &[b"extends", b"keyof", b"is", b"in", b"readonly", b"asserts"];
unsafe fn type_literal_brace_opener(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    brace: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b':' | b'|' | b'&' | b'=' | b',' | b'<' | b'(' | b'[' | b'?' => true,
            b'>' => w > 0 && *src.add(w - 1) == b'=',
            _ => false,
        };
    }
    k == IDENT && !prop_name(src, w) && word_is_any(src, w, TYPE_LITERAL_HEAD_WORDS)
}
#[inline]
unsafe fn label_after_restricted(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    label: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, label);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    *kind.add(w) == IDENT
        && !prop_name(src, w)
        && (ident_is(src, w, b"break") || ident_is(src, w, b"continue"))
        && !lt_in_range(src, bm_next1(st, w + 1, n), label)
}
#[inline]
unsafe fn module_specifier_asi(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    spec: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, spec);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    *kind.add(w) == IDENT
        && !prop_name(src, w)
        && (ident_is(src, w, b"from") || ident_is(src, w, b"import"))
}
unsafe fn void_is_type_position(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
    depth: u32,
) -> bool {
    let q = bm_prev_sig(st, kind, p);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b':' => !colon_marks_value(t, src, st, kind, n, w, true, depth + 1),
            b',' => class_like_walk(src, st, kind, w),
            _ => false,
        };
    }
    k == IDENT
        && !prop_name(src, w)
        && word_is_any(src, w, &[b"implements", b"extends"])
        && class_like_walk(src, st, kind, w)
}

pub(super) unsafe fn ts_type_region_open(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, lt);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    type_annotation_asi(t, src, st, kind, n, w, 0)
        || member_or_param_colon_in_type(t, src, st, kind, n, w, 0)
        || (kind_at(kind, w) >= OP_KIND_BASE
            && *src.add(w) == b':'
            && (paren_return_type_colon(t, src, st, opch, kind, n, w)
                || index_signature_colon(src, st, kind, n, w)))
}
unsafe fn member_or_param_colon_in_type(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    depth: u32,
) -> bool {
    if depth > 4 || *kind.add(colon) < OP_KIND_BASE || *src.add(colon) != b':' {
        return false;
    }
    let mut q = bm_prev_sig(st, kind, colon);
    if q >= 0 && kind_at(kind, q as usize) == IDENT && !prop_name(src, q as usize) {
        let u = bm_prev_sig(st, kind, q as usize);
        if u >= 0 && kind_at(kind, u as usize) >= OP_KIND_BASE && *src.add(u as usize) == b'(' {
            return true;
        }
    }
    while q >= 0 {
        let w = q as usize;
        if *kind.add(w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
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
                        None => return false,
                    }
                }
                b'{' | b'(' => {
                    let mut p = bm_prev_sig(st, kind, w);
                    if c == b'('
                        && p >= 0
                        && kind_at(kind, p as usize) == IDENT
                        && !prop_name(src, p as usize)
                        && ident_is(src, p as usize, b"new")
                    {
                        p = bm_prev_sig(st, kind, p as usize);
                        if p >= 0
                            && kind_at(kind, p as usize) == IDENT
                            && ident_is(src, p as usize, b"abstract")
                        {
                            p = bm_prev_sig(st, kind, p as usize);
                        }
                    }
                    if p < 0 {
                        return false;
                    }
                    let pp = p as usize;
                    if kind_at(kind, pp) >= OP_KIND_BASE {
                        let pc = *src.add(pp);
                        if c == b'(' && pc == b',' {
                            return true;
                        }
                        if c == b'(' && pc == b'*' {
                            let f = bm_prev_sig(st, kind, pp);
                            if f >= 0
                                && kind_at(kind, f as usize) == IDENT
                                && !prop_name(src, f as usize)
                                && ident_is(src, f as usize, b"function")
                            {
                                return true;
                            }
                        }
                        if c == b'(' && pc == b']' {
                            if let Some(lb) = match_delim_back(src, st, kind, pp, b'[', b']') {
                                if member_start_before(src, st, kind, lb, true) {
                                    return true;
                                }
                            }
                        }
                    }
                    if type_head_keyword(src, st, kind, pp, c) {
                        return true;
                    }
                    return type_annotation_asi(t, src, st, kind, n, pp, depth + 1)
                        || member_or_param_colon_in_type(t, src, st, kind, n, pp, depth + 1);
                }
                b'[' => {
                    let p = bm_prev_sig(st, kind, w);
                    if p < 0 || kind_at(kind, p as usize) < OP_KIND_BASE {
                        return false;
                    }
                    let pc = *src.add(p as usize);
                    if matches!(pc, b':' | b'|' | b'&' | b',')
                        || (pc == b'<' && *src.add(p as usize + 1) != b'<')
                    {
                        return true;
                    }
                    return pc == b'=' && type_alias_head(src, st, kind, p as usize);
                }
                b';' => return false,
                b'?' => {
                    if *src.add(w + 1) != b'?'
                        && *src.add(w + 1) != b'.'
                        && (w == 0 || *src.add(w - 1) != b'?')
                    {
                        let mut f = w + 1;
                        while f < n && is_ws(*src.add(f)) {
                            f += 1;
                        }
                        if !matches!(*src.add(f), b':' | b',' | b')' | b']') {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}
#[inline]
unsafe fn type_head_keyword(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    p: usize,
    opener: u8,
) -> bool {
    let mut w = p;
    for _ in 0..8 {
        if *kind.add(w) >= OP_KIND_BASE
            && *src.add(w) == b'>'
            && !(w > 0 && *src.add(w - 1) == b'=')
        {
            let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
                return false;
            };
            let q = bm_prev_sig(st, kind, lt);
            if q < 0 {
                return false;
            }
            w = q as usize;
            continue;
        }
        if !matches!(*kind.add(w), IDENT | IDENT_ESC) {
            return false;
        }
        if !prop_name(src, w) {
            if opener == b'{' && (ident_is(src, w, b"interface") || ident_is(src, w, b"class")) {
                return true;
            }
            if opener == b'(' && ident_is(src, w, b"function") {
                return true;
            }
        }
        let q = bm_prev_sig(st, kind, w);
        if q < 0 {
            return false;
        }
        w = q as usize;
    }
    false
}

/// Cap for scanning `>` runs: matching `<` is within 100 bytes
/// Hitting the cap returns `None` (fuse), so it can only widen the residual, never split a shift.
const GT_SCAN_CAP: usize = 1 << 16;

/// Bytes `gt_run_closes_type_args` reacts to. Everything else is skipped without touching a bitmap.
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

const CTX_HOPS: u32 = 32;
const ENCLOSING_SCAN_CAP: usize = 1 << 16;

/// The `<` opening the outermost of the `run` nested type-argument lists that
/// the run of `>` bytes at `gt` would close, or `None` when the region is not
/// delimiter-balanced.
///
/// Reads source bytes backward rather than walking tokens: the matching `<`
/// is a hundred bytes away, but the token walk that finds it has to cross
/// every `(`/`[`/`{` in between and match each one back to its opener, which
/// measured ~5,000 cycles a site on `ts_zod.ts` (+1.08 cyc/B). Angles are
/// gated on `opch & st` so literal interiors and JSX tag punctuation are
/// invisible; the bracket counters are gated on `st` alone.
///
/// The counters are what separate the two readings. A type-argument list is
/// always delimiter-balanced, so an unmatched `(`/`[`/`{`, or a `;` outside
/// any of them, proves the region is not one — that is what rejects
/// `(a << 3) | (a >>> 29)`, `o[(y = e) >> 2]` and `x >>= 8`. A `<<` whose
/// second byte is no longer a token start is a fused shift; one that
/// `lt_run_split` already split counts as two openers.
unsafe fn gt_run_closes_type_args(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    gt: usize,
    run: usize,
) -> Option<usize> {
    let mut depth = run as i32;
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let lo = gt.saturating_sub(GT_SCAN_CAP);
    let mut i = gt;
    while i > lo {
        i -= 1;
        let c = *src.add(i);
        if !GT_SCAN_DELIM[c as usize] {
            continue;
        }
        if !bm_get(st, i) {
            continue;
        }
        match c {
            b'>' => {
                if !bm_get(opch, i) {
                    continue;
                }
                if i > 0 && *src.add(i - 1) == b'=' {
                    continue;
                }
                let nx = *src.add(i + 1);
                if (nx == b'>' || nx == b'=') && !bm_get(st, i + 1) {
                    return None;
                }
                depth += 1;
            }
            b'<' => {
                if !bm_get(opch, i) {
                    continue;
                }
                let nx = *src.add(i + 1);
                if nx == b'=' || (nx == b'<' && !bm_get(st, i + 1)) {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    return (par == 0 && brk == 0 && brc == 0).then_some(i);
                }
            }
            b')' => par += 1,
            b'(' => {
                par -= 1;
                if par < 0 {
                    return None;
                }
            }
            b']' => brk += 1,
            b'[' => {
                brk -= 1;
                if brk < 0 {
                    return None;
                }
            }
            b'}' => {
                // A substitution-closing `}` is the start of the next
                // template segment, and its `${` was swallowed by the
                // preceding one — counting it would leave every
                // `Array<Map<A, `p${s}q`>>` looking brace-unbalanced.
                let kk = kind_at(kind, i);
                if kk == TMPL_MIDDLE || kk == TMPL_TAIL {
                    continue;
                }
                brc += 1;
            }
            b'{' => {
                brc -= 1;
                if brc < 0 {
                    return None;
                }
            }
            _ => {
                if par == 0 && brk == 0 && brc == 0 {
                    return None;
                }
            }
        }
    }
    None
}

#[inline(always)]
unsafe fn word_len(src: *const u8, w: usize) -> usize {
    let mut e = w + 1;
    while is_word(*src.add(e)) {
        e += 1;
    }
    e - w
}

unsafe fn word_is_any(src: *const u8, w: usize, words: &[&[u8]]) -> bool {
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

const fn kw(k: TokenKind) -> u8 {
    k as u8
}

const KW_EXTENDS: u8 = kw(TokenKind::KwExtends);
const KW_IS: u8 = kw(TokenKind::KwIs);
const KW_IN: u8 = kw(TokenKind::KwIn);
const KW_KEYOF: u8 = kw(TokenKind::KwKeyof);
const KW_TYPEOF: u8 = kw(TokenKind::KwTypeof);
const KW_READONLY: u8 = kw(TokenKind::KwReadonly);
const KW_UNIQUE: u8 = kw(TokenKind::KwUnique);
const KW_INFER: u8 = kw(TokenKind::KwInfer);
const KW_ABSTRACT: u8 = kw(TokenKind::KwAbstract);
const KW_NEW: u8 = kw(TokenKind::KwNew);
const KW_ASSERTS: u8 = kw(TokenKind::KwAsserts);
const KW_IMPORT: u8 = kw(TokenKind::KwImport);
const KW_AS: u8 = kw(TokenKind::KwAs);
const KW_AWAIT: u8 = kw(TokenKind::KwAwait);
const KW_YIELD: u8 = kw(TokenKind::KwYield);
const KW_DELETE: u8 = kw(TokenKind::KwDelete);
const KW_FUNCTION: u8 = kw(TokenKind::KwFunction);
const KW_CLASS: u8 = kw(TokenKind::KwClass);
const KW_INSTANCEOF: u8 = kw(TokenKind::KwInstanceof);
const KW_SUPER: u8 = kw(TokenKind::KwSuper);
const KW_THIS: u8 = kw(TokenKind::KwThis);
const KW_SWITCH: u8 = kw(TokenKind::KwSwitch);
const KW_CASE: u8 = kw(TokenKind::KwCase);
const KW_RETURN: u8 = kw(TokenKind::KwReturn);
const KW_THROW: u8 = kw(TokenKind::KwThrow);
const KW_VAR: u8 = kw(TokenKind::KwVar);
const KW_LET: u8 = kw(TokenKind::KwLet);
const KW_CONST: u8 = kw(TokenKind::KwConst);
const KW_IF: u8 = kw(TokenKind::KwIf);
const KW_ELSE: u8 = kw(TokenKind::KwElse);
const KW_FOR: u8 = kw(TokenKind::KwFor);
const KW_WHILE: u8 = kw(TokenKind::KwWhile);
const KW_DO: u8 = kw(TokenKind::KwDo);
const KW_BREAK: u8 = kw(TokenKind::KwBreak);
const KW_CONTINUE: u8 = kw(TokenKind::KwContinue);
const KW_WITH: u8 = kw(TokenKind::KwWith);
const KW_TRY: u8 = kw(TokenKind::KwTry);
const KW_CATCH: u8 = kw(TokenKind::KwCatch);
const KW_FINALLY: u8 = kw(TokenKind::KwFinally);
const KW_DEBUGGER: u8 = kw(TokenKind::KwDebugger);
const KW_DEFAULT: u8 = kw(TokenKind::KwDefault);
const KW_EXPORT: u8 = kw(TokenKind::KwExport);
const KW_ENUM: u8 = kw(TokenKind::KwEnum);

#[inline(always)]
fn type_illegal_kind(k: u8) -> bool {
    matches!(
        k,
        KW_AWAIT
            | KW_YIELD
            | KW_DELETE
            | KW_FUNCTION
            | KW_CLASS
            | KW_INSTANCEOF
            | KW_SUPER
            | KW_SWITCH
            | KW_CASE
            | KW_RETURN
            | KW_THROW
            | KW_VAR
            | KW_LET
            | KW_CONST
            | KW_IF
            | KW_ELSE
            | KW_FOR
            | KW_WHILE
            | KW_DO
            | KW_BREAK
            | KW_CONTINUE
            | KW_WITH
            | KW_TRY
            | KW_CATCH
            | KW_FINALLY
            | KW_DEBUGGER
            | KW_DEFAULT
            | KW_EXPORT
            | KW_ENUM
    )
}

#[inline(always)]
fn type_prefix_kind(k: u8) -> bool {
    matches!(
        k,
        KW_KEYOF
            | KW_TYPEOF
            | KW_READONLY
            | KW_UNIQUE
            | KW_INFER
            | KW_ABSTRACT
            | KW_NEW
            | KW_ASSERTS
            | KW_IMPORT
            | KW_EXTENDS
            | KW_IS
            | KW_IN
            | KW_AS
    )
}

unsafe fn type_list_legal(
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
        if w == skip || k == WS || k == LCOM || k == BCOM {
            w = bm_next1(st, w + 1, hi);
            continue;
        }
        let c = *src.add(w);
        let mut ok_paren = false;
        let was_this = this_head;
        this_head = false;
        if k == IDENT || k == IDENT_ESC {
            let kk = t.kwts.lookup(src.add(w), word_len(src, w)) as u8;
            this_head = kk == KW_THIS;
            if !start && brc == 0 && !matches!(kk, KW_EXTENDS | KW_IS | KW_IN) {
                return false;
            }
            if type_illegal_kind(kk) {
                return false;
            }
            if kk == KW_EXTENDS {
                cond_ok = true;
            }
            start = type_prefix_kind(kk);
        } else if k == NUM || k == BIGINT || k == STR || k == TMPL_NOSUB || k == TMPL_TAIL {
            if !start && brc == 0 && k != TMPL_TAIL {
                return false;
            }
            start = false;
        } else if k == TMPL_HEAD || k == TMPL_MIDDLE {
            if k == TMPL_HEAD && !start && brc == 0 {
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Follow {
    Split,
    Fuse,
    Ctx,
}

const FOLLOW_SPLIT_WORDS: &[&[u8]] =
    &[b"in", b"instanceof", b"as", b"satisfies", b"extends", b"implements"];

unsafe fn gt_follower(src: *const u8, n: usize, mut i: usize) -> Follow {
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
            let wl = super::classify::unicode_ws_len(src, i);
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Type,
    Expr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum After {
    Head,
    Paren,
    Bracket,
}

enum Encl {
    Open(usize, u8),
    Top,
    Capped,
}

const LT_OPERAND_WORDS: &[&[u8]] = &[
    b"return",
    b"debugger",
    b"break",
    b"continue",
    b"default",
    b"class",
    b"function",
    b"throw",
    b"yield",
    b"await",
    b"typeof",
    b"void",
    b"delete",
    b"in",
    b"instanceof",
    b"case",
    b"else",
    b"do",
    b"of",
    b"extends",
    b"implements",
    b"as",
    b"satisfies",
    b"keyof",
    b"infer",
    b"readonly",
    b"is",
    b"asserts",
    b"unique",
];
const EXPR_WORDS: &[&[u8]] = &[
    b"new",
    b"return",
    b"throw",
    b"yield",
    b"await",
    b"typeof",
    b"delete",
    b"void",
    b"in",
    b"instanceof",
    b"case",
    b"else",
    b"do",
    b"of",
    b"if",
    b"while",
    b"for",
    b"switch",
    b"catch",
    b"with",
    b"default",
    b"let",
    b"const",
    b"var",
];
const TYPE_WORDS: &[&[u8]] = &[
    b"class",
    b"interface",
    b"type",
    b"function",
    b"extends",
    b"implements",
    b"as",
    b"satisfies",
    b"is",
    b"asserts",
    b"keyof",
    b"infer",
    b"readonly",
    b"unique",
    b"abstract",
    b"declare",
    b"enum",
    b"namespace",
    b"module",
    b"get",
    b"set",
    b"static",
    b"public",
    b"private",
    b"protected",
    b"override",
    b"accessor",
];
const MEMBER_OPEN_WORDS: &[&[u8]] = &[
    b"class",
    b"interface",
    b"enum",
    b"return",
    b"case",
    b"in",
    b"of",
    b"typeof",
    b"await",
    b"yield",
    b"delete",
    b"void",
    b"instanceof",
    b"new",
    b"export",
    b"default",
    b"throw",
    b"as",
    b"satisfies",
    b"extends",
    b"implements",
    b"keyof",
];
const BLOCK_OPEN_WORDS: &[&[u8]] = &[b"else", b"do", b"try", b"finally", b"declare", b"global"];
const GENERATOR_STAR_WORDS: &[&[u8]] =
    &[b"static", b"async", b"public", b"private", b"protected", b"override", b"abstract"];
const CLASS_WALK_STOP_WORDS: &[&[u8]] = &[
    b"function",
    b"return",
    b"if",
    b"while",
    b"for",
    b"switch",
    b"catch",
    b"with",
    b"new",
    b"typeof",
    b"await",
    b"yield",
    b"else",
    b"do",
    b"try",
    b"finally",
    b"namespace",
    b"module",
];

unsafe fn enclosing_opener(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
    stop_semi: bool,
    cap: usize,
) -> Encl {
    let lo = from.saturating_sub(cap);
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let mut ang: i32 = 0;
    let mut i = from + 1;
    while i > lo {
        i -= 1;
        let c = *src.add(i);
        if !GT_SCAN_DELIM[c as usize] || !bm_get(st, i) {
            continue;
        }
        match c {
            b'>' => {
                if bm_get(opch, i) && !(i > 0 && *src.add(i - 1) == b'=') {
                    ang += 1;
                }
            }
            b'<' => {
                if !bm_get(opch, i) {
                    continue;
                }
                let nx = *src.add(i + 1);
                if nx == b'=' || (nx == b'<' && !bm_get(st, i + 1)) {
                    continue;
                }
                if nx == b'<' || (i > 0 && *src.add(i - 1) == b'<' && bm_get(st, i - 1)) {
                    let first = if nx == b'<' { i } else { i - 1 };
                    if !lt_run_opens_type_args(src, st, opch, kind, n, first) {
                        continue;
                    }
                }
                if ang == 0 {
                    return Encl::Open(i, b'<');
                }
                ang -= 1;
            }
            b')' => par += 1,
            b'(' => {
                if par == 0 {
                    return Encl::Open(i, b'(');
                }
                par -= 1;
            }
            b']' => brk += 1,
            b'[' => {
                if brk == 0 {
                    return Encl::Open(i, b'[');
                }
                brk -= 1;
            }
            b'}' => {
                let kk = kind_at(kind, i);
                if kk == TMPL_MIDDLE || kk == TMPL_TAIL {
                    continue;
                }
                brc += 1;
            }
            b'{' => {
                if brc == 0 {
                    return Encl::Open(i, b'{');
                }
                brc -= 1;
            }
            _ => {
                if stop_semi && par == 0 && brk == 0 && brc == 0 && ang == 0 {
                    return Encl::Top;
                }
            }
        }
    }
    if lo == 0 { Encl::Top } else { Encl::Capped }
}

unsafe fn class_like_walk(src: *const u8, st: *const u64, kind: *const u8, from: usize) -> bool {
    let mut q = from as i64;
    while q >= 0 {
        let w = q as usize;
        let k = kind_at(kind, w);
        if k == IDENT || k == IDENT_ESC {
            if !prop_name(src, w) {
                if word_is_any(src, w, &[b"class", b"interface", b"enum"]) {
                    return true;
                }
                if word_is_any(src, w, CLASS_WALK_STOP_WORDS) {
                    return false;
                }
            }
        } else if k >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b'.' | b',' => {}
                b')' => match match_delim_back(src, st, kind, w, b'(', b')') {
                    Some(o) => {
                        q = bm_prev_sig(st, kind, o);
                        continue;
                    }
                    None => return false,
                },
                b'>' => {
                    if w > 0 && *src.add(w - 1) == b'=' {
                        return false;
                    }
                    match angle_match_back(src, st, kind, w) {
                        AngleMatch::Found(lt) => {
                            q = bm_prev_sig(st, kind, lt);
                            continue;
                        }
                        _ => return false,
                    }
                }
                _ => return false,
            }
        } else if !matches!(k, STR | NUM | TMPL_NOSUB) {
            return false;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

unsafe fn brace_is_member_container(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    brace: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k == IDENT || k == IDENT_ESC {
        if prop_name(src, w) {
            return chain_head(src, st, kind, w).is_some_and(|h| class_like_walk(src, st, kind, h));
        }
        if word_is_any(src, w, MEMBER_OPEN_WORDS) {
            return true;
        }
        if word_is_any(src, w, BLOCK_OPEN_WORDS) {
            return false;
        }
        return class_like_walk(src, st, kind, w);
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        return match c {
            b')' | b'>' => class_like_walk(src, st, kind, w),
            b'=' => *src.add(w + 1) != b'>',
            b'{' | b';' | b'}' => false,
            _ => true,
        };
    }
    false
}

unsafe fn optional_method_marker(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    question: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, question);
    if q < 0 {
        return false;
    }
    let mut name = q as usize;
    let nk = kind_at(kind, name);
    if nk >= OP_KIND_BASE {
        if *src.add(name) != b']' {
            return false;
        }
        let Some(lb) = match_delim_back(src, st, kind, name, b'[', b']') else {
            return false;
        };
        name = lb;
    } else if !matches!(nk, IDENT | STR | NUM | BIGINT | PRIV_IDENT) {
        return false;
    }
    if !member_start_before(src, st, kind, name, true) {
        return false;
    }
    match enclosing_opener(src, st, opch, kind, n, q as usize, false, ENCLOSING_SCAN_CAP) {
        Encl::Open(o, b'{') => brace_is_type_literal(t, src, st, opch, kind, n, o),
        Encl::Capped => true,
        _ => false,
    }
}

unsafe fn ternary_colon(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
) -> bool {
    let mut debt: u32 = 0;
    let mut q = bm_prev_sig(st, kind, colon);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
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
                b'{' | b'(' | b'[' | b';' => return false,
                b'>' if !(w > 0 && *src.add(w - 1) == b'=') => {
                    if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                }
                b':' => debt += 1,
                b'?' => {
                    if *src.add(w + 1) != b'?'
                        && *src.add(w + 1) != b'.'
                        && (w == 0 || *src.add(w - 1) != b'?')
                    {
                        let mut j = w + 1;
                        while is_ws(*src.add(j)) {
                            j += 1;
                        }
                        let optional = matches!(*src.add(j), b',' | b']' | b')' | b':')
                            || (*src.add(j) == b'('
                                && optional_method_marker(t, src, st, opch, kind, n, w));
                        if !optional {
                            if debt == 0 {
                                return !extends_precedes_question(src, st, kind, w);
                            }
                            debt -= 1;
                        }
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

const SIGNATURE_HEAD_WORDS: &[&[u8]] = &[
    b"function",
    b"async",
    b"get",
    b"set",
    b"static",
    b"public",
    b"private",
    b"protected",
    b"readonly",
    b"abstract",
    b"override",
    b"accessor",
    b"declare",
    b"constructor",
];

unsafe fn signature_colon(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    rparen: usize,
) -> Option<Ctx> {
    let lp = match_delim_back(src, st, kind, rparen, b'(', b')')?;
    let mut q = bm_prev_sig(st, kind, lp);
    if q < 0 {
        return None;
    }
    let mut w = q as usize;
    if kind_at(kind, w) >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b'>' && !(w > 0 && *src.add(w - 1) == b'=') {
            let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
                return None;
            };
            q = bm_prev_sig(st, kind, lt);
            if q < 0 {
                return None;
            }
            w = q as usize;
        } else if c == b'*' {
            q = bm_prev_sig(st, kind, w);
            if q < 0 {
                return None;
            }
            w = q as usize;
        } else {
            if c != b'?' || *src.add(w + 1) == b'?' || *src.add(w + 1) == b'.' {
                return None;
            }
            let nm = bm_prev_sig(st, kind, w);
            if nm < 0 {
                return Some(Ctx::Expr);
            }
            let mut name = nm as usize;
            let nk = kind_at(kind, name);
            if nk >= OP_KIND_BASE {
                if *src.add(name) != b']' {
                    return Some(Ctx::Expr);
                }
                let Some(lb) = match_delim_back(src, st, kind, name, b'[', b']') else {
                    return Some(Ctx::Expr);
                };
                name = lb;
            } else if !matches!(
                nk,
                IDENT | IDENT_ESC | STR | NUM | BIGINT | PRIV_IDENT | PRIV_IDENT_ESC
            ) {
                return Some(Ctx::Expr);
            }
            return if member_start_before(src, st, kind, name, false) {
                Some(Ctx::Type)
            } else {
                Some(Ctx::Expr)
            };
        }
    }
    let k = kind_at(kind, w);
    if k != IDENT && k != IDENT_ESC {
        return None;
    }
    if !prop_name(src, w) && word_is_any(src, w, SIGNATURE_HEAD_WORDS) {
        return Some(Ctx::Type);
    }
    let head = chain_head(src, st, kind, w)?;
    let pq = bm_prev_sig(st, kind, head);
    if pq < 0 {
        return Some(Ctx::Type);
    }
    let pw = pq as usize;
    let pk = kind_at(kind, pw);
    if pk >= OP_KIND_BASE {
        let c = *src.add(pw);
        if c == b'?' && *src.add(pw + 1) != b'?' && *src.add(pw + 1) != b'.' {
            return if extends_precedes_question(src, st, kind, pw) {
                Some(Ctx::Type)
            } else {
                Some(Ctx::Expr)
            };
        }
        if matches!(c, b'{' | b';' | b'}' | b',' | b'*' | b'>' | b']') {
            return Some(Ctx::Type);
        }
        return None;
    }
    if (pk == IDENT || pk == IDENT_ESC)
        && !prop_name(src, pw)
        && word_is_any(src, pw, SIGNATURE_HEAD_WORDS)
    {
        return Some(Ctx::Type);
    }
    None
}

unsafe fn colon_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    hops: u32,
) -> Ctx {
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 {
        return Ctx::Expr;
    }
    let v = q as usize;
    let vk = kind_at(kind, v);
    if (vk == IDENT || vk == IDENT_ESC) && !prop_name(src, v) && ident_is(src, v, b"default") {
        return Ctx::Expr;
    }
    let q2 = bm_prev_sig(st, kind, v);
    if q2 >= 0 {
        let u = q2 as usize;
        if kind_at(kind, u) == IDENT && !prop_name(src, u) && ident_is(src, u, b"case") {
            return Ctx::Expr;
        }
        if (vk == IDENT || vk == IDENT_ESC) && !prop_name(src, v) && is_binder_keyword(src, kind, u)
        {
            return Ctx::Type;
        }
    }
    if vk >= OP_KIND_BASE {
        let c = *src.add(v);
        if c == b'?' && !matches!(*src.add(v + 1), b'?' | b'.') {
            return Ctx::Type;
        }
        if c == b')' {
            if let Some(ctx) = signature_colon(src, st, kind, v) {
                return ctx;
            }
        }
        if c == b']' || c == b'}' {
            let open = if c == b']' { b'[' } else { b'{' };
            if let Some(op) = match_delim_back(src, st, kind, v, open, c) {
                let bp = bm_prev_sig(st, kind, op);
                if bp >= 0 && is_binder_keyword(src, kind, bp as usize) {
                    return Ctx::Type;
                }
            }
        }
    }
    if conditional_type_question(src, st, kind, colon).is_some() {
        return Ctx::Type;
    }
    if ternary_colon(t, src, st, opch, kind, n, colon) {
        return Ctx::Expr;
    }
    if annotation_colon_is_declaration(t, src, st, kind, n, colon, 0) {
        return Ctx::Type;
    }
    if vk >= OP_KIND_BASE && *src.add(v) == b')' {
        return Ctx::Type;
    }
    match enclosing_opener(src, st, opch, kind, n, colon, false, ENCLOSING_SCAN_CAP) {
        Encl::Open(o, b'{') => {
            let bp = bm_prev_sig(st, kind, o);
            if bp >= 0 {
                let bw = bp as usize;
                if kind_at(kind, bw) >= OP_KIND_BASE
                    && matches!(*src.add(bw), b')' | b'>')
                    && class_like_walk(src, st, kind, bw)
                {
                    return Ctx::Type;
                }
            }
            ctx_after_token(t, src, st, opch, kind, n, bp, After::Head, hops + 1)
        }
        Encl::Open(..) => Ctx::Type,
        Encl::Top => Ctx::Expr,
        Encl::Capped => Ctx::Type,
    }
}

unsafe fn eq_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    eq: usize,
    hops: u32,
) -> Ctx {
    let nx = *src.add(eq + 1);
    if nx == b'>' {
        return arrow_context(t, src, st, opch, kind, n, eq, hops);
    }
    if nx == b'=' || (eq > 0 && matches!(*src.add(eq - 1), b'=' | b'!' | b'<' | b'>')) {
        return Ctx::Expr;
    }
    if type_alias_head(src, st, kind, eq) {
        return Ctx::Type;
    }
    match enclosing_opener(src, st, opch, kind, n, eq, true, GT_SCAN_CAP) {
        Encl::Open(_, b'<') => Ctx::Type,
        _ => Ctx::Expr,
    }
}

unsafe fn arrow_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    eq: usize,
    hops: u32,
) -> Ctx {
    let q = bm_prev_sig(st, kind, eq);
    if q < 0 {
        return Ctx::Expr;
    }
    let v = q as usize;
    if kind_at(kind, v) < OP_KIND_BASE || *src.add(v) != b')' {
        return Ctx::Expr;
    }
    let Some(lp) = match_delim_back(src, st, kind, v, b'(', b')') else {
        return Ctx::Expr;
    };
    let mut b = bm_prev_sig(st, kind, lp);
    if b >= 0 {
        let bw = b as usize;
        if kind_at(kind, bw) >= OP_KIND_BASE
            && *src.add(bw) == b'>'
            && !(bw > 0 && *src.add(bw - 1) == b'=')
        {
            match angle_match_back(src, st, kind, bw) {
                AngleMatch::Found(lt) => b = bm_prev_sig(st, kind, lt),
                _ => return Ctx::Expr,
            }
        }
    }
    if b >= 0 {
        let bw = b as usize;
        let bk = kind_at(kind, bw);
        if (bk == IDENT || bk == IDENT_ESC) && !prop_name(src, bw) {
            if ident_is(src, bw, b"async") {
                return Ctx::Expr;
            }
            if ident_is(src, bw, b"new") {
                b = bm_prev_sig(st, kind, bw);
                if b >= 0
                    && kind_at(kind, b as usize) == IDENT
                    && ident_is(src, b as usize, b"abstract")
                {
                    b = bm_prev_sig(st, kind, b as usize);
                }
            }
        }
    }
    ctx_after_token(t, src, st, opch, kind, n, b, After::Head, hops + 1)
}

const MEMBER_MODIFIER_WORDS: &[&[u8]] = &[
    b"static",
    b"async",
    b"get",
    b"set",
    b"public",
    b"private",
    b"protected",
    b"override",
    b"abstract",
    b"readonly",
    b"declare",
    b"accessor",
];

unsafe fn paren_is_statement_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lp: usize,
) -> bool {
    let h = bm_prev_sig(st, kind, lp);
    h >= 0
        && kind_at(kind, h as usize) == IDENT
        && !prop_name(src, h as usize)
        && word_is_any(src, h as usize, &[b"if", b"while", b"for", b"with"])
}

unsafe fn decorator_before(src: *const u8, st: *const u64, kind: *const u8, w: usize) -> bool {
    let Some(h) = chain_head(src, st, kind, w) else {
        return false;
    };
    let at = bm_prev_sig(st, kind, h);
    at >= 0 && *src.add(at as usize) == b'@'
}

unsafe fn member_start_before(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    pos: usize,
    allow_comma: bool,
) -> bool {
    let p = bm_prev_sig(st, kind, pos);
    if p < 0 {
        return false;
    }
    let pw = p as usize;
    let pk = kind_at(kind, pw);
    if pk >= OP_KIND_BASE {
        let c = *src.add(pw);
        if c == b')' {
            let Some(lp) = match_delim_back(src, st, kind, pw, b'(', b')') else {
                return false;
            };
            let d = bm_prev_sig(st, kind, lp);
            return d >= 0
                && matches!(kind_at(kind, d as usize), IDENT | IDENT_ESC)
                && decorator_before(src, st, kind, d as usize);
        }
        return matches!(c, b'{' | b';' | b'}' | b'*') || (allow_comma && c == b',');
    }
    if (pk == IDENT || pk == IDENT_ESC) && !prop_name(src, pw) {
        return word_is_any(src, pw, MEMBER_MODIFIER_WORDS) || decorator_before(src, st, kind, pw);
    }
    false
}

unsafe fn computed_key_at_member_start(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lb: usize,
) -> bool {
    member_start_before(src, st, kind, lb, true)
}

unsafe fn operand_head_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    op: usize,
    after_break: bool,
    hops: u32,
) -> Ctx {
    let mut q = bm_prev_sig(st, kind, op);
    let mut start: i64 = -1;
    while q >= 0 {
        let v = q as usize;
        let vk = kind_at(kind, v);
        if vk >= OP_KIND_BASE {
            let c = *src.add(v);
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, v, open, c) {
                        Some(o) => {
                            if after_break && c == b')' && paren_is_statement_head(src, st, kind, o)
                            {
                                return Ctx::Type;
                            }
                            start = o as i64;
                            q = bm_prev_sig(st, kind, o);
                            continue;
                        }
                        None => return Ctx::Type,
                    }
                }
                b'>' => {
                    if v > 0 && *src.add(v - 1) == b'=' {
                        break;
                    }
                    match angle_match_back(src, st, kind, v) {
                        AngleMatch::Found(lt) => {
                            start = lt as i64;
                            q = bm_prev_sig(st, kind, lt);
                            continue;
                        }
                        _ => return Ctx::Expr,
                    }
                }
                b'.' => {
                    start = v as i64;
                    q = bm_prev_sig(st, kind, v);
                    continue;
                }
                b'-' if start >= 0
                    && matches!(kind_at(kind, start as usize), NUM | BIGINT)
                    && !matches!(*src.add(v + 1), b'-' | b'=') =>
                {
                    start = v as i64;
                    q = bm_prev_sig(st, kind, v);
                    continue;
                }
                _ => break,
            }
        } else if matches!(vk, IDENT | IDENT_ESC | NUM | BIGINT | STR | TMPL_NOSUB) {
            if (vk == IDENT || vk == IDENT_ESC)
                && !prop_name(src, v)
                && word_is_any(src, v, &[b"as", b"satisfies"])
            {
                return if after_break { Ctx::Expr } else { Ctx::Type };
            }
            start = v as i64;
            q = bm_prev_sig(st, kind, v);
            continue;
        } else if vk == TMPL_TAIL {
            let mut depth = 1u32;
            let mut h = bm_prev_sig(st, kind, v);
            while h >= 0 {
                let hk = kind_at(kind, h as usize);
                if hk == TMPL_TAIL {
                    depth += 1;
                } else if hk == TMPL_HEAD {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                h = bm_prev_sig(st, kind, h as usize);
            }
            if h < 0 {
                return Ctx::Expr;
            }
            start = h;
            q = bm_prev_sig(st, kind, h as usize);
            continue;
        } else {
            break;
        }
    }
    if start < 0 {
        return ctx_after_token(t, src, st, opch, kind, n, q, After::Head, hops + 1);
    }
    let p = bm_prev_sig(st, kind, start as usize);
    ctx_after_token(t, src, st, opch, kind, n, p, After::Head, hops + 1)
}

unsafe fn enclosing_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
    stop_semi: bool,
    hops: u32,
) -> Ctx {
    if stop_semi && class_like_walk(src, st, kind, from) {
        return Ctx::Type;
    }
    let cap = if stop_semi { GT_SCAN_CAP } else { ENCLOSING_SCAN_CAP };
    match enclosing_opener(src, st, opch, kind, n, from, stop_semi, cap) {
        Encl::Open(o, b'<') => enclosing_list_context(t, src, st, opch, kind, n, o, hops + 1),
        Encl::Open(o, b'(') => ctx_after_token(
            t,
            src,
            st,
            opch,
            kind,
            n,
            bm_prev_sig(st, kind, o),
            After::Paren,
            hops + 1,
        ),
        Encl::Open(o, b'[') => ctx_after_token(
            t,
            src,
            st,
            opch,
            kind,
            n,
            bm_prev_sig(st, kind, o),
            After::Bracket,
            hops + 1,
        ),
        Encl::Open(o, _) => {
            if brace_is_member_container(src, st, kind, o) {
                Ctx::Type
            } else {
                Ctx::Expr
            }
        }
        Encl::Top => Ctx::Expr,
        Encl::Capped => Ctx::Type,
    }
}

unsafe fn enclosing_list_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    hops: u32,
) -> Ctx {
    if hops > CTX_HOPS {
        return Ctx::Type;
    }
    if ctx_for_lt(t, src, st, opch, kind, n, lt, hops + 1) == Ctx::Type {
        return Ctx::Type;
    }
    let lim = (lt + 2 * GT_SCAN_CAP).min(n);
    let (close, capped) = angle_close_fwd_capped(src, st, opch, kind, lt + 1, lim, 1);
    match close {
        Some(gt) => match gt_follower(src, n, gt + 1) {
            Follow::Split => {
                if type_list_legal(t, src, st, kind, lt + 1, gt) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            Follow::Fuse => {
                if lt_head_is_operand(t, src, st, opch, kind, n, lt, hops + 1) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            Follow::Ctx => Ctx::Expr,
        },
        None => {
            if capped {
                Ctx::Type
            } else {
                Ctx::Expr
            }
        }
    }
}

unsafe fn ctx_after_token(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: i64,
    mode: After,
    hops: u32,
) -> Ctx {
    if hops > CTX_HOPS {
        return Ctx::Type;
    }
    if p < 0 {
        return Ctx::Expr;
    }
    let w = p as usize;
    let k = kind_at(kind, w);
    if k == IDENT || k == IDENT_ESC {
        if prop_name(src, w) {
            let Some(h) = chain_head(src, st, kind, w) else {
                return Ctx::Expr;
            };
            return ctx_after_token(
                t,
                src,
                st,
                opch,
                kind,
                n,
                bm_prev_sig(st, kind, h),
                mode,
                hops + 1,
            );
        }
        if word_is_any(src, w, EXPR_WORDS) {
            return Ctx::Expr;
        }
        if ident_is(src, w, b"async") {
            return if mode == After::Paren { Ctx::Expr } else { Ctx::Type };
        }
        if word_is_any(src, w, TYPE_WORDS) {
            return Ctx::Type;
        }
        return match mode {
            After::Head => Ctx::Type,
            After::Paren => Ctx::Expr,
            After::Bracket => {
                let Some(h) = chain_head(src, st, kind, w) else {
                    return Ctx::Expr;
                };
                ctx_after_token(
                    t,
                    src,
                    st,
                    opch,
                    kind,
                    n,
                    bm_prev_sig(st, kind, h),
                    After::Head,
                    hops + 1,
                )
            }
        };
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        return match c {
            b')' => Ctx::Expr,
            b'.' => {
                if *src.add(w + 1) != b'.' || *src.add(w + 2) != b'.' {
                    return Ctx::Expr;
                }
                match enclosing_opener(src, st, opch, kind, n, w, true, GT_SCAN_CAP) {
                    Encl::Open(o, b'[') => ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Bracket,
                        hops + 1,
                    ),
                    Encl::Open(o, b'(') => ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Paren,
                        hops + 1,
                    ),
                    _ => Ctx::Expr,
                }
            }
            b'*' => {
                if mode != After::Head || *src.add(w + 1) == b'*' {
                    return Ctx::Expr;
                }
                let pv = bm_prev_sig(st, kind, w);
                if pv < 0 {
                    return Ctx::Expr;
                }
                let pw = pv as usize;
                let pk = kind_at(kind, pw);
                if pk >= OP_KIND_BASE {
                    return if matches!(*src.add(pw), b'{' | b';' | b'}') {
                        enclosing_context(t, src, st, opch, kind, n, w, false, hops)
                    } else {
                        Ctx::Expr
                    };
                }
                if (pk == IDENT || pk == IDENT_ESC) && !prop_name(src, pw) {
                    if ident_is(src, pw, b"function") {
                        return Ctx::Type;
                    }
                    if word_is_any(src, pw, GENERATOR_STAR_WORDS) {
                        return enclosing_context(t, src, st, opch, kind, n, w, false, hops);
                    }
                }
                Ctx::Expr
            }
            b']' => {
                if mode != After::Bracket {
                    return Ctx::Expr;
                }
                match match_delim_back(src, st, kind, w, b'[', b']') {
                    Some(o) => ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Bracket,
                        hops + 1,
                    ),
                    None => Ctx::Expr,
                }
            }
            b':' => colon_context(t, src, st, opch, kind, n, w, hops),
            b'=' => eq_context(t, src, st, opch, kind, n, w, hops),
            b'(' => ctx_after_token(
                t,
                src,
                st,
                opch,
                kind,
                n,
                bm_prev_sig(st, kind, w),
                After::Paren,
                hops + 1,
            ),
            b'[' => ctx_after_token(
                t,
                src,
                st,
                opch,
                kind,
                n,
                bm_prev_sig(st, kind, w),
                After::Bracket,
                hops + 1,
            ),
            b'{' => {
                if mode == After::Head && brace_is_member_container(src, st, kind, w) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            b';' | b'}' => {
                if mode == After::Head {
                    enclosing_context(t, src, st, opch, kind, n, w, false, hops)
                } else {
                    Ctx::Expr
                }
            }
            b',' => enclosing_context(t, src, st, opch, kind, n, w, true, hops),
            b'<' => {
                let nx = *src.add(w + 1);
                if nx == b'=' || (nx == b'<' && !bm_get(st, w + 1)) {
                    Ctx::Expr
                } else {
                    enclosing_list_context(t, src, st, opch, kind, n, w, hops)
                }
            }
            b'>' => {
                let nx = *src.add(w + 1);
                if ((nx == b'>' || nx == b'=') && !bm_get(st, w + 1)) || mode == After::Paren {
                    Ctx::Expr
                } else {
                    Ctx::Type
                }
            }
            b'|' | b'&' => {
                if *src.add(w + 1) == c {
                    Ctx::Expr
                } else {
                    operand_head_context(t, src, st, opch, kind, n, w, false, hops)
                }
            }
            b'?' => {
                let nx = *src.add(w + 1);
                if nx == b'?' || nx == b'.' {
                    Ctx::Expr
                } else if extends_precedes_question(src, st, kind, w) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            _ => Ctx::Expr,
        };
    }
    Ctx::Expr
}

unsafe fn lt_head_is_operand(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    hops: u32,
) -> bool {
    let q = bm_prev_sig(st, kind, lt);
    if q < 0 {
        return true;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    let broke = lt_in_range(src, bm_next1(st, w + 1, n), lt);
    let asi_head = |t: &Tables| {
        broke && operand_head_context(t, src, st, opch, kind, n, lt, true, hops + 1) == Ctx::Type
    };
    if k == IDENT || k == IDENT_ESC {
        if !prop_name(src, w) && word_is_any(src, w, LT_OPERAND_WORDS) {
            return true;
        }
        if broke {
            return declarator_without_init(src, st, kind, w) || asi_head(t);
        }
        return ctx_for_lt(t, src, st, opch, kind, n, lt, hops + 1) == Ctx::Type;
    }
    if matches!(k, NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL) {
        return asi_head(t);
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b')' {
            if let Some(lp) = match_delim_back(src, st, kind, w, b'(', b')') {
                if paren_is_statement_head(src, st, kind, lp) {
                    return true;
                }
            }
            return asi_head(t);
        }
        if c == b']' {
            return asi_head(t);
        }
        if c == b'>' && !(w > 0 && *src.add(w - 1) == b'=') && broke {
            if let AngleMatch::Found(lt2) = angle_match_back(src, st, kind, w) {
                let p = bm_prev_sig(st, kind, lt2);
                if p >= 0 && matches!(kind_at(kind, p as usize), IDENT | IDENT_ESC) {
                    return asi_head(t);
                }
            }
            return true;
        }
        if c == b'.' {
            return *src.add(w + 1) == b'.' && *src.add(w + 2) == b'.';
        }
        if c == b'}' {
            return match match_delim_back(src, st, kind, w, b'{', b'}') {
                Some(o) => !brace_opens_value(t, src, st, kind, n, o, true, 0),
                None => true,
            };
        }
        if (c == b'+' || c == b'-') && (*src.add(w + 1) == c || (w > 0 && *src.add(w - 1) == c)) {
            let first = if *src.add(w + 1) == c { w } else { w - 1 };
            return !incdec_is_postfix(src, st, kind, n, first);
        }
        if c == b'!' && *src.add(w + 1) != b'=' {
            return !bang_is_postfix(src, st, kind, n, w);
        }
        return !(c == b'?' && *src.add(w + 1) == b'.');
    }
    false
}

unsafe fn ctx_for_lt(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    hops: u32,
) -> Ctx {
    if hops > CTX_HOPS {
        return Ctx::Type;
    }
    let q = bm_prev_sig(st, kind, lt);
    if q < 0 {
        return Ctx::Type;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if matches!(k, IDENT | IDENT_ESC | PRIV_IDENT | PRIV_IDENT_ESC) {
        if !prop_name(src, w) && ident_is(src, w, b"this") {
            let p = bm_prev_sig(st, kind, w);
            if p >= 0 {
                let pw = p as usize;
                let pk = kind_at(kind, pw);
                if pk == IDENT
                    && !prop_name(src, pw)
                    && word_is_any(src, pw, &[b"extends", b"implements"])
                {
                    return Ctx::Type;
                }
                if pk >= OP_KIND_BASE && *src.add(pw) == b',' && class_like_walk(src, st, kind, pw)
                {
                    return Ctx::Type;
                }
            }
            return Ctx::Expr;
        }
        if !prop_name(src, w) && word_is_any(src, w, LT_OPERAND_WORDS) {
            return Ctx::Type;
        }
        let Some(head) = chain_head(src, st, kind, w) else {
            return Ctx::Expr;
        };
        let hp = bm_prev_sig(st, kind, head);
        if hp >= 0
            && kind_at(kind, hp as usize) >= OP_KIND_BASE
            && *src.add(hp as usize) == b')'
            && member_start_before(src, st, kind, head, false)
        {
            return enclosing_context(t, src, st, opch, kind, n, head, false, hops + 1);
        }
        return ctx_after_token(t, src, st, opch, kind, n, hp, After::Head, hops + 1);
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b']' {
            if let Some(o) = match_delim_back(src, st, kind, w, b'[', b']') {
                if computed_key_at_member_start(src, st, kind, o) {
                    return enclosing_context(t, src, st, opch, kind, n, w, false, hops + 1);
                }
                if lt_in_range(src, w + 1, lt) {
                    return ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Bracket,
                        hops + 1,
                    );
                }
            }
            return Ctx::Expr;
        }
        if c == b'}' {
            return match match_delim_back(src, st, kind, w, b'{', b'}') {
                Some(o) if brace_opens_value(t, src, st, kind, n, o, true, 0) => Ctx::Expr,
                _ => Ctx::Type,
            };
        }
        if c == b'.' {
            let spread = (*src.add(w + 1) == b'.' && *src.add(w + 2) == b'.')
                || (w >= 2 && *src.add(w - 1) == b'.' && *src.add(w - 2) == b'.');
            return if spread { Ctx::Type } else { Ctx::Expr };
        }
        if c == b')' || (c == b'?' && *src.add(w + 1) == b'.') {
            return Ctx::Expr;
        }
        if (c == b'+' || c == b'-') && (*src.add(w + 1) == c || (w > 0 && *src.add(w - 1) == c)) {
            let first = if *src.add(w + 1) == c { w } else { w - 1 };
            return if incdec_is_postfix(src, st, kind, n, first) { Ctx::Expr } else { Ctx::Type };
        }
        if c == b'!' && *src.add(w + 1) != b'=' {
            return if bang_is_postfix(src, st, kind, n, w) { Ctx::Expr } else { Ctx::Type };
        }
        return Ctx::Type;
    }
    if matches!(k, STR | NUM | BIGINT) && member_start_before(src, st, kind, w, true) {
        return enclosing_context(t, src, st, opch, kind, n, w, false, hops + 1);
    }
    Ctx::Expr
}

const JSX_EXPR_WORDS: &[&[u8]] = &[
    b"return",
    b"throw",
    b"yield",
    b"await",
    b"typeof",
    b"void",
    b"delete",
    b"case",
    b"in",
    b"instanceof",
    b"of",
    b"else",
    b"do",
    b"new",
];
const JSX_TYPE_PAREN_WORDS: &[&[u8]] = &[
    b"keyof",
    b"readonly",
    b"infer",
    b"as",
    b"satisfies",
    b"is",
    b"asserts",
    b"unique",
    b"abstract",
    b"extends",
    b"implements",
];

unsafe fn paren_return_type_colon(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 || kind_at(kind, q as usize) < OP_KIND_BASE || *src.add(q as usize) != b')' {
        return false;
    }
    let Some(lp) = match_delim_back(src, st, kind, q as usize, b'(', b')') else {
        return false;
    };
    let h = bm_prev_sig(st, kind, lp);
    if h >= 0
        && kind_at(kind, h as usize) == IDENT
        && !prop_name(src, h as usize)
        && ident_is(src, h as usize, b"case")
    {
        return false;
    }
    !ternary_colon(t, src, st, opch, kind, n, colon)
}

unsafe fn index_signature_colon(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 || kind_at(kind, q as usize) < OP_KIND_BASE || *src.add(q as usize) != b']' {
        return false;
    }
    let Some(lb) = match_delim_back(src, st, kind, q as usize, b'[', b']') else {
        return false;
    };
    if !member_start_before(src, st, kind, lb, true) {
        return false;
    }
    let mut i = bm_next1(st, lb + 1, n);
    while i < n && matches!(*kind.add(i), WS | LCOM | BCOM) {
        i = bm_next1(st, i + 1, n);
    }
    if i >= n || kind_at(kind, i) != IDENT {
        return false;
    }
    let mut j = bm_next1(st, i + 1, n);
    while j < n && matches!(*kind.add(j), WS | LCOM | BCOM) {
        j = bm_next1(st, j + 1, n);
    }
    j < n && *kind.add(j) >= OP_KIND_BASE && *src.add(j) == b':'
}

unsafe fn arrow_after_paren_group(src: *const u8, st: *const u64, lp: usize, n: usize) -> bool {
    let lim = (lp + GT_SCAN_CAP).min(n);
    let Some(rp) = paren_close_fwd(src, st, lp, lim) else {
        return false;
    };
    let mut i = skip_ws_fwd(src, rp + 1, lim);
    if i + 1 < lim && *src.add(i) == b'=' && *src.add(i + 1) == b'>' {
        return true;
    }
    if i >= lim || *src.add(i) != b':' {
        return false;
    }
    let mut depth: i32 = 0;
    i += 1;
    while i < lim {
        if bm_get(st, i) {
            match *src.add(i) {
                b'(' | b'[' | b'{' | b'<' => depth += 1,
                b')' | b']' | b'}' | b'>' => {
                    if *src.add(i) == b'>' && i > 0 && *src.add(i - 1) == b'=' {
                        if depth == 0 {
                            return true;
                        }
                    } else {
                        depth -= 1;
                        if depth < 0 {
                            return false;
                        }
                    }
                }
                b';' => return false,
                b',' if depth == 0 => return false,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

pub(super) unsafe fn jsx_site_is_expression(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let mut pos = lt;
    let mut hops = 0;
    loop {
        hops += 1;
        if hops > CTX_HOPS {
            return false;
        }
        let q = bm_prev_sig(st, kind, pos);
        if q < 0 {
            return true;
        }
        let w = q as usize;
        let k = kind_at(kind, w);
        if k == IDENT || k == IDENT_ESC {
            if prop_name(src, w) {
                return false;
            }
            if ident_is(src, w, b"new") && *src.add(pos) == b'(' {
                return false;
            }
            return word_is_any(src, w, JSX_EXPR_WORDS);
        }
        if k < OP_KIND_BASE {
            return false;
        }
        let c = *src.add(w);
        match c {
            b'=' => {
                if *src.add(w + 1) == b'>' || *src.add(w + 1) == b'=' {
                    return true;
                }
                if type_alias_head(src, st, kind, w) {
                    return false;
                }
                return !matches!(
                    enclosing_opener(src, st, opch, kind, n, w, true, GT_SCAN_CAP),
                    Encl::Open(_, b'<')
                );
            }
            b';' | b'}' => return true,
            b')' | b']' | b'!' | b'~' | b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'.' => {
                return true;
            }
            b'?' => {
                let nx = *src.add(w + 1);
                if nx == b'?' || nx == b'.' {
                    return true;
                }
                return !extends_precedes_question(src, st, kind, w);
            }
            b':' => {
                if conditional_type_question(src, st, kind, w).is_some()
                    || paren_return_type_colon(t, src, st, opch, kind, n, w)
                    || index_signature_colon(src, st, kind, n, w)
                    || annotation_colon_is_declaration(t, src, st, kind, n, w, 0)
                {
                    return false;
                }
                if ternary_colon(t, src, st, opch, kind, n, w) {
                    return true;
                }
                match enclosing_opener(src, st, opch, kind, n, w, false, ENCLOSING_SCAN_CAP) {
                    Encl::Open(o, b'{') => {
                        let b = bm_prev_sig(st, kind, o);
                        if b < 0 {
                            return true;
                        }
                        let bw = b as usize;
                        let bk = kind_at(kind, bw);
                        if bk == IDENT || bk == IDENT_ESC {
                            return !prop_name(src, bw) && word_is_any(src, bw, JSX_EXPR_WORDS);
                        }
                        if bk < OP_KIND_BASE {
                            return false;
                        }
                        return match *src.add(bw) {
                            b'(' | b',' | b'[' | b'?' => true,
                            b'=' => {
                                *src.add(bw + 1) != b'>'
                                    && !type_alias_head(src, st, kind, bw)
                                    && !matches!(
                                        enclosing_opener(
                                            src,
                                            st,
                                            opch,
                                            kind,
                                            n,
                                            bw,
                                            true,
                                            GT_SCAN_CAP
                                        ),
                                        Encl::Open(_, b'<')
                                    )
                            }
                            b')' => !class_like_walk(src, st, kind, bw),
                            b';' | b'}' | b'{' => true,
                            _ => false,
                        };
                    }
                    Encl::Open(_, b'(') => return false,
                    Encl::Open(o, b'[') => {
                        pos = o + 1;
                        continue;
                    }
                    Encl::Top => return true,
                    _ => return false,
                }
            }
            b',' => match enclosing_opener(src, st, opch, kind, n, w, true, GT_SCAN_CAP) {
                Encl::Open(o, b'(') | Encl::Open(o, b'[') | Encl::Open(o, b'{') => {
                    pos = o + 1;
                    continue;
                }
                Encl::Top => return true,
                _ => return false,
            },
            b'(' | b'[' | b'{' => {
                if c == b'(' && arrow_after_paren_group(src, st, w, n) {
                    return false;
                }
                let b = bm_prev_sig(st, kind, w);
                if b < 0 {
                    return true;
                }
                let bw = b as usize;
                let bk = kind_at(kind, bw);
                if bk == IDENT || bk == IDENT_ESC {
                    if prop_name(src, bw) {
                        return true;
                    }
                    if c == b'(' && ident_is(src, bw, b"function") {
                        return false;
                    }
                    if word_is_any(src, bw, JSX_TYPE_PAREN_WORDS) {
                        return false;
                    }
                    if c == b'{' {
                        if word_is_any(src, bw, MEMBER_OPEN_WORDS)
                            || word_is_any(src, bw, BLOCK_OPEN_WORDS)
                        {
                            return true;
                        }
                        return !class_like_walk(src, st, kind, bw);
                    }
                    return true;
                }
                if bk < OP_KIND_BASE {
                    return true;
                }
                match *src.add(bw) {
                    b')' | b']' => return c != b'{' || !class_like_walk(src, st, kind, bw),
                    b'*' if c == b'(' => {
                        let f = bm_prev_sig(st, kind, bw);
                        return !(f >= 0
                            && kind_at(kind, f as usize) == IDENT
                            && !prop_name(src, f as usize)
                            && ident_is(src, f as usize, b"function"));
                    }
                    b':' | b'<' | b'|' | b'&' | b'?' => return false,
                    b'>' => {
                        return c != b'{'
                            && bw > 0
                            && *src.add(bw - 1) == b'='
                            && arrow_context(t, src, st, opch, kind, n, bw - 1, 0) == Ctx::Expr;
                    }
                    b'=' => {
                        if *src.add(bw + 1) == b'>' {
                            return true;
                        }
                        if type_alias_head(src, st, kind, bw) {
                            return false;
                        }
                        return !matches!(
                            enclosing_opener(src, st, opch, kind, n, bw, true, GT_SCAN_CAP),
                            Encl::Open(_, b'<')
                        );
                    }
                    b';' | b'}' if c == b'(' => {
                        return !enclosing_brace_is_type_literal(t, src, st, opch, kind, n, bw);
                    }
                    b'{' if c == b'(' => {
                        return !brace_is_type_literal(t, src, st, opch, kind, n, bw);
                    }
                    b'(' | b'[' | b',' | b'{' => {
                        pos = bw + 1;
                        continue;
                    }
                    _ => return true,
                }
            }
            _ => return false,
        }
    }
}

const JSX_BLOCK_WORDS: &[&[u8]] = &[b"else", b"do", b"try", b"finally", b"static", b"global"];
const TYPE_LITERAL_WORDS: &[&[u8]] = &[
    b"as",
    b"satisfies",
    b"extends",
    b"implements",
    b"keyof",
    b"readonly",
    b"is",
    b"asserts",
    b"in",
];

unsafe fn brace_is_type_literal(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b'=' => {
                if *src.add(w + 1) == b'>' {
                    arrow_context(t, src, st, opch, kind, n, w, 0) == Ctx::Type
                } else {
                    type_alias_head(src, st, kind, w)
                        || matches!(
                            enclosing_opener(src, st, opch, kind, n, w, true, GT_SCAN_CAP),
                            Encl::Open(_, b'<')
                        )
                }
            }
            b'>' => {
                if w > 0 && *src.add(w - 1) == b'=' {
                    arrow_context(t, src, st, opch, kind, n, w - 1, 0) == Ctx::Type
                } else {
                    class_like_walk(src, st, kind, w)
                }
            }
            b':' => colon_context(t, src, st, opch, kind, n, w, 0) == Ctx::Type,
            b')' => class_like_walk(src, st, kind, w),
            b'{' | b';' | b'}' | b'.' => false,
            _ => true,
        };
    }
    if k == IDENT {
        if prop_name(src, w) || word_is_any(src, w, JSX_BLOCK_WORDS) {
            return false;
        }
        if word_is_any(src, w, TYPE_LITERAL_WORDS) {
            return true;
        }
        return class_like_walk(src, st, kind, w);
    }
    false
}

unsafe fn enclosing_brace_is_type_literal(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
) -> bool {
    match enclosing_opener(src, st, opch, kind, n, from, false, ENCLOSING_SCAN_CAP) {
        Encl::Open(o, b'{') => brace_is_type_literal(t, src, st, opch, kind, n, o),
        Encl::Capped => true,
        _ => false,
    }
}

pub(super) unsafe fn type_parameter_list_head(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, lt);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b'*' => {
                let p = bm_prev_sig(st, kind, w);
                p >= 0
                    && kind_at(kind, p as usize) == IDENT
                    && !prop_name(src, p as usize)
                    && ident_is(src, p as usize, b"function")
            }
            b'{' => brace_is_type_literal(t, src, st, opch, kind, n, w),
            b';' | b',' => enclosing_brace_is_type_literal(t, src, st, opch, kind, n, w),
            b'}' => match_delim_back(src, st, kind, w, b'{', b'}')
                .is_some_and(|o| brace_is_type_literal(t, src, st, opch, kind, n, o)),
            b')' | b']' | b'>' => {
                lt_in_range(src, w + 1, lt)
                    && enclosing_brace_is_type_literal(t, src, st, opch, kind, n, w)
            }
            _ => false,
        };
    }
    if k == IDENT {
        if prop_name(src, w) {
            return false;
        }
        if ident_is(src, w, b"function") {
            return true;
        }
        if ident_is(src, w, b"new") {
            return type_parameter_list_head(t, src, st, opch, kind, n, w);
        }
        if member_start_before(src, st, kind, w, true) {
            return match enclosing_opener(src, st, opch, kind, n, w, false, ENCLOSING_SCAN_CAP) {
                Encl::Open(o, b'{') => {
                    brace_is_type_literal(t, src, st, opch, kind, n, o)
                        || brace_opens_object_literal(t, src, st, kind, n, o, true, 0)
                }
                Encl::Capped => true,
                _ => false,
            };
        }
        return lt_in_range(src, bm_next1(st, w + 1, n), lt)
            && enclosing_brace_is_type_literal(t, src, st, opch, kind, n, w);
    }
    matches!(k, NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL)
        && lt_in_range(src, bm_next1(st, w + 1, n), lt)
        && enclosing_brace_is_type_literal(t, src, st, opch, kind, n, w)
}

/// Forward angle match: from `i` at `depth`, the `>` that brings it to 0, or
/// `None` on an unmatched closer, a `;` outside every bracket, or the cap.
/// Same gating as [`gt_run_closes_type_args`] — `opch & st` for angles, `st`
/// for the bracket counters — and the same balance requirement at the close.
unsafe fn angle_close_fwd(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    i: usize,
    lim: usize,
    depth: i32,
) -> Option<usize> {
    angle_close_fwd_capped(src, st, opch, kind, i, lim, depth).0
}

unsafe fn angle_close_fwd_capped(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    mut i: usize,
    lim: usize,
    mut depth: i32,
) -> (Option<usize>, bool) {
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    while i < lim {
        let c = *src.add(i);
        if GT_SCAN_DELIM[c as usize] && bm_get(st, i) {
            let op = bm_get(opch, i);
            match c {
                b'<' => {
                    if op && *src.add(i + 1) != b'=' {
                        depth += 1;
                    }
                }
                b'>' => {
                    if op && !(i > 0 && *src.add(i - 1) == b'=') {
                        depth -= 1;
                        if depth == 0 {
                            return ((par == 0 && brk == 0 && brc == 0).then_some(i), false);
                        }
                    }
                }
                b'(' => par += 1,
                b')' => {
                    par -= 1;
                    if par < 0 {
                        return (None, false);
                    }
                }
                b'[' => brk += 1,
                b']' => {
                    brk -= 1;
                    if brk < 0 {
                        return (None, false);
                    }
                }
                b'{' => brc += 1,
                b'}' => {
                    let kk = kind_at(kind, i);
                    if kk != TMPL_MIDDLE && kk != TMPL_TAIL {
                        brc -= 1;
                        if brc < 0 {
                            return (None, false);
                        }
                    }
                }
                _ => {
                    if par == 0 && brk == 0 && brc == 0 {
                        return (None, false); // `;`
                    }
                }
            }
        }
        i += 1;
    }
    (None, true)
}

/// The `)` matching the `(` at `i`, or `None` past `lim`.
unsafe fn paren_close_fwd(
    src: *const u8,
    st: *const u64,
    mut i: usize,
    lim: usize,
) -> Option<usize> {
    let mut d: i32 = 0;
    while i < lim {
        if bm_get(st, i) {
            match *src.add(i) {
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

#[inline]
unsafe fn skip_ws_fwd(src: *const u8, mut i: usize, lim: usize) -> usize {
    while i < lim && is_ws(*src.add(i)) {
        i += 1;
    }
    i
}

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
unsafe fn lt_run_opens_type_args(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
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
    let hc = *src.add(head);
    if !(is_id_start(hc) || (hc == b'\\' && *src.add(head + 1) == b'u')) {
        return false;
    }
    let Some(gt) = angle_close_fwd(src, st, opch, kind, lt + 2, lim, 1) else {
        return false;
    };
    let lp = skip_ws_fwd(src, gt + 1, lim);
    if lp >= lim || *src.add(lp) != b'(' {
        return false;
    }
    let Some(rp) = paren_close_fwd(src, st, lp, lim) else {
        return false;
    };
    let ar = skip_ws_fwd(src, rp + 1, lim);
    if ar + 1 >= lim || *src.add(ar) != b'=' || *src.add(ar + 1) != b'>' {
        return false;
    }
    // The outer list must close too, or this was a comparison against a
    // generic arrow function and the parser would have backtracked as well.
    angle_close_fwd(src, st, opch, kind, ar + 2, lim, 1).is_some()
}

/// `coalesce` entry for a `<<` run: true when the two `<` must stay separate
/// tokens. Cold — `<<` is shift-left everywhere except this one shape.
#[inline(never)]
pub(super) unsafe fn lt_run_split(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
) -> bool {
    lt_run_opens_type_args(src, st, opch, kind, n, p)
}

/// `coalesce` entry for a `>`-run (`>>`, `>>>`, and a `>` glued to `=`): the
/// number of leading `>` bytes to leave unfused, or 0 to fuse as today. Cold
/// by construction — `>>` occurs once per ~110 KB of production TypeScript.
///
/// A balanced region is necessary but not sufficient: TypeScript's
/// speculative type-argument parse in expression position also needs the
/// region to scan as types (`foo(a<b + 1, c<d >> (e))` is a shift) and the
/// token after the list to be one that may follow type arguments
/// (`foo(a<b, c<d >> e)` is a shift). A glued `=` makes the rescan yield a
/// compound operator, so `a<b>=c` fuses, while type context never rescans
/// and `var v: Foo<T>= 1` splits.
#[inline(never)]
pub(super) unsafe fn gt_run_split(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
    run: usize,
) -> usize {
    let mut g = 1usize;
    while g < run && *src.add(p + g) == b'>' {
        g += 1;
    }
    let mut closes = g;
    let head = loop {
        if let Some(h) = gt_run_closes_type_args(src, st, opch, kind, p, closes) {
            if closes == g {
                break h;
            }
            if ctx_for_lt(t, src, st, opch, kind, n, h, 0) == Ctx::Type {
                return closes;
            }
        }
        if closes == 1 {
            return 0;
        }
        closes -= 1;
    };
    if g < run && *src.add(p + g) == b'=' {
        return if ctx_for_lt(t, src, st, opch, kind, n, head, 0) == Ctx::Type { g } else { 0 };
    }
    match gt_follower(src, n, p + g) {
        Follow::Split => {
            if type_list_legal(t, src, st, kind, head + 1, p) {
                if type_list_head_is_relational(t, src, st, opch, kind, n, head) {
                    return 0;
                }
                return g;
            }
        }
        Follow::Fuse => {
            return if lt_head_is_operand(t, src, st, opch, kind, n, head, 0) { g } else { 0 };
        }
        Follow::Ctx => {}
    }
    if ctx_for_lt(t, src, st, opch, kind, n, head, 0) == Ctx::Type { g } else { 0 }
}

unsafe fn type_list_head_is_relational(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    head: usize,
) -> bool {
    let hq = bm_prev_sig(st, kind, head);
    if hq < 0 {
        return false;
    }
    let hw = hq as usize;
    let hk = kind_at(kind, hw);
    if hk == IDENT && !prop_name(src, hw) && ident_is(src, hw, b"this") {
        let p = bm_prev_sig(st, kind, hw);
        if p < 0 {
            return false;
        }
        let pw = p as usize;
        let pk = kind_at(kind, pw);
        if pk == IDENT && !prop_name(src, pw) && word_is_any(src, pw, &[b"extends", b"implements"])
        {
            return false;
        }
        if pk >= OP_KIND_BASE && *src.add(pw) == b',' && class_like_walk(src, st, kind, pw) {
            return false;
        }
        return ctx_after_token(t, src, st, opch, kind, n, p, After::Head, 0) == Ctx::Type;
    }
    if hk >= OP_KIND_BASE && *src.add(hw) == b'>' && !(hw > 0 && *src.add(hw - 1) == b'=') {
        if let AngleMatch::Found(lt2) = angle_match_back(src, st, kind, hw) {
            return as_gated_type_ref(t, src, st, kind, n, lt2);
        }
    }
    hk == IDENT
        && !prop_name(src, hw)
        && as_type_operand(src, st, kind, hw)
        && lt_in_range(src, bm_next1(st, hw + 1, n), head)
}

unsafe fn this_type_head_before(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lt: usize,
) -> bool {
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 || kind_at(kind, b as usize) != IDENT || prop_name(src, b as usize) {
        return false;
    }
    if !ident_is(src, b as usize, b"this") {
        return false;
    }
    let p = bm_prev_sig(st, kind, b as usize);
    if p < 0 {
        return false;
    }
    let pw = p as usize;
    let pk = kind_at(kind, pw);
    if pk >= OP_KIND_BASE {
        return matches!(*src.add(pw), b'|' | b'&') && *src.add(pw + 1) != *src.add(pw);
    }
    pk == IDENT && !prop_name(src, pw) && word_is_any(src, pw, &[b"as", b"satisfies"])
}

#[inline(never)]
unsafe fn type_args_head_before(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lt: usize,
) -> bool {
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 {
        return false;
    }
    let bw = b as usize;
    let bk = *kind.add(bw);
    if bk == IDENT || bk == IDENT_ESC {
        return prop_name(src, bw) || !word_is_any(src, bw, LT_OPERAND_WORDS);
    }
    bk >= OP_KIND_BASE && matches!(*src.add(bw), b')' | b']')
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
        if k == STR {
            let e = bm_next1(st, qi + 1, n);
            if !lt_in_range(src, e, p) {
                return false;
            }
            return module_specifier_asi(src, st, kind, qi)
                || (ts && type_annotation_asi(t, src, st, kind, n, qi, 0));
        }
        if k == TMPL_NOSUB || k == TMPL_TAIL {
            return ts
                && lt_in_range(src, bm_next1(st, qi + 1, n), p)
                && type_annotation_asi(t, src, st, kind, n, qi, 0);
        }
        if k == REGEX || k == PRIV_IDENT || k == PRIV_IDENT_ESC || k == JEND || k == JSX_LT {
            return false;
        }
        if k == TMPL_HEAD || k == TMPL_MIDDLE {
            return true;
        }
        if k == NUM {
            let we = bm_next0(word, qi, n);
            let de = bm_next0(digit, qi, n);
            if de >= we {
                return ts
                    && lt_in_range(src, we, p)
                    && type_annotation_asi(t, src, st, kind, n, qi, 0);
            }
            let a = glue_anchor(src, st, qi);
            if prev_regex_sim(t, src, n, a, p, anchor_seed_tail(t, src, st, kind, n, a)) {
                return true;
            }
            return ts
                && lt_in_range(src, we, p)
                && type_annotation_asi(t, src, st, kind, n, qi, 0);
        }
        if k == IDENT || k == IDENT_ESC {
            let e = bm_next1(st, qi + 1, n);
            if prop_name(src, qi) {
                return ts
                    && lt_in_range(src, e, p)
                    && type_annotation_asi(t, src, st, kind, n, qi, 0);
            }
            let (ws, we) = match word_run_end(src, qi, e) {
                RunEnd::Seg(ss, se, _) => (ss, se),
                RunEnd::Blank(_) => {
                    q = bm_prev1(st, qi);
                    continue;
                }
            };
            if k == IDENT && t.is_regex_keyword(src.add(ws), we - ws) {
                if ts
                    && we - ws == 4
                    && ws == qi
                    && ident_is(src, ws, b"void")
                    && as_type_operand(src, st, kind, qi)
                {
                    return false;
                }
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
            if k == IDENT && we - ws == 2 && *src.add(ws) == b'o' && *src.add(ws + 1) == b'f' {
                return of_is_forof_keyword(t, src, st, kind, n, qi);
            }
            if lt_in_range(src, e, p) {
                if label_after_restricted(src, st, kind, n, qi) {
                    return true;
                }
                if declarator_without_init(src, st, kind, qi) {
                    return true;
                }
                if ts && type_annotation_asi(t, src, st, kind, n, qi, 0) {
                    return true;
                }
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
                if brace_close_is_regex(t, src, st, kind, n, qi, ts) {
                    return true;
                }
                return ts
                    && lt_in_range(src, qi + 1, p)
                    && type_annotation_asi(t, src, st, kind, n, qi, 0);
            }
            if ch == b')' {
                if paren_close_is_regex(src, st, kind, qi) {
                    return true;
                }
                return ts
                    && lt_in_range(src, qi + 1, p)
                    && (type_annotation_asi(t, src, st, kind, n, qi, 0)
                        || signature_return_type(src, st, kind, qi));
            }
            if ts && ch == b'>' && !(qi > 0 && *src.add(qi - 1) == b'=') {
                if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, qi) {
                    if this_type_head_before(src, st, kind, lt) {
                        return true;
                    }
                    if as_gated_type_ref(t, src, st, kind, n, lt) {
                        return false;
                    }
                    if type_args_head_before(src, st, kind, lt) {
                        let hb = bm_prev_sig(st, kind, lt) as usize;
                        if lt_in_range(src, bm_next1(st, hb + 1, n), lt)
                            && type_annotation_asi(t, src, st, kind, n, hb, 0)
                        {
                            return true;
                        }
                        return lt_in_range(src, qi + 1, p)
                            && type_annotation_asi(t, src, st, kind, n, qi, 0);
                    }
                }
                return true;
            }
            if ch == b']' {
                return ts
                    && lt_in_range(src, qi + 1, p)
                    && type_annotation_asi(t, src, st, kind, n, qi, 0);
            }
            return true;
        }
        return true;
    }
    true
}
#[cfg(test)]
mod tests {
    use crate::error::diag_code;
    use crate::options::default_options;
    use crate::token::TokenKind;
    use crate::{Lexer, PAD};

    fn kinds_of(code: &str, ts: bool, jsx: bool) -> Vec<TokenKind> {
        let mut buf = code.as_bytes().to_vec();
        let n = buf.len();
        buf.resize(n + PAD, 0);
        let mut opts = default_options();
        opts.ts = ts;
        opts.jsx = jsx;
        let mut lx = Lexer::new();
        let count = lx.lex(&buf, n, opts);
        lx.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
    }

    fn first_slash_kind(code: &str, ts: bool) -> Option<TokenKind> {
        let mut buf = code.as_bytes().to_vec();
        let n = buf.len();
        buf.resize(n + PAD, 0);
        let mut opts = default_options();
        opts.ts = ts;
        let mut lx = Lexer::new();
        let count = lx.lex(&buf, n, opts);
        let kinds = lx.kinds()[..count].to_vec();
        (0..count)
            .filter(|&i| !kinds[i].is_trivia() && buf[lx.spans[i].start as usize] == b'/')
            .map(|i| kinds[i])
            .next()
    }

    #[track_caller]
    fn regex(code: &str, ts: bool) {
        let ks = kinds_of(code, ts, false);
        assert_eq!(
            first_slash_kind(code, ts),
            Some(TokenKind::RegExp),
            "expected the first `/` to open a regex in {code:?}: kinds {ks:?}"
        );
    }

    #[track_caller]
    fn division(code: &str, ts: bool) {
        let ks = kinds_of(code, ts, false);
        assert!(!ks.contains(&TokenKind::RegExp), "expected division in {code:?}: kinds {ks:?}");
        assert!(ks.contains(&TokenKind::Slash), "expected a `/` in {code:?}: kinds {ks:?}");
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
            let re = ks.iter().filter(|&&kk| kk == TokenKind::RegExp).count();
            let sl = ks.iter().filter(|&&kk| kk == TokenKind::Slash).count();
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
        assert!(!ks.contains(&TokenKind::RegExp), "x! / 2 must stay division: {ks:?}");
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
        assert!(ks.contains(&TokenKind::RegExp), "TS class decl with type params: {ks:?}");
        let ks = kinds_of("x = f < T > {} / re / g;", true, false);
        assert!(!ks.contains(&TokenKind::RegExp), "TS relational re-read must divide: {ks:?}");
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
        assert!(ks.contains(&TokenKind::JsxLt), "export default <App/> must open JSX: {ks:?}");
        let ks = jsx("if (x) <App/>;");
        assert!(ks.contains(&TokenKind::JsxLt), "if (x) <App/> must open JSX: {ks:?}");
        let ks = jsx("x = a < b;");
        assert!(!ks.contains(&TokenKind::JsxLt), "a < b is a comparison: {ks:?}");
        let ks = jsx("f(x) < y;");
        assert!(!ks.contains(&TokenKind::JsxLt), "f(x) < y is a comparison: {ks:?}");
        let ks = jsx("a++ < b;");
        assert!(!ks.contains(&TokenKind::JsxLt), "a++ < b is a comparison: {ks:?}");
        let ks = jsx("x = a > {} < b;");
        assert!(!ks.contains(&TokenKind::JsxLt), "a > {{}} < b is a comparison chain: {ks:?}");
    }

    #[test]
    fn jsx_replay_oracle() {
        let jsx = |code: &str| kinds_of(code, false, true);
        let ks = jsx("function* items(d) { for (const x of d) yield <li id={x}/>; }");
        assert!(ks.contains(&TokenKind::JsxLt), "yielded JSX element must frame: {ks:?}");
        let ks = jsx("var await = 1, max = 10;\nif (await < max) done();");
        assert!(!ks.contains(&TokenKind::JsxLt), "await < max is a comparison: {ks:?}");
        assert!(ks.contains(&TokenKind::Lt), "expected a plain `<`: {ks:?}");
        let ks = jsx("async function f() { return await <Spinner/>; }");
        assert!(ks.contains(&TokenKind::JsxLt), "awaited JSX element must frame: {ks:?}");
        let ks =
            jsx("var await = 1, g = 2;\nvar el = <a b={async () => await 1} c={await /2/g}/>;");
        assert!(!ks.contains(&TokenKind::RegExp), "container leak, expected division: {ks:?}");
    }

    #[test]
    fn ts_declarator_type_annotation_forces_asi() {
        regex("let x: T\n/re/g.exec(s);", true);
        regex("let x: string\n/re/.exec(s);", true);
        regex("let x: number\n/re/.exec(s);", true);
        regex("let x: A | B\n/re/.exec(s);", true);
        regex("let x: T[]\n/re/.exec(s);", true);
        regex("let x: (T)\n/re/.exec(s);", true);
        regex("let x: typeof y\n/re/.exec(s);", true);
        regex("let x: a.b.C\n/re/.exec(s);", true);
        regex("var x: T\n/re/.exec(s);", true);
        regex("let x: T\n/re/gimsuy.exec(s);", true);
        regex("let a = 1, b: T\n/re/.exec(s);", true);
    }

    #[test]
    fn ts_declarator_type_annotation_already_resolved() {
        regex("let x: Array<T>\n/re/.exec(s);", true);
        regex("let x: {a: T}\n/re/.exec(s);", true);
        regex("interface I { a: T }\n/re/.exec(s);", true);
    }

    #[test]
    fn ts_initialised_declarator_stays_division() {
        division("let x= T\n/re/g.exec(s);", true);
        division("let x: number = 1\n/re/g.exec(s);", true);
        division("let x: T = y\n/re/g.exec(s);", true);
        division("let a = b\n/hi/g.exec(c);", false);
        division("let a = b, c = d\n/hi/g.exec(e);", false);
    }

    #[test]
    fn ts_type_alias_forces_asi() {
        regex("type A = T\n/re/.exec(s);", true);
        regex("type A = B | C\n/re/.exec(s);", true);
        regex("declare function f(): T\n/re/.exec(s);", true);
    }

    #[test]
    fn declarator_without_initializer_forces_asi() {
        regex("let x\n/re/.exec(s);", false);
        regex("var a\n/re/.test(b);", false);
        regex("let x\n/re/.exec(s);", true);
        regex("let a, b\n/re/.test(c);", false);
        regex("let a = 1, b\n/re/.test(c);", false);
    }

    #[test]
    fn module_specifier_forces_asi() {
        regex("import y from 'y'\n/re/.exec(s);", false);
        regex("import 'y'\n/re/.exec(s);", false);
        regex("export * from 'y'\n/re/.exec(s);", false);
        regex("export {a} from 'y'\n/re/.exec(s);", false);
        division("x = 'y' / 2;", false);
        division("x = f('y') / 2;", false);
    }

    #[test]
    fn restricted_production_keywords_precede_regex() {
        regex("for(;;){ break\n/re/.test(b); }", false);
        regex("for(;;){ continue\n/re/.test(b); }", false);
        division("x.break / 2;", false);
        division("x.continue / 2;", false);
    }

    #[test]
    fn ts_return_type_keyword_name_is_not_operand() {
        regex("function f(): void {}\n/re/.test(x);", true);
        regex("function f(): void {} /re/.test(x);", true);
        division("x = function(): void {} / 2;", true);
        division("x = void {} / 2;", false);
        division("x = typeof {} / 2;", false);
    }

    #[test]
    fn ts_composite_type_forms_force_asi() {
        regex("let x: A extends B ? C : D\n/re/.exec(s);", true);
        regex("let x: (a: T) => U\n/re/.exec(s);", true);
        regex("let x: 'lit'\n/re/.exec(s);", true);
        regex("let x: [A, B]\n/re/.exec(s);", true);
        regex("let x: readonly A[]\n/re/.exec(s);", true);
        regex("let x: keyof T\n/re/.exec(s);", true);
        regex("let x: Map<A, B<C>>\n/re/.exec(s);", true);
        regex("let x: T | null\n/re/.exec(s);", true);
        regex("declare const x: T\n/re/.exec(s);", true);
        regex("export const x: T\n/re/.exec(s);", true);
    }

    #[test]
    fn ts_pathological_type_annotation_forces_asi() {
        regex(
            r#"let x: {
  readonly [K in keyof T as `get${Capitalize<K & string>}`]-?:
    T[K] extends infer U extends (...a: [x: string, ...r: number[]]) => infer R
      ? new (m: typeof import("./m").default) => U extends { a: infer V } ? V : R
      : { [k: string]: readonly string[] }
} & (abstract new () => void) & T
/re/g.exec(s)"#,
            true,
        );
        regex("let x: {[K in keyof T as `g${K & string}`]-?: T[K]}\n/re/g.exec(s);", true);
        regex("let x: T extends infer U extends F ? A : B\n/re/g.exec(s);", true);
        regex("let x: typeof import(\"./m\").default\n/re/g.exec(s);", true);
        regex("let x: abstract new () => void\n/re/g.exec(s);", true);
        regex("let x: (...a: [x: string, ...r: number[]]) => R\n/re/g.exec(s);", true);
        division(
            r#"let x = y as {
  readonly [K in keyof T as `get${Capitalize<K & string>}`]-?: T[K]
} & (abstract new () => void) & T
/re/g.exec(s)"#,
            true,
        );
    }

    #[test]
    fn ts_type_literal_brace_forces_asi() {
        regex("type A = { a: T }\n/re/g.exec(s);", true);
        regex("type A = {}\n/re/g.exec(s);", true);
        regex("type A<T> = { a: T }\n/re/g.exec(s);", true);
        regex("type A = { [K in keyof T]: T[K] }\n/re/g.exec(s);", true);
        regex("type A = { (): T }\n/re/g.exec(s);", true);
        regex("let x: A | { a: T }\n/re/g.exec(s);", true);
        regex("let x: A & { a: T }\n/re/g.exec(s);", true);
        regex("let x: A extends B ? C : { a: T }\n/re/g.exec(s);", true);
        regex("let x: A extends B ? { a: T } : C\n/re/g.exec(s);", true);
        regex("declare function f(): A | { a: T }\n/re/g.exec(s);", true);
        division("x = {a: 1}\n/re/g.exec(s);", true);
        division("let x = {a: 1}\n/re/g.exec(s);", true);
        division("let x: T = {a: 1}\n/re/g.exec(s);", true);
        division("x = class {}\n/re/g.exec(s);", true);
        division("let v = {} as T\n/re/g.exec(s);", true);
        division("f({} / 2);", true);
    }

    #[test]
    fn ts_as_void_is_division() {
        division("let x = y as void\n/re/g.exec(s);", true);
        division("let x = y as void / 2;", true);
        division("let x = y satisfies void\n/re/g.exec(s);", true);
        division("let x: T = y as void\n/re/g.exec(s);", true);
        regex("x = void /re/.source;", true);
        regex("let x: void\n/re/g.exec(s);", true);
        regex("function f(): void {}\n/re/.test(x);", true);
        division("x.as / 2;", true);
    }

    #[test]
    fn tsx_generic_function_type_in_type_position() {
        let tsx = |code: &str| kinds_of(code, true, true);
        let type_params = |code: &str| {
            let ks = tsx(code);
            assert!(!ks.contains(&TokenKind::JsxLt), "{code:?} must not open JSX: {ks:?}");
        };
        let jsx = |code: &str| {
            let ks = tsx(code);
            assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        };

        type_params("let a: <T>(x: T) => T = null!;");
        type_params("let b: { f: <T>(x: T) => T } = null!;");
        type_params("type F = <T>(x: T) => T;");
        type_params("function g(): <T>(x: T) => T { return null!; }");
        type_params("let k: (f: <T>(x: T) => T) => void = null!;");
        type_params("interface I { m: <T>(x: T) => T }");
        type_params("class C { m: <T>(x: T) => T = null!; }");
        type_params("let n: { a: { b: <T>(x: T) => T } } = null!;");
        type_params("let h = <T,>(x: T) => x;");
        type_params("type A = { m: <T>(a: T) => T };");
        type_params("type A<U> = { m: <T>(a: T) => T };");
        type_params("let z: A | <T>(a: T) => T = null!;");
        type_params("declare function f(a: <T>(a: T) => T): void;");
        type_params("function f(a: <T>(a: T) => T) {}");
        type_params("const r = f<{ m: <T>(x: T) => T }>(0);");
        type_params("const r = g<<T>(x: T) => T>(0);");
        type_params("let w = h<{ a: { b: <T>(x: T) => T } }>(1);");
        type_params("var a = <T>(x: T) => x;");

        jsx("function Badge(): JSX.Element { return <em>(new)</em>; }");
        jsx("const Note = () => <Callout>(see docs)</Callout>;");
        jsx("const p = <code>/usr/bin</code>;");
        jsx("const q = <div>\n  // Not Comment\n</div>;");
        jsx("const r = <div>\n\t/*test*/\n</div>;");
        jsx("const s = <div>(x)</div>;");
        jsx("let o2 = { m: <Foo>(bar)</Foo> };");
        jsx("f({ a: <T>(x)</T> });");
        jsx("function g() { return f(c ? a : <T>(x)</T>); }");
        jsx("function g(cb) { cb(c ? a : <Foo>(y)</Foo>); }");
        jsx("function g() { return c ? a : <T>(x)</T>; }");
        jsx("const r = f<{ m: <T>(hi)</T> }.m > (0);");
        jsx("let u = <T>(a)</T>;");
        jsx("let u = <T\n>(a)</T\n>;");
        jsx("class Panel { footer = beta ? null : <Note>(stable)</Note>; }");
        jsx("function Row(icon = flag ? null : <Badge>(new)</Badge>) { return icon; }");
        jsx("let u = <T>(a)</ /*b*/ T>;");
        jsx("let u = <T>(a)</\u{a0}T>;");
        jsx("const a = <div\u{a0}id=\"x\">hi</div>;");
        jsx("const a = <Foo extends />;");
        jsx("const a = <Foo extends>x</Foo>;");
        type_params("interface Props { onRender?: <Item>(x: Item) => Item }");
        type_params("class C { m?: <T>(x: T) => T }");
        type_params("function f(a?: X, b: <T>(x: T) => T) {}");
        type_params("class C { a = 1;\n m: <T>(x: T) => T = null!; }");
        type_params("interface I<T> { m: <U>(x: U) => U }");
        type_params("class C<T> { m: <U>(x: U) => U = null!; }");
        type_params("function f<T>(cb: <U>(x: U) => U) {}");

        let ks = tsx("f(a << b, c);");
        assert!(!ks.contains(&TokenKind::JsxLt), "shift must not open JSX: {ks:?}");
        let ks = tsx("x <<= 1;");
        assert!(!ks.contains(&TokenKind::JsxLt), "shift-assign must not open JSX: {ks:?}");
        let ks = tsx("let v = a < b > (c);");
        assert!(!ks.contains(&TokenKind::JsxLt), "comparison chain: {ks:?}");
    }

    #[test]
    fn jsx_candidate_scans_stay_bounded() {
        let tsx = |code: &str| kinds_of(code, true, true);
        let ks = tsx("let a: <T>(x: T) => T = null!;");
        assert!(!ks.contains(&TokenKind::JsxLt), "type position: {ks:?}");
        let far = format!("let a = <T>(x){}</T>;", " ".repeat(70 * 1024));
        let ks = tsx(&far);
        assert!(ks.contains(&TokenKind::JsxLt), "no arrow after the parameters: JSX");
        let many = "const r = f<{ m: <T>(x: T) => T }>(0);\n".repeat(5000);
        let ks = tsx(&many);
        assert!(!ks.contains(&TokenKind::JsxLt), "every site is a function type: {ks:?}");
    }

    /// `JSXText` is `SourceCharacter but not one of {, <, > or }`, so a `}`
    /// reached before any `{` or `<` proves the candidate is not an element.
    /// That decides the pair which shares the prefix `const r = f<{ m: <T>(`
    /// and has opposite answers, even when a same-named close tag exists
    /// elsewhere in the file for the probe to find.
    #[test]
    fn bare_brace_in_children_rules_out_jsx() {
        let tsx = |code: &str| kinds_of(code, true, true);
        let ks = tsx("const r = f<{ m: <T>(x: T) => T }>(0);\nconst z = <T>hi</T>;");
        assert_eq!(
            ks.iter().filter(|k| **k == TokenKind::JsxLt).count(),
            2,
            "only the element on line 2 is JSX: {ks:?}"
        );
        // The other half of the pair: children are `(hi)`, which is text, so
        // this stays JSX.
        let ks = tsx("const r = f<{ m: <T>(hi)</T> }.m > (0);");
        assert!(ks.contains(&TokenKind::JsxLt), "clean children stay JSX: {ks:?}");
        // A `{` container may legitimately contain `}` inside a string, so the
        // scan stops there and leaves the answer to the probe.
        let ks = tsx("const a = <div>{\"}\"}</div>;");
        assert!(ks.contains(&TokenKind::JsxLt), "container braces are not text: {ks:?}");
    }

    #[test]
    fn long_walks_resolve_exactly() {
        let spine = ["T"; 300].join(" & ");
        regex(&format!("let x: {spine}\n/re/g.exec(s);"), true);
        let body: String = (0..400).map(|i| format!("  a{i}: T;\n")).collect();
        regex(&format!("let x: {{\n{body}}} & U\n/re/g.exec(s);"), true);
        let args: String = (0..3000).map(|i| format!("a{i}, ")).collect();
        division(&format!("f({args}0) / 2"), false);
        regex(&format!("if ({args}0) /re/.test(s)"), false);
        let deep = 2000usize;
        let nested = format!("{}x{}", "[".repeat(deep), "]".repeat(deep));
        division(&format!("y = {nested} / 2"), false);
        let blocks = format!("{}x\n{}", "{".repeat(deep), "}\n/a/\n".repeat(deep));
        let ks = kinds_of(&blocks, false, false);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::RegExp).count(), deep, "{}", ks.len());
        let heritage: String = (0..100).map(|i| format!("I{i}, ")).collect();
        division(&format!("x = class extends A implements {heritage}J {{}} / 2"), true);
        let members: String = (0..2000).map(|i| format!("  m{i}(): Foo<Bar<T>> {{}}\n")).collect();
        let ks = kinds_of(&format!("class C {{\n{members}  m<T = A<B>>() {{}}\n}}"), true, false);
        assert!(
            !ks.iter().any(|k| matches!(k, TokenKind::RShift | TokenKind::URShift)),
            "{}",
            ks.len()
        );
    }

    #[test]
    fn jsx_tag_name_after_comment() {
        let tsx = |code: &str| kinds_of(code, true, true);
        for code in [
            "var x = </**/div></div>;",
            "var x = < /*a*/ ></ /*b*/>;",
            "var x = <//c\ndiv></div>;",
            "var x = < /*a*/ div /*b*/ />;",
            "var x = <></>;",
            "var x = < ></ >;",
            "var x = <></ /*b*/>;",
            "var x = <div></ /*b*/ div>;",
            "var x = <div></div /*b*/>;",
            "var x = <div></ //c\ndiv>;",
        ] {
            let ks = tsx(code);
            assert!(!ks.contains(&TokenKind::RegExp), "{code:?} must not invent a regex: {ks:?}");
            assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        }
        let ks = tsx("var x = a < b / c;");
        assert!(!ks.contains(&TokenKind::JsxLt), "comparison must stay relational: {ks:?}");
    }

    #[test]
    fn unicode_zero_width_space_is_whitespace() {
        let ks = kinds_of("x\u{200b}\ny", false, false);
        assert_eq!(
            ks.iter().filter(|&&k| k == TokenKind::Ident).count(),
            2,
            "U+200B must separate tokens: {ks:?}"
        );
        division("let a = 1;\u{200b}let b = 2 / 3;", false);
    }

    #[test]
    fn unicode_next_line_is_whitespace() {
        division("var a = 1;\u{85}var b = 2 / 3;", false);
        let ks = kinds_of("var a = 1;\u{85}var b = 2;", false, false);
        assert_eq!(
            ks.iter().filter(|&&k| k == TokenKind::KwVar).count(),
            2,
            "U+0085 must separate tokens: {ks:?}"
        );
    }

    #[test]
    fn jsx_self_close_allows_whitespace() {
        let tsx = |code: &str| kinds_of(code, false, true);
        for code in [
            "const a = <N x=\"v\"/>;\nconst b = 1;",
            "const a = <N x=\"v\" / >;\nconst b = 1;",
            "const a = <N x=\"v\" /\n>;\nconst b = 1;",
            "const a = <N x=\"v\"\t/\t>;\nconst b = 1;",
        ] {
            let ks = tsx(code);
            assert!(ks.contains(&TokenKind::JsxTagEnd), "{code:?} must self-close: kinds {ks:?}");
            assert!(
                ks.iter().filter(|&&k| k == TokenKind::KwConst).count() == 2,
                "{code:?} must not swallow the next statement: kinds {ks:?}"
            );
        }
        let ks = tsx("const a = <N x=\"v\" / y>;");
        assert!(!ks.contains(&TokenKind::JsxTagEnd), "lone slash: kinds {ks:?}");
    }

    #[test]
    fn unicode_line_separator_after_token() {
        division("a++\u{2028}/re/g.exec(s);", false);
        division("a++\u{2029}/re/g.exec(s);", false);
        division("a--\u{2028}/re/g.exec(s);", false);
        division("a++\u{00a0}/re/g.exec(s);", false);
        division("let x = y as void\u{2028}/re/g.exec(s);", true);
        division("let x = y as void\u{2029}/re/g.exec(s);", true);
        division("let x = y as void\u{00a0}/re/g.exec(s);", true);
        division("let x = y! as void\u{2028}/re/g.exec(s);", true);
        regex("a\u{2028}++/re/.lastIndex;", false);
        regex("a\u{2029}++/re/.lastIndex;", false);
        regex("return\u{00a0}++/re/.lastIndex;", false);
        regex("let x: T\u{2028}/re/g.exec(s);", true);
        regex("let x: {a: T}\u{2028}/re/g.exec(s);", true);
    }

    #[test]
    fn ts_non_null_before_as_is_division() {
        division("let x = y! as {a: T}\n/re/g.exec(s);", true);
        division("let x = y! as {}\n/re/g.exec(s);", true);
        division("let x = y! as Array<T>\n/re/g.exec(s);", true);
        division("let x = y! as a.b.C<D>\n/re/g.exec(s);", true);
        division("let x = y! as import('m').T<U>\n/re/g.exec(s);", true);
        division("let x = y!! as {a: T} / 2;", true);
        division("let x = f(y)! as {a: T} / 2;", true);
        division("let x = y! satisfies {a: T} / 2;", true);
        division("let x = !y\n/re/g.test(s);", true);
        regex("x = !/re/.test(s);", true);
        regex("if (!a) /re/.test(s);", true);
    }

    #[test]
    fn ts_import_type_chain_head() {
        division("let x = y as import('m').T<U>\n/re/g.exec(s);", true);
        division("let x = y as import('m').T<U> / 2;", true);
        division("let x = y satisfies import('m').T<U>\n/re/g.exec(s);", true);
        division("let x = y as import('m').a.b.T<U>\n/re/g.exec(s);", true);
        regex("let x: import('m').T<U>\n/re/g.exec(s);", true);
        regex("let x: typeof import('./m').default\n/re/g.exec(s);", true);
    }

    #[test]
    fn value_ternary_and_arrow_stay_division() {
        division("let x = c ? a : b\n/re/.exec(s);", true);
        division("let x = c ? a : b\n/re/.exec(s);", false);
        division("let f = (a) => b\n/re/.exec(s);", false);
        division("let x: T = c ? a : b\n/re/.exec(s);", true);
        division("export const x: T = 1\n/re/g.exec(s);", true);
    }

    #[test]
    fn tsx_asi_then_jsx_element() {
        let tsx = |code: &str| kinds_of(code, true, true);
        let ks = tsx("let v: T\n<div />;");
        assert!(ks.contains(&TokenKind::JsxLt), "type-annotation ASI must open JSX: {ks:?}");
        let ks = tsx("let v\n<div />;");
        assert!(
            ks.contains(&TokenKind::JsxLt),
            "uninitialised declarator ASI must open JSX: {ks:?}"
        );
        let ks = tsx("import R from 'r'\n<div />;");
        assert!(ks.contains(&TokenKind::JsxLt), "module-specifier ASI must open JSX: {ks:?}");
        let ks = tsx("const a = 1, div = 2;\nexport const r = a < div;");
        assert!(!ks.contains(&TokenKind::JsxLt), "a < div stays a comparison: {ks:?}");
    }

    #[test]
    fn ts_numeric_literal_type_forces_asi() {
        regex("let x: 1\n/re/.exec(s);", true);
        regex("let x: -1\n/re/.exec(s);", true);
        regex("let x: 1n\n/re/.exec(s);", true);
        regex("let x: -1n\n/re/.exec(s);", true);
        regex("let x: 0x1f\n/re/.exec(s);", true);
        regex("let x: 1e3\n/re/.exec(s);", true);
        regex("let a = 1, b: 2\n/re/.exec(s);", true);
        regex("type A = 1\n/re/.exec(s);", true);
        regex("declare function f(): 1\n/re/.exec(s);", true);
    }

    #[test]
    fn numeric_value_stays_division() {
        division("let x = 1\n/re/g.exec(s);", true);
        division("let x: number = 1\n/re/g.exec(s);", true);
        division("x = a - 1\n/re/g.exec(s);", true);
        division("let x: T = a - 1\n/re/g.exec(s);", true);
        division("x = 1\n/re/g.exec(s);", false);
        division("x = 1n\n/re/g.exec(s);", false);
        division("x = 0x1f\n/re/g.exec(s);", false);
    }

    #[test]
    fn ts_template_literal_type_forces_asi() {
        regex("let x: `lit`\n/re/.exec(s);", true);
        regex("let x: `p${string}s`\n/re/.exec(s);", true);
        regex("let x: `${number}`\n/re/.exec(s);", true);
        regex("let x: `a${'b'}c${number}d`\n/re/.exec(s);", true);
        regex("type A = `lit`\n/re/.exec(s);", true);
    }

    #[test]
    fn template_value_stays_division() {
        division("let x = `lit`\n/re/g.exec(s);", true);
        division("let x = `p${y}s`\n/re/g.exec(s);", true);
        division("let x: T = `lit`\n/re/g.exec(s);", true);
        division("x = `lit`\n/re/g.exec(s);", false);
        division("x = `p${y}s`\n/re/g.exec(s);", false);
    }

    #[test]
    fn ts_definite_assignment_forces_asi() {
        regex("let x!: T\n/re/.exec(s);", true);
        regex("let x!: T[]\n/re/.exec(s);", true);
        regex("let x!: 1\n/re/.exec(s);", true);
        regex("let a = 1, b!: T\n/re/.exec(s);", true);
        division("x! / 2;", true);
        division("x!\n/re/g.exec(s);", true);
    }

    #[test]
    fn restricted_production_label_precedes_regex() {
        regex("outer: for(;;){ break outer\n/re/.test(b); }", false);
        regex("outer: for(;;){ continue outer\n/re/.test(b); }", false);
        regex("outer: for(;;){ break outer\n/re/.test(b); }", true);
        division("outer: for(;;){ break\nouter\n/re/g.test(b); }", false);
        division("x.break / 2;", false);
        division("x.continue / 2;", false);
    }

    #[test]
    fn tsx_literal_type_asi_then_jsx_element() {
        let tsx = |code: &str| kinds_of(code, true, true);
        for code in [
            "let v: 1\n<div />;",
            "let v: -1\n<div />;",
            "let v: 1n\n<div />;",
            "let v: `lit`\n<div />;",
            "let v: `p${string}s`\n<div />;",
            "let v!: T\n<div />;",
        ] {
            let ks = tsx(code);
            assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        }
        let ks = kinds_of("outer: for(;;){ break outer\n<div />; }", false, true);
        assert!(ks.contains(&TokenKind::JsxLt), "labelled break ASI must open JSX: {ks:?}");
        let ks = tsx("const a = 1, div = 2;\nconst r = a - 1 < div;");
        assert!(!ks.contains(&TokenKind::JsxLt), "a - 1 < div stays a comparison: {ks:?}");
    }
    fn diag_codes_of(code: &str, ts: bool, jsx: bool) -> Vec<u16> {
        let mut buf = code.as_bytes().to_vec();
        let n = buf.len();
        buf.resize(n + PAD, 0);
        let mut opts = default_options();
        opts.ts = ts;
        opts.jsx = jsx;
        let mut lx = Lexer::new();
        lx.lex(&buf, n, opts);
        lx.lanes.diags.iter().map(|d| d.code).collect()
    }

    fn is_fused_gt(k: TokenKind) -> bool {
        matches!(
            k,
            TokenKind::RShift | TokenKind::URShift | TokenKind::RShiftEq | TokenKind::URShiftEq
        )
    }

    #[track_caller]
    fn gt_run_fused(code: &str) {
        let ks = kinds_of(code, true, false);
        assert!(ks.iter().any(|k| is_fused_gt(*k)), "{code:?} must fuse the `>` run: {ks:?}");
    }

    #[track_caller]
    fn gt_run_split(code: &str) {
        let ks = kinds_of(code, true, false);
        assert!(
            !ks.iter().any(|k| is_fused_gt(*k) || *k == TokenKind::Ge),
            "{code:?} must split the `>` run: {ks:?}"
        );
    }

    #[test]
    fn gt_run_in_expression_position_follows_the_type_argument_lookahead() {
        for follower in [
            "e",
            "1",
            "\"s\"",
            "[1]",
            "[]",
            "{}",
            "!e",
            "+e",
            "-e",
            "typeof e",
            "this",
            "new E()",
            "function () {}",
        ] {
            gt_run_fused(&format!("foo(a<b, c<d >> {follower});"));
        }
        gt_run_fused("foo(a<b, c<d, e<f >>> g);");
        gt_run_fused("x = a<b>>c;");
        gt_run_fused("x = a<b>>=c;");
        gt_run_fused("x = a<b>>>c;");
        for follower in [
            "(e)",
            "`t`",
            "== e",
            "& e",
            "| e",
            "* e",
            "/ e",
            "% e",
            "?? e",
            "as X",
            "instanceof e",
            ", e",
            ".y",
            "",
        ] {
            gt_run_split(&format!("foo(a<b, c<d >> {follower});"));
        }
        gt_run_split("foo(a<b, c<d >>\ne);");
        gt_run_split("foo(a<b, c<d >> /* c\n */ e);");
        gt_run_split("x = a<b<c>>;");
        gt_run_split("x = a<b<c>>.y;");
        gt_run_split("x = a<b<c>>(y);");
        gt_run_split("x = a<b<c>>`t`;");
    }

    #[test]
    fn gt_run_in_type_context_always_splits() {
        for code in [
            "let x: a<b<c>>[] = y;",
            "let x: a<b<c>>[1] = y;",
            "function f(): a<b<c>> { return null! }",
            "class X extends a<b<c>> {}",
            "class X extends a<b<c>> implements D {}",
            "interface I extends B<B<string>> {}",
            "class X<T extends A<B<C>>> {}",
            "class X<T = A<B>> {}",
            "function f<T = A<B>>() {}",
            "class X { m<T = A<B>>() {} }",
            "class X { a: T; m<T = A<B>>() {} }",
            "class X { static m<T = A<B>>() {} }",
            "let o = { m<T = A<B>>() {} };",
            "let x: Map<K, Foo<Bar<T>>[]> = y;",
            "x = f<K, Foo<Bar<T>>[]>(y);",
            "var x = <Array<Base>>[d1, d2];",
            "type T = A<B<C>> | D;",
            "type T = A<B<C>> extends D ? E : F;",
            "let y = x as A<B<C>> as D;",
            "let x: (A<B<C>>[]) = y;",
            "let x: A | Foo<Bar<T>>[] = y;",
            "let f: (x: T) => Foo<Bar<T>>[] = y;",
            "let x: T extends U ? Foo<Bar<T>>[] : never = y;",
            "let x: Obj[Foo<Bar<T>>[0]] = y;",
            "let x: [A, Foo<Bar<T>>[]] = y;",
            "let x: { a: T, b: Foo<Bar<T>>[] } = y;",
            "function f(x: A | Foo<Bar<T>>[]) {}",
            "interface I { a: Map<K, Foo<Bar<T>>[]> }",
            "class X extends React.Component<Props<T>> {}",
            "let x = <Foo<Bar<T>>>y;",
            "let x = <A<B<C<D>>>>y;",
            "let x = <Map<K, Set<V>>>y;",
            "let s = <Foo<Bar<T>>>\"str\";",
            "let n = <Foo<Bar<T>>>-1;",
            "let x = <Foo<Bar<T>>>[d1, d2];",
            "let x = <Foo<Bar<T>>>{ a: 1 };",
            "class C<T extends List<List<T>> > {}",
            "let x: Map<K, List<List<T>> > = y;",
            "declare function f<T>(p: T): Foo<Bar<T>>[]\n<Foo>hello world</Foo>;",
        ] {
            gt_run_split(code);
        }
    }

    #[test]
    fn gt_glued_to_eq_splits_only_in_type_context() {
        let ts = |code: &str| kinds_of(code, true, false);
        for code in [
            "var v : Foo<T>= 1;",
            "type X<T>= T;",
            "let v: Foo<Bar<T>>= 1;",
            "let v: Foo<Bar<Quux<T>>>= 1;",
            "let x: Map<A, 1>= y;",
        ] {
            let ks = ts(code);
            assert!(
                ks.contains(&TokenKind::Eq)
                    && !ks.iter().any(|k| is_fused_gt(*k) || *k == TokenKind::Ge),
                "{code:?} closes a type-argument list: {ks:?}"
            );
        }
        assert!(ts("x = a<b>=c;").contains(&TokenKind::Ge));
        assert!(ts("if (i>=0) {}").contains(&TokenKind::Ge));
        assert!(ts("x = y >= 1;").contains(&TokenKind::Ge));
        assert!(ts("foo(a<b, c<d>>= e);").contains(&TokenKind::RShiftEq));
    }

    #[test]
    fn gt_run_region_must_scan_as_a_type_list() {
        for code in [
            "foo(a<b + 1, c<d >> (e));",
            "foo(a<b * 2, c<d >> (e));",
            "foo(a<b(c), d<e >> (f));",
            "foo(a<b - 1, c<d >> (e));",
            "foo(a<b?.c, d<e >> (f));",
            "foo(a<b ? c : d, e<f >> (g));",
            "foo(a<b | c ? d : e, f<g >> (h));",
            "foo(a<b + 1, c<d<e >>> (f));",
            "foo(a<b, c<d + 1 >> (e));",
            "foo(a<b c, d<e >> (f));",
            "{ a<b + 1, c<d >> (e) }",
            "x; a<b + 1, c<d >> (e);",
        ] {
            gt_run_fused(code);
        }
        for code in [
            "foo(a<b.c, d<e >> (f));",
            "foo(a<b[0], c<d >> (e));",
            "foo(a<-1, c<d >> (e));",
            "foo(a<{ +readonly [K in T]+?: U }, c<d >> (e));",
            "foo(a<b < c, d<e >> (f));",
            "foo(a<`p${string}`, c<d >> (e));",
            "foo(a<typeof import('m').T, c<d >> (e));",
            "foo(a<new () => T, c<d >> (e));",
            "foo(a<[x?, ...y[]], c<d >> (e));",
            "foo(a<T extends U ? X : Y, c<d >> (e));",
            "foo(a<{ m(): T }, c<d >> (e));",
            "foo(a<(x: T) => U, c<d >> (e));",
            "foo(a<keyof T, c<d >> (e));",
            "foo(a<<T>(x: T) => T, c<d >> (e));",
        ] {
            gt_run_split(code);
        }
    }

    #[test]
    fn lt_lt_openers_close_with_a_split_gt_run() {
        for code in [
            "let e: Map<<T>(x: T) => T, Set<U>> = null!;",
            "const r = f<<T>(x: T) => Array<T>>(0);",
        ] {
            let ks = kinds_of(code, true, false);
            assert!(
                !ks.iter().any(|k| matches!(k, TokenKind::LShift | TokenKind::RShift)),
                "{code:?}: {ks:?}"
            );
        }
    }

    #[test]
    fn tsx_generic_function_type_is_decided_by_its_arrow() {
        let jsx_count = |code: &str| {
            kinds_of(code, true, true).iter().filter(|k| **k == TokenKind::JsxLt).count()
        };
        let tail = "\nconst z = <T>hi</T>;";
        for head in [
            "const r = f<{ m: <T>(x: T) => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: T) => { a: T } }>(0);",
            "const r = f<{ m: <T>(x: T) => T | Q<T> }>(0);",
            "const r = f<{ m: <T>(x: Array<T>) => T }>(0);",
            "const r = f<{ m: <T>(x: { a: T }) => T }>(0);",
            "const r = g(f<{ m: <T>(x: T) => Array<T> }>(0));",
            "const r = f<{ m: <T>(x: T /* ) */) => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: T)\n  => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: `a${T}b`) => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: `a${`b${T}`}c`) => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: \"(\") => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: ')') => Array<T> }>(0);",
            "const r = f<{ m: <T>(x: T /* ( */) => Array<T> }>(0);",
        ] {
            assert_eq!(jsx_count(&format!("{head}{tail}")), 2, "{head:?}");
        }
        for code in [
            "const r = f<{ m: <T>(x: T) => Array<T> }>(0);\nconst s = \"</T>\";",
            "const r = f<{ m: <T>(x: T) => Array<T> }>(0);\n// see </T>\n",
        ] {
            assert_eq!(jsx_count(code), 0, "{code:?}");
        }
        let big = format!(
            "const r = f<{{ m: <T>(x: T) => Array<T> }}>(0);\n{}",
            "const tail = 1;\n".repeat(6000)
        );
        assert_eq!(jsx_count(&big), 0, "a site more than 64 KiB from EOF");
        for code in [
            "const r = f<{ m: <T>(hi)</T> }.m > (0);",
            "const r = f<{ m: <T>(x: T) {'=>'} T</T> }.m > (0);",
            "let u = <T>(a)</T>;",
            "let u = <T>(a) /* => */</T>;",
        ] {
            assert_eq!(jsx_count(code), 2, "{code:?}");
        }
    }

    #[test]
    fn tsx_generic_arrow_in_expression_position_is_diagnosed() {
        let diagnosed = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(
                codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
                "{code:?} must be diagnosed: {codes:?}"
            );
            let ks = kinds_of(code, true, true);
            assert!(!ks.contains(&TokenKind::JsxLt), "{code:?} lexes as type parameters: {ks:?}");
        };
        let silent = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(
                !codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
                "{code:?} is valid: {codes:?}"
            );
        };
        for code in [
            "var a = <T>(x: T) => x;",
            "x = { m: <T>(x: T) => T };",
            "f(<T>(x: T) => x);",
            "function g() { return <T>(x: T) => x; }",
            "const a = [<T>(x: T) => x];",
            "x = cond ? <T>(x: T) => x : y;",
            "x = (<T>(x: T) => x);",
            "<T>(x: T) => x;",
            "var j = <T>() => {}</T>;",
            "let o = { m: <T>() => {}</T> };",
        ] {
            diagnosed(code);
        }
        for code in [
            "const r = f<{ m: <T>(x: T) => T }>(0);",
            "let a: <T>(x: T) => T = null!;",
            "function f(cb: <T>(x: T) => T) {}",
            "type F = <T>(x: T) => T;",
            "let x: A | <T>(x: T) => T = null!;",
            "const r = f<(<T>(x: T) => T)>(0);",
            "class C<T = <U>(x: U) => U> {}",
            "let x: [<T>(x: T) => T] = null!;",
            "let h = <T,>(x: T) => x;",
            "const r = f<{ m: <T>(hi)</T> }.m > (0);",
            "interface I { m: <T>(x: T) => T }",
            "let x: { f: <T>(x: T) => T } = null!;",
            "let w = h<{ a: { b: <T>(x: T) => T } }>(1);",
        ] {
            silent(code);
        }
    }
    #[test]
    fn named_function_expression_body_then_slash_is_division() {
        division("const x = function foo() {} /42/i", false);
        division("(function foo() {} /42/i)", false);
        division("!function fn() {} /42/i;", false);
        division("x = function* generation() {} /42/i;", false);
        division("void async function fn() {}\n/foo/g", false);
        division("x = async function* g() {}\n/foo/g", false);
        regex("function foo() {}\n/42/i", false);
        regex("function* generation() {}\n/42/i", false);
        regex("async function fn() {}\n/42/i", false);
    }

    #[test]
    fn postfix_increment_then_block_then_regex() {
        regex("x++\n{}\n/foo/", false);
        regex("x--\n{}\n/foo/", false);
        regex("a[0]++\n{}\n/foo/", false);
        division("x = y++ / 2", false);
    }

    #[test]
    fn gt_run_head_context_covers_heritage_members_and_spreads() {
        for code in [
            "interface X extends A, B<C<D>> {}",
            "class X extends A implements B, C<D<E>> {}",
            "interface X<T> extends A<T>, Pick<B<T>, Exclude<keyof C<T>, \"x\">> {\n  a: 1;\n}",
            "f(x => { const a: A<B<C>>[] = []; });",
            "f(function () {\n  const list: A<B<number>>[] = [];\n});",
            "f(() => { const {a}: A<B<C>>[] = x; });",
            "f(() => { const [a]: A<B<C>>[] = x; });",
            "const m = new Map<string, (t: string, k?: X) => Thenable<number>>();",
            "const f = promisify<(a: string, options?: ncp.Options) => Promise<void>>(util.promisify(ncp));",
            "class C { x?: Exclude<Record<U>>[]; }",
            "class C { static override 0.5?: Exclude<Record<U>>[]; }",
            "let x: [a?: X, ...Set<C<Foo>>[]];",
            "function f([a, b]: [obj?: X, ...Set<a.b.C<Foo>>[]]) {}",
            "let x: | K<Map<V<Foo>>>[] | X;",
            "declare function f(this: | K<Map<V<Foo>>>[][] | X, y?: Z);",
            "class C { #p<T = A<B<C>>>(x) {} }",
            "class C { static #p<T = A<B<C>>>(x) {} }",
            "class C { *m<T = A<B<C>>>(x) {} }",
            "class C { static async *m<T = A<B<C>>>(x) {} }",
            "class C { m() {} *g<T = A<B<C>>>(x) {} }",
            "class C { x = 1; *g<T = A<B<C>>>(x) {} }",
            "function *g<T = A<B<C>>>() {}",
            "x = function* <T = A<B<C>>>() {};",
            "let o = { *m<T = A<B<C>>>() {} };",
            "type T = { a(): [...T1, ...T2]\n  <const T1 extends A<B>, const T2 extends A<B>>(x: T1): T2 }",
            "export const f: {\n  <T2 extends ReadonlyArray<unknown>>(that: T2): <T1 extends ReadonlyArray<unknown>>(self: T1) => [...T1, ...T2]\n  <const T1 extends ReadonlyArray<unknown>, const T2 extends ReadonlyArray<unknown>>(self: T1, that: T2): [...T1, ...T2]\n} = dual(2, x);",
            "let a: T[]\n<A<B<C>>>(y);",
        ] {
            gt_run_split(code);
        }
        for code in [
            "let o = { ...a<b<c>>[0] };",
            "x = [...a<b<c>>[0]];",
            "f(...a<b<c>>[0]);",
            "x = a * b<c<d>>[0];",
            "x = a ** b<c<d>>[0];",
            "function* g() { yield* a<b<c>>[0]; }",
            "switch (x) { case {}: b<c<d>>[0]; }",
            "let o = { const: 1, [a]: b<c<d>>[0] };",
            "a, b<c<d>>[0];",
            "foo(a<(b ? c : d), e<f >> (g));",
        ] {
            gt_run_fused(code);
        }
        let ks = kinds_of("f(x => { const a: A<B<C>>[] = []; }, { a: b<c<d>>[0] });", true, false);
        assert_eq!(ks.iter().filter(|k| is_fused_gt(**k)).count(), 1, "{ks:?}");
    }

    #[test]
    fn gt_follower_after_a_line_break_still_rejects_the_shift_operands() {
        for code in [
            "let x = f<A<B<C>>>\n+1;",
            "let x = f<A<B<C>>>\n-1;",
            "let x = f<A<B<C>>>\n// c\n+1;",
            "let x = f<A<B<C>>> /* c\n */ +1;",
            "let x = f<A<B<C>>>\u{2028}+1;",
        ] {
            gt_run_fused(code);
        }
        let ks = kinds_of("let x = f<A<B<C>>>\n<div/>;", true, true);
        assert!(ks.contains(&TokenKind::URShift), "{ks:?}");
        for code in [
            "let x = f<A<B<C>>>\n++y;",
            "let x = f<A<B<C>>>\n--y;",
            "let x = f<A<B<C>>>\n[0];",
            "let x = f<A<B<C>>>\n(0);",
            "x = f<A<B<C>>>\n<< y;",
            "x = f<A<B<C>>>\n<= y;",
            "x = f<A<B<C>>>\n-= y;",
            "x = f<A<B<C>>>\n!y;",
            "x = a<b<c>> <= d;",
            "x = a<b<c>> << d;",
            "x = a<b<c>> += d;",
            "let x: Foo<Bar<T>>\n+1;",
            "type X = Foo<Bar<T>>\n+1;",
            "class C<T extends List<List<T>>\n> {}",
        ] {
            gt_run_split(code);
        }
        let ks = kinds_of("let x: Map<K, V<W<T>>>\n<div/>;", true, true);
        assert!(!ks.iter().any(|k| is_fused_gt(*k)), "{ks:?}");
    }

    #[test]
    fn gt_run_in_class_expression_heritage_lists() {
        for code in [
            "(class implements Thenable<V<number>>, Bar<Promise<-1, Thenable<intrinsic, void>>> {\n\n})\n",
            "[class implements Array, Set<Bar<never>> {}];",
            "f(a, class implements B, C<D<E>> {});",
            "x = c ? class implements A, V<T<U>> {} : d;",
            "try {} catch (e) { interface I extends A, V<T<U>> {} }",
            "(class <const T, U,> implements Exclude<$JSX<null>>, Exclude<Pick<this>> {});",
            "class <U, V extends Map<K<L>>> {}",
            "x = function <U extends A<B<C>>>() {};",
            "x = y?.[function <U extends A<B<C>>>(_: 'lit', n) {}];",
            "let o = { [k]<T = A<B<C>>>() {} };",
            "class C { [k]<T = A<B<C>>>() {} }",
            "class C { static [k]<T = A<B<C>>>() {} }",
            "class C { m() {} [k]<T = A<B<C>>>() {} }",
            "x = y as Foo | Bar<Baz<T>>[];",
            "x = y as Foo & Bar<Baz<T>>[];",
            "let x: typeof y | Bar<Baz<T>>[];",
        ] {
            gt_run_split(code);
        }
        for code in ["x = [a, [b]<c<d>>[0]];", "x = a, b<c<d>>[0];"] {
            gt_run_fused(code);
        }
    }

    #[test]
    fn gt_run_head_after_a_line_break_follows_asi() {
        for code in [
            "debugger\n<Map<Partial>>baz;",
            "for (;;) { break\n<Map<Partial>>baz; }",
            "for (;;) { continue\n<Map<Partial>>baz; }",
            "declare const x: bigint /*\n*/ <T<0x1f>>$;",
            "declare const 名前: bigint /*\n*/ <T<0x1f>>$;",
            "let x: bigint\n<Map<Partial>>baz;",
            "let x: Foo\n<Map<P>>baz;",
            "let x: Foo.Bar\n<Map<P>>baz;",
            "let x: Foo[]\n<Map<P>>baz;",
            "let x: Foo<Bar>\n<Map<P>>baz;",
            "let x: (Foo)\n<Map<P>>baz;",
            "let x: 1\n<Map<P>>baz;",
            "let x: \"lit\"\n<T<0x1f>>$;",
            "let x: `lit`\n<T<0x1f>>$;",
            "let x: true\n<T<0x1f>>$;",
            "let x: typeof y\n<Map<P>>baz;",
            "type X = Foo\n<Map<P>>baz;",
            "function f(): Foo\n<Map<P>>baz;",
            "if (x)\n<Map<P>>baz;",
            "if (x) <Map<P>>baz;",
            "while (x)\n<Map<P>>baz;",
            "let x = <Foo><Bar<Baz<T>>>y;",
            "let x = <Foo>\n<Bar<Baz<T>>>y;",
            "return\n<Map<P>>baz;",
        ] {
            gt_run_split(code);
        }
        for code in [
            "x = y as Foo\n<Map<P>>baz;",
            "x = y satisfies Foo\n<Map<P>>baz;",
            "x = y as Foo<Bar>\n<Map<P>>baz;",
            "let x = y\n<Map<P>>baz;",
            "let x = 1\n<Map<P>>baz;",
            "x\n<Map<P>>baz;",
            "foo()\n<Map<P>>baz;",
            "x = new Foo\n<Map<P>>baz;",
            "function f() { return y\n<Map<P>>baz; }",
            "throw y\n<Map<P>>baz;",
            "x = 1\n<Map<Partial>>baz;",
        ] {
            gt_run_fused(code);
        }
    }

    #[test]
    fn class_body_after_a_heritage_list_with_type_arguments_precedes_regex() {
        for code in [
            "class Foo implements A, V<T> {}\n/el/.exec(s);",
            "class Foo implements A, V<T<U>> {}\n/el/.exec(s);",
            "class Foo extends y implements React.FC, V<unique symbol, true> {\n\n}\n/el/.exec(s);",
            "interface I extends A, V<T> {}\n/el/.exec(s);",
        ] {
            regex(code, true);
        }
        division("x = class implements A, V<T> {} / 2;", true);
        division("x = class extends A implements B, V<T<U>> {} / 2;", true);
    }

    #[test]
    fn gt_run_third_tier_head_contexts() {
        for code in [
            "_(...<Promise<Bar, intrinsic>><React.FC<Bar<1n>>>[obj]);",
            "x = [...<Record<number, Promise<K<T<Record<K>>>>>>1e3] as Foo;",
            "export default <keyof Partial<typeof import('m'), 1e3>>arr;",
            "interface I extends A.B { a: Foo<Bar<T>>[] }",
            "interface y<K> extends React.FC<T<K>>, React.FC { 'a': K<Promise<Thenable<U, \"lit\">>>[Map<T>] }",
            "interface I { m?(): Promise<Array<X>>[] }",
            "class C { m?(): Promise<Array<X>>[] }",
            "interface I { [k]?(): Promise<Array<X>>[] }",
            "x = y as Foo<Bar<T>> - 1;",
            "x = y as Foo<Bar<T>> < z;",
            "x = y as Foo<Bar<T>> + 1;",
            "x = y satisfies Record<Bar<React.FC<Map<Pick<1>>>>> - Props;",
            "throw <A<B>>el! satisfies Array<V<object>> & Pick<Exclude<1n>> - --$[s];",
            "let o = { \"k\"<T = A<B<C>>>() {} };",
            "let o = { 1<T = A<B<C>>>() {} };",
            "class C { @dec() \"\"<T = A<B<C>>>(x) {} }",
            "class C { @dec m<T = A<B<C>>>(x) {} }",
            "class C { @dec [k]<T = A<B<C>>>(x) {} }",
            "let x: [Bar<U>?, ...Foo: U<Array<1>>[]];",
            "class C<V = <x>(bar: `p${Map<T<U>>}s` | a.b.C<T<Set<V>>>[]) => R> {}",
            "f<A extends B<C, D> ? E : F<G<H>>>(x);",
            "x = a as | $JSX<Partial<T<void>>>[] | B;",
            "declare const c: V extends R<A, B> ? E : Record<Map<V<Set<1>>>>[];",
            "declare const c: V<K<T>> extends R<A, T extends B ? C : D> ? E : Record<Map<V<Set<1>>>>[];",
            "class C extends (e) { x: Foo<Bar<T>>[]; }",
            "class C extends (e) { #p!: Foo<Bar<T>>[]; }",
            "class C extends (e) { @dec #p: Record<T<T<Array>>[]>; }",
            "let x: Foo.Bar[Baz<Qux<T>>];",
        ] {
            gt_run_split(code);
        }
        for code in [
            "x = a ? (b) : c<d<e>>[0];",
            "f(a, b ? (c) : d<e<f>>[0]);",
            "let o = { x: a ? (b) : c<d<e>>[0] };",
        ] {
            gt_run_fused(code);
        }
        let ts = |code: &str| kinds_of(code, true, false);
        for code in ["x = y satisfies T<string, React.FC>>>this.baz;", "x = y as Foo<Bar>>>z;"] {
            let ks = ts(code);
            assert_eq!(
                ks.iter().filter(|k| **k == TokenKind::RShift).count(),
                1,
                "{code:?}: {ks:?}"
            );
            assert!(!ks.contains(&TokenKind::URShift), "{code:?}: {ks:?}");
        }
        let ks = ts("x = y as Foo<Bar<T>>>>z;");
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 2, "{ks:?}");
    }

    #[test]
    fn this_heads_and_value_braces_before_a_run() {
        for code in [
            "class C implements this<Record, this>, K<Record<Pick<T>>> {}",
            "this<A<B>>(x);",
            "class C { m() { return this<A<B>>(x); } }",
            "interface I { a: T }\n<Map<P>>baz;",
            "function f() {}\n<Map<P>>baz;",
            "type X = A extends B<C, D extends E ? F : G> ? H : I<J<K>>[];",
        ] {
            gt_run_split(code);
        }
        for code in [
            "new Foo<this<this, a.b.C<symbol>>>(x);",
            "x = y as this<Foo<this>>[];",
            "(e)<true[a.b.C], this<Pick<boolean>>>(f);",
            "x = this<A<B>>[0];",
            "x = class extends c {}\n<React.FC<K>>ete;",
            "x = y as V | { get baz(): Partial<Array<V, object>> }\n<T<Thenable>>c;",
            "x = a - 1\n<Map<P>>baz;",
            "x = a * 'n'<b<c>>[0];",
            "x = {}\n<Map<P>>baz;",
            "x = class {}\n<Map<P>>baz;",
            "x = a > b ? c : d<e<f>>[0];",
        ] {
            gt_run_fused(code);
        }
        let ks = kinds_of("x = a >>ete<this<1n, Map>>(arr);", true, false);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 2, "{ks:?}");
        for code in [
            "let Foo: -1\n\n<React.FC<undefined, this>>a;",
            "let x: -1 | -2\n<Map<P>>baz;",
            "class C { @dec() #p<V = Set<Bar<T>>>() {} }",
            "class C { @dec() static m<T = A<B<C>>>() {} }",
            "class C { @dec() async *'n'<V = Promise<Promise<T>>>() {} }",
            "let o = { *'n'<T = A<B<C>>>() {} };",
        ] {
            gt_run_split(code);
        }
    }

    #[test]
    fn jsx_element_type_argument_runs_never_reach_coalesce() {
        for code in [
            "const d = <div<A<B>>>x</div>;",
            "const d = <div<Missing<AlsoMissing>>></div>;",
            "const e = <Link<React.ImgHTMLAttributes<HTMLElement>> element=\"img\" src=\"src\" />;",
            "const f = <Foo<A<B>> bar={a >> b} />;",
            "x = <Foo<A<B>>/>\ny = a >> (b);",
        ] {
            let ks = kinds_of(code, true, true);
            assert!(
                !ks.iter().any(|k| is_fused_gt(*k)) || code.contains(">> "),
                "{code:?}: {ks:?}"
            );
            assert!(!ks.contains(&TokenKind::LShift), "{code:?}: {ks:?}");
        }
        let ks = kinds_of("const d = <div<A<B>>>x</div>;", true, true);
        assert!(ks.contains(&TokenKind::JsxText), "{ks:?}");
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 3, "{ks:?}");
        let ks = kinds_of("const f = <Foo<A<B>> bar={a >> b} />;", true, true);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
        let ks = kinds_of("x = <Foo<A<B>>/>\ny = a >> (b);", true, true);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
    }

    #[test]
    fn tsx_element_type_arguments_with_generic_function_type() {
        let ks = kinds_of("<Component<<T>(v: T) => void> />", true, true);
        assert!(!ks.contains(&TokenKind::LShift), "{ks:?}");
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
        let ks = kinds_of("const a = <Box<(x: T) => Foo<T>> prop={1} />;", true, true);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
    }

    fn names_of(code: &str, ts: bool, jsx: bool) -> String {
        kinds_of(code, ts, jsx)
            .iter()
            .map(|k| match k {
                TokenKind::Number
                | TokenKind::Decimal
                | TokenKind::Float
                | TokenKind::Binary
                | TokenKind::Octal
                | TokenKind::Hex => "NUMBER",
                TokenKind::StringCooked => "STRING",
                TokenKind::IdentEscaped => "IDENT",
                _ => k.name(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn spans_of(code: &str, ts: bool, jsx: bool) -> Vec<(u32, u32)> {
        let mut buf = code.as_bytes().to_vec();
        let n = buf.len();
        buf.resize(n + PAD, 0);
        let mut opts = default_options();
        opts.ts = ts;
        opts.jsx = jsx;
        let mut lx = Lexer::new();
        let count = lx.lex(&buf, n, opts);
        let kinds = lx.kinds()[..count].to_vec();
        (0..count)
            .filter(|&i| !kinds[i].is_trivia())
            .map(|i| (lx.spans[i].start, lx.spans[i].end))
            .collect()
    }

    #[track_caller]
    fn stream(code: &str, ts: bool, jsx: bool, want: &str) {
        assert_eq!(names_of(code, ts, jsx), want, "{code:?}");
    }

    #[test]
    fn jsx_element_as_attribute_value_opens_a_nested_frame() {
        for (code, want) in [
            (
                "<App foo=<div>bar</div> />;",
                "JSX_LT IDENT IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END / JSX_TAG_END ;",
            ),
            (
                "const a = <Foo key=<T></T>>{x}\n  <b />\n</Foo>;",
                "const IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT > JSX_LT / IDENT JSX_TAG_END > { IDENT } JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "const a = <Foo key=<T/> other=\"x\">{x}</Foo>;",
                "const IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END IDENT = STRING > { IDENT } JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "x = <a b=<c>{d}</c>>{e}<f/></a>;",
                "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT > { IDENT } JSX_LT / IDENT JSX_TAG_END > { IDENT } JSX_LT IDENT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "const a = <Foo prop=<Bar><Baz /></Bar> />;",
                "const IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT > JSX_LT IDENT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END / JSX_TAG_END ;",
            ),
            (
                "x = <a b=<>t</> />;",
                "IDENT = JSX_LT IDENT IDENT = JSX_LT > JSX_TEXT JSX_LT / JSX_TAG_END / JSX_TAG_END ;",
            ),
            (
                "x = <a b= /* c */ <i/> d=\"1\"/>;",
                "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END IDENT = STRING / JSX_TAG_END ;",
            ),
            (
                "x = <a b=\n<i/>\n/>;",
                "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END / JSX_TAG_END ;",
            ),
            (
                "x = <a b=<c d=<e/>/>/>;",
                "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END / JSX_TAG_END / JSX_TAG_END ;",
            ),
            (
                "x = <a b=<c/>/>\ny = z / 2;",
                "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END / JSX_TAG_END IDENT = IDENT / NUMBER ;",
            ),
            (
                "x = <a b=\"<\" c={1 < 2} />;",
                "IDENT = JSX_LT IDENT IDENT = STRING IDENT = { NUMBER < NUMBER } / JSX_TAG_END ;",
            ),
        ] {
            stream(code, false, true, want);
        }
        stream(
            "const a = <Foo<T> key=<Bar<U> x=\"1\"/> />;",
            true,
            true,
            "const IDENT = JSX_LT IDENT < IDENT > IDENT = JSX_LT IDENT < IDENT > IDENT = STRING / JSX_TAG_END / JSX_TAG_END ;",
        );
    }

    #[test]
    fn jsx_child_element_name_after_trivia() {
        stream(
            "const a = <div>\n  x<br />\n  < br />\n  y\n</div>;",
            true,
            true,
            "const IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
        );
        for (code, want) in [
            (
                "x = <a>\n  <b/>\n  < c />\n  <d\n  />\n</a>;",
                "IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
            ),
            ("x = <a>< /a>;", "IDENT = JSX_LT IDENT > JSX_LT / IDENT JSX_TAG_END ;"),
            (
                "x = <a>< /* c */ b/></a>;",
                "IDENT = JSX_LT IDENT > JSX_LT IDENT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "x = <a>< ></ ></a>;",
                "IDENT = JSX_LT IDENT > JSX_LT > JSX_LT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "x = <a>t< b>u</ b ></a>;",
                "IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
            ),
        ] {
            stream(code, false, true, want);
        }
    }

    #[test]
    fn jsx_names_glue_every_hyphen() {
        for (code, want) in [
            ("x = <a--b/>;", "IDENT = JSX_LT IDENT / JSX_TAG_END ;"),
            ("y = <a-/>;", "IDENT = JSX_LT IDENT / JSX_TAG_END ;"),
            (
                "z = <a b-c-=\"1\" d--e=\"2\"/>;",
                "IDENT = JSX_LT IDENT IDENT = STRING IDENT = STRING / JSX_TAG_END ;",
            ),
            ("<div-\n    // comment\n/>;", "JSX_LT IDENT / JSX_TAG_END ;"),
            ("x = <a--b>t</a--b>;", "IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;"),
            (
                "x = <a b={c - d} e={-1}/>;",
                "IDENT = JSX_LT IDENT IDENT = { IDENT - IDENT } IDENT = { - NUMBER } / JSX_TAG_END ;",
            ),
            ("x = <a-b:c-d/>;", "IDENT = JSX_LT IDENT : IDENT / JSX_TAG_END ;"),
        ] {
            stream(code, false, true, want);
        }
        assert_eq!(spans_of("x = <a--b/>;", false, true)[3], (5, 9));
        assert_eq!(spans_of("y = <a-/>;", false, true)[3], (5, 7));
        stream(
            "<Foo<-1> data-x=\"1\"/>;",
            true,
            true,
            "JSX_LT IDENT < - NUMBER > IDENT = STRING / JSX_TAG_END ;",
        );
    }

    #[test]
    fn tsx_function_expression_type_parameters_after_star_or_async() {
        for (code, want) in [
            (
                "export const foo = function* <T>() {};",
                "export const IDENT = function * < IDENT > ( ) { } ;",
            ),
            (
                "const h = async function* <T>() {};",
                "const IDENT = async function * < IDENT > ( ) { } ;",
            ),
            ("z = async function <T>() {};", "IDENT = async function < IDENT > ( ) { } ;"),
            ("x = function <T>() {};", "IDENT = function < IDENT > ( ) { } ;"),
            ("x = function* f<T>() {};", "IDENT = function * IDENT < IDENT > ( ) { } ;"),
            (
                "f(function* <T>(x: T) {});",
                "IDENT ( function * < IDENT > ( IDENT : IDENT ) { } ) ;",
            ),
            (
                "x = function* <T>() { yield <T>(a)</T>; };",
                "IDENT = function * < IDENT > ( ) { yield JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END ; } ;",
            ),
            (
                "x = a * <T>(b)</T>;",
                "IDENT = IDENT * JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
            ),
        ] {
            stream(code, true, true, want);
        }
    }

    #[test]
    fn tsx_call_and_construct_signatures_are_type_parameters() {
        for (code, want) in [
            (
                "type X = { <T>(x: T): U; }",
                "type IDENT = { < IDENT > ( IDENT : IDENT ) : IDENT ; }",
            ),
            (
                "interface X { <T>(x: T): U; new <T>(x: T): U; m<T>(x: T): U; }",
                "interface IDENT { < IDENT > ( IDENT : IDENT ) : IDENT ; new < IDENT > ( IDENT : IDENT ) : IDENT ; IDENT < IDENT > ( IDENT : IDENT ) : IDENT ; }",
            ),
            ("let x: { <T>(x: T): U };", "let IDENT : { < IDENT > ( IDENT : IDENT ) : IDENT } ;"),
            (
                "type X = { a: T, <U>(x: U): V }",
                "type IDENT = { IDENT : IDENT , < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "type X = { a: T; <U>(x: U): V }",
                "type IDENT = { IDENT : IDENT ; < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "type X = { a: void\n <U>(x: U): V }",
                "type IDENT = { IDENT : void < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "type X = { a: {b: T}\n <U>(x: U): V }",
                "type IDENT = { IDENT : { IDENT : IDENT } < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "interface I extends A { <T>(x: T): U }",
                "interface IDENT extends IDENT { < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "type F = () => { <T>(x: T): U }",
                "type IDENT = ( ) => { < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "x = y as { <T>(x: T): U };",
                "IDENT = IDENT as { < IDENT > ( IDENT : IDENT ) : IDENT } ;",
            ),
            (
                "type X = [{ <T>(x: T): U }];",
                "type IDENT = [ { < IDENT > ( IDENT : IDENT ) : IDENT } ] ;",
            ),
            (
                "type X = A | { <T>(x: T): U };",
                "type IDENT = IDENT | { < IDENT > ( IDENT : IDENT ) : IDENT } ;",
            ),
            (
                "type X = { new <T>(x: T): U }",
                "type IDENT = { new < IDENT > ( IDENT : IDENT ) : IDENT }",
            ),
            (
                "declare function f(x: { <T>(x: T): U }): void;",
                "declare function IDENT ( IDENT : { < IDENT > ( IDENT : IDENT ) : IDENT } ) : void ;",
            ),
        ] {
            stream(code, true, true, want);
        }
        for code in [
            "if (a) { <T>(x)</T> }",
            "function f() { <T>(x)</T> }",
            "L: { <T>(x)</T> }",
            "switch (a) { case 1: { <T>(x)</T> } }",
            "x = () => { <T>(x)</T> };",
            "{ <T>(x)</T> }",
            "{ a, <T>(x)</T> }",
            "function f() { a; <T>(x)</T> }",
            "namespace N { <T>(x)</T> }",
            "try { <T>(x)</T> } finally {}",
            "class C { static { <T>(x)</T> } }",
            "class C { m() { <T>(x)</T> } }",
            "f(a, <T>(x)</T>);",
            "x = [a, <T>(x)</T>];",
            "if (x)\n<T>(y)</T>",
        ] {
            let ks = kinds_of(code, true, true);
            assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
            assert!(!ks.contains(&TokenKind::Lt), "{code:?} must open JSX: {ks:?}");
        }
    }

    #[test]
    fn escaped_type_parameter_name_after_lt_lt() {
        stream(
            "let s: a<<\\u{62}c>(x: T) => T>;",
            true,
            false,
            "let IDENT : IDENT < < IDENT > ( IDENT : IDENT ) => IDENT > ;",
        );
        stream(
            "const r = f<<\\u{62}c>(a: \\u{62}c) => \\u{62}c>(y);",
            true,
            false,
            "const IDENT = IDENT < < IDENT > ( IDENT : IDENT ) => IDENT > ( IDENT ) ;",
        );
    }

    #[test]
    fn escaped_identifiers_are_identifiers_in_every_walk() {
        regex("let \\u{62}c\n/re/.x;", false);
        regex("let \\u{62}c: T\n/re/.x;", true);
        regex("declare const \\u{62}c: Set<T>\n/re/.x;", true);
        regex("var a = 1, \\u{62}c\n/re/.x;", false);
        regex("for (;;) { break \\u{6f}uter\n/re/.test(b); }", false);
        regex("type \\u{41} = T\n/re/.exec(s);", true);
        regex("declare function \\u{66}(): T\n/re/.exec(s);", true);
        division("x = \\u{62}c\n/re/g.exec(s);", false);
        division("\\u{62}c / 2;", false);
        division("x.\\u{62}c / 2;", false);
        gt_run_split("type \\u{41} = Foo<Bar<T>>[];");
        gt_run_split("let \\u{62}c: Foo<Bar<T>>[] = y;");
    }

    #[test]
    fn function_or_class_expression_after_arrow_or_shift_is_a_value() {
        for code in [
            "x = a => function(){} / b / c;",
            "x = () => function y(){} / b / c;",
            "x = async () => function(){} / b / c;",
            "x = a => class {} / b / c;",
            "x = a >> function(){} / 2 / 3;",
            "x = (null >> async function (){} / n);",
            "x = a >>> function(){} / 2;",
            "x = a > function(){} / 2;",
            "x = a => class {}\n/re/.test(s);",
        ] {
            division(code, false);
        }
        division("x = <T>function(){} / 2;", true);
        regex("x = a => {}\n/re/.test(s);", false);
        regex("x = () => {}\n/re/.test(s);", false);
    }

    #[test]
    fn function_expression_return_type_then_body_is_a_value() {
        for code in [
            "x = [function (): T {}\n/ 2 / 3];",
            "x = [function (): x is T {}\n/ 2 / 3];",
            "x = [function (): asserts x is T {}\n/ 2 / 3];",
            "x = [function (): this is T {}\n/ 2 / 3];",
            "x = [function (): A<B> {}\n/ 2 / 3];",
            "x = [function (): { a: T } {}\n/ 2 / 3];",
            "x = [function <T>(a: T): T {}\n/ 2 / 3];",
            "x = [async function (): Promise<T> {}\n/ 2 / 3];",
            "x = [function* (): Generator<T> {}\n/ 2 / 3];",
            "x = [function f(): T {}\n/ 2 / 3];",
            "x = function(): T {}\n/ 2 / 3;",
        ] {
            division(code, true);
        }
        regex("function f(): T {}\n/re/.test(x);", true);
        regex("function f(): { a: T } {}\n/re/.test(x);", true);
        regex("let x: { a: T }\n/re/.test(x);", true);
        let ks = kinds_of("x = [function (): T {}\n< y];", true, true);
        assert!(!ks.contains(&TokenKind::JsxLt), "{ks:?}");
        assert!(ks.contains(&TokenKind::Lt), "{ks:?}");
        let codes = diag_codes_of("x = [function (): T {}\n< y];", true, true);
        assert!(codes.is_empty(), "{codes:?}");
    }

    #[test]
    fn label_colon_inside_a_function_expression_body_is_a_statement() {
        for (code, want) in [
            (
                "x = function (){\nouter: for (;;) { break outer; }\n/x*/;\n};",
                "IDENT = function ( ) { IDENT : for ( ; ; ) { break IDENT ; } REGEXP ; } ;",
            ),
            (
                "x = function (){\nouter: { break outer; }\n/x*/;\n};",
                "IDENT = function ( ) { IDENT : { break IDENT ; } REGEXP ; } ;",
            ),
            (
                "(function (){\nouter: for (;;) { break outer; }\n/x*/;\n});",
                "( function ( ) { IDENT : for ( ; ; ) { break IDENT ; } REGEXP ; } ) ;",
            ),
            (
                "x = () => {\nouter: { break outer; }\n/x*/;\n};",
                "IDENT = ( ) => { IDENT : { break IDENT ; } REGEXP ; } ;",
            ),
            (
                "x = class { m() { outer: { break outer; }\n/x*/; } };",
                "IDENT = class { IDENT ( ) { IDENT : { break IDENT ; } REGEXP ; } } ;",
            ),
            (
                "x = function (){ a: {} /re/.test(s); };",
                "IDENT = function ( ) { IDENT : { } REGEXP . IDENT ( IDENT ) ; } ;",
            ),
        ] {
            stream(code, false, false, want);
        }
        division("x = function(){ return {a: {} / 2} };", false);
        division("x = function(){ a = c ? d : {} / 2 };", false);
        division("x = { a: {} / 2 };", false);
    }

    #[test]
    fn bodiless_function_signature_before_a_line_break_ends_the_statement() {
        for (code, want) in [
            ("declare function y()\n/[/\\]]/.x;", "declare function IDENT ( ) REGEXP . IDENT ;"),
            (
                "declare function y(a: T)\n/re/.x;",
                "declare function IDENT ( IDENT : IDENT ) REGEXP . IDENT ;",
            ),
            (
                "declare function f<T>(p: T)\n/re/.x;",
                "declare function IDENT < IDENT > ( IDENT : IDENT ) REGEXP . IDENT ;",
            ),
            (
                "export declare function y()\n/re/.x;",
                "export declare function IDENT ( ) REGEXP . IDENT ;",
            ),
            (
                "function f(a: T)\n/re/.exec(s);",
                "function IDENT ( IDENT : IDENT ) REGEXP . IDENT ( IDENT ) ;",
            ),
        ] {
            stream(code, true, false, want);
        }
        let ks = kinds_of("declare function y()\n<div/>;", true, true);
        assert!(ks.contains(&TokenKind::JsxLt), "{ks:?}");
        division("x = f()\n/ 2 / 3;", true);
        division("f<T>()\n/ 2 / 3;", true);
        division("new Foo()\n/ 2 / 3;", true);
        division("x = function(){}\n/ 2 / 3;", true);
    }

    #[test]
    fn legacy_octal_literal_ends_before_a_dot() {
        for (code, want) in [
            ("x = 010.5;", "IDENT = NUMBER NUMBER ;"),
            ("x = 010.toString();", "IDENT = NUMBER . IDENT ( ) ;"),
            ("x = 00.5;", "IDENT = NUMBER NUMBER ;"),
            ("x = 07.;", "IDENT = NUMBER . ;"),
            ("x = 08.5;", "IDENT = NUMBER ;"),
            ("x = 019.5;", "IDENT = NUMBER ;"),
            ("x = 09e1;", "IDENT = NUMBER ;"),
            ("x = 0.5;", "IDENT = NUMBER ;"),
            ("x = 0x10.5;", "IDENT = NUMBER NUMBER ;"),
            ("x = 010n;", "IDENT = NUMBER IDENT ;"),
            ("x = 08n;", "IDENT = NUMBER IDENT ;"),
        ] {
            stream(code, false, false, want);
        }
        assert_eq!(spans_of("x = 010.5;", false, false)[2], (4, 7));
        assert_eq!(spans_of("x = 010.5;", false, false)[3], (7, 9));
    }

    #[test]
    fn operand_heads_before_a_value_brace() {
        for (code, want) in [
            (
                "x = <a b={ {k: v} / 2 }/>;",
                "IDENT = JSX_LT IDENT IDENT = { { IDENT : IDENT } / NUMBER } / JSX_TAG_END ;",
            ),
            (
                "x = <a>{ {k: v} / 2 }</a>;",
                "IDENT = JSX_LT IDENT > { { IDENT : IDENT } / NUMBER } JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "x = <a>{c}{ {k: v} / 2 }</a>;",
                "IDENT = JSX_LT IDENT > { IDENT } { { IDENT : IDENT } / NUMBER } JSX_LT / IDENT JSX_TAG_END ;",
            ),
            (
                "x = {}\n{ {k: v}\n/re/.test(s) }",
                "IDENT = { } { { IDENT : IDENT } REGEXP . IDENT ( IDENT ) }",
            ),
            ("{ {k: v}\n/re/.test(s) }", "{ { IDENT : IDENT } REGEXP . IDENT ( IDENT ) }"),
        ] {
            stream(code, false, true, want);
        }
        division("f(...{a: 1} / 2);", false);
        division("x = [...function(){} / 2];", false);
        division("f(...class {} / 2);", false);
        division("for (const k of {a: 1} / 2) ;", false);
        division("for (x of {} / 2) ;", false);
        division("x = a, void {} / 2;", false);
        division("f(a, void {} / 2);", false);
    }

    #[test]
    fn type_context_before_a_declaration_keyword() {
        gt_run_split("x = () => {}\n<Map<P>>baz;");
        gt_run_split("x = async () => {}\n<Map<P>>baz;");
        regex("declare function f(): Foo<T>\nclass C {}\n/re/.test(s);", true);
        regex("let x: Foo<T>\nfunction f() {}\n/re/.test(s);", true);
        division("x = <T>\nfunction(){} / 2;", true);
        stream(
            "class C<T> extends B implements I, void {}\n/=/.test(s);",
            true,
            false,
            "class IDENT < IDENT > extends IDENT implements IDENT , void { } REGEXP . IDENT ( IDENT ) ;",
        );
    }

    #[test]
    fn keyword_named_generic_members() {
        for (code, want) in [
            (
                "class C { delete<const U>(x: U): U {} }",
                "class IDENT { delete < const IDENT > ( IDENT : IDENT ) : IDENT { } }",
            ),
            (
                "class C { m() {} return<T>(x: T) {} }",
                "class IDENT { IDENT ( ) { } return < IDENT > ( IDENT : IDENT ) { } }",
            ),
            ("x = { typeof<T>(x: T) {} };", "IDENT = { typeof < IDENT > ( IDENT : IDENT ) { } } ;"),
            (
                "type X = { void<T>(x: T): U };",
                "type IDENT = { void < IDENT > ( IDENT : IDENT ) : IDENT } ;",
            ),
        ] {
            stream(code, true, true, want);
        }
        for code in [
            "{ delete <T>(x)</T> }",
            "function f() { a; typeof <T>(x)</T> }",
            "if (a) { void <T>(x)</T> }",
        ] {
            let ks = kinds_of(code, true, true);
            assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        }
    }

    #[test]
    fn template_literal_type_inside_a_lt_lt_list() {
        for code in [
            "(a?.c)<<n>() => T<F<G<\"lit\">>>, readonly R<0x1f, Foo>[], `${Foo<Map<R>>}`,>(y);",
            "let e: Map<<T>(x: T) => `${T}`, U>;",
        ] {
            let ks = kinds_of(code, true, false);
            assert!(!ks.contains(&TokenKind::LShift), "{code:?}: {ks:?}");
        }
    }

    #[test]
    fn unicode_whitespace_before_a_glued_number() {
        division("{\u{2028}5. // c\n/\u{a0}b / c; }", false);
        division("x = \u{a0}5.\n/ 2 / 3;", false);
        division("x = \u{2028}5\n/ 2 / 3;", false);
    }

    #[test]
    fn function_head_walks_cross_every_return_type_shape() {
        division(
            "const P = function a(x: A): abstract new () => abstract new () => keyof any {}\n/ 2 / 3;",
            true,
        );
        division("x = [function (): _ is Record<Promise<V>> {}\n/ 2 / 3];", true);
        regex(
            "function y(): x is abstract new () => typeof import('m') extends boolean ? K<A, B<unique symbol>> : void[] {}\n/[a-z]+/v.x;",
            true,
        );
        regex("async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;", true);
        regex("function f(): { a: T } {}\n/re/.test(x);", true);
        regex(
            "async function obj(): K<U, $JSX<Array<U>>> {\nclass C implements V, E<$JSX<-1>> {\n\n}\n/[\\]]/.test(s);\n}",
            true,
        );
        let ks = kinds_of(
            "x = [async function $(): _ is Record<Promise<V>>{} < function Bar(arr?: any): K<this>{}];",
            true,
            true,
        );
        assert!(ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::JsxLt), "{ks:?}");
    }

    #[test]
    fn gt_run_heads_after_asi_and_this() {
        gt_run_split("let cb\n<Promise<V<Promise<-1>>>>cb;");
        gt_run_split("declare const Foo: `x${`y${Record<A<B>>}`}`\n<Array<Thenable<T>>>a;");
        gt_run_split(
            "(f() satisfies Thenable<`lit`> extends 0x1f ? keyof (X) : U<Set<Record<V<X<Map<X<K<1.5>>, Foo>>>>, Foo>> > el);",
        );
        gt_run_fused("x = y satisfies this<$JSX<-1>>\n[class <K, V> extends (a) {}];");
        gt_run_fused("x = a[0].b<c<d>>[0];");
        let ks = kinds_of("x = [f].#p <\n{ _ }| 's'>= b;", true, false);
        assert!(ks.contains(&TokenKind::Ge), "{ks:?}");
        regex("type Foo = | {} | bigint\n<import('m').baz<Bar<T<P>>>>/'/.x;", true);
        regex("let x: T\n<U>/re/.test(s);", true);
        division("let x = y\n<T>/re/.source;", true);
    }

    #[test]
    fn brace_after_a_generic_return_type_is_a_body() {
        for (code, ts, jsx) in [
            (
                "async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;",
                true,
                false,
            ),
            ("async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;", true, true),
            ("function f(): () => Array<symbol> {}\n/re/.x;", true, false),
            ("function f(): A | B<C> {}\n/re/.x;", true, false),
            (
                "x = function* (): P<Q<R>> {}\nwhile (a) { async (p): B => 1\n}\n/re/.x;",
                true,
                false,
            ),
        ] {
            let ks = kinds_of(code, ts, jsx);
            assert_eq!(
                {
                    let mut buf = code.as_bytes().to_vec();
                    let n = buf.len();
                    buf.resize(n + PAD, 0);
                    let mut opts = default_options();
                    opts.ts = ts;
                    opts.jsx = jsx;
                    let mut lx = Lexer::new();
                    let count = lx.lex(&buf, n, opts);
                    let kinds = lx.kinds()[..count].to_vec();
                    (0..count)
                        .filter(|&i| {
                            !kinds[i].is_trivia() && buf[lx.spans[i].start as usize] == b'/'
                        })
                        .map(|i| kinds[i])
                        .next()
                },
                Some(TokenKind::RegExp),
                "{code:?}: {ks:?}"
            );
        }
        division("x = f < T > {} / 2;", true);
        division("x = c ? function* (...a): Pick<E<F<G>>>[][] {} / 2 : null;", true);
    }

    #[test]
    fn relational_heads_before_a_balanced_run() {
        gt_run_split("x = { a: 1, ...<T<U<V>>>[baz] };");
        gt_run_split("f(...<Thenable<K>>[], false);");
        gt_run_split("x = c ? function* (...a): Pick<E<F<G>>>[][] {} : null;");
        gt_run_fused("x = (y)--<z ? 1 : $<A, B<C>>().p>> w;");
        gt_run_fused("x = y satisfies Exclude<never>\n<typeof obj | Exclude<K<'lit'>>>(baz);");
        gt_run_fused("x = y as Pick | this<U<V<W>>>\n/'/.x;");
        gt_run_fused("const x = Foo >>> (n)`\n`\n<U<Bar>>foo;");
        gt_run_split("f(this<A<B>>(x));");
        gt_run_split("return this<A<B>>(x);");
        gt_run_split("interface x extends this<Thenable<Record<Record>>>, U {}");
        gt_run_split("class C<V, T> implements this<Array<undefined>>, T {}");
        gt_run_split("class C implements this<Map<\"lit\">>, this {}");
        regex(
            "let c: T = y satisfies A<B<C>> extends infer U ? U : import('m').a<(V<never>)[Set]>\nnamespace Foo { class X {} }\n/foo/.exec(s);",
            true,
        );
        gt_run_fused("x = y satisfies A\n<Set<K>, Partial<U>>ab;");
        gt_run_split("x = y as A<B<C>> as D;");
        gt_run_fused("x = b()!\n<Array<Set<unknown>>>Foo;");
        gt_run_fused("x = c! < 0.5>>> a;");
        division("export default { a: 1 } / obj(x);", false);
        regex("x = y as Pick | this<U<V<W>>>\n/'/.x;", true);
        regex("x = y satisfies this<U<V<W>>>\n/'/.x;", true);
        division("x = this<A<B>> / 2;", true);
        regex("x ? a : [b][c]()\n{}\n/y/.exec(s);", false);
        regex("f(class { accessor x = y })\n{ }\n/</.test(s);", false);
        let ks = kinds_of("x = a > b<c<d>>>>(e);", true, false);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 3, "{ks:?}");
        assert!(ks.contains(&TokenKind::RShift), "{ks:?}");
    }

    #[test]
    fn constructor_type_parameter_annotations_are_type_regions() {
        let silent = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(codes.is_empty(), "{code:?}: {codes:?}");
            let ks = kinds_of(code, true, true);
            assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
        };
        silent("declare function b(obj: new (bar: <T>() => U<T>, arr: T<U>) => unknown): void;");
        silent("declare function b(obj: abstract new (bar: <T>() => U) => unknown): void;");
        silent("let x: new (cb: <T>(x: T) => T) => U;");
        silent("(a, b = c): <T>(x: T) => U => 1;");
        silent("type X = { a: T; [k: string]: <a>(s) => T }");
        silent("class C extends obj<abstract new () => (<T>(a) => true)> {}");
        silent("x = f((a): <T>(x: T) => U => 1);");
        silent("let x: Foo<new ({ a, b }: <s>({ a }: {}, s) => string[]) => U>;");
        silent("let x: [c?: <arr>() => new () => false | Thenable, d];");
        silent("class C { set [fn](cb: <Props>() => T) {} }");
        silent("for (const k in (b: <obj>() => Record) => { bar; }) {}");
    }

    #[test]
    fn optional_markers_are_not_ternaries_for_the_jsx_diagnostic() {
        let silent = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(codes.is_empty(), "{code:?}: {codes:?}");
            let ks = kinds_of(code, true, true);
            assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
        };
        let diagnosed = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(
                codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
                "{code:?} must be diagnosed: {codes:?}"
            );
        };
        for code in [
            "let x: A<(a?: <T>() => U) => V>;",
            "let x: A<(arr?: <cb>() => keyof bigint) => 'lit'>;",
            "let f: (a?: <T>() => U) => V;",
            "function f(a?: <T>() => U) {}",
            "let x: readonly [b?: <b>(cb, [a, b]: | Record<K>) => U];",
            "let x: [b?: <b>(cb) => U, c?: <d>() => V];",
            "class C { b?(): <N>(n: U) => V }",
            "class C { static b?(): <N>(n: U) => V }",
            "class C { [k]?(): <N>(n: U) => V }",
            "class C { 'k'?(): <N>(n: U) => V }",
            "class C { #p?(): <N>(n: U) => V }",
            "interface I { b?(): <N>(n: this[][], arr: U) => <fn>(s: T) => U }",
            "type X = { b?(): <N>(n: U) => V }",
            "let o: { b?(): <N>(n: U) => V };",
            "class C { a = 1; b?(): <N>(n: U) => V }",
            "class C { m?: <T>(x: T) => T }",
        ] {
            silent(code);
        }
        for code in [
            "x = c ? (a) : <T>(x: T) => x;",
            "{ c ? (a) : <T>(x: T) => x }",
            "f(c ? (a) : <T>(x: T) => x);",
            "x = c ? a : <T>(x: T) => x;",
        ] {
            diagnosed(code);
        }
    }

    #[test]
    fn member_and_parameter_annotations_are_type_regions_for_the_jsx_diagnostic() {
        let silent = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(codes.is_empty(), "{code:?}: {codes:?}");
            let ks = kinds_of(code, true, true);
            assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
        };
        let diagnosed = |code: &str| {
            let codes = diag_codes_of(code, true, true);
            assert!(
                codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
                "{code:?} must be diagnosed: {codes:?}"
            );
        };
        for code in [
            "class C { a: T;\n  b: <y>() => U }",
            "interface I { a: T;\n  readonly b: <y>() => U }",
            "class C { a = 1; b!: <y>() => U }",
            "class C { #p!: <arr>(...x: T[]) => U }",
            "class C extends (e) { #p!: <arr>() => U }",
            "x = (class extends (e) { public b!: <bar>() => U });",
            "class C { @dec() b: <bar>() => U }",
            "class C { @dec @dec() override cb: <bar>() => K }",
            "class C { m() {} public s: <N>() => U }",
            "class C { static { x; } b: <y>() => U }",
            "class C { static [k]: <Foo>(bar) => P }",
            "class C { static [y.#p`{`]: <Foo>(bar) => P }",
            "class C { private static?: 'lit'; public out!: <arr>(s) => U }",
            "interface I { 'x': T; 'y': <y>() => U }",
            "interface I { c?(): T; b: <obj>(baz?: T) => K }",
            "type X = { a: T; [k: string]: T; b: <y>() => U }",
            "let x: new (a) => (s: T) => { _?(): <P>(b: T) => U };",
            "abstract class Foo<T = { s: P<false>, bar?(): {}, n?(): <_>() => this }> extends fn<A> {}",
            "function f<T = { n?(): <_>() => U }>() {}",
            "let x: Map<K, { n?(): <_>() => U }>;",
            "type X<T = { n?(): <_>() => U }> = T;",
            "x = function* (el, [b, c]: <T>(y: T) => U) {};",
            "x = function (el, [b, c]: <T>(y: T) => U) {};",
            "f(function* (x?: <T>() => U) {});",
            "x = (s?: <bar>(cb: T) => U) => 1;",
            "x = ({ a }, arr: <T>(obj: T) => U) => 1;",
            "f($ = ({ a }, arr: <T>(obj: T) => U) => 1);",
            "class C { m($ = ({ a }, arr: <T>(obj: T) => U) => 1) {} }",
            "[a, b] = ($, x, c: <T>(s: T) => U) => 1;",
            "(_?: <Foo>(...c: T[]) => U, ...n) => 1;",
            "x = (a: <T>() => U): R => 1;",
            "let x = async function <V, K = { 'obj': <x>(a: T) => U }>() {};",
            "class C { [a << b]() {}
 in<K>(el: T) {} }",
            "class C { [(b[class <U extends P<Q<R>>> {}] << (x)] ??= y)]() {}
 @dec() in<K>(el: T) {} }",
            "class C { x = a << b;
 m<K>(el: T) {} }",
            "let e: Map<<T>(x: T) => T, { m<K>(el: T): U }>;",
            "type X = Set<A, (B), (<arr>() => C)[]>;",
            "let arr: (c: keyof T<[Map], (<P>() => Q<M>), [bar?: K]>) => U;",
            "type X = { foo?: P; ([a, b]: T, obj: U, c: <B>([a, b]: S) => R): V }",
            "type X = { (c: <B>(a: S) => R): V }",
            "interface I { m(): T; (c: <B>(a: S) => R): V }",
            "type X = { a: T }\n{ (c: <B>(a: S) => R): V }",
            "type T = { new (b: X, s: <el>() => U): V }",
            "function* obj(a: T, b: <x>(fn?: U) => V) {}",
            "class C { foo(a = $, { s }, el: <x>(_: T) => U) {} }",
            "class C { #p([b, $]: <b>({ a }: T) => U) {} }",
            "class C { constructor(private a: T, obj: <c>() => U) {} }",
            "async function* el(fn, n: <N>({ a, b }: T, c: U) => V) {}",
            "let x: { obj([a, b]: <n>(o: T) => U): V };",
            "class C { @dec() *accessor(b: T = x++, s: <a>() => boolean): asserts x {} }",
            "x = class { constructor(p: any = f(), bar, x?: <s>(this: string) => P) {} };",
            "function f(a: T, ...b: <Foo>(c: U) => V[]) {}",
            "f(a, (s: <el>() => U) => 1);",
            "using a = f, o: <obj>() => U = g;",
            "let a = f, o: <obj>() => U;",
        ] {
            silent(code);
        }
        for code in [
            "{ a: 1; b: <T>(x: T) => x }",
            "function f() { x = 1; y: <T>(x: T) => x }",
            "L: <T>(x: T) => x;",
            "x = (a, b);\ny: <T>(x: T) => x;",
            "f(a, <T>(x: T) => x);",
            "new C(a, <T>(x: T) => x);",
            "f(a, b ? c : <T>(x: T) => x);",
        ] {
            diagnosed(code);
        }
    }

    #[test]
    fn ts_postfix_bang_before_a_block_brace() {
        regex("[typeof ab]!\n{\n}\n/x/.test(s);", true);
        regex("x!\n{}\n/x/.test(s);", true);
        regex("f(x)!\n{}\n/x/.test(s);", true);
        division("x = !{} / 2;", true);
        division("x = a && !{}.b / 2;", true);
        division("x!\n/re/g.exec(s);", true);
    }

    #[test]
    fn adjacent_type_atoms_end_an_annotation() {
        division("let x: Foo<T>\nf(y)\n/re/.test(s);", true);
        division("let x: T\nf(y)\n/re/.test(s);", true);
        division("let x: T[]\nf(y)\n/re/.test(s);", true);
        division("let x: `lit`\nf(y)\n/re/.test(s);", true);
        regex("let x: typeof import('m').default\n/re/.test(s);", true);
        regex("let x: asserts y is T\n/re/.test(s);", true);
        regex("let x: abstract new () => T\n/re/.test(s);", true);
        regex("let x: readonly unique symbol[]\n/re/.test(s);", true);
        regex("let x: A extends infer U extends B ? C : D\n/re/.test(s);", true);
        regex("let x: `a${T}b${U}c`\n/re/.test(s);", true);
        regex("let x: { [K in keyof T as `x${K}`]: T[K] }\n/re/.test(s);", true);
        regex("let x: <T>(a: T) => T\n/re/.test(s);", true);
        regex("let x: new <T>(a: T) => T\n/re/.test(s);", true);
        regex("let x: <T>(a: T) => T | undefined\n/re/.test(s);", true);
        regex("let a: A, b: B, c: <T>(a: T) => T\n/re/.test(s);", true);
        regex("declare function f<T>(a: T): <U>(b: U) => T\n/re/.test(s);", true);
        regex("type A<T> = <T>(a: T) => T\n/re/.test(s);", true);
        for code in [
            "declare let x: <T>(a: T) => T\n<div />;",
            "let x: <T>(a: T) => T | undefined\n<div />;",
            "let a: A, b: B, c: <T>(a: T) => T\n<div />;",
            "type A = <T>(a: T) => T\n<div />;",
            "declare function f(): <T>(a: T) => T\n<div />;",
        ] {
            let ks = kinds_of(code, true, true);
            assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        }
        let ks = kinds_of("let P: Array<bigint, this>\n_(x)\n<(P().foo);", true, true);
        assert!(ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::JsxLt), "{ks:?}");
        let codes = diag_codes_of("let P: Array<bigint, this>\n_(x)\n<(P().foo);", true, true);
        assert!(codes.is_empty(), "{codes:?}");
    }

    #[test]
    fn replay_hops_return_types_and_type_parameters() {
        regex("x = async (): T => { await /re/; };", true);
        regex("x = async (): typeof cb => { await /re/; };", true);
        regex("var $: <baz>() => 1n | T = async (): typeof cb => { await /<div>/ };", true);
        division("x = (): T => { var await = 1; return await /2/g; };", true);
        regex("x = async function f(): T { await /re/; };", true);
        regex("x = async function (): Promise<T> { await /re/; };", true);
        regex("x = async (): Promise<T> => { await /re/; };", true);
        regex("x = function* <T>(): Generator<T> { yield /re/; };", true);
        regex("class C { async m(): Promise<T> { await /re/; } }", true);
        regex("switch (async function f(): typeof import('m') { await /}/; }) {}", true);
        division("x = function (): T { var await = 1; return await /2/g; };", true);
        division("x = (): T => { var await = 1; return await /2/g; };", true);
    }
}
