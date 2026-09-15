use crate::{
    opmap::OP_KIND_BASE,
    tables::{Tables, is_digit, is_id_start, is_ws},
};

use super::super::{
    BCOM, BIGINT, IDENT, IDENT_ESC, LCOM, NUM, PRIV_IDENT, PRIV_IDENT_ESC, STR, TMPL_HEAD,
    TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS,
    bitmap::{bm_get, bm_next1},
    find::{find_line_terminator, unicode_ws_len},
    scan::scan_block_comment,
};

use super::common::{
    AngleMatch, KW_AWAIT, KW_BREAK, KW_CASE, KW_CATCH, KW_CLASS, KW_CONST, KW_CONTINUE,
    KW_DEBUGGER, KW_DEFAULT, KW_DELETE, KW_DO, KW_ELSE, KW_ENUM, KW_EXPORT, KW_EXTENDS, KW_FINALLY,
    KW_FOR, KW_FUNCTION, KW_IF, KW_IN, KW_INSTANCEOF, KW_IS, KW_LET, KW_RETURN, KW_SUPER,
    KW_SWITCH, KW_THIS, KW_THROW, KW_TRY, KW_VAR, KW_WHILE, KW_WITH, KW_YIELD, LT_OPERAND_WORDS,
    angle_match_back, annotation_colon_is_declaration, as_gated_type_ref, as_type_operand,
    bang_is_postfix, bm_prev_sig, brace_opens_object_literal, brace_opens_value, chain_head,
    class_like_walk, conditional_type_question, declarator_without_init, extends_precedes_question,
    ident_is, incdec_is_postfix, is_binder_keyword, kind_at, lt_in_range, match_delim_back,
    prop_name, type_alias_head, type_annotation_asi, type_prefix_kind, word_is_any, word_len,
};

#[cfg(test)]
mod tests;

const CTX_HOPS: u32 = 32;

const ENCLOSING_SCAN_CAP: usize = 1 << 16;

/// Cap for scanning `>` runs: matching `<` is within 100 bytes
/// Hitting the cap returns `None` (fuse), so it can only widen the residual, never split a shift.
const GT_SCAN_CAP: usize = 1 << 16;

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

const GENERATOR_STAR_WORDS: &[&[u8]] =
    &[b"static", b"async", b"public", b"private", b"protected", b"override", b"abstract"];

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

const FOLLOW_SPLIT_WORDS: &[&[u8]] =
    &[b"in", b"instanceof", b"as", b"satisfies", b"extends", b"implements"];

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Follow {
    Split,
    Fuse,
    Ctx,
}

enum Encl {
    Open(usize, u8),
    Top,
    Capped,
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
pub unsafe fn gt_run_split(
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

/// `coalesce` entry for a `<<` run: true when the two `<` must stay separate
/// tokens. Cold — `<<` is shift-left everywhere except this one shape.
#[inline(never)]
pub unsafe fn lt_run_split(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
) -> bool {
    lt_run_opens_type_args(src, st, opch, kind, n, p)
}

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

unsafe fn computed_key_at_member_start(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lb: usize,
) -> bool {
    member_start_before(src, st, kind, lb, true)
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

unsafe fn decorator_before(src: *const u8, st: *const u64, kind: *const u8, w: usize) -> bool {
    let Some(h) = chain_head(src, st, kind, w) else {
        return false;
    };
    let at = bm_prev_sig(st, kind, h);
    at >= 0 && *src.add(at as usize) == b'@'
}

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
            let wl = unicode_ws_len(src, i);
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

#[inline]
unsafe fn skip_ws_fwd(src: *const u8, mut i: usize, lim: usize) -> usize {
    while i < lim && is_ws(*src.add(i)) {
        i += 1;
    }
    i
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
