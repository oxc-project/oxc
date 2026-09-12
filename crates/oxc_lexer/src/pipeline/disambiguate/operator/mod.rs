use crate::{
    opmap::{OP_KIND_BASE, OP_QDOT},
    tables::{Tables, is_digit, is_glue_join, is_word, is_ws},
};

use super::super::{
    BCOM, BIGINT, HASHBANG, IDENT, IDENT_ESC, JEND, JSX_LT, LCOM, NUM, PRIV_IDENT, PRIV_IDENT_ESC,
    REGEX, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS,
    bitmap::{bm_next0, bm_next1, bm_prev1},
    scan::scan_number,
};

use super::{
    AngleMatch, LT_OPERAND_WORDS, angle_match_back, as_gated_type_ref, as_type_operand,
    bm_prev_sig, brace_opens_value, declarator_without_init, ident_is, kind_at, lt_in_range,
    match_delim_back, of_is_forof_keyword, prop_name, replay::replay_is_keyword,
    signature_return_type, trivia_at, type_annotation_asi, word_is_any,
};

#[cfg(test)]
mod tests;

enum RunEnd {
    Seg(usize, usize, bool),
    Blank(bool),
}

/// Check if `p` is a position where an operator cannot go.
///
/// Operator position is directly after a complete value, e.g. after `a`, `f(x)`, or `a[0]`.
/// Anywhere else, an operator can't go, and something must start instead:
///
/// - An expression e.g. `x = /re/`, `x = <Foo />`.
/// - A statement e.g. `if (c) /re/.test(s)`, `if (c) <Foo />`.
/// - In TypeScript, a type e.g. `let f: <T>(x: T) => T`.
///
/// Returns `true` if `p` is not in operator position, `false` if it is.
///
/// Callers use this to decide:
///
/// - `/` starts a regex if `true`, or is a division operator (`/` or `/=`) if `false`.
/// - `<` may start a JSX element if `true`, or is a less-than operator if `false`.
///   In TS, a `<` in operator position can also open type arguments e.g. `f<T>()`.
///
/// This is a looser test than [`operand_position`], which checks whether *only*
/// an expression can start at a position.
///
/// The two differ where a statement can start, e.g. after `;` or `else`, or at the start of the file.
/// A `/` there starts a regex, so this function returns `true`.
/// But a `{` there opens a block, not an object literal, so `operand_position` returns `false`.
///
/// [`operand_position`]: super::operand_position
pub unsafe fn not_operator_position(
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
                        return replay_is_keyword(t, src, st, kind, n, qi, ts, false);
                    }
                    if ident_is(src, ws, b"await") {
                        return replay_is_keyword(t, src, st, kind, n, qi, ts, true);
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
