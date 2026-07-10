use crate::opmap::{OP_KIND_BASE, OP_QDOT};
use crate::tables::{Tables, is_digit, is_glue_join, is_word, is_ws};

use super::bitmap::{bm_next0, bm_next1, bm_prev1};
use super::find::scan_number;
use super::{
    BCOM, HASHBANG, IDENT, IDENT_ESC, JEND, JSX_LT, LCOM, NUM, PRIV_IDENT, PRIV_IDENT_ESC, REGEX,
    STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS,
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
unsafe fn prev_regex_sim(t: &Tables, src: *const u8, n: usize, a: usize, p: usize) -> bool {
    let mut lastk: i32 = -1;
    let mut ls = 0usize;
    let mut le = 0usize;
    let mut pos = a;
    while pos < p {
        let c = *src.add(pos);
        if c == b'.' && pos + 1 < n && is_digit(*src.add(pos + 1)) {
            ls = pos;
            pos = scan_number(src, n, pos);
            le = pos;
            lastk = NUM as i32;
            continue;
        }
        if is_digit(c) {
            ls = pos;
            pos = scan_number(src, n, pos);
            le = pos;
            lastk = NUM as i32;
            continue;
        }
        if is_word(c) {
            ls = pos;
            while pos < n && is_word(*src.add(pos)) {
                pos += 1;
            }
            le = pos;
            lastk = IDENT as i32;
            continue;
        }
        if c == b'.' || c == b'+' || c == b'-' || c == b'?' {
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
            ls = pos;
            le = pos + opl;
            lastk = 1000;
            pos += opl;
            continue;
        }
        break;
    }
    if lastk == -1 {
        return true;
    }
    if lastk == NUM as i32 {
        return false;
    }
    if lastk == IDENT as i32 {
        if ls > 0 && *src.add(ls - 1) == b'.' && (ls < 2 || *src.add(ls - 2) != b'.') {
            return false;
        }
        return t.is_regex_keyword(src.add(ls), le - ls);
    }
    !(*src.add(le - 1) == b')' || *src.add(le - 1) == b']')
}
/// Distance cap (in token starts) for the backward delimiter matches below;
/// past it we fall back to the safe legacy "`}` means regex" answer. Only
/// pathological input gets near it.
const BRACE_MATCH_CAP: u32 = 1024;
/// Previous significant token start before `pos` (skipping trivia), or -1 at
/// start of input.
#[inline]
unsafe fn bm_prev_sig(st: *const u64, kind: *const u8, pos: usize) -> i64 {
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
/// Is the token at `pos` in operand (expression-only) position? Strict
/// whitelist; anything else returns false. Deliberately not `prev_is_regex`:
/// a regex may follow `;`/`:`/`else`, but a `{` there is a block, so reusing
/// it would be unsound. Complement of acorn's `braceIsBlock`.
#[inline]
unsafe fn operand_position(src: *const u8, st: *const u64, kind: *const u8, pos: usize) -> bool {
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false; // start of input => statement position
    }
    let p = q as usize;
    if *kind.add(p) < OP_KIND_BASE {
        // Of the value kinds, only a template substitution (`${ {..} }`)
        // puts what follows in operand position.
        let k = *kind.add(p);
        return k == TMPL_HEAD || k == TMPL_MIDDLE;
    }
    is_operand_punct(*src.add(p))
}
/// Does the identifier at `pos` equal exactly `kw`? The following-byte check
/// rejects longer identifiers (the source pad makes it safe at EOF).
#[inline]
unsafe fn ident_is(src: *const u8, pos: usize, kw: &[u8]) -> bool {
    let mut i = 0;
    while i < kw.len() {
        if *src.add(pos + i) != kw[i] {
            return false;
        }
        i += 1;
    }
    !is_word(*src.add(pos + kw.len()))
}
/// Match the close punctuator at `from` back to its opener, counting only
/// punctuator delimiters — template-closing `}`s and cleared literal
/// interiors are invisible. None if unbalanced or past the cap.
#[inline]
unsafe fn match_delim_back(
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
/// Does the `{` at `brace` open a value (object literal or function-
/// expression body) rather than a block? Class expressions and TS `as {..}`
/// are not handled; they fall back to regex, same as legacy.
#[inline]
unsafe fn brace_opens_value(src: *const u8, st: *const u64, kind: *const u8, brace: usize) -> bool {
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false; // start of input => block
    }
    let p = q as usize;
    if *kind.add(p) < OP_KIND_BASE {
        let k = *kind.add(p);
        return k == TMPL_HEAD || k == TMPL_MIDDLE; // `${ {..} }`
    }
    let ch = *src.add(p);
    if is_operand_punct(ch) {
        return true; // object literal in operand position
    }
    // `{` preceded by `)` may be a function-expression body:
    // `function (..) { } /`. Match the `)` to its `(`; if an anonymous
    // `function` sits before the `(` in operand position, the `{}` is a
    // value and the `/` is division.
    if ch == b')' {
        if let Some(lp) = match_delim_back(src, st, kind, p, b'(', b')') {
            let w = bm_prev_sig(st, kind, lp);
            if w >= 0 {
                let wp = w as usize;
                if *kind.add(wp) < OP_KIND_BASE
                    && ident_is(src, wp, b"function")
                    && operand_position(src, st, kind, wp)
                {
                    return true;
                }
            }
        }
    }
    false
}
/// A `/` right after `}`: regex if the `}` closed a block, division if it
/// closed a value. Cold; any uncertainty falls back to regex (the legacy
/// answer), which can never introduce a stream-swallowing regex.
#[inline]
unsafe fn brace_close_is_regex(src: *const u8, st: *const u64, kind: *const u8, qi: usize) -> bool {
    match match_delim_back(src, st, kind, qi, b'{', b'}') {
        Some(brace) => !brace_opens_value(src, st, kind, brace),
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
            return prev_regex_sim(t, src, n, glue_anchor(src, st, qi), p);
        }
        if k == IDENT {
            if qi > 0 && *src.add(qi - 1) == b'.' && (qi < 2 || *src.add(qi - 2) != b'.') {
                return false;
            }
            let e = bm_next1(st, qi + 1, n);
            return t.is_regex_keyword(src.add(qi), e - qi);
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
                return prev_regex_sim(t, src, n, glue_anchor(src, st, qi), p);
            }
            // `}` closed either a block (regex follows) or a value (division).
            if ch == b'}' {
                return brace_close_is_regex(src, st, kind, qi);
            }
            return !(ch == b')' || ch == b']');
        }
        return true;
    }
    true
}
