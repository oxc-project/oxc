//! Is a `<` in a `.tsx` file TypeScript syntax, rather than the start of a JSX element?
//!
//! In `.tsx` files `<T>` can open either a JSX element or a type parameter list,
//! as in `let f: <T>(x: T) => T` or `function <T>(x: T) {}`.
//! `carve` settles most cases by looking ahead.
//! When that isn't enough, it asks these questions, which look back at what comes before the `<`:
//!
//! - [`ts_type_region_open`]: Does a type start here, as in after the `:` in `let f: <T>(x: T) => T`?
//! - [`type_parameter_list_head`]: Does this `<` open a type parameter list,
//!   as in `function <T>` or a method `m<T>()`?
//! - [`jsx_site_is_expression`]: Is this `<` in expression position?
//!   There TypeScript reads `<T>(x: T) => x` as a JSX element, which never closes.
//!   So this decides whether that error is reported.

use crate::{
    opmap::OP_KIND_BASE,
    tables::{Tables, is_ws},
    token::tk,
};

use super::super::super::bitmap::bm_next1;

use super::super::common::{
    AngleMatch, angle_match_back, annotation_colon_is_declaration, bm_prev_sig,
    brace_opens_object_literal, class_like_walk, conditional_type_question,
    extends_precedes_question, ident_is, kind_at, lt_in_range, match_delim_back, prop_name,
    type_alias_head, type_annotation_asi, word_is_any,
};

use super::{
    bytes::{ENCLOSING_SCAN_CAP, Encl, GT_SCAN_CAP, arrow_after_paren_group, enclosing_opener},
    context::{
        BLOCK_OPEN_WORDS, CTX_HOPS, Ctx, MEMBER_OPEN_WORDS, arrow_context, brace_is_type_literal,
        member_start_before, ternary_colon,
    },
};

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

pub unsafe fn ts_type_region_open(
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
    if q >= 0 && kind_at(kind, q as usize) == tk!(Ident) && !prop_name(src, q as usize) {
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
                        && kind_at(kind, p as usize) == tk!(Ident)
                        && !prop_name(src, p as usize)
                        && ident_is(src, p as usize, b"new")
                    {
                        p = bm_prev_sig(st, kind, p as usize);
                        if p >= 0
                            && kind_at(kind, p as usize) == tk!(Ident)
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
                                && kind_at(kind, f as usize) == tk!(Ident)
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
        if !matches!(*kind.add(w), tk!(Ident) | tk!(IdentEscaped)) {
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

pub unsafe fn type_parameter_list_head(
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
                    && kind_at(kind, p as usize) == tk!(Ident)
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
    if k == tk!(Ident) {
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
    matches!(k, tk!(Number) | tk!(BigInt) | tk!(String) | tk!(TemplateNoSub) | tk!(TemplateTail))
        && lt_in_range(src, bm_next1(st, w + 1, n), lt)
        && enclosing_brace_is_type_literal(t, src, st, opch, kind, n, w)
}

pub unsafe fn jsx_site_is_expression(
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
        if k == tk!(Ident) || k == tk!(IdentEscaped) {
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
                        if bk == tk!(Ident) || bk == tk!(IdentEscaped) {
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
                if bk == tk!(Ident) || bk == tk!(IdentEscaped) {
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
                            && kind_at(kind, f as usize) == tk!(Ident)
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
        && kind_at(kind, h as usize) == tk!(Ident)
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
    while i < n && matches!(*kind.add(i), tk!(Whitespace) | tk!(LineComment) | tk!(BlockComment)) {
        i = bm_next1(st, i + 1, n);
    }
    if i >= n || kind_at(kind, i) != tk!(Ident) {
        return false;
    }
    let mut j = bm_next1(st, i + 1, n);
    while j < n && matches!(*kind.add(j), tk!(Whitespace) | tk!(LineComment) | tk!(BlockComment)) {
        j = bm_next1(st, j + 1, n);
    }
    j < n && *kind.add(j) >= OP_KIND_BASE && *src.add(j) == b':'
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
